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

#### Ladder

- [fix: a missing fixture fails instead of passing opening][1] (done)
- [docs: plan the fixture change][2] (done)
- [fix: an absent fixture panics naming how to get it][3]
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

##### fix: an absent fixture panics naming how to get it

`test_helpers::fixture` returns a path or panics, the message naming the `[test] fixtures` key, the
path it looked at, and what a fixture is, and `dr1()` in `src/lookup.rs` returns its tuple rather
than an `Option`.

##### test: fixture tests are ignored without the fixture

The `dr1_*` tests take `#[ignore = "needs fixture dr-1"]`, `[validate] full` runs
`cargo test -- --include-ignored`, and the acceptance check is run both ways.

##### fix: a missing fixture fails instead of passing closing

Closing out the cycle.

# References

[1]: #fix-a-missing-fixture-fails-instead-of-passing-opening
[2]: #docs-plan-the-fixture-change
[3]: #fix-an-absent-fixture-panics-naming-how-to-get-it
[4]: #test-fixture-tests-are-ignored-without-the-fixture
[5]: #fix-a-missing-fixture-fails-instead-of-passing-closing
