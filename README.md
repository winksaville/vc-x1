# vc-x1

- [Overview](#vc-x1)
- [Architecture](ARCHITECTURE.md)
- [Terminology](#terminology)
- [Build and install](#build-and-install)
- [Usage](#usage)
  - [Revision shortcuts](#revision-shortcuts)
  - [Shell completion](#shell-completion)
  - [validate-desc](#validate-desc)
  - [fix-desc](#fix-desc)
  - [validate-todo](#validate-todo)
  - [fix-todo](#fix-todo)
  - [validate-bot](#validate-bot)
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
  joined by `.`); not a "path".
- a datum in a transcript's `--fields` inventory is a **field**, e.g. `message.content[].type`, its
  levels joined by `.`, with `[]` marking array elements.

## Build and install

A plain Rust crate: `cargo build`, `cargo test`, and

```
cargo install --path . --locked
```

installs one binary named by the manifest's `[package].name`. That name is deliberate, the
**single-name convention** (adopted 0.78.3; design record in
`notes/chores/chores-16.md`):

- On `main` the name is `vc-x1`: installing from a main-positioned tree gives you the stable
  binary, and that is the whole promotion ceremony.
- A development line renames `[package].name` to its own name (e.g. `vc-x1-dev`,
  `vc-x1-<topic>`) at cycle open and back to `vc-x1` at close-out, next to the version bump
  those steps already make. Each line's install then produces its own binary, and concurrent
  lines coexist on `PATH` under their own names.
- The guard is mechanical, not memory: `build.rs` refuses to build a tree whose version
  carries a dev suffix (`0.79.1-2`) while the manifest still says `vc-x1`. A build script
  runs on **every** cargo verb, `cargo install` included (a test would not: `cargo install`
  never runs tests), so a forgotten rename fails before any binary exists to clobber your
  stable one. The full matrix (verified 2026-08-06):

  | `[package].name` | `version` | any cargo build/test/install |
  |---|---|---|
  | `vc-x1` | `0.78.3-1` | **fails**: a dev rung wearing the stable name |
  | `vc-x1-exper1` | `0.78.3-1` | passes: a normal dev rung |
  | `vc-x1-exper1` | `0.78.3` | passes: installs `vc-x1-exper1`, stable untouched (a merge-landing close-out renames in its merge commit) |
  | `vc-x1` | `0.78.3` | passes: a close-out or stable tree |

  The failing quadrant reads (via `cargo::error` directives: terse fact-then-refusal lines,
  fenced by empty directives, rather than a build-script panic dump; the remedy is the rename
  at cycle open):

  ```
  error: vc-x1@0.78.3-1: 
  error: vc-x1@0.78.3-1: A suffixed version is used with the stable package name, `vc-x1`
  error: vc-x1@0.78.3-1: so refusing to build `vc-x1` as it is the stable binary name.
  error: vc-x1@0.78.3-1: 
  error: build script logged errors
  ```
- The binary banners the name it was invoked as (`argv[0]`), so a renamed manifest, a copy,
  or a symlink all self-report truthfully; the version in the banner identifies the exact
  commit either way.

vc-x1 links jj-lib and refuses to run when the installed `jj`'s version disagrees with it
(override per invocation with `--allow-jj-mismatch`; see `vc-x1 version`).

## Usage

```
vc-x1 list [-r REVISION] [-n COMMITS]  # List commits in a jj repo
vc-x1 desc [-r REVISION] [-n COMMITS]  # Show full description of a commit
vc-x1 chid [-r REVISION] [-n COMMITS]  # Print changeID(s) for a revision
vc-x1 show [-r REVISION] [-n COMMITS]  # Show commit details and diff summary
vc-x1 agent-session <FILE> [OPTS]        # Display a session transcript as a conversation
vc-x1 validate-desc [OPTS]                 # Validate commit descriptions
vc-x1 fix-desc [OPTS]                     # Fix commit descriptions (dry-run default)
vc-x1 validate-todo [FILE]                # Check todo-file entry numbering
vc-x1 fix-todo [FILE]                     # Renumber todo file (dry-run default)
vc-x1 validate-bot [OPTS]                 # Check the bot repo is published
vc-x1 config [OPTS]                       # Print / validate settable config keys
vc-x1 clone <REPO> [NAME] [OPTS]          # Clone a dual-repo project
vc-x1 init <TARGET> [OPTS]                # Create a new dual-repo project
vc-x1 symlink [TARGET] [OPTS]             # Create Claude Code project symlink
vc-x1 sync [OPTS]                          # Fetch + sync both repos to their remotes
vc-x1 squash-push [BOOKMARK] [OPTS]        # Squash @ into @-, advance a bookmark, push
vc-x1 push [BOOKMARK] [OPTS]               # Commit both repos, push work, squash-push bot
vc-x1 version                              # Report vc-x1, jj-lib, jj, and jj-data versions
vc-x1 --version                            # Print version
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

### Positional shorthand

REVISION and COMMITS can be given as positional arguments, omitting `-r` and `-n`:

```
vc-x1 list x                # just x (1 commit)
vc-x1 list x 5              # x.. 5 -> x + 4 ancestors (5 commits)
vc-x1 list x.. 5            # same as above
vc-x1 list ..x 3            # 2 descendants + x (3 commits)
vc-x1 list ..x.. 3          # 1 descendant, x, 1 ancestor (3 commits)
vc-x1 list ..x.. 1          # just x (1 commit)
```

**Defaults:** with no arguments, REVISION is `@` and COMMITS is 1 (just the working copy).

Named flags `-r`/`--revision`, `-n`/`--commits`, `-R`/`--repo`, and `-s`/`--scope` take precedence
over positional arguments.

### Multi-repo queries

`-s`/`--scope` selects workspace sides by role keyword; `work` is the work repo, `agent` is the
`.claude` repo, `work,agent` is both. The workspace root is found by walking up from cwd (the existing
`find_workspace_root` rule).

```
vc-x1 chid -s work,bot          # both sides of the workspace
vc-x1 chid -s work              # just the work repo
vc-x1 chid -s agent             # just the bot repo
vc-x1 list @.. 3 -s work,bot    # works with chid / desc / list / show
```

`-R`/`--repo` takes a single path. Used alone it points at one jj repo (workspace lookup is
skipped); combined with `-s` it overrides the workspace root for the role lookup:

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

$ vc-x1 list @.. 3 -s work,bot
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

### agent-session

Display a Claude Code session transcript (`.claude/<uuid>.jsonl`) as a readable conversation. The
transcript format is undocumented and evolves; parsing is tolerant: unknown entry types are skipped
and counted, malformed lines (e.g. a live session's truncated last line) warn to stderr and never
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
rendered; they're counted in the summary as "skipped".

Default items resolve git-style, most specific wins:

1. CLI `--<item>` / `--no-<item>` flags
2. workspace `.vc-config.toml`: `[agent-session].items`
3. user `~/.config/vc-x1/config.toml`: `[agent-session].items`
4. built-in: `headers,user,assistant,tool,summary`

```toml
# In ~/.config/vc-x1/config.toml (all your workspaces) or
# <workspace>/.vc-config.toml (this workspace, committed):
[agent-session]
items = "headers,user,assistant,tool,summary"
```

```
# Default conversation view
vc-x1 agent-session .claude/<uuid>.jsonl

# Everything: thinking, tool results, meta/system too
vc-x1 agent-session --all FILE

# Prompts only: what was asked, nothing else
vc-x1 agent-session --none --user FILE

# Default view minus tool calls and headers
vc-x1 agent-session --no-tool --no-headers FILE

# Slices by source JSONL line (same unit in every view)
vc-x1 agent-session --lines 40 FILE      # first 40 source lines
vc-x1 agent-session --lines -15 FILE     # last 15 source lines
vc-x1 agent-session --lines 100,20 FILE  # 20 lines from Index 100
vc-x1 agent-session --lines 100,-20 FILE # 20 lines ending at Index 100
vc-x1 agent-session --lines 0 FILE       # stats summary only
```

| Flag | Description |
|------|-------------|
| `--<item>` / `--no-<item>` | Add / remove one of the eight items (last one wins) |
| `--all` / `--none` | Base: every item on / off (aliases `--no-none` / `--no-all`) |
| `--lines SPEC` | Slice by source JSONL line, 0-based Index (`N` first, `-N` last, `I,C` from I, `I,-C` ending at I; `0` = summary only); the same unit in every view |
| `--result-lines N` | Max lines shown per tool result [default: 10]; `0` = unlimited. Resolves CLI > workspace `.vc-config.toml` `[agent-session].result-lines` > user config > built-in, same as `[agent-session].items` |

Cut points show an `... (N source lines skipped)` marker, and a sliced run's summary ends with
`--lines selected K of M source lines`; its stats describe only the slice. Timestamps are shown as
UTC (`Z`) exactly when the source timestamp carries it, observed always, but the format is
undocumented, so anything else would pass through verbatim.

**Alternate views.** The transcript format is undocumented and evolves, so agent-session doubles as a
schema explorer. The parser keeps every field Anthropic writes while the typed layer consumes a
known subset; the difference is the unexplored surface:

| Flag | View |
|------|------|
| `--fields` | Field inventory, grouped by each line's `type`: every field observed (e.g. `message.content[].type`, with `.` for nesting and `[]` for array elements) with its count, value kinds, and short samples |
| `--unknown` | Like `--fields`, but only fields the typed layer does not consume: the unmodeled / new surface |
| `--raw` | Pretty-printed source lines (unparseable lines pass through verbatim); no summary or markers |
| `--per-line` | With `--fields`/`--unknown` (implies `--fields`): one fields section per source line instead of aggregating |
| `--col-width N` | First column width in these views [default: 68]; longer field names overflow. Resolves CLI > workspace `.vc-config.toml` `[agent-session].col-width` > user config > built-in, same as `[agent-session].items` |

The default 68 aligns the type column for ~99% of observed field names; only a tail of
`snapshot.trackedFileBackups.<absolute path>.*` keys, whose embedded absolute paths can be
arbitrarily long, overflow.

`--lines` uses the same source-line unit here, e.g. `--fields --lines 0,1` inventories just the
first line. `--fields`/`--unknown` ignore item flags; `--raw` conflicts with them.

```
# What does this format actually contain today?
vc-x1 agent-session --fields FILE

# What don't we model yet / what did Anthropic add?
vc-x1 agent-session --unknown FILE

# Inspect one entry in full (source line 42)
vc-x1 agent-session --raw --lines 42,1 FILE

# Walk a region line by line, fields-table style
vc-x1 agent-session --per-line --lines 40,5 FILE
```

### validate-desc

Read-only scan of commit descriptions against the other repo. The other repo is read from
`.vc-config.toml` (`repos.agent`) by default, or overridden with `--other-repo`. Reports status per
commit: `ok` (valid ochid), `lost` (ochid: lost), `none` (ochid: none), `err` (issues found), or
`miss` (no ochid trailer).

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

Fix commit descriptions against the other repo. Default is dry-run; use `--no-dry-run` to write
changes. Reads other repo from `.vc-config.toml` by default.

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

### validate-todo

Check that the `## Todo` and `## Bugs` sections of a todo file are numbered `1..N` in document
order, with continuation-line indent matching each entry's number-prefix width. Read-only; exits
non-zero if any entry needs fixing.

```
# Check TODO.md (the default)
vc-x1 validate-todo

# Check a specific file
vc-x1 validate-todo path/to/todo.md
```

Each flagged entry prints its corrected first line and a `[line: ...]` tag: the entry's line number
and what is off (`was N`, `indent A -> B`). Use `fix-todo` to apply the fixes.

### fix-todo

Renumber a todo file's `## Todo` and `## Bugs` sections to `1..N` and normalize each entry's
continuation-line indent. Dry-run by default, printing each changed entry's corrected line so the
output is the result; `--no-dry-run` writes the file in place.

```
# Dry-run: print the corrected lines
vc-x1 fix-todo

# Renumber the file in place
vc-x1 fix-todo --no-dry-run

# Operate on a specific file
vc-x1 fix-todo path/to/todo.md
```

| Flag | Description |
|------|-------------|
| `--no-dry-run` | Write the renumbered file in place [default: dry-run] |

### validate-bot

Check the bot repo is in its expected at-rest state: `main` matching `main@origin`, with its remote
refs tracked. At rest the two always match, because the bookmark only moves inside a `push` /
`squash-push` run, which publishes it in the same invocation, so a mismatch means an earlier publish
was lost. Read-only and cheap (two `jj` lookups, no build steps); exits non-zero on any finding and
fixes nothing. Resolve with `vc-x1 squash-push -R <bot-repo>`.

```
# Check ./.claude (run from the project root)
vc-x1 validate-bot

# Explicit bot-repo path
vc-x1 validate-bot -R path/to/.claude
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

- `[TARGET]`: `work`, `agent`, `work,agent` (default), or an explicit config-file path. The keywords
  resolve against the surrounding workspace's `.vc-config.toml` files and filter to that side's
  keys; a path carries no side information, so it gets the whole schema. The user config
  (`~/.config/vc-x1/config.toml`) has no keyword and is reached only by passing its path.
- `--validate`: instead of printing, load the target's actual config file(s) and flag any key the
  schema doesn't recognize (a typo, a key in the wrong section, or an unknown key), plus, for
  keyword targets, the legacy-schema rejection and the resolved-agreement invariant of a dual
  workspace's `[repos]` registries. Exits non-zero if any problem is found. This is an opt-in strict
  check; a normal config load silently ignores unknown keys, for forward-compatibility.

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
vc-x1 config work

# Print the whole schema for one file (the user config has no
# keyword, reach it by path)
vc-x1 config ~/.config/vc-x1/config.toml

# Check both sides' config files: unknown keys, legacy-schema
# rejection, and the [repos] resolved-agreement invariant
vc-x1 config --validate

# Check one explicit file
vc-x1 config ../other/.vc-config.toml --validate
```

| Flag | Description |
|------|-------------|
| `[TARGET]` | `work`, `agent`, `work,agent`, or a config-file path [default: work,bot] |
| `--validate` | Check the target config file(s) instead of printing the schema |

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

The file is read in the same format as an instance config: the `toml` fences, concatenated in
document order, form the TOML, and build.rs shares the filter (`src/md_fence.rs`) with the
loader. Prose between fences is documentation and never reaches a parser.

References are derived, not written: build.rs joins the single `[vc-config] reference-base` (the
bare repo url) with `/blob/HEAD/vc-config.md#` and each key's heading anchor, so links follow the
repo's default branch and no branch name is baked in; the test suite fails if a key has no
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

### clone

Clone an existing dual-repo project. Runs `git clone --recursive` to get both repos, initializes
`jj` in each, and creates the Claude Code symlink.

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

Requires `jj` to be installed. The `.claude` bot repo is cloned automatically via `git submodule` if
the source project was created with `vc-x1 init`.

### init

Create a new dual-repo project: a work repo with a `.claude` bot repo as a git submodule. Both repos
are initialized with `git` and `jj`, configured with `.vc-config.toml`, and pushed to GitHub. The
bot repo is added as a submodule so `git clone --recursive` clones both.

```
# Create public project in current directory (GitHub via gh)
vc-x1 init my-project

# owner/name shorthand, path targets, or a parent directory
vc-x1 init myorg/my-project
vc-x1 init ~/projects/my-project

# Create private repos
vc-x1 init my-project --private

# Local bare remotes instead of GitHub (offline; used by fixtures)
vc-x1 init my-project --repo local=/path/to/parent

# Preview without executing
vc-x1 init my-project --dry-run

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
| `<TARGET>` | URL, owner/name shorthand, path, or bare NAME |
| `[NAME]` | Repo directory name override (URL / owner/name forms only) |
| `--account <NAME>` | Pick `[account.<a>]` from user config |
| `--repo <CAT[=VAL]>` | Repo target, e.g. `local=<PARENT>` for local bare remotes |
| `--por` | Plain single repo (no `.claude/` companion) |
| `--private` | Create private GitHub repos [default: public] |
| `--dry-run` | Show what would be done without executing |
| `--push-retries <N>` | Max push retries after repo creation [default: 5] |
| `--push-retry-delay <N>` | Seconds between push retries [default: 3] |
| `--use-template <CODE[,BOT]>` | Seed both repos from template dirs (see below) |
| `--config <none\|PATH>` | Override the canned `.vc-config.toml` write |

**`--use-template`**. Value is `CODE[,BOT]`. If `BOT` is omitted, defaults to the sibling directory
`<CODE>.claude` (file-name concat, not path join, since the two templates are not nested).
Non-hidden contents are copied recursively into each target; hidden entries (names starting with
`.`) are skipped since init creates the repo's own hidden files (`.vc-config.toml`, `.gitignore`,
`.git/`, `.jj/`). If either template has a `README.md` at its root, its first line is rewritten to
`# <repo-name>`: `<name>` for the work repo and `<name>.claude` for the bot repo. For local
verification without hitting GitHub, combine `--use-template` with `--repo local=<PARENT>`.

Requires `gh` (authenticated) and `jj` to be installed (`gh` is skipped under `--repo local=...`).

### symlink

Create or verify the Claude Code project symlink. Claude Code stores session data in
`~/.claude/projects/<encoded-path>/`. This command creates a symlink from that location to the local
`.claude` directory.

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

Fetch and sync a set of repos to their remotes in one atomic operation: fetch, converge the
bookmark, reposition `@`. Repo set defaults to the dual-repo workspace pair (`.` and `.claude`);
narrow it with `-s` / `--scope`, or point at a different workspace root or single repo with `-R` /
`--repo`. There are no modes: verify-then-act happens inside a single invocation against one fetch
snapshot (a separate check-then-apply pair of runs would race the remote).

Per repo, `sync` classifies the local bookmark against its remote:

| State | Meaning | Action |
|------|---------|--------|
| up-to-date | local == remote | none |
| behind | local is ancestor of remote | `jj bookmark set <b> -r <b>@<remote>` |
| ahead | remote is ancestor of local | none (push is a separate step) |
| diverged | neither is ancestor | `jj rebase -b <local-head> -d <b>@<remote>` |
| no remote | bookmark has no `@<remote>` counterpart | none, skip |

`--bookmark` names a **work-repo** bookmark only: the bot repo is a linear journal on `main` by
design, so its side of every step (tracking preflight, classify, act, reposition) always uses `main`
regardless of the flag.

After the bookmark action above, `sync` repositions `@` onto the freshly-synced bookmark as a final
pass, run after every repo syncs cleanly. The rule differs by repo (`@-` is the parent of `@`; `<b>`
is the synced `--bookmark`):

- **Work repo** (the workspace root):
  - `@` is clean (empty) and `<b>` sits ahead of `@-` on the same line -> `jj new <b>` starts a
    fresh `@` on the new tip (the old empty `@` is auto-abandoned).
  - `@` has changes -> `sync` asks before moving it; pass `--rebase` to answer yes up front (`jj
    rebase -b @ -d <b>`). Declining, or not being a TTY with no `--rebase`, leaves `@` in place and
    says so.
  - `@` already sits on `<b>`, or `<b>` isn't on `@-`'s line (diverged / `@` ahead) -> `@` is left
    untouched, with a note why.
- **Bot repo** (`.claude`): no-op when `@-` is already the `main` tip, so `@` keeps its change id
  and any live session writes stay in the working copy. When `main` moved, `jj new main` starts a
  fresh empty `@` on the new tip; the prior `@` (e.g. `/exit`'s trailing session writes) is
  preserved as a sibling head. If `@-` isn't on `main`, `sync` errors rather than strand it.

On any failure during fetch/classify/act/reposition (conflicted rebase, subprocess error, anything)
`sync` **stops where the failing step stopped**. Nothing is auto-reverted, so the state can be
inspected as-is (jj's operation log holds everything; nothing is lost by stopping). Before acting,
sync captures each repo's pre-sync `jj op` id in memory; the failure report lists every repo's op
id and the undo is explicit, per repo: `jj op restore <op> -R <repo>`. Nothing persists across
invocations: a snapshot that outlives its run is a stale target waiting to rewind unrelated work.
That is also why the former `revert` subcommand was removed at 0.78.3: it restored the persisted
snapshot with no guard against discarding operations recorded after the failed sync. A safer
revert would derive its target from the op log and refuse when unrelated operations sit in
between; until such a design exists, `jj op log` + `jj op restore` is the recovery.

```
vc-x1 sync                            # workspace-default scope
vc-x1 sync --rebase                   # rebase a dirty @ onto the bookmark without asking
vc-x1 sync --scope=work               # only the work repo
vc-x1 sync --scope=agent                # only the bot repo
vc-x1 sync --scope=work,bot           # both (explicit form of the dual default)
vc-x1 sync -R ../other                # sync ../other as a single repo
vc-x1 sync -R ../other --scope=work,bot   # ../other as workspace root
```

**Repo set resolution.** `-R` and `--scope` compose:

1. Neither: workspace-default scope, `work,agent` if `repos.agent` is non-empty, else `work`. POR (no
   `.vc-config.toml`) -> `work` resolved to cwd.
2. `-R PATH` alone: sync just the repo at `PATH`.
3. `--scope=work|bot|work,bot` alone: workspace roles, resolved via the discovered workspace root's
   `.vc-config.toml` (`work` -> root, `agent` -> the root-joined `agent` path).
4. `-R PATH --scope=ROLES`: roles resolved against `PATH` as the workspace root.

Scope is cwd-portable: from `.claude/`, `vc-x1 sync` walks up to the workspace root and resolves
repos by absolute path.

| Flag | Description |
|------|-------------|
| `-R, --repo <PATH>` | Workspace root, or a single repo to sync alone. Composes with `--scope` |
| `--scope <SCOPE>` | `work|bot|work,bot`: workspace roles to sync. Composes with `-R` |
| `-q, --quiet` | Suppress all output; exit code signals result (for scripts) |
| `--bookmark <NAME>` | Bookmark to sync in the work repo (bot repo always syncs `main`) [default: main] |
| `--remote <NAME>` | Remote to sync against [default: origin] |
| `--rebase` | Rebase a non-empty `@` onto the synced bookmark without prompting (work repo only) |

**Output shape.** Sync collapses output based on what it finds:

- **All up-to-date**: one-line summary, `sync: N repos are up to date, nothing to sync`. Nothing
  else (no-op reposition lines are debug-level). Makes "sprinkle sync everywhere" genuinely cheap.
  Scope is bookmark-vs-remote tracking; `@` may have uncommitted working-copy changes, and sync
  intentionally doesn't speak to that (use `jj st` for working-copy state).
- **Action needed** (`behind` / `diverged`): per-repo fetch + state lines, then the actions run.
- **`--quiet`**: no output at any level; exit code is the only signal. Intended for scripts that
  just need success/failure.

**Note on the `behind` case.** jj's `git fetch` already fast-forwards a tracked local bookmark when
it's a strict ancestor of the incoming remote, so in the common case `sync` reports `up-to-date`
rather than `behind`. The `behind` branch covers untracked bookmarks and edge configs where
auto-advance is disabled.

### squash-push

Squash `@` into `@-`, advance a bookmark, and push, capturing a repo's trailing working-copy writes
and publish them in one step. Rewriting an already-pushed commit this way is a deliberate
published-history rewrite, so the push is a forced update.

The primary use case is folding the bot repo's session tail: session data keeps landing in `@` after
the last commit (including the record of the push itself), and only the user, acting after the bot
goes quiet, can capture all of it. Zero ceremony by design:

```
# In .claude: squash @ -> @-, advance main, push
vc-x1 squash-push -R .claude

# Bare invocation: same, in the current directory's repo
vc-x1 squash-push

# Custom bookmark and squash pair
vc-x1 squash-push feature -R . --squash @,@--
```

| Flag | Description |
|------|-------------|
| `[BOOKMARK]` | Bookmark to advance and push [default: main] |
| `-R, --repo <PATH>` | Path to jj repo [default: .] |
| `--squash [<SOURCE,TARGET>]` | Squash pair [default: @,@-] |

Behavior notes:

- Runs fully in-process, so a failure is a visible non-zero exit. (Replaces the `finalize`
  subcommand, whose detached background child could be killed silently at command exit.)
- With an empty `@` and the bookmark already at the remote it reports "already sync'd" and exits 0;
  with an empty `@` but the remote behind, it skips the squash and still pushes.
- If the bookmark doesn't match `BOOKMARK@origin` at start (an earlier publish was lost; see
  [validate-bot](#validate-bot)), it says so and proceeds: publishing is its job.
- Preflight refuses bad states before rewriting anything: unresolvable squash revsets, an
  ochid-dropping squash (see [Testing the ochid-trailer guard](#testing-the-ochid-trailer-guard)),
  conflicts, a missing / untracked / non-forward bookmark, an undescribed push target.

See [inline session push + squash-push
(0.69.0)](./notes/chores/chores-13.md#feat-inline-session-push--squash-push-0690) for design
details.

### push

Commit both repos, push the work repo's BOOKMARK, and squash-push the bot repo's `main`: one command
with two interactive approval gates. Replaces the old multi-step manual choreography (`jj commit` ×
2 -> `jj bookmark set` × 2 -> `jj git push` -> squash-push) with a single invocation.

```bash
vc-x1 push main                                     # interactive
vc-x1 push main --title "..." --body "..."          # skip $EDITOR
vc-x1 push main --yes --title "..." --body "..."    # full non-interactive
vc-x1 push main --dry-run                           # preview
```

Stages, top to bottom:

| Stage | What it does |
|-------|--------------|
| `review` | Print `jj diff --stat` for both repos; prompt `[y/N]` (first approval gate) |
| `message` | Compose title+body from `--title`/`--body` or an `$EDITOR` template; second approval gate. Skipped when neither repo has pending changes, since a publish-only run needs no message |
| `commit-work` | `jj commit` work repo with ochid trailer pointing at `.claude` (skipped when `@` is empty) |
| `commit-bot` | `jj commit` `.claude` with ochid trailer pointing at the work repo (skipped if `.claude` is clean) |
| `bookmark-set` | `jj bookmark set <bookmark> -r @- -R .` and `jj bookmark set main -r @- -R .claude` |
| `push-work` | Verify the bookmark's remote refs are tracked, then `jj git push --bookmark <bookmark> -R .` |
| `squash-push-bot` | In-process squash of `.claude`'s trailing session writes + push `main` (see [squash-push](#squash-push)) |

**Rerunning is always safe.** There is no saved state and no resume: every stage checks its own
precondition and does nothing when its work is already done, so re-running after a failure is simply
running. A recorded position would describe a world that may have changed for reasons push cannot
know. If a run fails, push stops and reports. Please fix and try again.

Failures in `commit-work` / `commit-bot` / `bookmark-set` roll both repos back via `jj op restore`
to a snapshot taken moments earlier in the same process. Once `push-work` succeeds the work is
published: from there a change is either a new commit appended on top by the next push, or an amend
of what was pushed, where [`squash-push`](#squash-push) folds the working copy into the last commit
and force-updates the remote. See [Recovery](./notes/cycle-protocol.md#recovery).

| Flag | Description |
|------|-------------|
| `[BOOKMARK]` | Work-repo bookmark to advance (the bot repo always advances `main`); positional form of `--bookmark` |
| `--bookmark <NAME>` | Same as positional (mutually exclusive) |
| `-y, --yes` | Auto-approve both gates (non-interactive use) |
| `--title <STR>` / `--body <STR>` | Skip `$EDITOR` for the message stage |
| `--dry-run` | Print what would run, no side effects |
| `--step` | Pause after every stage for an extra continue-prompt |
| `--no-squash-push` | Stop before `squash-push-bot` (run it manually) |

See [Add push subcommand (0.37.0)](./notes/chores/chores-05.md#add-push-subcommand-0370) for the
full design and [per-step record](./notes/chores/chores-05.md#per-step-record) for what each
`0.37.0-N` dev step shipped.

### Testing push + squash-push

Always test against a throwaway fixture, never the live workspace. Scaffold one with `vc-x1 init
<TARGET> --repo local=<PARENT>` (no GitHub, no network), then run the complete flow end-to-end. The
work repo uses plain `jj git push`; the bot repo uses `vc-x1 squash-push` to squash trailing writes
and push in one shot.

`--repo local=<PARENT>` lays out:
```
<PARENT>/
  remote-work.git/     bare git remote for the work repo
  remote-bot.git/   bare git remote for the .claude bot repo
  <NAME>/              work repo (jj colocated, main tracks origin)
    .vc-config.md      work=".", agent=".claude"
    .gitignore         /.claude /.git /.jj /target
    .claude/           bot repo (jj colocated, main tracks origin)
      .vc-config.md    work="..", agent="."
      .gitignore       .git .jj
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
vc-x1 squash-push -R "$bot"

# 3. cleanup when done (init also created a symlink under
#    ~/.claude/projects/ pointing at the fixture, remove it too)
rm -rf "$parent"
```

**Why `jj git push` for the work repo but `squash-push` for `.claude`?** The work repo's workflow is
a plain dev commit on `@-` that we push directly. The bot repo mirrors the bot's runtime pattern:
session writes land in `@` (above the last committed commit), and squash-push folds those trailing
writes into that commit just before pushing, so one atomic state goes upstream.

The run is fully in-process and synchronous, so what you see is the whole flow:
```
$ vc-x1 squash-push -R "$bot"
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
vc-x1 squash-push -R "$bot"

# example 2, normal case: clear the description; the squash
# proceeds and the run pushes
jj describe -R "$bot" -m ''
vc-x1 squash-push -R "$bot"

# cleanup (also remove the fixture's ~/.claude/projects symlink)
rm -rf "$parent"
```

Output captured from a real run; regenerate these transcripts with
[`support/gen-exmpl-1-3.sh`](support/README.md#gen-example-1---3-output). Example 1 is refused with
exit 1:

```
$ vc-x1 squash-push -R "$bot"
error: refusing squash @ -> @-: the squash would drop ochid: trailers
the destination's message lacks:
  /abc123abc123
merge the messages by hand (`jj describe @- -R /tmp/vc-x1-guard-hmevVM/work/.claude`) or clear
the source's description, then retry
```

Example 2 with the description cleared squashes and pushes, exit 0:

```
$ vc-x1 squash-push -R "$bot"
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

Paths always start with `/` (the workspace root, i.e. vc-x1). Each repo has a `.vc-config.toml` that
identifies its location within the workspace, so tools can resolve these paths locally.

For full details see:
- [Git trailer convention](./notes/chores/chores-01.md#git-trailer-convention)
  - [ochid (Other Change ID)](./notes/chores/chores-01.md#ochid-other-change-id)
  - [ChangeID path syntax](./notes/chores/chores-01.md#changeid-path-syntax)
  - [.vc-config.toml](./notes/chores/chores-01.md#vc-configtoml)

## jj Tips for Git Users

Coming from git, start with these; one home per fact, so this section points rather than
repeats:

- [agent-data/jj.md](agent-data/jj.md): the project's quick reference (command basics, the
  working-copy `@` model, revset addressing) plus the dual-repo linking rules. Pinned and
  human-readable; `agent-data/` holds the universal rule files, not agent-only material.
- The [jj docs](https://docs.jj-vcs.dev/latest/) and
  [Steve Klabnik's Jujutsu tutorial](https://steveklabnik.github.io/jujutsu-tutorial/): the
  maintained tutorials; generic jj pedagogy lives upstream, not in this repo.
- `jj-tips.md` in the template repository (path recorded in custom.md): the family's worked
  tutorial with terminal transcripts, e.g. why `jj log` shows fewer commits than `gitk --all`
  and how force-pushes look in jj.

## Testing

The repo ships two flavors of tests:

- **In-process tests**: `#[cfg(test)] mod tests { ... }` blocks inside `src/*.rs`. These call
  library code directly (no subprocess spawn) and run fastest. The dual / POR fixture tests under
  `src/init.rs::tests` build throwaway workspaces by invoking `init::init` as a function.
- **CLI subprocess integration tests**: files under `tests/`. These spawn the `vc-x1` binary that
  Cargo built; its absolute path lives in the `CARGO_BIN_EXE_vc-x1` env var that Cargo sets at
  compile time, which `env!("CARGO_BIN_EXE_vc-x1")` reads and bakes into the test binary as a string
  literal. Argument parsing, exit codes, and stdout/stderr are all exercised end-to-end. Each
  subprocess gets a `HOME` override so the user's real `~/.config/vc-x1/` can't leak in or get
  clobbered.

Run everything with:

```bash
cargo test                 # unit + integration
cargo test --bins          # binary unit tests only (in-process)
cargo test --test cli_init # one integration test crate
```

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

Fixture directories are named `vc-x1-test-<tag>-<ts>-<n>` (in-process) and
`vc-x1-cli-test-<tag>-<ts>-<n>` (CLI). RAII drop removes them on test exit; SIGKILLs / panics in
`Drop` can leak, so search and clean manually:

```bash
find "${VC_X1_TEST_TMPDIR:-${TMPDIR:-/tmp}}" -maxdepth 1 \
     -name 'vc-x1-*test*' -mtime +1 -exec rm -rf {} +
```

A future enhancement (tracked in `TODO.md` at the repo root) extends the priority chain through
`~/.config/vc-x1/config.toml` and project-local `.vc-config.toml`.

### Preserving fixtures for debugging

When a test fails and you want to inspect the workspace it built, set `$VC_X1_TEST_KEEP`. The RAII
`Drop` impls (`Fixture`, `FixturePor`, `CliFixture`) skip `remove_dir_all` and print the preserved
path on stderr:

```bash
VC_X1_TEST_KEEP=1 cargo test -- --nocapture 2>&1 | grep TEST_KEEP
```

Two shell gotchas worth remembering:

- The preservation message goes to **stderr**, so a plain `cargo test | grep ...` won't see it; use
  `2>&1 |` or write the full output to a file.
- `--nocapture` is needed to bypass libtest's stdout/stderr capture; without it, the messages get
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
chatter into every real CLI run's stdout. Remove it before committing; `debug!` is the form that
stays.

Gotchas, in the order people hit them:

- `CliLogger::new(...)` alone does nothing: it only builds the value. `init` is what registers the
  global logger and raises `log::max_level`; without both, `debug!` stays a no-op.
- `--nocapture` is required, same as the fixture-preserving recipe above: debug lines go to stderr
  and libtest swallows them otherwise.
- `RUST_LOG` has no effect: `CliLogger` never reads env vars (that is env_logger's convention, not
  this logger's).
- Remove the `init` line when done. The logger is process-global, so left in place it makes every
  test in the binary chatty, and a second test calling `init` would panic: the global logger can be
  set only once per process.

## Support

Helper scripts for maintaining this repo's docs and examples live in [`support/`](support/); see
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
reference, not a status doc; most of what's there is forward-looking, not yet implemented.

## Contributing

The tool's internal structure (module map, the CLI-args / ops-`Context`+`Params` split, the
subcommand model, and the two in-flight migrations) is described in
[ARCHITECTURE.md](ARCHITECTURE.md). Start there to orient.

The bot-facing cycle workflow (cycles for Preparation / Work / Close-out, per-commit flow with the
cargo cycle `fmt` / `clippy` / `test` / `install`, commit description shape, ochid trailers,
pushing) lives in [`notes/cycle-protocol.md`](notes/cycle-protocol.md).

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

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
