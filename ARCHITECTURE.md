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
  - reusable **leaves**: `account`, `config`, `dry_run`,
    `private`, `push_retry`, `repo`, `scope`, `squash`,
    `use_template`.
  - reusable **bundles**: `provision_bundle`.
- `src/common.rs` — `CommonArgs` (the shared positional-rev /
  `-R` repo-list / `-n` / `--limit` / `-L` flag set flattened
  by `chid`, `desc`, `list`, `show`) plus the repo-iteration
  and revision-resolution helpers they share.

Subcommand layer scaffolding:

- `src/context.rs` — `Context`: the shared platform handle
  every subcommand runs against. Today it carries the loaded
  `UserConfig` and the `--log` path; built once at startup
  via `Context::load(log)`.
- `src/config.rs` — `UserConfig` (`~/.config/vc-x1/config.toml`).
- `src/scope.rs` — `Scope` enum (`Roles(Vec<Side>)` for the
  dual-repo `code`/`bot` roles, `Single(PathBuf)` for
  single-repo mode) and `parse_scope`.

Subcommand modules — each holds an `XxxArgs` `#[derive(Args)]`
struct and a `pub fn x(...)` entrypoint:

| Module | Subcommand | Notes |
| --- | --- | --- |
| `chid.rs` | `chid` | flattens `CommonArgs` |
| `desc.rs` (+ `desc_helpers.rs`) | `desc` | flattens `CommonArgs` |
| `list.rs` | `list` | flattens `CommonArgs` |
| `show.rs` | `show` | flattens `CommonArgs` |
| `validate_desc.rs` | `validate-desc` | migrated (0.48.0-4) — `validate_desc(&Context, &ValidateDescParams)`, `From<&ValidateDescArgs>` |
| `fix_desc.rs` | `fix-desc` | |
| `clone.rs` | `clone` | migrated (0.48.0-2) — `clone_repo(&Context, &CloneParams)`, `From<&CloneArgs>` |
| `init.rs` (+ `init/params.rs`) | `init` | subcommand-layer worked example (0.44.0) |
| `symlink.rs` | `symlink` | migrated (0.48.0-1) — `symlink(&Context, &SymlinkParams)`, `From<&SymlinkArgs>` |
| `sync.rs` | `sync` | migrated (0.48.0-3) — `sync(&Context, &SyncParams)`, `From<&SyncArgs>` |
| `finalize.rs` | `finalize` | migrated (0.46.0) — `finalize(&Context, &FinalizeParams)`, `TryFrom<&FinalizeArgs>` |
| `push.rs` | `push` | resumable state machine |

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
conversion is fallible) at the binary edge. `init` (0.44.0)
is the worked example; `finalize` (0.46.0) added the `TryFrom`
/ `Context.log` variant; the `0.48.0-N` cycle sweeps the rest.
Per-port status is the table below; the live checklist is the
"Subcommand layer / CLI decoupling" item in
[`notes/todo.md`](notes/todo.md).

| Subcommand | Status |
| --- | --- |
| `init` | done (0.44.0) — `init(&Context, &InitParams)`; `From<&InitArgs>` |
| `finalize` | done (0.46.0) — `finalize(&Context, &FinalizeParams)`; `TryFrom<&FinalizeArgs>` (fallible boundary); `--log` moved onto `Context` |
| `sync` | done (0.48.0-3) — `sync(&Context, &SyncParams)`; `From<&SyncArgs>` |
| `chid` | not started |
| `desc` | not started |
| `list` | not started |
| `show` | not started |
| `validate-desc` | done (0.48.0-4) — `validate_desc(&Context, &ValidateDescParams)`; `From<&ValidateDescArgs>` |
| `fix-desc` | not started |
| `clone` | done (0.48.0-2) — `clone_repo(&Context, &CloneParams)`; `From<&CloneArgs>` |
| `push` | not started |
| `symlink` | done (0.48.0-1) — `symlink(&Context, &SymlinkParams)`; `From<&SymlinkArgs>` |

The rest are planned as one multi-step cycle (`0.48.0-N`), one
step per subcommand (`chid`/`desc`/`list`/`show` ride with the
separate "CommonArgs sweep" since their Migration A and B are
entangled).

Out of scope for the early ports (deferred until a real
consumer surfaces): typed errors, returned outcomes vs
`println!`, the `ProgressSink`, and `Context` fields beyond
`UserConfig` + the `--log` path (`finalize` surfaced `log`).

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
- **`chid` / `desc` / `list` / `show`** — share
  `common::CommonArgs`, a per-domain shared struct rather
  than an `options_flags/` leaf. The "CommonArgs sweep"
  todo item folds these into the leaf model (and drops the
  repeatable `-R`/`--repo` in favor of the `--scope` path
  form).
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
