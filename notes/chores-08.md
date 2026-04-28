# Chores-08.md

General chores notes for the 0.41.1 init+clone redesign cycle.

This file opens at 08 (skipping 07) because chores-07.md is the
0.42.0-cycle file living on `main`. The 0.41.1 cycle branched
off `6747a27` (the 0.41.0 close-out commit), where chores-07.md
doesn't exist. Starting at 08 here avoids any rebase collision
on chores-07 when `main` later rebases on top of
`init-clone-refactor`.

For the broader forking model and partner-bookmark technique
that drove this branch's existence, see
[`forks-multi-user.md`](forks-multi-user.md).

## init + clone redesign (0.41.1)

Empirical validation 2026-04-27 against `vc-x1 0.42.0-3` (see
[`vc-x1-init.md`](vc-x1-init.md)) surfaced cosmetic anomalies
and a substantive design gap. Init's flag surface
(`--repo-local`, `--repo-remote`, `--owner`, `--dir`,
`[NAME]`) carries 6+ mutually-exclusive preflight checks just
to prevent impossible combinations from being typed. Clone
already implements a unified-positional alternative (URL /
`owner/name` shorthand); this cycle extends that pattern to
init, adds POR support to both, and unifies their CLI
surfaces.

Branched off `6747a27` (0.41.0 close-out). On close-out, the
in-flight 0.42.0 work on `main` rebases on top.

### Command structure

```
vc-x1 init  <TARGET> [NAME] [--scope code,bot|por] [--private] [--dry-run]
vc-x1 clone <TARGET> [NAME] [--scope code,bot|por]             [--dry-run]
```

Identical surfaces modulo `--private` (init only).

`<TARGET>` accepts:

- **URL** — `git@host:owner/name(.git)?`,
  `https://...(.git)?` (detected by `://` or SSH
  `<host>:<path>` shape with `@`).
- **`owner/name` shorthand** — single `/`, no path prefix;
  resolved to `git@github.com:owner/name.git`.
- **Path-prefixed** — `./X`, `../X`, `/X`, `~/X`, `~`.
  Path IS the target; last component is the workspace name.
  Bare `.` and bare `NAME` are errors (require explicit
  prefix).

`[NAME]`:

- For URL / `owner/name` forms: overrides the derived
  destination dir name. `vc-x1 clone owner/foo my-name`
  clones into `./my-name/` instead of `./foo/`.
- For path-prefix form: error if combined (path already
  specifies the full target).
- Path-form for clone: error (clone needs a remote URL; see
  "Operations" for clone's target rules).

`--scope`:

- `code,bot` (default): both repos, dual-repo layout.
- `por`: single repo, no `.vc-config.toml` written.
- `code` / `bot` standalone: error. Reason — these are
  config-lookup keywords; init has no config to look up
  against, and clone's simplification omits them too. The
  manual decomposition (clone code as POR, clone bot as
  POR, place at `.claude/`, run `vc-x1 symlink`) covers the
  rare cases where a user wants the dual layout composed
  by hand.

`--private`: only meaningful for init when creating a new
GitHub repo. Sets visibility at `gh repo create`. Silently
warn-and-ignore if the remote already exists. Errors on
clone (clone never creates a remote).

`--dry-run`: validate the operation as far as possible
without side effects. Print what would be done.

### Operations

**Init** is *smart* — it detects the target dir's state and
chooses the right operation.

| Target state | `--scope=code,bot` | `--scope=por` |
|---|---|---|
| empty / nonexistent | create dual-repo from scratch | create POR |
| POR (jj+git, no config) | upgrade to dual | error (already POR) |
| POR (git only, no `.jj/`) | bootstrap `jj git init`, then upgrade | error (already POR-shape) |
| code POR + `.claude/` peer POR (configs missing) | write configs only | error |
| dual-repo (configs present) | error (already there) | error (downgrade) |
| single-repo (illegal shape) | error | error |
| anything else (random files, etc.) | error with diagnostic | error with diagnostic |

**Clone** is *dumb* — it just clones URLs into target dirs.

| `--scope=por` | clone the URL into `<target>` |
| `--scope=code,bot` | clone code URL into `<target>`, derive bot URL (`<URL>.claude`), clone into `<target>/.claude/`, then create symlink via `vc-x1 symlink` |

For both clone modes: target must NOT exist (`vc-x1 clone`
errors if anything is there). Use `vc-x1 sync` to update an
existing checkout.

`.vc-config.toml` files for the dual-repo case are *tracked
content in the remotes*, not files clone writes. They ride
along with the clone. If a remote doesn't have them
committed, `vc-x1 init <path> --scope=code,bot` from inside
the cloned dir handles the config-only upgrade case (see
init's state table above).

`--scope=code,bot` is implemented as in-process composition
of the `--scope=por` primitive (called twice: once for code
URL, once for derived bot URL) plus a `symlink::create`
call. No subprocess overhead.

If init or clone fails partway through, manual cleanup is
required (no rollback).

### Example layouts (local repos)

cwd = `~/prgs`

`vc-init test-repos/tf1 --scope=code,bot`
`vc-init test-repos/tf1`

TODO: add actual output of ls or tree cmds (fill at
close-out).

`vc-init ../test-repos/tf1 --scope=code,bot`
`vc-init ../test-repos/tf1`

TODO: add actual output of ls or tree cmds (fill at
close-out).

### Preflight

- Verify `jj` is installed (`jj --version`); friendly error
  with install link if missing.
- For init URL / `owner/name` forms: probe with
  `git ls-remote <url>`. Exists → clone. Doesn't → create
  via `gh repo create` (errors if host isn't GitHub or
  `gh` missing).
- For clone URL / `owner/name` forms: probe with
  `git ls-remote <url>`. Exists → clone. Doesn't → error
  (clone is dumb, no auto-create).

### Edits

- `Cargo.toml`: bump to `0.41.1`.
- New `src/repo_url.rs` (or fold into `src/common.rs`):
  lift `derive_name` and `resolve_url` from `clone.rs`. Add
  `parse_target(s) -> Target` enum
  `{Url(String), OwnerName(String, String), Path(PathBuf)}`.
  Single source of truth for positional parsing across init
  and clone.
- `src/clone.rs`:
  - `CloneArgs`: `<TARGET>` + optional `[NAME]` positionals;
    add `--scope code,bot|por` (default `code,bot`); drop
    `--dir`. Old `--repo` positional rename: `repo` →
    `target`.
  - Refactor body into `clone_one(url, target_dir)` (the
    primitive, also used by `--scope=por`) and
    `clone_dual(code_url, target_dir)` (orchestrator —
    derives bot URL, calls `clone_one` twice, calls
    `symlink::create`).
  - Pre-clone check: error if `target_dir` exists.
- `src/init.rs`:
  - `InitArgs`: drop `owner`, `repo_local`, `repo_remote`,
    `dir`; replace bare-name `name` positional with
    `<TARGET>` + optional `[NAME]`.
  - Remove the 6 mutually-exclusive flag checks.
  - New `detect_state(path) -> InitTargetState`
    `{Empty, PorJjGit, PorGitOnly, PorWithPeerPor,
    SingleRepoWorkspace, DualRepoWorkspace, Other(reason)}`.
  - Refactor body into `init_one(target_dir, opts)` (the
    `--scope=por` primitive, also used as a step inside
    `--scope=code,bot`) and `init_dual(target_dir, opts)`
    (orchestrator — calls `init_one` twice, writes both
    `.vc-config.toml` files, ochid-links,
    `symlink::create`).
  - POR upgrade paths:
    - PorJjGit + `--scope=code,bot` → skip code-side init,
      run bot-side `init_one` into `.claude/`, write
      configs, ochid-link existing code commit + new bot
      commit, symlink.
    - PorGitOnly + `--scope=code,bot` → `jj git init`
      first, then PorJjGit path.
    - PorWithPeerPor + `--scope=code,bot` → write only the
      two `.vc-config.toml` files + symlink (both repos
      already exist).
- `src/test_helpers.rs`: `Fixture::new_opts` reshapes to
  use the new `target` positional shape (and `[NAME]` if
  needed).
- `notes/todo.md`: cycle ladder in `## In Progress`.
- `notes/chores-08.md`: per-step post-impl subsections
  (this file).
- `notes/vc-x1-init.md`: close out cosmetic anomalies (fold
  into final close-out).

### Cycle structure — multi-step

- `-0` — plan + version bump + `notes/forks-multi-user.md`
  capture + `notes/chores-08.md` (this section) +
  `notes/vc-x1-init.md` brought forward + partner
  bookmarks set up.
- `-1` — lift `derive_name` / `resolve_url` /
  `parse_target` to shared module; clone migrates
  internally (no behavior change yet).
- `-2` — clone reshape: `<TARGET>` + `[NAME]`
  positionals, add `--scope code,bot|por`, refactor into
  `clone_one` / `clone_dual`. Add target-exists pre-check.
- `-3` — init reshape: drop old flags, add `<TARGET>` +
  `[NAME]`, add `--scope=por`, refactor into `init_one` /
  `init_dual`. Existing create-from-empty operations work
  via the new shape.
- `-4` — init POR detection + upgrade paths
  (PorJjGit, PorGitOnly auto-bootstrap, PorWithPeerPor
  config-only).
- `-5` — `test_helpers::Fixture` migration; audit
  downstream callers across the test suite.
- final — cycle close-out: fill in Example Layout outputs,
  address `notes/vc-x1-init.md` cosmetic anomalies, drop
  the `-N` suffix.

### Decisions made during design

- **Version + cycle line.** This work + the sync `--check`
  fix land on the 0.41.x line. Init+clone = 0.41.1. Sync
  fix = a separate cycle (likely 0.41.2). Then rebase the
  in-flight 0.42.0 work on top of both.
- **Path-prefix vocabulary.** `./NAME` and the standard
  prefixes (`../`, `/`, `~/`, `~`) only. Bare `.` and
  bare `NAME` are errors — explicit prefix required.
- **`--private` on existing remote.** Warn and ignore;
  visibility was set at create time.
- **Cosmetic anomalies** from `notes/vc-x1-init.md` —
  addressed at close-out, not deferred.
- **`--scope=code` and `--scope=bot` for clone.** Dropped
  from the menu. Manual decomposition (two `--scope=por`
  clones + `vc-x1 symlink`) covers the use case.
- **Composition over duplication.** `--scope=code,bot` is
  implemented as in-process composition of the
  `--scope=por` primitive — single source of truth for
  the actual clone/init operation, thin wrapper for the
  dual case.
- **Branch fork mechanics.** Code-side `init-clone-refactor`
  bookmark created at `6747a27`; bot-side partner bookmark
  at current `.claude` `main`. Main left alone as recovery
  anchor. See [`forks-multi-user.md`](forks-multi-user.md)
  for the full discussion.

# References

[1]: forks-multi-user.md
