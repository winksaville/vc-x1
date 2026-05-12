# Architecture

How the `vc-x1` binary is structured: the clap-aware CLI
layer and the (emerging) clap-free subcommand layer.

This is a living document; the app is under continuous
development and the repo will keep changing. Per-cycle design
discussion lives in `notes/chores-*.md`; near-term tasks in
`notes/todo.md`.

## Overview

`vc-x1` is a single binary that supports subcommands,
`vc-x1 <subcommand>`, for managing a jj-git **dual-repo workspace**
with an app repo plus a `.claude` bot-session repo. The commits
cross-reference each other via `ochid:` trailers at the bottom
of each commit message. See [`README.md`](README.md)
for the user-facing picture and [`CLAUDE.md`](CLAUDE.md) for
the bot workflow.

## Two layers: CLI args vs subcommand Context + Params

Goal: separate clap-aware argument parsing from subcommand
operation logic, so a future front-end (TUI, library
embedding) can call the same core without dragging clap
along. The trigger was `args.account.account`-style
nested-leaf access leaking into subcommand bodies; accessor
shortcuts were rejected as hiding the mismatch rather than
fixing it.

- **CLI layer (clap-aware)**:
  Builds `XxxParams` from the user-facing `XxxArgs`
  `#[derive(Args)]` clap struct. Options/flags used in more
  than one subcommand live as flattened leaves in
  `src/options_flags/`.
- **Subcommand layer (clap-free)**:
  Entrypoint shape: `fn x(ctx: &Context, params: &XxxParams) -> Result<…>`.
  And this isolates the subcommand from user and clap.
  - `Context` — common state shared with all subcommands.
  - `XxxParams` — a subcommand's parameters, derived from
    config or command-line arguments.
- **Boundary conversion**:
  Converting `XxxArgs` to `XxxParams` uses either
  `impl From<&XxxArgs> for XxxParams` or
  `impl TryFrom<&XxxArgs> for XxxParams`, depending on whether
  the conversion can fail.

Two parameters per subcommand; the signature documents what
the subcommand depends on. `Context` is the same across all
subcommands; `XxxParams` carry the per-subcommand inputs.

Rules for the subcommand layer (some still aspirational — see
[Migration A](#migration-a--args--context--params)):

1. Plain options, flat fields, `Default` — no clap types in
   `XxxParams`. Domain types (`RepoSelector`, `Scope`) are
   fine; leaf wrappers (`RepoOption`) are not.
2. Typed errors (`enum XError`, not `Box<dyn Error>`) — a
   TUI matches variants to pick dialogs; CLI formats them.
3. Returned outcomes, not `println!` — each subcommand
   returns a structured result; CLI formats it, a library
   writes nothing to stdout itself.
4. Progress via an optional `&mut dyn ProgressSink` on
   long-running subcommands — CLI installs a stderr sink, tests a
   recording sink.
5. No globals, no implicit cwd/env reads in subcommands — CLI
   resolves cwd/env once at startup and builds `Context`.

None of this is sacred — the whole structure is an experiment
and the repo keeps changing. The current scope is deliberately
narrow:

- a small `Context` — two params, not one god blob;
- no trait-based DI yet;
- one crate, not a workspace split;
- `src/options_flags/` leaves left in place — only their
  consumers change shape.

Widen any of it when the evidence (a second front-end, a second
consumer crate, a real platform-surface divergence) calls for
it — not before.

### Naming

Earlier drafts of this design (and the `notes/chores-09.md`
capture) used `Workspace` / `XOptions`. The implementation
(0.44.0) chose `Context` / `XxxParams` instead:

- `Workspace` → `Context` — Cargo owns "workspace" formally,
  and this codebase already uses "workspace" for the
  dual-repo project root (`find_workspace_root`).
- `XOptions` → `XxxParams` — avoids visual collision with
  `Option<T>` (a params struct mixes required and
  `Option<T>`-typed fields) and with `src/options_flags/`.

## Module map

CLI layer:

- `src/main.rs` — the clap edge:
  - `pub struct Cli` — clap `Parser`.
  - `pub enum Commands` — clap `Subcommand`; matched in the
    dispatch arms.
  - `cli_with_banner()` — walks the command tree adding the
    `vc-x1 X.Y.Z` banner as `before_help` on every subcommand.
  - `CompleteEnv` wiring for dynamic completion.
  - `bm_track` — a permanent `main`-bookmark tracking
    diagnostic printed on entry/exit of every command.

- `src/options_flags/` — options/flags shared across
  subcommands, so a flag is defined once (parser, help,
  completer); Migration B tracks what's been moved here so
  far. See
  [`src/options_flags/README.md`](src/options_flags/README.md).
  - reusable **leaves**: `account`, `commit_limit`, `config`,
    `dry_run`, `private`, `push_retry`, `repo`, `repo_label`,
    `revision`, `scope`, `squash`, `use_template`.
  - reusable **bundles**: `common_bundle`
    (`CommonOptionFlagBundle` — the read-only commit-query OFs,
    flattened by `chid` / `desc` / `list` / `show`),
    `provision_bundle`.
- `src/common.rs` — the repo-iteration and revision-resolution
  helpers the read-only commit-query subcommands share:
  `for_each_repo`, `collect_ids`, `resolve_revset`, `DotSpec` /
  `parse_dot_rev` / `resolve_spec` (`..` notation + count
  reconciliation), `Header` / `resolve_header`, the `format_*`
  printers, plus `find_workspace_root` / `default_scope` /
  `scope_to_repos`. Also still holds `CommonArgs` — being
  retired: the 0.49.0-1 sub-steps move the four read-only
  subcommands onto `options_flags::common_bundle::CommonOptionFlagBundle`
  one at a time (`chid` at 0.49.0-1.1), and `CommonArgs` is
  removed once the last lands; 0.49.0-2 then swaps `-R` for
  `-s`/`--scope`.

Subcommand layer scaffolding:

- `src/context.rs` — `Context`: the shared platform handle
  every subcommand runs against. Today it carries the loaded
  `UserConfig` and the `--log` path; built once at startup
  via `Context::load(log)`.
- `src/config.rs` — `UserConfig` (`~/.config/vc-x1/config.toml`).
- `src/scope.rs` — `Scope` enum (`Roles(Vec<Side>)` for the
  dual-repo `code`/`bot` roles, `Single(PathBuf)` for
  single-repo mode) and `parse_scope`. `parse_scope` /
  `Scope::Single` are `#[allow(dead_code)]` until 0.49.0-2
  wires `-s`/`--scope` into the read-only commit-query commands.

Subcommand modules — each holds an `XxxArgs` `#[derive(Args)]`
struct and a `pub fn x(...)` entrypoint:

| Module | Subcommand | Notes |
| --- | --- | --- |
| `chid.rs` | `chid` | flattens `options_flags::common_bundle::CommonOptionFlagBundle` (0.49.0-1.1; Migration A → 0.49.0-3) |
| `desc.rs` (+ `desc_helpers.rs`) | `desc` | flattens `common::CommonArgs` → `CommonOptionFlagBundle` in 0.49.0-1.2; Migration A → 0.49.0-4 |
| `list.rs` | `list` | flattens `common::CommonArgs` + `-w`/`--width` → bundle in 0.49.0-1.3; Migration A → 0.49.0-5 |
| `show.rs` | `show` | flattens `common::CommonArgs` + `-f`/`--files` → bundle in 0.49.0-1.4; Migration A → 0.49.0-6 |
| `validate_desc.rs` | `validate-desc` | migrated (0.48.0-4) — `validate_desc(&Context, &ValidateDescParams)`, `From<&ValidateDescArgs>` |
| `fix_desc.rs` | `fix-desc` | migrated (0.48.0-5) — `fix_desc(&Context, &FixDescParams)`, `From<&FixDescArgs>` |
| `clone.rs` | `clone` | migrated (0.48.0-2) — `clone_repo(&Context, &CloneParams)`, `From<&CloneArgs>` |
| `init.rs` (+ `init/params.rs`) | `init` | subcommand-layer worked example (0.44.0) |
| `symlink.rs` | `symlink` | migrated (0.48.0-1) — `symlink(&Context, &SymlinkParams)`, `From<&SymlinkArgs>` |
| `sync.rs` | `sync` | migrated (0.48.0-3) — `sync(&Context, &SyncParams)`, `From<&SyncArgs>` |
| `finalize.rs` | `finalize` | migrated (0.46.0) — `finalize(&Context, &FinalizeParams)`, `TryFrom<&FinalizeArgs>` |
| `push.rs` | `push` | resumable state machine; migrated (0.48.0-6) — `push(&Context, &PushParams)`, `From<&PushArgs>` |

Support:

- `src/repo_utils.rs`, `src/url.rs`, `src/toml_simple.rs`,
  `src/logging.rs` — jj/repo helpers, URL parsing, a minimal
  TOML reader, the CLI logger.
- `src/test_helpers.rs`, `src/test_tmp_root.rs` —
  `#[cfg(test)]` fixtures (`Fixture` / `FixturePor`) and
  tempdir resolution. Subcommand modules also carry their
  own `#[cfg(test)] mod tests` (and `integration_tests` for
  `sync` / `push`).

## Subcommand model

Adding a subcommand `x`:

1. Create `src/x.rs` with `pub struct XxxArgs`
   (`#[derive(Args)]`), composing leaves from
   `src/options_flags/` where one exists.
2. Add a variant `X(x::XxxArgs)` to `Commands` in
   `src/main.rs` with a `///` summary (and
   `#[command(long_about = …)]` for the long form).
3. Add a dispatch arm in `main()` calling `x::x(&args)` (or,
   once ported, `x::x(&ctx, &params)`).
4. Doc comments on `#[arg(...)]` fields drive `--help`; add
   `#[arg(verbatim_doc_comment, …)]` on any field whose doc
   comment uses bullets (clap otherwise reflows them into
   prose).
5. Dynamic value completion (`--account=<TAB>`) is an
   `ArgValueCompleter` attached to the leaf in
   `src/options_flags/` — completion is a clap-aware concern
   and stays at this layer.

## Migration A — args → Context + Params

Port each subcommand's `pub fn x(args: &XxxArgs)` to
`pub fn x(ctx: &Context, params: &XxxParams)`, adding an
`XxxParams` flat struct + a `From` (or `TryFrom`, if the
conversion is fallible) at the binary edge. `init` (0.44.0) is
the worked example; `finalize` (0.46.0) added the `TryFrom` /
`Context.log` variant; the `0.48.0` cycle swept the standalone
subcommands; the `0.49.0` cycle is the "CommonArgs sweep" for
the four `CommonArgs`-flattening ones — first Migration B
(decompose `CommonArgs` → `options_flags/` leaf-bundle, -1) and
the `--scope` rollout (`-R`/`--repo` → `-s`/`--scope`, -2), then
Migration A against the final shape (`chid` -3, `desc` -4,
`list` -5, `show` -6; these embed a shared clap-free
`CommonParams`, and all four take a fallible `TryFrom<&XxxArgs>`
because scope→repo resolution can error). See the Migration B
section below and `notes/chores-09.md`. The live checklist is
the "Subcommand layer / CLI decoupling" item in
[`notes/todo.md`](notes/todo.md).

| Subcommand | Status |
| --- | --- |
| `init` | done (0.44.0) — `init(&Context, &InitParams)`; `From<&InitArgs>` |
| `finalize` | done (0.46.0) — `finalize(&Context, &FinalizeParams)`; `TryFrom<&FinalizeArgs>` (fallible boundary); `--log` moved onto `Context` |
| `symlink` | done (0.48.0-1) — `symlink(&Context, &SymlinkParams)`; `From<&SymlinkArgs>` |
| `clone` | done (0.48.0-2) — `clone_repo(&Context, &CloneParams)`; `From<&CloneArgs>` |
| `sync` | done (0.48.0-3) — `sync(&Context, &SyncParams)`; `From<&SyncArgs>` |
| `validate-desc` | done (0.48.0-4) — `validate_desc(&Context, &ValidateDescParams)`; `From<&ValidateDescArgs>` |
| `fix-desc` | done (0.48.0-5) — `fix_desc(&Context, &FixDescParams)`; `From<&FixDescArgs>` |
| `push` | done (0.48.0-6) — `push(&Context, &PushParams)`; `From<&PushArgs>` (collapses the two bookmark spellings) |
| `chid` | Migration A pending (0.49.0-3) — the 0.49.0 CommonArgs sweep; introduces the shared `CommonParams` |
| `desc` | Migration A pending (0.49.0-4) — the 0.49.0 CommonArgs sweep |
| `list` | Migration A pending (0.49.0-5) — the 0.49.0 CommonArgs sweep |
| `show` | Migration A pending (0.49.0-6) — the 0.49.0 CommonArgs sweep (`TryFrom`, `FileLimit` parse) |

Out of scope for these ports (deferred until a real consumer
surfaces): typed errors, returned outcomes vs `println!`, the
`ProgressSink`, and `Context` fields beyond `UserConfig` + the
`--log` path (`finalize` surfaced `log`).

## Migration B — per-subcommand flags → `src/options_flags/`

Independently of Migration A, per-subcommand inline `#[arg]`
fields are being lifted into reusable leaves/bundles under
`src/options_flags/` so a flag is defined (parser, help,
completer) exactly once. See
[`src/options_flags/README.md`](src/options_flags/README.md)
for the leaf / bundle / Pattern-A mechanics and the
Flag-vs-Option classification.

State today:

- **`init`** — fully composed from leaves/bundles
  (`account`, `repo`, `scope`, the `provision` bundle,
  `use_template`, `config`).
- **`chid` / `desc` / `list` / `show`** — being migrated off
  `common::CommonArgs` onto
  `options_flags::common_bundle::CommonOptionFlagBundle` (the
  0.49.0 "CommonArgs sweep" cycle, one subcommand per sub-step:
  `chid` 0.49.0-1.1, `desc` -1.2, `list` -1.3, `show` -1.4 —
  `CommonArgs` removed when the last lands). The bundle is
  composed of the `revision` and `commit_limit` leaves, the
  `repo_label` leaf (the `-l`/`-L` pair), plus the two
  positionals (`REVISION` / `COMMITS`) and the `-R`/`--repo`
  list inline; `-R`/`--repo` is then swapped for a `--scope`
  leaf (`code|bot|code,bot|<path>` — wiring up `scope.rs`'s
  `parse_scope` + `Scope::Single`) in 0.49.0-2; the positionals
  stay inline. (Two of the leaves — `RevisionOption`,
  `CommitLimitOption` — both use the `value` field-name
  convention, so each carries an explicit `#[arg(id = …)]` to
  keep clap arg ids unique when flattened into the same struct.)
- **`finalize`** — `--squash` lifted to the `squash` leaf
  (which carries the `value`-field naming convention — see
  [`src/options_flags/README.md`](src/options_flags/README.md)
  "Adding a new leaf"). `--delay` / `--detach` / `--exec` /
  `--repo` / `--push` stay inline (no second consumer yet).
- **`sync` / `push` / `clone` / `validate-desc` /
  `fix-desc` / `symlink`** — still mostly inline `#[arg]`
  fields. The `--scope` retrofits queued in
  [`notes/todo.md`](notes/todo.md) are the usual entry
  point for converting one.

## See also

- [`README.md`](README.md) — user-facing overview and
  per-subcommand usage.
- [`CLAUDE.md`](CLAUDE.md) — bot workflow, versioning,
  commit/push conventions, code conventions.
- [`src/options_flags/README.md`](src/options_flags/README.md)
  — leaf/bundle patterns for Migration B.
- [`notes/todo.md`](notes/todo.md) — live task list,
  including both migrations.
- [`notes/chores-09.md`](notes/chores-09.md) — the
  0.43.0 / 0.44.0 design capture this document supersedes.
