# Chores-03

## Add `fn claude-symlink` (0.27.0)

Port claude-symlink from shell script below to a fn so
it can be used with a "new" command which will create the dual repos.
The base fn should probably not do any I/O it should return
appropriate error information and other `fn` so the caller can
handle errors as needed.

### Implementation

Added `src/symlink.rs` with a pure-logic / I/O separation:

- `encode_path()` — encodes a path the Claude Code way (`/` and `.` → `-`)
- `compute_plan()` — pure function, returns a `SymlinkPlan` with action
  (`Create`, `Replace`, `AlreadyCorrect`, or error) without touching the filesystem
- `probe_symlink()` — reads what exists at a path (nothing, file, or symlink)
- `execute_plan()` — creates/replaces the symlink per the plan
- `symlink()` — subcommand handler with interactive prompt for replacement

CLI: `vc-x1 symlink [TARGET] [--symlink-dir PATH] [-l] [-y]`
```
#!/bin/bash
# claude-symlink.sh - Create a symlink for Claude Code project directories
# Authors: wink@saville.com and Claude Opus 4.5

set -euo pipefail


usage() {
    echo "Usage: $(basename "$0") [-l] <target> [symlink-dir]" >&2
    echo "  -l          - List contents of symlinked directory after creation" >&2
    echo "  target      - Path to link to (e.g., ./.claude)" >&2
    echo "  symlink-dir - Directory for symlink (default: \$HOME/.claude/projects)" >&2
    exit 1
}

list_contents=false

while getopts ":l" opt; do
    case $opt in
        l) list_contents=true ;;
        *) usage ;;
    esac
done
shift $((OPTIND - 1))

if [[ $# -lt 1 ]]; then
    usage
fi

target="$1"
symlink_dir="${2:-$HOME/.claude/projects}"

# Create target directory if it doesn't exist
if [[ ! -e "$target" ]]; then
    mkdir -p "$target"
    echo "Created target directory: $target"
fi

# Resolve target to absolute path
abs_target="$(cd "$(dirname "$target")" && pwd)/$(basename "$target")"

# Get current directory and convert to dash-separated name
# Claude Code encodes paths by replacing / and . with -
cwd="$(pwd)"
symlink_name="${cwd//\//-}"
symlink_name="${symlink_name//./-}"

# Ensure symlink directory exists
mkdir -p "$symlink_dir"

symlink_path="$symlink_dir/$symlink_name"

# Check if something already exists at symlink path
if [[ -e "$symlink_path" || -L "$symlink_path" ]]; then
    if [[ -L "$symlink_path" ]]; then
        current_target="$(readlink "$symlink_path")"
        echo "Existing symlink: $symlink_path -> $current_target"
        read -rp "Replace with new target? [y/N] " response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            echo "Aborted." >&2
            exit 1
        fi
    else
        echo "Error: '$symlink_path' exists and is not a symlink" >&2
        exit 1
    fi
fi

# Create the symlink
ln -sfn "$abs_target" "$symlink_path"

echo "Created: $symlink_path -> $abs_target"

if [[ "$list_contents" == true ]]; then
    echo ""
    echo "Contents of $symlink_path:"
    ls -la "$symlink_path"/
fi
```

## Add `init` command (0.28.0)

This command will create the dual repos use by vc-x1 to hold the code
and the bot conversation. It will create a vc-config.toml file in both
repos and then use the `fn claude-symlink`. It should probably will use
`gh` commands, but a library is also an option, to create the repos
and then use `jj git init` for `jj` support.

### Implementation

Added `src/init.rs` — orchestrates dual-repo creation via `git`, `jj`, and `gh` CLI.

CLI: `vc-x1 init <NAME> [--owner OWNER] [--dir PATH] [--private] [--dry-run]`

Flow:
1. Preflight: verify `gh` auth, `jj` installed, no local/remote conflicts
2. Create both repos locally with `git init` + `jj git init --colocate`
3. Write `.vc-config.toml` and `.gitignore` to both
4. Commit `.claude` repo ("Initial commit" with ochid to code repo)
5. Create `<name>.claude` GitHub repo, push
6. Remove `.claude` contents, `git submodule add` re-clones it
7. Commit code repo ("Initial commit" with ochid + `.gitmodules`)
8. Create `<name>` GitHub repo, push
9. Create Claude Code symlink

Both repos end up with a single "Initial commit" with ochid cross-references.

Key lessons:
- jj doesn't understand git submodules. The code repo must use pure git
  commands (init, add, commit, push) throughout. `jj git init --colocate`
  and `jj bookmark set` are only run at the very end after everything is pushed.
- `.claude` is in `.gitignore`, so `git submodule add --force` is needed.
- After `git submodule add` re-clones `.claude`, jj must `bookmark track`
  the remote before it can push.
- GitHub repos may not be SSH-accessible immediately after `gh repo create`
  (propagation delay). Push uses retry with configurable attempts and delay.
- Session repo ochid initially uses `/none` placeholder (no code repo jj
  changeID yet), then gets fixed via `jj describe --ignore-immutable` after
  jj is initialized on the code repo at the end.

### Submodule + ochid circular dependency bug (0.29.0)

**Problem:** The original init flow created a circular dependency between
submodule commit pinning and ochid cross-references.

Git submodules pin to a specific commit hash. When `jj describe` or
`git commit --amend` rewrites a commit to add/fix an ochid trailer, the
git hash changes. This makes the submodule ref stale. Updating the
submodule ref in the code repo changes its tree, which changes its
hash and jj change ID, which makes `.claude`'s ochid stale. Fixing
`.claude`'s ochid changes its hash again... infinite loop.

The original flow hit this because it:
1. Committed `.claude` with a placeholder ochid (`/none`)
2. Added `.claude` as a submodule, committed and pushed the code repo
3. Amended `.claude` to fix the ochid → hash changed, submodule ref stale
4. Clone then saw two `.claude` commits with the same jj change ID
   (the pre-amend from the submodule ref, and the post-amend from
   origin/main) → jj reported "divergent"

**Solution:** Establish ochids before the submodule relationship exists.
Do all rewriting while the repos are independent, then add the submodule
as a separate (non-rewritten) commit.

Revised flow:
1. `git init` + `jj git init --colocate` on both repos
2. Write config files, `jj commit` both with placeholder ochids
3. Get both jj change IDs
4. `jj describe` both with correct ochids — hashes change but chids
   are stable, and no submodule link exists yet so nothing goes stale
5. Remove jj from both (`git clean -xdf` removes `.jj/`)
6. Create `.claude` GitHub repo, push
7. `git submodule add` in code repo — second commit with ochid
   pointing to `.claude`'s first commit
8. Create code GitHub repo, push
9. `jj git init --colocate` on both repos (final)
10. Create Claude Code symlink

Trade-off: the code repo ends up with 2 commits (initial + submodule
add) while `.claude` has 1. Squashing them would change the code repo's
chid, making `.claude`'s ochid stale — the same circular problem.
This is acceptable; repos won't always have 1:1 commit counts, and the
viewer needs to handle that anyway.

## Add `clone` command (0.29.0)

This command will clone the dual repos, probably using `gh`, or library,
with our dual repo system plus `fn claude-symlink` to be sure a symlink
exists. And then `jj git init` in each repo for `jj` support.

### Implementation

Added `src/clone.rs` — thin wrapper for cloning dual-repo projects.

CLI: `vc-x1 clone <REPO> [--dir PATH] [--dry-run] [--verbose]`

Where `<REPO>` can be:
- `owner/name` (GitHub shorthand, resolved to `git@github.com:owner/name.git`)
- Full SSH URL (`git@github.com:owner/name.git`)
- Full HTTPS URL (`https://github.com/owner/name.git`)

Flow:
1. Preflight: check target dir doesn't exist, jj is installed
2. `git clone --recursive` — clones code repo + `.claude` submodule
3. `jj git init --colocate` in code repo
4. `jj git init --colocate` in `.claude` (if submodule exists)
5. Create Claude Code symlink via `symlink::compute_plan` / `execute_plan`

Helper functions:
- `derive_name()` — extracts project name from any repo format
- `resolve_url()` — converts `owner/name` shorthand to SSH URL

### Remaining dev steps

**0.29.0-dev1** (done): Add clone command, fix init submodule/ochid
circular dependency, add verbose command output to init.

**0.29.0-dev2**: Make clone functional end-to-end.
- Add `jj bookmark track main --remote=origin` in init step 10
  (jj hints this is needed for future pulls)
- Remove clone's `git checkout -B main origin/main` workaround
  (no longer needed since init produces correct submodule refs)
- Test clone against a repo created with the fixed init

**0.29.0-dev3**: Polish init and clone output.
- Gate init's stdout/stderr command output behind `--verbose,-v`
  (keep step headers and summary in normal mode)
- Add optional positional `NAME` parameter to clone
  (`vc-x1 clone owner/repo my-dir`, like `git clone`)

**0.29.0**: Final release.
- Remove `-devN` from version
- Move todo item to Done
- Update chores with final notes

## Design for fn-claude-symlink, new, and clone

### claude-symlink (fn)

Port the bash script to a reusable Rust function. The function should
avoid doing I/O directly — return results/errors so callers can handle
them. This is the foundation for both `new` and `clone`.

### new command

Minimal initial implementation — create two repos with only:
- `vc-config.toml`
- `.gitignore`
- `.jj/` and `.git/` (via `git init` + `jj git init --colocate`)
- `.gitmodules` in the code repo (`.claude` as a git submodule)

The `.claude` repo is a git submodule of the code repo so that
`git clone --recursive` can clone both at once.

#### Order of operations

1. Create both repos locally, `git init` + `jj git init --colocate`
2. Add `vc-config.toml` and `.gitignore` to both
3. Commit `.claude` repo ("Initial Commit" with ochid cross-reference)
4. Create `<name>.claude` GitHub repo via `gh`, push `.claude`
5. Clean out `.claude` contents (so `git submodule add` can repopulate)
6. `git submodule add <github-url-for-.claude>` in code repo
7. Commit code repo ("Initial Commit" with ochid, `.gitmodules` + submodule ref)
8. Create `<name>` GitHub repo via `gh`, push code repo
9. Run symlink logic

Both repos end up with a single "Initial Commit", both with ochid
cross-references. The submodule is present from the first commit of
the code repo.

### clone command

Thin wrapper: `git clone --recursive` + `jj git init --colocate` in
both repos + symlink setup. Most of the heavy lifting is in `new`;
`clone` is straightforward once the submodule relationship exists.

## Universal `--verbose` and `common::run()` refactor (0.30.0)

Standardized output convention across all commands:
- **stdout** (`println!`): user-facing progress and results
- **stderr** (`eprintln!`): diagnostic detail, only with `--verbose,-v`

Changes:
- Added `--verbose,-v` flag to all commands: `chid`, `desc`, `list`,
  `show` (via CommonArgs), `validate-desc`, `fix-desc`, `symlink`,
  `clone`, `init`
- Moved `fn run()` to `common.rs` — single implementation used by
  `init`, `clone`, and `fix-desc`. Verbose mode shows command line
  with cwd, stdout, and stderr; normal mode is silent unless failure.
- Replaced all `Command::new()` calls outside `common::run()` and
  `finalize.rs` to use `common::run()` instead: `jj_chid()`,
  `gh_whoami()`, `gh_repo_exists()`, preflight checks, `jj_describe()`
- Removed bold ANSI codes from `chid` output entirely
- Updated CLAUDE.md with two-checkpoint commit/push/finalize workflow

## Adopt `log` crate with per-module filtering (0.31.0)

### Background

Currently output uses three separate mechanisms:
- `println!` for user-facing progress (stdout)
- `eprintln!` for verbose diagnostic detail (stderr, behind `--verbose`)
- `finalize::log_msg` for file-based tracing (finalize only)

These should be unified under a single logging system.

### Crate choice

Evaluated several options:
- `tracing` — full-featured but heavier, async/span features unused
- `log` — standard facade, lightweight, what libraries expect
- `env_logger` — easy but env-var-only (`RUST_LOG=...`), poor CLI UX
- `log4rs`, `fern`, `slog` — specialized, more than needed

Decision: **`log`** as the facade with a thin custom subscriber that
routes based on CLI flags rather than env vars.

### Design

Level routing:
- `info!` and above → stdout (step headers, results) — always shown
- `debug!` → stderr — only with `--verbose`
- `trace!` → file — only with `--log <path>`
- `error!`/`warn!` → stderr — always shown

CLI interface:
- `-v` / `--verbose` — shorthand for "everything at debug on stderr"
- `--log-filter "vc_x1::init=debug"` — per-module runtime filtering
- `--log <path>` — write to file (replaces finalize's `log_msg`)
- `RUST_LOG` — fallback for power users, lowest priority

The `log` macros automatically tag with the module path (e.g.
`log::debug!("foo")` in `src/init.rs` → target `vc_x1::init`), so
per-module filtering comes for free.

Replaces: `println!`, `eprintln!` (verbose), `finalize::log_msg`.
Custom subscriber is ~50-80 lines.

## Per-line/per-thread runtime log points (future)

### Vision

Runtime-switchable instrumentation at individual call sites, addressable
by name from the CLI, with optional thread granularity. Think DTrace
probes or Linux tracepoints — each log point has a unique ID that can
be toggled independently at runtime.

```rust
log_point!(LP_INIT_STEP5, "ochid cross-reference: {}", chid);
```

Where `LP_INIT_STEP5` is a runtime-mutable flag, toggleable from CLI.
This goes beyond what `log`, `tracing`, or any standard Rust crate
provides. Would need a custom `LogPoint` system, possibly built on
top of `log`.

Not planned for a specific version — captured for future exploration.

## Windows symlink support

The symlink code uses `#[cfg(unix)]` which covers both Linux and macOS
(macOS is Unix). The `#[cfg(not(unix))]` path returns an error, which
effectively means Windows is unsupported.

Windows does support symlinks via `std::os::windows::fs::symlink_dir`,
but it requires elevated privileges or developer mode enabled. It's a
one-line addition when needed:

```rust
#[cfg(windows)]
std::os::windows::fs::symlink_dir(&self.abs_target, &self.symlink_path)?;
```

Also unclear whether Claude Code uses `~/.claude/projects/` symlinks
on Windows at all — it may handle session directories differently there.

Not planned until there's a Windows user for vc-x1.

## Finalize --squash refactor (0.31.0-dev5)

The original finalize had `--source @` and `--target @-` as required
defaults, `--bookmark` was required, and running with no real work to
do (e.g. single-commit repo) would error on "root commit is immutable".

### Design changes

1. **`--squash [SOURCE,TARGET]`** replaces `--source`/`--target`:
   - `--squash` (no value) → defaults to `@,@-` (the 90% case)
   - `--squash @,@--` → custom source/target pair
   - No `--squash` → no squash step
2. **`--bookmark` is optional** — omit it to skip bookmark update
3. **`--push` requires `--bookmark`** — can't push without a bookmark
4. **No args = print help** — `vc-x1 finalize` with nothing to do
   shows help instead of silently doing nothing
5. **Help text updated** — "Squash, set bookmark, and/or push a jj repo"

### Result

Every behavior is opt-in. The command composes:
- `vc-x1 finalize --bookmark main` → just set bookmark
- `vc-x1 finalize --squash --bookmark main --push` → full workflow
- `vc-x1 finalize --squash @,@-- --bookmark main` → custom squash

The typical session-end command becomes:
```
vc-x1 finalize --repo .claude --squash --bookmark main --delay 10 --detach --push
```

## Remove submodule from init/clone (0.31.1)

Git submodules made the dual-repo relationship visible, but in practice
they added complexity: the submodule entry updates on every `.claude`
commit, requiring special handling when pushing the code repo. Treating
the two repos as fully independent from git's perspective is simpler.

### init changes

Removed Step 8 (submodule add). Previously init would:
1. `remove_dir_all(.claude)`
2. `git submodule add --force <session_url> .claude`
3. Commit the submodule reference as a second commit

Now the code repo has a single initial commit. The `.claude` directory
is already in `.gitignore` so git ignores it. Steps renumbered from
11 to 10.

### clone changes

- Replaced `git clone --recursive` with plain `git clone` for the
  code repo
- Added a second clone step: derives the session repo URL by appending
  `.claude` before `.git` (e.g. `owner/repo.git` → `owner/repo.claude.git`)
  and clones it into `.claude/`
- Session clone failure is non-fatal (logged and skipped) for repos
  that have no session repo
- Steps renumbered from 4 to 5

### derive_session_url convention

- `git@github.com:owner/repo.git` → `git@github.com:owner/repo.claude.git`
- `https://github.com/owner/repo.git` → `https://github.com/owner/repo.claude.git`
- `https://github.com/owner/repo` → `https://github.com/owner/repo.claude`
