# Chores-10

Continuation of `chores-09.md` (which is closed at `0.48.4` —
the `0.42.0`–`0.48.x` cycles). This file covers the `0.49.0`
cycle onward, plus a living **refactor-tracking** section (the
per-subcommand status — for the Context+Params port and the
options_flags extraction — that used to sit in `ARCHITECTURE.md`).

See [`../ARCHITECTURE.md`](../ARCHITECTURE.md) for what those
two refactors *are* and why; this file is the *how* and the
*live status*. Reference numbering is file-local — see
[`README.md`](README.md#reference-numbering); chores-10 starts
at `[1]`.

## Refactor tracking — Context+Params / options_flags

Two parallel refactors of the subcommand layer (see
`ARCHITECTURE.md`). Status as of the entries below; update in
place as cycles land.

### Context+Params port — `pub fn x(args: &XxxArgs)` → `pub fn x(ctx: &Context, params: &XxxParams)`

| Subcommand | Status |
| --- | --- |
| `init` | done (0.44.0) — worked example; `From<&InitArgs>` |
| `finalize` | done (0.46.0) — `TryFrom<&FinalizeArgs>` (fallible); `--log` onto `Context` |
| `symlink` | done (0.48.0-1) — `From<&SymlinkArgs>` |
| `clone` | done (0.48.0-2) — `From<&CloneArgs>` |
| `sync` | done (0.48.0-3) — `From<&SyncArgs>` |
| `validate-desc` | done (0.48.0-4) — `From<&ValidateDescArgs>` |
| `fix-desc` | done (0.48.0-5) — `From<&FixDescArgs>` |
| `push` | done (0.48.0-6) — `From<&PushArgs>` (collapses the two bookmark spellings) |
| `chid` | pending (0.49.0-3) — introduces the shared `CommonParams`; `TryFrom` |
| `desc` | pending (0.49.0-4) — `TryFrom` |
| `list` | pending (0.49.0-5) — `TryFrom` |
| `show` | pending (0.49.0-6) — `TryFrom`; also parses `--files` → `FileLimit` at the boundary |

Out of scope for the ports (deferred until a real consumer
surfaces): typed errors, returned-outcomes-vs-`println!`,
`ProgressSink`, `Context` fields beyond `UserConfig` + `--log`.

### options_flags extraction — per-subcommand inline `#[arg]` → `src/options_flags/` leaves/bundles

| Subcommand(s) | Status |
| --- | --- |
| `init` | fully composed — `account` / `repo` / `scope` / the `provision` bundle / `use_template` / `config` |
| `finalize` | `--squash` → `squash` leaf; `--delay` / `--detach` / `--exec` / `--repo` / `--push` still inline |
| `chid` / `desc` / `list` / `show` | flatten `options_flags::common_args::CommonArgs` (relocated there at 0.49.0-1.1 — an inline-fields bundle, no per-flag sub-leaves; see [Bundle](../src/options_flags/README.md#architecture)); `-R`/`--repo` → `-s`/`--scope` leaf in 0.49.0-2 |
| `sync` / `push` / `clone` / `validate-desc` / `fix-desc` / `symlink` | mostly inline; the `--scope` retrofits queued in `todo.md` are the usual entry point |

## chore: open 0.49.0 — finish Migration A (0.49.0-0)

Commits: [[1]]

Multi-step. Finish Migration A: port the last four subcommands
`pub fn x(args: &XxxArgs)` → `pub fn x(ctx: &Context, params:
&XxxParams)`, same shape as the `0.48.0` sweep. `chid` / `desc`
/ `list` / `show` all `#[command(flatten)]` `common::CommonArgs`,
so the cycle also adds a shared clap-free `CommonParams` they
reuse. Mechanical; no behavior change.

(Superseded at 0.49.0-1 — the cycle was expanded to the full
"CommonArgs sweep", B-first; see the next section's `### Cycle
re-scope`. This section keeps the original narrow framing as the
record of what the 0.49.0-0 commit's plan was.)

### Per-step shape

- `common::CommonParams` (flat, clap-free): the resolved
  `DotSpec` + `Header` + raw `repos: Vec<PathBuf>`, with
  `impl From<&CommonArgs>` doing the `resolve_spec` /
  `resolve_header` work at the binary edge. The `..` parsing and
  `-l`/`-L` resolution move out of the subcommand bodies.
- `XxxParams`: flat struct embedding `CommonParams`, plus the
  subcommand's own fields — `list` adds `width: usize`, `show`
  adds `files: FileLimit` (the `--files` string is parsed at the
  boundary). `impl From<&XxxArgs>` (total) for `chid` / `desc` /
  `list`; `impl TryFrom<&ShowArgs>` for `show` (fallible —
  `FileLimit::parse`), mirroring `finalize`.
- `pub fn x(args)` → `pub fn x(_ctx: &Context, params:
  &XxxParams)`; `ctx` unused (uniform-signature placeholder, as
  in `symlink` / `validate-desc` / `fix-desc`).
- `main.rs` dispatch arm builds `Context::load(cli.log)` + the
  params; the `suppress_banner` match keeps reading
  `a.common.no_label` off the args (clap edge, unchanged).
- Tests: existing `XxxArgs` parse tests untouched; add a small
  "construct `XxxParams` directly" test per the worked-example
  precedent.

### Ladder (original — superseded)

Smallest first; `show` (the `TryFrom` + `FileLimit` parse) last.

- 0.49.0-0 plan + version bump + this section + todo ladder
  (current)
- 0.49.0-1 chid + introduce `CommonParams` in `common.rs`
- 0.49.0-2 desc
- 0.49.0-3 list
- 0.49.0-4 show
- 0.49.0 close-out — drop suffix, todo→Done, ARCHITECTURE.md
  Migration A table all-done (12/12)

This cycle is Migration A only. Migration B for these four
(folding `CommonArgs` into the `options_flags/` leaf model,
dropping the repeatable `-R`/`--repo` for the `--scope` path
form) stays the separate "CommonArgs sweep" todo.

### Per-substep contract

Per `notes/substep-protocol.md`: `cargo fmt` / `clippy
--all-targets -- -D warnings` / `test` / `install --path .
--locked` + retest before each commit; bump `Cargo.toml` at
sub-step start; flip todo ladder markers; pair commits across
both repos with ochid trailers.

## refactor: CommonArgs → options_flags (0.49.0-1.1)

The options_flags-extraction relocation, and where the cycle is
re-scoped (see `### Cycle re-scope`). Relocate `common::CommonArgs`
— the shared arg set for `chid` / `desc` / `list` / `show` —
from `src/common.rs` into `src/options_flags/common_args.rs`.
Inline fields, no per-flag sub-leaves (an "inline-fields bundle"
— see [Bundle](../src/options_flags/README.md#architecture);
rationale in `### Why not decompose into leaves`). Pure
relocation — `vc-x1 chid -h` etc. byte-identical.

- `CommonArgs` → `options_flags/common_args.rs` (`impl OptionFlagBundle`).
- `src/common.rs`: `CommonArgs` + `use clap::Args` dropped (the
  `for_each_repo` / `collect_ids` / `resolve_*` / `format_*`
  helpers stay).
- `chid` / `desc` / `list` / `show`: flatten the relocated
  struct; each gains the `//!` module docstring it lacked;
  bodies otherwise unchanged.
- `main.rs`: `suppress_banner` reads `a.common.no_label` for all
  four (back to pre-cycle).

`0.49.0-1.2` (the ARCHITECTURE.md slim + chores-10 reorg, own
section below) follows; the two are kept separate at close-out —
no squash, no extra `0.49.0-1` commit.

An earlier `0.49.0-1.1` decomposed `CommonArgs` into per-flag
leaves; backed out on review, amended in place to the relocation
above — see `### Why not decompose into leaves`.

### Why not decompose into leaves

The first take decomposed `CommonArgs` into per-flag
`options_flags/` leaves (`revision` / `commit_limit` /
`repo_label`) flattened into a `common_bundle` — mirroring
`init` + `provision_bundle`. Backed out:

- **No reuse.** `init`'s leaves are reused (`--dry-run` →
  `clone`, etc.) — a per-flag leaf is the unit of sharing. The
  four here share the *whole set*; none of `revision` / `limit`
  / `label` / etc. is reused elsewhere, so per-flag leaves buy
  nothing the shared struct doesn't already give.
- **`value`/`id` friction.** Two single-field-`value` leaves
  ([README](../src/options_flags/README.md)) collide on the clap
  arg *id* (derived from the field name) when co-flattened —
  `clap_builder` panics. The fix (`#[arg(id = …)]` on each) is
  the "not obvious when `id` vs `value`" sharp edge —
  self-inflicted by decomposing leaves that didn't need to be.

Rule of thumb:

- Leaf — when *that flag / small group* is genuinely shared at
  that granularity (`--dry-run`, `--squash`).
- Inline-fields bundle — when a *whole arg set* is shared by N
  commands but its parts aren't reused.
- Extract a leaf later, when a second consumer is real:
  `CommonArgs` then `#[command(flatten)]`s the extracted type
  (one definition, no duplication).

### Cycle re-scope (0.49.0-1)

0.49.0-0 opened a "finish Migration A" cycle for these four
subcommands; at 0.49.0-1 review it was expanded to the full
**"CommonArgs sweep"** (`chores-06` / `chores-07`'s long plan)
and re-ordered B-first so the Context+Params ports land once.
(The 0.49.0-0 commit/section keep the original framing; this
records the change. "Migration A" / "Migration B" were the
working names — renamed here to the Context+Params port and the
options_flags extraction; the frozen 0.48.x / 0.49.0-0
commit-recording sections keep the old names.) The three parts:

- **options_flags extraction** (0.49.0-1, sub-steps -1.1
  relocation, -1.2 docs reorg):
  - relocate `CommonArgs` → `options_flags/common_args.rs`
    (inline-fields bundle, no per-flag leaves);
  - move all four subcommands onto it;
  - remove `src/common.rs`'s `CommonArgs`.
- **`--scope`** (0.49.0-2) — `-R`/`--repo` → `-s`/`--scope`
  (`code|bot|code,bot|<path>`):
  - wires up `scope.rs`'s `parse_scope` + `Scope::Single` (both
    `#[allow(dead_code)]`, built in 0.42.0 for this) and
    `common.rs`'s `default_scope` / `scope_to_repos`;
  - `-R` dropped, not aliased; arbitrary multi-repo
    (`-R . -R .claude`) not preserved — per the 0.42.0 capture
    (`chores-06` 0.41.0-4); `--scope=<path>` covers single-repo,
    default `default_scope()`;
  - `for_each_repo` takes an already-resolved `Vec<PathBuf>`
    (comma-expansion + `["."]` default move to the boundary);
  - `init`'s `--scope` (the separate `ScopeKind` `code,bot|por`)
    left alone — the "por/dual parity" todo.
- **Context+Params port** (0.49.0-3..-6) — `XxxParams` + `fn
  x(&Context, &XxxParams)` ports against the post-B/-scope
  `CommonArgs`:
  - introduces the shared clap-free `CommonParams` (resolved
    `DotSpec` + `Header` + repos);
  - all four take a fallible `TryFrom<&XxxArgs>` (scope→repo
    resolution can error);
  - `show` also parses `--files` → `FileLimit` at the boundary.

Revised ladder (supersedes the 0.49.0-0 one):

- 0.49.0-1 options_flags extraction — relocate `CommonArgs` to
  `options_flags/common_args.rs`; this re-scope. Landed as two
  kept-separate sub-steps (no `0.49.0-1` close-out commit):
  - 0.49.0-1.1 the relocation + all four importers
  - 0.49.0-1.2 docs: slim ARCHITECTURE.md; start chores-10
- 0.49.0-2 `-R`/`--repo` → `-s`/`--scope` (wire `parse_scope` /
  `default_scope` / `scope_to_repos`; slim `for_each_repo`) —
  may split into `-2.N`
- 0.49.0-3 chid Context+Params port + introduce `CommonParams`
- 0.49.0-4 desc Context+Params port
- 0.49.0-5 list Context+Params port
- 0.49.0-6 show Context+Params port (`TryFrom`, `FileLimit` parse)
- 0.49.0 close-out — drop suffix, todo→Done (Context+Params port
  12/12 + CommonArgs sweep), README + ARCHITECTURE.md

## docs: slim ARCHITECTURE.md; chores-10 (0.49.0-1.2)

Acting on review feedback that `ARCHITECTURE.md` had drifted
into "how" / transient territory — per-subcommand status,
version-by-version tables, sub-step ladders. Pull that out so
`ARCHITECTURE.md` is "what + some why, generic" and the transient
tracking lives here. Also renamed the two cross-cutting
refactors: "Migration A" → the Context+Params port, "Migration
B" → the options_flags extraction (the frozen 0.48.x / 0.49.0-0
commit-recording sections keep the old names; a note in `###
Cycle re-scope` records the change).

- `ARCHITECTURE.md` — slimmed to "what + some why, generic":
  - kept: Overview, the two-layer section + rationale + `Naming`,
    a generic module map (kinds of module — no per-subcommand
    table, no version annotations), the Subcommand-model recipe,
    See-also;
  - the two refactor sections (`## args → Context + Params` /
    `## per-subcommand flags → src/options_flags/`) → short
    "what + why" + a pointer here for live status (the
    per-subcommand status table, the version-by-version "done"
    list, the "State today" bullets are gone).
- `notes/chores-10.md` — new (this file): the `## Refactor
  tracking` tables (out of `ARCHITECTURE.md`) + the `0.49.0-*`
  sections moved out of `chores-09.md`.
- `notes/chores-09.md` — the `0.49.0-*` sections removed; it now
  ends at `0.48.4` (all done/closed).
- `notes/README.md` — notes that `chores-09` is closed and
  `chores-10` is the active file with the tracking section.
- `notes/todo.md` — "Design:" link repointed to `chores-10.md`;
  the `0.49.0-1` sub-step ladder shows `-1.1` (the relocation)
  and `-1.2` (this), kept separate — see the `0.49.0-1.1`
  section.
- `src/options_flags/README.md` — the "Migration B" mention →
  "the options_flags extraction".

# References

[1]: https://github.com/winksaville/vc-x1/commit/10788bd158c4 "10788bd158c4574fe5a10fab41ea32e4becc86d3"
