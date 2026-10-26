# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### fix: the fixture builder drives commit

Not a `## Todo` entry: found at the Land of **feat: vc-x1 commit takes a revision** (2026-10-26),
when full validation on fwlaptop failed for want of `.fixtures/dr-1`.

#### Problem

`support/fixtures/build-dr-1.py` drives `vc-x1 push <bookmark> --yes --title ... --body ...` and
`vc-x1 squash-push -R <agent>`, two forms that went on 2026-10-20 with **feat: vc-x1 commit,
install, and run-dev**, so the dr-1 fixture cannot be built. Nothing noticed, since the fourteen
`dr1_*` tests read a built fixture and never run the builder: 3900x still holds one built on
2026-10-18, and fwlaptop, whose clone has none, is where the build was first needed.

#### Solution

- The builder commits each rung with `vc-x1 commit -s b --bookmark <bookmark>`, and squash-pushes
  the agent repo with `-s a --yes`.
- A test runs the builder against the vc-x1 the test build makes, so a change to the commands it
  drives fails validation the day it is made.

#### Acceptance check

With `.fixtures/` removed, `python3 support/fixtures/build-dr-1.py .fixtures --local` builds the
fixture, and `vc-x1 validate` passes, the fourteen `dr1_*` tests and the new builder test with it.

Result (2026-10-26, fwlaptop): passed. The builder test also fails against the builder as it was,
on `push`'s retired `--title`.

#### Deliberation

- Opened on the unlanded closing: the bookmark is created at the closing of **feat: vc-x1 commit
  takes a revision** rather than at `main`, and Land takes both cycles, wink's call (2026-10-26).
  - The closing stays as pushed, so no published commit is rewritten.
  - Bends [Opening](AGENTS.md#opening)'s bookmark-on-`main`, for this cycle's opening only.

#### Ladder

- [fix: the fixture builder drives commit][f0] (done)

##### fix: the fixture builder drives commit

The builder follows the commands it drives, and a test keeps it following them.

- **dr1_builder_runs_against_this_vc_x1**, in `tests/cli_fixture_builder.rs`, runs
  `build-dr-1.py --local` into its own tempdir with `VC_X1` set to the binary the test build makes,
  and checks that both repos are colocated. It is ignored like the `dr1_*` tests, so full runs it
  and fast does not, at about 3.5 s.
  - `HOME` and `XDG_CONFIG_HOME` point at the tempdir, by `tests/common`'s `isolate_config`, now
    public, so init's Claude Code symlink lands there and not under the user's home.
  - Spawning `python3` is entry 4 of the spawn register in `clippy.toml`, test fixture setup,
    widened to name it.
- The builder's transcript text still quotes `vc-x1 push <bookmark> --title ...`: it is what the
  fixture's simulated sessions typed, and the `dr1_*` tests read it as history.
- Filed with it: **init and clone take --scope por**, from the same session.
- A single-step cycle, so no close-out shape is chosen. No agent-file changed, so
  `notes/agent-files-size.md` gains no row.

# References

[f0]: #fix-the-fixture-builder-drives-commit
