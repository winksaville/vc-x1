# vc-x1

- [Overview](#vc-x1)
- [Architecture](ARCHITECTURE.md)
- [Terminology](#terminology)
- [Build and install](#build-and-install)
- [Usage](#usage)
  - [Shell completion](#shell-completion)
  - [status](#status)
  - [agent-files](#agent-files)
  - [validate-desc](#validate-desc)
  - [fix-desc](#fix-desc)
  - [validate-todo](#validate-todo)
  - [fix-todo](#fix-todo)
  - [validate-anchors](#validate-anchors)
  - [validate-config](#validate-config)
  - [validate-agent](#validate-agent)
  - [config](#config)
  - [clone](#clone)
  - [init](#init)
  - [symlink](#symlink)
  - [sync](#sync)
  - [squash-push](#squash-push)
  - [push](#push)
  - [Testing push + squash-push](#testing-push--squash-push)
  - [Testing the ochid-trailer guard](#testing-the-ochid-trailer-guard)
- [Cross-repo Linking with Git Trailers](#cross-repo-linking-with-git-trailers)
- [jj Tips for Git Users](#jj-tips-for-git-users)
- [The session lock](#the-session-lock)
- [Thoughts for the future](#thoughts-for-the-future)
- [Contributing](#contributing)
- [License](#license)

This is experiment 1 to explore creating a Vibe Coding (vc) environment. We will investigate ways of
using the dual jj-git repo concept, explored in
[hw-jjg-bot](https://github.com/winksaville/hw-jjg-bot.git) to initially make it easy to see how the
code base evolved. This is made possible by the fact that we have two repos one with the code and
one with the conversation with the bot.

I've chosen the jj-git environment because jj provides the concept that each commit has an immutable
changeID as well as the mutable commitID of git. The idea is that each commit made on repo A writes
the changeID in the commit message to repo B. Thus there is a cross reference between the two repos
and this will allow vc-x1 to show how the repo evolved and the entity (bot or human) can more
clearly understand **how** and most importantly **why** the code evolved.

The solution space is wide open, from trivial CLI, web or app based (mobile/non-mobile). In
addition, I could see this as an extension to existing programming editors like vscode and zed or
even creating our own IDE for vc.

See [Initial commit with dual jj-git
repos](./notes/chores/chores-01.md#initial-commit-with-dual-jj-git-repos) for how the initial commit
was created with the dual jj-git repos. After doing so and I then created this README.md file.

## Terminology

A vc-x1 workspace pairs two jj-git repos, and this README names their two sides **work** and
**bot**:

- **work repo**: the repo at the project root, holding the work product itself (source code, docs,
  the repo you would have anyway).
- **bot repo**: the `.claude` repo nested inside it, holding the bot's session data (the
  conversation that produced the work).
- **work** / **bot**: the two sides as roles, used wherever a command or doc needs to say which repo
  it means (e.g. push's `commit-work` / `commit-bot` stages).

A few naming conventions this project keeps, to avoid ambiguous jargon:

- **path** is reserved for a filesystem location, qualified as **relative** or **absolute** when it
  matters (e.g. `../other`, `/home/you/repo`).
- a configuration setting is a **config key**, e.g. `agent-session.col-width` (its TOML table and key
  joined by `.`), not a "path".
- a datum in a transcript's `--fields` inventory is a **field**, e.g. `message.content[].type`, its
  levels joined by `.`, with `[]` marking array elements.

## Build and install

vc-x1 is not a build system. The config says what the steps are, the build and install commands
and where the binaries land, and vc-x1 runs them in order and checks their results: it builds
before it runs or installs, reports what it is about to copy, refuses only a copy that would be
wrong, and never lets the dev build replace the release. Cargo, not vc-x1, decides what to
rebuild.

A plain Rust crate: `cargo build` and `cargo test` of one binary, `vc-x1`, named by the
manifest's `[package].name`, which never changes. The `[build]` table in `.vc-config.md` holds two
builds, each a plain `cargo build` leaving its binary in `target/`, off the PATH, so nothing a
cycle builds replaces the `vc-x1` on the PATH until the cycle lands. Both are `--locked`, so they
compile the same dependencies, the ones `Cargo.lock` commits:

- Dev: `vc-x1 build-dev` runs `[build] dev`, `cargo build --locked`, leaving the rung's debug
  binary at `target/debug/vc-x1`. After a validation it reuses the test build, so it costs under a
  second, and `vc-x1 run-dev` runs it first, so what runs is always the current code.
- Release: `vc-x1 build` runs `[build] release`, `cargo build --release --locked`, leaving the
  release build at `target/release/vc-x1`, its `-V` naming the commit it was built from
  ([build](#build)).
- Install: at a cycle's Land, `vc-x1 install` runs the release build, which cargo skips when
  nothing changed, then copies it into `~/.cargo/bin`, the `[install] release-dir`, after
  reporting what it is copying ([install](#install)).
- Install-dev: mid-cycle, `vc-x1 install-dev` runs the dev build, then copies it into
  `~/.cargo/bin`, the `[install] dev-dir`, as `vc-x1-dev`, beside the landed `vc-x1`, so the
  cycle's code can be tried from any directory ([install-dev](#install-dev)). With `install`, the
  two commands that touch the PATH.

Where each build leaves its binary is the config's to say, `[build] release-bin` and `dev-bin`:
vc-x1 never works it out from cargo or the package's name.

The commands in the order a cycle uses them, a rung's loop first and Land after:

| Command | What it does | When |
|---|---|---|
| `vc-x1 validate` | fmt, clippy, every test | every rung, before review |
| `vc-x1 validate --fast` | the unit tests only | while iterating |
| `vc-x1 build-dev` | the dev build, into `target/debug` | to build without running |
| `vc-x1 run-dev` | `build-dev`, then runs it | trying the change |
| `vc-x1 install-dev` | `build-dev`, then copies it into `~/.cargo/bin` as `vc-x1-dev` | trying the change elsewhere |
| `vc-x1 build` | the release, into `target/release` | at Land, or to try the release profile |
| `vc-x1 run` | `build`, then runs it | the smoke check before install |
| `vc-x1 install` | `build`, then reports it and copies it into `~/.cargo/bin` | at Land |

`validate` checks the source and makes no build of either kind. `run`, `run-dev`, `install`, and
`install-dev` build first, as `cargo run` does, and cargo rebuilds only what changed, so the binary
`run` smoke-tests is the one `install` copies.

- `target/debug/vc-x1 -V` names the exact rung, since the version-of-record carries the cycle's
  suffix (`0.85.6-1`), and the binary banners the name it was invoked as (`argv[0]`), so a copy
  or a symlink self-reports truthfully.
- Before 2026-10-20 a cycle renamed the package to `vc-x1-dev` instead, guarded by `build.rs`
  (the single-name convention, 0.78.3, design record in `notes/chores/chores-16.md`). The rename
  and the guard are gone: a build script cannot see an install's root, and building off the PATH
  makes the rename unnecessary.

The first install, and any install when the `vc-x1` on the PATH is too old to have `install`, is
made from the source: `cargo run --locked -- install` builds the dev binary and has it build the
release and install it, report and all.

vc-x1 links jj-lib and refuses to run when the installed `jj`'s version disagrees with it
(override per invocation with `--allow-jj-mismatch`, and see `vc-x1 version`).

## Usage

```
vc-x1 list [-r REVISION] [-n COMMITS]  # List commits in a jj repo
vc-x1 desc [-r REVISION] [-n COMMITS]  # Show full description of a commit
vc-x1 chid [-r REVISION] [-n COMMITS]  # Print changeID(s) for a revision
vc-x1 show [-r REVISION] [-n COMMITS]  # Show commit details and diff summary
vc-x1 status [-s SCOPE] [-R PATH]          # Working-copy status by scope, w|a|b|p, and the clean verdict (alias st)
vc-x1 lookup [-s S] --line FILE:LINE     # The other repo's window for a line in this one
vc-x1 agent-session --file F [OPTS]      # Display a session transcript as a conversation
vc-x1 validate-desc [OPTS]                 # Validate commit descriptions
vc-x1 fix-desc [OPTS]                     # Fix commit descriptions (dry-run default)
vc-x1 validate-todo [--file F]            # Check todo-file entry numbering
vc-x1 fix-todo [--file F]                 # Renumber todo file (dry-run default)
vc-x1 validate [--fast] [OPTS]            # Run the workspace's configured validation commands
vc-x1 validate-agent [OPTS]               # Check the agent repo is published
vc-x1 config [OPTS]                       # Print / validate settable config keys
vc-x1 clone <REPO> [NAME] [OPTS]          # Clone a dual-repo project
vc-x1 init <TARGET> [OPTS]                # Create a new dual-repo project
vc-x1 symlink [TARGET] [OPTS]             # Create Claude Code project symlink
vc-x1 sync [OPTS]                          # Fetch + sync a repo, or both, to its remote
vc-x1 squash-push -s a [--bookmark NAME]  # Squash @ into @-, advance a bookmark, push
vc-x1 commit -s b --title T --body B       # Commit both working copies as a rung's pair
vc-x1 land -s b --onto T [--source S]      # Land a made cycle: fast-forward, push, delete its bookmark
vc-x1 squash -s b [--title T --body B]     # Fold the working copies into the pair
vc-x1 merge -s w --target T --from S...    # Add sources to a target's parents, keeping its change
vc-x1 move-bookmark -s w --bookmark B --target T  # Point an existing bookmark at a commit, locally
vc-x1 push -s b [-r REV] [--bookmark NAME] # Publish a rung's pair, or a name for a commit made
vc-x1 version                              # Report vc-x1, agent-files, jj-lib, jj, and jj-data versions
vc-x1 agent-files version                  # Print the workspace's agent-files set version, bare
vc-x1 agent-files diff [A] [B] [-c]        # Name the set files that differ between two copies of the set
vc-x1 agent-files copy [SRC] [DST] [-c]    # Make DST's set a copy of SRC's, uncommitted
vc-x1 agent-files size [DIR]               # Count the set's lines, and slide the table that records them
vc-x1 --version                            # Print version, with the workspace's agent-files version
vc-x1 --help                           # Print help
```

### jj version gate

vc-x1 embeds jj-lib and writes the same repo data the installed `jj` reads, so every subcommand
refuses to run when the installed `jj`'s version differs from the embedded jj-lib's. The
`version` subcommand is the one exception: it reports the verdict instead of acting on it, and
withholds its `jj-data` lines on a mismatch. The rule and its rationale live in
[notes/jj-version-policy.md](notes/jj-version-policy.md).

**The `..` notation:** dots on the revision show which direction to list commits:

- `x..`: x at top, ancestors below (older commits)
- `..x`: descendants above (newer commits), x at bottom
- `..x..`: both directions, x in the middle

COMMITS is the total number of commits to show (x is always included). Without dots, `x` with a
count defaults to `x..` (ancestors). For `..x..`, the budget is split: ancestors get the extra on
odd counts.

```
vc-x1 list -r @              # just @ (1 commit)
vc-x1 list -r @.. -n 5       # @ + 4 ancestors (5 commits, @ at top)
vc-x1 list -r ..@ -n 3       # 2 descendants + @ (3 commits, @ at bottom)
vc-x1 list -r ..@.. -n 3     # 1 descendant, @, 1 ancestor (3 commits)
vc-x1 list -r ..@.. -n 4     # 1 descendant, @, 2 ancestors (4 commits)
```

### jj config and the new-file size limit

vc-x1 reads jj's config as `jj` does: the defaults, the user's (`~/.config/jj/config.toml`, or
`$JJ_CONFIG`), and the repo's own, which jj keeps under the user's config dir and `jj config set
--repo` writes.

A snapshot takes no new file over `snapshot.max-new-file-size`, 1 MiB by default. `jj` warns and
goes on without it, but in vc-x1 that is an error, at every subcommand that snapshots, push
included, before either repo commits, so nothing is committed without a word. The error
names each refused file and its size, and the fixes:

- add the file to `.gitignore`
- raise the limit for that repository only, `jj -R <repo> config set --repo
  snapshot.max-new-file-size <size>`
- in an agent-repo, whose files are its session transcripts, lift the limit there, `jj -R
  <agent-repo> config set --repo snapshot.max-new-file-size 0`, 0 meaning no limit

vc-x1 never sets the limit itself, since a repo's value overrides the one the user chose. A file
the limit refused stays out until one of these is done, and then the command is run again.

### Shell completion

vc-x1 provides tab completion for all subcommands and flags using claps
[unstable-dynamic](https://docs.rs/clap_complete/latest/clap_complete/env/struct.CompleteEnv.html)
only feature. It is a "simple" implementation and does not handle completing revisions but still
useful. To enable it, add one of the following to your shell's startup file:

```bash
# bash (~/.bashrc)
source <(COMPLETE=bash vc-x1)

# zsh (~/.zshrc)
source <(COMPLETE=zsh vc-x1)

# fish (~/.config/fish/config.fish)
source (COMPLETE=fish vc-x1 | psub)
```

Completions are generated dynamically by the binary, so they stay in sync with the installed version
automatically.

### Revision and count

REVISION is `-r`/`--revision` and COMMITS is `-n`/`--commits`, with `..` on either side of the
revision opening that side to the count:

```
vc-x1 list -r x                # just x (1 commit)
vc-x1 list -r x -n 5           # x.. 5 -> x + 4 ancestors (5 commits)
vc-x1 list -r x.. -n 5         # same as above
vc-x1 list -r ..x -n 3         # 2 descendants + x (3 commits)
vc-x1 list -r ..x.. -n 3       # 1 descendant, x, 1 ancestor (3 commits)
vc-x1 list -r ..x.. -n 1       # just x (1 commit)
```

**Defaults:** with no arguments, REVISION is `@` and COMMITS is 1 (just the working copy).

Every argument is a flag: the positionals these commands once took, `list x 5`, are refused, naming
the flag form, `-r x -n 5`.

### Multi-repo queries

`-s`/`--scope` selects workspace sides by role keyword. `work` is the work repo, `agent` is the
`.claude` repo, `both` (or the spelled-out `work,agent`) is both. The workspace root is found by
walking up from cwd (the existing `find_workspace_root` rule).

```
vc-x1 chid -s work,bot          # both sides of the workspace
vc-x1 chid -s work              # just the work repo
vc-x1 chid -s agent             # just the bot repo
vc-x1 list -r @.. -n 3 -s work,bot  # works with chid / desc / list / show
```

`-R`/`--repo` takes a single path. Used alone it points at one jj repo (workspace lookup is
skipped), and combined with `-s` it overrides the workspace root for the role lookup:

```
vc-x1 chid -R .                  # single repo at .
vc-x1 chid -R .claude            # single repo at .claude
vc-x1 chid -R ../other -s work,bot   # both sides of ../other workspace
```

Defaults preserve prior behavior: no flag -> `[.]`, `-R foo` alone -> `[foo]`. When multiple repos
are resolved (any `-s` that names more than one role), output is labeled with bold `=== path ===`
headers by default. Control the label with `-l`/`--label` and `-L`/`--no-label`:

```
vc-x1 chid -s work,agent          # default label: === path ===
vc-x1 chid -s work,bot -l "---"   # custom label:  --- path ---
vc-x1 chid -s work,bot -L         # no label (raw output)
```

Examples:

```
$ vc-x1 chid -s work,bot
=== . ===
kwoyvposvsmv

=== .claude ===
zzpksklxyrnw

$ vc-x1 chid -s work,bot -L
kwoyvposvsmv
zzpksklxyrnw

$ vc-x1 list -r @.. -n 3 -s work,bot
=== . ===
kwoyvposvsmv eae175c3c1b5 (no description set)
pqxtxxpnsmot b0c46930a640 Reorganize notes (0.19.1)
ssokyszzwsxw eeb15bc6d839 Unify .. notation and CLI (0.19.0)

=== .claude ===
zzpksklxyrnw 2388185e42ee (no description set)
tzupykyyvnrp 81ce22e34d41 Reorganize notes (0.19.1)
slmkmroqtqtp 1c33ccaa567d Unify .. notation and CLI (0.19.0)

$ vc-x1 desc -r @- -s work,bot
=== . ===
pqxtxxpnsmot b0c46930a640 Reorganize notes: move older done items to done.md, update todos (0.19.1)

    Move completed items [1]-[13] and their references from todo.md to done.md.
    Replace completed jj-organization todo with new items.

    ochid: tzupykyyvnrp

=== .claude ===
tzupykyyvnrp 81ce22e34d41 Reorganize notes: move older done items to done.md, update todos (0.19.1)

    Session: reviewed and committed notes reorganization for 0.19.1.

    ochid: pqxtxxpnsmot

$ vc-x1 chid -s work,bot -l "---"
--- . ---
kwoyvposvsmv

--- .claude ---
zzpksklxyrnw
```

Most decoration strings work unquoted (`---`, `===`, `>>>`, `:::`, `+++`). Use single quotes for
strings containing shell metacharacters like `*`, `!`, `#`, `$`, or `~` (e.g. `-l '***'`). Double
quotes won't protect against `!` (bash history expansion).

With a single repo resolved, no label is printed, backward compatible with the no-flag default.
Multi-repo (`-s work,bot`) is supported for `chid`, `desc`, `list`, and `show`. `squash-push`
remains single-repo.

`-s` is keyword-only: `work`, `agent`, `work,agent`, `agent,work`. Path-based single-repo operation uses
`-R` (above).

### Write scopes

The write commands, and `status`, take `-s` as one of four scopes: `w|work` the work repo, `a|agent`
the agent repo, `b|both` the pair, and `p|por` a Plain Old Repo, one repo with no pair, the one
`-R` names. `-s` is required wherever a write command takes it, so none guesses which repos it
acts on, and each takes the scopes that make sense for it, refusing the others with the reason.

Scopes: `w`=work, `a`=agent, `b`=both, `p`=por.

| Command | Scopes | Why the subset |
|---|---|---|
| `sync`, `squash`, `squash-push`, `push`, `status`, `move-bookmark`, `delete-bookmark` | `w a b p` | |
| `validate-desc` | `w a b` | a plain repo has no other repo to check its descriptions against |
| `commit` | `w a b p` | `a` refused with a REV, with `--opening` or `--closing`, and on an agent `@` paired to a work commit |
| `land` | `b p` | `w` could publish trailers naming agent commits left unpublished, and `a` has no cycle |
| `merge` | `w p` | the agent repo never holds a merge |

The rule: a scope is valid when what it publishes names nothing it leaves unpublished. A commit made
by `commit -s b` carries an `ochid:` trailer naming its partner in the other repo, so publishing
one side of a pair alone can leave a trailer naming a commit origin does not have. The repo
commands act on the agent repo as a repo of its own and link nothing, so they take `a`. The cycle
commands, which make or publish a cycle's commits, refuse it or hem it in, since a cycle runs on
the work repo's bookmark and the agent repo has none.

### status

Working-copy status of the workspace's repos in one call, alias `st`, and the verdict At rest's
"clean" asks for. Each scoped repo prints under its label, `work` for the work repo and the agent
repo's directory name for the agent repo, in the shape `jj st` prints: the changed paths with
their letter (`M`, `A`, `D`, a rename showing as a `D` and an `A`), the `Working copy (@)` line,
and one `Parent commit (@-)` line per parent. One verdict line ends the output:

- `status: clean` when every scoped `@` is empty and has no description
- `status: dirty: <label> @ has changes, ...` naming each repo that is not, and why: `@ has
  changes` or `@ is described`, since an empty described `@` is an intent nothing has published

The scope is `-s`/`--scope`, the write commands' four, `w|work`, `a|agent`, `b|both`, or `p|por`,
`work` by default. `both` is the At rest check, both repos' `@` empty. `por` reports a plain repo
as itself, labeled `por`, resolved as the write commands resolve it, so it is refused on either
side of a workspace. Tab completion offers the four long names. The exit status is success either
way: the verdict is the line, not the code. The positional scope, `status both`, and the comma
forms `work,agent` and `agent,work` went in 0.85.7, so a scope reads the same on every command.

The workspace is `-R`/`--repo` when given, else found from the current directory. A plain jj repo
with no vc-x1 config answers for `work` as itself, and `agent` there is an error naming the
missing config. A plain repo nested inside a workspace's tree, a scratch repo under `tmp/`,
answers as itself too: the nearest jj repo wins unless it is one of the workspace's own sides, so
running from inside the agent repo still means the workspace.

```
vc-x1 status                    # the work repo, the default scope
vc-x1 st -s b                   # both repos and the At rest verdict
vc-x1 st -s a                   # the agent repo alone
vc-x1 st -s p -R ../scratch     # a plain repo, as itself
vc-x1 st -s b -R ../other       # ../other's workspace
```

```
work (/home/me/proj):
Working copy changes:
M TODO.md
Working copy  (@) : xxolwztqspnz 5e7322c164bb (no description set)
Parent commit (@-): plnxxqtpwqrs 48d678c8efb4 main | feat: the last landed cycle

.agent-session (/home/me/proj/.agent-session):
The working copy has no changes.
Working copy  (@) : kpuqynnomnxv 360bdc189b1c (empty) (no description set)
Parent commit (@-): vtkwkumoqlpx b424f97b1a2c main | feat: the last landed cycle

status: dirty: work @ has changes
```

### lookup

Show the lines in the other repo that a line in this one relates to. `vc-x1 lookup [-s SCOPE]
--line FILE:LINE` takes a line as an editor, a compiler, or grep prints it, places the file on one
side of the workspace, and resolves it to the window on the other side, work-to-transcript or
transcript-to-work. The requirements are in [What the lookup command
needs](notes/transcript-write.md#what-the-lookup-command-needs).

- `-s`/`--scope` is the side the line is on, `work` or `agent`. Omitted, it is inferred from the
  file's path. Named, a `FILE` that does not exist relative to the current directory is taken
  relative to that side's repo root, so a repo-relative path works from anywhere once the side is
  said. A named side the path disagrees with is an error, never a hint.
- `--line FILE:LINE` splits at its last colon, so a path holding a colon still parses, and the line
  must exist in the file on disk. The old `[SCOPE] FILE:LINE` positionals are refused, naming `-s`
  and `--line`.
- The workspace is `-R`/`--repo` when given, else found from the file.
- `-r`/`--revision REV` reads the line from the file as of REV instead of from disk, for a line
  the working copy no longer holds, such as a cycle-record line the next opening deleted.

A work line prints the commit it arrived in, blamed at the working copy after a snapshot, and
that commit's partner by its `ochid:` trailer. When the line moved, the commit that first wrote
it and that commit's partner follow. A commit with no trailer prints its candidate partners, the
agent commits within `--tolerance` seconds of it, 60 by default, nearest first:

```
work TODO.md:21
    The first decision needs revising.
commit  kxpuoxoznwyn TODO.md:21 fix: the design note closing
written yuoqvtxvozko fix: the design note opening
partner vxkqvspultnv fix: the design note closing
written partner qxqzomnwkonk fix: the design note opening
```

An agent line, a session file's line, prints as the rendered transcript entry, then the agent
commit that appended it, blamed at the agent repo's working copy after a snapshot, and every work
commit that commit's `ochid:` trailers name, the work commits its push published. Each work
commit's diff is the window, printed as `FILE:START-END` regions with their removed and added
lines. When the line is a transcript write the window narrows to the files it wrote, one for a
`Write` or `Edit` and every changed file a `Bash` command names, and any other line is discussion
and takes the diff whole. A line not yet pushed says so:

```
agent 1d000000-0000-4000-8000-000000000002.jsonl:17
    1d000000-0000-4000-8000-000000000002.jsonl:17 2026-09-12 20:24:58Z tool Edit: .../dr-1/design.md
commit  upzpmzpyuqzo 1d000000-0000-4000-8000-000000000002.jsonl:17 fix: revise the first decision
partner ntuwppkqupwv fix: revise the first decision
write   design.md in ntuwppkqupwv
window  design.md:3-3
  - The first decision.
  + The first decision, revised and checked.
```

```
vc-x1 lookup --line TODO.md:53                   # the side inferred from the path
vc-x1 lookup -s agent --line e355f8b2.jsonl:1200  # repo-relative under the agent repo
vc-x1 lookup -s work --line TODO.md:53 -R ../proj # a workspace elsewhere
vc-x1 lookup --line TODO.md:11 -r main-          # the line as of a revision
```

### agent-files

The agent-files set is `AGENTS.md`, the files under `agent-data/`, and the project layer
`custom.md`, as [AGENTS.md](AGENTS.md#terminology) defines it. The group runs before the
workspace loads, so it answers in a plain repo too.

`agent-files version` prints the set version, the name of the `agent-data/agent-files-vX.Y.Z`
file with its prefix removed, bare, one per line when a bump left several, and fails outside a
workspace or in one without the file.

`agent-files diff [A] [B]` compares two copies of the set, one line per file, names only: `same`,
`differs`, `only in A`, or `only in B`, then `N of M differ`. It answers whether a re-sync is a
copy, and its exit status is non-zero when anything differs, as `diff`'s is. `custom.md` is
reported as the project layer and not compared unless `-c`/`--custom` compares it like the rest,
for a family whose project layers are meant to be identical, and `--no-custom` overrides a config
that says so.

`A` is the operand, else the config's `agent-files.diff.dir`, else `family.template`, the config
values relative to the config file's directory. `B` is the operand, else this workspace. So a
bare `diff` compares the payload against this workspace, one operand compares that copy against
this workspace, and two operands compare any two copies from anywhere, no workspace needed. The
header line says where each came from, and a missing or non-directory operand is an error naming
what to set.

`agent-files copy [SRC] [DST]` makes the set in `DST` a byte copy of the one in `SRC`: it copies
what differs or exists only in `SRC`, deletes what exists only in `DST` under `agent-data/`,
never touches `TODO.md`, and copies `custom.md` only with `-c`/`--custom`. The operands resolve
as `diff`'s do: `SRC` is the operand, else `agent-files.copy.dir`, else `family.template`, and
`DST` is the operand, else this workspace. So a bare `copy` re-syncs this workspace from the
payload, and two operands copy between any two directories, which is how a maintainer folds an
adopter's set into the payload. It prints its source and destination and each step first,
refuses a `DST` whose jj working copy already changes a set file, so the copy's changes are the
only ones there, and leaves the result uncommitted: `jj diff` there is the review, and the
commit is yours to make. A `DST` outside any jj repo gets no guard, and the run says so.

`agent-files size [DIR]` counts the set's markdown, `AGENTS.md`, `custom.md`, and
`agent-data/*.md`, one line per file and a total. The version marker has no `.md` extension, so it
takes no row and no place in the file count, and being empty it would add nothing to the total
either. `agent-data/rationale.md` is counted: its number is shown in angle brackets and left out
of the total, being the rules' why rather than a rule, so a rule gaining a why does not read as
the set growing.

It then slides the per-file table in `notes/agent-files-size.md`, the one headed `| File |`: a new
leftmost column labelled by the set's version file goes in, the rightmost column goes out, and the
window keeps the width it had. A file new to the set takes its sorted slot with empty cells behind
it, one gone from the set keeps its remaining numbers until the window passes them, and the `total`
row stays last. It is dry-run by default, printing the table it would write, and `--no-dry-run`
applies it. `DIR` is the operand, else this workspace, `--file` names another notes file, and
`--label` names a column the version file cannot: write `--label=- v0.1.0` for a label opening on
a `-`. The `## Counts` table above is left alone, since its note is prose.

```
vc-x1 agent-files diff                             # the payload against this workspace
vc-x1 agent-files diff ../iiac-perf                # a peer's copy against this workspace
vc-x1 agent-files diff ../iiac-perf -c             # custom.md too
vc-x1 agent-files diff ../iiac-perf ../zc-ring-x1  # two peers, from anywhere
vc-x1 agent-files copy                             # re-sync this workspace from the payload
vc-x1 agent-files copy ../iiac-perf -c             # take a peer's set, custom.md included
vc-x1 agent-files copy ../iiac-perf ../vc-x1-template/work  # fold a peer's set into the payload
vc-x1 agent-files size                             # the count, and the slide it would make
vc-x1 agent-files size --no-dry-run                # slide the table
```

```
agent-files copy: from ../vc-x1-template/work (family.template) into /home/me/proj (this workspace)
copy   AGENTS.md
copy   agent-data/notes.md
delete agent-data/agent-files-v0.1.0
3 step(s) applied, left uncommitted for review
```

```
agent-files size: /home/me/proj (this workspace)
AGENTS.md                     384
custom.md                      12
agent-data/code.md             94
agent-data/rationale.md     <543>
agent-data/versioning.md      204
9 files, 1774 lines
notes/agent-files-size.md: column v0.2.6 in, column v0.2.3 out
| File | v0.2.6 | v0.2.5 | v0.2.4 |
|---|---:|---:|---:|
| AGENTS.md | 384 | 384 | 384 |
| total | 1774 | 1774 | 2315 |
re-run with --no-dry-run to apply
```

```
agent-files diff: ../vc-x1-template/work (family.template) against /home/me/proj (this workspace)
AGENTS.md                       differs
agent-data/agent-files-v0.1.0   only in /home/me/proj
agent-data/code.md              same
agent-data/notes.md             differs
custom.md                       project layer, not compared (-c compares it)
3 of 4 differ
```

### agent-session

Display a Claude Code session transcript (`.claude/<uuid>.jsonl`) as a readable conversation. The
transcript format is undocumented and evolves, so parsing is tolerant: unknown entry types are
skipped and counted, and malformed lines (e.g. a live session's truncated last line) warn to
stderr and never
fail the run.

For what the transcript format looks like (entries, entry types, the `.`/`[]` field notation) and an
example that walks a small sample through these commands, see
[notes/transcript-format.md](notes/transcript-format.md).

Output is composed of eight **items**, each independently toggleable:

| Item | Emits | Default |
|------|-------|---------|
| `summary` | trailing stats line (shown / hidden / skipped counts) | on |
| `headers` | `=== role 2026-07-17 04:17:09Z ===` turn headers (UTC) | on |
| `user` | typed user prompts | on |
| `assistant` | assistant reply text | on |
| `tool` | `[tool] Name: gist` call one-liners | on |
| `thinking` | `[thinking]` blocks | off |
| `results` | `[result]` / `[result:error]` lines (capped by `--result-lines`) | off |
| `meta` | meta user lines, `--- system ... ---` lines, sidechain entries | off |

Toggle with `--<item>` / `--no-<item>` (last one wins). `--all` / `--none` reset the base (every
item on / off) and then per-item flags adjust (`--no-all` and `--no-none` are aliases of `--none`
and `--all`). Bookkeeping line types (file-history snapshots, progress markers, ...) are never
rendered, and they're counted in the summary as "skipped".

Default items resolve git-style, most specific wins:

1. CLI `--<item>` / `--no-<item>` flags
2. the work side's `.vc-config.md`: `[agent-session].items`
3. user `~/.config/vc-x1/config.toml`: `[agent-session].items`
4. built-in: `headers,user,assistant,tool,summary`

```toml
# In ~/.config/vc-x1/config.toml (all your workspaces) or
# <workspace>/.vc-config.md (this workspace, committed):
[agent-session]
items = "headers,user,assistant,tool,summary"
```

```
# Default conversation view
vc-x1 agent-session --file .claude/<uuid>.jsonl

# Everything: thinking, tool results, meta/system too
vc-x1 agent-session --all --file F

# Prompts only: what was asked, nothing else
vc-x1 agent-session --none --user --file F

# Default view minus tool calls and headers
vc-x1 agent-session --no-tool --no-headers --file F

# Slices by source JSONL line (same unit in every view)
vc-x1 agent-session --lines 40 --file F      # first 40 source lines
vc-x1 agent-session --lines -15 --file F     # last 15 source lines
vc-x1 agent-session --lines 100,20 --file F  # 20 lines from Index 100
vc-x1 agent-session --lines 100,-20 --file F # 20 lines ending at Index 100
vc-x1 agent-session --lines 0 --file F       # stats summary only
```

| Flag | Description |
|------|-------------|
| `--<item>` / `--no-<item>` | Add / remove one of the eight items (last one wins) |
| `--all` / `--none` | Base: every item on / off (aliases `--no-none` / `--no-all`) |
| `--lines SPEC` | Slice by source JSONL line, 0-based Index (`N` first, `-N` last, `I,C` from I, `I,-C` ending at I, `0` = summary only), the same unit in every view |
| `--result-lines N` | Max lines shown per tool result [default: 10], `0` = unlimited. Resolves CLI > the work side's `.vc-config.md` `[agent-session].result-lines` > user config > built-in, same as `[agent-session].items` |

Cut points show an `... (N source lines skipped)` marker, and a sliced run's summary ends with
`--lines selected K of M source lines`, and its stats describe only the slice. Timestamps are
shown as UTC (`Z`) exactly when the source timestamp carries it, observed always, but the format is
undocumented, so anything else would pass through verbatim.

**Alternate views.** The transcript format is undocumented and evolves, so agent-session doubles as a
schema explorer. The parser keeps every field Anthropic writes while the typed layer consumes a
known subset, and the difference is the unexplored surface:

| Flag | View |
|------|------|
| `--fields` | Field inventory, grouped by each line's `type`: every field observed (e.g. `message.content[].type`, with `.` for nesting and `[]` for array elements) with its count, value kinds, and short samples |
| `--unknown` | Like `--fields`, but only fields the typed layer does not consume: the unmodeled / new surface |
| `--raw` | Pretty-printed source lines (unparseable lines pass through verbatim), no summary or markers |
| `--per-line` | With `--fields`/`--unknown` (implies `--fields`): one fields section per source line instead of aggregating |
| `--col-width N` | First column width in these views [default: 68], and longer field names overflow. Resolves CLI > the work side's `.vc-config.md` `[agent-session].col-width` > user config > built-in, same as `[agent-session].items` |

The default 68 aligns the type column for ~99% of observed field names, and only a tail of
`snapshot.trackedFileBackups.<absolute path>.*` keys, whose embedded absolute paths can be
arbitrarily long, overflow.

`--lines` uses the same source-line unit here, e.g. `--fields --lines 0,1` inventories just the
first line. `--fields`/`--unknown` ignore item flags, and `--raw` conflicts with them.

```
# What does this format actually contain today?
vc-x1 agent-session --fields --file F

# What don't we model yet / what did Anthropic add?
vc-x1 agent-session --unknown --file F

# Inspect one entry in full (source line 42)
vc-x1 agent-session --raw --lines 42,1 --file F

# Walk a region line by line, fields-table style
vc-x1 agent-session --per-line --lines 40,5 --file F
```

### validate-desc

Read-only scan of commit descriptions against the other repo. `-s` is required and says whose
commits are checked: `w` the work repo's against the agent repo, `a` the reverse, `b` both
directions, one table per side under its label. `-R` is optional and says where, a path anywhere in
the workspace, the cwd without it. `-s p` is refused, a plain repo having no other repo. The other
repo is the workspace's other side, or `--other-repo` for one side. Reports status per commit:
`ok` (valid ochid), `lost` (ochid: lost), `none` (ochid: none), `err` (issues found), or `miss`
(no ochid trailer).

```
# Both directions, all ancestors of @ on each side
vc-x1 validate-desc -s b @..

# The work repo's commits against an explicit other repo
vc-x1 validate-desc -s w --other-repo .claude @..

# The agent repo's commits, a specific range
vc-x1 validate-desc -s a -r @.. -n 20
```

Use `--help` for the full status label legend.

### fix-desc

Fix commit descriptions against the other repo. Default is dry-run, so use `--no-dry-run` to write
changes. Reads other repo from `.vc-config.md` by default.

```
# Dry-run: show what would be fixed
vc-x1 fix-desc -r @..

# Actually fix
vc-x1 fix-desc -r @.. --no-dry-run

# Add missing ochid trailers by matching title
vc-x1 fix-desc -r @.. --add-missing

# Limit fixes
vc-x1 fix-desc -r @.. --no-dry-run -m 3

# Use a fallback for IDs not found in other repo
vc-x1 fix-desc -r @.. --fallback /.claude/lost
```

| Flag | Description |
|------|-------------|
| `--other-repo <PATH>` | Override other repo from .vc-config.md |
| `--no-dry-run` | Write fixes [default: dry-run] |
| `--add-missing` | Infer and add ochid for commits without one |
| `-m, --max-fixes <N>` | Stop fixing after N commits changed [default: all] |
| `--fallback <VALUE>` | Replacement for IDs not found in other repo |
| `--id-len <N>` | Expected changeID length [default: 12] |
| `--title <TEXT>` | Replace commit title at the same time |

Use `--help` for the full status label legend.

### validate-todo

Check that the `## Todo` and `## Bugs` sections of a todo file are numbered `1..N` in document
order, with continuation-line indent matching each entry's number-prefix width. Read-only, and exits
non-zero if any entry needs fixing.

```
# Check TODO.md (the default)
vc-x1 validate-todo

# Check a specific file
vc-x1 validate-todo --file path/to/todo.md
```

Each flagged entry prints its corrected first line and a `[line: ...]` tag: the entry's line number
and what is off (`was N`, `indent A -> B`). Use `fix-todo` to apply the fixes.

### fix-todo

Renumber a todo file's `## Todo` and `## Bugs` sections to `1..N` and normalize each entry's
continuation-line indent. Dry-run by default, printing each changed entry's corrected line so the
output is the result, and `--no-dry-run` writes the file in place.

```
# Dry-run: print the corrected lines
vc-x1 fix-todo

# Renumber the file in place
vc-x1 fix-todo --no-dry-run

# Operate on a specific file
vc-x1 fix-todo --file path/to/todo.md
```

| Flag | Description |
|------|-------------|
| `--no-dry-run` | Write the renumbered file in place [default: dry-run] |

### validate-anchors

Check a markdown record's own links, so a dead anchor is found by a command rather than by a
reader. Three checks, all within one file:

- every `](#slug)` link and `[N]: #slug` definition resolves to a heading in the same file
- every `[[N]]` citation has an `[N]:` definition, and every definition is cited, counting the
  `[text][N]` reference-link form a ladder rung uses for its own subsection
- no two headings slug to the same anchor, which markdown resolves silently, so every link to it
  reaches the first heading

Cross-file targets (`](other.md#slug)`) are counted but not resolved, since resolving one means
slugging another file. Read-only, and exits non-zero when anything is found.

```
# Every markdown record in the workspace
vc-x1 validate-anchors

# Specific files, which is also how the frozen history is checked,
# since notes/chores/ and notes/done.md are out of the default set
vc-x1 validate-anchors --file TODO.md --file notes/bugs.md

# -v adds per-file counts, -vv a line per site (every link with how
# it resolved, every heading with the slug it produced)
vc-x1 -vv validate-anchors TODO.md
```

The summary counts links rather than files, so a pass says how much it covered and the cross-file
tally names what it did not:

```
validate-anchors: 40 files, 220 link(s) checked, 8 failed, 220 cross-file skipped, 511 heading(s)
```

A failing target names the nearest heading in the file when one is close enough to be worth
naming, so the fix usually does not need the file opened.

The slug algorithm is the one [notes.md](./agent-data/notes.md#markdown-anchor-links) documents:
lowercase, drop every character that is not alphanumeric, a hyphen, or an underscore, and map each
remaining space to one hyphen. Adjacent spaces are not collapsed, so `a + b` becomes `a--b` while
`a: b` becomes `a-b`.

### validate

Run the workspace's configured validation commands: the work side's `[validate] full` table, or
`fast` with `--fast` (see [Workspace config tables](#workspace-config-tables)). Each element is
one command, split on whitespace with no shell in between, run in order from the work repo root
and printed before it runs. The first non-zero exit stops the run, naming the command, its
status, and where in the table it stopped, and `validate` exits non-zero. An empty or missing
table is an error, not a pass: nothing to run is not a validation. `-R DIR` names a workspace
other than the one containing the current directory.

```
# The per-commit validation, from anywhere in the workspace
vc-x1 validate

# The quick check between pushes
vc-x1 validate --fast

# Another workspace
vc-x1 validate -R ../other-project
```

Sample output, with a failing table (`["true", "false", "touch never"]`):

```
validate: [1/3] true
validate: [2/3] false
error: validate: `false` failed (exit 1), stopping at 2/3 of validate.full
```

| Option | Description |
|--------|-------------|
| `--fast` | Run the `validate.fast` table instead of `validate.full` |
| `-R, --repo <DIR>` | Workspace to validate [default: the one containing the current directory] |

### build

Build the workspace's release with `build`, or its dev build with `build-dev`, by the work side's
`[build]` table (see [Workspace config tables](#workspace-config-tables)). The lists run as
[validate](#validate)'s do: each element one command, split on whitespace with no shell in between,
run in order from the work repo root, the first failure stopping the run. Neither touches the PATH,
which is `vc-x1 install`'s.

- `vc-x1 build` runs `[build] release`, which builds the release, and then checks it left
  `[build] release-bin`.
- `vc-x1 build-dev` runs `[build] dev`, the dev build. A command per build, named with the `-dev`
  suffix as `run` and `run-dev` are.
- Neither tracks whether its binary is out of date: cargo does, rebuilding only what changed, so
  a build with nothing changed is quick and leaves the binary as it was.
- Both commands run their lists with two variables added to the environment,
  `VC_X1_BUILD_COMMIT`, the full commit id of `@-`, and `VC_X1_BUILD_STATE`, `clean` or `dirty`,
  so an artifact that compiles them in can say what it was built from wherever it is copied.
  `validate` passes the same two, so the test build it leaves has the same values and
  `build-dev` reuses it rather than recompiling. vc-x1 compiles them in, as semver build metadata
  in its banner, and as a `build` line in `vc-x1 version`:

  ```
  $ target/release/vc-x1 -V
  vc-x1 0.85.7+d70cf9ecdfd9 (agent-files v0.3.0)
  ```

  `.dirty` follows the commit when `@` was not empty. A build none of the three made, a
  hand-typed `cargo build` or `cargo install`, takes the commit from git's `HEAD`, which in a jj
  colocated repo is `@-`, and follows it with `.unknown`, since whether `@` held edits is a jj
  question its build script cannot ask. A build with no `.git` to read, a source package's,
  carries no metadata.
- Nothing is checked first, so a release may be built from a cycle's bookmark to try the release
  profile.

```
# Build the release, from the workspace containing the current directory
vc-x1 build

# Build the dev build without running it
vc-x1 build-dev

# Print the release build's list, and run nothing
vc-x1 build --dry-run
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Print the list, and run nothing |
| `-R, --repo <DIR>` | Workspace to build from, for both commands [default: the one containing the current directory] |

### install

Build the release if it is out of date, then copy it into the install directory, at a cycle's Land.
It first runs `[build] release` exactly as [build](#build) does. Cargo rebuilds only what changed,
so with nothing changed the binary already tested is the one copied, and a missing or stale one is
built rather than refused. It then copies `[build] release-bin`, `target/release/vc-x1` here, into
the one directory `[install] release-dir` names, `~/.cargo/bin` here, under the same file name.
`$PATH` is not searched: the directory being on it is what puts the copy on the PATH. `~` in the
directory expands against your home, so the committed value fits every machine. The copy is a
temporary file renamed into place, so a running copy of the old binary is undisturbed and a failed
copy leaves it as it was.

`[install] release`, a command list in `[build] release`'s shape, runs in place of the copy when
present, after the same build, report, and refusals, so a project that installs some other way, a
signed copy or one to another machine, says how. The elements run with no shell, each split on
whitespace, and then each word's placeholders are filled: `{build.release-bin}` and
`{install.release-dir}`, the values of the keys `install` reads, resolved for the run, `~`
expanded and `--dir` applied. A value with spaces stays one word, and any other `{name}` is an
error before anything runs. The list names every path it writes, the file name included, and a
script takes the values as arguments, `sh scripts/sign.sh {build.release-bin}
{install.release-dir}`. This workspace's list is the copy written out, so the config says what
`install` does:

```toml
[install]
release = [
  "cp {build.release-bin} {install.release-dir}/vc-x1.new",
  "mv {install.release-dir}/vc-x1.new {install.release-dir}/vc-x1",
]
release-dir = "~/.cargo/bin"
```

The rename is what makes it safe over a running binary, since a `cp` onto a running executable
fails. A failed `cp` leaves the `.new` file behind, which the next install overwrites, where the
built-in copy removes its own. The log shows each command with its placeholders filled.

It first reports what it is copying, from the binary's own `-V`:

- the version, noting a suffixed (pre-release) one and a manifest version that differs
- the commit, and whether it is `main` and `main@origin`
- the state, `clean`, `dirty`, or `unknown`

A binary whose `-V` carries no commit, another project's that does not compile the build
variables in, is reported as `commit unknown` and `state unknown`.

The report refuses nothing. Which builds may be installed is the release process's call, not the
tool's: here a cycle's Land runs `install` on `main` after the push, and a project releasing from
a branch or a tag makes it differently. `install` refuses only when the copy itself would be
wrong, listing every reason: no binary at `release-bin` after the build, a binary whose `-V` will
not run, or a missing install directory.

`--dry-run` builds and reports, and copies nothing, and runs no list. `--dir <DIR>` copies into
`DIR` instead of `[install] release-dir` for the one run, so a test of `install` leaves the PATH
alone, a relative `DIR` taken against the current directory. A workspace with no `[install]
release-dir`, and no `--dir`, has nothing to install, and `install` says so before building
anything.

```
# At Land, on main after the push: build and smoke-test, then install, its build a no-op
vc-x1 build
vc-x1 run -- -V
vc-x1 install

# Testing install itself: the copy goes to tmp/bin, and ~/.cargo/bin is untouched
mkdir -p tmp/bin
cargo run --locked -- install --dir tmp/bin
```

The report, run on a cycle's bookmark with edits in `@`, then the list:

```
install: '/home/me/proj/target/release/vc-x1' -> list `install.release`
install:   version 0.85.7-6, a suffixed (pre-release) version
install:   commit 82e97636b7f6, not `main`
install:   state dirty
install: [1/2] cp /home/me/proj/target/release/vc-x1 /home/me/.cargo/bin/vc-x1.new
install: [2/2] mv /home/me/.cargo/bin/vc-x1.new /home/me/.cargo/bin/vc-x1
install: 2 command(s) passed (install.release)
```

The build's own lines, `build: [1/1] cargo build --release --locked` and the rest, come first.
Without a list the first line names the destination, `-> '/home/me/.cargo/bin/vc-x1'`, and the
copy ends `install: copied`. A dry run stops after the report and says so, running and copying
nothing.

| Option | Description |
|--------|-------------|
| `--dry-run` | Build and report, and copy nothing |
| `--dir <DIR>` | Copy into `DIR` instead of `[install] release-dir`, for this run |
| `-R, --repo <DIR>` | Workspace to install from [default: the one containing the current directory] |

### install-dev

Build the dev build, then copy it into the install directory with `[install] dev-suffix`, `-dev`
when absent, appended to its file name. It first runs `[build] dev` exactly as [build-dev](#build)
does, then copies `[build] dev-bin`, `target/debug/vc-x1` here, into the directory `[install]
dev-dir` or `--dir` names, as `vc-x1-dev`. The landed `vc-x1` and the cycle's `vc-x1-dev` then sit
side by side on the PATH, so the dev build can be tried from any directory, another project's
included, where `vc-x1 run-dev` would run that project's dev build instead. The suffix is what lets
`dev-dir` be `release-dir`'s own directory, the dev build beside the release without replacing it,
and without a list `install-dev` refuses before the build when its destination would be the
release's, as an empty suffix in that directory or a `--dir` onto it would make it. A list writes
where it says, so it names the suffix itself, `{install.dev-suffix}`, as this workspace's does.

The report, the refusals, the copy, `--dry-run`, and `--dir` are [install](#install)'s, the binary
being the dev build's, and an `[install] dev` list runs in place of the copy as `[install] release`
does for `install`, its placeholders `{build.dev-bin}` and `{install.dev-dir}`. With a list the
`-dev` name is the list's to write, as this workspace's does: a list that left it off would write
the release's name and, with `dev-dir` the release's own directory, replace the release.

```
# Mid-cycle: put the rung's code on the PATH as vc-x1-dev, and try it in another project
vc-x1 install-dev
cd ../other-project
vc-x1-dev status
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Build and report, and copy nothing |
| `--dir <DIR>` | Copy into `DIR` instead of `[install] dev-dir`, for this run |
| `-R, --repo <DIR>` | Workspace to install from [default: the one containing the current directory] |

### run and run-dev

Build, then run the build's binary with the given arguments, as `cargo run` builds before it runs.
`vc-x1 run` runs `[build] release` exactly as [build](#build) does, then `[build] release-bin`,
and `vc-x1 run-dev` runs `[build] dev`, then `[build] dev-bin`, so what runs is always the current
code. Neither touches the PATH, so a build is tried before [install](#install) or
[install-dev](#install-dev) copies it.

- The arguments go to the binary untouched. One that starts with `-` comes after `--`, as with
  `cargo run`, since vc-x1's own flags are read first: `vc-x1 run -V` prints vc-x1's banner,
  and `vc-x1 run -- -V` runs the release's `-V`.
- The binary runs in the current directory, with the terminal, and its exit code is vc-x1's, so
  a failing run fails the same way.
- Before it runs, its `-V` is read, and a version other than the manifest's is a warning naming
  the key: the build passed but left an older binary where the config says. A binary whose `-V`
  prints no version, another project's, is run without the check.
- `--dry-run` prints the build's list and the command, and runs nothing.
- Completion is the binary's own: `vc-x1 run-dev <words><TAB>` asks the dev build, in clap's
  completion mode, to complete `<words>`, so in iiac-perf `vc-x1 run-dev run <TAB>` offers its
  benches. The binary is asked as it stands, never built, and one that is not there, or answers
  nothing, offers nothing. A binary built with clap's dynamic completion answers; another offers
  nothing.

```
# The smoke check before install: build the release and print its version
vc-x1 run -- -V

# Try the rung's code: build the dev build and run its status
vc-x1 run-dev status -s b

# See what would run
vc-x1 run-dev --dry-run status -s b
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Print the build's list and the command, and run nothing |
| `-R, --repo <DIR>` | Workspace to run from [default: the one containing the current directory] |
| `[ARGS]...` | Arguments for the binary, after `--` when the first starts with `-` |

### validate-agent

Check the agent repo is in its expected at-rest state: `main` matching `main@origin`, with its remote
refs tracked. At rest the two always match, because the bookmark only moves inside a `push` /
`squash-push` run, which publishes it in the same invocation, so a mismatch means an earlier publish
was lost. Read-only and cheap (two `jj` lookups, no build steps), and exits non-zero on any
finding and fixes nothing. Resolve with `vc-x1 squash-push -s a`.

```
# Check ./.claude (run from the project root)
vc-x1 validate-agent

# Explicit bot-repo path
vc-x1 validate-agent -R path/to/.claude
```

| Flag | Description |
|------|-------------|
| `-R, --repo <PATH>` | Path to the bot repo [default: .claude] |

`vc-x1 squash-push` reports the condition and proceeds, since publishing is its job. `vc-x1 push` no
longer checks it: a bot repo that hasn't been squash-pushed yet isn't a broken state, and the next
push publishes it anyway.

### config

Print the settable config keys for a target config file as an annotated, commented schema: for each
key its description, the command/context it's `used by`, its default (or a value marked `# example`
when there is no default), a `reference` url pointing at that key's documentation, and a commented
assignment line ready to paste in. The schema is generated at build time from the `vc-config.md`
prototype at the repo root (see
[vc-config.md (the schema prototype)](#vc-configmd-the-schema-prototype)), so it can't drift
from what the code actually reads.

- `[TARGET]`: `work`, `agent`, `work,agent` (default), or an explicit config-file path.
  - a keyword resolves against the surrounding workspace's config files and filters to that
    side's keys: `agent` prints the `[agent-session]` and `[repos]` keys the agent repo's config
    may hold, `work` adds `[family]` and `[validate]`, and `work,agent` prints both sides, one
    block each, under a header naming the file it resolved to
  - anything else is taken as a path to a config file. A path carries no side information, so
    in print mode it gets the whole schema for every home (the user-tier keys included) with the
    path as a label, and the file is not opened. The user config (`~/.config/vc-x1/config.toml`)
    has no keyword and is reached only this way
  - the old `bot` keywords are rejected with the `agent` spelling as the fix-it (0.80.0)

Checking a config file against the schema is [`validate-config`](#validate-config), a separate
subcommand since 0.80.6. `config` prints, `validate-config` checks, and the two take the same
`[TARGET]`.

A short sample of the printed schema (workspace home):

```
[agent-session]
# agent-session.col-width: Default --col-width: first-column width in the --fields / --unknown /
#   --per-line views
#   used by: agent-session --col-width
#   default: 68
#   reference: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessioncol-width
# col-width = 68

[repos]
# repos.work: The work repo's path, relative to this config file's directory ("." in the work repo,
#   ".." in the bot repo); the entry resolving to the config's own directory names the side
#   used by: find_workspace_root, side detection (structural; written by init)
#   default: (required; role-specific, see init)
#   reference: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#reposwork
work = "."   # example
```

A key with no default (`default.account`, in the user home) instead renders a commented example
value:

```
# default.account: Account profile (an [account.<name>] section) to use when --account is absent
#   used by: --account (init and account-aware commands)
#   default: (none)
#   reference: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#defaultaccount
# account = "work"   # example
```

```
# Print the workspace-settable keys, both sides
vc-x1 config

# Print only the work side's keys
vc-x1 config -s work

# Print the whole schema for one file (the user config has no
# keyword, reach it by path)
vc-x1 config --file ~/.config/vc-x1/config.toml

# Check a config file: see validate-config
vc-x1 validate-config
```

| Flag | Description |
|------|-------------|
| `-s, --scope <SCOPE>` | The side whose config: `work`, `agent`, or `work,agent` [default: work,agent] |
| `--file <PATH>` | A config file by path, the user config's one spelling, exclusive with `-s` |

The old `[TARGET]` positional is refused, naming `-s` for a side keyword and `--file` for a path.
`validate-config` takes the same two flags.

### validate-config

Check the workspace's config files. `config` prints the schema, and this checks a file against it,
which is why the two are separate subcommands. Four checks:

- every key is one `vc-config.md` declares for that side, so a typo, a key in the wrong section,
  or an unknown key is a finding, and a `str-list` key holding a plain string is one too
- the legacy `[workspace]` schema is rejected with a fix-it, and a dual workspace's two `[repos]`
  registries must resolve to the same pair
- the file's own `[[N]]` references and `#anchor` links resolve, and a `vc-config.md#<anchor>` link
  names a real key section. That last one is a schema lookup rather than a crawl of the other file,
  since build.rs derives those anchors from the key paths
- a side's config carries `repos.work`, the one key every instance config has. Not required of a
  path target, which carries no side information and may name the user config

```
# Both sides of the surrounding workspace
vc-x1 validate-config

# One side, or one explicit file
vc-x1 validate-config -s work
vc-x1 validate-config --file ../other/.vc-config.md

# -v adds per-file counts, -vv a line per site (every key, every
# link, every required key, with how each resolved)
vc-x1 -vv validate-config
```

The summary counts what was checked, so a pass says how much it covered rather than only that it
passed:

```
validate-config: 2 file(s), 9 key(s) and 18 link(s) checked, 2 required key(s), 0 problem(s) found
```

This is an opt-in strict check, since a normal config load ignores unknown keys for
forward-compatibility. A missing file is reported and skipped rather than failed, for a keyword
side the workspace lacks and, today, for a path as well, so a mistyped path checks clean (a Todo
tightens this). Was `config --validate` before 0.80.6, which now errors with this spelling as the
fix-it.

### vc-config.md (the schema prototype)

[vc-config.md](vc-config.md) at the repo root (no leading dot) is the single source of truth for
the settable-key schema above, and at the same time its long-form documentation. It is not a
config file: one `##` section per settable key holds that key's prose and, in a `toml` fence,
that key's metadata, and build.rs parses the file and generates the schema table plus the
built-in default constants the binary compiles against. One file drives every surface:

- the `config` subcommand's printed schema (above)
- the commented blocks `init` writes into a new workspace's instance config
- the built-in defaults themselves (e.g. `agent-session --col-width`'s 68), consumed from the
  generated constants, so behavior cannot disagree with documentation
- each key's `reference:` url, so a reader of any generated config can click through to that
  key's own section
- [vc-config-model.md](vc-config-model.md), a model config file carrying every table a workspace
  may hold with each key's default or a typical value. Generated rather than maintained, so
  "carries every key" holds by construction, and a test fails when the committed copy falls
  behind the schema

The file is read in the same format as an instance config: the `toml` fences, concatenated in
document order, form the TOML, and build.rs shares the filter (`src/md_fence.rs`) with the
loader. Prose between fences is documentation and never reaches a parser.

References are derived, not written: build.rs joins the single `[vc-config] reference-base` (the
bare repo url) with `/blob/HEAD/vc-config.md#` and each key's heading anchor, so links follow the
repo's default branch and no branch name is baked in, and the test suite fails if a key has no
matching heading. A per-key `reference` entry overrides the derivation for the odd key documented
elsewhere. A fork customizes both in one place: point `reference-base` at your repo and edit your
own copy of the file.

Editing the prototype is a source change like any other: edit, `cargo build`, and the change is live
(build.rs re-runs whenever the file changes). A malformed prototype, an unknown metadata entry, a
missing `doc`, a non-integer `usize` default, or a duplicate key path fails the build with a message
naming the offending table. A key's section looks like:

````markdown
## agent-session.col-width

First-column width in `agent-session`'s field-inventory views: `--fields`, `--unknown`, and
`--per-line`. The conversation view never consults it.

```toml
[agent-session.col-width]
homes = ["user", "workspace-code", "workspace-agent"]
kind = "usize"
doc = "Default --col-width: first-column width in the --fields / --unknown / --per-line views"
used-by = "agent-session --col-width"
default = 68
```
````

### Workspace config tables

A work-side `.vc-config.md` holds up to eight tables:

- `[repos]`: the two repos' local paths, written by `init`. The one table the agent repo's config
  carries too.
- `[remote]` (0.84.12): remote names rather than paths. Its one key, `agent-repo`, is the
  agent-repo's name on its remote, which `clone` reads before there is an agent-repo to ask.
  - Only the last URL segment is recorded. The owner and the host come from the work repo's own
    remote, so the two repos are siblings in one namespace.
  - An absent key means a name derived from `repos.agent`'s last component: a dot-led one is
    appended to the work repo's name (`.claude` gives `<name>.claude`), any other is used whole.
- `[family]` (0.80.0): the agent-file family this repo belongs to, which was prose in the project
  layer before.
- `[validate]` (0.80.0): the commands that validate this repo, also prose in the project layer
  before.
- `[build]` (0.85.7): the dev and release builds, and the binaries they leave, read by `build`,
  `build-dev`, `run`, `run-dev`, `install`, and `install-dev` ([Build and
  install](#build-and-install)).
- `[install]` (0.85.7): in `[build]`'s order, `release` and `dev`, optional lists run in place of
  the copy, and `release-dir` and `dev-dir`, where `install` copies the release build and
  `install-dev` the dev build, `~` expanded.
- `[agent-files.diff]` and `[agent-files.copy]` (0.83.0): the defaults for the `agent-files`
  commands' `DIR` operand and `--custom` choice.

All but `[repos]` are work side only: `validate-config` reports them unknown in the agent repo's
config.

````markdown
```toml
[repos]
work = "."
agent = ".claude"                 # the agent repo's directory, relative to this file
```

```toml
[remote]
agent-repo = "vc-x1.claude"       # the agent repo's name on its remote
```

```toml
[family]
member = "vc-x1"                  # this repo's name in the family, and its messages record
template = "../vc-x1-template/work"  # the payload directory holding the pinned agent-files
messages = "../vc-x1-messages"    # the family's notification repo
```

```toml
[validate]
full = [                          # the per-commit validation, in order, one command per element
  "cargo fmt",
  "cargo clippy --all-targets -- -D warnings",
  "cargo test",
]
fast = ["cargo test --bins"]      # the subset a ladder rung runs
```

```toml
[build]
release = ["cargo build --release --locked"]  # the release build
release-bin = "target/release/vc-x1"  # its binary, copied by install
dev = ["cargo build --locked"]    # the dev build
dev-bin = "target/debug/vc-x1"    # its binary, run by run-dev, copied by install-dev
```

```toml
[install]
release = [                       # the release install, the copy written out
  "cp {build.release-bin} {install.release-dir}/vc-x1.new",
  "mv {install.release-dir}/vc-x1.new {install.release-dir}/vc-x1",
]
release-dir = "~/.cargo/bin"      # where install copies the release build
dev = [                           # the dev install, as vc-x1-dev
  "cp {build.dev-bin} {install.dev-dir}/vc-x1-dev.new",
  "mv {install.dev-dir}/vc-x1-dev.new {install.dev-dir}/vc-x1-dev",
]
dev-dir = "~/.cargo/bin"          # where install-dev copies the dev build
```

```toml
[agent-files.diff]
dir = "../iiac-perf"              # default DIR for agent-files diff, else family.template
custom = true                     # compare custom.md too, as -c/--custom would

[agent-files.copy]
dir = "../vc-x1-template/work"    # default DIR for agent-files copy, else family.template
custom = false                    # never copy custom.md unless -c/--custom
```
````

The `custom` keys are `bool` keys, a bare `true` or `false`: `validate-config` reports a quoted
or other value as a finding by shape, and the commands refuse a config that holds one. A `dir`
is relative to the config file's directory, and absent, the command falls back to
`family.template`. The flags win over the table for one run: `-c`/`--custom` turns custom.md
on, `--no-custom` turns it off, and a `DIR` operand replaces `dir`.

The `[validate]` lists are `str-list` keys: a TOML array of strings, one element per invocation,
on one line or spread to the closing `]`. Each element is one command whose exit status is
checked, so a command with its own spaces and dashes is one quoted string.
[`vc-x1 validate`](#validate) (and `--fast`) runs them, `validate-config` checks the tables'
keys and shapes, and `config -s work` prints the schema with every key's example:

```
vc-x1 validate-config                              # both sides
vc-x1 validate-config -s work                      # the work side only
vc-x1 validate-config -s agent                     # the agent side only
vc-x1 validate-config --file .claude/.vc-config.md # a specific file, checked against every home
vc-x1 config -s work                    # the work side's schema, [family], [validate], [agent-files.*]
vc-x1 config -s agent                   # the agent side's schema
vc-x1 config --file .claude/.vc-config.md # the whole schema, labelled with the path
```

### clone

Clone an existing dual-repo project. Runs `git clone --recursive` to get both repos, initializes
`jj` in each, and creates the Claude Code symlink.

```
# Clone using a full URL
vc-x1 clone git@github.com:owner/my-project.git
vc-x1 clone https://github.com/owner/my-project

# A slashed source with no path prefix (`owner/my-project`) is
# refused: it reads equally as a path and as the retired owner/name
# shorthand.

# Clone with a custom directory name
vc-x1 clone https://github.com/owner/my-project my-local-name

# Clone into a specific parent directory
vc-x1 clone https://github.com/owner/my-project --dir ~/projects

# Preview without executing
vc-x1 clone https://github.com/owner/my-project --dry-run
```

| Flag | Description |
|------|-------------|
| `--dir <PATH>` | Parent directory [default: cwd] |
| `--dry-run` | Show what would be done without executing |
| `-v, --verbose` | Verbose output |

Requires `jj` to be installed. The `.claude` bot repo is cloned automatically via `git submodule` if
the source project was created with `vc-x1 init`.

### init

Create a new dual-repo project: a work repo with its agent repo nested inside it, at
`.agent-session` unless `--agent-dir` names another directory. Both repos are initialized with `git`
and `jj`, configured with `.vc-config.md`, and pushed to GitHub. The work repo ignores the agent
repo's directory, and its config records where the agent repo is and what its remote calls it, so
`vc-x1 clone` fetches both.

The agent repo's remote name is the work repo's name plus `.agent-session`, beside the work repo
under the same owner:

- `--agent-suffix` replaces the suffix, and must begin with `.` or `-`.
- `--agent-repo` gives the whole name instead, and the two conflict.
- The directory and the remote name are chosen separately, so `--agent-dir` changes neither name.
- A workspace created before 0.84.12 records no remote name, which reads as `.claude`, the suffix
  init used then.

```
# Create public project in current directory (GitHub via gh)
vc-x1 init my-project

# A URL or a path target. A slashed target with no path prefix
# (`myorg/my-project`) is refused: it reads equally as a path and
# as the retired owner/name shorthand.
vc-x1 init https://github.com/myorg/my-project
vc-x1 init ~/projects/my-project
vc-x1 init ./sub/dir/my-project

# Create private repos
vc-x1 init my-project --private

# Local bare remotes instead of GitHub (offline; used by fixtures)
vc-x1 init my-project --repo local=/path/to/parent

# Preview without executing
vc-x1 init my-project --dry-run

# Grow an existing plain directory into a dual workspace
vc-x1 init . --adopt

# Name the agent repo's directory and its remote
vc-x1 init my-project --agent-dir .sessions --agent-suffix -sessions
vc-x1 init my-project --agent-repo my-project.claude

# Seed both repos from template directories (sibling layout)
vc-x1 init my-project --use-template \
    ../vc-x1-work-repo-template,../vc-x1-bot-repo-template

# BOT omitted defaults to the `<CODE>.claude` sibling
vc-x1 init my-project --use-template ../tmpl
# Equivalent to:
vc-x1 init my-project --use-template ../tmpl,../tmpl.claude
```

| Flag | Description |
|------|-------------|
| `<TARGET>` | URL, path, or bare NAME |
| `[NAME]` | Repo directory name override (URL form only) |
| `--account <NAME>` | Pick `[account.<a>]` from user config |
| `--repo <CAT[=VAL]>` | Repo target, e.g. `local=<PARENT>` for local bare remotes |
| `--por` | Plain single repo (no agent repo) |
| `--adopt` | Grow an existing TARGET into a dual workspace (see below) |
| `--agent-dir <DIR>` | The agent repo's directory, one name [default: .agent-session] |
| `--agent-repo <NAME>` | The agent repo's whole remote name |
| `--agent-suffix <SUFFIX>` | The work repo's name plus this names the agent repo [default: .agent-session] |
| `--private` | Create private GitHub repos [default: public] |
| `--dry-run` | Show what would be done without executing |
| `--push-retries <N>` | Max push retries after repo creation [default: 5] |
| `--push-retry-delay <N>` | Seconds between push retries [default: 3] |
| `--use-template <CODE[,BOT]>` | Seed both repos from template dirs (see below) |
| `--config <none\|PATH>` | Override the canned config write, under the name the source's carrier calls for |

**`--use-template`**. Value is `CODE[,BOT]`. If `BOT` is omitted, defaults to the sibling directory
`<CODE>.claude` (file-name concat, not path join, since the two templates are not nested).
Non-hidden contents are copied recursively into each target, and hidden entries (names starting with
`.`) are skipped since init creates the repo's own hidden files (`.vc-config.md`, `.gitignore`,
`.git/`, `.jj/`). If either template has a `README.md` at its root, its first line is rewritten to
`# <repo-name>`: `<name>` for the work repo and `<name>.claude` for the bot repo. For local
verification without hitting GitHub, combine `--use-template` with `--repo local=<PARENT>`.

**`--adopt`**. Without it an existing TARGET is refused, named by what it holds. With it, init
grows the target into a dual workspace:

- A plain directory, no repo: both repos are created around it, and everything in it that its
  `.gitignore` does not exclude is the work repo's first commit, whatever the file sizes. Files
  over jj's new-file size limit (1MiB by default) are listed, since jj would otherwise leave them
  out. A `.gitignore` it already has is kept and given the lines it lacks, the agent directory's
  and `/.claude/settings.local.json`.
- A jj repo colocated with git and with no workspace config: its history stays, and one commit,
  "Adopt as a dual-repo workspace", goes on top, carrying `.vc-config.md`, the `.gitignore` lines,
  and the `ochid:` trailer to the agent repo's first commit. With an `origin`, the agent repo's
  remote is derived beside it, `--repo` and `--account` are refused, and the work side pushes
  nothing: the commit is yours to land. With no origin, the remotes come from `--repo` as for a
  fresh init, and both are pushed. A working copy with uncommitted work is refused, and a git-only
  repo is pointed at `jj git init --colocate`.
- A single-repo workspace, one whose config declares `repos.work` alone, such as `init --por`
  makes: adopted as the repo above, except that its config is edited in place. `agent =` goes into
  `[repos]` and `agent-repo =` under `[remote]`, and every other line, prose and comments
  included, stays as it was.
- A dual-repo workspace is refused, having nothing to grow.

`--adopt` is refused with `--por` and with `--use-template`, and on a TARGET that does not exist.

Requires `gh` (authenticated) and `jj` to be installed (`gh` is skipped under `--repo local=...`).

### symlink

Create or verify the Claude Code project symlink. Claude Code stores session data in
`~/.claude/projects/<encoded-path>/`. This command creates a symlink from that location to the
workspace's agent-repo directory. The encoded path is the project's absolute path with every
character that is not an ASCII letter or digit replaced by `-`, so `/home/me/io_uring.x1` is
`-home-me-io-uring-x1`.

```
# Create symlink for current project (default target: the directory repos.agent names)
vc-x1 symlink

# Specify a different target
vc-x1 symlink /path/to/session-dir

# Replace existing symlink without prompting
vc-x1 symlink -y

# List contents after creation
vc-x1 symlink -l
```

| Flag | Description |
|------|-------------|
| `--symlink-dir <PATH>` | Override symlink parent [default: ~/.claude/projects] |
| `-l, --list` | List contents of symlinked directory after creation |
| `-y, --yes` | Replace existing symlink without prompting |

### sync

Fetch and sync a set of repos to their remotes in one atomic operation: fetch, converge the
bookmark, reposition `@`. With no flags the repo set is `.`, the one repo `sync` is run in, whichever
side of a workspace that is. Name a side or both with `-s` / `--scope`, or point at a different
workspace root or single repo with `-R` / `--repo`. There are no modes: verify-then-act happens
inside a single invocation against one fetch snapshot (a separate check-then-apply pair of runs
would race the remote).

Sync is **safe by default**: before it fetches, it finds out what the fetch would do, and holds a
repo back only where that would tangle local work. A fetch is not a look. jj's `git fetch`:

- moves a tracked bookmark that is behind to the remote's
- leaves one that diverged conflicted
- when the remote rewrote a commit the clone has, as `squash-push` does from another clone,
  follows the rewrite and rebases `@` onto it, which can leave conflict markers in a file `@`
  had edited

So sync looks first, and fetches nothing to do it:

1. It asks the remote where each of its bookmarks is (`git ls-remote`) and compares with the
   last fetch.
2. For the bookmarks that moved, it downloads their commits into the git store with no ref
   moved, so jj sees nothing, and tells each move apart by ancestry: a **fast-forward** only
   added commits, a **rewrite** replaced some.
3. It checks the repo for **local work**: uncommitted changes in `@` (not asked of the agent
   repo, whose `@` holds the running session's writes), an `@-` that is not on the remote, or a
   bookmark with commits the remote does not have.

What it then does:

| The remote | The repo | Sync |
|------------|----------|------|
| has not moved | anything | nothing to fetch. Local work is noted and left untouched |
| only fast-forwarded | anything | fetches. A fast-forward replaces nothing, so local work is untouched |
| moved a bookmark | that bookmark has local commits of its own | **held back**: a fetch would leave it conflicted |
| rewrote or deleted a bookmark | holds local work | **held back**: a fetch would rebase the work onto the rewrite |
| rewrote a bookmark | holds no local work | fetches, and follows the rewrite |

Untouched is not brought up to date: an `@` with changes, or an unpushed bookmark, stays on the
old commit, and moving it onto the new one is the user's act.

A repo that is held back is **not fetched**. Sync prints why, syncs the other repos, and exits
non-zero naming it, so a `--quiet` caller learns it was not synced. Commit and push the work, or
pass `--force`, the go to sync the repo anyway. `--rebase` is the same flag under its earlier
name.

Per repo, a `sync` that fetches classifies the local bookmark, as it was before the fetch,
against its remote:

| State | Meaning | Action |
|------|---------|--------|
| up-to-date | local == remote | none |
| behind | local is ancestor of remote | `jj bookmark set <b> -r <b>@<remote>` |
| rewritten | the remote replaced the local commit (a forced push) | none, the fetch followed it |
| ahead | remote is ancestor of local | none (push is a separate step) |
| diverged | neither is ancestor | `jj rebase -b <local-head> -d <b>@<remote>`. `--force` only |
| no remote | bookmark has no `@<remote>` counterpart | none, skip |

The bookmark is read before the fetch because afterwards a repo that was behind reads as up to
date.

**`--dry-run`** does the same looking and stops there. It reports, per repo, where the synced
bookmark stands against the remote, the remote's other bookmarks that moved, the local work it
found, and a verdict, what a sync would do:

```
$ vc-x1 sync --dry-run
.: 'main' is behind origin by a fast-forward (local 1f2e8188b7a5, remote 0e62ea2401e5)
.: holds local work:
.:   bookmark topic has commits origin does not
.: a sync would follow origin and leave the local work untouched
```

| The verdict | When |
|-------------|------|
| a sync would have nothing to do | the remote has not moved |
| a sync would follow `<remote>` | the remote moved, and nothing holds the repo back |
| ... and leave the local work untouched | the same, where the repo holds local work |
| a sync would be held back | one of the two held-back rows above, with the reason listed |
| a sync would rebase `<b>` onto `<remote>` | diverged, with `--force` |

No bookmark, `@`, file, or remote-tracking ref (`<b>@<remote>`) changes, so it is safe in the
agent repo under a running session. What it does change: it makes the working-copy snapshot any
jj command makes, and it downloads the moved bookmarks' commits into the git store, where no
ref names them. They are the commits a fetch would bring. It exits 0 whenever the look itself
succeeded, whatever it found.

`--bookmark` names a **work-repo** bookmark only: the bot repo is a linear journal on `main` by
design, so its side of every step (tracking preflight, classify, act, reposition) always uses `main`
regardless of the flag.

After the bookmark action above, `sync` repositions `@` onto the freshly-synced bookmark as a final
pass, run after every repo syncs cleanly. The rule differs by repo (`@-` is the parent of `@`, `<b>`
is the synced `--bookmark`):

- **Work repo** (the workspace root):
  - `@` is clean (empty) and `<b>` sits ahead of `@-` on the same line -> `jj new <b>` starts a
    fresh `@` on the new tip (the old empty `@` is auto-abandoned).
  - `@` has changes -> left in place, on the old commit, and sync says so. With `--force` it is
    rebased: `jj rebase -b @ -d <b>`.
  - `@` already sits on `<b>`, or `<b>` isn't on `@-`'s line (diverged / `@` ahead) -> `@` is left
    untouched, with a note why.
- **Bot repo** (`.claude`): no-op when `@-` is already the `main` tip, so `@` keeps its change id
  and any live session writes stay in the working copy. When `main` moved, `jj new main` starts a
  fresh empty `@` on the new tip, and the prior `@` (e.g. `/exit`'s trailing session writes) is
  preserved as a sibling head. If `@-` isn't on `main`, `sync` errors rather than strand it.

On any failure during fetch/classify/act/reposition (conflicted rebase, subprocess error, anything)
`sync` **stops where the failing step stopped**. Nothing is auto-reverted, so the state can be
inspected as-is (jj's operation log holds everything, so nothing is lost by stopping). Before
acting, sync captures each repo's pre-sync `jj op` id in memory, and the failure report lists
every repo's op id. The undo is explicit, per repo: `jj op restore <op> -R <repo>`. Nothing
persists across
invocations: a snapshot that outlives its run is a stale target waiting to rewind unrelated work.
That is also why the former `revert` subcommand was removed at 0.78.3: it restored the persisted
snapshot with no guard against discarding operations recorded after the failed sync. A safer
revert would derive its target from the op log and refuse when unrelated operations sit in
between. Until such a design exists, `jj op log` + `jj op restore` is the recovery.

```
vc-x1 sync -s b                       # both repos of the dual workspace the cwd is in
vc-x1 sync -s b --dry-run             # look only: ask the remote, report, change nothing
vc-x1 sync -s b --force               # sync a repo sync would hold back, rebasing as needed
vc-x1 sync -s w                       # only the work repo
vc-x1 sync -s a                       # only the agent repo
vc-x1 sync -s p -R ../other           # ../other, a plain repo with no pair
vc-x1 sync -s b -R ../other           # the dual workspace ../other is in
```

**Repo set resolution.** `-s` says which repos and is required, `-R` says where and is optional,
the rule every write command follows, so a sync never guesses:

1. `-s w|a|b`: the sides of a dual workspace, resolved from the workspace `-R`, or the cwd, is in,
   by its root or either repo, through the root's `.vc-config.md` (`w` -> root, `a` -> the
   root-joined `agent` path, `b` -> both, work first). Outside a workspace it is an error naming
   `-s p`.
2. `-s p`: the repo `-R`, or the cwd, names, as itself. On either side of a dual workspace it is
   an error naming the side and its scope, since a plain operation on a side is half a workspace.
3. No `-s`: an error naming the four.

A named scope is cwd-portable: from the agent repo's directory, `vc-x1 sync -s b` walks up to the
workspace root and resolves both repos by absolute path.

| Flag | Description |
|------|-------------|
| `-s, --scope <SCOPE>` | Required: `w|work`, `a|agent`, `b|both` of a dual workspace, or `p|por`, a plain repo |
| `-R, --repo <PATH>` | Where: a path in the workspace, or the plain repo [default: the cwd] |
| `-q, --quiet` | Suppress all output, exit code signals result (for scripts) |
| `--bookmark <NAME>` | Bookmark to sync in the work repo (bot repo always syncs `main`) [default: main] |
| `--remote <NAME>` | Remote to sync against [default: origin] |
| `--force` | Sync a repo sync would hold back: rebase a diverged bookmark onto its remote, and a non-empty work repo `@` onto the synced bookmark. `--rebase` is the same flag |
| `--dry-run` | Look only: report each repo's state, its local work, and what a sync would do, moving no bookmark, `@`, or remote-tracking ref |

**Output shape.** Sync collapses output based on what it finds:

- **All up-to-date**: one-line summary, `sync: N repos are up to date, nothing to sync`. Nothing
  else (no-op reposition lines are debug-level). Makes "sprinkle sync everywhere" genuinely cheap.
  Scope is bookmark-vs-remote tracking, `@` may have uncommitted working-copy changes, and sync
  intentionally doesn't speak to that (use `jj st` for working-copy state).
- **Anything else** (`behind` / `rewritten` / `ahead` / `diverged`): per-repo fetch + state
  lines, then the actions run.
- **Held back**: the reasons are listed, and there is no fetch line.
- **Nothing to fetch, with local work**: one line saying the remote has not moved, and the local
  work listed as left untouched.
- **`--dry-run`**: the one-line summary when every repo is level and holds no local work,
  otherwise per repo its state and the local work found.
- **`--quiet`**: no output at any level, and the exit code is the only signal. Intended for
  scripts that just need success/failure.

**Note on the `behind` case.** jj's `git fetch` already fast-forwards a tracked local bookmark when
it's a strict ancestor of the incoming remote. Sync reports `behind` all the same, since it
classifies from where the bookmark was before the fetch, and its own `bookmark set` then covers
untracked bookmarks and edge configs where auto-advance is disabled.

### squash-push

Squash `@` into `@-`, advance a bookmark, and push, capturing a repo's trailing working-copy writes
and publish them in one step. Rewriting an already-pushed commit this way is a deliberate
published-history rewrite, so the push is a forced update.

The primary use case is folding the bot repo's session tail: session data keeps landing in `@` after
the last commit (including the record of the push itself), and only the user, acting after the bot
goes quiet, can capture all of it. Zero ceremony by design:

`--bookmark` defaults to the bookmark of the line you are on, the nearest one at or above the squash
target. There is no literal default: on the agent repo the line's bookmark is `main`, and on a work
repo running a cycle it is the topic bookmark, so a bare run publishes the line you are working and
never advances `main`. Several candidates, or none, is an error asking you to name one, since a
wrong guess publishes something. A literal `main` default used to sit here, and on 2026-09-17 a bare
run on the work repo mid-cycle advanced `main` onto the cycle tip and pushed it, which is Land's
fast-forward step performed by accident.

Every run begins by asking whether it has work, and ends by saying what it left. A run with nothing
to do prints `<label>: clean` and exits 0 having touched nothing, so the command is safe to call
when you are unsure, and every other run prints that line again after the push. With work found it
acts without asking, which is what `squash-push.yes` defaults to, and `--ask` turns a confirmation
prompt on. The details of each are in the behavior notes below.

```
# The agent repo: squash @ -> @-, advance main, push, from anywhere in the workspace
vc-x1 squash-push -s a

# Both repos, work first: the fold-and-publish that asks nothing
vc-x1 squash-push -s b

# A plain repo with no pair, the current directory's
vc-x1 squash-push -s p

# Name the bookmark explicitly, with a custom squash pair, on the work repo
vc-x1 squash-push -s w --bookmark feature --squash @,@--

# Confirm before acting, overriding a configured squash-push.yes
vc-x1 squash-push -s a --ask

# Say what it found and what it would do, and change nothing
vc-x1 squash-push -s a --dry-run
```

`-s` is required and `-R` optional, as for every write command: `-s` says which repos, `-R` says
where, a path anywhere in the workspace for `w|a|b` or the plain repo itself for `p`, and the cwd
without it. See [Repo set resolution](#sync) under `sync`.

```
.claude: dirty: @ has changes. squash-push? [y/N] y
squash-push: squashing @ -> @-...
squash-push: setting bookmark 'main' to @-...
squash-push: pushing 'main' to origin...
squash-push: done
.claude: clean
```

```
# Nothing to do: no squash, no push, nothing touched
vc-x1 squash-push -s a
.claude: clean
```

Every at-rest run prints the verdict line before it acts as well as after, so a run that is about
to do something says what it found first:

```
vc-x1: dirty: bookmark behind the squash target
squash-push: @ is empty, skipping squash, still pushing
squash-push: setting bookmark 'topic' to @-...
squash-push: pushing 'topic' to origin...
squash-push: done
vc-x1: clean
```

| Flag | Description |
|------|-------------|
| `--bookmark <NAME>` | Bookmark to advance and push [default: the line's own, see above]; the old positional is refused, naming this flag |
| `-R, --repo <PATH>` | Path to jj repo [default: .] |
| `--squash [<SOURCE,TARGET>]` | Squash pair [default: @,@-] |
| `-y, --yes` | Act without asking, once the precheck found work |
| `--ask` | Ask before acting, overriding a configured `squash-push.yes` |

Behavior notes:

- Runs synchronously, so a failure is a visible non-zero exit. (Replaces the `finalize`
  subcommand, whose detached background child could be killed silently at command exit.)
- A precheck asks whether the run has work, reading the verdict `status` computes (the working copy
  at rest, the bookmark at its origin) plus this command's own question of whether the bookmark has
  reached the squash target. A bookmark behind its target has a commit to publish even when it
  matches origin, which is why the third condition is there. Nothing to do prints `<label>: clean`
  and exits 0 having touched nothing, `<label>` being the repo's directory name since the command
  takes a path rather than a scope.
- Every other run prints the same line after the push, so the state it left is reported rather than
  looked up. That line is a report only: the agent repo's transcript grows while the push runs, so a
  correct push often reads `dirty: @ has changes` a moment later, and the exit code says whether the
  push completed, not what the after-check found.
- With work found, it asks before acting when asking is on. `squash-push.yes` sets the default and
  is `true`, so a bare run still acts without asking as it always has. `--ask` turns the prompt on
  for one run and `-y`/`--yes` turns it off, which is why `--ask` exists rather than a `--no-yes`: a
  boolean flag cannot turn a configured `true` back off. The key is read from the repo `-R` points
  at, so the two sides of a workspace may answer differently. With the prompt on, a non-tty run is
  an error naming `--yes` rather than a hang, the rule [`push --step`](#push) follows, and declining
  is an error too, so a scripted run learns it did not happen. A run the precheck found clean never
  reaches the prompt.
- With an empty `@` but work still to do, it skips the squash and still pushes.
- If the bookmark doesn't match `<bookmark>@origin` at start (an earlier publish was lost, see
  [validate-agent](#validate-agent)), it says so and proceeds: publishing is its job.
- As [`push`](#push)'s `squash-push-bot` stage it says nothing and asks nothing. Every line above is
  addressed to a person, and mid-push there is neither a person to read them nor a settled state
  worth describing: the mismatch is normal there, since push's `bookmark-set` just moved the
  bookmark and this stage is what publishes it. The precheck's decision still applies, so a stage
  with nothing to do still does nothing.
- Preflight refuses bad states before rewriting anything: unresolvable squash revsets, an
  ochid-dropping squash (see [Testing the ochid-trailer guard](#testing-the-ochid-trailer-guard)),
  conflicts, a missing / untracked / non-forward bookmark, an undescribed push target.

See [inline session push + squash-push
(0.69.0)](./notes/chores/chores-13.md#feat-inline-session-push--squash-push-0690) for design
details.

### commit

Commit the working copies as a rung's pair, one commit per repo of a dual workspace, each stamped
with the other's change id, and push it in the same invocation. The diff and the description are
reviewed before the commit, so what is pushed is what was reviewed, and `--no-push` keeps a commit
local for a look at it by `vc-x1 show -s both @-` before `push`. Both change ids are
known before either commit is made, since a jj commit keeps the working copy's change id, so each
trailer is written once.

```
# The work half now, the body from a file, the agent half at push
vc-x1 commit -s w --title "feat: a rung" --body @tmp/body.md

# Both commits now, from anywhere in the workspace
vc-x1 commit -s b --title "feat: a rung" --body "The problem.

* A facet.
  - Its solution."

# A plain repo, the description with no trailer
vc-x1 commit -s p -R ../other --title "..." --body "..."

# A draft made by hand, the undescribed @-, described in place as the rung, its trailer stamped
vc-x1 commit -s b -r @- --title "feat: a rung" --body @tmp/body.md

# The cycle's opening, " opening" appended, its bookmark, feat-a-cycle, created at @- and pushed
vc-x1 commit -s b --opening --title "feat: a cycle" --body @tmp/body.md

# The cycle's closing, its tip @-, onto main: one commit above main is finalized in place, more
# get the closing commit, its title and body from the opening and the config
vc-x1 commit -s b --closing @-

# The same, the tip named by the cycle's bookmark, onto a configured or named trunk
vc-x1 commit -s b --closing feat-a-cycle --land dev
```

`-r, --revision REV` is the commit the description is for: `@` by default, committed as the
working copy, else `@-` or any draft below it, described in place with no new commit and its
trailer stamped, so a commit made by hand becomes a rung's. The draft must be unpublished, no
remote bookmark reaching it, carry no trailer yet, since a pair's commit is re-described by
`squash --title --body`, and hold a change. `@` is left as it is, as `push` leaves new work, the
line's bookmark advances to the draft, and a draft below `@-` is kept local, since push publishes
`@-`. Not with `--opening` or `--closing`, whose commits are made from `@`. The positional it was
is refused, naming `-r`.

`--closing <BM|REV>` closes the cycle whose tip the argument names, a bookmark or a revision as jj
resolves them, a name that is both being the bookmark, onto the trunk, `--land`'s bookmark or
`[cycle] trunk`, `main` when absent, read where it is now, so a trunk that moved during the cycle
is joined by the merge rather than needing a rebase. The commits between decide the shape:

- One, the opening: a single-step cycle, finalized in place with no new commit. `@` is folded into
  it when it sits on it, and it takes the bare cycle title, its body and trailer kept, so its
  change id and its partner stand, and the agent `@` is left for `push`. The line's bookmark
  follows the rewrite. A commit that is not an opening is refused, unless `--title` is given.
- More: the closing commit, the trunk and the tip merged into `@`, as `merge --target @ --from
  <trunk> --from <tip>` does, then committed, which keeps those parents, carrying the record move
  and what `@` holds. `--dead-end` always makes it, a one-commit branch included.
- None: refused, the tip being in the trunk's ancestry.

A tip that is not `@-`, any commit on any line, is closed by a detour: `@` moves onto it first, so
the closing is made from the tip's working copy, and returns after, to the same commit when it
held work and to a fresh empty one on its parent when it did not. The commits above such a tip
stay as they are, their line unclosed, and two heads close one at a time, each landed before the
next, since the second's first parent is the trunk as it then stands.

`--title` and `--body` are optional with `--closing`. The cycle title is read from the opening, its
" opening" removed, and " closing" is appended for a closing commit, " dead-end closing" with
`--dead-end`. A given `--title` is the bare cycle title. The closing commit's
body defaults to `[closing] body` in `.vc-config.md`, its `{title}` the cycle title, and a
finalized commit keeps the opening's. Without `--closing` a title ending " closing" is warned
about, since it may be a closing not asked for.

With `[record] doing` and `closed` set, a closing moves the cycle's record before it commits, so the
commit carries it: the doing file's `## In Progress` block and its `# References` replace the
closed file's `## Closed` block and references, and the doing file's `## In Progress` is reset to
"_No cycle currently in progress._". A finalized commit's kept body gains "See details in
<closed>.". The move is planned and checked before anything is made, so nothing in progress, or a
missing heading, is refused with both files as they were.

`--land [TRUNK]` lands the cycle in the same invocation: the closing is made, the trunk is
fast-forwarded to it and pushed with the agent half as its partner, and the closing's bookmark is
deleted locally and on `origin`, `--skip local|remote|both` keeping what it names. The trunk is
`--land`'s value, else `[cycle] trunk`, else `main`, the closing's first parent either way, so a
cycle lands with `commit --closing @- --land`. The closing's own push to the cycle's bookmark is
skipped, the bookmark being deleted. `--land` is the closing then [`land`](#land), one code path,
in one step: to install and test the closing before it is published, close without `--land` and
run `land` after.

The closing's bookmark is the one its commit carries and a Land deletes, and `--closing`'s
argument decides it. A bookmark named is that one. A revision means the current bookmark on the
tip, else the nearest parent's above the trunk. Several at that height are refused with the list,
since naming one of them settles it, and none means the closing carries none, kept local unless
`--land` lands it, since a push would publish it on the trunk. The branch's other bookmarks are
merged by the Land and left as they are, each named after it, yours to keep or delete. Every check runs first, so a
refusal leaves everything as it was: the trunk must be a local bookmark at its `origin` position,
since a trunk that moved stops a Land, the closing must merge onto it, and the branch must descend
from it, so the tree landed is the one reviewed. A conflicted closing is made but not landed. The
install stays `vc-x1 install`, after.

```bash
vc-x1 commit -s b --closing @- --land        # the cycle at @-, finalized or merged, landed on main
vc-x1 commit -s b --closing @- --land dev    # the same, landed on dev, which is also the merge's base
```

`--opening [BOOKMARK]` makes the cycle's opening: `--title` is the bare cycle title and " opening"
is appended, a title carrying it already refused. The bookmark is the one named, else the bare
title's slug, the anchor a heading of the title gets, `feat: a cycle` naming `feat-a-cycle`. The bookmark is created at the work `@-`, where
the cycle's branch starts, and pushed to `origin`, a name for a commit already public, so nothing
unreviewed goes out, and the opening itself stays local until the push after its review. `--skip
local` takes a bookmark already at `@-` and pushes it, `--skip remote` creates it and pushes
nothing, and `--skip both` only checks it. Every check runs before the commit: a bookmark to create
must not exist, a push must not move one published elsewhere, and `@-` must already be on a remote.

The line's bookmark advances to the work commit, locally: the nearest bookmark below the work
`@-`, as `push` resolves it, the one `--bookmark` names, or `--opening`'s. jj moves a bookmark with
its commit through a later squash or amend, so the advance holds through the review's fixes, and
the remote moves only at `push`. `--no-advance` leaves the bookmark where it is, and a line with no
bookmark or several is refused before anything is made. The agent repo's `main` is push's to set.

`--dry-run` prints the descriptions it would write, trailers included, and commits nothing. `-s`
is required: `b` makes both commits now, `w` makes the work commit now, its trailer naming
the agent `@` all the same, and leaves the agent half to `push`, which commits the journal as the
partner, `a` commits the agent repo alone, unpaired, with no trailer, a fix-up that belongs to no
rung, and `p` commits a plain repo. An agent `@` that is a work commit's pending partner is
committed with a warning, its change id kept, so the work commit's trailer still resolves.

The commit is pushed in the same invocation, as `vc-x1 push` would after it: a pair by `push -s
b`, a plain repo by `push -s p`, and an agent commit by publishing the agent repo's `main`. The diff
and the description are reviewed before the commit, so what is pushed is what was reviewed.
`--no-push` keeps one commit local, for a look at it before `vc-x1 push`, `[commit] push = false`
makes that the default, and `--push` overrides the key. A commit with `--no-advance` pushes only
with `--push`, since its bookmark was left where it was.
`-R` is optional and says where. `--body` is the text, or `@PATH` to read it from a file, since a
body in commit-body form has markers and blank lines a shell quote mangles. The work repo's `@`
must hold a change, and with `b` the agent repo's `@`, the session's journal, is committed as it
stands. A failure after the first commit restores both repos from `jj op` snapshots.

### land

Land a cycle whose last commit is already made: `--onto` fast-forwarded to `--source` and pushed,
and the cycle's bookmark deleted locally and on `origin`, every check made before anything moves.
It is the Land of a closing made without `--land`, or of a single-step commit, and the second half
of `commit --closing ... --land`, which runs the same code.

```bash
vc-x1 land -s b --onto main                          # the one bookmark besides main, landed on it
vc-x1 land -s b --source feat-a-cycle --onto main    # the cycle's bookmark named
vc-x1 land -s b --source @- --onto dev               # a revision, its bookmark the nearest one
vc-x1 land -s b --source feat-c --onto main --delete-intermediate-bookmarks   # a stack of cycles
vc-x1 land -s p -R ../other --onto main --dry-run    # a plain repo, every check, nothing moved
```

Testing before publishing is the reason to use it over `commit --land`. A Land only fast-forwards,
so `main` becomes the source commit itself, its id and its tree, and the release build of the
committed source, installed off the PATH by `install --dir`, is the binary that ships:

```bash
vc-x1 commit -s b --closing @-             # the closing, pushed to the cycle's bookmark, main unmoved
mkdir -p tmp/candidate                     # install --dir takes a directory that exists
vc-x1 install --dir tmp/candidate          # its release build, the plain name, off the PATH
tmp/candidate/vc-x1 -V                     # the closing's commit, clean
# ... the final testing, with tmp/candidate/vc-x1 ...
tmp/candidate/vc-x1 land -s b --onto main --dry-run   # the steps it would take
tmp/candidate/vc-x1 land -s b --onto main             # the tree tested, landed and published
tmp/candidate/vc-x1 install                # now on main, it installs itself under the plain name
cmp tmp/candidate/vc-x1 ~/.cargo/bin/vc-x1 # the binary tested is the one installed
```

The plain `vc-x1` on the PATH is left to what is on `main` until the Land, so whatever relies on it
never runs a candidate, and the candidate makes its own Land. A binary embeds its commit and
whether the tree was clean, no time, so the build after the fast-forward is the build tested. A
`main` that moved meanwhile is refused, and the rebase it needs makes a new tree, built and tested
again. A longer test is the same with a directory that lasts, put first on the PATH of the shell
it is tried from. `commit --closing ... --land` is the one step when no test pause is wanted. A Land
is always published: `--dry-run` makes every check and says each step, and is the look before it.

`--source <BM|REV>` names the cycle's last commit, a bookmark, the cycle's, or a revision, whose
current bookmark, else its nearest parent's above `--onto`, is the cycle's, several at that height
refused with the list, as `commit --closing` decides it. Left out, it is the one local bookmark
besides `--onto`, every local bookmark counted, and none or several is refused with their names,
so a stack of cycles names its tip. `--onto <BOOKMARK>` is any local bookmark at its `origin`
position that `--source` descends from, so a bushy tree lands anywhere, and one that is not an
ancestor is refused: `land` makes no merge, which is `commit --closing`'s. A conflicted source is
not landed.

The publish follows where the source is. At the work `@-`, whose pair may be unfinished, the trunk
goes through `push`, which finishes it. Below it, the pair is finished, so the trunk is pushed
alone and the agent `main` published. `--skip local|remote|both` keeps that side of the cycle's
bookmark. The branch's other bookmarks are merged by the Land and left as they are, each named
after it, and `--delete-intermediate-bookmarks` deletes those strictly between `--onto` and
`--source`, locally and on origin as `--skip` allows, a stack of cycles landing together. A
bookmark at the tip beside the cycle's is kept either way, since it may be one you put there.

`-s` is required: `b` a dual workspace, the agent `main` published with the trunk, and `p` a
plain repo. `w` alone could publish trailers naming agent commits not yet published, and `a`
alone has no cycle to land, so both are refused ([Write scopes](#write-scopes)). The install
stays `vc-x1 install`.

### squash

Continue a rung: fold the working copies into the pair `commit` made. A review's fix in the work
`@` folds into the work commit, and the journal into the agent commit, each keeping its change id,
so both trailers stay right without being touched.

```
vc-x1 squash -s b                                    # both sides
vc-x1 squash -s w                                    # the work side only, a draft too
vc-x1 squash -s w --yes                              # a commit already on main's remote
vc-x1 squash -s b --title "feat: ..." --body @tmp/body.md   # and rewrite the descriptions
vc-x1 squash -s p -R ../other                        # a plain repo's @ into its @-
```

`-s` is required and `-R` optional. The agent side goes by where the partner stands: committed by
`commit -s b`, so the journal folds into it, or pending after `commit -s w`, so the journal already
is the partner and push commits it. A published side goes by where it was published:

- a draft, published on a topic bookmark only, is rewritten with a warning, since the cycle is not
  landed and the next push only moves its bookmark
- a commit on `main`'s remote is history others have, refused without `--yes`
- a published partner with only the journal to fold is left for push, whose agent half folds it

A work `@-` with no trailer is not a pair's. `--title` with `--body` rewrites the descriptions and
keeps each trailer, the one sanctioned re-describe of a trailer-carrying commit, warned or refused
as a fold is. `--dry-run` prints what it would fold and re-describe, and changes nothing.

### merge

Add the sources named to one target commit's parents, after the ones it has, keeping the target's
own change, description, and change id, so its trailer stays right. The target's tree becomes the
merge of all its parents with its own change applied, `jj rebase -r <target> -o <p1> ... -o <s1>
...`. One target, empty or not, and N sources, with no assumption about where either lives: each
`--from` is a revset of one or more commits, a bookmark, `main`, a change id, an earlier merge, a
root, added oldest first. Merging sources one at a time and all at once give the same tree.

```
vc-x1 merge -s w --target @ --from A --from B          # two lines of work into @
vc-x1 merge -s w --target @ --from C                   # one more, after A and B
vc-x1 merge -s w --target X --from 'bookmarks(glob:"f-*")'  # every bookmark matching, at once
vc-x1 merge -s w --target @ --from main --dry-run      # say what it would do
```

`-s`, `--target`, and `--from` are required, `-s` being `w` or `p`, since the agent repo never
holds a merge. A source that is already a parent is skipped, and a merge adding nothing changes
nothing. Removing a parent is not a merge: `jj undo` takes back the last one, and `jj rebase -r
<target> -o ...` sets any parents. Refused: a source that is the target or below it, a cycle, and
a target in jj's `immutable()`, under jj's aliases and yours, unless `--ignore-immutable`, as `jj`
refuses it. A rung pushed to its cycle bookmark is mutable to jj, and the push after its rewrite
takes approval like any other. Reported and allowed: a conflicted result, left as jj leaves it
with `jj resolve --list` named. A source that is an ancestor of another, `main` below the last
rung, is the no-fast-forward shape a closing is, so it is neither refused nor reported.

### move-bookmark

Point one existing local bookmark at one commit, `jj bookmark set <bookmark> -r <target>`, the
repositioning a bookmark needs before it is pushed, since `push` only sets the line's bookmark to
`@-`. Nothing is pushed.

```
vc-x1 move-bookmark -s w --bookmark feature --target @-       # forward to the last commit
vc-x1 move-bookmark -s w --bookmark main --target feature     # main to where feature is
vc-x1 move-bookmark -s w --bookmark feature --target X --allow-backwards  # off its line
vc-x1 move-bookmark -s w --bookmark feature --target @- --dry-run         # say what it would do
vc-x1 move-bookmark -s b --bookmark main --target @- --yes                # main in both repos
vc-x1 move-bookmark -s p -R ../other --bookmark main --target @-          # any repo, unchecked
```

`-s`, `--bookmark`, and `--target` are required. `-s w` moves the work repo's bookmark. `-s a`, the
agent repo, and `-s b`, both, take `--yes`, since the agent repo's `main` is otherwise set by push's
agent half: the same name in each, the target resolved in each, and every repo checked before any
moves. `-s p` takes the repo from `-R`, required, as it is, with none of the workspace's checks, so
it moves a bookmark in either side of a dual workspace or any jj repo, and careful use is yours.
The bookmark must already exist, none being created, and the target is a revset of exactly one
commit, a bookmark or a rev. A move backward or sideways, leaving
commits off the bookmark's line, is refused unless `--allow-backwards`, as `jj` refuses it, and a
bookmark already at the target is left as it is.

### delete-bookmark

Delete one bookmark locally and on `origin`, the disposal a cycle's bookmark gets at Land, `jj
bookmark delete` and its push in one invocation. The remote goes first, so a failed push leaves the
local bookmark, and a side already gone is no error, so a rerun finishes the job. The commits stay,
and the commit the bookmark was at is printed so it can be recreated.

```bash
vc-x1 delete-bookmark -s w --bookmark feat-a-cycle                # local and on origin
vc-x1 delete-bookmark -s w --bookmark feat-a-cycle --skip local   # origin only
vc-x1 delete-bookmark -s w --bookmark feat-a-cycle --skip remote  # local only
vc-x1 delete-bookmark -s p -R ../other --bookmark old --dry-run   # print, delete nothing
```

`-s` is required: `w` the work repo, `a` the agent repo, `b` both, and `p` the repo `-R` names, as
it is.

### push

Publish a rung's pair, or a name for a commit already made. Push carries no description and writes
none, so what was reviewed is what goes out: `commit` makes the pair and pushes it unless told not
to, and push publishes one it kept local, completing the agent half as its partner.

Every argument is a flag. `-r` is the commit to publish, the work `@-` when left out, and
`--bookmark` is its name on the remote, so the commit and its name are never one word read two
ways, as jj keeps them apart in `jj bookmark set NAME -r REV`. At `@-` the pair goes out on the
line's bookmark, or the one named, a new name created at `@-` and pushed tracked. Any other `-r`
is a commit already made, and a name alone goes out for it, no pair: the one bookmark at it, or
`--bookmark`'s, that bookmark or a new one created there.

```bash
vc-x1 push -s b                        # the pair, on the bookmark the work @- sits on
vc-x1 push -s b --bookmark feature     # the same, the bookmark named as a check
vc-x1 push -s b --bookmark topic       # the pair on a new name, created at @- and pushed tracked
vc-x1 push -s b -r side                # a bookmark off the line, on a commit made, published where it is
vc-x1 push -s b -r xyz --bookmark name # a new name for a commit made, created there and pushed
vc-x1 push -s w                        # the work half only
vc-x1 push -s a                        # the agent half only, the remainder of a run that stopped
vc-x1 push -s p -R ../other            # a plain repo's @- on its bookmark, no trailer asked for
vc-x1 push -s b --dry-run              # plan and print, move nothing
```

Push publishes the work `@-`, so new work in `@`, the next rung started, is left as it is, and
push says so. What push refuses, with both repos as they were:

- a work `@-` with no `ochid:` trailer: not a pair's commit, so `commit` makes one first
- two bookmarks on the line and none named, or a named one not on the line
- a `-r` of several commits, the working copy, which `commit` turns into a commit, or the root,
  which has nothing to publish, and a `-r` with no bookmark at it and none named, or with a
  `--bookmark` that is elsewhere, which `move-bookmark` moves
- `-r` with `-s a`, the agent half publishing the agent `main`
- the old positional, naming `--bookmark` or `-r`, the one it was

The two halves are independent, since each trailer is fixed before push runs, so `push -s b` is
`push -s a` then `push -s w`, the order the pair is completed in:

| Half | What it does |
|------|--------------|
| agent | Complete the partner by the state found: the agent `@-` names the work commit (`commit -s b`), so the journal folds into it; or the agent `@` is the change the work commit names (`commit -s w`), or the agent repo changed since, so the journal is committed as the partner with the work commit's own text. Then squash-push `main` (see [squash-push](#squash-push)) |
| work | Check the work trailer names the partner, restamping the unpublished work commit when it does not, said out loud. Then `jj bookmark set <bookmark> -r @-`, verify its remote refs are tracked, and `jj git push --bookmark <bookmark>` |

**Rerunning is always safe.** There is no saved state and no resume: every stage checks its own
precondition and does nothing when its work is already done, so re-running after a failure is simply
running. If a run fails, push stops and reports. A half that fails before its own push restores the
repo it touched via `jj op restore` to a snapshot taken moments earlier in the same process. Once a
half's push succeeds that half is published: from there a change is either a new pair on top, or an
amend of what was pushed, where [`squash-push`](#squash-push) folds the working copy into the last
commit and force-updates the remote. See the protocol's
[at-rest contract](AGENTS.md#at-rest-push-stop-squash-push).

| Flag | Description |
|------|-------------|
| `-s, --scope <SCOPE>` | Required: `b|both` the pair, `w|work` the work half, `a|agent` the agent half, `p|por` a plain repo |
| `-R, --repo <PATH>` | Where: a path in the workspace, or the plain repo [default: the cwd] |
| `-r, --revision <REV>` | The commit to publish, one: `@-` the pair, any other a name alone [default: `@-`] |
| `--bookmark <NAME>` | Its name on the remote: a bookmark on the line, one at REV, or a new name created there [default: the line's, or the one at REV] |
| `--dry-run` | Plan and print what would run, no side effects |
| `--step` | Pause after every stage for an extra continue-prompt |
| `-y, --yes` | Pass the `--step` gates without asking |

See [Add push subcommand (0.37.0)](./notes/chores/chores-05.md#add-push-subcommand-0370) for the
full design and [per-step record](./notes/chores/chores-05.md#per-step-record) for what each
`0.37.0-N` dev step shipped.

### Testing push + squash-push

Always test against a throwaway fixture, never the live workspace. Scaffold one with `vc-x1 init
<TARGET> --repo local=<PARENT>` (no GitHub, no network), then run the complete flow end-to-end. The
work repo uses plain `jj git push`, and the bot repo uses `vc-x1 squash-push` to squash trailing
writes and push in one shot.

`--repo local=<PARENT>` lays out:
```
<PARENT>/
  remote-work.git/     bare git remote for the work repo
  remote-bot.git/   bare git remote for the .claude bot repo
  <NAME>/              work repo (jj colocated, main tracks origin)
    .vc-config.md      work=".", agent=".claude"
    .gitignore         /target /.claude /.git /.jj /.claude/settings.local.json
    .claude/           bot repo (jj colocated, main tracks origin)
      .vc-config.md    work="..", agent="."
      .gitignore       .git .jj /settings.local.json
```

```bash
parent=$(mktemp -u /tmp/vc-x1-test-XXXXXX)
vc-x1 init "$parent/work" --repo local="$parent"
work="$parent/work"
bot="$parent/work/.claude"

# 1. work repo: described commit -> advance main -> push
echo hello > "$work/hello.txt"
jj describe @ -R "$work" -m 'feat: add hello.txt'
jj bookmark set main -r @ -R "$work"
jj git push -R "$work"

# 2. bot repo: trailing writes -> squash-push (fold into @-, push)
echo notes > "$bot/notes.md"
vc-x1 squash-push -s a -R "$bot"

# 3. cleanup when done (init also created a symlink under
#    ~/.claude/projects/ pointing at the fixture, remove it too)
rm -rf "$parent"
```

**Why `jj git push` for the work repo but `squash-push` for `.claude`?** The work repo's workflow is
a plain dev commit on `@-` that we push directly. The bot repo mirrors the bot's runtime pattern:
session writes land in `@` (above the last committed commit), and squash-push folds those trailing
writes into that commit just before pushing, so one atomic state goes upstream.

The run is synchronous, no background child, so what you see is the whole flow:
```
$ vc-x1 squash-push -s a -R "$bot"
squash-push: squashing @ -> @-...
squash-push: setting bookmark 'main' to @-...
squash-push: pushing 'main' to origin...
squash-push: done
```

Preflight failures (bookmark missing, non-tracking remote, squash revset unresolved, conflicts, push
target lacks a description) exit with a non-zero status and a pointed error on stderr before
anything is rewritten.

### Testing the ochid-trailer guard

`squash-push` refuses to drop `ochid:` trailers: when the squash source's message carries trailers
the destination's message lacks, `--use-destination-message` would silently discard them and leave
the counterpart repo's cross-links dangling (the incident is recorded in [record finalize ochid-loss
bug (0.65.1)](./notes/chores/chores-13.md#docs-record-finalize-ochid-loss-bug-0651)). The guard runs
in preflight, before anything is rewritten.

`squash-push` always pushes, so the examples run against a throwaway init fixture (same shape as
[Testing push + squash-push](#testing-push--squash-push)) using its bot repo:

```bash
parent=$(mktemp -u /tmp/vc-x1-guard-XXXXXX)
vc-x1 init "$parent/work" --repo local="$parent"
bot="$parent/work/.claude"

# example 1, refusal: journal described on @ WITH an ochid
# trailer; squash-push exits 1 naming the trailer
echo b >> "$bot/notes.md"
jj describe -R "$bot" -m 'new journal

ochid: /abc123abc123'
vc-x1 squash-push -s a -R "$bot"

# example 2, normal case: clear the description; the squash
# proceeds and the run pushes
jj describe -R "$bot" -m ''
vc-x1 squash-push -s a -R "$bot"

# cleanup (also remove the fixture's ~/.claude/projects symlink)
rm -rf "$parent"
```

Output captured from a real run, and these transcripts are regenerated with
[`support/gen-exmpl-1-3.sh`](support/README.md#gen-example-1---3-output). Example 1 is refused with
exit 1:

```
$ vc-x1 squash-push -s a -R "$bot"
error: refusing squash @ -> @-: the squash would drop ochid: trailers
the destination's message lacks:
  /abc123abc123
merge the messages by hand (`jj describe @- -R /tmp/vc-x1-guard-hmevVM/work/.claude`) or clear
the source's description, then retry
```

Example 2 with the description cleared squashes and pushes, exit 0:

```
$ vc-x1 squash-push -s a -R "$bot"
squash-push: squashing @ -> @-...
squash-push: setting bookmark 'main' to @-...
squash-push: pushing 'main' to origin...
squash-push: done
```

## Cross-repo Linking with Git Trailers

Commits in each repo use [git trailers](https://git-scm.com/docs/git-interpret-trailers) to
cross-reference their counterpart in the other repo. The `ochid` (Other Change ID) trailer contains
a workspace-root-relative path and jj changeID:

```
ochid: /.claude/xvzvruqo   # points to a .claude repo change
ochid: /wtpmottv            # points to an work repo change
```

Paths always start with `/` (the workspace root, i.e. vc-x1). Each repo has a `.vc-config.md` that
identifies its location within the workspace, so tools can resolve these paths locally.

For full details see:
- [Git trailer convention](./notes/chores/chores-01.md#git-trailer-convention)
  - [ochid (Other Change ID)](./notes/chores/chores-01.md#ochid-other-change-id)
  - [ChangeID path syntax](./notes/chores/chores-01.md#changeid-path-syntax)
  - [.vc-config.toml](./notes/chores/chores-01.md#vc-configtoml)

## jj Tips for Git Users

Coming from git, start with these. One home per fact, so this section points rather than
repeats:

- [agent-data/jj.md](agent-data/jj.md): the project's quick reference (command basics, the
  working-copy `@` model, revset addressing) plus the dual-repo linking rules. Pinned and
  human-readable, and `agent-data/` holds the universal rule files, not agent-only material.
- The [jj docs](https://docs.jj-vcs.dev/latest/) and
  [Steve Klabnik's Jujutsu tutorial](https://steveklabnik.github.io/jujutsu-tutorial/): the
  maintained tutorials, and generic jj pedagogy lives upstream, not in this repo.
- `jj-tips.md` in the template repository (path recorded in custom.md): the family's worked
  tutorial with terminal transcripts, e.g. why `jj log` shows fewer commits than `gitk --all`
  and how force-pushes look in jj.
- [Read a change in gitk at full context](agent-data/jj.md#read-a-change-in-gitk-at-full-context):
  what gitk's New version, Old version and Diff views each show once the context is raised past
  the longest file, and how to read a whole range that way.

## The session lock

An optional agent-files rule, off in this workspace since 2026-10-13: while an agent session runs
it holds an exclusive `flock` on `.agent-running` at the workspace root, so a second session
started in the same workspace stops at its acquaint instead of working beside the first. Without
it, two sessions in one workspace lose no data but leave a wrong record, and keeping to one
session per workspace is the user's job ([What two sessions do with no
lock](notes/session-lock.md#what-two-sessions-do-with-no-lock)). It needs Linux's `flock` tool:
macOS does not ship it and Windows has neither the tool nor the call.

The instruction an agent follows is the entry in
[agent-data/session-lock.md](agent-data/session-lock.md#the-instruction), the set's copy, and the
design, the tests, and why it was turned off are in [notes/session-lock.md](notes/session-lock.md).

### Turning it on

1. Copy the entry in [The instruction](agent-data/session-lock.md#the-instruction) into
   `custom.md > ## Project conventions and overrides`, first in its list.
2. Add `/.agent-running` to the work-repo's `.gitignore`.
3. Start a new agent session. An edit to the instruction reaches only the sessions started after
   it, so a running session goes on without the lock.

Turning it off is undoing both edits, and the next session starts without it.

### Checking that it works

- The session's acquaint begins with `flock -xn .agent-running true`, run alone, and the session
  says "Lock taken." once it has started its holder, `flock -xn .agent-running sleep infinity`,
  as a background command.
- From a shell at the workspace root while the session runs, `flock -xn .agent-running true;
  echo $?` prints `1`, the lock held, and `fuser -v .agent-running` names the holder, a `flock`
  and its `sleep`. With no session running the same probe prints `0`.
- A second session started in the workspace and told "acquaint" responds "Another session holds
  `.agent-running`, stopping." and runs no further command.
- When the first session exits, its holder exits with it, and the probe prints `0` again.

Two known faults, neither fixed ([Turning it on
again](notes/session-lock.md#turning-it-on-again)):

- A session that has forgotten it holds the lock probes again at a reacquaint, fails against its
  own holder, and stops as if another session held it.
- A session that is killed rather than ended can leave the sandbox's empty placeholder files on
  disk, the holder being a background command ([The sandbox's placeholder
  files](notes/session-lock.md#the-sandboxs-placeholder-files)).

## Testing

The repo ships two flavors of tests:

- **Unit tests**: `#[cfg(test)] mod tests { ... }` blocks inside `src/*.rs`. These call
  library code directly (no subprocess spawn) and run fastest. The dual / POR fixture tests under
  `src/init.rs::tests` build throwaway workspaces by invoking `init::init` as a function.
- **CLI subprocess integration tests**: files under `tests/`. These spawn the `vc-x1` binary that
  Cargo built, and its absolute path lives in the `CARGO_BIN_EXE_vc-x1` env var that Cargo sets at
  compile time, which `env!("CARGO_BIN_EXE_vc-x1")` reads and bakes into the test binary as a string
  literal. Argument parsing, exit codes, and stdout/stderr are all exercised end-to-end. Each
  subprocess gets a `HOME` override so the user's real `~/.config/vc-x1/` can't leak in or get
  clobbered.

Run everything with:

```bash
cargo test                 # unit + integration
cargo test --bins          # unit tests only, no binary spawned
cargo test --test cli_init # one integration test crate
```

### Acceptance fixtures

The lookup's tests run over pre-created dual workspaces with documented relationships, one per
name under the directory the work side's `[test] fixtures` key names, `.fixtures` here, inside
the workspace and gitignored. The fixture tests are `#[ignore]`, so `cargo test` passes on a bare
clone and `cargo test -- --include-ignored`, the full validation, runs them. A test whose fixture
is absent panics, naming the directory and the command that builds the fixture there, so a missing
fixture fails rather than passes.

- `dr-1`, `dr` for dual repo, is the simple one-to-one shape: a work repo and its agent repo at
  `.claude`, laid out as `vc-x1 clone` lays a dual workspace out, run through four cycles by
  the scripted agent in `support/fixtures/build-dr-1.py`. Its README lists every relationship
  and `relationships.json` beside it carries them for the tests. Its remotes are
  `github.com/winksaville/vc-x1-fixtures-dr-1-work` and `vc-x1-fixtures-dr-1-agent`.
- To build it: `python3 support/fixtures/build-dr-1.py .fixtures --local`, the command the
  panic names. `git clone` of the two remotes, then `jj git init --colocate` in each, is the
  other way. **dr-1 is built in Rust** in `TODO.md` retires the script.
- A one-work-to-many-agents fixture is a numbered sibling, `dr-2`.

### Test tempdir location

Both test layers create throwaway fixtures under a tempdir. The parent directory resolves in
priority order:

1. `$VC_X1_TEST_TMPDIR`: explicit env override.
2. `std::env::temp_dir()`: standard fallback (`$TMPDIR` on Unix, else `/tmp`).

Useful when you want tests on a tmpfs / SSD / project-local path without exporting `TMPDIR`
globally:

```bash
VC_X1_TEST_TMPDIR=/dev/shm/vc-x1 cargo test
```

Fixture directories are named `vc-x1-test-<tag>-<ts>-<n>` (unit) and
`vc-x1-cli-test-<tag>-<ts>-<n>` (CLI). RAII drop removes them on test exit, while SIGKILLs /
panics in `Drop` can leak, so search and clean manually:

```bash
find "${VC_X1_TEST_TMPDIR:-${TMPDIR:-/tmp}}" -maxdepth 1 \
     -name 'vc-x1-*test*' -mtime +1 -exec rm -rf {} +
```

A future enhancement (tracked in `TODO.md` at the repo root) extends the priority chain through
the project-local `.vc-config.md`.

### Preserving fixtures for debugging

When a test fails and you want to inspect the workspace it built, set `$VC_X1_TEST_KEEP`. The RAII
`Drop` impls (`Fixture`, `FixturePor`, `CliFixture`) skip `remove_dir_all` and print the preserved
path on stderr:

```bash
VC_X1_TEST_KEEP=1 cargo test -- --nocapture 2>&1 | grep TEST_KEEP
```

Two shell gotchas worth remembering:

- The preservation message goes to **stderr**, so a plain `cargo test | grep ...` won't see it. Use
  `2>&1 |` or write the full output to a file.
- `--nocapture` is needed to bypass libtest's stdout/stderr capture, and without it the messages get
  swallowed by the test runner's pretty-printer.

`VC_X1_TEST_KEEP` is a debugging knob: every fixture-creating test in the run leaks its tempdir
while it's set. Clean up with the `find` recipe above, or just `rm -rf` the announced paths.

### Debugging: seeing debug! output from a test

The crate logs through the `log` crate facade, but only the CLI installs a logger (`CliLogger::init`
at startup, driven by `-v`), so under `cargo test` every `debug!` call is a silent no-op. To watch
the debug output of one test, install the logger at the top of that test, temporarily:

```rust
// 1 = debug, 2 = trace; Some("path") also captures to a file
crate::logging::CliLogger::init(1, None);
```

and run just that test with capture disabled:

```bash
cargo test --bins <test-name-substring> -- --nocapture
```

For example, the index-lock retry test (`mutation_survives_transient_index_lock` in `src/jj.rs`)
then shows the backoff live on stderr:

```
git lockfile contended (attempt 1): Could not acquire lock for index file; retrying in 25ms
git lockfile contended (attempt 2): Could not acquire lock for index file; retrying in 50ms
git lockfile contended (attempt 3): Could not acquire lock for index file; retrying in 100ms
test jj::tests::mutation_survives_transient_index_lock ... ok
```

**The zero-setup variant.** For a quick one-off look, skip the logger entirely: drop a temporary
`println!` at the point of interest (right next to the `debug!` is natural) and run with
`--nocapture`. `println!` needs no logger, no level, no init line in the test:

```bash
cargo test --bins <test-name-substring> -- --nocapture
```

Same output, one line of setup. The trade-offs: it prints for every caller (no level to turn it
off), and unlike the test-local `init` line it sits in production code, where a forgotten one ships
chatter into every real CLI run's stdout. Remove it before committing, since `debug!` is the form
that stays.

Gotchas, in the order people hit them:

- `CliLogger::new(...)` alone does nothing: it only builds the value. `init` is what registers the
  global logger and raises `log::max_level`, and without both `debug!` stays a no-op.
- `--nocapture` is required, same as the fixture-preserving recipe above: debug lines go to stderr
  and libtest swallows them otherwise.
- `RUST_LOG` has no effect: `CliLogger` never reads env vars (that is env_logger's convention, not
  this logger's).
- Remove the `init` line when done. The logger is process-global, so left in place it makes every
  test in the binary chatty, and a second test calling `init` would panic: the global logger can be
  set only once per process.

## Support

Helper scripts for maintaining this repo's docs and examples live in [`support/`](support/), and see
[support/README.md](support/README.md).

### gen-exmpl-1-3.sh

Regenerates the captured transcripts for [Testing the ochid-trailer
guard](#testing-the-ochid-trailer-guard) by running the two examples against a throwaway init
fixture and printing the `$ command` + output blocks ready to paste. Details: [Gen Example 1 - 3
output](support/README.md#gen-example-1---3-output).

## Thoughts for the future

Forward-looking design discussion lives in [`notes/forks-multi-user.md`](notes/forks-multi-user.md):
forking the dual-repo workspace, multi-user collaboration, multi-line `ochid:` trailers, bot-repo
size and scaling thresholds, URL-shaped per-user repos for distributed projects. Treat as a design
reference, not a status doc, and most of what's there is forward-looking, not yet implemented.

## Contributing

The tool's internal structure (module map, the CLI-args / ops-`Context`+`Params` split, the
subcommand model, and the two in-flight migrations) is described in
[ARCHITECTURE.md](ARCHITECTURE.md). Start there to orient.

The agent-facing cycle workflow (opening / commits / closing, the per-rung flow with `vc-x1
validate`, commit description shape, ochid trailers, pushing) is AGENTS.md's
[Cycle protocol](AGENTS.md#cycle-protocol).

Bot-facing conventions are canonical in [AGENTS.md](AGENTS.md) (hard rules + file map), its
`agent-data/` satellites, and the project layer [custom.md](custom.md):

- [Notes file conventions](agent-data/notes.md): Todo format, Reference numbering, Notes references
  (`[[N]]` citation style), Markdown anchor links, Retiring Done entries, and Chores conventions
  (section headers / Done entries exact-title rule, content rules, `Commits:` line format).
- [Prose form](agent-data/prose.md#prose-form): intro + bullets shape for long-lived prose (commit
  bodies, chores, doc comments).
- [Code conventions](agent-data/code.md): doc comments on every file / fn / method, `// OK: ...` on
  `unwrap*` calls.

Task tracking and release details: near-term tasks in [TODO.md](TODO.md), per-release details in
`notes/chores/chores-*.md`, and notes-specific formatting rules in
[notes/README.md](notes/README.md).

### No process spawns

vc-x1 runs jj and git through the jj-lib and gix libraries, never by spawning the `jj` or `git`
binaries, with one temporary exception named below. The ban is enforced, and the exceptions are
not: there is no allowlist. What exists is
three things, and it pays to know which is which.

- **The ban is config.** `clippy.toml` lists `std::process::Command::new` under
  `disallowed-methods`, and `Cargo.toml` sets that lint to deny, so a bare call anywhere fails
  `cargo clippy`, which `vc-x1 validate` runs.
- **An exception is an attribute.** Clippy has no way to list permitted sites, so the only grant
  is `#[allow(clippy::disallowed_methods)]` on the call itself, with a comment naming the entry
  it claims. Ten sites carry one today.
- **The register is prose.** The numbered comment at the top of `clippy.toml` names the five
  kinds of spawn that are permitted and why: the `jj -V` version probe, init's
  `gh` calls, the test helpers and the CLI tests' launcher, validate and build running the
  configured commands, and sync's remote look, `git ls-remote` and a `git fetch` that moves no ref. The
  last is the one spawn of `git`, and is temporary: jj-lib has no query for the commits a
  remote's bookmarks are on, and the entry closes when it offers one. Nothing checks the
  register against the attributes. A site with an attribute and no
  entry passes the build and is a review finding.

To see every granted site:

```bash
rg '^\s*#.allow.*disallowed_methods' -A 1
```

jj-lib's own git children, the network legs of fetch, push and clone, are jj-lib's business and
no lint here sees them.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
