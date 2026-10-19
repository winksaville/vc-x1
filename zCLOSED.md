# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### test: the tests build the dr-1 fixture

(wink + agent, 2026-10-19, opened 2026-10-19, closed as a dead end 2026-10-20) A fresh clone's
full test run fails all 14 `dr1_*` tests, each with the same panic, and the remedy needs Python.
The noise hides the one cause, and Python is a dependency the project does not otherwise have, nor
one Windows and macOS can be assumed to have.

#### Problem

The `dr-1` fixture is built only by `support/fixtures/build-dr-1.py`, which needs `python3` and
drives whatever `vc-x1` is on the PATH rather than the code under test. A clone without it fails
the fourteen `dr1_*` tests one by one with the same panic, and cargo's rerun hint after them, so
the one cause is buried under fourteen copies of it.

#### Solution

A dead end, abandoned at the plan rung's review before any of the port began.

- Tried: the plan as filed, below, as one cycle of nine rungs: a plan rung mapping the port of
  the Python builder piece by piece, four port rungs by layer, the tests building their fixture,
  and the Python builder's retirement.
- Stopped: the plan rung's map read only through the Python builder, and the nine rungs were too
  much to hold at once. The plan rung was never pushed, so it leaves the ladder.
- Replaced by the `## Todo` entry **dr-1 is built in Rust**, at the top, its plan the layers
  as suggestions, run as rungs of one cycle or as cycles of their own, decided when it opens.
- Kept: [notes/fixture-dr-1.md](notes/fixture-dr-1.md), what the fixture holds and the case each
  of its cycles exists for, with no reference to how it is built, and a pointer to it from
  `notes/README.md`. Also what the cycle raised in `## Todo`: **push checks the description's
  widths**, and the dead-end item of **agent-files(proposal): v0.3.1**.
- Reverted: nothing. The opening changed only this record, the version, and the package name,
  and Land restores the name as for any cycle.

The plan as filed, which **dr-1 is built in Rust** inherits:

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

- Result: not run. The cycle stopped before any of the work the check covers, and the check passes
  to **dr-1 is built in Rust**, the Todo replacing this cycle.

#### Deliberation

- Multi-step, nine rungs, a trapezoid the default: opening, a plan rung, the port in four rungs,
  the tests building their fixture, the retirement of the Python builder, closing. Six at the
  bookmark, the agent's proposal and wink's go, then the port split in four at the plan rung's
  review.
  - The port is split by layer, wink's question and the agent's proposal: the transcript writer,
    the jj plumbing, the agent with its first cycle, then the other cycles and the record, each
    a rung tested before the next starts. The first plan mapped the whole Python builder in one
    piece and was hard to read for anyone who had not read the Python.
  - A port one function at a time with the rest left in Python was weighed and set aside: the
    functions share the clock, the transcript counters, and the record in memory, so each step
    would pass that state across the language boundary through files, and the Rust side is
    test-only code Python cannot call. All of it would be thrown away at the end.
  - A separate cycle for the port was weighed and set aside: the ladder already gives each layer
    its own review, the rungs past the opening were unpushed so the reshape cost nothing, and the
    cycle's title stays true.
  - While the Python builder is in the tree, each port rung compares its layer's output with the
    Python one's, ids, absolute paths, and key order aside, a throwaway check recorded in the
    rung, since a committed test cannot need Python.
- Closed as a dead end at the plan rung's review, wink's call (2026-10-20): even split by layer,
  the plan read only through the Python builder. The work returns to `## Todo` as one entry, the
  goal, with the layers its suggested plan, each designed when it starts.
  - Five entries, one per layer, were weighed and set aside, wink's call: they are steps of one
    goal, none stating a problem of its own, and each opened one would leave the others citing an
    entry that is gone.
  - The opening was pushed and its agent-repo commit's `ochid:` trailer names it, so the cycle
    closes and lands rather than being dropped with its bookmark, the case the dead-end item of
    **agent-files(proposal): v0.3.1** was filed for.
  - Re-describing the opening as the first layer's was weighed and set aside: the agent-repo's
    opening would keep the old title, an exception to record, where a dead end bends nothing.
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
- Unseeded, decided at the plan rung: change ids differ per build, which the generated
  relationships file absorbs, and byte-reproducibility is out of reach anyway, since the
  transcripts carry the build directory's absolute paths.
- The jj CLI stays for the builder's repo operations, a deviation from the solution as filed,
  which said no `jj` or `git` would be involved. The record is the truth the lookup tests check
  lookup against, and lookup is built on jj-lib, so the record is read with the CLI. The tests
  already need it, and git runs only as jj's own push subprocess, which jj-lib needs too. What goes
  is `python3` and the installed `vc-x1`.
- The in-process calls get their chosen time and committer, and no signing, from a test-only
  override of one thread's jj settings, since a test may not write the process's environment and
  in-process jj-lib otherwise reads the developer's jj config, signing key included. Found at the
  plan rung's review, when a sandbox hid the key and a test's in-process `init` failed to sign.
- The record is kept in `zDOING.md` per the trial that file describes, and the ladder's labels
  start at `[0]`, continuing the local experiment the last cycle began.

#### Ladder

- [test: the tests build the dr-1 fixture opening][0] (done)
- [test: the tests build the dr-1 fixture dead-end closing][1] (done)

##### test: the tests build the dr-1 fixture opening

The cycle's setup commit: create and publish the bookmark at `main`, move the Todo entry into this
block, bump the version-of-record, and rename the package to `vc-x1-dev`.

##### test: the tests build the dr-1 fixture dead-end closing

Closing out the cycle as a dead end, the first under the dead-end item of
**agent-files(proposal): v0.3.1**. What closing taught:

* A dead end's title says so.
  - The closing is the cycle title plus " dead-end closing", wink's rule for
    **agent-files(proposal): v0.3.1**, run here ahead of it. By the same rule the title cap does
    not bind the suffix, so this title's 55 characters stand.
* The plan rung leaves no commit.
  - It was never pushed, so the ladder is the opening and this closing. Its one lasting product,
    `notes/fixture-dr-1.md`, lands here, and its subsection, the port mapped against the Python
    builder, survives only in the agent-repo's transcript.
* Version: the closing carries `0.85.5`, and Land releases it, since `main` gains the note and the
  Todo entries. Whether a dead end should release nothing stays open in **agent-files(proposal):
  v0.3.1**.
* Close-out shape: trapezoid, the dead-end form, reshaped at Land. The closing becomes the merge
  on `main`, so the opening stays reachable through its second parent and its agent-repo trailer
  resolves.

# References

[0]: #test-the-tests-build-the-dr-1-fixture-opening
[1]: #test-the-tests-build-the-dr-1-fixture-dead-end-closing
