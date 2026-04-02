# vc-x1 new and clone subcommands design spec

## Prerequisites

- The `symlink` subcommand from `symlink-design.md` must be implemented first.
  Both `new` and `clone` call the symlink logic internally (with `yes: true`).
- `gh` CLI must be installed and authenticated (`gh auth status`). Check early,
  error with a clear message if missing.
- `jj` must be installed.

## Conventions

- Code repo: `<name>` (e.g. `my-project`)
- Session repo: `<name>.claude` (always `<name>` + `.claude` suffix)
- Submodule path inside code repo: `.claude`
- GitHub owner: inferred from `gh` (`gh api user --jq .login`), overridable
  with `--owner`
- SSH URLs: `git@github.com:<owner>/<repo>.git`

## `vc-x1 new <name>`

Creates a new dual-repo project from scratch, seeded from the template repos.

### CLI

```
vc-x1 new <name> [OPTIONS]
```

Arguments:
- `NAME` — project name (becomes both the directory name and GitHub repo name)

Options:
- `--owner <OWNER>` — GitHub user/org (default: current `gh` user)
- `--dir <PATH>` — parent directory to create project in (default: cwd)

### Steps

All commands below are described as what the tool executes. Abort on any
failure with a clear message about what step failed.

#### 1. Preflight checks

- Verify `gh` is installed and authenticated
- Verify `jj` is installed
- Verify `<dir>/<name>` does not already exist locally
- Verify neither `<owner>/<name>` nor `<owner>/<name>.claude` exist on GitHub
  (`gh repo view <owner>/<name>` — check exit code)

#### 2. Create the session repo (`<name>.claude`)

This must exist on GitHub before we can `git submodule add` it.

```sh
gh repo create <owner>/<name>.claude --public --clone
cd <name>.claude
```

Seed it from the template's session repo content. For the first pass, the
simplest approach:

```sh
gh repo clone winksaville/vc-template-x1.claude -- --depth 1 /tmp/<tmpdir>
# Copy all files except .git from tmpdir into <name>.claude
# rm -rf /tmp/<tmpdir>
```

Rewrite `.vc-config.toml` to fix the `workspace.path` to `/.claude` (should
already be correct from template, but verify/write it).

Do obvious string substitutions in copied files:
- In `.vc-config.toml`: no change expected (path is `/.claude`, other-repo
  is `..` or `/`)
- In any `CLAUDE.md` or `README.md`: replace `vc-template-x1` with `<name>`

Initial commit and push:

```sh
jj git init --colocate
jj describe -m "feat: Initial session repo for <name>"
jj bookmark create main -r @
jj git remote add origin git@github.com:<owner>/<name>.claude.git
jj bookmark track main --remote=origin
jj git push
```


#### 3. Create the code repo (`<name>`)

```sh
cd <dir>
gh repo create <owner>/<name> --public --clone
cd <name>
```

Seed from the template's code repo:

```sh
gh repo clone winksaville/vc-template-x1 -- --depth 1 /tmp/<tmpdir>
# Copy all files except .git, .gitmodules, and .claude from tmpdir
# rm -rf /tmp/<tmpdir>
```

We explicitly do NOT copy `.gitmodules` or the `.claude` gitlink from the
template — those point to `vc-template-x1.claude` and would have a dangling
SHA. We create fresh ones in step 4.

Do obvious string substitutions in copied files:
- `CLAUDE.md`: replace `vc-template-x1` with `<name>`, replace
  `hw-jjg-bot` with `<name>` (the header comment references this)
- `README.md`: replace `vc-template-x1` with `<name>`
- `.vc-config.toml`: should be fine as-is (`path = "/"`,
  `other-repo = ".claude"`)

#### 4. Wire up the submodule

From inside `<name>/`:

```sh
git submodule add git@github.com:<owner>/<name>.claude.git .claude
```

This creates:
- `.gitmodules` with the correct URL and `path = .claude`
- The gitlink (mode `160000`) pointing to `<name>.claude`'s HEAD commit
- `.git/modules/<name>.claude/` with the cloned submodule

#### 5. Initialize jj

```sh
# In code repo
jj git init --colocate

# In .claude submodule
cd .claude
jj git init --colocate
cd ..
```

#### 6. Initial commit and push the code repo

```sh
jj describe -m "feat: Initial dual-repo project <name>"
jj bookmark create main -r @
jj git remote add origin git@github.com:<owner>/<name>.git
jj bookmark track main --remote=origin
jj git push
```


#### 7. Create the claude-code symlink

Call the internal symlink function (same logic as `vc-x1 symlink`):
- target: `.claude`
- yes: true (non-interactive)

#### 8. Summary output

Print a recap:

```
Created dual-repo project: <name>
  Code repo:    https://github.com/<owner>/<name>
  Session repo: https://github.com/<owner>/<name>.claude
  Local path:   <dir>/<name>
  Symlink:      ~/.claude/projects/<encoded> -> <dir>/<name>/.claude
```

## `vc-x1 clone <url>`

Clones an existing dual-repo project.

### CLI

```
vc-x1 clone <url> [OPTIONS]
```

Arguments:
- `URL` — git URL of the code repo (SSH or HTTPS)

Options:
- `--dir <PATH>` — parent directory to clone into (default: cwd)

### Steps

#### 1. Preflight checks

- Verify `jj` is installed
- Parse repo name from URL (strip trailing `.git`, take last path component)
- Verify `<dir>/<name>` does not already exist locally

#### 2. Clone with submodules

```sh
git clone --recursive <url> <dir>/<name>
```

If the clone succeeds but `.claude` is empty (user's repo might not have
been set up with submodules), try:

```sh
cd <name>
git submodule update --init
```

If `.claude` is still empty or `.gitmodules` doesn't exist, warn the user
that this doesn't appear to be a dual-repo project and skip session-repo
setup.

#### 3. Initialize jj

```sh
cd <name>
jj git init --colocate

cd .claude
jj git init --colocate
cd ..
```

#### 4. Create the claude-code symlink

Call the internal symlink function:
- target: `.claude`
- yes: true

#### 5. Validate

Check that `.vc-config.toml` exists and `workspace.other-repo` matches
`.claude`. Warn (don't error) if missing or mismatched.

#### 6. Summary output

```
Cloned dual-repo project: <name>
  Code repo:    <url>
  Session repo: <submodule-url from .gitmodules>
  Local path:   <dir>/<name>
  Symlink:      ~/.claude/projects/<encoded> -> <dir>/<name>/.claude
```

## String substitutions for `new`

When copying template files, perform these replacements. This is a simple
find-and-replace pass, not a template engine.

| Pattern | Replacement | Files |
|---|---|---|
| `vc-template-x1` | `<name>` | CLAUDE.md, README.md |
| `hw-jjg-bot` | `<name>` | CLAUDE.md |
| `winksaville/vc-template-x1.claude` | `<owner>/<name>.claude` | .gitmodules (but we don't copy this — created by submodule add) |

If additional patterns are found that should be replaced, they can be added
to a table in `.vc-config.toml` or a separate config in a future iteration.

## Implementation notes

### External commands

Both subcommands shell out to `gh`, `git`, and `jj` via `std::process::Command`.
This matches the existing pattern in vc-x1 (finalize shells out to `jj`).

Helper function suggestion:

```rust
fn run_cmd(program: &str, args: &[&str], cwd: &Path) -> Result<String, Error>
```

That captures stdout, checks exit status, and returns a meaningful error
with the command that failed.

### Temp directory for template cloning

Use `tempfile::TempDir` (or `std::env::temp_dir()` + uuid if avoiding the
dep) for the shallow clones of the template repos. Clean up on success or
failure.

### The `gh repo create --clone` nuance

`gh repo create` with `--clone` creates the remote repo AND clones it locally.
However, it creates an empty repo. We then populate it from the template
content. This is intentional — GitHub's `--template` flag would copy the
gitlink SHA which is exactly the problem we're avoiding.

### Error recovery

If the command fails partway through, it may leave partial state (a GitHub
repo created but not populated, a local directory partially set up). For the
first pass, just print what happened and let the user clean up. A future
improvement could add `--cleanup-on-failure` that deletes created repos.

### DRY: shared helpers

Do NOT duplicate command sequences between `new` and `clone`. Extract
shared operations as small functions that each subcommand composes:

```rust
fn run_cmd(program: &str, args: &[&str], cwd: &Path) -> Result<String>
fn check_tool(name: &str) -> Result<()>           // verify gh/jj installed
fn gh_user() -> Result<String>                     // current gh user
fn jj_init_colocate(path: &Path) -> Result<()>     // jj git init --colocate
fn jj_initial_commit(path: &Path, msg: &str, remote_url: &str) -> Result<()>
fn create_symlink(target: &Path, yes: bool) -> Result<()>  // from symlink subcommand
fn seed_from_template(src_repo_url: &str, dest: &Path, exclude: &[&str]) -> Result<()>
fn replace_in_files(dir: &Path, replacements: &[(&str, &str)]) -> Result<()>
```

The subcommand handlers (`cmd_new`, `cmd_clone`) should read as a short
sequence of these calls — if either handler has more than ~5 lines of
inline logic that isn't a helper call, it probably needs extraction.

The `jj_initial_commit` helper encapsulates: describe, bookmark create,
remote add, bookmark track, git push. One function, called for both the
code repo and session repo in `new`. If `--no-jj` is ever added later,
only these helpers need a second code path — not the callers.

## Future extensions (not in first pass)

- `--no-jj` flag to support plain git (only the helpers need a second path)
- `--template <url>` flag to use a different template pair
- `vc-x1 init` to retrofit an existing repo into the dual-repo scheme
- `--private` flag for private GitHub repos
- Automatic CLAUDE.md customization beyond string replacement
- `--cleanup-on-failure` for atomic-ish creation
