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

### feat: the other commands take flags only

Picked from `## Todo` as **feat: the other commands take flags only** (wink + agent, 2026-10-27),
split from **feat: push, commit, and squash-push take flags only**, with **build, install, and run
take --dev** and **init and clone take --scope por** folded in at that cycle's opening, and the
`agent-files` subcommands added at this one's.

#### Problem

**feat: push, commit, and squash-push take flags only** made every argument of the three commands
that publish a flag, and the rest still take positionals: a revision and a count on the read
commands, a scope or a file on `lookup`, a file on the file commands, a side or a path on `config`
and `validate-config`, directories on the `agent-files` subcommands, and a source or target and a
name on `init`, `clone`, and `symlink`. So a word on the command line still reads by its place, and
the rule is not yet one the README can state.

Two of the folded entries are the same kind of problem:

- `build-dev`, `install-dev`, and `run-dev` are `build`, `install`, and `run` with the dev build,
  each's help saying so, and a suffix reads as a command of its own, so `land-dev` seemed a
  two-step Land. Found at **feat: vc-x1 land** (wink + agent, 2026-10-26).
- `init` and `clone` alone spell the single-repo form `--por`, where the write commands spell it
  `-s p|por`, and creating a POR from the docs took four tries: `init --por <name>` stops on "no
  account specified", `--account` is read as a GitHub owner when it names an `[account.<name>]`
  section, and only the URL target, which needs no config, works (wink + agent, 2026-10-26).

#### Solution

Every argument a flag, the command names aside, each old positional kept hidden and refused with
the flag that replaces it by `options_flags::retired_positional::refusal`, as the first cycle did.
The flag names marked provisional are settled at their rung.

- `chid`, `desc`, `list`, `show`, `validate-desc`, `fix-desc`: `[REVISION] [COMMITS]` go, since
  `-r, --revision` and `-n, --commits` are there already.
- `lookup`: the side is `-s`, which [Scope is a flag on every command, and push resolves its own
  bookmarks](TODO.md#scope-is-a-flag-on-every-command-and-push-resolves-its-own-bookmarks) already
  asks for, and the line `--line FILE:LINE`.
- `agent-session --file PATH`, `validate-anchors --file PATH`, repeatable, and `validate-todo` and
  `fix-todo --file PATH`, `TODO.md` by default.
- `config` and `validate-config`: `[TARGET]`, a side keyword or a path, splits into `-s` for the
  side, `work,agent` by default, and `--file PATH` for a config file, the user config included.
- `agent-files diff --from A --to B`, `agent-files copy --from SRC --to DST`, and `agent-files size
  --dir DIR`, provisional, their defaults unchanged.
- `init --target URL|PATH|NAME --name NAME`, provisional, `--repo` being taken, `clone --from
  URL|PATH --name NAME`, provisional, and `symlink --target PATH`.
- `init` and `clone` take `-s/--scope`, `both` by default and `por`, `work` and `agent` refused,
  and `--por` errors naming `--scope por`:
  - the dual-only and POR-only checks (`--config`, `--agent-dir`, `--agent-repo`, `--agent-suffix`,
    `--use-template`'s `,BOT`) move to the scope
  - the no-account and unknown-account errors name the URL target as the way that needs no config,
    the user config's path, and a minimal `[account.<name>]` section
  - stale help fixed: init's "Create a new dual-repo project" and `--por`'s "`.claude/`
    companion", `--use-template`'s `--scope=por`, and `--config`'s `.vc-config.toml`.
- `build`, `install`, and `run` take `-d/--dev`, choosing the config's `release`/`dev` pairs, and
  `--release` the release profile, so `install --dev --release` installs a release candidate beside
  the landed binary. `build-dev`, `install-dev`, and `run-dev` stay as hidden commands refused with
  the new spelling. `run` keeps its trailing arguments, the built binary's, `--dev` before them.
- The README states the rule in its CLI section, and every command's section and summary line uses
  the flags. `clippy.toml`'s spawn register entry 7, the completion tests, and the agent-files'
  `install-dev`, the last told to **agent-files(proposal): v0.3.1**.

#### Acceptance check

`cargo test` passes with a test per command that its old positional is refused naming its flag,
and per renamed `-dev` command that it is refused naming `--dev`. Every subcommand's `vc-x1-dev
<cmd> --help` Usage line shows no positional, `run`'s trailing `[ARGS]...` aside. Live: `vc-x1-dev
install --dev` installs the dev build as `vc-x1-dev`, `vc-x1-dev run --dev -- -V` prints the dev
build's banner, `vc-x1-dev show -s both @-` is refused naming `-r @-`, and `vc-x1-dev init -s por
--target <url> --dry-run` plans a POR.

#### Deliberation

- One cycle for the rest, the folded entries with it, wink's call (2026-10-27): each is a command's
  arguments brought to the flags, and the README states the rule once all of them hold it.
- The `agent-files` subcommands join at this opening: the Todo entry listed the commands from the
  top-level help, and `diff`, `copy`, and `size` take directories by position.
- `config` and `validate-config` split `[TARGET]` in two, since it takes a side keyword or a path
  in one place, the ambiguity the rule removes, and `-s` is what a side is everywhere else.
- `init --target`, not `--repo`, which already names the remote's category and value.
- The `-dev` rung comes last among the commands, since every rung before it installs with
  `install-dev`, and from it on the cycle installs with `install --dev`.
- The provisional flag names stand as written, wink's go at the opening's review (2026-10-27),
  each still a rung's to settle if its code shows a reason.
- Waiver, wink's "go, complete cycle except for landing on main" (2026-10-27), at the opening's
  review.
  - Covers the opening and every rung through the closing: the work review, the description review,
    the per-push approval, and the hard stop after each push.
  - Does not cover Land: the closing is pushed without `--land`, and the close-out shape is the
    default, trapezoid, until wink chooses at the Land.
  - Validation still runs before every push, and a deviation from the plan is still a stop.

#### Ladder

- [feat: the other commands take flags only opening][o0] (done)
- [feat: the read commands take -r and -n only][o1] (done)
- [feat: lookup takes -s and --line][o2] (done)
- [feat: the file commands take --file][o3]
- [feat: config and validate-config take -s and --file][o4]
- [feat: agent-files diff, copy, and size take flags][o5]
- [feat: init, clone, and symlink take flags and --scope por][o6]
- [feat: build, install, and run take --dev][o7]
- [docs: the README states every argument is a flag][o8]
- [feat: the other commands take flags only closing][o9]

##### feat: the other commands take flags only opening

Opens the cycle: the Todo and its two folded entries moved here, the `agent-files` subcommands
added, the version bumped.

##### feat: the read commands take -r and -n only

`chid`, `desc`, `list`, `show`, `validate-desc`, and `fix-desc` lose `[REVISION] [COMMITS]`, kept
hidden and refused naming `-r` and `-n`, their flags already there.

- `retired_positional::refuse_rev_count`, the pair's refusal, `-r X -n N` or either alone, beside
  `refusal`, both through one wording. The hidden count is a `String`, so `list @ x` is told
  `-r @ -n x` rather than clap's parse error.
- `CommonArgs::refuse_retired`, called first in the four `to_params`, and `validate-desc` and
  `fix-desc` call the pair's refusal themselves, their args being their own.
- `common::resolve_spec` takes `-r` and `-n` alone, the flag-over-positional precedence gone with
  the positionals, and its tests lose the two that tested that precedence and gain a named revision
  with a count.
- The README's "Positional shorthand" section is "Revision and count", its examples in the flag
  form, and its other `list` and `fix-desc` examples follow.
- `tests/cli_smoke.rs` called `chid @` five times, found by the full validation as the first
  cycle's `cli_sync` call was, the `.arg("chid").arg("@")` spelling missing the search for the
  array form. No agent-file and no support script calls a read command with a positional.

##### feat: lookup takes -s and --line

`lookup`'s `SCOPE FILE:LINE` positionals become `-s` and `--line`, the scope-then-target form gone.

- `--line FILE:LINE` is required unless the retired positionals were given, so a bare `lookup`
  still gets clap's "required" error and an old invocation gets the refusal, `--line X` for one
  word and `-s S --line X` for two. `-s` was already the side's flag form.
- `parse_side` went with the positional scope it parsed, `-s`'s own parser being one side only, so
  "SCOPE given twice" has nothing left to catch.
- The completion test's two `lookup` cases complete `--line`'s value, which keeps the `FilePath`
  hint, and the README, the help, and `notes/transcript-write.md`'s two statements of the form
  name the flags.

##### feat: the file commands take --file

`agent-session`, `validate-anchors`, `validate-todo`, and `fix-todo` take their file by `--file`,
repeatable for `validate-anchors`.

##### feat: config and validate-config take -s and --file

`[TARGET]` splits into `-s` for a side and `--file` for a config file's path.

##### feat: agent-files diff, copy, and size take flags

`diff` and `copy` take their two directories by flag, and `size` its one.

##### feat: init, clone, and symlink take flags and --scope por

`init`'s and `clone`'s target and name, and `symlink`'s target, become flags, and `init` and
`clone` take `-s both|por`, `--por` refused naming it, with the account errors and stale help
fixed.

##### feat: build, install, and run take --dev

`build`, `install`, and `run` take `--dev` and `--release`, and the `-dev` commands are kept hidden
and refused naming `--dev`.

##### docs: the README states every argument is a flag

The README's CLI section states the rule and its one exception, `run`'s trailing arguments, and the
agent-files' `-dev` invocations are told to the v0.3.1 proposal.

##### feat: the other commands take flags only closing

Closing out the cycle.

# References

[o0]: #feat-the-other-commands-take-flags-only-opening
[o1]: #feat-the-read-commands-take--r-and--n-only
[o2]: #feat-lookup-takes--s-and---line
[o3]: #feat-the-file-commands-take---file
[o4]: #feat-config-and-validate-config-take--s-and---file
[o5]: #feat-agent-files-diff-copy-and-size-take-flags
[o6]: #feat-init-clone-and-symlink-take-flags-and---scope-por
[o7]: #feat-build-install-and-run-take---dev
[o8]: #docs-the-readme-states-every-argument-is-a-flag
[o9]: #feat-the-other-commands-take-flags-only-closing
