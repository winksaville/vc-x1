# vc-x1

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

## Usage

```
vc-x1 list [REV [COUNT]] [OPTIONS]   # List commits in a jj repo
vc-x1 desc [REV [COUNT]] [OPTIONS]   # Show full description of a commit
vc-x1 chid [REV [COUNT]] [OPTIONS]   # Print changeID(s) for a revision
vc-x1 finalize [OPTIONS]             # Squash working copy into target (daemonizes by default)
vc-x1 --version                       # Print version
vc-x1 --help                          # Print help
```

### Revision shortcuts

`list`, `desc`, and `chid` accept up to two positional arguments as
shorthand for `-r` (revision) and `-l` (limit).

**The `..` notation:** dots show where the list continues from the
revision:

- `x..` — x at top, ancestors below (older commits)
- `..x` — descendants above (newer commits), x at bottom
- `..x..` — both directions, x in the middle

COUNT is the number of commits on each dotted side (x is always
included and not counted). Bare `x COUNT` defaults to `x..` (ancestors),
the common case. Bare `x` without COUNT shows just that one commit.

```
vc-x1 list x                # just x (1 commit)
vc-x1 list x 5              # x.. 5 → x + 5 ancestors (6 commits, x at top)
vc-x1 list x.. 5            # x + 5 ancestors (6 commits, x at top)
vc-x1 list ..x 3            # 3 descendants + x (4 commits, x at bottom)
vc-x1 list ..x.. 3          # 3 descendants, x, 3 ancestors (7 commits)
vc-x1 list ..x.. 0          # just x (1 commit)
```

**Defaults:** with no arguments, REV is `@` and COUNT is 0 (just the
working copy).

Named flags `-r`/`--revision`, `-l`/`--limit`, and `-R`/`--repo` still
work and take precedence over positional arguments.

### finalize

Solves the session repo coherence problem: when the bot commits `.claude`,
the act of committing generates more session data. `finalize` daemonizes,
waits, then squashes the trailing writes into the target commit.

The last action in a session is always:

```
vc-x1 finalize --repo .claude --delay 5 --push
```

The command returns immediately and prints a status line:

```
Finalize is running (pid 12345, log `/tmp/vc-x1-finalize-1710412800000.log`)
```

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

# Foreground (blocks, logs to test dir)
vc-x1 finalize --foreground --repo "$dir" --log "$dir/finalize.log"
cat "$dir/finalize.log"

# Daemonized (returns immediately, child logs to test dir)
dir2=$(mktemp -d /tmp/vc-x1-test-XXXXXX)
(cd "$dir2" && jj git init)
vc-x1 finalize --repo "$dir2" --log "$dir2/finalize.log"
cat "$dir2/finalize.log"
```

The log file shows timestamped (nanoseconds) entries with PIDs, covering
the full flow: `main` entry/exit, `finalize` entry/exit, `daemonize`
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

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
