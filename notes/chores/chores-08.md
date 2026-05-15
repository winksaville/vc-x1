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
[`forks-multi-user.md`](../forks-multi-user.md).

## init + clone redesign (0.41.1)

Empirical validation 2026-04-27 against `vc-x1 0.42.0-3` (see
[`vc-x1-init.md`](../vc-x1-init.md)) surfaced cosmetic anomalies
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
- `notes/chores/chores-08.md`: per-step post-impl subsections
  (this file).
- `notes/vc-x1-init.md`: close out cosmetic anomalies (fold
  into final close-out).

### Cycle structure — multi-step

- `-0` — plan + version bump + `notes/forks-multi-user.md`
  capture + `notes/chores/chores-08.md` (this section) +
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

### Literal lift: extract init_one / init_dual (0.41.1-6.1)

`init_with_symlink`'s real-execution body factored into a dispatcher
plus two new functions: `init_one` (POR primitive) and `init_dual`
(dual orchestrator with interleaved code+session, cross-ref ochids,
and `.claude/`-preserving code-side clean). Pure code reorganization
— no behavior change. The 320-test suite from -6.0 (including the
3 new POR tests) is green against the lifted code, confirming
behavior preservation directly.

**Edits:**

- `src/init.rs`: new `init_one` and `init_dual`; `init_with_symlink`
  trimmed to a dispatch tail (`if is_dual { init_dual(...) } else
  { init_one(...) }`). Three duplicate `session_*` unwraps from the
  original dual path consolidated to top-of-fn in `init_dual`.
- `Cargo.toml`: 0.41.1-6.0 → 0.41.1-6.1.
- `notes/chores/chores-08.md`: this subsection.
- `notes/todo.md`: -6.0 → done, -6.1 → current.

**Mechanics:**

Authored on a sibling commit (bookmark `single-dual-1`) off -5's
close-out (`f5ec4d8`), then rebased onto -6.0 once tests landed.
`notes/todo.md` was peeled out of `single-dual-1` before rebase
(via `jj restore --from @-` while editing the bookmarked commit)
to keep -6.0's todo.md changes authoritative.

**File-size note:**

`src/init.rs` grew ~80 lines — per-side branches duplicated across
`init_one` and `init_dual`. The DRY-up across -6.2/-6.3/-6.4 should
bring it back below the original.

**Why a literal lift before -6.2:**

The substantive DRY refactor (-6.2/-6.3/-6.4 extract
`init_one_create`, `init_one_finalize`, `cross_ref_ochids` and
collapse `init_dual` to compose them) needs clean entry points.
Extracting `init_one` and `init_dual` as end-to-end functions
creates the call sites without yet touching internal structure, so
each subsequent extraction is reviewable against a stable surface.

### Extract create_repo + module reshape (0.41.1-6.2)

First substantive DRY pass on `init_with_symlink`. Lifts steps 1-5
(mkdir, git/jj init, configs, optional template, initial commit)
into a generalized per-side primitive `create_repo` living in a new
`src/repo_utils.rs` module. The dual orchestrator drops from ~180
lines to ~110 by replacing two ~50-line per-side blocks with two
`create_repo` calls. Same suite (320) plus 3 new unit tests for
the helper itself: total 323, all green.

**New module `src/repo_utils.rs`:**

- `pub fn create_repo(target, info_label, config: Option<&str>,
  gitignore: Option<&str>, template: Option<&Path>, name,
  ochid_strategy) -> Result<chid>`. Per-side primitive returning
  the new initial commit's chid.
- `pub enum OchidStrategy { None, Placeholder }` — initial-commit
  message policy. POR uses `None` (plain "Initial commit"); dual
  uses `Placeholder` ("Initial commit\n\nochid: /none", rewritten
  in step 6 once both chids are known).
- 3 unit tests: `strategy_none_writes_plain_commit`,
  `strategy_placeholder_writes_ochid_none`,
  `no_config_or_gitignore_writes_neither_file`. Last one pins the
  new None-skips-write semantics.

**Generalizations from -5/-6.1's inline shape:**

- `info_label` (was `side_label`) — drops the dual-specific "side"
  connotation; the label is just for `info!()` narration. POR
  passes `"code"`; dual passes `"code"` then `"bot"`; future
  upgrade paths or scratch-repo callers can pass anything else.
- `config` and `gitignore` are `Option<&str>` (was `&str`). `None`
  skips that file's write. Future use cases:
  - PorJjGit / PorWithPeerPor upgrade paths in -6.7/-6.8 leave
    existing `.vc-config.toml` in place.
  - A hypothetical `vc-x1 scratch-repo` subcommand wanting a bare
    git+jj init without project-specific files.

**Module rename `repo_url.rs` → `url.rs`:**

The old name was misleading — the module's contents (`Target`
enum, `parse_target`, `derive_name`, `resolve_url`,
`derive_session_url`) parse and derive *URLs and target strings*,
not "repos". Pure string manipulation, no repo concept.

- File renamed; jj tracks via content.
- `repo: &str` parameter on `derive_name` / `resolve_url` →
  `url: &str`. Internal references updated to match.
- Doc comments tightened: "repo URL" → "URL", "Resolve a repo
  argument" → "Resolve a target string", "session repo URL from a
  code repo URL" → "session URL from a code-side URL".
- All callers updated: `init.rs`, `clone.rs`, `main.rs` (`mod
  repo_url;` → `mod url;`).

**Renames in `init.rs`:**

- `init_dual` → `create_dual`. Symmetric with `create_repo` and
  signals that this is a creation-time orchestrator (not a
  generic init flow). `init_one` stays — we expect it to collapse
  into a 2-call composition by -6.4 (just `create_repo` +
  `finalize_repo`) and may not survive as a standalone function.
- `init_with_symlink`'s dispatcher updated to call `create_dual`.

**Visibility surface (`pub(crate)`) — to support cross-module use
from `repo_utils`:**

- `jj_chid` (was `fn`).
- `VC_CONFIG_APP_ONLY`, `VC_CONFIG_CODE`, `GITIGNORE_APP_ONLY`,
  `GITIGNORE_CODE` (were `const`). Used by repo_utils tests.
- `copy_template_recursive`, `rewrite_readme_first_line` were
  already `pub(crate)`.

**Cosmetic delta in user-visible output:**

Dual mode's step narration shifts from one interleaved pass
(`Step 1: ...` once, both sides mkdir, `Step 2: ...` once, etc.)
to two per-side passes (all five steps for code, then all five
for bot). Same actions, same order — just `info!()` ordering.
Not covered by tests.

**WIP ladder:**

Single commit. Bundled the `repo_url` → `url` rename and
`init_dual` → `create_dual` rename into the same -6.2 commit
since the whole change is "module structure & naming."

### Extract push_repo + rename create_repo → create_local_repo (0.41.1-6.3)

Second DRY pass on `init_with_symlink`. Lifts the post-create
per-side machinery (bookmark + clean + checkout, remote
provision/push, jj re-init + tracking) into a single helper
`push_repo` in `src/init.rs`. Both orchestrators (`init_one`,
`create_dual`) collapse hard: `init_one` from ~90 lines to ~35,
`create_dual` from ~165 lines to ~85. Suite stays at 323, all
green.

**Rename `create_repo` → `create_local_repo`:**

- Function in `src/repo_utils.rs`; 3 call sites in `init.rs`
  (1 in `init_one`, 2 in `create_dual`); 3 test `expect()`
  strings; all doc-comment references.
- Explicitly local — no remote interaction; pairs with the new
  `push_repo` (remote-side counterpart).

**New helper `push_repo` (in `init.rs`):**

```
fn push_repo(
    target, info_label, step_label_provision,
    clean_exclude: Option<&str>,
    plan, args, visibility,
    remote_url, gh_slug, bare_path,
) -> Result<String>  // final chid after re-init
```

Wraps three former-step regions per side:

- Bookmark `main` at `@-`, `git clean -xdf` (with optional
  `--exclude`), `git checkout main`.
- Calls `run_remote_step` (provision via
  GhCreate/LocalBareInit/ExternalPreExisting + `git remote add` +
  `git push -u origin main` with retry).
- `jj git init --colocate`, restore bookmark, track, verify, return
  `jj_chid("@-", target)`.

`clean_exclude` is the only per-call asymmetry: code-side dual
passes `Some(".claude")` to preserve the nested session repo;
session-side and POR pass `None`.

Lives in `init.rs` (not `repo_utils.rs`) because it depends on
`InitPlan` / `InitArgs` / `Provisioner` — types tightly coupled to
`init`'s plan-and-execute shape. If those decouple later, push_repo
can move.

**Doc-comment cleanup on `create_local_repo`:**

- Headline restructured into `Performs:` (5 actions) and
  `Parameters:` (one bullet per arg) lists. The old prose about
  "Steps 1-5" went away — `create_local_repo` no longer narrates
  step numbers, so the doc shouldn't either.
- "Per-side narration" paragraph dropped — when there's no step
  numbering, there's no step-vs-side narration trade-off to
  describe.
- Drops `Step N:` prefixes from `info!()` lines: `Creating ...`,
  `Initializing ...`, `Writing ...`, `Copying ...`,
  `Committing ...`, plus a final `Created local repo ... chid = ...`
  summary line.
- Skipped-step narration dropped: when both `config` and
  `gitignore` are `None`, or `template` is `None`, no `info!()` is
  emitted (was previously `(skipped — ...)`).

**Cosmetic delta in user-visible output:**

Two changes vs -6.2:

1. `info!()` lines drop `Step N:` prefixes (carried by
   `create_local_repo` and the new `push_repo`).
2. Dual mode now narrates fully sequential per side
   (session: bookmark/clean/checkout → push → re-init, then code:
   same), instead of the prior interleaved pattern (Step 7 both
   sides, then push session, then push code, then re-init both).
   Same subprocess invocations in the same order *within* each
   side; only across-side ordering shifts.

Not covered by tests.

**In-process tests (dual counterparts to -6.0's POR fixture
tests):**

Four new tests in `init.rs::tests` pin dual-shape invariants under
the new `push_repo` extraction. These are in-process fixture-driven
tests (calling `init::init_with_symlink` directly via
`Fixture::new`), not subprocess invocations of the `vc-x1` binary.
True CLI integration tests are scheduled for -6.4.

- `dual_fixture_creates_dual_repo_layout` — both repos and both
  bare origins present; POR-shape `remote.git` absent.
- `dual_fixture_writes_code_and_session_config_files` — code side
  has `path = "/"` + `other-repo = ".claude"`; session side has
  `path = "/.claude"` + `other-repo = ".."`; code-side
  `.gitignore` excludes `/.claude`.
- `dual_fixture_both_sides_track_origin` — `verify_tracking`
  passes on both `work` and `work/.claude` (per-side step 10
  ran for each `push_repo` call).
- `dual_fixture_preserves_claude_across_code_clean` — `.claude/.jj`,
  `.claude/.git`, and `.claude/.vc-config.toml` survive the
  code-side clean. Pins the `clean_exclude = Some(".claude")` path.

Suite: 323 → 327, all green.

**WIP ladder:**

Single commit; the `create_repo` → `create_local_repo` rename, the
`push_repo` extraction, and the dual-fixture in-process tests ride
together since the rename is mechanical and the new tests validate
the extracted helper's behavior.

### CLI subprocess integration tests + tempdir-root sharing (0.41.1-6.4)

First true CLI integration tests for the project. Adds a `tests/`
crate that spawns the `vc-x1` binary that Cargo built (via
`env!("CARGO_BIN_EXE_vc-x1")`) so argument parsing, exit codes, and
stdout/stderr are exercised end-to-end — distinct from the
in-process fixture tests under `src/init.rs::tests`, which call
`init::init_with_symlink` as a Rust function.

This step bumps the test-strategy floor from "in-process only" to
"both layers"; it doesn't yet retire any in-process tests. The
in-process tests stay as fast unit-ish coverage; subprocess tests
are added as state-mutating cases warrant a real binary spawn.

**New harness — `tests/common/mod.rs`:**

- `vc_x1() -> Command` — `Command::new(env!("CARGO_BIN_EXE_vc-x1"))`.
- `run_ok` / `run_err` — wrap `Command::output`, panic with
  stdout+stderr embedded if exit status doesn't match expectation.
- `CliFixture` — RAII tempdir owner. Each subprocess invocation
  gets `HOME` overridden to `<base>/home/` so the user's real
  `~/.config/vc-x1/` can't leak in or get clobbered.
- `unique_base(tag)` — mirrors `src/test_helpers::unique_base` but
  uses a `vc-x1-cli-test-` prefix to distinguish from in-process
  fixtures' `vc-x1-test-` paths.
- Crate-level `#![allow(dead_code)]` — standard idiom for
  `tests/common/`: each `tests/*.rs` compiles as its own crate, so
  helpers used by some test crates but not others would otherwise
  warn.

**New test files:**

- `tests/cli_smoke.rs` (2 tests):
  - `cli_version_runs` — `vc-x1 --version` exits 0 and stdout
    contains `"vc-x1"`. Pins `CARGO_BIN_EXE_vc-x1` resolves and
    the binary actually runs.
  - `cli_help_lists_init` — `vc-x1 --help` lists the `init`
    subcommand. Pins clap's subcommand surface compiled in.
- `tests/cli_init.rs` (2 tests, counterparts to in-process
  fixtures):
  - `cli_init_por_creates_layout` — mirrors
    `por_fixture_creates_single_repo_layout`.
  - `cli_init_dual_creates_layout` — mirrors
    `dual_fixture_creates_dual_repo_layout`.

**Tempdir-root override (`$VC_X1_TEST_TMPDIR`):**

Both `unique_base` helpers (in-process and CLI) resolve the parent
directory in priority order:

1. `$VC_X1_TEST_TMPDIR` (if set and non-empty).
2. `std::env::temp_dir()` (= `$TMPDIR` on Unix, else `/tmp`).

Useful for steering tests onto a tmpfs / SSD / project-local path
without exporting `TMPDIR` globally. Future work (per
`notes/todo.md`) extends the chain through
`~/.config/vc-x1/config.toml` and project-local
`.vc-config.toml`.

**Shared `resolve_tmp_root` via `#[path]`:**

`vc-x1` is binary-only (no `lib.rs`), so the integration-test
crate can't import from `src/test_helpers.rs` (which is
`#[cfg(test)]`-gated inside `main.rs`). To avoid duplicating the
~10-line resolver, lifted it into `src/test_tmp_root.rs` (pure
stdlib) and reach it from two contexts:

- `src/main.rs` — `#[cfg(test)] mod test_tmp_root;` (sibling of
  `test_helpers`); `src/test_helpers.rs` then
  `use crate::test_tmp_root::resolve_tmp_root;`.
- `tests/common/mod.rs` —
  `#[path = "../../src/test_tmp_root.rs"] mod test_tmp_root;` +
  `use test_tmp_root::resolve_tmp_root;`.

Two separate compilations of the same source — no cross-crate
linking. Constraint maintained: the file is dependency-free
(no `crate::*` imports), so the integration-test crate (whose
crate root is `tests/cli_init.rs`, not `src/main.rs`) can compile
it. When the priority chain extends to config files, the
config-loading should pass its result *in*, preserving this
constraint.

The user framed this as a precedent: **strive to share code, not
mandatory but a "good to have"**. The duplication threshold for
applying `#[path]` is judgment-driven; here, ~10 lines × 2 sites
+ expected growth made it worthwhile.

**Preserve-on-drop knob (`$VC_X1_TEST_KEEP`):**

A second env var, also resolved through `src/test_tmp_root.rs`,
suppresses RAII tempdir cleanup for debugging. When set
(non-empty), each fixture's `Drop` skips `remove_dir_all` and
prints `VC_X1_TEST_KEEP set; preserving <path>` on stderr.

```bash
VC_X1_TEST_KEEP=1 cargo test -- --nocapture 2>&1 | grep TEST_KEEP
```

(Both `2>&1` and `--nocapture` are required — `eprintln!` goes
to stderr; libtest captures stdout/stderr by default.)

Implementation:

- `src/test_tmp_root.rs` exposes `pub fn should_keep_tempdir()`
  alongside `resolve_tmp_root`. Pure-policy form `keep_decision`
  is factored out (takes `Option<&str>`) so the env-var-reading
  wrapper isn't called from unit tests — env mutation isn't
  thread-safe and would race with parallel `Drop`s.
- 3 unit tests on `keep_decision` (`unset`, `empty`, `nonempty`)
  pin the policy without touching `$VC_X1_TEST_KEEP`.
- All three RAII `Drop` impls (`Fixture`, `FixturePor`,
  `CliFixture`) consult `should_keep_tempdir()`. Same pattern in
  all three: skip removal + `eprintln!` when keep, otherwise
  best-effort `remove_dir_all`.
- New `tests/cli_keep.rs` (single-test binary) exercises the
  env-var path end-to-end: sets `VC_X1_TEST_KEEP=1` (`unsafe`
  per Rust 1.83+), creates a `CliFixture`, drops it, asserts
  the tempdir survives, manually removes, restores env. Lives
  in its own test crate so the env-write doesn't race with
  parallel reads in `cli_init` / `cli_smoke` (separate
  processes, no shared env).

**`cfg_tempdir` leak fix (`src/config.rs::tests`):**

`config.rs`'s test module had a long-standing leak: a private
`cfg_tempdir(tag) -> PathBuf` helper created
`/tmp/vc-x1-cfg-<tag>-<ts>/` and never cleaned up — every test
invocation that called it (8 tests) leaked one dir. After many
local `cargo test` runs, the user's `/tmp` had ~217 stale
`vc-x1-cfg-*` dirs.

Fixed by replacing the bare `PathBuf` helper with a `CfgTempDir`
RAII struct:

- Same shape as the other fixtures (counter + `resolve_tmp_root`
  + `should_keep_tempdir` in `Drop`).
- New naming convention `vc-x1-cfg-<tag>-<ts>-<n>` aligns with
  `vc-x1-test-*` and `vc-x1-cli-test-*`.
- `write_cfg(tag, contents)` now returns `(CfgTempDir, PathBuf)`
  — caller binds the guard to a `_cfg` slot to keep it alive
  for the test scope; existing 6 call sites updated 1:1.
- New unit test `cfg_tempdir_drop_cleans_up` pins the cleanup
  invariant: capture path, drop guard, assert path is gone.

This isn't strictly part of the -6.4 brief, but the cycle is
specifically about tempdir hygiene and the leak surfaced while
verifying `should_keep_tempdir` worked. Bringing config tests
under the shared infra (rather than letting one module quietly
leak) keeps the new precedent honest.

**README.md `## Testing` section:**

New top-level section before `## Contributing`. Documents:

- The two flavors (in-process vs CLI subprocess) and how to run
  each (`cargo test`, `--bins`, `--test cli_init`).
- The `$VC_X1_TEST_TMPDIR` env var with example usage and the
  cleanup `find` recipe for SIGKILL leaks.
- The `$VC_X1_TEST_KEEP` env var with the full
  `2>&1 | grep`/`--nocapture` incantation and the two shell
  gotchas (stderr vs stdout, libtest capture).
- Pointer to the todo for the future config-file extension.

**Test counts:**

- 331 binary unit tests (327 prior + 3 `keep_decision` +
  1 `cfg_tempdir_drop_cleans_up`).
- 5 `cli_init` (2 init tests + 3 `keep_decision` via
  `#[path]`).
- 4 `cli_keep` (1 keep-fixture test + 3 `keep_decision` via
  `#[path]`).
- 5 `cli_smoke` (2 smoke tests + 3 `keep_decision` via
  `#[path]`).
- Total: 345, all green. `keep_decision` runs in 4 contexts
  due to the `#[path]` include; each run is microseconds and
  also confirms the shared file compiles cleanly in every
  test crate.

**WIP ladder:**

Single commit. Six concerns ride together because they form
one coherent story about tempdir hygiene:

1. CLI subprocess harness (`tests/common/`).
2. First subprocess test files (`cli_smoke`, `cli_init`).
3. `$VC_X1_TEST_TMPDIR` resolution + shared `resolve_tmp_root`
   via `#[path]` — the precedent the user wanted to establish.
4. `$VC_X1_TEST_KEEP` knob + Drop wiring across all three
   fixtures + `cli_keep` end-to-end test.
5. `cfg_tempdir` leak fix using the same shared infra.
6. README + todo + chores docs.

The harness alone isn't useful without tests; the env-var
support isn't useful without docs; sharing the resolver and
keep-decision with the in-process side is the precedent
("strive to share, not mandatory but a good to have"). Fixing
`cfg_tempdir` while we're already in the area aligns the rest
of the codebase with that precedent.

### Init-lifecycle refactor: prepare/commit split + cross_ref_ochids + init_one elimination (0.41.1-6.5)

Third DRY pass on the init lifecycle. Splits `create_local_repo`
into `prepare_local_repo` + `commit_initial`, lifts step-6
cross-reference rewriting into a named helper, eliminates
`init_one` (inlined into `init_with_symlink`'s POR branch), and
moves role-specific `.vc-config.toml`/`.gitignore` writing out
of the lifecycle primitive into per-role helpers in `init.rs`.
Suite stays at 331, all green.

**`create_local_repo` → `prepare_local_repo` + `commit_initial`:**

Splits the former primitive into two sequential operations:

```
fn prepare_local_repo(target, info_label, template, name) -> Result<()>
fn commit_initial(target, info_label, ochid_strategy)     -> Result<String>
```

`prepare_local_repo` does mkdir + `git init` + `jj git init
--colocate` + optional template copy + README rewrite. The
working copy is left uncommitted so the caller can drop
role-specific files into the tree. `commit_initial` then runs
`jj commit` (with the appropriate `OchidStrategy` shape) and
returns the new initial commit's chid.

The split is what makes role-config extraction work: the caller
runs prepare → write_code_config → commit, and the config files
land in the initial commit. (See "Bug caught by the new
substep protocol" below for what happened when the seam
wasn't there.)

**Role-config helpers (in `init.rs`):**

```
fn write_por_config(dir)     -> writes VC_CONFIG_APP_ONLY + GITIGNORE_APP_ONLY
fn write_code_config(dir)    -> writes VC_CONFIG_CODE     + GITIGNORE_CODE
fn write_session_config(dir) -> writes VC_CONFIG_SESSION  + GITIGNORE_SESSION
```

Each writes both `.vc-config.toml` and `.gitignore` for its
role. They live in `init.rs` (not `repo_utils.rs`) because the
constants they reference are role-specific to vc-x1's workspace
layout — `repo_utils` stays role-agnostic.

**`cross_ref_ochids` (in `repo_utils.rs`):**

```
fn cross_ref_ochids(code_dir, code_chid, session_dir, session_chid) -> Result<()>
```

Lifted verbatim from `create_dual`'s step-6 region: rewrite both
initial commits' placeholder `ochid: /none` trailers via
`jj describe @-` once each side's chid is known. Pairs with
`OchidStrategy::Placeholder` from `commit_initial` (which writes
the placeholder) — the two together implement the dual-mode
cross-reference dance.

**`init_one` eliminated:**

The POR-branch wrapper inlined into `init_with_symlink`. After
the configs/gitignores split out into `write_por_config` and
the lifecycle split into prepare + commit, `init_one`'s body
collapsed to ~25 lines of straight-line composition — no
per-side abstraction left to justify a separate function.
Replaced the dispatch `if is_dual { create_dual } else {
init_one }` with `if is_dual { return create_dual(...); }`
plus the inlined POR body.

`create_dual` survives as a separate function (its body is
larger and the unwrap-extract-then-orchestrate shape is a
distinct unit).

**`create_dual` cleanup:**

Stale "Step N" inline comments dropped — the function names
(`prepare_local_repo`, `cross_ref_ochids`, `push_repo`) now
narrate the lifecycle on their own. Doc comment rewritten as a
bulleted composition list referencing functions, not step
numbers (step numbers shift if the lifecycle reorders, function
names don't).

The `info!()` lines inside `create_local_repo` and `push_repo`
still carry `"Step N: ..."` labels — those land in -6.7 (single-
word `label: body` convention).

**Bug caught by the new substep protocol:**

Substep (1) extracted role-config writing out of
`create_local_repo` into post-call helpers. That left
`.vc-config.toml`/`.gitignore` written *after* `jj commit`
finalized the initial commit — i.e., they sat in the new empty
working copy as uncommitted files. `push_repo`'s subsequent
`git clean -xdf` would have wiped them entirely, and the
`push_happy_claude_clean` integration test caught the related
symptom (`.claude main` advanced because the session side had
pending config files at push time).

Resolved by substep (5): split the lifecycle (`prepare_local_repo`
+ `commit_initial`) and reorder callers to write configs
between the two. The seam now exists where role-config writing
needs to land — between prepare and commit.

Lesson logged for the substep protocol: run `cargo test --bins`
after each substep, not only at close-out — would have caught
the regression at (1) instead of (4).

**Substep protocol — first use:**

This cycle was the first to use the per-substep `@` workflow:
each substep got its own `jj new` working copy, with the chain
squashed into a single commit at close-out via
`jj squash --from "@---..@" --into @---`. Wins on bisection
(two `jj edit` jumps localized the (1)→(5) regression) and
per-substep diff cleanliness. Frictions: missing
per-substep `cargo test --bins` (above), and squash mechanics
that are worth jotting in CLAUDE.md if the protocol is adopted
formally.

**WIP ladder:**

Five substeps, squashed at close-out:

1. Drop `config`/`gitignore` params from `create_local_repo`;
   add `write_{por,code,session}_config` helpers in `init.rs`.
2. Extract `cross_ref_ochids` into `repo_utils.rs`.
3. Eliminate `init_one` — inline into `init_with_symlink`'s POR
   branch.
4. `create_dual` collapse — drop stale `Step N` comments,
   tighten doc.
5. Fix: split `create_local_repo` into `prepare_local_repo` +
   `commit_initial` (regression from (1) — role-config was
   landing after the initial commit instead of in it).

### --config option + create_por extraction (0.41.1-6.6)

Pre-impl design. Two concerns in one substep ladder:

1. Extract `create_por()` from `fn init`'s inline POR body
   (currently lines ~1177-1216 of `src/init.rs`), mirroring
   `create_dual()` as a sibling creation-time orchestrator.
2. Add a reusable `--config` flag in a new module
   `src/options_flags/config.rs` that gates POR's
   `.vc-config.toml` write. New `src/options_flags/` directory
   collects per-flag modules; `src/` is getting large.

#### create_por extraction

`fn init`'s POR branch is currently inline straight-line
composition. Lift verbatim into:

```rust
fn create_por(
    args: &InitArgs,
    plan: &InitPlan,
    templates: Option<(PathBuf, Option<PathBuf>)>,
    visibility: &str,
    _create_symlink: bool,
) -> Result<(), Box<dyn std::error::Error>>
```

- Signature mirrors `create_dual` for shape-symmetry. The
  `create_symlink` parameter is unused in POR (no symlink in
  single-repo); kept for now so the dispatcher stays trivial.
- Composes (in order, parallel to `create_dual`'s doc list):
  - `prepare_local_repo` → conditional config write (see
    below) → `commit_initial` (`OchidStrategy::None`).
  - `push_repo` for code side (no `clean_exclude`).
  - No cross-reference, no session, no symlink.

Dispatcher in `fn init` — exhaustive `match` on
`args.scope`:

```rust
match args.scope {
    ScopeKind::CodeBot => return create_dual(args, &plan, templates, visibility, create_symlink),
    ScopeKind::Por     => return create_por(args, &plan, templates, visibility, create_symlink),
}
```

`is_dual` (computed earlier in `fn init` for the dry-run/print
branch) stays in place for that branch; only the final dispatch
line changes from the existing `if is_dual { … }` to the
`match`. Exhaustive `match` means future `ScopeKind` variants
won't compile until handled here.

#### --config flag (reusable)

New module `src/options_flags/config.rs`:

```rust
pub enum ConfigKind {
    None,
    Path(PathBuf),
}

pub fn parse_config_kind(s: &str, default: ConfigKind) -> ConfigKind;
```

`src/options_flags/mod.rs` only re-exports the submodule (`pub
mod config;`); use sites import `crate::options_flags::config::
{ConfigKind, parse_config_kind}` so future flag modules can
share the namespace without name collisions.

`src/main.rs` adds `mod options_flags;` alongside the existing
`mod args;`.

Parser (match-based, infallible):

- `""` → `default` (caller-supplied — each consumer plugs in
  its own canonical canned shape).
- `"none"` → `ConfigKind::None`.
- Anything else → `ConfigKind::Path(s.into())`.

The default lives at the call site, not in the enum. Init's POR
passes a default that resolves to "write the canned
`VC_CONFIG_APP_ONLY`" (substep-3 wiring decides the exact
shape — likely a marker variant the consumer interprets, or a
synthetic `Path` to a temp file).

No path-prefix discipline (`./`, `~/`, etc.) — `--config` has
only one keyword (`none`), so any other string is unambiguously
a path. Parser returns `ConfigKind` directly (no `Result`)
because empty input has a defined fallback rather than an error.

`InitArgs`:

```rust
/// Override the default `.vc-config.toml` write (POR only).
///
/// - Absent: write the canned single-repo `.vc-config.toml`.
/// - `--config none`: skip writing `.vc-config.toml` entirely.
/// - `--config <path>`: copy `<path>` to `.vc-config.toml`
///   (bytewise; no schema validation).
///
/// Only valid with `--scope=por`. `.gitignore` is always
/// written regardless of `--config`.
#[arg(long, value_name = "none|PATH", verbatim_doc_comment)]
pub config: Option<String>,
```

Raw `Option<String>` at the arg layer; the consumer calls
`parse_config_kind(s, default)` itself. This keeps Option A's
caller-supplied default available without forcing a static
closure adapter into clap's `value_parser` slot.

Optional on the command line so scripts and bots can be
explicit while interactive use stays lazy.

#### Preflight checks (in `plan_init`)

- `args.config.is_some()` with `--scope=code,bot` → error:
  `--config is only valid with --scope=por (dual-mode configs
  are per-side and unconditional)`.
- `parse_config_kind(s, ConfigKind::None) → Path(p)` where `p`
  doesn't exist → error: `--config: path does not exist: <p>`.
  (Existence check only — readability surfaces later as a
  copy-time error with a clear `failed to copy <src>` message.)

#### Conditional write in `create_por`

```rust
match args.config.as_deref() {
    None => write_por_vc_config(&plan.project_dir)?,
    Some(s) => match parse_config_kind(s, ConfigKind::None) {
        ConfigKind::None    => {} // skip — user asked us not to write
        ConfigKind::Path(p) => copy_user_config(&p, &plan.project_dir)?,
    },
}
write_por_gitignore(&plan.project_dir)?;
```

Resolved at impl time (substep 3):

- **No tempfile / sentinel for the canned default.** The
  consumer dispatches on `args.config.as_deref()` directly:
  `None` → canned write, `Some(s)` → parser. The parser's
  `default` parameter is passed `ConfigKind::None` and is only
  reachable if the user passes `--config ""` explicitly (in
  which case "skip" is a reasonable interpretation). This
  avoids both the tempfile dance and the sentinel-Path hack.
- **`write_por_config` split (option (a) from the open
  question).** New helpers `write_por_vc_config` (canned
  config) and `write_por_gitignore` (canned gitignore). The
  latter is called unconditionally after the conditional
  config write. Symmetric with the existing
  `write_code_config` / `write_session_config` shape.
- **`copy_user_config` lives in `init.rs`.** Sits next to
  `write_por_vc_config` / `write_por_gitignore`. Bytewise
  `std::fs::copy` (no TOML parse, no validation). Malformed
  user content surfaces later via `find_workspace_root` /
  config readers. No new module needed.

#### WIP ladder (squashed at close-out)

Three substeps:

1. Extract `create_por()` and convert dispatcher to `match` —
   verbatim lift; no behavior change.
2. Create `src/options_flags/{mod,config}.rs` with `ConfigKind`
   + `parse_config_kind` and unit tests; wire `mod
   options_flags;` into `src/main.rs`; no consumers.
3. Wire `--config` into `InitArgs`; conditional write in
   `create_por`; preflight + integration tests.

#### Decisions made during design

- **`Default` not a `ConfigKind` variant.** Absence-of-flag is
  encoded by passing the caller's default to
  `parse_config_kind` (Option A — see "Parser" above). The
  default lives at the consumer call site, not in the enum,
  so each command can plug in its own canonical canned shape
  without baking command-specific knowledge into the type.
- **Parser is infallible.** Empty input has a defined fallback
  (the `default` parameter), so no `Result` return is needed
  — return `ConfigKind` directly. `parse_scope_kind` returns
  `Result` because every input shape is a definite parse —
  there's no "give me a default" fallback shape there.
- **`--config` only valid with `--scope=por` for now.** Dual
  mode's per-side configs (`write_code_config` /
  `write_session_config`) are out of scope for -6.6; revisit
  when a use case surfaces.
- **Path validation: existence + readable, no TOML parse.**
  Trust user-provided files; downstream readers surface
  malformed content. Preflight stays cheap.
- **`Path(_)` accepts any non-`none` string.** Single keyword,
  no `./` discipline needed.
- **`create_symlink` parameter retained on `create_por`.** Kept
  in the signature for shape-symmetry with `create_dual`;
  ignored in the body. Drop in a follow-up if it accumulates
  warnings.
- **Dispatcher uses exhaustive `match` on `args.scope`.** Future
  `ScopeKind` variants force a compile error here until handled
  — preferable to `if is_dual { … } else { … }`, which silently
  routes any new variant to the POR arm.
- **`src/options_flags/` directory — default home for all flag
  types/parsers.** Each flag's typed value + value parser lives
  here, regardless of how many subcommands consume it today.
  Two reasons: (1) any flag may pick up additional consumers as
  the CLI grows, and (2) the directory is shaped to be liftable
  into its own crate for cross-project reuse later. `ScopeKind`
  / `parse_scope_kind` currently lives in `src/args.rs` for
  historical reasons; relocating to
  `src/options_flags/scope.rs` is a deferred consistency
  cleanup (not in -6.6).

### OF refactor (0.41.1-6.7)

Sub-sub-step ladder (1)–(8) lifts every init OF into
`src/options_flags/`, introduces the leaf / bundle / Pattern-A
composition patterns, and locks the Flag/Option taxonomy by
value-domain. Full per-substep edit lists live in commit
bodies; this section captures the close-out shape, the
sub-sub-step ToC, and durable decisions / watch-items.

#### Close-out shape — separate commits, no mid-cycle squash

Wink and the bot chose at close-out to **keep the 8
sub-sub-step commits separate** rather than squash them into
one `-6.7` commit (the prior pattern for `-6.0` through
`-6.6`). Reasoning: the decomposition is itself an artifact —
readers see leaf → bundle → naming progression; bisect lands
on a specific stage; commit bodies double as design history.
Squashing erases that.

This sets a precedent: future steps may land as N commits when
the decomposition is informative. The default for focused
single-direction work is still one commit per step.

The close-out commit at version `0.41.1-6.7` (no `-N` suffix)
is itself a 9th commit holding the docs fold (substep-style →
CLAUDE.md), in-flight pointer drop, this section, and todo.md
flips. Total app-side: 9 commits for `-6.7`.

#### Sub-sub-step ToC

| commit | scope |
|---|---|
| `0.41.1-6.7-1` | cycle setup — Cargo bump + (1)–(7) ladder in todo.md |
| `0.41.1-6.7-2` | `ConfigFlag` leaf + leaf/bundle/Pattern-A README; `FlagBundle` + `FlagParser` markers; init flattens `ConfigFlag` |
| `0.41.1-6.7-3` | `DryRunFlag` + `PrivateFlag` leaves; init flatten |
| `0.41.1-6.7-4` | `PushRetryFlags` leaf; `run_retry` takes `&PushRetryFlags`; multi-field-leaf-as-ref convention |
| `0.41.1-6.7-5` | `UseTemplateFlag` + `AccountFlag` leaves; init flatten; substep-style.md authored |
| `0.41.1-6.7-6` | `ScopeFlag` + `RepoFlag` leaves; `args.rs` retired |
| `0.41.1-6.7-7` | `ProvisionCommon` bundle (DryRun + Private + PushRetry) — first OF bundle |
| `0.41.1-6.7-8` | Flag/Option naming pass — three Bundle markers + two Parser contracts; rename 6 leaves `*Flag → *Option`; `ProvisionCommon → ProvisionOptionFlagBundle` |

#### Naming watch — `ProvisionOptionFlagBundle`

The first OF bundle landed as `ProvisionOptionFlagBundle` (25
chars). Iteration considered `ProvisionCommon`,
`ProvisionOptFlagBundle`, `ProvisionOptionFlag`,
`ProvisionOFB`, and `ProvisionOptFlgBdle`. The full-word form
won on consistency (matches `OptionParser`, `OptionBundle`,
`*Option` leaves elsewhere), but Wink flagged at lock-in: "my
guess we'll end up with something shorter in the future, just
guessing."

- **Why kept on the watch list:** the type appears in
  `init.rs` imports, `test_helpers.rs` fixtures, and any
  future provisioning subcommand (clone, etc.). Length will
  be felt as more bundles land and more sites reference them.
- **If a future cycle revisits naming:** the previously
  rejected fallback was `ProvisionOptionFlag` (19 chars,
  drops `Bundle` suffix; cue then lives in the file name and
  marker impl, not the type name). Abbreviations
  (`Bdle` / `Flg` / `OFB`) were explicitly rejected as
  decoder-ring problems — don't re-propose them.

### Init fn collapse (0.41.1-6.8)

Single-edit sub-step. Merges the 1-line `init` wrapper into
the renamed `init_with_symlink` body, leaving one public
function `pub fn init(args, create_symlink) -> Result<…>`.

- `src/init.rs`: drop the `init` wrapper; rename
  `init_with_symlink` → `init`; relax visibility to `pub`;
  fold the two doc comments into one.
- `src/main.rs`: `init::init(&init_args)` → `init::init(&init_args, true)`.
- `src/test_helpers.rs`: import + two call sites
  (`init_with_symlink(&args, false)` → `init(&args, false)`)
  + module / `Fixture` / `FixturePor` doc comments.
- `src/init.rs:2329` + `:2480`: stale test-section banner
  refs (the section banners still rightly name `init`, just
  with the new function name).
- `src/sync.rs:848`, `src/repo_utils.rs:224`,
  `tests/cli_init.rs:5`: cross-references to `init::init`
  / `init`/`create_dual` removed entirely. The reader of a
  sync test or a `repo_utils` test doesn't need a tour
  through `init.rs` to understand the contract under test —
  these refs were implementation-detail leaks. README.md
  keeps the `init::init` name as a where-to-look pointer
  for new readers but drops the `create_symlink` aside
  (the symlink behavior is documented at the function
  itself).

**Why a function parameter, not an `InitArgs` field.**
`create_symlink` is a test-harness suppression toggle, not a
CLI option. Adding it to `InitArgs` would either leak a
`#[arg(skip)]` test-only field into the user-facing arg
struct or surface a flag in `--help` that has no user
meaning. A plain function parameter keeps the layer
separation honest: the CLI surface (`InitArgs` + clap
derives) describes user-facing inputs; the
`create_symlink` parameter encodes the call-site contract
(CLI passes `true`, tests pass `false`).

**Why drop the dual entry point.** Pre-collapse, callers
chose between `init(args)` (CLI shorthand, hardcoded `true`)
and `init_with_symlink(args, bool)` (tests). Two functions
for one operation reads as ceremony — main.rs gains one
literal `true` and stops paying for the wrapper.

**Dual/Por split parked as -6.9.** Discussion around -6.8
also surfaced the question of whether the planned
`InitDualArgs` / `InitPorArgs` clap split is worth doing.
Independent question from the symlink toggle. Parked to
`-6.9` and may not happen — revisit at -6.9 design time.

### Cycle continuation



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
  anchor (code side). See
  [`forks-multi-user.md`](../forks-multi-user.md) for the full
  discussion.

  **Correction at close-out (0.41.1-6.7):** the bot-side
  partner bookmark turned out to be unused — `.claude` is a
  linear journal that only needs `main`, and all 8
  sub-sub-step `.claude` commits + the close-out landed on
  `.claude main` regardless of the partner bookmark's
  position. The partner bookmark misled the bot during
  close-out push (it pushed app-side `main` instead of
  `init-clone-refactor` because the convention that "the
  bookmark to push is `main`" was carried over from
  single-branch projects). The convention is now codified in
  `CLAUDE.md > ## Sub-step Workflow > .claude cadence`:
  `.claude` does not maintain partner bookmarks for app-side
  branches. App-side keeps feature branches; `.claude` stays
  on `main`.

### Cycle close-out (0.41.1)

`0.41.1` ships the init + clone redesign — `<TARGET> [NAME]
+ --scope` shape across both subcommands, user-config (account
/ repo / scope / push-retry / use-template / config flags
lifted into `options_flags/`), `Provisioner` trichotomy
(`GhCreate` / `LocalBareInit` / `ExternalPreExisting`), and
the init refactor with extracted `prepare_local_repo` /
`commit_initial` / `cross_ref_ochids` / `push_repo` helpers.
The init function shape stabilized at `pub fn init(args,
create_symlink)` in `-6.8`.

**Deferred to follow-on cycles** (moved to `notes/todo.md` >
`## Todo`):

- **Symmetric `.vc-config.toml` schema** — the second half of
  `0.41.1-6`'s original title. Direction sketched:
  workspace-root-relative `code = "/"` and `bot = "/.claude"`
  entries readable from either repo, with side detection
  walking up to find workspace root via `find_workspace_root`
  then mapping cwd-relative-to-root onto the configured
  paths. Real design + implementation work, not a close-out
  tidy-up; deferred so the close-out doesn't become a
  squeezed-in redesign.
- **`test_helpers::Fixture` migration + downstream callers**
  (was `0.41.1-7`) — folds in `Fixture` →
  `TestFixtureDual` / `FixturePor` → `TestFixturePor`
  rename. Mechanical and doesn't depend on this branch's
  context, so safe to defer past the merge.
- **init dual|por arg split** (was `0.41.1-6.9`) — via
  `#[command(flatten)]` of `ProvisionOptionFlagBundle` +
  `provision_side(role, …)` shared helper. Parked as
  "may or may not happen"; the `-6.8` discussion identified
  this as orthogonal to the init function-shape work. Revisit
  at design time if/when CLI rework warrants it.

**Cycle shape on close-out.** Top-level steps `-0` through
`-6` shipped; the `-6` step itself decomposed into nine
sub-substeps (`-6.0` through `-6.8`) plus an inner `-6.7`
close-out commit that kept its own eight sub-sub-step
commits separate (decomposition was informative for the OF
refactor). The 0.41.1 close-out is a single commit on top
of that chain; whether to squash any of the underlying
0.41.1 commits at merge time is a separate decision (see
"Next").

**Next.** Rebase `main` (carrying 0.42.0-0 through
0.42.0-4.6) onto `init-clone-refactor`, so 0.41.1 lands
first on `main` and 0.42.0's commits replay on top — the
direction the rebase note in `notes/todo.md` calls for
("0.42.0 rebases on top of 0.41.1 at close-out"). Conflict
surface is identified in
`notes/init-clone-refactor-conflict.md`; key overlap files
are `src/init.rs`, `src/scope.rs`, `src/sync.rs`,
`CLAUDE.md`, and `notes/todo.md`. Squash strategy and
pre-resolution of the docs conflicts (CLAUDE.md, todo.md)
to be decided before the rebase actually runs.

# References

[1]: ../forks-multi-user.md
