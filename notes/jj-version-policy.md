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

## No carve-out

Every subcommand is gated. The one exception is `vc-x1 version`,
and it is not really an exception: it reports the verdict rather
than acting on it, and on a refusal it prints our version, their
version and the reason, then says the `jj-data` lines are
withheld. The diagnostic you run when the gate has stopped you
has to answer.

An earlier draft exempted the commands that do not open a repo
(the markdown ones, `config`, `bot-session`, `symlink`). That was
dropped 2026-07-31, before it ever shipped, for a reason worth
keeping: such a list enforces only its own completeness. A new
subcommand can be made to force a decision, but an existing
command that grows a repo read later stays classified as safe,
silently, and nothing fails. "Provably does not open a repo" was
the stated property and grep was the actual proof.

Two commands need no exemption. `--help` exits inside clap during
parse, and shell completion exits inside `CompleteEnv` on main's
first line; both are before the gate however it is written. The
cost of bluntness is therefore narrower than it looks: a markdown
linter that needs a version-matched jj, which is what
`--allow-jj-mismatch` is for.

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
  releases that store the same bytes still trip the gate.
  Measured instance, 2026-07-31: `0.42` and `0.43` ship
  byte-identical `.proto` files and the same
  `COMMIT_INDEX_SEGMENT_FILE_FORMAT_VERSION`, and diffing their
  sources shows `content_hash.rs`, `simple_op_store.rs`,
  `op_store.rs` and `default_index/{readonly,mutable}.rs`
  unchanged, with the two changed storage-adjacent files
  (`backend.rs`, `local_working_copy.rs`) changing only doc
  comments, an unused constructor, a string literal and a
  redundant `.max(1)`. A vc-x1 linking `0.42` would refuse
  against `jj 0.43` while being safe to run.
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

Relaxing the *coarseness* is a different lever, and a weaker one
than it first looked. A schema fingerprint hashing jj-lib's
shipped `.proto` files at build time would turn "a green build
after a bump is not evidence" into a red build for one narrow
class, a change in the shape of the stored schema. It cannot make
an override safe. A fixed schema still permits the same fields
being populated with different meanings, non-protobuf state like
the index segments moving, and content hashing changing so that
the same logical data lands under different ids. The 0.42/0.43
comparison above ruled those out by reading a source diff across
four files, which is not a check anyone will reliably repeat on
every bump. (Cargo also hands a build script only its own
`CARGO_MANIFEST_DIR`, so locating a dependency's `.proto` needs
`cargo metadata` or the registry layout.)

The stronger lever watches the output rather than the source. Ids
are where every serialization and hashing decision surfaces, so a
change in any of the three classes above shows up as a changed
id. Two ways to look, over one fixture:

- record ids under the current jj-lib, commit them, and let the
  next bump re-run them.
- build a probe against N-1 and N, run both over the same
  fixture, and diff. Answers "did this bump move the data?"
  before we commit to the bump.

Both are samples, not proofs: they compare versions on one
fixture, so a change touching a path the fixture does not
exercise reports "same" and is wrong. Tracked by the `## Todo`
entry "validate-repo-data".

The 0.42/0.43 finding recorded under
[Known holes](#known-holes) is itself a one-off, established by
reading a source diff in the 2026-07-31 session. Nothing re-runs
it, so it is true of that pair on that date and should not be
read as a standing property of adjacent releases.
