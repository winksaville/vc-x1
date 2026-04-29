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
vc-x1 init  <TARGET> [NAME] [--scope code,bot|por] [--account <a>] [--repo <cat>[=<val>]] [--private] [--dry-run]
vc-x1 clone <TARGET> [NAME] [--scope code,bot|por]                                                    [--dry-run]
```

Init carries `--account`, `--repo`, and `--private`;
clone has none of them. Clone's TARGET is the source
(URL/path); init's TARGET is the workspace name/path
combined with config-resolved remote.

`<TARGET>` accepts:

- **URL** — `git@host:owner/name(.git)?`,
  `https://...(.git)?` (detected by `://` or SSH
  `<host>:<path>` shape with `@`).
- **`owner/name` shorthand** — single `/`, no path prefix;
  resolved to `git@github.com:owner/name.git`.
- **Path-prefixed** — `./X`, `../X`, `/X`, `~/X`, `~`,
  bare `.`, or bare `..`. Path IS the target; last component
  is the workspace name (consumer canonicalizes `.` / `..`
  to a real basename via `canonicalize` + `file_name`).
- **Bare alphanumeric NAME** — for **init** only, becomes
  the workspace name; the remote URL is resolved via the
  user config's `--repo` chain
  ([`User config`](#user-config-0411-3-redesigned-in-0411-4)).
  Missing required config keys produce the
  step-specific errors documented there. For **clone**,
  bare NAME is an error (genuinely ambiguous — "missing
  `./`?" / "missing `/name` suffix?"); clone has no
  config-driven defaults.

`[NAME]`:

- **init**: URL / `owner/name` forms → overrides the derived
  workspace name (workspace at `cwd/<NAME>`). Path forms →
  error if combined (path already specifies the destination).
- **clone**: all forms (URL, `owner/name`, Path-as-source) →
  destination dir name in cwd. Without `[NAME]`, derived
  from the source via `derive_name`. For clone, TARGET is
  the *source* (URL or local repo path) and `[NAME]` is
  the *destination* — symmetric with `git clone <src>
  [<dest>]`.

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

`--account <name>` (init only): override
`[default].account`. Determines which `[account.<name>]`
section is consulted for `--repo` resolution.

`--repo <cat>[=<val>]` (init only): pick the repo target
for this run.
- `--repo` absent → use account's `repo.default` →
  category's value.
- `--repo <cat>` → use `<cat>`'s configured value.
- `--repo <cat>=<val>` → use literal `<val>`.

Built-in categories: `remote` (value is a URL prefix —
init appends `/<NAME>.git`) and `local` (value is the
parent dir for fixture bare repos at
`<parent>/remote-{code,claude}.git`). Replaces the old
`--owner` / `--repo-local` / `--repo-remote` /
`--fixture` flag set with one unified mechanism.

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

**Clone** is *dumb* — it just clones sources (URLs or local
repo paths) into target dirs.

| `--scope=por` | clone the source into `<target_dir>` |
| `--scope=code,bot` | clone code source into `<target_dir>`; derive bot source (`<source>.claude`); clone into `<target_dir>/.claude/`; create symlink via `vc-x1 symlink` |

For both clone modes: `<target_dir>` must NOT exist
(`vc-x1 clone` errors if anything is there). Use
`vc-x1 sync` to update an existing checkout.

Path-form for clone is symmetric with `git clone /local/bare.git`
— useful for local fixtures, CI scratch dirs, integration
tests against bare repos. Init/clone differ on what TARGET
*means* (init: destination; clone: source) but share the
same `parse_target` acceptance set.

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
  `derive_session_url` to a new `src/repo_url.rs`; stage
  `Target` + `parse_target` for `-2`/`-3` consumers; both
  `clone` and `init` migrate at their existing call sites
  (init.rs had verbatim duplicates worth de-duping in the
  same step). No behavior change.
- `-2` — clone reshape: `<TARGET>` + `[NAME]`
  positionals, add `--scope code,bot|por`, refactor into
  `clone_one` / `clone_dual`. Add target-exists pre-check.
- `-3` — user config (first cut):
  `~/.config/vc-x1/config.toml` reader; flat
  `[default]` + `[github]` schema; standalone module.
  Superseded by `-4`'s redesign before any consumer
  wired in.
- `-4` — user config redesign: replace `-3`'s flat
  schema with `[account.<name>].repo.category.<cat>`
  multi-account / category structure, literal values,
  three-level resolution chain. Standalone; `-5` is the
  first consumer. See
  [`User config`](#user-config-0411-3-redesigned-in-0411-4).
- `-5` — init reshape: drop old flags
  (`--owner` / `--dir` / `--repo-local` / `--repo-remote`),
  add `<TARGET>` + `[NAME]`, `--account`, `--repo`,
  `--scope code,bot|por`. Refactor into `init_one` /
  `init_dual`. Existing create-from-empty operations
  work via the new shape.
- `-6` — init POR detection + upgrade paths
  (PorJjGit, PorGitOnly auto-bootstrap, PorWithPeerPor
  config-only).
- `-7` — `test_helpers::Fixture` migration; audit
  downstream callers across the test suite.
- final — cycle close-out: fill in Example Layout outputs,
  address `notes/vc-x1-init.md` cosmetic anomalies, document
  `~/.config/vc-x1/config.toml` in `README.md` (simplest /
  full-one-account / full-two-account examples), drop
  the `-N` suffix.

### User config (0.41.1-3, redesigned in 0.41.1-4)

User-global config at `~/.config/vc-x1/config.toml`
(XDG-compliant; honors `$XDG_CONFIG_HOME`). Backs init's
account- and repo-target resolution. No magic fallbacks:
missing file or missing key → predictable error pointing
at the exact key to set.

`-3` shipped a flat first-cut schema (`[default]
repo-remote-provider`, `[github] owner`); `-4` replaces it
with the account/category structure below before init
becomes the first consumer in `-5`. Since `-3` had no
consumers, the rewrite is non-breaking.

Schema:

```toml
[default]
account = "home"      # default --account when absent
debug   = "trace"     # default --debug value when used without arg

[account.home]
repo.default          = "remote"                       # default --repo cat when absent
repo.category.remote  = "git@github.com:winksaville"   # value for --repo remote (no =val)
repo.category.local   = "~/test-fixtures"              # value for --repo local (no =val)

[account.work]
repo.default          = "remote"
repo.category.remote  = "git@github.com:anthropic"
repo.category.local   = "/work/fixtures"
```

Three layers — each level has its own default-finding key:

| CLI                       | Step 1: account                | Step 2: category                  | Step 3: value                                    |
|---|---|---|---|
| (no `--repo`/`--account`) | `[default].account`            | `[account.<a>.repo].default`      | `[account.<a>.repo.category].<cat>`              |
| `--account <a>`           | `<a>` (CLI)                    | `[account.<a>.repo].default`      | `[account.<a>.repo.category].<cat>`              |
| `--repo <cat>`            | `[default].account` (or CLI)   | `<cat>` (CLI)                     | `[account.<a>.repo.category].<cat>`              |
| `--repo <cat>=<val>`      | `[default].account` (or CLI)   | `<cat>` (CLI)                     | `<val>` (CLI, literal)                           |

Values are **literal targets**, not section-name
pointers. For `category = "remote"`, the value is a URL
prefix (init appends `/<NAME>.git`); for `category =
"local"`, it's the parent dir for fixture bares. Built-in
categories `remote` and `local` are recognized; any other
category name errors in `-5` (forward-compat: future
cycles can add a `kind` field for user-defined categories).

Resolution errors (each step has its own message):

- Step 1 missing → `no account specified; set [default].account, use --account <name>, or write a top-level [repo] section`.
- Step 2 missing → `no default category for account '<a>'; set [account.<a>.repo].default or use --repo <cat>`.
- Step 3 missing → `no value for --repo <cat> in account '<a>'; set [account.<a>.repo.category].<cat> or use --repo <cat>=<val>`.

#### Single-account shorthand: top-level `[repo]`

`[account.<name>]` boilerplate is overhead when there's
only one account. The loader accepts a top-level `[repo]`
section as a single-account shorthand:

```toml
[repo]
default          = "remote"
category.remote  = "git@github.com:winksaville"
category.local   = "~/test-fixtures"
```

3-4 lines for a complete config — no `[default].account`,
no `[account.<name>]`. Resolution: when no `--account` is
given and no `[default].account` is set, the loader uses
the top-level `[repo]` block as the implicit account.

Mixing top-level `[repo]` **and** `[account.*]` sections
is rejected at load time (ambiguous which one init should
consult — error: `mixing top-level [repo] with [account.*]
is ambiguous`).

Top-level error format uses `[repo.category].<cat>` rather
than `[account.<a>.repo.category].<cat>`.

Per-workspace `.vc-config.toml` override (ancestor walk)
is deferred — chicken-and-egg at init time, and the
global file covers the dominant use case.

Module shape:

- `pub struct UserConfig { default_account, default_debug, top_level_repo: Option<AccountConfig>, accounts: HashMap<String, AccountConfig> }`.
- `pub struct AccountConfig { repo_default, repo_category: HashMap<String, String> }`.
- `pub struct RepoSelector { category, value: Option<String> }` — parsed `--repo` form.
- `load()` / `load_from(path)` — load + group dotted keys.
- `resolve_repo(cfg, account_override, repo_cli) -> (cat, value)` — three-step chain.

`-4` lands the rewritten module standalone; `-5` (init
reshape) is the first consumer.

#### Refactoring opportunities (post-0.41.1)

Two cleanups surfaced while sizing the module that we
deliberately deferred to keep `-3` scoped:

- **Unify `.vc-config.toml` accessors onto Pattern B.**
  The tree has two patterns for reading TOML config:
  - Pattern A (`desc_helpers.rs`, `fix_desc.rs`,
    `validate_desc.rs`): call site does
    `toml_simple::toml_load(path)` and passes the
    resulting `HashMap<String, String>` to map-typed
    accessor fns (`other_repo_from_config(&map)`,
    `ochid_prefix_from_config(&map)`).
  - Pattern B (`push::resolve_state_layout`, new
    `config::load_from`): function takes a path,
    returns a typed struct with the fields it cares
    about; conversion baked in.
  Pattern B is more discoverable and testable. A
  `WorkspaceConfig` struct with `load_from(path)` could
  replace the Pattern-A helpers across desc_helpers /
  fix_desc / validate_desc. ~50 LOC, mechanical.
- **Layered config precedence** — once `WorkspaceConfig`
  is typed, layering user → workspace → CLI becomes
  natural (workspace can override `[github].owner` for a
  specific project). Init can't use this layer because
  it runs *before* a workspace exists, but post-init
  commands could.

Both candidates for the 0.41.2 cycle. See `notes/todo.md`.

### init reshape (0.41.1-5)

Init's flag surface replaced with `<TARGET> [NAME] --account
--repo --scope`. Six mutually-exclusive flag checks gone;
dispatch is now `parse_target` + the config `--repo` chain.

**InitArgs surface:**

- Dropped: `--owner`, `--dir`, `--repo-local`,
  `--repo-remote`, the old `Vec<Side>` scope.
- Added: `target` positional (URL, `owner/name`, path, or
  bare NAME), optional `[NAME]`, `--account`,
  `--repo <cat>[=<val>]`, `--scope code,bot|por` (typed
  `ScopeKind`).

**Dispatch (`plan_init(args, cfg)`):**

- `Target::Url(u)` / `OwnerName(o, n)` → `plan_from_url`.
  URL is explicit; `--account` and `--repo` are rejected
  ("config not consulted").
- `Target::Path(p)` → `plan_from_path`. Path IS the
  destination; basename names the repo; remote via the
  config chain.
- `Target::BareName(n)` → `plan_from_bare_name`.
  Destination at `cwd/<n>`; remote via the config chain.
- Path/BareName route through `config::resolve_repo` to
  `plan_remote` (URL prefix + `/<NAME>.git`) or
  `plan_local` (`<parent>/remote-{code,claude}.git` for
  dual; `<parent>/remote.git` for POR).

**Shared `src/args.rs`:**

Cross-subcommand CLI types/parsers lifted from init/clone:
`ScopeKind { CodeBot, Por }` (replaces `CloneScope` + a
private `InitScope`), `parse_scope_kind` (subcommand-
agnostic error wording), `parse_repo_arg`. 0.42.0 sum-type
cycle is expected to extend `ScopeKind` with `Single(_)` /
path-form variants.

**`config::resolve_repo` short-circuit:**

`--repo <cat>=<val>` fast-paths — fully self-contained, no
account lookup. Lets config-less invocations and test
fixtures work without touching disk.

**Bug fix (dogfood-surfaced):**

`parse_target` mis-classified `host:owner/name` (SSH form
missing the `git@` prefix) as `OwnerName(host:owner, name)`,
producing a doubled-up URL
`git@github.com:host:owner/name.git`. Fix: reject `:` in
either half of `owner/name` shorthand. Catch-all error
gained a "looks like an SSH URL missing the 'git@' prefix;
did you mean 'git@…'?" suggestion.

**Debug logging:**

`debug!` (visible at `-v`) added for config-and-CLI
provenance:

- `init` entry — InitArgs fields.
- `parse_target` — input → variant.
- `plan_init` — final plan summary (project_dir, name,
  code_url, provisioner, slugs, bare paths).
- `config::load_from` — file hit/miss + parsed summary.
- `config::resolve_repo` — per-step source annotation
  (`--account CLI` / `[default].account` / `top-level
  [repo]` / `--repo CLI value` /
  `[account.<a>.repo.category].<cat>`).

**Cleanups:**

Dropped: `gh_whoami` (only consumer was old
`plan_default_github`); `Scope::is_bot_only` and
`Scope::is_empty` (only consumers were `Vec<Side>`
validation); `normalize_remote` (only consumer was old
`plan_external_remote`).

**Tests:**

319 passing. `init.rs` test module rewritten end-to-end
against the new `plan_init(args, cfg)` signature; new
`args_for(target)` builder + four `cfg_*` config builders;
new tests cover URL/OwnerName/Path/BareName dispatch,
account override, top-level `[repo]` shorthand,
`--repo cat=val` short-circuit, and all error paths
(URL+`--account`, URL+`--repo`, Path+`[NAME]`,
BareName+`[NAME]`, bare-name+empty config, unknown
category, `--scope=por`+comma template).

`test_helpers::Fixture::new_opts` migrated to path TARGET +
`--repo local=<base>`; `sync` and `push` integration tests
confirm the same on-disk layout.

**Deferred:**

`init_one` / `init_dual` extraction — `init_with_symlink`
operates on `InitPlan` (shape unchanged), so the extraction
is a refactor not blocking functionality. Fold into -6
alongside POR detection if natural; or its own step.

**WIP ladder:**

`0.41.1-5.0` dispatch reshape → `-5.1` tests + bug fix +
debug logging + CLAUDE.md per-file review subsection +
todo rebase note → `-5.2` `gh_whoami` / `Scope` deletions +
this chores entry → `-5` done marker.

### POR baseline integration tests (0.41.1-6.0)

First micro-commit of the -6 cadence. Lands integration tests for
`--scope=por` before any production-code change in -6.1+. Authored
on a fresh `@` off `f5ec4d8` (-5 close-out) so the test commit sits
*upstream* of the literal-lift refactor (preserved as bookmark
`single-dual-1`); when that bookmark rebases as -6.1, the same suite
gates behavior preservation directly.

**Tests added (`src/init.rs`):**

- `por_fixture_creates_single_repo_layout` — `<base>/work/` exists,
  no `.claude/` peer, single bare at `<base>/remote.git`, no
  dual-shape `remote-{code,claude}.git`.
- `por_fixture_writes_app_only_config_files` — `path = "/"` with
  no `other-repo`, `.gitignore` has no `/.claude` exclusion.
- `por_fixture_main_tracks_origin` — `verify_tracking(&fx.work,
  "main")` succeeds (pins step 10 ran).

**Helper (`src/test_helpers.rs`):**

- `FixturePor { base, work }` + `::new(tag)` + `Drop`. Distinct type
  from `Fixture` because the POR layout has no `claude` peer —
  `Option<PathBuf>` on `Fixture` would force every dual-using caller
  to unwrap or pattern-match on access.

**Coverage gap closed:**

POR shipped in -5 with manual-dogfood validation only; `Fixture`
built dual layouts exclusively, so no `cargo test` exercised
`init_with_symlink`'s POR branches. -6.0 brings that to three
integration tests (320 total, 317 baseline + 3 new).

**Why this lands before the refactor:**

Tests *downstream* of a refactor only prove the test was written to
match the refactor's output. Tests *upstream* demonstrate behavior
preservation directly — every rebase from -6.1 onward must keep the
same suite green.

### Decisions made during design

- **Version + cycle line.** This work + the sync `--check`
  fix land on the 0.41.x line. Init+clone = 0.41.1. Sync
  fix = a separate cycle (likely 0.41.2). Then rebase the
  in-flight 0.42.0 work on top of both.
- **Path-prefix vocabulary.** `./NAME` and the standard
  prefixes (`../`, `/`, `~/`, `~`), plus bare `.` and `..`
  (POSIX cwd / parent — unambiguous). Bare alphanumeric
  `NAME` is an error — explicit prefix required.
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
