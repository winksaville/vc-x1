# vc-x1

- [Overview](#vc-x1)
- [Usage](#usage)
  - [Revision shortcuts](#revision-shortcuts)
  - [fix-ochid](#fix-ochid)
  - [finalize](#finalize)
  - [Testing finalize](#testing-finalize)
- [Cross-repo Linking with Git Trailers](#cross-repo-linking-with-git-trailers)
- [jj Tips for Git Users](#jj-tips-for-git-users)
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
vc-x1 fix-ochid --other-repo <PATH> [OPTS]  # Validate/fix ochid trailers
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

### fix-ochid

Validates and fixes ochid trailers across commit history. Checks three
properties: correct path prefix, correct changeID length, and that the
ID resolves in the other repo. Default is dry-run — use `--no-dry-run`
to write changes.

```
# Dry-run: show what would be fixed
vc-x1 fix-ochid -r @.. --other-repo .claude

# Actually fix
vc-x1 fix-ochid -r @.. --other-repo .claude --no-dry-run

# Add missing ochid trailers by matching title + timestamp (within 60s)
vc-x1 fix-ochid -r @.. --other-repo .claude --add-missing

# Use a fallback for IDs not found in other repo
vc-x1 fix-ochid -r @.. --other-repo .claude --fallback /.claude/lost
```

Key flags:

| Flag | Description |
|------|-------------|
| `--other-repo <PATH>` | Path to the counterpart repo (required) |
| `--no-dry-run` | Write fixes (default is dry-run) |
| `--add-missing` | Infer and add ochid for commits without one |
| `--fallback <VALUE>` | Replacement for IDs not found in other repo |
| `--id-len <N>` | Expected changeID length (default 12) |
| `--title <TEXT>` | Replace commit title at the same time |

The `--add-missing` flag matches commits by exact title and committer
timestamp within 60 seconds. It only adds the trailer when exactly one
match is found — ambiguous or zero matches are skipped.

### finalize

Solves the session repo coherence problem: when the bot commits `.claude`,
the act of committing generates more session data. `finalize` waits,
then squashes the trailing writes into the target commit.

By default finalize runs in the foreground. Use `--detach` to run in
the background (the bot uses this so the session can end immediately).

The bot's last action in a session:

```
vc-x1 finalize --repo .claude --bookmark main --delay 10 --detach --push
```

If there is non-written session data after a session ends (e.g.
finalize failed or was skipped), run it manually:

```
vc-x1 finalize --repo .claude --bookmark main --push
```

This runs in the foreground, squashes, advances the bookmark, and pushes.

See [finalize subcommand](./notes/chores-01.md#finalize-subcommand-for-session-repo-coherence)
for full option reference and design details.

### Testing finalize

Always test against a throwaway jj repo, never the live workspace.
Use `mktemp -d` for unique directories so results can't be confused
with a previous run:

```bash
# Create and init a temp repo
dir=$(mktemp -d /tmp/vc-x1-test-XXXXXX)
(cd "$dir" && jj git init)

# Foreground (default, blocks, logs to test dir)
vc-x1 finalize --repo "$dir" --bookmark main --log "$dir/finalize.log"
cat "$dir/finalize.log"

# Detached (returns immediately, child logs to test dir)
dir2=$(mktemp -d /tmp/vc-x1-test-XXXXXX)
(cd "$dir2" && jj git init)
vc-x1 finalize --detach --repo "$dir2" --bookmark main --log "$dir2/finalize.log"
cat "$dir2/finalize.log"
```

The log file shows timestamped (nanoseconds) entries with PIDs, covering
the full flow: `main` entry/exit, `finalize` entry/exit, `detach`
spawn, and `finalize_exec` in the child process.

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

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
