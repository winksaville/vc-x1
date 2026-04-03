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
