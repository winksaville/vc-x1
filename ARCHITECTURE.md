# Architecture

How the `vc-x1` binary is structured: the clap-aware CLI layer
and the (emerging) clap-free subcommand layer. This is the
"what + some why" — generic and slow-moving. The *how* and the
per-cycle / per-subcommand status live in `notes/chores/chores-*.md`
and `notes/todo.md`.

This is a living document; the app is under continuous
development and the repo will keep changing.

## Overview

`vc-x1` is a single binary that supports subcommands,
`vc-x1 <subcommand>`, for managing a jj-git **dual-repo workspace**
with an app repo plus a `.claude` bot-session repo. The commits
cross-reference each other via `ochid:` trailers at the bottom
of each commit message. See [`README.md`](README.md) for the
user-facing picture and [`CLAUDE.md`](CLAUDE.md) for the bot
workflow.

## Two layers: CLI args vs subcommand Context + Params

Goal: separate clap-aware argument parsing from subcommand
operation logic, so a future front-end (TUI, library embedding)
can call the same core without dragging clap along. The trigger
was `args.account.account`-style nested-leaf access leaking into
subcommand bodies; accessor shortcuts were rejected as hiding
the mismatch rather than fixing it.

- **CLI layer (clap-aware)**: builds `XxxParams` from the
  user-facing `XxxArgs` `#[derive(Args)]` clap struct.
  Options/flags used in more than one subcommand live as
  flattened leaves/bundles in `src/options_flags/`.
- **Subcommand layer (clap-free)**: entrypoint shape
  `fn x(ctx: &Context, params: &XxxParams) -> Result<…>` —
  isolates the subcommand from clap and from the user.
  - `Context` — platform state shared by every subcommand.
  - `XxxParams` — that subcommand's inputs, derived from
    config / command-line args.
- **Boundary conversion**: `XxxArgs → XxxParams` via
  `impl From<&XxxArgs>` or `impl TryFrom<&XxxArgs>`, depending
  on whether the conversion can fail.

Two parameters per subcommand; the signature documents what the
subcommand depends on. `Context` is the same across all
subcommands; `XxxParams` carry the per-subcommand inputs.

Rules for the subcommand layer (some still aspirational — see
[the Context+Params port](#args--context--params)):

1. Plain options, flat fields, `Default` — no clap types in
   `XxxParams`. Domain types (`RepoSelector`, `Scope`) are
   fine; leaf wrappers (`RepoOption`) are not.
2. Typed errors (`enum XError`, not `Box<dyn Error>`) — a TUI
   matches variants to pick dialogs; CLI formats them.
3. Returned outcomes, not `println!` — each subcommand returns
   a structured result; CLI formats it, a library writes
   nothing to stdout itself.
4. Progress via an optional `&mut dyn ProgressSink` on
   long-running subcommands — CLI installs a stderr sink, tests
   a recording sink.
5. No globals, no implicit cwd/env reads in subcommands — CLI
   resolves cwd/env once at startup and builds `Context`.

None of this is sacred — the whole structure is an experiment.
The current scope is deliberately narrow: a small `Context`
(two params, not one god blob); no trait-based DI yet; one
crate, not a workspace split; `src/options_flags/` leaves left
in place — only their consumers change shape. Widen any of it
when the evidence (a second front-end, a second consumer crate,
a real platform-surface divergence) calls for it — not before.

### Naming

Earlier drafts of this design used `Workspace` / `XOptions`;
the implementation (0.44.0) chose `Context` / `XxxParams`:

- `Workspace` → `Context` — Cargo owns "workspace" formally,
  and this codebase already uses "workspace" for the dual-repo
  project root (`find_workspace_root`).
- `XOptions` → `XxxParams` — avoids visual collision with
  `Option<T>` (a params struct mixes required and
  `Option<T>`-typed fields) and with `src/options_flags/`.

## Module map

The *kinds* of module and what they're for — not an exhaustive
catalog (read the `//!` docstrings; `notes/chores/chores-*.md` for
per-subcommand refactor status):

**CLI layer:**

- `src/main.rs` — the clap edge: `Cli` (`Parser`), `Commands`
  (`Subcommand`) + the dispatch arms, `cli_with_banner()` (the
  per-subcommand `vc-x1 X.Y.Z` banner), `CompleteEnv` wiring,
  and `bm_track` (the permanent `main`-bookmark tracking
  diagnostic on every command's entry/exit).
- `src/options_flags/` — reusable `#[derive(Args)]` *leaves*
  (one flag, or a small fixed pair) and *bundles*
  (`#[command(flatten)]` of N leaves) shared across
  subcommands, so a flag is defined (parser, help, completer)
  exactly once. See
  [`src/options_flags/README.md`](src/options_flags/README.md).
- `src/common.rs` — helpers the read-only commit-query
  subcommands (`chid` / `desc` / `list` / `show`) share:
  `for_each_repo`, `collect_ids`, `resolve_revset`, the
  `DotSpec` / `..`-notation parsing, `Header` /
  `resolve_header`, the `format_*` printers, and the
  `-R`/`-s` resolution stack — `find_workspace_root` /
  `default_scope` / `scope_to_repos` / `resolve_repos`
  (the top-level entry combining the `-R PATH` override and
  the `-s code|bot|code,bot` role selection).

**Subcommand-layer scaffolding:**

- `src/context.rs` — `Context`, the shared platform handle
  (today: the loaded `UserConfig` + the `--log` path; built
  once at startup via `Context::load(log)`).
- `src/config.rs` — `UserConfig` (`~/.config/vc-x1/config.toml`).
- `src/scope.rs` — `Scope` (`Roles(Vec<Side>)` for the
  dual-repo `code`/`bot` roles, `Single(PathBuf)` for explicit
  single-repo mode) and `parse_scope`.

**Subcommand modules** — `src/<x>.rs` holds the subcommand's
`XxxArgs` (`#[derive(Args)]`, composing `options_flags/` leaves
where one exists) and the entrypoint `pub fn x(...)`. Modules:
`chid`, `desc` (+ `desc_helpers`), `list`, `show`,
`validate_desc`, `fix_desc`, `clone`, `init` (+ `init/params`),
`symlink`, `sync`, `finalize`, `push` (a resumable state
machine). Which ones have ported to `(ctx, params)` and which
compose `options_flags/` leaves: `notes/chores/chores-*.md` +
`notes/todo.md`.

**Support:**

- `src/repo_utils.rs`, `src/url.rs`, `src/toml_simple.rs`,
  `src/logging.rs` — jj/repo helpers, URL parsing, a minimal
  TOML reader, the CLI logger.
- `src/test_helpers.rs`, `src/test_tmp_root.rs` —
  `#[cfg(test)]` fixtures and tempdir resolution. Subcommand
  modules also carry their own `#[cfg(test)] mod tests` (and
  `integration_tests` for `sync` / `push`).

## Subcommand model

Adding a subcommand `x`:

1. Create `src/x.rs` with `pub struct XxxArgs`
   (`#[derive(Args)]`), composing leaves from
   `src/options_flags/` where one exists.
2. Add a variant `X(x::XxxArgs)` to `Commands` in `src/main.rs`
   with a `///` summary (and `#[command(long_about = …)]` for
   the long form).
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

## args → Context + Params

"The Context+Params port" — each subcommand's
`pub fn x(args: &XxxArgs)` is structured as
`pub fn x(ctx: &Context, params: &XxxParams)`, with an
`XxxParams` flat struct + a `From` (or `TryFrom`, if the
conversion is fallible) at the binary edge. `init` (0.44.0)
is the worked example; complete across all 12 current
subcommands as of 0.49.0, and the shape new subcommands
follow. Out of scope until a real consumer surfaces: typed
errors, returned-outcomes-vs-`println!`, `ProgressSink`,
`Context` fields beyond `UserConfig` + `--log`. Per-subcommand
status: `notes/chores/chores-*.md` (the "Refactor tracking" section).

## per-subcommand flags → `src/options_flags/`

Independently of the Context+Params port — "the options_flags
extraction" — per-subcommand inline `#[arg]` fields are being
lifted into reusable leaves/bundles under `src/options_flags/`
so a flag is defined (parser, help, completer) exactly once. See
[`src/options_flags/README.md`](src/options_flags/README.md)
for the leaf / bundle / Pattern-A mechanics and the
Flag-vs-Option classification; `notes/chores/chores-*.md` for
per-subcommand status.

## See also

- [`README.md`](README.md) — user-facing overview and
  per-subcommand usage.
- [`CLAUDE.md`](CLAUDE.md) — bot workflow, versioning,
  commit/push conventions, code conventions.
- [`src/options_flags/README.md`](src/options_flags/README.md)
  — leaf/bundle patterns for the options_flags extraction.
- [`notes/todo.md`](notes/todo.md) — live task list, including
  both refactors.
- `notes/chores/chores-*.md` — per-cycle design captures and the
  refactor-tracking tables (currently
  [`chores-10.md`](notes/chores/chores-10.md); `chores-09.md` and
  earlier are closed).
