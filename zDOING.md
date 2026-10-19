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

### test: the tests build the dr-1 fixture

(wink + agent, 2026-10-19, opened 2026-10-19) A fresh clone's full test run fails all 14 `dr1_*`
tests, each with the same panic, and the remedy needs Python. The noise hides the one cause, and
Python is a dependency the project does not otherwise have, nor one Windows and macOS can be
assumed to have.

#### Problem

The `dr-1` fixture is built only by `support/fixtures/build-dr-1.py`, which needs `python3` and
drives whatever `vc-x1` is on the PATH rather than the code under test. A clone without it fails
the fourteen `dr1_*` tests one by one with the same panic, and cargo's rerun hint after them, so
the one cause is buried under fourteen copies of it.

#### Solution

Provisional, the Todo's plan as filed:

- Port `support/fixtures/build-dr-1.py` to Rust as test-only code, compiled under `#[cfg(test)]`,
  so it never ships. It drives the crate's own init and push code in-process and jj-lib directly,
  with commit times pinned, so no `python3`, `jj`, `git`, or installed `vc-x1` is involved, and the
  fixture is built by the code under test. The script's `git apply` becomes a file write.
- `fixture` in `src/test_helpers.rs` builds the fixture once per test process into a fresh temp
  dir, its result held in a `OnceLock`. Every test that needs `dr-1` asks through it, so whichever
  asks first builds it, and no test has to be named to sort first. The rest park until the build
  ends, then run in parallel.
- The builder returns its failure as a value, never a panic, since a panic leaves the `OnceLock`
  empty and the next waiter builds again.
- A failed build stops the run: one banner written straight to the stderr handle, which bypasses
  the harness's output capture, then `std::process::exit`, so the cause is the last thing before
  cargo's two lines.
- The `#[ignore]`s go, so `cargo test` runs the 14 tests, and the fast and full `[validate]`
  lists differ only in what remains, their comment in `.vc-config.md` rewritten.
- The temp dir records no absolute path that must survive a move, and the fixture makes no
  symlink and goes through no checkout, so `core.autocrlf` cannot rewrite its lines.
- `VC_X1_TEST_KEEP` keeps the built fixture for a look by hand.
- Delete the Python script, and with it the `[test] fixtures` key and the gitignored `.fixtures/`.
  The two empty GitHub fixture repos are wink's to remove.
- README.md's "Acceptance fixtures" section is rewritten. It is already stale: it names
  `../vc-x1-fixtures` and says a test skips when its fixture is absent.

#### Acceptance check

In a fresh `vc-x1 clone` at the bookmark's tip, with `python3` off the PATH, `cargo test` runs the
fourteen `dr1_*` tests, none ignored, and they pass. With the builder forced to fail, the run
prints the banner once and stops, with no per-test failure list.

#### Deliberation

- Multi-step, six rungs, a trapezoid the default, the agent's proposal and wink's go at the
  bookmark: opening, a plan rung, the port, the helper, the retirement of the Python builder,
  closing.
  - The port is the bulk, and the plan rung maps each step of the script to the crate call or
    jj-lib call that replaces it before any code changes, so the port is reviewed against
    something written down.
- Rust, not Python, wink's call: no dependency the project does not otherwise have, and Windows
  and macOS in view.
  - The Windows and macOS points are design intent until a CI run on them checks it: line
    endings, symlinks, path shapes, case-insensitive filesystems.
- Built by the tests, not by `build.rs`: a build script cannot tell `cargo test` from
  `cargo install`, runs before the crate it would need is compiled, and sees only
  build-dependencies.
- A fresh temp dir per test run, not a cached `.fixtures/dr-1`: the build is deterministic and
  took about two seconds under Python, so caching buys little and costs a version stamp, a
  build-then-rename, and a race between concurrent runs.
- Every user asks, no gate test, wink's point: a gate named to sort first is fragile, and every
  `dr1_*` test already asks through `dr1()`, so whichever asks first builds. What is given up is
  only an earlier stop.
- The failure stops the run, wink's ask: fail fast, front and center.
  - Prototyped 2026-10-19 in a scratch crate: the banner showed through the output capture, a
    running test was cut off, and a filtered run that needs no fixture was untouched.
  - The `ctor` crate's before-main hook was weighed and set aside: a new dependency, jj-lib run
    before the runtime is fully set up, and a build even for filtered runs.
- Fixtures that cannot be generated, such as the real-history ladder **lookup is tested on a
  reordered ladder** wants, stay with **Fixtures are named by their source**: fetched, and an
  absent one a loud stop with the remedy.
- Seeding jj's randomness, open: the Python build does not, so change ids differ per build, which
  is fine while the relationships file is generated beside the fixture. The plan rung decides
  whether a byte-reproducible fixture is worth having.
- The record is kept here, in `zDOING.md`, per the trial this file describes, and the ladder's
  labels start at `[0]`, continuing the local experiment the last cycle began.

#### Ladder

- [test: the tests build the dr-1 fixture opening][0] (done)
- [docs: plan the Rust fixture builder][1]
- [test: dr-1 is built in Rust][2]
- [test: the dr-1 tests build their fixture][3]
- [chore: the Python fixture builder goes][4]
- [test: the tests build the dr-1 fixture closing][5]

##### test: the tests build the dr-1 fixture opening

The cycle's setup commit: create and publish the bookmark at `main`, move the Todo entry into this
block, bump the version-of-record, and rename the package to `vc-x1-dev`.

##### docs: plan the Rust fixture builder

The script's steps, each mapped to the crate code or jj-lib call that replaces it, written here
before any code changes, so the port is reviewed against a plan.

##### test: dr-1 is built in Rust

The builder, ported to test-only Rust, builds the fixture and its relationships file into a temp
dir, with a test that the fixture it builds is complete.

##### test: the dr-1 tests build their fixture

The fixture helper builds the fixture once per test process and stops the run with one banner when
the build fails, and the fourteen tests lose their `#[ignore]`.

##### chore: the Python fixture builder goes

The Python script, the `[test] fixtures` key, and `.fixtures/` go, and the README's fixture section
says how the fixture is made now.

##### test: the tests build the dr-1 fixture closing

Closing out the cycle.

# References

[0]: #test-the-tests-build-the-dr-1-fixture-opening
[1]: #docs-plan-the-rust-fixture-builder
[2]: #test-dr-1-is-built-in-rust
[3]: #test-the-dr-1-tests-build-their-fixture
[4]: #chore-the-python-fixture-builder-goes
[5]: #test-the-tests-build-the-dr-1-fixture-closing
