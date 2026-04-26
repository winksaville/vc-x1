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
vc-x1 sync [OPTS]                          # Fetch + sync both repos to their remotes
vc-x1 finalize --bookmark <B> [OPTS]       # Squash working copy into target
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
the session repo. For local verification without hitting GitHub,
combine `--use-template` with `--repo-local <PARENT>`.

Requires `gh` (authenticated) and `jj` to be installed (`gh` is
skipped under `--repo-local`).

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

### sync

Fetch and sync a set of repos to their remotes in a single command.
Repo set defaults to the dual-repo workspace pair (`.` and
`.claude`); override with `-R` / `--repo` for single-repo projects
or arbitrary multi-repo workspaces. Dry-run by default — re-run with
`--no-dry-run` to apply.

Per repo, `sync` classifies the local bookmark against its remote:

| State | Meaning | Action on `--no-dry-run` |
|------|---------|--------------------------|
| up-to-date | local == remote | none |
| behind | local is ancestor of remote | `jj bookmark set <b> -r <b>@<remote>` |
| ahead | remote is ancestor of local | none (push is a separate step) |
| diverged | neither is ancestor | `jj rebase -b <local-head> -d <b>@<remote>` |
| no remote | bookmark has no `@<remote>` counterpart | none — skip |

After the bookmark action above, `sync` also rebases `@` onto the
(possibly advanced) bookmark when `@` isn't already a descendant —
without this step, `jj git fetch`'s auto-fast-forward would leave `@`
dangling off the pre-fetch bookmark commit. This matters for `.claude`,
where `/exit`'s trailing session writes always sit on `@`.

On any failure — conflicted rebase, subprocess error, anything — `sync`
restores every repo to its starting state via `jj op restore`. Either
every repo advances or none do. Working-copy files are preserved
across the revert: jj rewinds the operation log but leaves disk
content untouched, and any conflicted commits introduced by the failed
rebase are abandoned on the way back.

```
vc-x1 sync                            # workspace-default scope, --check
vc-x1 sync --check                    # explicit form of the default
vc-x1 sync --no-check                 # workspace-default scope, apply
vc-x1 sync --scope=code               # only the app repo
vc-x1 sync --scope=bot                # only the bot repo
vc-x1 sync --scope=code,bot           # both (explicit form of the dual default)
vc-x1 sync -R .                       # arbitrary repo set (no workspace lookup)
vc-x1 sync -R .,.claude -R ../other   # mixed: repeat + comma-separate
```

Scripts and automation should pass `--check` or `--no-check`
explicitly rather than rely on the default — defaults can shift,
explicit flags lock in the contract. Interactive use can take the
default.

**Repo set resolution.** Sync picks the repo list in this order:

1. `-R` / `--repo` — exactly that list.
2. `--scope=code|bot|code,bot` — workspace roles, resolved via the
   workspace root's `.vc-config.toml` (`code` → root,
   `bot` → `root.join(other-repo)`).
3. Neither — workspace-default scope: `code,bot` if
   `[workspace] other-repo` is non-empty, else `code`. POR (no
   `.vc-config.toml`) → `code` resolved to cwd.

`-R` and `--scope` are mutually exclusive — they answer different
questions ("arbitrary repo set" vs "workspace roles"). Scope is
cwd-portable: from `.claude/`, `vc-x1 sync` walks up to the
workspace root and resolves repos by absolute path.

| Flag | Description |
|------|-------------|
| `-R, --repo <PATH>` | Repo to sync; repeatable or comma-separated. Mutually exclusive with `--scope` |
| `--scope <SCOPE>` | `code|bot|code,bot` — workspace roles to sync. Mutually exclusive with `-R` |
| `--check` | Verify only — fetch + classify, error if any repo needs action (default) |
| `--no-check` | Apply — fetch + classify, then rebase/fast-forward as needed |
| `-q, --quiet` | Suppress all output; exit code signals result (for scripts) |
| `--bookmark <NAME>` | Bookmark to sync in each repo [default: main] |
| `--remote <NAME>` | Remote to sync against [default: origin] |

**Output shape.** Sync collapses output based on what it finds:

- **All up-to-date** — one-line summary:
  `sync: N repos, all bookmarks up-to-date`. Nothing else.
  Makes "sprinkle sync everywhere" genuinely cheap. Scope is
  bookmark-vs-remote tracking — `@` may have uncommitted
  working-copy changes; sync intentionally doesn't speak to
  that (use `jj st` for working-copy state).
- **Action needed** (`behind` / `diverged`) — per-repo fetch +
  state lines, then **fatal in `--check` mode** with
  `sync: N repos need action — resolve with vc-x1 sync --no-check
  and re-run`. Under `--no-check`, the actions run instead.
- **`--quiet`** — no output at any level; exit code is the only
  signal. Intended for scripts that just need success/failure.

**Note on the `behind` case.** jj's `git fetch` already fast-forwards a
tracked local bookmark when it's a strict ancestor of the incoming
remote, so in the common case `sync` reports `up-to-date` rather than
`behind`. The `behind` branch covers untracked bookmarks and edge
configs where auto-advance is disabled.

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

### push

Dual-repo commit+push+finalize in one resumable command with two
interactive approval gates. Replaces the old multi-step manual
choreography (`jj commit` × 2 → `jj bookmark set` × 2 →
`jj git push` → `vc-x1 finalize`) with a single invocation.

```bash
vc-x1 push main                                     # interactive
vc-x1 push main --title "..." --body "..."          # skip $EDITOR
vc-x1 push main --yes --title "..." --body "..."    # full non-interactive
vc-x1 push main --dry-run                           # preview
vc-x1 push main --from commit-app                   # resume at specific stage
vc-x1 push --status                                 # show saved state
```

Stage machine (runs top-to-bottom; each stage's success persists
to `.vc-x1/push-state.toml` so interrupts resume mid-flow):

| Stage | What it does |
|-------|--------------|
| `preflight` | `vc-x1 sync --check`, `cargo fmt`, `cargo clippy -D warnings`, `cargo test` |
| `review` | Print `jj diff --stat` for both repos; prompt `[y/N]` (first approval gate) |
| `message` | Compose title+body from `--title`/`--body`, persisted state, or `$EDITOR` template; second approval gate |
| `commit-app` | `jj commit` app repo with ochid trailer pointing at `.claude` |
| `commit-claude` | `jj commit` `.claude` with ochid trailer pointing at app (skipped if `.claude` is clean) |
| `bookmark-both` | `jj bookmark set <bookmark> -r @- -R .` and `-R .claude` |
| `push-app` | `jj git push --bookmark <bookmark> -R .` |
| `finalize-claude` | `vc-x1 finalize --repo .claude --squash --push <bookmark> --delay 10 --detach` |

Failures in `commit-app` / `commit-claude` / `bookmark-both` roll
both repos back via `jj op restore` to the snapshot recorded at
the start of `commit-app`. Past `push-app` the remote boundary is
crossed and recovery is forward-only (see "Late changes after
push" in CLAUDE.md).

| Flag | Description |
|------|-------------|
| `[BOOKMARK]` | Bookmark to advance; positional form of `--bookmark` |
| `--bookmark <NAME>` | Same as positional (mutually exclusive) |
| `-y, --yes` | Auto-approve both gates (non-interactive use) |
| `--title <STR>` / `--body <STR>` | Skip `$EDITOR` for the message stage |
| `--dry-run` | Print what would run, no side effects, no state written |
| `--step` | Pause after every stage for an extra continue-prompt |
| `--from <STAGE>` | Jump to a specific stage (advanced / resume) |
| `--status` | Print saved state's current stage and exit |
| `--restart` | Clear saved state; start from stage 1 |
| `--recheck` | Re-run preflight on resume (default: skip if last succeeded) |
| `--no-finalize` | Stop before `finalize-claude` (run it manually) |

State file path is configurable via `.vc-config.toml`'s `[push]`
section:

```toml
[push]
state-dir = ".vc-x1"          # default
state-file = "push-state.toml"  # default
```

`push` warns (non-fatal) when the configured state dir isn't
matched in `.gitignore`.

See [Add push subcommand (0.37.0)](./notes/chores-05.md#add-push-subcommand-0370)
for the full design and [per-step record](./notes/chores-05.md#per-step-record)
for what each `0.37.0-N` dev step shipped.

### Testing push + finalize

Always test against a throwaway fixture, never the live workspace.
Scaffold one with `vc-x1 init --repo-local <PARENT>` (no GitHub, no
network), then run the complete push + finalize flow end-to-end.
The code repo uses plain `jj git push`; the session repo uses
`vc-x1 finalize` to squash trailing writes and push in one shot.

`--repo-local` lays out:
```
<PARENT>/
  remote-code.git/     bare git remote for code repo
  remote-claude.git/   bare git remote for .claude session repo
  <NAME>/              code repo (jj colocated, main tracks origin)
    .vc-config.toml    path="/",       other-repo=".claude"
    .gitignore         /.claude /.git /.jj /target /.vc-x1
    .claude/           session repo (jj colocated, main tracks origin)
      .vc-config.toml  path="/.claude", other-repo=".."
      .gitignore       .git .jj
```

```bash
parent=$(mktemp -u /tmp/vc-x1-test-XXXXXX)
vc-x1 init work --repo-local "$parent"
work="$parent/work"
session="$parent/work/.claude"

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
rm -rf "$parent"
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

Bot-following workflow, commit conventions, and code style are
canonical in [CLAUDE.md](CLAUDE.md):

- [Versioning during development](CLAUDE.md#versioning) — `-N`
  pre-release suffix convention (single-step vs multi-step).
- [Commit message style](CLAUDE.md#commit-message-style).
- [Commit-Push-Finalize Flow](CLAUDE.md#commit-push-finalize-flow) —
  two-checkpoint per-step discipline.
- [Code Conventions](CLAUDE.md#code-conventions) — doc comments on
  every file / fn / method, `// OK: …` on `unwrap*` calls,
  ask-on-ambiguity, stuck detection.
- [Pre-commit checklist](CLAUDE.md#pre-commit-checklist).

Task tracking and release details live under [notes/](notes/):
near-term tasks in [notes/todo.md](notes/todo.md), per-release
details in `notes/chores-*.md`, and notes-specific formatting
rules in [notes/README.md](notes/README.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
