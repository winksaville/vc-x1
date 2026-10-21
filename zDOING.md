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

### feat: vc-x1 install stages, deploys, and runs builds

(wink + agent, 2026-10-21, opened 2026-10-21) The last two work rungs of **feat: vc-x1 commit,
install, and run-dev**, deferred at its closing, run as their own cycle and widened at the opening
into three stages, dev, release, and global, each with its command. The `[install]` keys that
cycle landed, `release`, `dev`, and `dev-bin`, are read by nothing yet, so `vc-config.md`'s prose
names these subcommands ahead of the binary until this cycle lands.

#### Problem

Land's promotion is a hand-typed `cargo install --path .`, which checks nothing: it installs
whatever the working copy holds, pushed or not, at a suffixed version or a bare one, and it
replaces the PATH binary in the same step that builds the release, so the release is first run
when it is already the one consumers get. The `build.rs` guard that once refused a suffixed
version under the plain name is gone, and could never see the bookmark or the remote. The dev
binary lives off the PATH under `tmp/dev`, reached only by typing its path, and nothing says when
it is stale. The `[install]` table that names them is read by nothing.

#### Solution

Provisional, three stages, the opening's plan:

| Stage | Command | Where it lands | Run by |
|---|---|---|---|
| Dev | `vc-x1 install --dev` | `tmp/dev` | `vc-x1 run-dev` |
| Release | `vc-x1 install` | `tmp/rel` | `vc-x1 run` |
| Global | `vc-x1 install-global` | the PATH, as the config says | the plain name |

- `vc-x1 install` runs `[install] release`, now `--root tmp/rel`, from any commit, and writes a
  stamp beside the binary naming the commit it was built from and whether `@` was empty.
  `--dev` runs `[install] dev` as before. Neither checks anything.
- `vc-x1 install-global` deploys the staged release by running `[install] global`, which copies
  `[install] release-bin` to its destination, so the binary deployed is the binary tested. It
  refuses unless `@-` is `main` at `main@origin`, `@` is empty, the version is bare, the stamp
  names `main@origin`'s commit with an empty `@`, and the staged binary's `-V` is the manifest's
  version.
- `vc-x1 run` runs `[install] release-bin`, and `vc-x1 run-dev` runs `[install] dev-bin`, each
  with the given arguments from any directory of the workspace, and each warning when the
  binary's `-V` is not the manifest's version.
- The schema gains `release-bin` and `global`, `vc-config-model.md` is regenerated, and this
  workspace's `.vc-config.md` carries the values.

#### Acceptance check

`install` leaves `tmp/rel/bin/vc-x1` and its stamp, `run -V` prints the manifest's version,
`install --dev` leaves `tmp/dev/bin/vc-x1` at the manifest's version, and `run-dev -V` prints it.
`install-global` refuses on the cycle's bookmark, and refuses a stamp from any commit but
`main@origin`'s. This cycle's own Land stages by the dev binary's `install`, smoke-tests by its
`run -V`, and deploys by its `install-global`, the first deployment the command makes.

#### Deliberation

- Multi-step, five rungs, a trapezoid the default, the agent's proposal: opening, install stages
  the release, install-global deploys it, run and run-dev, closing.
  - Install first, since install-global reads its stamp and run reads its binary.
  - run and run-dev are one rung, sharing the binary's resolution, the pass-through, and the
    staleness check.
- Widened at the opening, wink's call (2026-10-21): a release staged under `tmp/rel` before it
  replaces anything on the PATH, `vc-x1 run` to run it, and a separate global step. Folded in
  rather than filed, since it moves the install rung's guard, so building install alone first
  would be rework. Retitled from "feat: vc-x1 install and run-dev" before the opening commit
  pushed, the bookmark renamed with it.
- The guard moves to the global step, the one consumers see, so `install` stages a release-profile
  build from a cycle's bookmark too, the check for a release-only failure mid-cycle.
- install-global copies rather than rebuilds: a second `cargo install` would deploy a fresh build,
  not the one tested. The cost is cargo's install tracking, which no longer knows the binary, so
  `cargo install --list` omits it and `cargo uninstall` does not remove it.
  - `[install] global` is a command list like the others, so a workspace picks its destination,
    `~/.cargo/bin` or `~/bin`, and its method, and an adopter with nothing to deploy, a library
    crate, leaves it out, where install-global says so and stops.
- The provenance is a stamp file beside the staged binary, written by `install`, not a commit id
  compiled in: the closing carries the bare version before it lands, so the version alone cannot
  tell a release staged from the unlanded closing from one staged from `main`, and a stamp works
  for any adopter's artifact, where a compiled-in id works only for binaries that print it.
  - Open: the stamp's name and form, at the install rung.
- A failed check refuses, the agent's lean, now install-global's: it is the step consumers see,
  and a warning scrolls past. Open until wink's say at its rung.
- Open: the name `install-global`, wink's working name, against `deploy`, shorter and saying the
  step leaves the workspace, which `install --global` would hide.
- No rename, a bend of Opening step 6 and Land step 1, as the last cycle ran, wink's say
  (2026-10-21): the package keeps its plain name, the dev build installs under `tmp/dev`, and the
  `build.rs` guard is already gone. The bend covers this cycle's pushes and its Land, which
  restores no name, and nothing else.
- The dev install is a debug build, wink's call (2026-10-21), made in the opening so every rung's
  validation has it: `cargo install --debug` in `[install] dev` and in full's last entry. Measured
  here, it reuses full's `cargo test` build and takes 0.7 s against a release build's 9.7 s, and
  10.5 s against 40 s cold. The debug binary runs three to five times slower, sub-second for most
  commands and 1.4 s for `agent-session` on a 31 MB transcript, and its overflow checks and
  `debug_assert!` suit a build that exists to be tested.
  - The release profile is then built only by `install`, which the staged release makes cheap to
    run mid-cycle.
  - Per workspace: iiac-perf, a latency harness, keeps a release dev build, and `vc-config.md`'s
    example stays the release form, the general case.
- `run` and `run-dev` are about the workspace's artifact, not vc-x1 itself: in another adopter
  they run that project's builds, `vc-x1 run-dev run <bench>` in iiac-perf. So their rung adds:
  - A `-bin` value resolved as a shell does: one with a slash against the config file's
    directory, or as is when absolute, and a bare name looked up on the PATH, so the rename style,
    `dev-bin = "iiac-perf-dev"`, works too.
  - The arguments passed through untouched, so the previous release parses none of the new
    build's flags.
  - The staleness check comparing the version alone, the name being `<name>-dev` under the rename
    style, and skipping quietly when `-V`'s output does not parse.
- `vc-x1 run` was first set aside, the release being on the PATH under its own name, and the
  staged release reverses that: `run` is how the release is run before it is deployed. The two
  names stay, a dropped `--dev` flag being a silent run of the wrong build.
- The order, wink's call (2026-10-21), reversing the last closing's: this cycle and **feat: vc-x1
  land closes a cycle and lands it** run before **agent-files(proposal): v0.3.1**, so v0.3.1 is
  written once against the commands Land will use. The land entry moves above v0.3.1 in
  `## Todo` with it.
- The rules lag the tool until v0.3.1: `vc-x1 push` publishes a pair `vc-x1 commit` made and
  takes no `--title` or `--body`, and squash-push takes `-s`. This cycle follows the tool, as the
  last one's later rungs did.
- Open: whether a missing `-s` names the four scopes in the error, as the parent's plan said,
  rather than clap's required-argument error, `--scope` with the four in `--help`.
- Land's three install steps run the dev binary, `tmp/dev/bin/vc-x1`, since the installed
  `vc-x1 0.85.6` has none of the commands. After Land, full's last entry can become
  `vc-x1 install --dev`, ending the dev install's line written twice, a follow-up and not this
  cycle's.
- The opening also carries two Todo additions made the same day before it: **status takes the
  write commands' scope**, and the land entry's **Two commands, and their undos**.

#### Ladder

- [feat: vc-x1 install stages, deploys, and runs builds opening][0] (done)
- [feat: vc-x1 install stages the release][1]
- [feat: vc-x1 install-global deploys the staged release][2]
- [feat: vc-x1 run and run-dev run the builds][3]
- [feat: vc-x1 install stages, deploys, and runs builds closing][4]

##### feat: vc-x1 install stages, deploys, and runs builds opening

The cycle's setup commit: create and publish the bookmark at `main`, move the Todo entry into this
block, widened to three stages, and bump the version-of-record. Under the bend above it renames
nothing, and it makes the dev install a debug build, so the cycle's validations run on it.

##### feat: vc-x1 install stages the release

The release is built under its own root with a stamp naming the commit it came from, so it can be
run before anything on the PATH is replaced, and `--dev` is the dev install on demand.

##### feat: vc-x1 install-global deploys the staged release

The staged binary, the one tested, is copied to where consumers run it, after a check that it was
built from pushed `main` at a bare version.

##### feat: vc-x1 run and run-dev run the builds

The staged release and the dev build, both off the PATH, are run through the config from anywhere
in the workspace, with a warning when either is not the manifest's version.

##### feat: vc-x1 install stages, deploys, and runs builds closing

Closing out the cycle.

# References

[0]: #feat-vc-x1-install-stages-deploys-and-runs-builds-opening
[1]: #feat-vc-x1-install-stages-the-release
[2]: #feat-vc-x1-install-global-deploys-the-staged-release
[3]: #feat-vc-x1-run-and-run-dev-run-the-builds
[4]: #feat-vc-x1-install-stages-deploys-and-runs-builds-closing
