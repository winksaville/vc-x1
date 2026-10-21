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
into three stages, dev, release, and the PATH, each with its command. The keys that cycle landed
as `[install] release`, `dev`, and `dev-bin` were read by nothing, and become `[build]`'s here.

#### Problem

Land's promotion is a hand-typed `cargo install --path .`, which checks nothing: it installs
whatever the working copy holds, pushed or not, at a suffixed version or a bare one, and it
replaces the PATH binary in the same step that builds the release, so the release is first run
when it is already the one consumers get. The `build.rs` guard that once refused a suffixed
version under the plain name is gone, and could never see the bookmark or the remote. The dev
binary lives off the PATH under `tmp/dev`, reached only by typing its path, and nothing says when
it is stale. The table that names the builds is read by nothing.

#### Solution

Provisional, the plan as the install rung left it:

| Command | What it does | When |
|---|---|---|
| `vc-x1 validate` | fmt, clippy, every test | every rung, before review |
| `vc-x1 validate --fast` | the unit tests only | while iterating |
| `vc-x1 build-dev` | the dev build, into `tmp/dev` | to build without running |
| `vc-x1 run-dev` | `build-dev`, then runs it | trying the change |
| `vc-x1 build` | the release, into `tmp/rel`, with its stamp | at Land, or to try the release profile |
| `vc-x1 run` | runs the staged release | the smoke check before install |
| `vc-x1 install` | copies the staged release onto the PATH, guarded | at Land |

- `vc-x1 build` runs `[build] release`, `--root tmp/rel`, from any commit, and writes a stamp
  beside `[build] release-bin` naming the commit it was built from, the state of the source code
  that was compiled, and the binary's size and modification time. `vc-x1 build-dev` runs
  `[build] dev`. Neither checks anything, and neither touches the PATH.
- `vc-x1 install` deploys the staged release by running `[install]`'s command list, which copies
  `[build] release-bin` to its destination, so the binary deployed is the binary tested. It
  refuses unless `@-` is `main` at `main@origin`, `@` is empty, the version is bare, the stamp
  names `main@origin`'s commit with state `clean`, the binary's size and modification time are
  the stamp's, and the staged binary's `-V` is the manifest's version and, when it reports one,
  the stamp's commit.
- `vc-x1 run` runs `[build] release-bin` as staged, building nothing first, and `vc-x1 run-dev`
  runs `build-dev` and then `[build] dev-bin`, each with the given arguments from any directory of
  the workspace, and each warning when the binary's `-V` is not the manifest's version.
- `vc-x1 validate` checks the source and makes no build: `[validate] full` loses the dev build's
  line.
- The schema's `[build]` table holds `release`, `release-bin`, `dev`, and `dev-bin`, and
  `[install]` the deploy key, named at its rung. `vc-config-model.md` is regenerated, and this
  workspace's `.vc-config.md` carries the values.

#### Acceptance check

`build` leaves `tmp/rel/bin/vc-x1` and its stamp, `run -V` prints the manifest's version,
`build-dev` leaves `tmp/dev/bin/vc-x1` at the manifest's version, and `run-dev -V` prints it.
`install` refuses on the cycle's bookmark, refuses a stamp from any commit but `main@origin`'s,
and refuses a staged binary rebuilt after its stamp. This cycle's own Land stages by the dev
binary's `build`, smoke-tests by its `run -V`, and deploys by its `install`, the first deployment
the command makes.

#### Deliberation

- Multi-step, five rungs, a trapezoid the default, the agent's proposal: opening, build stages the
  release, install deploys it, run and run-dev, closing.
  - Build first, since install reads its stamp and run reads its binary.
  - run and run-dev are one rung, sharing the binary's resolution, the pass-through, and the
    staleness check.
- Widened at the opening, wink's call (2026-10-21): a release staged under `tmp/rel` before it
  replaces anything on the PATH, `vc-x1 run` to run it, and a separate step onto the PATH. Folded
  in rather than filed, since it moves the guard, so building the release alone first would be
  rework. Retitled from "feat: vc-x1 install and run-dev" before the opening commit pushed, the
  bookmark renamed with it.
- The names, settled at the build rung (2026-10-21), wink's calls, in three steps:
  - `install --dev` became its own command, so each build has one, named with the `-dev` suffix
    as `run` and `run-dev` are: one rule, the suffix always meaning the dev build, and no dropped
    flag giving the other.
  - The private builds are `build` and `build-dev`, and `install` is the one step that touches
    the PATH, as `cargo install` means. They were `install`, `install-dev`, and `install-global`,
    where "install" for a build that reaches no PATH was the confusion. The table follows,
    `[build]` for the builds and `[install]` for the deploy.
  - `validate` checks the source and makes no build of either kind: `[validate] full` lost its
    dev build line, and `run-dev` builds first, as `cargo run` does. What runs is always the
    current code, after a fast validation too, which the line at the end of full could not
    promise, and the dev build has one definition, `[build] dev`. A dev binary run by its path
    can still be stale, which `run-dev` is the remedy for.
  - `validate` keeps its name, not `validate-dev`: it acts on the source, not a build, has no
    plain counterpart to tell apart, and the agent-files name it, so a rename is a proposal. A
    release validation, if one comes, is named beside it.
  - The cycle's title and its bookends keep the pushed name, "install stages, deploys, and runs
    builds", the opening being published, and the rungs not yet pushed take the new names.
- One build in `tmp/` during development, wink's practice (2026-10-21): the dev build lives in
  `tmp/dev` and the staged release in `tmp/rel` only from a deliberate `build` until it is
  installed, so no stale release sits beside the dev build to be run by mistake. wink removed the
  build rung's `tmp/rel` at the status rung, and Land's `build` stages a fresh one.
- The guard sits in the step consumers see, `install`, so `build` stages a release-profile build
  from a cycle's bookmark too, the check for a release-only failure mid-cycle.
- `install` copies rather than rebuilds: a second `cargo install` would deploy a fresh build, not
  the one tested. The cost is cargo's install tracking, which no longer knows the binary, so
  `cargo install --list` omits it and `cargo uninstall` does not remove it.
  - The deploy is a command list like the others, so a workspace picks its destination,
    `~/.cargo/bin` or `~/bin`, and its method, and an adopter with nothing to deploy, a library
    crate, leaves it out, where `install` says so and stops.
- The provenance is a stamp file beside the staged binary, written by `build`, not a commit id
  compiled in: the closing carries the bare version before it lands, so the version alone cannot
  tell a release staged from the unlanded closing from one staged from `main`, and a stamp works
  for any adopter's artifact, where a compiled-in id works only for binaries that print it.
  - The commit compiled in as well, wink's call at the build rung (2026-10-21): `build` gives its
    commands `VC_X1_BUILD_COMMIT` and `VC_X1_BUILD_STATE`, and vc-x1 shows them as semver build
    metadata, `0.85.7+d70cf9ecdfd9`, `.dirty` for a build with edits, so a deployed binary says
    what it was built from. The stamp stays, the check that works for any adopter's artifact,
    and `install` also compares the binary's own commit with the stamp when the binary reports
    one.
    - The commit id, not the change id: a change keeps its id when amended, so a fix squashed
      into the closing after staging would leave a binary claiming to be the landed closing.
    - A hand-typed cargo build carries the commit too, wink's call (2026-10-21): `build.rs`
      takes the two variables when vc-x1 passed them, else reads git's `HEAD`, `@-` in a jj
      colocated repo, from the files with no spawn, and marks the build `.unknown`, since
      whether `@` held edits is a jj question a build script cannot ask without compiling
      jj-lib again. A source package has no `.git` and carries nothing.
      - Not yet: `.cargo_vcs_info.json`, the commit cargo itself writes into a package, which
        `build.rs` could read as a third source once vc-x1 is published.
    - The dev build carries them too, wink's call (2026-10-21), and `validate` passes them to
      every command of its list, so the test build and the dev build compile with the same
      values and `build-dev` reuses the test build, 0.37 s measured, rather than each recompiling
      the crate for the other.
  - The stamp: `<release-bin>.stamp`, `tmp/rel/bin/vc-x1.stamp` here, keys `toml_simple` reads
    back, `commit`, the full id of `@-`, and `state`, `clean` when `@` was empty and `dirty` when
    not, so a build with uncommitted edits is stamped as such rather than refused. No version,
    which `install` reads from the binary.
  - The second field is `state`, the state of the source code that was compiled, relative to the
    commit the build names, wink's naming and definition at the build rung (2026-10-21),
    replacing a `clean` flag once `unknown` made it three-valued: `clean`, `dirty`, or `unknown`,
    spelled the same in the variables, the stamp, and the banner, where `clean` is left out so a
    clean build reads `+fcc7f222a774`.
  - The binary's size and modification time in the stamp too, `size` and `mtime-ns`, wink's call
    at the build rung (2026-10-21), after a hand `cargo install --root tmp/rel` replaced the
    staged binary and left its stamp vouching for it: `install` compares them with the binary,
    catching a rebuild for any artifact, where the commit in `-V` catches it only for one that
    reports it.
    - A build-time `-V` check, recording whether the binary reports the commit, weighed and set
      aside: `install` runs `-V` anyway for the version and compares the commit there, so `build`
      spawns nothing. A hash, stronger, needs a dependency and reads the whole binary, and size
      with time catches every case met.
- Status's short scope forms, `st -s w`, folded into the build rung, wink's ask (2026-10-21):
  hidden aliases on status's keyword list, the smaller fix **status takes the write commands'
  scope** set aside, as a stopgap until that entry's fix runs. Unplanned work run in the current
  rung rather than inserted as its own, wink's say, so its edits share the rung's commit.
- Status's scope fixed properly as its own rung, inserted after the build rung, wink's call
  (2026-10-21): the Todo entry **status takes the write commands' scope**, planned for its own
  cycle, run now since the cycle is adding subcommands. The working copy was empty after the build
  rung's push, so nothing was set aside. The entry leaves `## Todo`, this block its record.
  - `-s` stays defaulted to `work` rather than required, the agent's choice at the rung, left for
    wink's review: requiring it is **every repo command takes -s, required**, which decides every
    read command together.
  - A parser per command, built by one function from the scopes the command offers, wink's call
    at the rung (2026-10-21), after `sync -s <tab>` offered nothing: the write commands took
    `parse_repo_scope` as a bare function, which gives completion no values. `commit` offers
    work, both, and por, `merge` work and por, `validate-desc` work, agent, and both, and the
    rest all four.
    - A scope a command does not offer is declared hidden rather than left out, so it still
      parses and reaches the command's own refusal, which says why, and completion and clap's
      listed values show only what is offered. `bot` is hidden the same way, keeping its fix-it.
    - The help's `REPO_SCOPE_FORMS` text, with its "`a` refused" notes, stays beside clap's own
      list of the offered values.
- `build` and `build-dev` take `-R` and no `-s`, as `validate` does: they build the work side's
  artifact and write no repo, and the `-s` sweep, **every repo command takes -s, required**,
  decides all three together.
- A failed check refuses, the agent's lean, `install`'s: it is the step consumers see, and a
  warning scrolls past. Open until wink's say at its rung.
- No rename, a bend of Opening step 6 and Land step 1, as the last cycle ran, wink's say
  (2026-10-21): the package keeps its plain name, the dev build lands under `tmp/dev`, and the
  `build.rs` guard is already gone. The bend covers this cycle's pushes and its Land, which
  restores no name, and nothing else.
- The dev build is a debug build, wink's call (2026-10-21), made in the opening so every rung's
  validation had it: `cargo install --debug` in the dev build's line. Measured here, it reuses the
  test build and takes 0.7 s against a release build's 9.7 s, and 10.5 s against 40 s cold. The
  debug binary runs three to five times slower, sub-second for most commands and 1.4 s for
  `agent-session` on a 31 MB transcript, and its overflow checks and `debug_assert!` suit a build
  that exists to be tested.
  - The release profile is then built only by `build`, which the staged release makes cheap to
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
  - Completion forwarded, wink's ask at the status rung (2026-10-21): `vc-x1 run-dev <args>` is
    completed by the `vc-x1` typed, which knows `run-dev` but not the dev build's own arguments,
    so `run-dev`'s and `run`'s completer hands the words after the subcommand to the binary it
    would run, in completion mode, and returns what it answers. Completion of those words is then
    the dev build's or the staged release's, as running them would be. A binary that does not
    speak clap's completion protocol answers nothing, and the forward offers nothing rather than
    failing.
    - The binary is the config's of the workspace the shell is in, so in iiac-perf the answer is
      iiac-perf's: its binaries speak the same protocol, clap_complete 4.6.7's to vc-x1's 4.6.9,
      and asked directly `iiac-perf-dev run <tab>` lists its benches (2026-10-21). The forward
      waits only on iiac-perf's `[build]` table naming its dev binary.
    - Completion never builds: `run-dev` builds before it runs, but a tab press must not, so the
      forward asks the dev binary as it stands and offers nothing when there is none.
    - The shell knows none of it: it runs `COMPLETE=bash vc-x1 -- vc-x1 run-dev run zcr` as for
      any command and shows what prints, and vc-x1 runs the target in completion mode with the
      words after the subcommand, `COMPLETE=bash iiac-perf-dev -- iiac-perf-dev run zcr`, and
      prints its answer.
    - Done in `main` before `CompleteEnv`, not as a clap completer: clap's value completers are
      given only the word being completed, and the forward needs every word after the
      subcommand. `main` sees completion mode, finds `run` or `run-dev` among the words, and
      forwards them with the cursor index shifted, leaving every other completion to clap. The
      spawn is the one the rung already registers to run the artifact.
- `vc-x1 run` was first set aside, the release being on the PATH under its own name, and the
  staged release reverses that: `run` is how the release is run before it is deployed. The two
  names stay, a dropped `--dev` flag being a silent run of the wrong build.
- The order, wink's call (2026-10-21), reversing the last closing's: this cycle and **feat: vc-x1
  land closes a cycle and lands it** run before **agent-files(proposal): v0.3.1**, so v0.3.1 is
  written once against the commands Land will use. The land entry moves above v0.3.1 in
  `## Todo` with it.
- Commit bodies in the two-paragraph form from the build rung on, wink's say (2026-10-21), ahead
  of **agent-files(proposal): v0.3.1**: the problem, then a paragraph opening "Solution:" and
  ending "See details in zCLOSED.md.", no bullets. A bend of Commit-body form covering this
  cycle's remaining commits, the bookends' pointer bodies unchanged, and nothing beyond the cycle.
- The rules lag the tool until v0.3.1: `vc-x1 push` publishes a pair `vc-x1 commit` made and
  takes no `--title` or `--body`, and squash-push takes `-s`. This cycle follows the tool, as the
  last one's later rungs did.
- Open: whether a missing `-s` names the four scopes in the error, as the parent's plan said,
  rather than clap's required-argument error, `--scope` with the four in `--help`.
- Land's three steps, `build`, `run -V`, and `install`, run the dev binary, `tmp/dev/bin/vc-x1`,
  since the installed `vc-x1 0.85.6` has none of the commands.
- The opening also carries two Todo additions made the same day before it: **status takes the
  write commands' scope**, and the land entry's **Two commands, and their undos**.

#### Ladder

- [feat: vc-x1 install stages, deploys, and runs builds opening][0] (done)
- [feat: vc-x1 build stages the release][1] (done)
- [feat: status takes the write commands' scope][2] (done)
- [feat: vc-x1 install deploys the staged release][3]
- [feat: vc-x1 run and run-dev run the builds][4]
- [feat: vc-x1 install stages, deploys, and runs builds closing][5]

##### feat: vc-x1 install stages, deploys, and runs builds opening

The cycle's setup commit: create and publish the bookmark at `main`, move the Todo entry into this
block, widened to three stages, and bump the version-of-record. Under the bend above it renames
nothing, and it makes the dev build a debug build, so the cycle's validations run on it.

##### feat: vc-x1 build stages the release

The release is built under its own root with a stamp naming the commit it came from, so it can be
run before anything on the PATH is replaced, and `build-dev` is the dev build on demand.

- `vc-x1 build` runs `[build] release`, now `--root tmp/rel`, and writes the stamp beside the new
  `[build] release-bin`, recording the commit, the state of the source code that was compiled,
  and the binary's size and modification time. The old stamp is removed before the build and the
  new one written only once it passes and the binary is there, so no stamp outlives a failed
  build.
- `vc-x1 build-dev` runs `[build] dev` and writes no stamp.
- Both builds' commands, and `validate`'s, get the commit and the state in their environment, and
  vc-x1 compiles them in, so its banner carries the commit as build metadata and `vc-x1 version`
  a `build` line, wherever the binary is copied.
- `build.rs` falls back to git's `HEAD` when no variables are passed, so a hand-typed cargo build
  reports its commit too, marked `.unknown`. The `HEAD` reader is its own file, compiled by
  `build.rs` and by the crate's tests.
- The names: the rung began as `install` with `--dev`, the dev build became `install-dev`, and
  both became `build` and `build-dev`, `install` kept for the step onto the PATH. The table is
  `[build]`, its keys the ones the last cycle landed under `[install]`.
- `[validate] full` no longer builds the dev binary, which `run-dev` will build first.
- Running a configured command list moved out of `validate` into a module the three share, so the
  one spawn site serves them, and clippy's register entry 5 is widened to say so.
- Status takes `w`, `a`, and `b` for its scope, hidden aliases of the long names, the stopgap for
  **status takes the write commands' scope**.
- The schema's `install.*` keys become `build.*`, with `build.release-bin` new and the release
  reworded as the staged build, and the model is regenerated. The README's build section
  describes the stages and the commands in a cycle's order, gains a `build` section, and lists
  `[build]` among the config tables.

##### feat: status takes the write commands' scope

Status's `-s` takes the same four scopes as every write command, so one parser serves them all,
and status's own positional and comma forms go.

- Status's `-s` takes `w|work`, `a|agent`, `b|both`, and `p|por`, absent `work`.
- Every `-s` taking a `RepoScope`, status's and the eight write commands', is a parser made by one
  builder from the scopes the command offers: the long names declared for completion, the short
  ones as hidden aliases, the scopes not offered and `bot` declared hidden so they still reach the
  command's refusal or the fix-it, and the value handed to `parse_repo_scope`.
- `p` reports the repo `-R` names or the cwd's as itself, labeled `por`, resolved by the write
  commands' resolver, so it is refused on a side of a workspace with the same error they give.
- The positional scope, `status both`, and the comma forms `work,agent` and `agent,work` are
  gone, and status's old keyword parser with them, the stopgap aliases included.
- The README's status section and synopsis describe the four scopes, and its examples use `-s`.

##### feat: vc-x1 install deploys the staged release

The staged binary, the one tested, is copied to where consumers run it, after a check that it was
built from pushed `main` at a bare version and is still the binary its stamp describes.

##### feat: vc-x1 run and run-dev run the builds

The staged release and the dev build, both off the PATH, are run through the config from anywhere
in the workspace, `run-dev` building first, with a warning when either is not the manifest's
version, and the words after either subcommand are completed by the binary it would run.

##### feat: vc-x1 install stages, deploys, and runs builds closing

Closing out the cycle.

# References

[0]: #feat-vc-x1-install-stages-deploys-and-runs-builds-opening
[1]: #feat-vc-x1-build-stages-the-release
[2]: #feat-status-takes-the-write-commands-scope
[3]: #feat-vc-x1-install-deploys-the-staged-release
[4]: #feat-vc-x1-run-and-run-dev-run-the-builds
[5]: #feat-vc-x1-install-stages-deploys-and-runs-builds-closing
