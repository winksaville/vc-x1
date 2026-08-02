# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

When a `## Todo` item is picked up, its text **moves** here
(never copied; one home per text). The picked-up task is a
`###` heading; a multi-cycle program adds one level, where the
program is the `###` and its current stage a `####` (headings
give the current work durable anchors, which numbered Todo
entries can't). The problem overview is followed by the
"plan", a bulleted list of the development "ladder". Each
rung is prepended with its commit reference, a literal
`[[N]]` placeholder until the commit is pushed, then
backfilled to a real file-local `[[n]]` ref (same pattern as
the chores As-built rungs):
   - [[N]] 0.xx.y-0 blah (done)
   - [[N]] 0.xx.y-1 blah blah (current)
   - [[N]] 0.xx.y-2 blah blah blah
   - [[N]] 0.xx.y close-out and validation

### Refactor: typed jj facade -> jj-lib in-process; end subprocess spawning

Version-control operations were ~30 hand-rolled `run("jj", ...)`
spawns plus every mutation, with per-module private wrappers
and raw-git vestiges in init: stderr parsing instead of typed
errors, and jj's single-attempt index-lock acquisition (the
push `bookmark-set` lock race in [bugs.md](notes/bugs.md))
can't be retried where it fails. A multi-ladder program; the
staged plan, design detail, and the eight absorbed former
Todos live in
[refactor-20260716.md](notes/refactor-20260716.md).

Program ladder in **straight trunk order**, one bullet per
commit that lands on the branch, so reading it top to bottom
is reading `git log --first-parent`. `refactor:` entries are
the program's rungs, one per cycle (adjacent stages
consolidated 2026-07-24; titles are the anticipated close-out
titles; unshipped versions are provisional, since jj-lib may
split into two cycles). `docs:` entries are interludes that
sit between cycles and belong to no rung. Every shipped ref
points at a close-out commit on `refactor-vc-x1`, treated as
permanent: the branch is long-lived and lands on main
merge-only, never rebased.

The order is load-bearing: a
trapezoid's `<base>` is the parent of its own first rung, not
the previous close-out, so 0.78.0 bases on 0.77.4. Taking the
close-out instead swallows the interludes into the merge's
ladder side, which already bit at 0.76.0, whose base was the
0.75.1 interlude. See
[the recipe](notes/cycle-protocol.md#trapezoid-close-out-recipe).

- [[1]] 0.73.0 refactor: DRY jj facade (done)
- [[2]] 0.74.0 refactor: hygiene riders (done)
- [[3]] 0.75.0 refactor: facade owns topology (done)
- [[12]] 0.75.1 docs: refactor program ladder + conventions (done)
- [[4]] 0.76.0 refactor: repo registry (done)
  - first trapezoid published by the four-step recipe
- [[13]] 0.76.1 docs: trapezoid close-out recipe (done)
- [[10]] 0.77.0 refactor: stateless push (done)
- [[11]] 0.77.1 docs: jj-lib design notes + trapezoid recipe (done)
- [[16]] 0.77.2 docs: typeable punctuation (done)
- [[18]] 0.77.3 docs: re-describe rule + defer punctuation
  sweep (done)
  - retires the two `0.77.x` docs rungs into a `## Todo`
    (source sweep) and the backlog (interlude shape)
- [[20]] 0.77.4 build: bump jj-lib to 0.43 (done)
  - not migration work; keeps the read-side compiling
    against the installed jj 0.43.0, and correct whichever
    way the mutation decision goes
- [[N]] 0.78.0 refactor: jj-lib migration (current)
- [[N]] 0.79.0 refactor: trapezoid-push + body-intro
  validation
  - the `## Todo` entry "refactor: trapezoid-push +
    body-intro validation"

#### refactor: jj-lib migration

Facade internals and mutations move in-process, ending jj and
git spawning; see
[the stage](notes/refactor-20260716.md#stage-jj-lib-migration).
Scope settled 2026-07-30: all three pieces, accepting the
op-store version coupling that the migration introduces. The
version gate at `-5` is what makes that coupling enforceable
rather than merely accepted, which is the change from the
2026-07-29 framing.

- [[21]] 0.78.0-0 chore: open the jj-lib migration cycle
  (done) [detail](#0780-0-chore-open-the-jj-lib-migration-cycle)
- [[22]] 0.78.0-1 docs: adopt universal AGENTS (done)
  [detail](#0780-1-docs-adopt-universal-agents)
  - inserted 2026-07-30: the AGENTS restructure proposed in
    vc-x1-work-repo-template becomes this repo's live
    instructions, dogfooded for the rest of the cycle; lands
    first so the remaining rungs run under the new rules
- [[23]] 0.78.0-2 feat: report jj-lib and jj-data versions
  (done)
  [detail](#0780-2-feat-report-jj-lib-and-jj-data-versions)
  - split out of the former `-2` on 2026-07-31: the rung had
    grown a `build.rs`, a module, and a CLI behavior change,
    which no `docs:` title covers
  - the measurement lands before the prose that cites it
- [[24]] 0.78.0-3 docs: jj-lib version coupling policy
  (done)
  [detail](#0780-3-docs-jj-lib-version-coupling-policy)
  - the policy proper goes to `notes/`, beside the risk
    section it supersedes; `TODO.md` keeps the narrative that
    moves to chores at close-out
  - retires three recorded conclusions at once, so they move
    together or the notes argue with themselves: the risk
    section's `jj --version` verdict, this ladder's
    write-path-only bullet, and the "Decisions at cycle open"
    claim that one direction is safe
- [[25]] 0.78.0-4 feat: jj-lib version gate (done)
  [detail](#0780-4-feat-jj-lib-version-gate)
  - moved ahead of the reads rung on 2026-07-31: the ladder
    put the gate before the *mutations*, but
    `common::load_repo` has called `load_at_head` since the
    facade moved to jj-lib, so the write-capable read path is
    live now and the hole is open today, not later
  - builds only the gate; both operands ship at `-2` and the
    rule is written down at `-3` in
    [the policy](notes/jj-version-policy.md), which this rung
    implements rather than re-decides
  - no carve-out: every subcommand gates. `version` is the one
    exception and barely one, reporting the verdict rather than
    acting on it, printing both versions and withholding the
    `jj-data` lines on a mismatch
  - the `.vc-config.toml` pin turns a `$PATH` sample into a
    declaration, but only matters once more than one jj is in
    play; it stays a Todo
- [[26]] 0.78.0-5 refactor: jj-lib reads (done)
  [detail](#0780-5-refactor-jj-lib-reads)
  - `jj log` templates become `Commit` accessors
  - `@`-relative reads stay behind: they need a working-copy
    snapshot, which is an op-store write, so they move with
    the mutations
- [[N]] 0.78.0-6 refactor: jj-lib mutations (done)
  [detail](#0780-6-refactor-jj-lib-mutations)
  - commit, describe, bookmark set/track, fetch, push, plus
    the `@`-relative reads deferred from `-5`
- [[N]] 0.78.0-7 refactor: context-owned repo sessions
  (done)
  [detail](#0780-7-refactor-context-owned-repo-sessions)
  - inserted 2026-08-01 at the `-6` review, design settled
    there: `Context` owns lazily-opened `RepoSession`s keyed
    by repo path, has-a and never is-a, because an
    invocation touches 0..N repos and repo-less commands
    (`version`) must not open one; verbs become session
    methods; the one-shot facade fns stay as wrappers for
    context-less callers
  - one op per verb stays: sharing a transaction across
    stages would change the op-log shape that push re-run
    and sync revert rely on
  - per-verb opens are the lifted subprocess lifecycle made
    visible, not a regression; this rung is the improvement
    over the spawned form, and push / squash-push / sync are
    its consumers today
  - ordered before the retry so the retry lands on the final
    frame, though it fits either shape
- [[N]] 0.78.0-8 fix: jj-lib index-lock retry (done)
  [detail](#0780-8-fix-jj-lib-index-lock-retry)
  - renumbered from `-7` by the session insert
  - bugs.md #1, with the `git init --bare` to gix rider
  - the retry classifies by error variant rather than
    substring, which is the real win: `SpawnInPath` and
    `UnsupportedGitOption` are never retryable, and treating
    the whole `Subprocess` arm as retryable would loop
    forever on a missing git binary
- [[N]] 0.78.0 refactor: jj-lib migration (close-out)

##### Decisions at cycle open

- **All three pieces**, decided 2026-07-30: jj-lib for reads,
  jj-lib for mutations, and the index-lock retry that is the
  headline prize. The 2026-07-29 session ended undecided
  between this and deferring mutations.
- **What changed is the coexistence objection, not the
  evidence.** The risk section concluded the coupling was
  "unenforceable, probably fine" because a `jj --version`
  check cannot answer whether two versions are compatible.
  That evaluates it as a compatibility oracle. As a guard on
  our own writes it fits the actual risk direction: the
  dangerous case is an old jj reading an op written by a
  newer jj-lib, and refusing to write on a mismatch closes
  exactly that. The safe direction, a newer jj reading our
  older op, is something jj must support anyway, since the
  user's own older jj wrote into that repo first.
  - **Half superseded 2026-07-31.** The conclusion stands;
    the "safe direction" sentence does not. See
    [why equality, and why at startup](#why-equality-and-why-at-startup).
    Kept as written because this section records what was
    decided at open, not what we believe now.
- **The gate lands at `-5`, before the mutations at `-6`**, so
  mutations arrive in a repo that already refuses on
  mismatch. The cost is one commit whose check guards nothing
  yet, which reads oddly in isolation; folding it into `-6`
  would avoid that but make one rung do two things.
  - renumbered 2026-07-31 by the `-2` split; the decision is
    unchanged, the gate still lands one rung ahead of the
    mutations
- **Deferring mutations was not free**, which is what settled
  it. The trapezoid reshape at 0.79.0 is a `jj rebase`, so
  "reads only" would leave that cycle either spawning or
  waiting.

##### The 0.43 bump was a preview of the cost

The `build: bump jj-lib to 0.43` interlude that immediately
precedes this cycle is worth reading as evidence rather than
housekeeping. Two releases of a pre-1.0 library produced two
breaks of different kinds: `use_glob_by_default` disappearing
from `RevsetParseContext`, which the compiler caught, and the
default revset string-pattern kind moving from substring to
glob, which it structurally could not.

We think the useful lesson is narrow and worth keeping in
front of us for the rest of this cycle: a green build after a
jj-lib bump is not evidence that the bump preserved behavior.
That is the treadmill cost the mutation decision accepts, and
it is paid on every bump, not only on the ones that touch the
op store.

##### 0.78.0-0 chore: open the jj-lib migration cycle

Preparation only. The `## Todo` entry moved into
`## In Progress` as this cycle block, carrying the ladder and
the version-gate design this session worked out, and
`fix-todo` renumbered the 19 entries left behind.

Two stale references surfaced while renumbering and were
fixed in the same commit: the `0.79.0` program rung pointed
at "Todo #2", a positional number the renumber invalidated,
and now names the entry by title as the convention requires;
and the program ladder still said `0.78.0` bases on `0.77.2`,
which two interludes had since made wrong.

The narrative lives here rather than in `chores-15.md` per
the one-home convention adopted at `0.77.0-2`. This commit
first did it the old way, because
[Chores conventions](/agent-data/notes.md#chores-conventions)
still describes the superseded per-commit build-up (overridden
for this repo in [custom.md](/custom.md)); the `## Todo`
entry "One home for a cycle's narrative" is what closes that
gap.

##### 0.78.0-1 docs: adopt universal AGENTS

Inserted after the cycle opened. The AGENTS restructure
(short universal AGENTS.md + `agent-data/` satellites +
`custom.md` as the one agent-editable instruction file) is
proposed in vc-x1-work-repo-template as
`AGENTS-vc-x1-f5-20260730.md` with a `-notes.md` companion,
and this repo adopts it now to dogfood it:

- the local copy is authoritative during the dogfood window;
  the template snapshot is frozen for discussion
- promotion back to the template happens en masse or
  incrementally as the local copy proves out
- findings land in `custom.md`'s dogfood log; semantic rule
  changes wait for that evidence

Semantics-preserving by design: rules keep their current
meaning, only the organization changes (checklists at the
moment of action, rationale behind them, project specifics
in `custom.md`). We think that keeps any adherence change
attributable to the structure, which is the hypothesis under
test.

##### 0.78.0-2 feat: report jj-lib and jj-data versions

`--version` reports three versions that answer different
questions, so the policy at `-3` can be written against
measured output instead of inference:

- ours, `CARGO_PKG_VERSION`, compile time
- jj-lib's, `JJ_LIB_VERSION`, resolved from `Cargo.lock` by a
  new `build.rs`
- the data's, read through jj-lib's public accessors only, one
  `jj-data` line per repo

The report is its own `version` subcommand, answered before
`Context::load` so it still works when the workspace is the
thing that is broken. That is what let the gate at `-4` name it
as its one exception: "the `version` subcommand gathers
`jj-data` lines only after the gate passes" is a sentence the
policy can hold; "the bare invocation with no subcommand" was
not. (Written when the gate was still `-5`.)

Version output now rides along with every run, so any captured
output says which version produced it. Stream by who asked:

- no flag: stderr. Provenance that was not asked for must not
  land in the stream `chid`, `desc`, `list` and `show` emit
  data on, or piping them breaks.
- `-V` / `--version`: the banner on stdout. An explicit request
  makes it data, capturable alongside the command's own output.
- `-VV`: the full report on stdout, then the subcommand runs.
  This is the one thing the subcommand cannot do, and what a
  bug report wants stamped on top of a real command's output.
- `--no-banner` silences the ambient one; `-V` still prints,
  since asking outranks suppressing.

Counted like the `-v` / `-vv` this CLI already teaches, so
version detail scales the way verbosity does and needs no
separate explanation.

We rejected `-V` versus `--version` as the terse/full split.
Those two being aliases is close to a universal CLI
convention, and this project prefers invariants that can be
stated in one line.

The ambient banner uses `eprintln!` rather than the logger,
because `CliLogger` routes by level and puts info on stdout.
The cost is that `--log` does not capture it.

The `build.rs` is `-5`'s mechanism arriving three rungs early,
which was not the plan: jj-lib exports no version constant and
no accessor for one, so printing the version at all requires
resolving it from the lock. `-5` inherits it and adds only the
comparison and the refusal. The lock is read from
`$CARGO_MANIFEST_DIR` rather than by walking ancestors,
because we are not a workspace and a walk can bind a sibling
project's lock, which is worse than failing.

`data_version` stops at `Workspace::load` and never calls
`load_at_head`, since resolving op heads can merge divergent
ones, which is a write. A version report must not mutate what
it reports on.

We think one wrong turn is worth recording. The `build.rs`
parser first shipped with unit tests, which never ran: cargo
does not compile a build script as a test target. They were
the exact defect class the `## Todo` entry "A committed
cycle-check runner" describes, a mechanism that looks like a
guarantee and is not, so they were deleted. The parser is
covered instead by a test asserting the compiled-in version
matches `Cargo.lock`, scanned deliberately unlike `build.rs`
scans it, so a parser that drifted onto the wrong
`[[package]]` block cannot agree with itself.

##### 0.78.0-3 docs: jj-lib version coupling policy

The policy proper goes to `notes/`, beside the risk section it
supersedes, and not into chores at close-out. Chores is
append-mostly history organized by when work happened; this
rule governs what the tool does from `-5` onward, and someone
asking "why does vc-x1 refuse to run" should not have to know
which cycle produced the answer. `TODO.md` keeps the
narrative, chores gets it verbatim, and the two cross-link
rather than restate, the same division `notes.md` draws
between a commit body and its chores section.

The rule lands as [jj-version-policy.md](notes/jj-version-policy.md),
a topic file rather than a section of the plan file: the plan
file becomes historical when the refactor program ends, and the
gate ships in the product. `notes/README.md` gains the general
form of that split, since it is not specific to this rule.

Three recorded conclusions retire together, since leaving any
one would have the notes arguing with themselves. Two were
annotated in place at `-2` as they were found; this rung
finishes the third and links all of them to the policy:

- the risk section's "a `jj --version` check does not work",
  which judged the check as a compatibility oracle. Its
  findings stand and are what the policy rests on; only the
  closing verdict is superseded, so the section is annotated
  rather than rewritten.
- this ladder's "refuse on the write path only" bullet, rewritten
  at `-2` when the startup gate was decided
- the "Decisions at cycle open" claim that a newer jj reading
  our older op is the safe direction, marked half-superseded at
  `-2`

A fourth surfaced while writing the policy: the `-5` carve-out
still listed `--version` among the commands that never open a
repo. That stopped being true at `-2`, when the report grew
`jj-data` lines. `-V` alone still qualifies; `version` and `-VV`
do not, and are ordered around the gate instead.

##### 0.78.0-4 feat: jj-lib version gate

Implements [the policy](notes/jj-version-policy.md) written at
`-3`; the design questions were settled there and this rung
re-decides none of them.

Moved ahead of the reads rung when the ladder's own reason for
its position turned out to be wrong. "The gate lands before the
mutations, so mutations arrive in a repo that already refuses"
assumed the exposure arrives with the mutations. It did not:
`common::load_repo` has called `load_at_head` since the facade
moved to jj-lib, and it backs `chid`, `desc`, `list` and `show`.
Op-head merging and index reindexing have been happening
in-process, ungated, for several cycles. The hole is open now,
and `-5` would have widened it first.

The gate applies to every subcommand, with no list of exempt
ones. It first shipped in this rung's working copy with a
carve-out: an `opens_repo` method, an exhaustive `match` on
`Commands` with no wildcard arm, so a new subcommand would fail
to compile until someone picked a side. That was dropped at
review, unpushed, on an argument that holds:

- the match enforces enumeration, not classification. A new
  subcommand does force a decision; an existing one that grows a
  repo read later stays classified as safe, silently, and nothing
  fails.
- the policy called the carve-out "provably does not open a
  repo". The actual proof was a grep of five modules for
  `load_repo` and friends, which is a point-in-time observation
  wearing the word "provably".
- two of the costs claimed for keeping the list were not real.
  `--help` exits inside clap's `e.exit()` during parse, and
  completion exits inside `CompleteEnv::complete()` on main's
  first line, so neither ever reached the gate.

What remains is one rule with no list to maintain: every
subcommand except `version` refuses on a mismatch. The cost is
that a markdown linter needs a version-matched jj, which is what
the per-invocation override is for.

`jj -V` is spawned once per process and cached, not once per
operation. Ironic in the cycle that ends spawning, and
unavoidable: it is a spawn on their side of the boundary.

Three failures, not one, because the fix differs each time: `jj`
absent from `$PATH`, `jj -V` unreadable, and a genuine mismatch.
Only the third mentions `--allow-jj-mismatch`, since the override
is meaningless for the other two.

A measured over-strictness landed in the policy's known holes on
the way: `0.42` and `0.43` store the same bytes, so a vc-x1
linking `0.42` would refuse against `jj 0.43` while being safe to
run. The route to that finding is worth more than the finding.
The first evidence offered was that the `.proto` files are
identical, and the conclusion drawn from it, that the data cannot
have changed, was wrong: a fixed schema still permits the same
fields meaning different things, non-protobuf state like the
index segments moving, and content hashing changing so the same
data lands under different ids. Ruling those out took a source
diff across four files. Nobody will repeat that on every bump,
which argues *for* the blunt gate, and it demotes the schema
fingerprint from "makes an override provably safe" to "catches
one narrow class at build time".

The refusal path is what the test drives, with a fake `jj` first
on `PATH` reporting `0.99.0`: the real pair matches here, so a
test that only exercised the match would pin nothing. It checks
all four consequences at once, that a repo-opening command
refuses and names the override, that the override works, that a
markdown linter is unaffected, and that `version` still answers
while withholding the `jj-data` lines.

##### What the data records about itself

Measured 2026-07-31 against `jj 0.43.0`, both repos identical:
`commit=git op=simple_op_store op-heads=simple_op_heads_store
index=default submodule=default working-copy=local`.

Every value is an identity, never a version. They are the
`.jj/repo/<backend>/type` files that
`RepoLoader::init_from_file_system` reads, and they are all
jj-lib exposes: 0.43 has no public version constant and no
accessor for one. This is the risk section's "the op store has
no version stamp", now observed rather than inferred from
reading jj-lib's source.

Nor can the stamp be recovered from the data itself. A proto3
message serializes to (field number, wire type) keys plus
payload bytes: no message name, no schema id, no field names.
That absence is exactly what makes an unknown field skippable,
so the property that lets jj evolve the format is the same
property that makes the evolution undetectable. Three reasons
sniffing the tags present cannot substitute:

- proto3 has no presence for scalars, so a field the writer
  left unset and a field the writer never heard of are the
  same bytes
- a new tag appears only once a newer jj populates it, so a
  newer jj that has the field but has not used it is
  byte-identical to an older one
- prost does not surface unknown tags at all. The derived
  `merge_field` routes an unrecognized tag to
  `encoding::skip_field`, which advances past it and discards
  it; there is no unknown-field set to inspect

The last point is the one that matters beyond detection, and
it is why a compile-time schema fingerprint would not help
either: equality needs two operands, and the data supplies
none. A stamp we wrote ourselves would record only what we
last wrote, would go stale the moment the user's jj wrote
without updating it, and would never be read by the old jj
that is the endangered party. `jj -V` stays the second operand
not because a version is the right thing to compare, but
because it is the only thing the other side emits. It is a
proxy for schema identity and the policy should say so.

##### Why equality, and why at startup

Two findings from the 2026-07-31 session, both of which
retire text written at cycle open.

**The loss is symmetric, so no direction is safe.** Because
prost discards unknown tags and the next writer serializes
from the decoded struct, an old jj and a new jj can each
destroy what the other wrote:

- ours newer than theirs: we write fields their jj skips, and
  their next write drops them
- ours older than theirs: they wrote fields we skip, and our
  next write drops them

Content addressing means the original blob survives under its
own id, so this is loss of current state rather than
destruction of history. The recorded rationale exempted the
second direction on the grounds that jj must support reading
its own older ops. That holds for jj reading. It stops holding
once our writes are in the picture, which they are. So the
test is `!=`, not an ordering comparison, and the reason is
better than the one we first wrote down.

Equality is also the honest response to an unanswerable
question. We cannot compute compatibility, because the data
publishes no schema and jj publishes no stability policy. An
unequal pair is not "incompatible", it is "unknown", and the
only correct response to unknown on a path that writes is
stop.

**Reads write, so the gate cannot be scoped to writes.**
"Read" in jj-lib means "does not write anything the caller
asked for". Three paths we already know:

- `load_at_head` resolves op heads and, when several have
  diverged, merges them and writes a new operation
- the index self-heals: a stale or format-mismatched index
  makes `DefaultIndexStore` reindex and write new segment
  files. `COMMIT_INDEX_SEGMENT_FILE_FORMAT_VERSION` is a
  compile-time constant, so a mismatched pair does not merely
  risk this, it guarantees churn in both directions
- any `@`-relative read needs a working-copy snapshot, which
  writes `tree_state` and can create a commit; this is
  already why `-5` defers those reads to `-6`

So the gate fires at startup and stops before anything opens a
repo. What it guarantees is narrow and should be stated
narrowly: not "no old jj will misread our op", only "we never
run against a jj differing from the one we can see". The
`$PATH` sample objection survives intact, since an editor
integration running a different jj is outside the gate.

Known holes and costs, recorded now so they are not
rediscovered at `-4`:

- `jj -V` prints `jj 0.43.0-<40-hex>`, and we compare triples.
  A jj built from git between releases claims the release
  triple while being arbitrarily far ahead of it, and a hash
  cannot be mapped to a schema, so that hole stays open.
- version equality is coarser than schema equality, so two
  releases with an identical op-store proto still trip the
  gate. Relaxing that safely needs a schema fingerprint:
  hash jj-lib's shipped `.proto` files at build time and fail
  the build when a bump changes them, turning "a green build
  after a bump is not evidence" into a red build for the
  op-store-shape class. It catches nothing semantic, so the
  0.43 glob-versus-substring change would still pass, and
  cargo hands a build script only its own
  `CARGO_MANIFEST_DIR`, so locating a dependency's `.proto`
  needs `cargo metadata` or the registry layout. Wants a
  `## Todo` entry of its own; not this cycle.
- a jj release stops vc-x1 entirely until the lock is bumped
  and revalidated, not just its writes. The override flag is
  what keeps the tool usable that day, so it is load-bearing
  rather than a nicety, and it is a per-invocation flag: a
  config key gets set once during a frustrating afternoon and
  then silently protects nothing.

The pedantry is deliberate and provisional. The measurement
that would let it relax: hash every file under `.jj/` in both
repos, run one command against a deliberately mismatched
jj-lib, hash again, diff. The index case should light up
immediately, which is itself the evidence for keeping the gate
broad; anything genuinely inert becomes a candidate for
narrowing later, backed by a measurement instead of an
assumption.

##### 0.78.0-5 refactor: jj-lib reads

The facade's internals flip: in-process through jj-lib is now
the default read path, and spawning is the carve-out rather
than the mechanism. The seam the DRY-facade cycle bought is
what made this a one-module change: every caller kept its
signature, so push, squash-push, sync, init and the registry
checks moved without being edited.

- The routing is per revset, not per call site, because the
  revs are runtime values: squash-push's source/target default
  to `@`/`@-` but are user-overridable, so no static split of
  the call sites exists. `references_working_copy` decides: a
  `@` is working-copy syntax (`@`, `@-`, `ws@`) unless it has
  symbol characters on both sides, the remote-bookmark form
  (`name@remote`).
- Working-copy revsets keep the spawn path on purpose (the
  ladder's standing caveat): the CLI auto-snapshots, so "is
  `@` empty right now?" answers about the filesystem, while an
  in-process `load_at_head` would answer about the last
  snapshot. Those reads move at `-6` with the mutation lift.
- The raw `log(repo, rev, template)` primitive is gone from
  the facade surface: jj-lib has no template engine (templates
  live in jj-cli), so its one caller, sync's bookmark-heads
  probe, became the typed `cids_short_of`.
- `rev_exists` and sync's `try_commit_id` now classify the
  unresolvable-revision error through one helper,
  `is_no_such_revision`: a typed
  `RevsetResolutionError::NoSuchRevision` downcast on the
  in-process path, the old stderr substrings on the spawn
  path. A first taste of the `-8` principle that
  classification is by variant, not wording.
- Parity is pinned by tests: the in-process accessors are
  compared against spawned `jj log` templates on a fixture
  repo, with revs pinned to concrete commit ids so the tests
  stay on the in-process path.
- `bookmark_list` / `bookmark_list_all` still spawn: their
  consumers parse the CLI listing textually. They are not
  `jj log` templates, so not this rung's scope; where they
  land (a typed view query, or a rider on `-6`) is an open
  ladder question.

##### 0.78.0-6 refactor: jj-lib mutations

The workspace/transaction/op-store lift. A new `jj::session`
module's `RepoSession` is the CLI's `WorkspaceCommandHelper`
plumbing reduced to what the facade's verbs need, written
against jj 0.43's `cli_util.rs` as the reference; the facade
grows the five publish-path verbs on top, and the `@`-read
carve-out from `-5` closes. Named for what it is (an open ->
mutate -> finish working session with one repo) and
backend-neutral on purpose; "engine", the working title, is
machinery you start, not a thing you open per operation.

- The session is three pieces: a settings loader replicating
  the CLI's config discovery (`/etc/jj`, `$JJ_CONFIG`, user
  files, `.jj/repo/config.toml`, `JJ_USER`/`JJ_EMAIL`), the
  snapshot cycle (git HEAD/refs import around the
  working-copy snapshot, under the CLI's own
  `git_import_export.lock`), and transaction finish (git HEAD
  reset + ref export, op commit, working-copy update).
  Colocation drives the git halves; these repos are
  colocated, so that fidelity is the bulk of the session
  module.
- Verbs: `commit`, `describe`, `bookmark_set`,
  `git_push_bookmark`, `git_fetch`. Call sites swapped in
  push, squash-push, sync, fix-desc, init, and repo_utils.
  The ladder's "bookmark track" had no call site to lift:
  jj-lib's `push_refs` marks pushed bookmarks tracked, which
  is the side effect init's no-`--allow-new` design relied on
  in the spawned form too.
- The `@`-read deferral resolves as predicted at `-5`:
  `references_working_copy` survives as the trigger, now
  routing to snapshot-then-read (`repo_for_read`) instead of
  to a spawn; `log_spawn` is deleted and
  `is_no_such_revision` drops its stderr-wording fallback,
  leaving only the typed `NoSuchRevision` downcast.
- Fetch returns typed changed-bookmark lines; sync's
  stderr-capture wrapper (`fetch_silent`) now just relabels
  them, keeping the clean-case silence it existed for.
- Documented deviations from the CLI, each at its function:
  no immutability preflights (rewrite targets are validated
  by callers), a small defaults layer for three keys whose
  defaults ship in the CLI's config files rather than
  jj-lib's, the auto-track map driven by
  `git.auto-local-bookmark` alone, and the fetch expression
  pinned to all-branches rather than the remote's refspec
  config.
- Still spawning after this rung: `jj squash` (squash-push),
  `jj new` / `jj rebase` / `jj op log` / `jj op restore`
  (sync, revert), `jj git clone` (clone), `jj diff --stat`
  (push preview), init's `gh` / `git init --bare` (the `-8`
  gix rider) / `jj git init --colocate` / `jj git remote
  add`, the facade's two bookmark listings, and the gate's
  `jj -V`, which is a spawn by definition. The migration
  stage's "removes spawning entirely" now reads as this
  cycle's five verbs plus a remainder with named homes.
- Validation is the existing integration suites now running
  entirely through `RepoSession` (init, push, squash-push, sync
  fixtures), plus facade tests pinning colocated-git export:
  after in-process commit / bookmark-set / describe, `git
  rev-parse` sees the same commit ids jj reports.

##### 0.78.0-7 refactor: context-owned repo sessions

`Context` grows the session map the `-6` review designed:
lazily-opened `RepoSession`s keyed by canonicalized repo path,
opened on first use by `Context::session` and reused for the
rest of the invocation. Has-a, never is-a: an invocation
touches 0..N repos, and a repo-less `version` never opens one.
The five verbs move from facade fns onto `RepoSession` as
methods; the facade keeps one-shot wrappers for context-less
callers.

- `SubcommandRunner::run` (and `dispatch`) take
  `&mut Context`: the sessions are exclusive mutable state,
  and the borrow checker enforcing one live session borrow at
  a time is what a `RefCell` would trade for runtime panics.
  The fifteen non-consumer subcommands change signature only.
- Verbs as `RepoSession` methods: `commit`, `describe`,
  `bookmark_set`, `git_push_bookmark`, `git_fetch`, plus the
  `one_commit` resolver and `complete_newline`.
  `DebugCallback` goes private to the session module.
- One-shot wrappers stay for the context-less callers:
  fix-desc (`describe`), init (`bookmark_set`,
  `git_push_bookmark`), repo_utils (`commit`, `describe`).
  `git_fetch`'s only caller is sync, context-ful, so its
  wrapper is deleted rather than kept unused.
- `RepoSession::snapshot` now reloads at the op-store head
  for every repo, not only colocated ones: a session outlives
  single verbs in a `Context`, and a spawned `jj` (squash,
  new, rebase, op restore) may commit operations between
  verbs. Reuse skips only the open (settings + workspace
  load); freshness is per-verb, unchanged.
- Consumers: push threads `ctx` through `mutate` and the four
  mutating stages, and its `squash-push-bot` stage passes the
  same `ctx` into squash-push, so one bot-repo session serves
  the whole run; squash-push and sync take `ctx` for their
  verb sites (`fetch_silent`, `act_on_state`). Reads stay
  one-shot facade fns.
- Tests: `test_helpers::test_ctx()` builds a
  default-user-config `Context`; the push / sync /
  squash-push integration tests pass a fresh one per op call,
  matching the production one-context-per-invocation shape.

##### 0.78.0-8 fix: jj-lib index-lock retry

The bugs.md #1 fix, landing on the `-7` frame as ordered: gix
gives `.git/index.lock` a single attempt, and a git-aware
watcher can hold it exactly when a mutation resets the index,
so the session retries the colocated git half itself. Plus the
planned rider: init's `git init --bare` becomes a gix call,
the last `git` spawn in init.

- `retry_git_lock` wraps the two colocated git blocks
  (`finish_tx`'s HEAD/index reset + ref export, the snapshot's
  intent-to-add + ref export), both strictly before the
  transaction commit, so a retried closure never doubles an
  op-store write.
- `is_lock_contention` classifies by type, never by message
  substring: walk the source chain, downcast each link to
  `gix::lock::acquire::Error`. Never-retryable failures (a
  missing git binary via `SpawnInPath`, an old git via
  `UnsupportedGitOption`) can never carry that type, so they
  classify false without being named, where a broader "retry
  git errors" rule would loop on them forever.
- Backoff: 5 attempts, 25 ms doubling, about 375 ms of
  waiting in total. The observed holds are watcher-brief;
  anything longer surfaces as the same error as before.
- gix becomes a direct dependency (no features of our own, so
  it resolves to exactly jj-lib's 0.85): the downcast needs
  type identity with the errors jj-lib returns.
- The rider: `init_bare_main` uses
  `ThreadSafeRepository::init_opts` with an in-memory
  `init.defaultBranch=main` override, standing in for the
  spawned form's `--initial-branch=main` so the user's git
  config cannot steer which branch vc-x1 publishes to.
- Tests: classifier and retry-loop units (including the
  give-up-after-budget and pass-through cases), a planted
  `.git/index.lock` released mid-backoff by a thread proving
  a mutation survives transient contention, and a bare-init
  test pinning HEAD to `refs/heads/main`.

## Todo

 Entries are in **strict priority rank**, #1 highest,
 descending. Reprioritize by moving an entry, then
 `vc-x1 fix-todo --no-dry-run TODO.md` to renumber.
 The numbers are positional rank, not stable IDs, so to refer
 to a Todo, name it by its **title** (a greppable mention;
 a numbered list item has no anchor to link to), not its
 number. Long-tail entries
 live in [todo-backlog.md](notes/todo-backlog.md). Use the
 [Prose form](/agent-data/prose.md#prose-form); deeper
 detail goes in `notes/chores/chores-NN.md` design
 subsections (link via `[N]` ref).

1. **validate-repo-data.** Golden ids for a fixture repo, so a
   jj-lib bump that moves the on-disk data fails loudly instead
   of building green. The gate at `0.78.0-4` refuses on a version
   mismatch precisely because we cannot tell whether the data
   moved; this is the check that could eventually tell us, and
   the route to relaxing the gate's coarseness. See
   [the policy](notes/jj-version-policy.md#how-this-could-be-relaxed).
   Two modes over one fixture and one id extractor:

   - **Ratchet**, in `cargo test`. Record ids under the current
     jj-lib, commit them, and let the *next* bump re-run them.
     Zero standing cost, catches drift the moment we take a new
     version.
   - **Live pair**, a `support/` script, not a `#[test]`, so
     `cargo test` never pays for it. Build a probe binary twice,
     against N-1 and N, run both over the same fixture, diff the
     reported ids. Generate a throwaway manifest in a temp dir
     for each version rather than adding a crate to our lock.
   - **Trigger the live pair on the jj-lib bump, not on our
     release cycle.** Our cycles run faster than jj's releases,
     so per-cycle mostly re-compares the same pair. The bump is
     when the answer can change, and it is also when the answer
     is most useful: "should we take 0.44?" is a question the
     probe can answer *before* we commit to the bump.
   - The probe needs only the storage-facing API: load a
     workspace, read operation / view / commit / change ids,
     create a commit. That is jj-lib's stable surface. The 0.43
     break that motivated this whole cycle
     (`use_glob_by_default` leaving `RevsetParseContext`) was in
     revset *parsing*, which the probe never touches, so keeping
     it compiling against N-1 should stay cheap.
   - **What it does not cover.** It compares two versions *on
     our fixture*. A change touching a path the fixture does not
     exercise reports "same" and is wrong. A sample, like `jj -V`
     is a sample; say so where it is documented rather than
     letting it read as proof.
   - **Watch operation ids and view ids first.** Those are jj's
     own content-addressed op-store hashes, so they move if
     hashing, serialization, or a stored field's meaning moves.
     Commit SHAs are gix's, computed from commit content, so they
     mostly pin git rather than jj and are the weaker signal.
   - **Change ids are goldenable, and are the best canary in
     the set.** Three cases: a commit authored in jj gets a
     random chid (`JJRng::new_change_id`); a git commit carrying
     a `change-id` extra header keeps the original; and a git
     commit without one gets a *deterministic* chid, the commit
     id's bytes `4..20` reversed and bit-reversed
     (`git_backend.rs`, `synthetic_change_id_from_git_commit_id`).
     Build the fixture by importing git commits and every chid
     is reproducible with no seeding at all.
   - That function's doc says "the exact algorithm for the
     computation should not be relied upon", so jj reserves the
     right to change it. That is a documented instance of the
     schema-invisible drift the gate exists for, and this test
     is what would catch it: the algorithm moving changes every
     synthetic chid at once.
   - **Determinism for the rest.** Operation ids embed
     timestamps and commit ids embed author and committer time,
     so those still need a pinned clock. Random chids, if the
     fixture needs any, are pinned by the `debug.randomness-seed`
     config key (`settings.rs`), which arrives through
     `StackedConfig` and so is reachable from jj-lib without
     going near the CLI.
   - **A committed fixture, not this repo.** Using vc-x1's own
     repo as the guinea pig was the original sketch, but its
     history grows every commit, so the goldens would churn and
     stop meaning anything. A small fixture stays stable and
     fast; this repo can still be a manual proving ground.
   - Read-only commands get the complementary assertion: hash
     every file under `.jj/` before and after, and record which
     ones are genuinely inert. That is the measurement the policy
     names as the way to narrow the gate from "every subcommand"
     to something smaller, backed by evidence.
2. **refactor: trapezoid-push + body-intro validation.**
   `vc-x1 trapezoid-push`, a **subcommand** rather than a flag
   on `push` (decided 2026-07-28), publishes a close-out as a
   non-fast-forward merge; body-intro validation rides as
   the first rung. See
   [trapezoid close-out](notes/refactor-20260716.md#stage-trapezoid-close-out)
   and
   [push body-intro validation](notes/refactor-20260716.md#stage-push-body-intro-validation).
   After jj-lib, so the reshape is built in-process.
   - `push` keeps a stateable invariant: it never produces
     a merge. A mode flag that rewires the stage sequence
     would cost that.
   - Shared implementation, not a second copy: the common
     pipeline (preflight, both gates, message, commit-work,
     commit-bot, bookmark-set, push-work, bot squash) moves
     into its own module that both subcommands call, with
     the reshape as the one inserted step. The
     stateless-push cycle shrinks that pipeline first,
     which is what makes the extraction cheap.
   - A backend `trait` (jj today, git or another VCS later)
     is the natural next abstraction if a second backend
     ever appears. Worth converting these concepts to
     traits then, not now: we are committed to jj, and a
     one-implementation trait buys nothing but indirection.
3. **A committed cycle-check runner.** The per-commit flow's
   validation (fmt -> clippy -> test -> install) exists only as
   prose in cycle-protocol.md, so it is recomposed by hand
   every commit, and a hand-composed shell one-liner can
   silently stop checking. Found at the 0.77.0 close-out: in
   `clippy ... 2>&1 | tail -2 && cargo test ...`, the pipeline's
   status is *tail's*, which is always 0, so the `&&` gate
   was decorative and `cargo test` ran even on a run where
   clippy had failed. The failures were caught by reading the
   output, not by any check.
   - The defect class is the one this cycle spent its time
     deleting: a mechanism that looks like a guarantee and
     isn't.
   - Write the sequence down once. Options, cheapest first:
     `support/cycle-check.sh` with `set -euo pipefail` (there
     is precedent in `support/gen-exmpl-1-3.sh`); a `justfile`
     target; or `cargo xtask`, where the steps are `Command`
     calls whose statuses are handled like any other
     `Result`, most aligned with the no-unwrap discipline and
     heaviest for four commands.
   - **Not a vc-x1 subcommand.** That line was drawn at
     0.69.0-3 when the hardcoded cargo preflight was removed:
     vc-x1 assumes nothing about a repo's contents beyond
     `.jj` and `.vc-config.toml`.
   - Until it exists: run validating commands as separate
     invocations, never piping one into `tail`/`grep`, and
     never `&&` after a piped stage. `${PIPESTATUS[0]}` is
     the escape hatch when a pipe is genuinely wanted.
   - Split by ownership: the *runner* is project-local (the
     cargo cycle is Rust-specific), but the *rule* (a
     validation step's exit status is checked, not read)
     belongs in cycle-protocol.md's per-commit flow, which
     fans out to the template family.
4. **One home for a cycle's narrative: TODO during, chores
   at close-out.** Today the ladder and its detail are
   maintained in both `TODO.md > ## In Progress` and the
   chores `### As-built ladder` while a cycle runs, so every
   rung is written twice and every `Commits:` backfill lands
   in two files. Instead keep it all in `TODO.md` as a
   succinct working ladder with the detail in sections
   beneath it, linked locally, and at close-out move the
   whole block into `notes/chores/chores-NN.md` as the
   durable record.
   - Preserves what the per-commit convention was protecting:
     the narrative is still written while the work is fresh,
     just in one place.
   - Removes the dual backfill: commit refs are filled once,
     in the working ladder, and travel with it. The
     `Commits:` line then goes too: the ladder carries the
     same refs per rung *with* titles. Sections with no
     ladder (a single-commit interlude) keep it.
   - Watch first, decide after: the remaining rungs of the
     0.77.0 cycle are the sample. If close-out migration
     turns out to lose detail that per-commit capture kept,
     that is the argument against.
   - Touches AGENTS.md [Chores conventions] and
     cycle-protocol.md's Chores sections + Close-out, both
     shared with the template family, so it fans out.
5. **Remove `revert`, and `.vc-x1/` with it.** `revert`
   promises "undo the sync"; it restores the pre-sync `jj op`
   recorded in `.vc-x1/sync-state.toml`, which means "rewind
   the repo to that moment". The two coincide only while
   nothing has happened since: one commit later, revert
   would silently rewind that too, and nothing readable at
   revert time distinguishes the cases. We are not in control
   enough to do this reliably; jj's own `jj op log` /
   `jj op undo` is both safer and more informative, since it
   shows what is being undone before committing to it.
   - Confirm what revert actually restores (both repos?
     bookmarks only? full op state?) before deleting:
     `src/revert.rs`, `src/sync/state.rs`.
   - Delete the subcommand, sync's `sync-state.toml` write,
     and the docs/help text that describe them
     (`README.md`, `src/main.rs` help strings).
   - `.vc-x1/` then empties, since push's `push-state.toml` is
     retired by the stateless-push cycle, so the directory,
     `init`'s `/.vc-x1` `.gitignore` line, and any leftover
     `[push]` state config keys go too.
   - Existing workspaces: **never edit their `.gitignore`
     automatically.** Inspect it, and when the `/.vc-x1` line
     is found, report that it is no longer needed and leave
     the removal to the user: a report, not a rewrite. It is
     the user's file, and a stale ignore line is harmless.
     *When* the check runs (which surface, and how often)
     is TBD; `config --validate` and the proposed
     `validate-repo` are the candidates, and push's
     `check_gitignore_coherence` is not (it retires with the
     state file).
   - Cheap now, expensive later: few workspaces depend on it
     today.
6. **`squash-push --title` / `--body`.** `squash-push` amends
   content only: it folds the working copy into the last
   commit and force-updates the remote, but the commit keeps
   its existing message. Fixing a published commit's *message*
   is therefore two steps (`jj describe -r @-`, then
   `squash-push`). Accepting `--title` / `--body` makes it
   one.
   - No new risk: squash-push already rewrites a published
     commit and force-updates the remote. This only changes
     which part of the commit it edits.
   - **ochid handling: tell, don't force.** A user-supplied
     body drops the `ochid:` trailer unless it repeats it,
     which silently breaks the cross-repo link. vc-x1 should
     *not* inject the trailer (unlike `push`, which authors
     the message and stamps it; here the user authors it and
     the tool shouldn't rewrite their text). It should error
     when the new message loses a trailer the commit had,
     naming what would be lost, with an explicit override
     flag for the case where dropping it is intended.
   - The content-side guard is the precedent: squash-push
     already refuses a squash that would drop source-only
     trailers (the 0.65.1 ochid-loss incident). Same check,
     new input.
   - **The guard has a hole the flags would close.** Today the
     two-step workaround routes around the very check that
     protects the trailer: `squash-push` guards the squash
     path, `jj describe` guards nothing, so the workaround is
     strictly less safe than the feature. Hit at the 0.77.2
     amend (2026-07-29), where fixing that commit's own
     close-out bookkeeping meant editing content *and*
     message, and the trailer survived only by hand-copying
     it. `vc-x1 fix-desc` can repair a dropped ochid by title
     match, so the failure is recoverable, not silent-forever.
   - Amending a just-pushed commit is a real workflow, not a
     rare one: backfill lands one push later by design, so
     every commit has a one-push window where its SHA is
     cited nowhere and a rewrite costs nothing. Message fixes
     naturally cluster there, which is exactly where the
     two-step shape bites.
7. **Restructure templates: single template repo + fixed bot
   seed manifest.** Replace the separate
   `vc-x1-work-repo-template` + `vc-x1-bot-repo-template`
   repos with the one work-repo template, whose live
   `.claude/` doubles as the bot-side seed source; retire
   `vc-x1-bot-repo-template`. `vc-x1 init` / `clone` updates
   for the new layout. First up after the refactor program.
   - `--use-template` rule: explicit `CODE,BOT` copies all
     non-hidden files from BOT (unchanged, the escape
     hatch for rich bot seeds); `CODE` alone seeds the bot
     side from a fixed manifest (`LICENSE-*`, `README.md`)
     taken from `<CODE>/.claude/`. The `<CODE>.claude`
     sibling default is dropped.
   - The manifest is the safety property: a live `.claude`
     has non-hidden session artifacts at top level, and
     the known subset is what lets it double as the seed
     source without leaking session history into new
     projects.
   - Manifest members missing in the source are skipped, so
     a code template with no `.claude/` content yields a
     bare-but-valid bot repo (the bot template is
     optional; init already generates the true minimum
     itself).
   - `memory/MEMORY.md` moves from copied to generated:
     it is intentionally empty (seeded only because Claude
     tends to create it otherwise), so init emits it like
     `.vc-config.toml` instead of copying, leaving no "is it
     still empty?" invariant in the template.
8. **ochid: bot-repo location qualifier.** An ochid is
   workspace-relative (`/.claude/<chid>`), so nothing in a
   published commit says *where* the companion bot repo
   lives (vc-x1's is `github.com/winksaville/vc-x1.claude`,
   discoverable only by convention). Anyone cloning just the
   work repo can't resolve bot-side ochids. Design already
   sketched in forks-multi-user.md
   [Per-user bot repos via URL-shaped ochid](notes/forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid):
   URL-shaped trailers, plus the complementary
   `.vc-config.toml` repo-index form; resolver dispatch is
   one rule (URL -> fetch, else workspace-relative), existing
   path-form trailers stay the backward-compatible case.
   - Cheap first rung: declare the companion's URL once in
     the committed `.vc-config.toml` (no trailer-format
     change; any work-repo clone then knows where the bot
     repo lives). Rides naturally with the refactor
     program's facade-owns-topology stage
     (bot-repo-location config).
   - Link rot + mirroring mitigations are in the same doc
     section.
9. **Version-number protocol is fragile: versions are
   baked into titles/bodies/todo/done/chores before the
   change lands.** The cycle protocol embeds an `X.Y.Z-N`
   version in commit titles and bodies, `## Todo` /
   `## Done` entries, and chores headers, all written
   while the work is in progress, i.e. before it lands.
   But version numbers are subject to change: in a public,
   merge-based flow (e.g. Linux), the version a change
   ships under is only fixed when it merges into `main`,
   so the landing version can't be anticipated while the
   work is underway. Pervasive version-in-text is
   therefore fragile for any non-linear / multi-contributor
   workflow. Promoted from Ideas at 0.65.2-0; slated for
   the cycle after 0.65.2.
   - Live in-repo example (2026-07-24): 0.72.0 was
     pre-assigned to the trapezoid close-out cycle, which
     paused on `support-trapezoid-commits` after `-1`; the
     refactor program then ran 0.73.0+ directly off the
     0.71.0 main tip, leaving 0.72.0 a permanent gap, since
     renumbering either branch would rewrite cross-linked
     history. Disposition recorded in the
     [split push.rs stage](notes/refactor-20260716.md#stage-split-pushrs).
   - Related numbering thought (2026-07-24): program-shaped
     work could claim one minor and number its cycles
     `X.Y.1..n` (the jj refactor's seven cycles would have
     been 0.73.1..0.73.7), with program membership encoded in
     the version. Trade-off: a per-prep "is this a program?"
     call vs today's decision-free minor-per-cycle.
   - Open question: what identifies a cycle's commits if
     not a pre-assigned version?
     - Needs to be unique within some agreed upon domain.
       A contributors email address would do it, but also
       a UUID (short-version) for a contribution. I could
       imagine a UUID generated from the initial email/issue
       that and then "version number" schema appended to that.
   - Surfaces to update once the identifier is chosen:
     cycle-protocol.md (title shape, Numbering), AGENTS.md
     (commit-recording headers), and the `vc-x1` validators
     that parse `(X.Y.Z)` strings.
10. **sync follow-up: extract `move-bookmark` command.** The
    "put the bookmark / `@` where it belongs" step at the end
    of sync (reposition logic) is useful standalone (e.g. the
    t1B scenario where `main` is right but `@` isn't on it)
    and deserves an honestly-named command instead of a mode.
    - `vc-x1 move-bookmark` (name open): no fetch; move `@`
      (and optionally the bookmark) onto a target under the
      same safety rules as sync's reposition step.
    - Sync's final step becomes a call to the same logic.
    - Follow-up to the 0.67.0 single-mode sync cycle.
11. **sync follow-up: retire the hidden `--check` alias;
    revisit push's auto-rollback.** The first half of this
    entry (push shelling out to `vc-x1 sync --check`, which
    was racy and not actually read-only) is done: 0.77.0-3
    deleted preflight outright, taking the shell-out and its
    PATH dependency with it. What survives:
    - Remove sync's deprecated hidden `--check` alias. Nothing
      invokes it now except `tests/cli_sync.rs`'s alias test,
      so this became actionable the moment preflight went.
    - Push's commit-stage rollback auto-runs `jj op restore`,
      which hides the evidence of what failed. This cycle
      deliberately kept it, since an in-process snapshot taken
      moments earlier is knowledge, not a guess, and both
      index-lock failures during 0.77.0 cost nothing because
      of it. Revisit only with a concrete case where the
      hidden evidence mattered.
12. **validate-numbering: rename the pair, check all
    sequence-managed notes files generically.** `validate-todo`
    / `fix-todo` only operate on the single file passed, so a
    renumber slip in `bugs.md`, `todo-backlog.md`, or
    `TODO.md`'s `## Ideas` section passes unnoticed, too weak
    for a pre-commit gate. Prereq for the pre-commit doc
    validators (Todo "pre-commit: single rule ...").
    - Rename the pair: `validate-todo` -> `validate-numbering`,
      `fix-todo` -> `fix-numbering`, since they validate
      numbered-sequence integrity, not todos specifically.
    - Generic detection: for every `#...#` section, validate the
      column-0 `^\d+\.␠` entries form a contiguous 1..N run.
      Drops the Todo/Bugs special-casing; auto-covers
      `## Ideas` and any new numbered section. Keep the
      column-0 anchor so indented sub-lists aren't counted.
    - Default scope: a fixed list of sequence-managed notes
      files (`TODO.md`, `todo-backlog.md`, `bugs.md`) so the
      no-arg pre-commit run covers them all. Fixed rather than
      a `notes/**.md` walk because prose docs
      (`cycle-protocol.md`, design notes) carry ordinary
      numbered lists that aren't managed sequences, and a walk
      would false-positive (markdown renders `1. 1. 1.` as
      1-2-3, a legitimate prose pattern).
    - Override args follow the `--init-from` convention:
      positional files/dirs (a dir -> its `*.md`) plus an
      `@<file>` manifest, additive, for ad-hoc validation of
      a specific file.
    - Add wrapper-level tests while restructuring: the analyze
      cores are covered (`todo_helpers` 15 tests,
      `desc_helpers` 22) but the `validate-todo` / `fix-todo` /
      `validate-desc` / `fix-desc` wrappers have none: file
      I/O, output formatting, exit codes, and the no-arg
      default path (changed to `TODO.md` at 0.69.2-2) are
      unexercised.
    - Open: revisit fixed-vs-glob at implementation if the
      fixed list proves annoying to maintain.
13. **pre-commit: single rule (no docs skip) + doc validators.**
    The pre-commit (cargo cycle: fmt/clippy/test/install) only
    checks code, so it's "skip-able for purely-docs commits",
    but that exception is exactly where checks slip (skipped on
    0.62.0-7/-8 until caught). (Since 0.69.0-3 push's
    `preflight` no longer re-runs the cargo cycle, because
    vc-x1 assumes nothing about repo contents, the pre-commit is
    the *only* gate, strengthening the no-skip case.)
    - Adopt one rule, no exception: the pre-commit runs before
      Work review on every commit. (docs: AGENTS.md Cycle
      Protocol summary + cycle-protocol.md per-commit-flow.)
    - Enrich the pre-commit so it's meaningful on docs commits:
      add the doc validators, `validate-numbering` (its own
      Todo, a prereq) plus `validate-repo` when it exists.
      Whether push's `preflight` may run them needs a decision
      against the content-agnostic principle (they read
      `notes/`, which is repo content; the repo-declared-checks idea
      was rejected 2026-07-15 in favor of "run checks
      yourself").
    - This dissolves the docs exception: with doc validators in
      the pre-commit there's always something to validate, so
      the carve-out stops making sense.
    - Its own near-term cycle (chosen over a 0.61.1 insert to
      avoid rewriting published 0.62.0-x history); no version
      pre-assigned; see the Todo "Version-number protocol is
      fragile" on fragile version targets.
14. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
    Today push assumes 1:1 symmetric WC commits with shared
    title/body. The interop / adoption scenario breaks that:
    the code side is worked single-repo style (commit +
    `jj git push` / `git push`, no `vc-x1 push` in the loop),
    so no bot pairings exist. One bot commit then records
    every code commit not yet covered by a prior `ochid:`,
    via a multi-line `ochid:` per the design in [[5]].
    - Out of scope: the trapezoid close-out, handled
      natively by the in-progress "feat: push merge
      close-out (trapezoid)" cycle, whose N-ochid stamping
      also covers a cycle held local and published all at
      once. This Todo is only the no-bot-pairings interop
      case; the stamping step's multi-line `ochid:` emit is
      shared groundwork.
    - Teach push to:
      - detect the shape (code WC empty, uncovered commits at
        the bookmark)
      - skip `commit-app`
      - compose a `.claude`-specific message
      - emit one `ochid:` line per uncovered commit
    - Open: computing "uncovered", likely a revset from the
      code bookmark back to the newest commit referenced by
      the bot journal's ochids.
15. **Run validate-bot at every vc-x1 invocation
    (config-gated).** The check is one jj spawn
    (`jj bookmark list main --all-remotes`), cheap enough
    to run at every execution, noted 2026-07-15 as a
    "could, not should". Design points:
    - locate the bot repo (`<cwd>/.claude` or config;
      shares the lookup with the refactor program's
      [facade-owns-topology stage](notes/refactor-20260716.md#stage-facade-owns-topology))
      and silently skip when absent
    - severity knob in `.vc-config.toml`
      (`warn|error|off`): unrelated commands (fix-todo)
      warn at most; push / squash-push / validate-bot
      already have their own handling from 0.69.0-3
16. **CLI reference lives in `--help`; README owns concepts.**
    Each command is described in three places (clap's
    `long_about`, a README section with a flag table, and
    sometimes AGENTS.md) and only the flag *descriptions*
    self-sync, because those come from the field doc
    comments. Every hand-written block drifts silently:
    0.69.0-4 found the init section documenting retired
    `--owner` / `--dir` / `--repo-local`, and 0.77.0-3 found
    push's `long_about` still advertising a state machine
    that had just been deleted. The fix is removing the
    duplication, not auditing it on a schedule.
    - `--help` becomes the reference: what a command does,
      its stages, its flags, its invariants. It ships with
      the binary, so it always matches the binary being run.
    - README keeps workflows and concepts (the dual-repo
      model, the cycle, testing recipes, worked examples)
      and points at `--help` instead of restating flag
      tables. Delete the tables; that is the drift source.
      The `## Usage` block is the same species: its trailing
      `#` comments have drifted into three columns (40, 43,
      44) as commands were added, because the alignment is
      hand-maintained and invisible. Left unaligned at 0.77.2
      deliberately, since this entry deletes the block.
    - Clap reflows prose and collapses bullets unless a
      field carries `verbatim_doc_comment`, so help owns the
      reference, not the explanations. `long_about` does
      preserve explicit newlines (0.77.0-3's push stage
      list renders as an aligned two-column list).
    - Optional enforcement, cheapest first: assert README
      has no flag-table rows; snapshot-test `--help` output
      so unintended changes surface in review; or generate
      the reference from clap and assert the committed file
      matches. The third rhymes with "config: extract
      flag-backed key descriptions from Clap", the same
      single-sourcing shape.
    - Sweep each section against `vc-x1 <cmd> -h`.
    - Consider regenerating transcripts via support
      scripts (the gen-exmpl pattern) so examples stay
      reproducible.
17. **Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs.** Adopted in chores-13 (0.69.2 ladder,
    backfilled during 0.70.0-0): each rung is prepended
    with its commit reference so the rung↔commit
    correlation is direct; `Commits:` stays as the
    section-level list. The convention's home,
    cycle-protocol.md Close-out ("Add an `### As-built
    ladder`..."), is in the shared doc set
    (family: vc-x1, vc-x1-work-repo-template, iiac-perf, zc-msg-x1,
    tprobe), so landing it everywhere needs a coordinated
    family-wide sync. Not included in the 2026-07-20
    vc-x1-work-repo-template sync (straight copy); still pending for the
    whole family, vc-x1 included.
    - **Byte-identical is the goal, not the current state.**
      The set is diverged today and will stay that way while
      vc-x1 and iiac-perf churn; convergence is reachable only
      by a deliberate coordinated pass once both are stable.
      So a local edit to a shared doc is not a violation and
      does not need family sign-off, it just adds to what that
      pass will have to reconcile.
18. **Shared-doc sync: per-commit chores convention.**
    0.71.0 changed how chores are recorded: each work commit
    appends its As-built rung + narrative as it lands, rather
    than the narrative waiting for close-out. That wording edit
    was made locally in vc-x1's `cycle-protocol.md` / `AGENTS.md` (the
    shared doc set; see the byte-identical note on the
    As-built-rungs Todo above). vc-x1-work-repo-template synced
    2026-07-20 (AGENTS.md + cycle-protocol.md matching again, plus
    the TODO.md move); iiac-perf, zc-msg-x1, and tprobe still
    diverge, so the plan is to fan out from
    vc-x1-work-repo-template (same family as
    the Todo "Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs").
19. **config: extract flag-backed key descriptions from Clap.**
    `config`'s key descriptions live in `config_schema.rs`
    (`doc`/`used_by`). For the handful of keys that map 1:1 to a
    CLI flag (`bot-session.col-width` ↔ `--col-width`,
    `--result-lines`), the description could instead be pulled
    from the Clap arg's help via `Cli::command()` introspection,
    so `vc-x1 config` and `--help` share one source and can't
    disagree.
    - Only ~2 keys map cleanly (most are config-only, flag-sets,
      or value-providers), so it's a partial source and the
      schema stays authoritative for the rest.
    - Defaults still come from the schema/consts (the args
      dropped `default_value_t`, so Clap no longer holds them).
    - Output format is unchanged, only the text source, so no
      rework of the 0.71.0-9 rendering.
20. **typeable punctuation: source sweep + rule rewording.**
    The [Typeable punctuation only](/agent-data/prose.md#typeable-punctuation-only)
    rule says "Banned" and then says transcribed text keeps its
    characters. Both cannot be true. The rule prohibits
    *authoring*; presence in a file is legitimate, so the
    rewording comes first and bounds the sweep that follows.
    Deferred out of the `0.77.x` ladder 2026-07-30, behind the
    refactor program.
    - Reword: "never authored" in place of "banned", and the
      absolute clause scoped to text we write rather than to
      bytes on disk.
    - Sweep `src/` + `tests/`, ~875 sites across all four
      characters (655 em dash, 166 arrow, 39 ellipsis, 1 en
      dash). The retired ladder entry counted em dashes only,
      the same subset-audit defect the 0.77.2 close-out
      recorded one section earlier.
    - `config_schema.rs` `doc:` strings and the error/log
      messages are user-visible output, so the cargo cycle is
      mandatory and the four README `vc-x1 config` samples
      regenerate by hand after `cargo install`. No test asserts
      on any of the four characters.
    - `notes/` (~805 sites) and the chores archive (1965) stay
      out of scope under "converts when touched". The archive
      is thick with transcribed tool output and published
      commit titles that must not convert, and heading
      conversions move anchors the notes files link into.

## Ideas

 Items not yet solid enough for `## Todo` (or surfaced
 during close-out / end-of-day before they're fully
 formed). Triaged at the next Preparation: promote to
 `## Todo` / `notes/todo-backlog.md`, fold into a
 picked-up cycle, or drop.

1. **`vc` as a code+conversation provenance tool (grander
   ambition).** Today `vc-x1` manages a dual repo (code +
   `.claude`) cross-linked by `ochid:`. The larger aim is
   to *surface* that link: view history with the
   conversation and the code side by side, giving provenance, the
   *why* of a change, not just the *what*. The dual-repo +
   `ochid` design is already the substrate; the cross-links
   make code↔conversation navigable, so the viewer is UI
   over an already-solved data link.
   - Build direction: keep resolution/assembly in `vc`, an
     editor-agnostic Rust engine/lib extending the
     `show` / `chid` / `desc` family ("given a commit,
     resolve its ochid and assemble the paired diff +
     conversation slice"); the editor add-on is a thin
     presentation layer over it.
   - Front-end leans a Zed add-on (Rust, preferred), maybe
     VSCode / other. Verify Zed's extension API can host a
     rich side-by-side panel before committing; an
     editor-agnostic core hedges the bet.
   - `vc-x2`? A rewrite is unwarranted: the audit's
     Commonality pass found the architecture sound (por is
     bolted on where an existing good pattern wasn't
     applied), so equalize incrementally. "vc-x2" only makes
     sense if the viewer changes the *core* architecture
     (an index / daemon / data model). Separate
     engine-rewrite (no) from product-reposition (open).
   - Possible artifact: a top-level
     `notes/design-cli/vision.md` framing the direction,
     with the parity and conversion docs as sub-designs.
2. **Restructure the design-cli parity docs (target
   0.63.0).** `por-dual-parity-audit.md` (~1200 lines)
   fuses a *frozen* audit (the `## 1`-`## 8` snapshot
   evidence) with a *living* design (axes, decisions,
   matrix, gap list); the "audit" name undersells it and
   the halves have different lifecycles. And
   `por-dual-parity.md` (the stub) overlaps on parity but
   uniquely holds the `por ↔ dual` conversion design.
   - Split the audit doc into a frozen audit snapshot + a
     living design doc (names TBD; could reclaim
     `por-dual-parity.md` for the design).
   - Refocus the stub to conversion-only and rename (e.g.
     `por-dual-conversion.md`); drop its redundant parity
     half.
   - Repoint refs (`todo.md` `[1]` + the `por -> dual` Todo,
     `copying.md`, the audit's internal anchors + Reading
     guide) and validate; `chores-10/11/12` mentions are
     historical and stay.
   - Promote the Gap-list items to anchored
     `#### Gap N: <title>` sub-headings so cross-cycle
     citations can deep-link a specific gap (markdown
     anchors headings, not list items). Trade-off: stable
     anchors, but the ordinal lives in the heading text
     (manual renumber on reorder), fine for a consumed
     backlog. The 3 `Gap #N` links in the `0.62.0`
     close-out chores narrative resolve only to the section
     until this lands.
   - Deferred from the 0.62.0 close-out: close-out is
     bookkeeping-only, and the split is substantive,
     anchor-heavy work warranting its own cycle.
3. **Chores retire into a session index (post-viewer).**
   Once the provenance viewer ("`vc` as a code+conversation
   provenance tool" above) can present a commit's session
   and code side by side, the hand-written chores narrative
   is a distillation of a conversation the bot repo already
   records verbatim, so the DRY argument that removed edit
   lists from chores (git owns the mechanics) then applies
   to the narrative too (the session owns it). Chores
   collapses to an index into the session.
   - The `ochid:` trailer links a work commit to a session
     *commit*; the index adds within-session granularity:
     which conversation span produced the commit, where the
     design argument happened. We think it can be generated
     (the transcript records when pushes happen), making it
     drift-proof where hand-written chores never were.
   - What survives: the curated design layer (the
     refactor-20260716.md pattern). Sessions are an
     immutable journal, good as record and poor to cite
     into, so live design references keep pointing at
     curated docs, not per-cycle narrative sections.
   - The template side already points this way: chores
     files are not seeded; a new project's history is its
     own commits + bot session from day one.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

_Migrated to [done.md](notes/done.md) on 2026-07-24 (the DRY jj facade
cycle and its two docs interludes: template repo names, notes rework)._

- build: bump jj-lib to 0.43: the local `jj` moved to 0.43.0,
  leaving the pin two releases behind. `use_glob_by_default`
  is gone from `RevsetParseContext` and `commit_change_ids()`
  returns a stream rather than an iterator, so `futures` joins
  the direct dependencies. The bump also moved the default
  revset string-pattern kind from substring to glob, which the
  compiler could not report and which no revset of ours uses
  [[19]]

- docs: re-describe rule + defer punctuation sweep: `jj
  describe` on a published or already-stamped commit is a
  history rewrite that silently drops the `ochid:` trailer, so
  it is coordinate-first; the sub-cycle ladder is the named
  exception, being local until its single Close-out push. Two
  planned `0.77.x` rungs retired, the source sweep to `## Todo`
  and interlude shape to the backlog [[17]]

- docs: typeable punctuation: `—`, `–`, `…` and `→` no longer
  authored in durable text, and the 553 sites in the five prose
  files converted. None can be typed at a terminal, so none can
  be grepped for, and an em dash next to option syntax reads as
  another flag. The rule sorts a character by the role it plays
  (naming it, doing a job, transcribed from outside) and warns
  that converting a heading moves its anchor. Scope covers
  commit titles and `src/`; that sweep is deferred to `## Todo`
  [[14]]

- docs: jj-lib design notes + trapezoid recipe: the op-store
  coexistence risk answered from jj-lib 0.41 source against an
  installed jj 0.40.0 (unenforceable, low blast radius, so a
  decision rather than a step 0.78.0 assumes), and the
  trapezoid recipe corrected where the 0.77.0 close-out found
  it wrong: step 4 is `jj git push`, not `vc-x1 push` [[15]]

- refactor: stateless push: push keeps no state and cannot
  resume: the state file, `--restart` / `--from` / `--status`,
  the stale-state verifier, the `[push]` config keys and the
  `.gitignore` coherence check are gone, and so is preflight,
  whose `sync --check` self-spawn forced the tests'
  stage-skipping flag. What replaces them is a property:
  every stage no-ops when its work is already done, so a
  failed run is re-run rather than resumed. bugs.md #3 is
  unrepresentable and #4 is fixed; `push.rs` 1480 -> 816
  lines, ~940 net removed; fifth stage of the jj refactor
  program [[6]]

- docs: trapezoid close-out recipe: the four steps that
  publish a trapezoid close-out consolidated into one
  definitive procedure in cycle-protocol.md (base rule,
  the two-parent verification, the sideways-move backfill
  embargo, recovery), with the refactor stage keeping only
  implementation deltas and README waiting for the flag;
  vocabulary collapsed to "trapezoid"; chores-15 opened
  [[7]]

- refactor: repo registry: `.vc-config.toml`'s `[workspace]`
  block becomes a `[repos]` registry: ordinary file-relative
  (or absolute) paths, side detection by self-resolution,
  resolved agreement + self-identification replacing the
  identical-block invariant, and ochid prefixes as canonical
  side labels decoupled from the bot dir's spelling; legacy
  reads consolidated in `src/legacy_vc_config.rs` for later
  retirement. De-gitify init rode as the last rung, so init's
  publish path is jj-only and bugs.md #1/#2 are fixed; fourth
  stage of the jj refactor program [[8]]

- docs: refactor program ladder + conventions: the refactor
  program's remaining stages consolidated into four cycles
  and laid out as a program ladder under a new heading-based
  `## In Progress` shape; the parked 0.72.0 branch declared
  quarry (version gap accepted); the version-first-bullet
  body convention added to cycle-protocol.md; chores-14
  0.75.0 rung refs backfilled [[9]]

_Migrated to [done.md](notes/done.md) on 2026-07-28 (the
hygiene-riders and facade-owns-topology cycles)._

# References

[1]: https://github.com/winksaville/vc-x1/commit/b5e40e7458b8 "b5e40e7458b8506574b2ae01f52f7ccae9023418"
[2]: https://github.com/winksaville/vc-x1/commit/946dc964b75c "946dc964b75ca29e2cc4b6c59f03aec2c364feee"
[3]: https://github.com/winksaville/vc-x1/commit/dc14a421d850 "dc14a421d8509e58fa05741fd1a868329540731e"
[4]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[5]: /notes/forks-multi-user.md
[6]: /notes/chores/chores-15.md#refactor-stateless-push
[7]: /notes/chores/chores-15.md#docs-trapezoid-close-out-recipe
[8]: /notes/chores/chores-14.md#refactor-repo-registry
[9]: /notes/chores/chores-14.md#docs-refactor-program-ladder--conventions
[10]: https://github.com/winksaville/vc-x1/commit/9d6f7c0b0f05 "9d6f7c0b0f05ae74dd7100d457b92b72d913404f"
[11]: https://github.com/winksaville/vc-x1/commit/3be698fcde83 "3be698fcde831b09949077e1ce934839ee01f4ea"
[12]: https://github.com/winksaville/vc-x1/commit/eb4a12eb3b56 "eb4a12eb3b561234d176953d3773960fb9f4cdaa"
[13]: https://github.com/winksaville/vc-x1/commit/2424e14f858d "2424e14f858d010e5c07e8821149a114b3d3dda5"
[14]: /notes/chores/chores-15.md#docs-typeable-punctuation
[15]: /notes/chores/chores-15.md#docs-jj-lib-design-notes--trapezoid-recipe
[16]: https://github.com/winksaville/vc-x1/commit/62d71818d78b "62d71818d78bc06ae8f5cc17ca060d30a08b6ea1"
[17]: /notes/chores/chores-15.md#docs-re-describe-rule--defer-punctuation-sweep
[18]: https://github.com/winksaville/vc-x1/commit/03df811a72fe "03df811a72fe61bdd013e34961e72aecd671c126"
[19]: /notes/chores/chores-15.md#build-bump-jj-lib-to-043
[20]: https://github.com/winksaville/vc-x1/commit/0cf200b9b3eb "0cf200b9b3eb2ad652b99e518edcdfe69b657075"
[21]: https://github.com/winksaville/vc-x1/commit/a2dbf57d8a2e "a2dbf57d8a2e64f5ae8cdc29bd1621b157881bdc"
[22]: https://github.com/winksaville/vc-x1/commit/84cec8c17610 "84cec8c176108dc7416570b70d62b85fc86c6049"
[23]: https://github.com/winksaville/vc-x1/commit/685ca885e1e0 "685ca885e1e09d381ac7897a94e5f2da77b17fc8"
[24]: https://github.com/winksaville/vc-x1/commit/deec79d0e75d "deec79d0e75de6106f6f8919b77844eb8afe4c83"
[25]: https://github.com/winksaville/vc-x1/commit/e4203c6d3679 "e4203c6d36799cb2dd8b6ff0eb8ddf9f64522aa2"
[26]: https://github.com/winksaville/vc-x1/commit/6c67ce0f4eb0 "6c67ce0f4eb0df2e9388ab84aca0d728f0d5f976"
