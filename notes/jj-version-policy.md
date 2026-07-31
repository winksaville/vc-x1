# jj version coupling policy

vc-x1 links jj-lib and writes into repos the user's own `jj`
binary also writes. The two versions are therefore coupled. This
file is the rule that governs the coupling: what is compared,
when, what happens on a mismatch, and what the check does and
does not buy.

Supersedes the conclusion of
[Design risk: op-store coexistence](refactor-20260716.md#design-risk-op-store-coexistence),
which is kept as the investigation that produced it. The operands
ship at `0.78.0-2`; the gate itself at `0.78.0-5`.

## The rule

If our jj-lib version and the user's `jj` version are not equal,
stop. Checked at startup, before anything opens a repo, with an
explicit per-invocation override flag.

## The operands

- **Ours**: the jj-lib version resolved from `Cargo.lock` and
  compiled in by `build.rs` as `JJ_LIB_VERSION`. jj-lib exports
  no version constant and no accessor for one, so the lock is the
  only statement of what we link. Compile-time by nature rather
  than by compromise: the linked library is the thing that writes
  ops.
- **Theirs**: the triple parsed from `jj -V`, which prints
  `jj 0.43.0-<40-hex>`. Compare the triple by prefix match or
  regex, never whole-string equality.

This rests on jj-cli and jj-lib releasing in lockstep. Verify
against the jj-cli manifest on each bump: equality becomes the
wrong test if they ever diverge.

## Equality, because compatibility is not computable

We cannot ask whether two versions are compatible. The data
publishes no schema (see [What the data records](#what-the-data-records))
and jj publishes no stability guarantee for its on-disk formats.
An unequal pair is not "incompatible", it is *unknown*, and the
only correct response to unknown on a path that writes is to
stop. Strict equality is the conservative reading of an
unanswerable question.

## No direction is exempt

The op store serializes with protobuf, and prost discards
unknown fields: a derived `merge_field` routes an unrecognized
tag to `encoding::skip_field`, which advances past it and drops
it. There is no unknown-field set retained. Since the next writer
serializes from the decoded struct, both directions lose data:

- ours newer than theirs: we write fields their jj skips, and
  their next write drops them.
- ours older than theirs: they wrote fields we skip, and our next
  write drops them.

Content addressing means the original blob survives under its own
id, so this is loss of current state rather than destruction of
history. An earlier framing exempted the second direction on the
grounds that jj must support reading its own older ops. That
holds for jj *reading*. It stops holding once our writes are in
the picture, which they are. Hence `!=`, not an ordering test.

## Reads write

"Read" in jj-lib means "does not write anything the caller asked
for", which is why the gate cannot be scoped to the write path.
Three read paths that write:

- `load_at_head` resolves op heads and, when several have
  diverged, merges them and writes a new operation.
- The index self-heals. A stale or format-mismatched index makes
  `DefaultIndexStore` reindex and write new segment files.
  `COMMIT_INDEX_SEGMENT_FILE_FORMAT_VERSION` is a compile-time
  constant, so a mismatched pair does not merely risk this, it
  guarantees churn in both directions.
- Any `@`-relative read needs a working-copy snapshot, which
  writes `tree_state` and can create a commit.

## What the gate guarantees

Narrowly: *we never run against a jj differing from the one we
can see*. Not "no old jj will ever misread our op". A user can
have several jj binaries installed, and what our `$PATH` resolves
to says nothing about what an editor integration or another shell
runs against the same repo. The check is a sample, and stating it
as anything more would make it the kind of mechanism that looks
like a guarantee and is not.

## The carve-out

Commands that provably do not open a repo are not gated, because
gating them buys no safety and costs real things: a `jj -V` spawn
on every shell-completion tab press, and a hard `jj`-on-`$PATH`
dependency for what is otherwise a markdown linter. The set:
`--help`, shell completion, `-V` (the banner alone), and the
markdown commands (`fix-todo`, `validate-todo`).

`vc-x1 version` and `-VV` are *not* in that set: they read each
repo's backend types, so they open repos. They are instead
ordered around the gate. The report is the diagnostic you run
when the gate has stopped you, so it must still answer: on a
mismatch it prints our version, their version and the verdict,
and says the `jj-data` lines are withheld.

The carve-out is defined as a property, "provably does not open a
repo", rather than as a list to be maintained by hand, so it can
be tested.

## Ordering

Parse args, init logging, gate, then everything else, with
nothing between the gate and the first repo access. Literally
first is not implementable: before the parse we cannot tell a
completion request from a real invocation, and we have no `-v` /
`--log` set up to report the failure through.

A missing `jj` on `$PATH` is a distinct failure from a version
mismatch and gets a distinct message, because the fix is
different.

## What the data records

Nothing usable, which is why `jj -V` is the second operand
despite being a poor one.

Measured 2026-07-31 against `jj 0.43.0`: `commit=git
op=simple_op_store op-heads=simple_op_heads_store index=default
submodule=default working-copy=local`. Every value is an
identity, never a version. They are the
`.jj/repo/<backend>/type` files that
`RepoLoader::init_from_file_system` reads, and they are all
jj-lib exposes.

The bytes themselves carry no more. A proto3 message serializes
to (field number, wire type) keys plus payload: no message name,
no schema id, no field names. That absence is exactly what makes
an unknown field skippable, so the property that lets jj evolve
the format is the property that makes the evolution undetectable.
Sniffing which tags are present does not substitute:

- proto3 has no presence for scalars, so a field the writer left
  unset and a field the writer never heard of are the same bytes.
- a new tag appears only once a newer jj populates it, so a newer
  jj that has the field but has not used it is byte-identical to
  an older one.
- prost does not surface unknown tags at all.

A compile-time schema fingerprint would not close this either.
Equality needs two operands and the data supplies none. A stamp
we wrote ourselves would record only what *we* last wrote, would
go stale the moment the user's jj wrote without updating it, and
would never be read by the old jj that is the endangered party.
So `jj -V` is a proxy for schema identity, not schema identity.

## Costs

- A jj release stops vc-x1 entirely, not just its writes, until
  the lock is bumped and revalidated. The override flag is what
  keeps the tool usable that day, so it is load-bearing rather
  than a nicety.
- The override is a per-invocation flag, never a config key. A
  config key gets set once during a frustrating afternoon and
  then silently protects nothing.
- One `jj -V` spawn per invocation, cached per process. Ironic in
  the cycle that ends subprocess spawning, but it is a spawn on
  their side of the boundary and there is no other way to ask.

## Known holes

- `jj -V` prints a release triple plus a build hash, and we
  compare triples. A jj built from git between releases claims
  the release triple while being arbitrarily far ahead of it, and
  a hash cannot be mapped to a schema.
- Version equality is coarser than schema equality, so two
  releases with an identical op-store proto still trip the gate.
- The `$PATH` sample problem, above: other jj binaries are
  outside the gate entirely.

## How this could be relaxed

The pedantry is deliberate and provisional. The measurement that
would justify narrowing it: hash every file under `.jj/` in both
repos, run one command against a deliberately mismatched jj-lib,
hash again, diff. The index case should light up immediately,
which is itself evidence for keeping the gate broad; anything
genuinely inert becomes a candidate for narrowing, backed by a
measurement rather than an assumption.

Relaxing the *coarseness* is a different lever: a schema
fingerprint that hashes jj-lib's shipped `.proto` files at build
time would turn "a green build after a bump is not evidence" into
a red build for the op-store-shape class, and would make an
override provably safe when the schema is unchanged. It catches
nothing semantic, and cargo hands a build script only its own
`CARGO_MANIFEST_DIR`, so locating a dependency's `.proto` needs
`cargo metadata` or the registry layout. Wants a `## Todo` entry
of its own.
