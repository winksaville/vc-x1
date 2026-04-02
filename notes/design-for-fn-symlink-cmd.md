# vc-x1 symlink subcommand design spec

## Open questions

- How does claude-code "project-files" work on MacOS and Windows, in parituclar
  will the symlink trick work?
 
## Background

Claude Code stores session data in `~/.claude/projects/<encoded-path>/`. We use
a dual-repo scheme where the session data lives in a `.claude` directory that is
a separate jj-git repo (the "session repo"), sibling to the code repo. A symlink
in `~/.claude/projects/` points to this local `.claude` directory so Claude Code
finds its session data there.

We currently use a bash script (`claude-symlink.sh`) for this. This spec ports
that functionality into vc-x1 as a native Rust subcommand, so `new` and `clone`
commands (planned later) can call it programmatically.

## Path encoding convention

The symlink name is the absolute working directory with every `/` replaced by `-`.

Example: cwd `/home/wink/data/prgs/vc-x1` produces symlink name
`-home-wink-data-prgs-vc-x1`, and the full symlink is:

```
~/.claude/projects/-home-wink-data-prgs-vc-x1 -> /home/wink/data/prgs/vc-x1/.claude
```

## CLI interface

```
vc-x1 symlink [OPTIONS] [TARGET]
```

Arguments:
- `TARGET` — directory to link to (default: read `other-repo` from
  `.vc-config.toml`, fallback to `.claude`)

Options:
- `--symlink-dir <PATH>` — override symlink parent directory
  (default: `~/.claude/projects`)
- `-l, --list` — list contents of the symlinked directory after creation
  (use the symlink path with trailing `/` to verify the symlink actually works)
- `-y, --yes` — replace existing symlink without prompting (needed for
  non-interactive use by future `new`/`clone` subcommands)

## Behavior

1. Resolve target to absolute path. If target directory doesn't exist, create it
   with `create_dir_all`.
2. Get `current_dir()`, convert to string, replace all `/` with `-` to form
   the symlink name.
3. Ensure symlink parent directory exists (`create_dir_all`).
4. Check what exists at the symlink path:
   - **Nothing** → create symlink.
   - **Existing symlink** → print current target via `read_link()`, prompt
     "Replace? [y/N]" unless `--yes` was passed. On yes, remove and recreate.
     On no, abort.
   - **File or directory** (not a symlink) → error and abort. Use
     `symlink_metadata()` (doesn't follow symlinks) to distinguish.
5. Create symlink with `std::os::unix::fs::symlink()`.
6. Print: `Created: <symlink_path> -> <abs_target>`
7. If `--list`, print contents by reading through the symlink path (end-to-end
   verification that the symlink works).

## Target default resolution

Fallback chain for target when no argument given:
1. `.vc-config.toml` field `workspace.other-repo` (already used elsewhere in vc-x1)
2. `.claude`

## Implementation notes

- `cfg(unix)` — `libc` is already a dependency. The `std::os::unix::fs::symlink`
  function is available. Windows support is not a current concern.
- Interactive prompt: use `std::io::stdin().read_line()` — no need for an
  external crate.
- This should be implemented as a reusable function (not just a subcommand
  handler) so future `new` and `clone` subcommands can call it with `yes: true`.
- Follow existing code patterns in vc-x1 for clap derive, error handling, etc.

## Existing script for reference

The original `claude-symlink.sh` (in `~/.local/bin/`) does exactly this in bash.
Key lines for the encoding:

```bash
cwd="$(pwd)"
symlink_name="${cwd//\//-}"
symlink_path="$symlink_dir/$symlink_name"
ln -sfn "$abs_target" "$symlink_path"
```

## Future context

After `symlink` works, the next subcommands planned are:
- `vc-x1 new <name>` — create both GitHub repos, set up submodule, init jj,
  call symlink
- `vc-x1 clone <url>` — `git clone --recursive`, init jj, call symlink
