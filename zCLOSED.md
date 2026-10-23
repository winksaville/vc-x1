# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

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

What was done. vc-x1 is not a build system: the config says what the steps are, and vc-x1 runs
them in order and checks their results, building before it runs or installs, reporting what it
is about to copy, refusing only a copy that would be wrong, and never letting the dev build
replace the release. Cargo decides what to rebuild.

| Command | What it does | When |
|---|---|---|
| `vc-x1 validate` | fmt, clippy, every test | every rung, before review |
| `vc-x1 validate --fast` | the unit tests only | while iterating |
| `vc-x1 build-dev` | the dev build, into `target/debug` | to build without running |
| `vc-x1 run-dev` | `build-dev`, then runs it | trying the change |
| `vc-x1 install-dev` | `build-dev`, then copies it into `~/.cargo/bin` as `vc-x1-dev` | trying the change elsewhere |
| `vc-x1 build` | the release, into `target/release` | at Land, or to try the release profile |
| `vc-x1 run` | `build`, then runs it, `run -- -V` the smoke check | before install |
| `vc-x1 install` | `build`, then reports it and copies it into `~/.cargo/bin` | at Land |

- `build` and `build-dev` run `[build] release` and `dev`, plain `cargo build --release --locked`
  and `cargo build --locked`, with the commit and state of the source in the commands'
  environment, so vc-x1's banner carries the commit wherever the binary is copied. A release
  checks it left `[build] release-bin`. Neither tracks staleness, which is cargo's: the stamp the
  build rung wrote went at the stamp rung.
- `install` and `install-dev` build first, report the binary from its `-V`, its version, its
  commit against `main` and `main@origin`, and its state, and refuse only a missing binary, a `-V`
  that will not run, or a missing directory. They copy `release-bin` into `[install] release-dir`
  and `dev-bin` into `dev-dir` with `dev-suffix`, `-dev` by default, appended, each as a temporary
  file renamed into place, safe over a running binary. `install-dev` refuses before the build a
  copy that would land on the release. `--dir` replaces the directory for one run, and
  `--dry-run` builds and reports and copies nothing.
- `[install] release` and `dev`, optional command lists, run in place of the built-in copy, their
  placeholders, `{build.release-bin}` and `{install.release-dir}` or the `dev` three, filled with
  the values resolved for the run. This workspace's lists write the copy out, so its config says
  what `install` does.
- `run` and `run-dev` build first, then run the binary in the current directory with the
  arguments untouched, after a warning when its `-V` is not the manifest's version, and exit with
  its code. An argument starting with `-` comes after `--`, as with `cargo run`. The words after
  either are completed by the binary it would run, asked in clap's completion mode and never
  built. All four build and run commands take `--dry-run`.
- `status` takes the write commands' four scopes, `w`, `a`, `b`, and `p`, through the one parser
  every `-s` uses.
- The schema's `[build]` holds `release`, `release-bin`, `dev`, and `dev-bin`, and `[install]`
  holds `release`, `release-dir`, `dev`, `dev-dir`, and `dev-suffix`, in `[build]`'s order. The
  model is regenerated, and this workspace's `.vc-config.md` carries the values. `[validate] full`
  no longer builds the dev binary.

#### Acceptance check

`build` leaves `target/release/vc-x1`, `run -- -V` prints the manifest's version, `build-dev` leaves
`target/debug/vc-x1` at the manifest's version, and `run-dev -- -V` prints it. `install --dry-run`
with nothing built builds the release, and on the cycle's bookmark reports the suffixed version, the
commit off `main`, and the dirty state. This cycle's own Land builds by the dev binary's `build`,
smoke-tests by its `run -- -V`, and installs by its `install`, the first install the command makes,
its report showing `main` and `main@origin`, clean, and a bare version.

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
  - The builds moved to `target/` at the install rung, below, so a release build stays in
    `target/release` until `cargo clean`, and `run`'s version warning is what flags a stale one.
- Guidance on the nuances in one place, wink's ask at the status rung (2026-10-21): the closing
  rung's README update, close-out step 3, gathers one build in `tmp/` during development, the
  attached value forms `-sb` and `-s=b`, and the completion setup until the shim of **Shell
  completion installs itself and asks the binary typed** lands.
- The guard sits in the step consumers see, `install`, so `build` stages a release-profile build
  from a cycle's bookmark too, the check for a release-only failure mid-cycle. Superseded at the
  install rung, below: `install` reports and refuses only a wrong copy, and the guard is Land's.
- `install` copies rather than rebuilds: a second `cargo install` would deploy a fresh build, not
  the one tested. The cost is cargo's install tracking, which no longer knows the binary, so
  `cargo install --list` omits it and `cargo uninstall` does not remove it.
  - `install` builds first after all, wink's call at the install rung (2026-10-21), after an
    `install` with nothing built refused for want of a binary: as a Makefile's install depends on
    its build. Not the rebuild set aside above, since `cargo build` rebuilds only what changed,
    so with nothing changed the binary tested is the binary copied, same file, size, and time,
    and only a stale one is rebuilt, which is the one that should not be copied as it was.
  - `run` builds first too, the same call, as `cargo run --release` does, so `run` and `run-dev`
    have one shape, build if needed and run. The plan's `run` built nothing so the smoke check
    ran exactly what `install` copies, and `install` building the same way keeps that.
  - The stamp's main job, catching a change between `build` and `install`, shrinks with nothing
    left between them, and **The release stamp is reconsidered now that install builds** asks
    whether it stays. It goes, at the stamp rung, below.
  - The deploy is a directory, not a command list, wink's say at the install rung (2026-10-21),
    changing the plan's command list: the config is committed and the destination is per
    machine, so `[install] dir = "~/.cargo/bin"` with vc-x1 expanding `~` fits every clone where a
    `cp` line would commit one home path, commands running with no shell to expand it. And the
    PATH binary replaces itself in later cycles, which `cp` onto a running executable fails at,
    "Text file busy", where a temporary file renamed into place works, as `cargo install` does.
    vc-x1 copies in process, so the copy spawns nothing. An adopter with nothing to deploy, a
    library crate, leaves the key out, where `install` says so and stops.
- The provenance is a stamp file beside the staged binary, written by `build`, not a commit id
  compiled in, superseded at the stamp rung (2026-10-22), where the stamp goes and the commit
  compiled in is the provenance: the closing carries the bare version before it lands, so the version alone cannot
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
  - The stamp: `<release-bin>.stamp`, `target/release/vc-x1.stamp` here, keys `toml_simple` reads
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
- A failed check refuses, wink's say at the install rung (2026-10-21), the agent's lean: it is the
  step consumers see, and a warning scrolls past. Every check runs first and every failure is
  listed at once, so one run says everything to fix.
  - Reversed at the rung's review, wink's call (2026-10-21), after `install -h` read as jargon and
    a run on the bookmark refused it for being off `main`: too strict, since some release
    processes release from a branch or a tag, and a `strict` mode cannot say which. Which builds
    may be installed is the release process's decision, so `install` reports and the user
    judges, and it refuses only a copy that would be wrong: no binary, a `-V` that will not run,
    or a missing install directory. Here Land is the guard: it runs `install` on `main` after the
    sync and the push, and reads the report.
  - Gathered but no longer judged: `@-` and whether `@` is empty, facts about the working copy
    rather than the binary, are dropped. The binary's own `-V` is the report's first source, and
    the stamp speaks for it only while its size and time still match.
  - A configured list, `[install] require`, turning chosen report lines into refusals for a
    project that wants the tool to enforce its policy, weighed and set aside until one asks.
- The builds are plain `cargo build`, wink's call at the install rung (2026-10-21), not
  `cargo install --root`: `cargo install` without `--root` writes into `~/.cargo/bin`, so a
  `[build]` line built on it is one dropped flag from doing what `build` promises not to. `--root`
  is only the install prefix, the binary copied to `<root>/bin` with cargo's `.crates.toml` and
  `.crates2.json` beside it, and the compile was always in `target/`, so `tmp/rel` and `tmp/dev`
  held copies. The binaries are now `target/release/vc-x1` and `target/debug/vc-x1`, and
  "staged" leaves the vocabulary.
  - The cost: `cargo clean` removes the release build and its stamp, and a hand
    `cargo build --release` replaces it, which the report's stamp line names.
  - `cargo build --artifact-dir`, a copy out of `target/` with no install, is unstable, so not
    used.
  - Both builds `--locked`, wink's call, so the two compile the same dependencies, the ones
    `Cargo.lock` commits. A version bump updates the lock with the manifest, so it never trips.
  - Where a build leaves its binary is the config's to say, `release-bin` and `dev-bin`: vc-x1
    never works it out from cargo or the package's name.
- vc-x1 runs the builds but does not track them, wink's question and the agent's proposal at the
  install rung's review (2026-10-22), after wink's "I'm not sure we're headed in the right
  direction to have vc-x1 be a build system":
  - The builds stay in vc-x1 because a hand-run Land sequence is where the agent's recorded
    failure recurs, a result not read: [Never mask a command's exit
    status](agent-data/rationale.md#working-practices) (2026-08-05) and [A failure is read before
    validation runs again](agent-data/rationale.md#the-per-rung-flow) (2026-10-15). A switch from
    shell to Python does not retire it, a `subprocess.run` without `check=True` being the same
    miss, and a command written once, reviewed, and tested does, as `validate` did for the
    validation. Running a configured list and checking what it left is not a build system, since
    cargo still does the building.
  - What would be one goes: the stamp and `build`'s `built` or `up to date` re-track staleness,
    which cargo already tracks, and the commit compiled into the binary, read from its `-V`, says
    what it was built from. Run as the inserted rung **feat: the release stamp goes, cargo tracks
    staleness**, so the install rung's commit stays the work reviewed.
- No rename, a bend of Opening step 6 and Land step 1, as the last cycle ran, wink's say
  (2026-10-21): the package keeps its plain name, the dev build lands off the PATH, and the
  `build.rs` guard is already gone. The bend covers this cycle's pushes and its Land, which
  restores no name, and nothing else.
- The dev build is a debug build, wink's call (2026-10-21), made in the opening so every rung's
  validation had it: `cargo install --debug` in the dev build's line. Measured here, it reuses the
  test build and takes 0.7 s against a release build's 9.7 s, and 10.5 s against 40 s cold. The
  debug binary runs three to five times slower, sub-second for most commands and 1.4 s for
  `agent-session` on a 31 MB transcript, and its overflow checks and `debug_assert!` suit a build
  that exists to be tested.
  - The release profile is then built only by `build`, which makes it cheap to run mid-cycle.
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
    the dev build's or the release build's, as running them would be. A binary that does not
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
  release built off the PATH reverses that: `run` is how the release is built and run before it
  is installed. The two names stay, a dropped `--dev` flag being a silent run of the wrong build.
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
- Land's three steps, `build`, `run -- -V`, and `install`, the last two building nothing new, run
  the dev binary, `target/debug/vc-x1`, since the installed `vc-x1`, wink's hand build of 0.85.7-1,
  predates `install` and `run`.
- The opening also carries two Todo additions made the same day before it: **status takes the
  write commands' scope**, and the land entry's **Two commands, and their undos**.

#### Ladder

- [feat: vc-x1 install stages, deploys, and runs builds opening][0] (done)
- [feat: vc-x1 build stages the release][1] (done)
- [feat: status takes the write commands' scope][2] (done)
- [feat: vc-x1 install copies the release build][3] (done)
- [feat: the release stamp goes, cargo tracks staleness][4] (done)
- [feat: install takes --dir][5] (done)
- [feat: vc-x1 install-dev puts the dev build on the PATH][6] (done)
- [feat: vc-x1 run and run-dev run the builds][7] (done)
- [feat: vc-x1 install stages, deploys, and runs builds closing][8] (done)

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

##### feat: vc-x1 install copies the release build

The release build, the one tested, is copied to where consumers run it, after a report of what it
is, for the user and the release process to judge.

- `vc-x1 install` copies `[build] release-bin` into the new `[install] dir`, `~` expanded, as a
  temporary file beside the destination renamed into place, the file name kept. `$PATH` is not
  searched.
- It runs the release build first through `build`'s own code, the stamp included, a no-op for
  cargo when nothing changed, and checks `[install] dir` before it, so a workspace with nothing
  to install builds nothing.
- `build` says whether the release binary was `built` or `up to date`, from its size and time
  before and after the list, so a no-op build shows that the binary tested is the one copied.
- It then gathers the facts, `main`, `main@origin`, the manifest's version, the stamp, the
  binary's size and time, and what its `-V` reports, and reports the version, the commit placed
  against `main` and `main@origin`, the state, and whether the stamp matches the binary, and
  refuses only a missing binary, a `-V` that will not run, or a missing install directory, listing
  every reason. `--dry-run` builds and reports, and copies nothing.
- Running the binary's `-V` is clippy's spawn register entry 7. Its output is read at the first
  line whose second word starts with a digit, so a line printed before the version is passed
  over, and output with no such line is unreadable and refused, found when wink's test
  `println!` ahead of the banner was read as the version `hi` and copied.
- `[build]` becomes plain `cargo build`, `--release --locked` and `--locked`, the binaries in
  `target/release` and `target/debug`, and "staged" leaves the help, the docs, and the messages.
- Every `unwrap_or*` the cycle added carries its `// OK:` comment, five of them in this rung and
  three the build rung pushed without one, `build_env`'s and two in the banner's state, a miss
  caught here.
- The schema gains `install.dir`, the build keys' examples follow the new lists, the model is
  regenerated, and the README's overview, `build` section, and config tables follow, with a new
  `install` section.
- Three Todo entries arrive with the rung's review: **Every repo's .gitignore ignores .claude, and
  settings stay out of agent-repos**, to run as the next rung, **A config file says what and its
  constraints, and the why is in the cycle record**, and **The release stamp is reconsidered now
  that install builds**.

##### feat: the release stamp goes, cargo tracks staleness

The release build is described by the commit compiled into it, and cargo alone decides what to
rebuild, so vc-x1 runs the builds without tracking them.

- Inserted after the install rung, wink's call (2026-10-22), with the Todo entry **The release
  stamp is reconsidered now that install builds**, which leaves `## Todo`, this block its record.
  Its code edits were made during the install rung's review and set aside as a patch, so that
  rung's commit stayed the work reviewed.
- `build` writes no stamp and no longer says `built` or `up to date`: the stamp file, the binary's
  size and time, and the comparison before and after the list go. A release still checks the
  list left `[build] release-bin`, and names the commit and state it was built from.
- `install` reports from the binary's `-V` alone, its version, commit, and state. A binary whose
  `-V` carries no commit, another adopter's, is reported as `commit unknown` and `state unknown`,
  the case the stamp covered, accepted as the cost.
- The help for `build`, `build-dev`, and `install`, the README's overview, `build`, and `install`
  sections and its config example, `vc-config.md`'s `build.release` and `build.release-bin`, the
  regenerated model, `.vc-config.md`, and clippy's register entry 7 drop the stamp.
- A 39-line duplicate in this file goes: the install rung's commit carried a second copy of the
  header's tail and the continuation notes of 2026-10-21, left by an agent edit that found the
  notes' end at the first `## In Progress`, the one inside the header's prose, and missed in that
  rung's review.

##### feat: install takes --dir

Inserted after the stamp rung, wink's call (2026-10-22), after a test of `vc-x1 install` by
`cargo run install` replaced `~/.cargo/bin/vc-x1` with the cycle's 0.85.7-3: testing `install`
copies onto the PATH, since `[install] dir` names it. `--dir <DIR>` overrides `[install] dir` for
one run, so a test copies into a scratch directory and the PATH binary is untouched. Weighed and
set aside: the `-dev` rename mid-cycle, which reaches the binary's name everywhere and Land's
restore, and `HOME=` for a test run, which is easy to forget and changes what the rest of the run
sees.

- `--dir` takes the place of `[install] dir` before the config is read for it, so a workspace with
  no `[install] dir` installs with `--dir`, and the refusal for a missing directory names "the
  install directory" rather than the key, since the flag may have named it.
- A relative `--dir` is taken against the current directory, as any path argument is, unlike
  `[install] dir`, which is taken against the config file's directory, and it gets no `~`
  expansion of its own, the shell's being enough. The README's example tests `install` into
  `tmp/bin`.
- Tried by `cargo run --locked -- install --dir tmp/bin`: the release copy went into `tmp/bin`, and
  `~/.cargo/bin/vc-x1` kept its time, the 0.85.7-3 test install.

##### feat: vc-x1 install-dev puts the dev build on the PATH

Inserted after the `--dir` rung, wink's call (2026-10-22): `install-dev` runs `build-dev`, then
copies `[build] dev-bin` into `[install] dev-dir` under its file name with `-dev` appended,
`~/.cargo/bin/vc-x1-dev` here, by `install`'s report and copy, and takes `--dir` too. The landed
`vc-x1` and the cycle's `vc-x1-dev` sit side by side on the PATH, what the `-dev` rename gave,
with nothing renamed and nothing for Land to restore.

- `run-dev` is not enough: it finds its binary by the config of the workspace it runs in, so in
  another project it runs that project's dev build, and only a dev binary on the PATH under its
  own name tries a cycle's vc-x1 there.
- The suffix began fixed, wink's call, with no key, and became `[install] dev-suffix` at the
  review, wink's call (2026-10-22), `-dev` when absent, a key in the table that uses it, not
  `[build]`, since the build leaves no suffixed name. The built-in copy appends it, the `dev` list
  takes it as `{install.dev-suffix}`, and a suffix holding a path separator is an error.
- The guard the fixed suffix gave returns as a check: without a list, `install-dev` refuses before
  the build when its destination is the release's, `install.release-dir` and the release's file
  name, the directories compared canonical, so an empty suffix or a `--dir` onto the release's
  directory cannot replace the release. A list writes where it says, out of the check's reach.
- The name the build rung set aside, "install-dev" for a build that reached no PATH, fits now,
  since this one does.
- `install-dev` is `install` with the dev build's list and binary: one params builder takes
  `dev`, as `build_params` does, the label, `install` or `install-dev`, prefixing every message,
  and a refusal for a missing binary names the key to check, `build.release-bin` or
  `build.dev-bin`, since the build ran just before and a binary missing after it is a wrong key.
- `[build] dev-bin`, in the schema since the build rung, gets its first reader, sharing
  `release-bin`'s code.
- `[install]` mirrors `[build]`, widened at the review, wink's call (2026-10-22): `release`,
  `release-dir`, `dev`, and `dev-dir`, in `[build]`'s order. `dir` becomes `release-dir`, unlanded
  so free to rename, and `dev-dir` is `install-dev`'s, required as the bin keys are, with no
  fallback to `release-dir`. Here both name `~/.cargo/bin`, the suffix keeping the two apart.
- `[install] release` and `dev` make the install scriptable as the build is: optional command
  lists that, when present, run in place of the built-in copy, after the same build, report, and
  refusals. Optional, not required as the build lists are, since the built-in copy is a default
  that needs no list. A dry run runs no list.
- The list's inputs are placeholders named by key, wink's call: `{build.release-bin}` and
  `{install.release-dir}`, or the `dev` pair, filled in each word after the whitespace split, so a
  value with spaces stays one word and no shell is added. Each is the value resolved for the run,
  `~` expanded and `--dir` applied, `--dir` made absolute, since otherwise `install --dir tmp/bin`
  would still write to the PATH. A `{name}` the command does not read is an error before the
  build. The first form, three `VC_X1_INSTALL_*` variables, needed a script for any use, since a
  list has no shell, and went.
- Weighed and set aside: a derived `{dest}`, the file name only vc-x1 knows, since with key-named
  placeholders the list names every path it writes and the user has the control, the `-dev` name
  included. The cost, accepted: a `dev` list that writes the release's name into a shared
  directory replaces the release, and vc-x1 cannot see it.
- This workspace sets both lists, wink's call, each the built-in copy written out, `cp` to a
  `.new` file and `mv` into place, safe over a running binary, so the config says what `install`
  does and the list path runs at every Land. The model's example is the same list, so a copied
  table works. `cp` and `mv` are not portable, and a `{cp}` placeholder would be additive later,
  in the new Todo entry **vc-x1 runs on Windows**.
- The rung planned for **Every repo's .gitignore ignores .claude, and settings stay out of
  agent-repos** leaves the ladder, wink's call (2026-10-22), its Todo entry run as a cycle of its
  own, so the run rung follows this one and the closing that.
- Placeholders stay in the install lists, two keys each, for this rung, wink's call. The general
  rule, any config string referencing any key by its resolved value, a flag overriding its key, is
  the new Todo entry **Any config string references any key**, of which the install lists are a
  subset.
- `command_list` gains `load_optional`, an absent key `None`, and placeholders offered by the
  caller, none for `validate` and `build`, whose lists are taken as written. The log shows each
  command with its placeholders filled, so it names the paths used. Clippy's register entry 5
  widens to the install lists.
- With a list the report's first line says `-> list` and the key, since vc-x1 no longer knows the
  destination, and a dry run says it ran nothing.
- The help, the README's overview, table, config notes, the `install` section's list paragraph,
  and a new `install-dev` section, `vc-config.md`'s `build.dev-bin` and the four `install.*`
  sections in place of `install.dir`, `.vc-config.md`, and the regenerated model follow.
- Tried by `cargo run --locked -- install-dev --dir tmp/bin`, through the `dev` list: the log
  showed `cp` and `mv` with the absolute paths, `tmp/bin/vc-x1-dev -V` reported 0.85.7-6, dirty,
  and nothing on the PATH changed. `install --dry-run` read the `release` list and ran nothing.

##### feat: vc-x1 run and run-dev run the builds

The release build and the dev build, both off the PATH, are run through the config from anywhere
in the workspace, each built first, with a warning when either is not the manifest's
version, and the words after either subcommand are completed by the binary it would run.

- Added at the `install-dev` review, wink's call (2026-10-22): `build`, `build-dev`, `run`, and
  `run-dev` take `--dry-run`, as `install` and `install-dev` do, so every pair takes it. For the
  builds and the runs it prints what would be run and runs nothing. `install`'s keeps building,
  since its report is of the binary that would be copied. `--dir` stays `install`'s, the one
  command with a destination.
- `run` and `run-dev` build through `build`'s own code, then run `release-bin` or `dev-bin` in the
  current directory with the arguments untouched, and vc-x1 exits with the binary's code, so a
  failing run fails the same way. They dispatch outside `SubcommandRunner`, whose `run` returns no
  code.
- An argument starting with `-` comes after `--`, as with `cargo run`: vc-x1's global flags, `-V`
  among them, are read before any trailing argument, so `vc-x1 run -V` prints vc-x1's banner, a
  parse test holds it, and the smoke check is `vc-x1 run -- -V`. The acceptance check, the
  cycle's deliberation, the land entry's step 6, and v0.3.1's Land step 4 say so.
- A bare `-bin` name looked up on the PATH, planned for the rename style's `iiac-perf-dev`, is
  dropped, wink's call (2026-10-22): under never-rename the config names the binary by path, and a
  PATH lookup would have `build`'s check and `install`'s copy act on whatever is on the PATH. The
  version check compares the version alone, the name being the build's own, and a `-V` with no
  version skips it.
- The completion forward, in `main` ahead of clap's `CompleteEnv`: a request whose words are
  `run`'s or `run-dev`'s arguments is answered by the binary it would run, asked with `COMPLETE`
  and the shell's variables inherited and `_CLAP_COMPLETE_INDEX` shifted, so it answers in the
  protocol the shell reads. The subcommand's own options, `--dry-run`, `-R`, and `--repo`, are
  skipped, `--` ends them, and the cursor on the subcommand, its options, or a flag before any
  argument stays clap's. The binary is never built, and none, or one that answers nothing, offers
  nothing. Planning is a pure function, so its cases are tested without a shell.
- Clippy's register entry 7 widens to `src/run_cmd.rs`, the run and the forward each a spawn of the
  workspace's build.
- Tried: `run-dev -- -V` and `run -- -V` printed 0.85.7-7, dirty, `run-dev --dry-run status -s b`
  printed the list and the command, a failing `run-dev -- no-such-subcommand` exited 2, and the
  forward, driven as bash drives it, completed `run-dev sta` to `status` and `run-dev status -s`
  to the four scopes, while `run` alone stayed clap's, `run` and `run-dev`.
- The help for the four commands, the README's new `run and run-dev` section and its `build`
  options, the Land example's `vc-x1 run -- -V`, `vc-config.md`'s `build.release`,
  `build.release-bin`, and `build.dev`, and the regenerated model follow.

##### feat: vc-x1 install stages, deploys, and runs builds closing

Closing out the cycle, its closing made by `vc-x1 commit -s b --closing`, the merge of `main` and
the run rung. What closing taught:

- Acceptance check: pass, as far as the closing can run it, Land's part at Land.
  - Run 2026-10-22 by the dev binary at 0.85.7: `build-dev` left `target/debug/vc-x1` and
    `run-dev -- -V` printed `vc-x1 0.85.7+e38b9fd5112a.dirty`. `cargo clean --release` removed
    `target/release`, and `install --dry-run` built the release in 14.4 s and reported `version
    0.85.7`, `commit e38b9fd5112a, not main`, and `state dirty`, running nothing of its list.
    `run -- -V` printed the same version.
  - The suffixed-version clause cannot hold at the bare closing version, and its report was seen
    mid-cycle, at the stamp rung's dry run at 0.85.7-4 and the `install-dev` rung's at -6.
  - Land's part, the dev binary's `build`, `run -- -V`, and `install` on `main`, the report
    showing `main` and `main@origin`, clean, and a bare version, is checked at Land.
- The Todo entry **feat: vc-x1 land closes a cycle and lands it** waited on this cycle for its step
  6, `build`, a `run -- -V` smoke check, and `install`, and its step and dependency bullets now say
  the commands are there. Kept a separate cycle, wink's call (2026-10-22).
- What outlives the cycle: the line the deliberation drew, vc-x1 runs the steps and checks their
  results and the config says what they are, opens the README's Build and install section, where
  the people it is for read it, chosen over a `notes/` topic file, which nothing would reference,
  wink's call. The README also gains the first install, `cargo run --locked -- install`.
- The `.gitignore` rung planned after the install rung left the ladder, its entry a cycle of its
  own.
- Version: the closing carries `0.85.7`, bare, and Land releases it, under the bend that renames
  nothing, so Land restores no name.
- No agent-file changed, so `notes/agent-files-size.md` gains no row.
- Close-out shape: trapezoid, the merge `commit --closing` makes, wink's call at the closing.

# References

[0]: #feat-vc-x1-install-stages-deploys-and-runs-builds-opening
[1]: #feat-vc-x1-build-stages-the-release
[2]: #feat-status-takes-the-write-commands-scope
[3]: #feat-vc-x1-install-copies-the-release-build
[4]: #feat-the-release-stamp-goes-cargo-tracks-staleness
[5]: #feat-install-takes---dir
[6]: #feat-vc-x1-install-dev-puts-the-dev-build-on-the-path
[7]: #feat-vc-x1-run-and-run-dev-run-the-builds
[8]: #feat-vc-x1-install-stages-deploys-and-runs-builds-closing
