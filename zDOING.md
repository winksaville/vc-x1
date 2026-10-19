# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

_None._

## In Progress

### fix: a missing fixture fails instead of passing

(wink + agent, 2026-10-17, opened 2026-10-18) `test_helpers::fixture` returns `None` when the
fixture's directory is absent, so its tests return early and pass, the notice on stderr hidden,
since `cargo test` shows a passing test's output only under `--nocapture`. With `../vc-x1-fixtures`
renamed to `../vc-x1-fixtures-xxx`, a `cargo clean` and `cargo test` passed, and the `dr1_*` tests
in `src/lookup.rs` checked nothing.

#### Problem

A test whose fixture is missing passes, so a bare clone's `cargo test` reports five checks it never
made, and the only sign is a line on stderr that `cargo test` hides.

#### Solution

An absent fixture is a panic that names how to get it, the fixture tests are `#[ignore]` so a bare
clone's `cargo test` reports them as ignored, and `[validate] full` runs them with
`--include-ignored`.

#### Acceptance check

With `../vc-x1-fixtures` renamed away: `cargo test` passes and reports the `dr1_*` tests as
ignored, and `cargo test -- --include-ignored` fails with the panic naming the path it looked at.
With the directory restored: both pass.

#### Deliberation

- Multi-step, five rungs, a trapezoid at the close, wink's vote: opening, a plan rung, the fix,
  the tests, closing. The work is small enough for one commit, and the point of this cycle is the
  ladder itself.
  - This cycle is the first run under the agent-files v0.3.0, an experiment on the unlanded
    proposal, wink's framing. Its bookmark starts from the proposal commit, not from `main`, so the
    session works under the v0.3.0 files on disk, and the commit lands behind the proposal when
    that lands. An experiment of the set, not a bend of a rule.
- The plan rung writes the plan into this record before any code changes, so the fix and the tests
  are reviewed against something written down.
- Ahead of **Fixtures are named by their source**, being small and closing the silent pass on its
  own.
- The record is kept here, in `zDOING.md`, per the trial this file describes, while the agent-files
  still name `TODO.md > ## In Progress`.
- Waiver (wink, 2026-10-18, "Permission to complete through -3 I'll be back to review"): the
  bookmark push and the pushes of the opening, the plan rung, the fix rung, and the tests rung are
  approved in advance, their work and description reviews with them, since the user is away. The
  closing rung's reviews and push, the close-out shape, and Land are outside it.
  - Extended (wink, 2026-10-18, "permission to complete the ladder through the new [4]") after
    the review of the fixture-workspace rung: its push, the rebase of the fix and tests rungs onto
    it with their version amendments, and the bookmark's re-push. The closing stays outside.

#### Ladder

- [fix: a missing fixture fails instead of passing opening][0] (done)
- [docs: plan the fixture change][1] (done)
- [feat: the fixture lives in the workspace][2] (done)
- [fix: an absent fixture panics naming how to get it][3] (done)
- [test: fixture tests are ignored without the fixture][4]
- [fix: a missing fixture fails instead of passing closing][5]

##### fix: a missing fixture fails instead of passing opening

The cycle's setup commit: create and publish the bookmark from the proposal commit, move the Todo
entry into this block, and bump the version-of-record.

##### docs: plan the fixture change

The plan, written into this record before any code changes, so the fix and the tests are reviewed
against something written down.

* `test_helpers::fixture` keeps a missing fixture quiet.
  - It returns `PathBuf` and panics when `[test] fixtures` is missing from `.vc-config.md` or the
    directory it names has no `<name>/.jj`. The message names the fixture, the path it looked at,
    the `[test] fixtures` key that set it, and where the fixture comes from: built by
    `support/fixtures/build-dr-1.py`, published on GitHub as `vc-x1-fixtures-dr-1-work` and
    `-agent`. It is test-only code, so the panic is the failure it should always have been.
  - `dr1()` in `src/lookup.rs` returns `(PathBuf, Value)`, and its five callers drop their
    `let Some(..) = dr1() else { return }`.
* A bare clone's `cargo test` must still pass, and say what it skipped.
  - The five `dr1_*` tests take `#[ignore = "needs fixture dr-1"]`, so `cargo test` reports them
    as ignored, with the reason, and never as passed.
  - A test of the helper itself, `fixture("no-such-fixture")`, is `#[should_panic]` on the message's
    "absent at", so the failure path is exercised without any fixture.
* The full validation must run them.
  - `.vc-config.md` `[validate] full` runs `cargo test -- --include-ignored` in place of
    `cargo test`, and `fast` keeps `cargo test`, so a rung's fast run needs no fixture and the full
    run before a push does. The ignored tests are run, and an absent fixture fails the run with the
    message above.
* On this machine the fixtures directory is renamed, `../vc-x1-fixtures-xxx`, the Todo's own
  experiment, so the tests rung would fail its full validation as the Todo intends.
  - The tests rung makes `../vc-x1-fixtures` a symlink to the renamed directory for its validation
    and the acceptance check's second half, removes it after, and says so. Whether the directory
    is renamed back is wink's call at the review, as is whether the symlink should have been a
    rename.

Rungs after this one: the fix rung is the first two facets' code, the tests rung is the third and
fourth, with the acceptance check run both ways and recorded at the closing.

##### feat: the fixture lives in the workspace

A fresh clone has no fixture, and the committed config named a sibling directory on one machine,
`../vc-x1-fixtures`, so where a fixture sits was a fact about wink's disk and nothing said how to
get one.

* The config named a path outside the workspace.
  - `[test] fixtures` is `.fixtures`, inside the workspace and gitignored, so any clone builds or
    fetches its own there and the committed file names no machine. The source-named config the Todo
    **Fixtures are named by their source** describes replaces the key later and keeps the place.
* The build script could not run against today's `vc-x1`.
  - `build-dr-1.py` passed `init` a slashed relative path, which init now refuses as the retired
    owner/name shorthand, and relied on the agent repo being `.claude`, which init's default moved
    away from. It passes an absolute path and `--agent-dir .claude`, and `build-dr-1.py .fixtures
    --local` built the fixture here in one run, the `vc-x1 0.85.3` on this machine driving it.
  - The build is reproducible in shape, not in ids: its change ids differ from the September
    copy's, and its `relationships.json` carries its own, so the fourteen `dr1_*` tests pass
    against it as they did against the old one. The published pair, once pushed, is the one with
    fixed ids.
* The helper's message still names the old place.
  - The fix rung's panic message, when it is rebased onto this rung, says to build into `.fixtures`
    with the script, or clone the published pair, and stops claiming the pair is on GitHub until it
    is: the two repos exist there and are empty (checked 2026-10-18).
* Inserting this rung reorders a pushed ladder, wink's call.
  - The bookmark moved back to the plan commit, this rung is `-2`, and the fix and tests rungs are
    rebased onto it as `-3` and `-4`, their change ids and so their `ochid:` pairs kept, their
    `Cargo.toml` versions amended. The cost is that the work-repo's ladder is no longer in time
    order while the agent-repo's history is, and the Todo **lookup is tested on a reordered ladder**
    records the before state and the hypotheses to test on this ladder before it lands.
  - The ladder's reference labels start at `[0]` so each equals its rung's version suffix, a local
    experiment under v0.3.0's "Local experiments", since a rung is only ever inserted at the current
    position and so the Nth label is always the `-N` commit. The set's specimen starts at `[1]`.

##### fix: an absent fixture panics naming how to get it

`test_helpers::fixture` returned `None` with a line on stderr when the fixture was absent, and every
caller returned early and passed.

* The helper's `Option` made absence a normal return.
  - It returns `PathBuf` and panics on a missing `[test] fixtures` key or a `<name>/.jj` that is not
    there. The message names the fixture, the path, the key, and where the fixture comes from, the
    `build-<name>.py` script and the two GitHub repos, so the message serves the next fixture too.
  - `dr1()` returns its tuple, and its callers lose their early return, so a test either runs its
    checks or fails in the helper, and the stderr notice that `cargo test` hid is gone. The plan
    counted five callers, the ones in `src/lookup.rs`, and the compiler found fourteen, the rest in
    `src/lookup/{blame,partner,window}.rs`, so the silent pass was wider than the Todo measured.
* Until the tests rung marks them `#[ignore]`, the fourteen tests fail wherever the fixture is
  absent, this machine included.
  - The symlink the plan gave the tests rung is made at this rung instead, since this rung's
    validation needs it first. It stays through the tests rung and goes after it.

##### test: fixture tests are ignored without the fixture

The `dr1_*` tests take `#[ignore = "needs fixture dr-1"]`, `[validate] full` runs
`cargo test -- --include-ignored`, and the acceptance check is run both ways.

##### fix: a missing fixture fails instead of passing closing

Closing out the cycle.

# References

[0]: #fix-a-missing-fixture-fails-instead-of-passing-opening
[1]: #docs-plan-the-fixture-change
[2]: #feat-the-fixture-lives-in-the-workspace
[3]: #fix-an-absent-fixture-panics-naming-how-to-get-it
[4]: #test-fixture-tests-are-ignored-without-the-fixture
[5]: #fix-a-missing-fixture-fails-instead-of-passing-closing
