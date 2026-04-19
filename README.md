# vc-x1

- [Overview](#vc-x1)
- [Usage](#usage)
  - [Revision shortcuts](#revision-shortcuts)
  - [Shell completion](#shell-completion)
  - [validate-desc](#validate-desc)
  - [fix-desc](#fix-desc)
  - [clone](#clone)
  - [init](#init)
  - [symlink](#symlink)
  - [finalize](#finalize)
  - [test-fixture](#test-fixture)
  - [Testing push + finalize](#testing-push--finalize)
- [Cross-repo Linking with Git Trailers](#cross-repo-linking-with-git-trailers)
- [jj Tips for Git Users](#jj-tips-for-git-users)
- [Contributing](#contributing)
- [License](#license)

This is experiment 1 to explore creating a Vibe Coding (vc) environment.
We will investigate ways of using the dual jj-git repo concept, explored
in [hw-jjg-bot](https://github.com/winksaville/hw-jjg-bot.git) to
initially make it easy to see how the code base evolved. This is
made possible by the fact that we have two repos one with the code
and one with the conversation with the bot.

I've chosen the jj-git environment because jj provides the concept that
each commit has an immutable changeID as well as the mutable commitID
of git. The idea is that each commit made on repo A writes the
changeID in the commit message to repo B. Thus there is a cross reference
between the two repos and this will allow vc-x1 to show how the repo
evolved and the entity (bot or human) can more clearly understand **how** and
most importantly **why** the code evolved.

The solution space is wide open, from trivial CLI, web or app based
(mobile/non-mobile). In addition, I could see this as an extension to
existing programming editors like vscode and zed or even creating our
own IDE for vc.

See [Initial commit with dual jj-git repos](./notes/chores-01.md#initial-commit-with-dual-jj-git-repos)
for how the initial commit was created with the dual jj-git repos. After
doing so and I then created this README.md file.

## Usage

```
vc-x1 list [-r REVISION] [-n COMMITS]  # List commits in a jj repo
vc-x1 desc [-r REVISION] [-n COMMITS]  # Show full description of a commit
vc-x1 chid [-r REVISION] [-n COMMITS]  # Print changeID(s) for a revision
vc-x1 show [-r REVISION] [-n COMMITS]  # Show commit details and diff summary
vc-x1 validate-desc [OPTS]                 # Validate commit descriptions
vc-x1 fix-desc [OPTS]                     # Fix commit descriptions (dry-run default)
vc-x1 clone <REPO> [NAME] [OPTS]          # Clone a dual-repo project
vc-x1 init <NAME> [OPTS]                  # Create a new dual-repo project
vc-x1 symlink [TARGET] [OPTS]             # Create Claude Code project symlink
vc-x1 finalize --bookmark <B> [OPTS]       # Squash working copy into target
vc-x1 test-fixture [--path PATH]           # Create throwaway jj repo + remote for testing
vc-x1 --version                            # Print version
vc-x1 --help                           # Print help
```

**The `..` notation:** dots on the revision show which direction to
list commits:

- `x..` — x at top, ancestors below (older commits)
- `..x` — descendants above (newer commits), x at bottom
- `..x..` — both directions, x in the middle

COMMITS is the total number of commits to show (x is always included).
Without dots, `x` with a count defaults to `x..` (ancestors).
For `..x..`, the budget is split: ancestors get the extra on odd counts.

```
vc-x1 list -r @              # just @ (1 commit)
vc-x1 list -r @.. -n 5       # @ + 4 ancestors (5 commits, @ at top)
vc-x1 list -r ..@ -n 3       # 2 descendants + @ (3 commits, @ at bottom)
vc-x1 list -r ..@.. -n 3     # 1 descendant, @, 1 ancestor (3 commits)
vc-x1 list -r ..@.. -n 4     # 1 descendant, @, 2 ancestors (4 commits)
```

### Shell completion

vc-x1 provides tab completion for all subcommands and flags using claps
[unstable-dynamic](https://docs.rs/clap_complete/latest/clap_complete/env/struct.CompleteEnv.html)
only feature. It is a "simple" implementation and does not handle completing
revisions but still useful. To enable it, add one of the following to your shell's startup file:

```bash
# bash (~/.bashrc)
source <(COMPLETE=bash vc-x1)

# zsh (~/.zshrc)
source <(COMPLETE=zsh vc-x1)

# fish (~/.config/fish/config.fish)
source (COMPLETE=fish vc-x1 | psub)
```

Completions are generated dynamically by the binary, so they stay in
sync with the installed version automatically.

### Positional shorthand

REVISION and COMMITS can be given as positional arguments, omitting
`-r` and `-n`:

```
vc-x1 list x                # just x (1 commit)
vc-x1 list x 5              # x.. 5 → x + 4 ancestors (5 commits)
vc-x1 list x.. 5            # same as above
vc-x1 list ..x 3            # 2 descendants + x (3 commits)
vc-x1 list ..x.. 3          # 1 descendant, x, 1 ancestor (3 commits)
vc-x1 list ..x.. 1          # just x (1 commit)
```

**Defaults:** with no arguments, REVISION is `@` and COMMITS is 1
(just the working copy).

Named flags `-r`/`--revision`, `-n`/`--commits`, and `-R`/`--repo`
take precedence over positional arguments.

### Multi-repo queries

The `-R`/`--repo` flag can be repeated or comma-separated to query
multiple repos at once. When multiple repos are given, output is
labeled with bold `=== path ===` headers by default:

```
vc-x1 chid -R . -R .claude      # repeated flag
vc-x1 chid -R .,.claude         # comma-separated
vc-x1 list @.. 3 -R .,.claude   # works with all read-only subcommands
```

Control the label between repos with `-l`/`--label` and `-L`/`--no-label`:

```
vc-x1 chid -R .,.claude            # default label: === path ===
vc-x1 chid -R .,.claude -l "---"   # custom label:  --- path ---
vc-x1 chid -R .,.claude -L         # no label (raw output)
```

Examples:

```
$ vc-x1 chid -R . -R .claude
=== . ===
kwoyvposvsmv

=== .claude ===
zzpksklxyrnw

$ vc-x1 chid -R . -R .claude -L
kwoyvposvsmv
zzpksklxyrnw

$ vc-x1 chid -R .,.claude
=== . ===
kwoyvposvsmv

=== .claude ===
zzpksklxyrnw

$ vc-x1 chid -R .,.claude -L
kwoyvposvsmv
zzpksklxyrnw

$ vc-x1 list @.. 3 -R .,.claude
=== . ===
kwoyvposvsmv eae175c3c1b5 (no description set)
pqxtxxpnsmot b0c46930a640 Reorganize notes (0.19.1)
ssokyszzwsxw eeb15bc6d839 Unify .. notation and CLI (0.19.0)

=== .claude ===
zzpksklxyrnw 2388185e42ee (no description set)
tzupykyyvnrp 81ce22e34d41 Reorganize notes (0.19.1)
slmkmroqtqtp 1c33ccaa567d Unify .. notation and CLI (0.19.0)

$ vc-x1 chid -R .,.claude -L
kwoyvposvsmv
zzpksklxyrnw

$ vc-x1 desc -r @- -R .,.claude
=== . ===
pqxtxxpnsmot b0c46930a640 Reorganize notes: move older done items to done.md, update todos (0.19.1)

    Move completed items [1]-[13] and their references from todo.md to done.md.
    Replace completed jj-organization todo with new items.

    ochid: tzupykyyvnrp

=== .claude ===
tzupykyyvnrp 81ce22e34d41 Reorganize notes: move older done items to done.md, update todos (0.19.1)

    Session: reviewed and committed notes reorganization for 0.19.1.

    ochid: pqxtxxpnsmot

$ vc-x1 chid -R .,.claude -l "---"
--- . ---
kwoyvposvsmv

--- .claude ---
zzpksklxyrnw
```

Most decoration strings work unquoted (`---`, `===`, `>>>`, `:::`,
`+++`). Use single quotes for strings containing shell metacharacters
like `*`, `!`, `#`, `$`, or `~` (e.g. `-l '***'`). Double quotes
won't protect against `!` (bash history expansion).

With a single repo (or no `-R`), no label is printed — backward
compatible with previous behavior. Multi-repo is supported for
`chid`, `desc`, `list`, and `show`. `finalize` remains single-repo.

### validate-desc

Read-only scan of commit descriptions against the other repo. The other
repo is read from `.vc-config.toml` (`workspace.other-repo`) by default,
or overridden with `--other-repo`. Reports status per commit: `ok`
(valid ochid), `lost` (ochid: lost), `none` (ochid: none), `err`
(issues found), or `miss` (no ochid trailer).

```
# Validate all ancestors of @ (reads other repo from .vc-config.toml)
vc-x1 validate-desc @..

# Validate with explicit other repo
vc-x1 validate-desc --other-repo .claude @..

# Validate specific range
vc-x1 validate-desc -r @.. -n 20
```

Use `--help` for the full status label legend.

### fix-desc

Fix commit descriptions against the other repo. Default is dry-run —
use `--no-dry-run` to write changes. Reads other repo from
`.vc-config.toml` by default.

```
# Dry-run: show what would be fixed
vc-x1 fix-desc @..

# Actually fix
vc-x1 fix-desc @.. --no-dry-run

# Add missing ochid trailers by matching title
vc-x1 fix-desc @.. --add-missing

# Limit fixes
vc-x1 fix-desc @.. --no-dry-run -m 3

# Use a fallback for IDs not found in other repo
vc-x1 fix-desc @.. --fallback /.claude/lost
```

| Flag | Description |
|------|-------------|
| `--other-repo <PATH>` | Override other repo from .vc-config.toml |
| `--no-dry-run` | Write fixes [default: dry-run] |
| `--add-missing` | Infer and add ochid for commits without one |
| `-m, --max-fixes <N>` | Stop fixing after N commits changed [default: all] |
| `--fallback <VALUE>` | Replacement for IDs not found in other repo |
| `--id-len <N>` | Expected changeID length [default: 12] |
| `--title <TEXT>` | Replace commit title at the same time |

Use `--help` for the full status label legend.

### clone

Clone an existing dual-repo project. Runs `git clone --recursive` to
get both repos, initializes `jj` in each, and creates the Claude Code
symlink.

```
# Clone using GitHub shorthand
vc-x1 clone owner/my-project

# Clone using full URL
vc-x1 clone git@github.com:owner/my-project.git

# Clone with a custom directory name
vc-x1 clone owner/my-project my-local-name

# Clone into a specific parent directory
vc-x1 clone owner/my-project --dir ~/projects

# Preview without executing
vc-x1 clone owner/my-project --dry-run
```

| Flag | Description |
|------|-------------|
| `--dir <PATH>` | Parent directory [default: cwd] |
| `--dry-run` | Show what would be done without executing |
| `-v, --verbose` | Verbose output |

Requires `jj` to be installed. The `.claude` session repo is cloned
automatically via `git submodule` if the source project was created
with `vc-x1 init`.

### init

Create a new dual-repo project — a code repo with a `.claude` session
repo as a git submodule. Both repos are initialized with `git` and `jj`,
configured with `.vc-config.toml`, and pushed to GitHub. The session
repo is added as a submodule so `git clone --recursive` clones both.

```
# Create public project in current directory
vc-x1 init my-project

# Specify owner and parent directory
vc-x1 init my-project --owner myorg --dir ~/projects

# Create private repos
vc-x1 init my-project --private

# Preview without executing
vc-x1 init my-project --dry-run

# Seed both repos from template directories (sibling layout)
vc-x1 init my-project --use-template ../vc-template-x1
# Equivalent to:
vc-x1 init my-project --use-template ../vc-template-x1,../vc-template-x1.claude
```

| Flag | Description |
|------|-------------|
| `--owner <OWNER>` | GitHub user/org [default: current `gh` user] |
| `--dir <PATH>` | Parent directory [default: cwd] |
| `--private` | Create private GitHub repos [default: public] |
| `--dry-run` | Show what would be done without executing |
| `--push-retries <N>` | Max push retries after repo creation [default: 5] |
| `--push-retry-delay <N>` | Seconds between push retries [default: 3] |
| `--use-template <CODE[,BOT]>` | Seed both repos from template dirs (see below) |
| `-v, --verbose` | Verbose output (show retry details) |

**`--use-template`**. Value is `CODE[,BOT]`. If `BOT` is omitted, defaults
to the sibling directory `<CODE>.claude` (file-name concat, not path
join — the two templates are not nested). Non-hidden contents are
copied recursively into each target; hidden entries (names starting
with `.`) are skipped since init creates the repo's own hidden files
(`.vc-config.toml`, `.gitignore`, `.git/`, `.jj/`). If either template
has a `README.md` at its root, its first line is rewritten to
`# <repo-name>` — `<name>` for the code repo and `<name>.claude` for
the session repo. The same flag is also available on `test-fixture`
for local verification without hitting GitHub.

Requires `gh` (authenticated) and `jj` to be installed.

### symlink

Create or verify the Claude Code project symlink. Claude Code stores
session data in `~/.claude/projects/<encoded-path>/`. This command
creates a symlink from that location to the local `.claude` directory.

```
# Create symlink for current project (default target: .claude)
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

### finalize

Atomically squash, set bookmark, and push a jj repo. The primary use
case is the bot finalizing its own session repo (`.claude`) at the end
of a session. The bot can't just `jj commit` because the act of
committing generates more session data — files written after the commit
would be lost. `finalize` solves this by:

1. **`--detach`**: spawning a background process so the bot session can
   end immediately (no more writes after the bot exits)
2. **`--delay`**: waiting for trailing writes to settle
3. **`--squash`**: squashing the working copy into the session commit
4. **`--bookmark`** + **`--push`**: advancing the bookmark and pushing

Every behavior is opt-in — omit any flag to skip that step.

The bot's last action in a session:

```
vc-x1 finalize --repo .claude --squash --bookmark main --delay 10 --detach --push
```

If there is non-written session data after a session ends (e.g.
finalize failed or was skipped), run it manually:

```
vc-x1 finalize --repo .claude --squash --bookmark main --push
```

This runs in the foreground, squashes, advances the bookmark, and pushes.

Other uses — `finalize` composes freely:

```
# Just set bookmark, no squash
vc-x1 finalize --bookmark main

# Squash with custom source/target
vc-x1 finalize --squash @,@-- --bookmark main
```

See [finalize subcommand](./notes/chores-01.md#finalize-subcommand-for-session-repo-coherence)
for design details.

### test-fixture

Scaffold a throwaway dual-repo jj workspace + local bare-git remotes
for testing `finalize` (and other subcommands) without touching live
workspace repos. Mirrors the real `vc-x1 init` layout minus the GitHub
side and the `~/.claude/projects/` symlink. Both repos get a described
initial commit with matching `ochid:` trailers, a tracked `main`
bookmark, and a pushed remote — so `finalize --push` flows work
end-to-end on either side.

**Local remotes, not GitHub.** Each `origin` points at a bare-git
directory alongside the work trees (`<base>/remote-code.git/`,
`<base>/remote-claude.git/`) — no network, no auth, no GitHub. Pushes
succeed against these local bare repos and stay inside the fixture,
so nothing leaks out and nothing needs cleanup on a remote service.
When you're done, `vc-x1 test-fixture-rm <base>` wipes the whole
thing.

```bash
vc-x1 test-fixture                 # base = $TMPDIR/vc-x1-test-<timestamp>
vc-x1 test-fixture --path /tmp/t1  # explicit path
vc-x1 test-fixture --use-template ../vc-template-x1  # seed from sibling templates
```

`--use-template` takes the same `CODE[,BOT]` value as `vc-x1 init`
(bot defaults to `<CODE>.claude` sibling). Non-hidden template contents
are copied into `work/` and `work/.claude/`, and each repo's
`README.md` first line is rewritten to `# work` / `# work.claude`.
This is the path for eyeballing the template-copy result without
hitting GitHub.

Layout:
```
<base>/
  remote-code.git/     bare git remote for code repo
  remote-claude.git/   bare git remote for .claude session repo
  work/                code repo (jj colocated, main tracks origin)
    .vc-config.toml    path="/",       other-repo=".claude"
    .gitignore         /.claude /.git /.jj /target
    .claude/           session repo (jj colocated, main tracks origin)
      .vc-config.toml  path="/.claude", other-repo=".."
      .gitignore       .git .jj
```

Example — running `vc-x1 test-fixture` (the timestamp suffix varies):
```
$ vc-x1 test-fixture
Creating test fixture at /tmp/vc-x1-test-8PD4x8
Step 1: Initializing bare git remotes...
Step 2: Initializing work repo (jj colocated)...
Initialized repo in "."
Hint: Running `git clean -xdf` will remove `.jj/`!
Step 3: Initializing .claude session repo (jj colocated)...
Initialized repo in "."
Hint: Running `git clean -xdf` will remove `.jj/`!
Step 4: Initial commits with placeholder ochids...
Working copy  (@) now at: znkwzomz 0ce34d9b (empty) (no description set)
Parent commit (@-)      : xoknyroz 93ca19eb initial commit
Working copy  (@) now at: kmtpuzox f8c9b5af (empty) (no description set)
Parent commit (@-)      : mwutluxv 40438e05 initial commit
Step 5: Setting ochid cross-references...
Rebased 1 descendant commits
Step 6: Setting bookmarks and wiring remotes...
Created 1 bookmarks pointing to xoknyroz 46b9cd88 main | initial commit
Created 1 bookmarks pointing to mwutluxv 31a49010 main | initial commit
Step 7: Pushing main to both remotes...
Changes to push to origin:
  Add bookmark main to 46b9cd886e64
Changes to push to origin:
  Add bookmark main to 31a490102db7

Fixture ready (local bare-git remotes, see README.md § test-fixture):
  Code repo:     /tmp/vc-x1-test-8PD4x8/work
  Session repo:  /tmp/vc-x1-test-8PD4x8/work/.claude
  Code remote:   /tmp/vc-x1-test-8PD4x8/remote-code.git
  Claude remote: /tmp/vc-x1-test-8PD4x8/remote-claude.git

Next steps — see README.md § Testing push + finalize for the full flow.
Quick reference with this fixture's paths:
  jj git push -R /tmp/vc-x1-test-8PD4x8/work
  vc-x1 finalize --repo /tmp/vc-x1-test-8PD4x8/work/.claude --squash --push main --detach
  vc-x1 test-fixture-rm /tmp/vc-x1-test-8PD4x8
```

### Testing push + finalize

Always test against a throwaway fixture, never the live workspace.
Scaffold one with `test-fixture` (above), then run the complete
push + finalize flow end-to-end. The code repo uses plain
`jj git push`; the session repo uses `vc-x1 finalize` to squash
trailing writes and push in one shot.

```bash
base=$(mktemp -u /tmp/vc-x1-test-XXXXXX)
vc-x1 test-fixture --path "$base"
work="$base/work"
session="$base/work/.claude"

# 1. code repo: described commit → advance main → push
echo hello > "$work/hello.txt"
jj describe @ -R "$work" -m 'feat: add hello.txt'
jj bookmark set main -r @ -R "$work"
jj git push -R "$work"

# 2. session repo: trailing writes → finalize (squash into @-, push)
echo notes > "$session/notes.md"
vc-x1 finalize --repo "$session" --squash --push main --detach \
    --log "$session/finalize.log"

# 3. inspect the detached child's log once it's done (≈10s by default)
sleep 12 && cat "$session/finalize.log"

# 4. cleanup when done
vc-x1 test-fixture-rm "$base"
```

**Why `jj git push` for code but `finalize` for `.claude`?** The
code repo's workflow is a plain dev commit on `@-` that we push
directly. The session repo mirrors the bot's runtime pattern:
session writes land in `@` (above the last committed dev commit),
and `finalize --squash @,@-` folds those trailing writes into the
dev commit just before pushing, so one atomic state goes upstream.

The log file shows timestamped (nanoseconds) entries with PIDs,
covering the full flow: `main` entry/exit, `finalize` entry/exit,
`detach` spawn, and `finalize_exec` in the child process.

Example — detached finalize against a fresh fixture. What the user
sees in the terminal (the parent process) is the pre-flight plan and
the detach confirmation; the child's work continues in the background:
```
$ vc-x1 finalize --repo "$session" --squash --push main \
    --detach --delay 1 --log "$session/finalize.log"
finalize: squash @ → @- in /tmp/vc-x1-test-8PD4x8/work/.claude
finalize: set bookmark 'main' mwutluxv 31a49010 → mwutluxv 31a49010 (@-)
finalize: push 'main' to remote
finalize: detached (pid 103787), log: /tmp/vc-x1-test-8PD4x8/finalize.log
```

A few seconds later the log file (authoritative when the caller
closes the child's pipes) shows the full run, including the child's
own squash/push output:
```
$ cat "$session/finalize.log"
[INFO ] vc_x1::finalize: finalize: squash @ → @- in /tmp/vc-x1-test-8PD4x8/work/.claude
[INFO ] vc_x1::finalize: finalize: set bookmark 'main' mwutluxv 31a49010 → mwutluxv 31a49010 (@-)
[INFO ] vc_x1::finalize: finalize: push 'main' to remote
[INFO ] vc_x1::finalize: finalize: detached (pid 103787), log: /tmp/vc-x1-test-8PD4x8/finalize.log
[INFO ] vc_x1::common: Working copy  (@) now at: ovumtpup fa1cb861 (empty) (no description set)
Parent commit (@-)      : mwutluxv 584571ba main* | initial commit
[INFO ] vc_x1::common: Nothing changed.
[INFO ] vc_x1::common: Changes to push to origin:
  Move sideways bookmark main from 31a490102db7 to 584571ba54af
```

Pre-flight failures (bookmark missing, non-tracking remote, squash
revset unresolved, push target lacks a description) exit the parent
synchronously with a non-zero status and a pointed error on stderr,
before the child is ever spawned.

## Cross-repo Linking with Git Trailers

Commits in each repo use [git trailers](https://git-scm.com/docs/git-interpret-trailers)
to cross-reference their counterpart in the other repo. The `ochid`
(Other Change ID) trailer contains a workspace-root-relative path
and jj changeID:

```
ochid: /.claude/xvzvruqo   # points to a .claude repo change
ochid: /wtpmottv            # points to an app repo change
```

Paths always start with `/` (the workspace root, i.e. vc-x1).
Each repo has a `.vc-config.toml` that identifies its location
within the workspace, so tools can resolve these paths locally.

For full details see:
- [Git trailer convention](./notes/chores-01.md#git-trailer-convention)
  — [ochid (Other Change ID)](./notes/chores-01.md#ochid-other-change-id)
  — [ChangeID path syntax](./notes/chores-01.md#changeid-path-syntax)
  — [.vc-config.toml](./notes/chores-01.md#vc-configtoml)

## jj Tips for Git Users

If you're coming from git, jj's log output can be surprising compared to
tools like `gitk --all`.

### Why `jj log` shows fewer commits than `gitk`

jj tracks *changes* (identified by change IDs), not individual git commits.
When you rewrite a change (`jj describe`, `jj rebase`, `jj squash`, etc.),
jj creates a new git commit and keeps the old one under `refs/jj/keep/*` as
undo history. `gitk --all` sees all of these obsolete commits; `jj log` only
shows the current version of each change.

### Useful commands

| Command | Description |
|---------|-------------|
| `jj log` | Show recent visible commits (default revset) |
| `jj log -r ::@` | Show **all** ancestors of the working copy |
| `jj log -r 'all()'` | Show all non-hidden commits (needed if you have multiple heads/branches) |
| `jj obslog -r <change-id>` | Show the evolution history of a single change |
| `jj op log` | Show operation history (each rewrite operation) |

In a single-branch workflow, `jj log -r ::@` and `jj log -r 'all()'` give
the same result. Use `all()` when you have multiple branches or heads.

## Contributing

Developer notes, conventions, and task tracking live in [notes/](notes/).
Start with [notes/README.md](notes/README.md), which covers:

- [Versioning during development](notes/README.md#versioning-during-development)
- [Code Conventions](notes/README.md#code-conventions) — including the
  `// OK: …` convention for `unwrap*` calls
- [Todo format](notes/README.md#todo-format)

Near-term tasks are in [notes/todo.md](notes/todo.md); per-release details
are in the `notes/chores-*.md` files.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
