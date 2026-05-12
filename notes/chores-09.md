# Chores-09.md

General chores notes — design captures (forward-looking) and
post-implementation chore entries. Same shape as chores-01..08.md;
09 starts here because chores-07 (0.42.0 cycle, 600+ lines) and
chores-08 (0.41.1 cycle, 1500+ lines) are both already large; the
init-clone-refactor rebase landing is a natural new-file
boundary.

Subsection headers use the trailing-version format from CLAUDE.md
when they correspond to a release: `## Description (X.Y.Z)`.

## chore: init-clone-refactor rebase landing (0.42.0-4.7)

Commits: [[1]]

Rebased main (0.42.0-0..-4.6) onto init-clone-refactor at its
0.41.1 close-out tip (`slvprlpw`). Chids preserved; ochid
trailers in the rebased commits stayed structurally valid (the
.claude pairings they point at had been WC-only during the icr
divergence; -4.7 itself re-establishes the .claude side).

### What needed real work

- **ttzwvpoq (0.42.0-1):** full `Scope` enum lifted wholesale
  from pre-rebase main; `#[allow(dead_code)]` on `is_bot_only`
  and `Single` / `parse_scope` since icr's bundle design keeps
  them dormant.
- **mvusyowm (0.42.0-2):** icr's bundle-flatten `InitArgs` +
  `ScopeOption`/`ScopeKind` already supersedes mvusyowm's
  parser-switch + `Option<Scope>` design; took icr wholesale
  across all 5 init.rs conflict regions. The cycle's narrative
  intent (custom `--scope` parser accepting paths) becomes
  dormant on the icr base; the `Scope::Single` variant survives
  for future use.
- **kyurxpnu (0.42.0-4.5):** CLAUDE.md substep references merged
  (lulqxovr's icr-side decomposition pointer + kyurxpnu's
  external `substep-protocol.md` / `jj-revsets.md` links) into
  a single continuation paragraph under `### Versioning`.
- **Reference renumbering** — cycle's `[72]`/`[73]`/`[74]`
  collided with icr's chores-08 anchors: `[72]→[76]`,
  `[73]→[77]`, `[74]→[78]`, plus self-refs in chores-07's
  -4.5 and -4.6 sections.

### Precautions taken (most unnecessary in retrospect)

- `main-2` local duplicate bookmark — never read.
- `gca-icr-main` common-ancestor marker — never referenced.
- `rslv-commit` cursor bookmark — handy as a navigation
  aid during cascade, not load-bearing (`@-` works).
- `../vc-x1-main` + `../vc-x1-icr` reference clones — useful
  for content lookup at specific chids; replaceable with
  `jj log -r <chid> --patch` against the local repo.
- Filesystem snapshots `vc-x1-20260505-1`, `vc-x1-20260506-1`
  — never restored from.
- `~/vc-x1-rebase-status.md` scratch file — useful for
  resuming across sessions, but ~200 lines was over-detailed;
  a 30-line cursor + decisions log would have sufficed.

### What was load-bearing

- `jj op restore` for local rewinds.
- origin's canonical pre-rebase main remaining intact until
  the explicit force-push at the end (the only fallback that
  mattered).
- jj's chid-preservation across rebase: ochid trailers stay
  valid pointers without manual fixup.
- Per-commit cargo cycle (fmt/clippy/test/install --locked)
  to verify each squash kept the chain buildable.

### Distilled recipe for next time

1. `jj rebase` of the divergent chain onto the new base.
2. Per conflicted commit: `jj new <commit>`, edit conflicts,
   cargo cycle, `jj squash`.
3. After cascade clears: cargo cycle at tip, `jj git push
   --bookmark main`.
4. `.claude` side: `vc-x1 push` to author the paired commit
   once main converges with origin.

Skip the duplicate bookmarks, reference clones, and
filesystem snapshots unless a specific reason emerges. The
op-log + origin are the safety net.

## chore: close 0.42.0 cycle at -4.7 (0.42.0)

Commits: [[2]]

Cycle closed at -4.7 (init-clone-refactor rebase landing)
rather than completing the originally planned full
`--scope` sweep. What shipped, what deferred, and why.

### Shipped

- 0.42.0-0 plan + version bump + new chores-07.md
- 0.42.0-1 `Scope` enum (`Roles(Vec<Side>) | Single(PathBuf)`)
- 0.42.0-2 custom CLI parser + `init --scope` retrofit
- 0.42.0-3 `sync --scope` retrofit (drop -R, add -s)
- 0.42.0-4.5 substep protocol + jj revsets cheatsheet
- 0.42.0-4.6 init-clone-refactor recovery + post-mortem
- 0.42.0-4.7 init-clone-refactor rebase landing

### Deferred to future cycles

Originally-planned -4 / -5 / -6 / -7 substeps — the
`--scope` sweep across the remaining subcommands — moved
back to `notes/todo.md > ## Todo`. Design references stay
at chores-07 [[3]]:

- `vc-x1 push --scope` (was -4; pivoted into substep
  protocol + icr work).
- `vc-x1 finalize --scope` (was -5).
- `vc-x1 clone --scope` (was -6).
- `Single(_)` end-to-end dogfood validation (was -7).

Plus already-scheduled but co-deferred items: `vc-x1
validate-desc / fix-desc --scope` and the CommonArgs
sweep across `chid`/`desc`/`list`/`show`.

### Narrative shift

The cycle started as a `--scope` sweep but pivoted at
-4.5:

1. Substep protocol formalization (-4.5) emerged as
   needed before further substep work.
2. init-clone-refactor recovery (-4.6) surfaced as
   higher-priority than `push --scope` once the icr
   branch was located.
3. Rebasing icr onto the cycle's tip (-4.7) became the
   natural pivot point. The bot thinks continuing
   `push --scope` past -4.7 would have lengthened an
   already-pivoted cycle past the point of useful
   narrative coherence; closing here and reopening
   fresh later produces cleaner history.

## docs: bot-data-formats + multi-user notes (0.42.1)

Commits: [[4]]

Documentation-only follow-on to the 0.42.0 close-out.
Forward-looking design captures for multi-user
collaboration, multi-bot vendor support, and bot-repo
scaling thresholds. No code change.

## chore: open 0.43.0 cycle (0.43.0-0)

Commits: [[5]]

Multi-step cycle to extract `#[cfg(test)] mod tests` (and
`mod integration_tests` where present) from oversized
production files into sibling-submodule layout. Pure
mechanical reshape — no behavior change, no API change.
Each test still reaches private items via
`use super::*;`.

### Goals

- Shrink production files so the actual code is easier
  to navigate (`init.rs` is 2576 lines today, ~1093 of
  which are tests).
- Establish a layout shape that future cycles can keep
  using (further splitting into `cli.rs` / `ops.rs` per
  the subcommand-layer architecture, etc.).
- Done as one cycle so the four files end up
  consistent rather than half-converted across multiple
  cycles.

### Per-file shape

Non-mod.rs layout (Rust 2018+ idiom): the production
file keeps its top-level path; a sibling directory holds
children. `src/X.rs` and `src/X/` coexist — Rust resolves
`mod foo;` declared in `src/X.rs` to `src/X/foo.rs`.

```
src/X.rs               ← production code (unchanged path)
src/X/
  tests.rs             ← moved from #[cfg(test)] mod tests
  integration_tests.rs ← moved from #[cfg(test)] mod
                         integration_tests (push, sync)
```

In `src/X.rs` after extraction:

```rust
// ...production code...

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;  // only for push, sync
```

In `src/X/tests.rs`:

```rust
//! Unit tests for the X module.

use super::*;
// ...test fns...
```

**Why non-mod.rs.** The `mod.rs` style names every
module's entry point identically — fuzzy finders show a
column of `mod.rs` rows that the reader must
disambiguate by directory. Multiple `tests.rs` files
don't have the same pain (the file is leaf content; the
directory tells the reader which module's tests they
are). Modern Rust (`cargo new`, RFC 2126) defaults to
non-mod.rs for exactly this reason.

### Sub-step ladder

Size-descending order (init has the most test bulk;
common is borderline):

- 0.43.0-0 plan + version bump + this section +
  todo.md ladder
- 0.43.0-1 src/init.rs (kept) + src/init/tests.rs
  (~1093 test lines)
- 0.43.0-2 src/push.rs (kept) + src/push/{tests.rs,
  integration_tests.rs} (~784 test lines + integration
  block)
- 0.43.0-3 src/sync.rs (kept) + src/sync/{tests.rs,
  integration_tests.rs} (~600 test lines, multiple
  `#[cfg(test)]` blocks)
- 0.43.0-4 src/common.rs (kept) + src/common/tests.rs
  (~384 test lines, borderline — call after seeing
  the first three land)
- 0.43.0 close-out

### Order rationale

`init` is the worked example: largest test bulk and the
file most-discussed in CLAUDE.md. Getting the pattern
right there sets the template for the others. The bot
thinks `common` is worth keeping for consistency but
reasonable to drop after seeing the first three land —
its 384 lines are below the threshold where extraction
visibly helps.

### Per-substep contract

Per `notes/substep-protocol.md`:

- Run `cargo fmt`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test`, `cargo install --path .
  --locked`, retest before each sub-step commit.
- Bump `Cargo.toml` version at the start of each
  sub-step.
- Mark prior sub-step `(done)` and current `(current)`
  in todo.md ladder.
- Pair commits across both repos with ochid trailers.

### Cycle exemption from per-file review

Test-module extraction is exactly the "code moving
across files" refactor case CLAUDE.md
(`### Per-file review checkpoints > Exceptions`)
exempts. Each sub-step commits as one unit; no
checkpoint between original-file edit and new-file
creation.

## refactor: extract init test module (0.43.0-1)

Commits: [[6]]

Extracted `mod tests` from `src/init.rs` into sibling
`src/init/tests.rs`. Production file keeps its top-level
path (`src/init.rs`); the new `src/init/` directory
holds only the test submodule. Tests reach private items
via `use super::*;`; no visibility changes needed.

Production code: 1481 lines (`src/init.rs`). Tests: 1091
lines (`src/init/tests.rs`, de-indented from the 4-space
wrapper indent of the original `mod tests { ... }` block).
Total +1 line vs original (`//! Unit tests for the init
module.` header + 1 blank line in tests.rs; `mod tests;`
forward declaration + 1 blank line in init.rs; replacing
the 4-line `mod tests { ... }` wrapper).

`cargo test` shows 358 unit + 14 integration tests
passing, identical to the pre-extraction baseline.

Initial draft used `src/init/mod.rs` for production code;
revised to non-mod.rs layout (`src/init.rs` retained,
`src/init/` holds children only) before commit. See
Per-file shape rationale above.

## refactor: extract push test modules (0.43.0-2)

Commits: [[7]]

Extracted both `mod tests` and `mod integration_tests`
from `src/push.rs` into siblings `src/push/tests.rs` and
`src/push/integration_tests.rs`. Production file kept at
`src/push.rs` (non-mod.rs layout). Tests reach private
items via `use super::*;`; no visibility changes.

Production code: 1487 lines (`src/push.rs`, was 2267).
Unit tests: 423 lines (`src/push/tests.rs`, de-indented
from 4-space wrapper). Integration tests: 359 lines
(`src/push/integration_tests.rs`, de-indented; preserves
the original `//!` header block as file-level docs).

`cargo test` baseline preserved: 358 unit + 14
integration tests, identical to pre-extraction.

## refactor: extract sync test modules (0.43.0-3)

Commits: [[8]]

Extracted `mod tests` and `mod integration_tests` from
`src/sync.rs` into siblings `src/sync/tests.rs` and
`src/sync/integration_tests.rs`. Same non-mod.rs layout
as -1 / -2. Tests reach private items via
`use super::*;`; no visibility changes.

Production code: 608 lines (`src/sync.rs`, was 1206).
Unit tests: 145 lines (`src/sync/tests.rs`).
Integration tests: 455 lines
(`src/sync/integration_tests.rs`; preserves the
original `//!` block as file-level docs).

The pre-existing `#[cfg(test)] use crate::scope::Side;`
scaffolding at the top of `sync.rs` (which existed only
to keep `Side` in scope for the inline `mod tests`) is
moved to the extracted test files alongside the rest
of their imports — the production file no longer
carries test-only `use` lines.

`cargo test` baseline preserved: 358 unit + 14
integration tests, identical to pre-extraction.

## refactor: extract common test module (0.43.0-4)

Commits: [[9]]

Extracted `mod tests` from `src/common.rs` into sibling
`src/common/tests.rs`. Same non-mod.rs layout as the
prior sub-steps. Tests reach private items via
`use super::*;`; no visibility changes.

Production code: 715 lines (`src/common.rs`, was 1099).
Unit tests: 385 lines (`src/common/tests.rs`).
No integration test module.

`cargo test` baseline preserved: 358 unit + 14
integration tests, identical to pre-extraction.

The chores-09 plan flagged this sub-step as
"borderline" — common.rs's test bulk (~384 lines) is
below the threshold where extraction visibly helps.
Done anyway for consistency with the cycle pattern.

## chore: close 0.43.0 cycle (0.43.0)

Commits: [[10]]

Cycle landed as five separate commits on `main`
(0.43.0-0 plan + four sub-step extractions). Chose
"keep separate" over the default squash because the
per-file decomposition is informative: each sub-step
shows one production file's reshape in isolation, and
the ladder is a clean before/after for anyone reading
git log later.

### Cycle outcome

- Four production files reshaped to non-mod.rs layout
  (`src/X.rs` retained, sibling `src/X/tests.rs`
  [+ `integration_tests.rs` for push/sync]).
- Production-side line counts:
  init 1094 (+`src/init/tests.rs` 1092);
  push 1487 (+`src/push/{tests.rs 423, integration_tests.rs 359}`);
  sync 608 (+`src/sync/{tests.rs 145, integration_tests.rs 455}`);
  common 715 (+`src/common/tests.rs 385`).
- 358 unit + 14 integration tests pass identically to
  the pre-cycle baseline at every sub-step.
- `Side` import scaffolding in `sync.rs` (artifact of
  the inline `mod tests`) moved into the test files
  during the -3 fixup squash; production file no
  longer carries test-only `use` lines.

### Layout pivot mid-cycle

The original plan called for `src/X/mod.rs` per file.
After -1 landed, web-claude flagged the mod.rs idiom
as obsolete (Rust 2018+ defaults to `src/X.rs` +
`src/X/<children>.rs`), and the pivot was made before
-2. -1 was squashed to match (so all four sub-steps
share the same shape on `main`).

## Subcommand-layer architecture — moved to `ARCHITECTURE.md`

A forward-looking design capture lived here (written
pre-implementation, with the names `Workspace` / `XOptions`).
It is now the living [`ARCHITECTURE.md`](../ARCHITECTURE.md)
at the repo root — the "Two layers: CLI args vs subcommand
Context + Params" section plus Migrations A and B. The
0.44.0 implementation settled the names as `Context` /
`XxxParams`; see `ARCHITECTURE.md` § Naming.

Conclusions worth keeping in the journal:

- Two parameters per subcommand (`&Context`, `&XxxParams`),
  not one merged "god context".
- Concrete structs, not trait-based DI, until a second
  front-end forces generalization.
- Same crate, separate modules — promote to a Cargo
  workspace only when a second consumer crate appears.
- `src/options_flags/` leaves stay; only their consumers
  change shape. Completion stays a clap-layer concern.
- Convert one subcommand end-to-end as the worked example
  before any sweep — done in 0.44.0 (`init`); the rest are
  the "Subcommand layer / CLI decoupling" todo item.

`ARCHITECTURE.md` is authoritative; update it (not this
section) as the migrations progress.

## refactor: introduce Context + InitParams (0.44.0)

Commits: [[11]]

Single-step cycle: introduce `Context` + `InitParams` and
port `init` to the new shape. Establishes the worked
example for the design now in
[`ARCHITECTURE.md`](../ARCHITECTURE.md); remaining
subcommands defer to later cycles.

### Naming: `Context` / `XxxParams`

This cycle named the handle `Context` (not `Workspace` —
Cargo owns that word, as does this codebase's
`find_workspace_root`) and the per-subcommand input
`XxxParams` (not `XOptions` — avoids visual collision with
`Option<T>`, and with `src/options_flags/`). Full rationale:
[`ARCHITECTURE.md`](../ARCHITECTURE.md) § Naming.

### Why init

`args.account.account` ergonomics — the trigger for the
whole design — lives here; init also has the deepest
leaf nesting (account / repo / scope / private /
template / push retry). If the pattern survives init,
sync / chid / push / clone port trivially.

### In scope

- `src/context.rs` (new): `Context` struct holding the
  workspace root path + loaded `UserConfig`. Built once
  at startup.
- `src/init/params.rs` (new): `InitParams` flat plain
  struct (owns its values) + `impl From<&InitArgs> for
  InitParams`.
- `src/init.rs`: port `plan_init` and any other
  init-side `pub fn` entrypoints to
  `fn (&Context, &InitParams)`. Body reads
  `params.account` instead of `args.account.account`.
- `src/main.rs`: build `Context` once at startup; the
  init dispatch arm passes `(&ctx, &(&args).into())`.
- `src/init/tests.rs`: pure-unit tests construct
  `Context` + `InitParams` directly (no clap
  `try_parse_from` plumbing); integration paths still
  exercise the clap edge.

### Out of scope (deferred)

- Sweep across sync / chid / push / clone / etc.
  Each is its own later cycle.
- Per `ARCHITECTURE.md`'s subcommand-layer rules: typed
  errors, returned outcomes (vs `println!`), `ProgressSink`.
  Keep this cycle to the structural split only.
- `Context` fields beyond workspace root + user
  config. Defer until a real consumer surfaces.

### Pre-commit checklist

Standard: `cargo fmt`, `cargo clippy --all-targets --
-D warnings`, `cargo test`, `cargo install --path .
--locked`, retest.

## docs: ARCHITECTURE.md + subcommand-layer naming (0.45.0)

Commits: [[12]]

Single-step docs cycle. Promotes the forward-looking
"Ops layer architecture" capture (this file) into a living
`ARCHITECTURE.md` at the repo root, and reconciles the
terminology: the design capture said `Workspace` / `XOptions`
and "ops layer"; the 0.44.0 implementation shipped `Context`
/ `XxxParams`. Settled on "subcommand layer" everywhere. No
code change beyond a doc comment.

## refactor: finalize → Context + FinalizeParams (0.46.0)

Commits: [[13]]

Single-step cycle: bring `finalize` from "partial" to fully
on the subcommand-layer shape. It already had a clap-free
`FinalizeOpts` built via `FinalizeArgs::into_opts(log)`; this
cycle renames it to `FinalizeParams`, gives it a
`TryFrom<&FinalizeArgs>` boundary (fallible — `--squash`
parsing + `--repo` canonicalization), threads a `&Context`,
and moves the `--log` path onto `Context` (finalize is its
first non-`UserConfig` consumer). No behavior change.

Decisions: kept the `From` / `TryFrom` split rather than
forcing `TryFrom` everywhere — `From` is the right trait when
the conversion is total (`init`), `TryFrom` when it isn't
(`finalize`). `--log` went on `Context` (resolved once at
startup, like `UserConfig`) rather than staying a
`FinalizeParams` field.

## refactor: finalize --squash → options_flags leaf (0.47.0)

Commits: [[14]]

Single-step. Lift `--squash` (the one flag with a planned second
consumer) into a shared `options_flags` leaf.

- `--squash` → `options_flags/squash.rs`: leaf `SquashOption`,
  value `SquashSpec`, parser `SquashSpecParser`.
- `--delay` / `--detach` / `--exec` / `--repo` / `--push` stay
  inline — no second consumer (terminal state, not unfinished).
- `value_parser = SquashSpec::parse` → bad `--squash` errors at
  parse time, not in `try_from`. Only behavior change.
- New single-field-leaf convention: field `value`, flag via
  `#[arg(long = "…")]`, consumer reads `args.<leaf>.value`.
- Pre-existing leaves keep field-name-as-flag — `0.47.1` sweep
  queued.

## chore: open 0.48.0 cycle — Migration A sweep (0.48.0-0)

Commits: [[15]]

Multi-step. Port the remaining subcommands `pub fn x(args:
&XxxArgs)` → `pub fn x(ctx: &Context, params: &XxxParams)`, same
shape as `init` (0.44.0) / `finalize` (0.46.0). Mechanical; no
behavior change.

### Per-step shape

- `XxxParams`: flat struct, plain fields (domain types OK, no
  clap leaf wrappers) + `impl From<&XxxArgs>` (or `TryFrom`, if
  the conversion is fallible) at the binary edge.
- `pub fn x(args)` → `pub fn x(ctx, params)`; body reads
  `params.*`.
- `main.rs` dispatch arm builds `Context::load(cli.log)` + the
  params.
- Tests updated; flip the ARCHITECTURE.md Migration A table row.

### Ladder

Smallest/simplest first; `push` (state machine) last.

- 0.48.0-0 plan + version bump + this section + todo ladder
  (current)
- 0.48.0-1 symlink
- 0.48.0-2 clone
- 0.48.0-3 sync
- 0.48.0-4 validate-desc
- 0.48.0-5 fix-desc
- 0.48.0-6 push (may decompose into `-6.N` sub-steps)
- 0.48.0 close-out — drop suffix, todo→Done, Migration A table
  all-done

`chid` / `desc` / `list` / `show` not in this cycle — they ride
the separate CommonArgs sweep (Migration A + B entangled).

### Out of scope

Typed errors, returned-outcomes-vs-`println!`, `ProgressSink`,
`Context` fields beyond `UserConfig` + `--log` — deferred until a
real consumer surfaces.

### Per-substep contract

Per `notes/substep-protocol.md`: `cargo fmt` / `clippy
--all-targets -- -D warnings` / `test` / `install --path .
--locked` + retest before each commit; bump `Cargo.toml` at
sub-step start; flip todo ladder markers; pair commits across
both repos with ochid trailers.

## refactor: symlink → Context + SymlinkParams (0.48.0-1)

Commits: [[16]]

Step 1 of the Migration A sweep. `symlink` is the cycle's
warm-up: no `UserConfig`, no `--log` use, no `symlink()` test
callers — a clean mechanical port.

- `SymlinkParams` (flat: `target` / `symlink_dir` / `list` /
  `yes`) + `impl From<&SymlinkArgs>` (total) in `symlink.rs`.
- `pub fn symlink(args)` → `pub fn symlink(_ctx: &Context,
  params: &SymlinkParams)`; `ctx` unused (uniform-signature
  placeholder), body reads `params.*`.
- `main.rs` `Symlink` arm: build `Context::load(cli.log)` +
  `SymlinkParams::from` (same shape as the init / finalize arms).
- Added the missing `//!` module docstring and the missing
  `///` on `symlink()` (pre-existing gaps, fixed in passing).
- Tests untouched — they exercise `SymLink` / `encode_path` /
  `probe`, not the subcommand fn.

## refactor: clone → Context + CloneParams (0.48.0-2)

Commits: [[17]]

Step 2 of the Migration A sweep. Same shape as `symlink`:
`clone` uses neither `UserConfig` nor `--log`, and its tests
parse `CloneArgs` rather than calling the subcommand fn.

- `CloneParams` (flat: `target` / `name` / `scope` / `dry_run`)
  + `impl From<&CloneArgs>` (total) in `clone.rs`.
- `pub fn clone_repo(args)` → `pub fn clone_repo(_ctx: &Context,
  params: &CloneParams)`; `ctx` unused (uniform-signature
  placeholder), body reads `params.*`.
- `main.rs` `Clone` arm: build `Context::load(cli.log)` +
  `CloneParams::from`.
- `clone_one` / `clone_dual` (`pub(crate)` helpers) and the
  tests unchanged.
- File already had its `//!` docstring and `///` on `clone_repo`
  — no doc-comment gaps to fix.

## refactor: sync → Context + SyncParams (0.48.0-3)

Commits: [[18]]

Step 3. First port where the args struct threads into private helpers.

- `SyncParams` (`quiet`,`bookmark`,`remote`,`no_check`,`scope`) +
  `From<&SyncArgs>`; drops `--check` (op reads only `no_check`).
- `sync(args)` → `sync(_ctx, params)`; `ctx` unused (signature
  placeholder).
- `&SyncArgs`→`&SyncParams` in `sync_repos`/`run_plan`/`act_on_state`
  and `resolve_args_to_repos`→`resolve_params_to_repos`.
- `main.rs` `Sync` arm: `Context::load(cli.log)` + `SyncParams::from`.
- `sync/integration_tests.rs`: `apply_args`→`apply_params` (drops
  `check`); `sync/tests.rs` untouched (clap-parse only).
- Added missing `//!` docstring; `sync()` doc `-R`→`--scope`.

## refactor: port validate-desc to Context (0.48.0-4)

Commits: [[19]]

Step 4. Straight port — no helper threading, no test callers.

- `ValidateDescParams` (mirrors `ValidateDescArgs`: `pos_rev`,
  `pos_count`, `revision`, `limit`, `repo`, `other_repo`, `id_len`)
  + `From<&ValidateDescArgs>`.
- `validate_desc(args)` → `validate_desc(_ctx, params)`; `ctx`
  unused (signature placeholder), body reads `params.*`.
- `main.rs` `ValidateDesc` arm: `Context::load(cli.log)` +
  `ValidateDescParams::from`.
- Added missing `//!` module docstring and `///` on `validate_desc()`.
- Tagalong: `CLAUDE.md` codifies the 50/72 commit-message rule
  (subject ≤50, body lines ≤72).

## refactor: port fix-desc to Context (0.48.0-5)

Commits: [[20]]

Step 5. Straight port — no test callers; the only threading is
the private `jj_describe` helper, which already took the repo
path by value (no signature change there).

- `FixDescParams` (mirrors `FixDescArgs` field-for-field:
  `pos_rev`, `pos_count`, `revision`, `limit`, `max_fixes`,
  `repo`, `other_repo`, `id_len`, `title`, `fallback`,
  `no_dry_run`, `add_missing`) + `From<&FixDescArgs>` (total).
- `fix_desc(args)` → `fix_desc(_ctx, params)`; `ctx` unused
  (signature placeholder), body reads `params.*`.
- `main.rs` `FixDesc` arm: `Context::load(cli.log)` +
  `FixDescParams::from` (same shape as the `ValidateDesc` arm).
- Added missing `//!` module docstring and `///` on
  `fix_desc()` (pre-existing gaps, fixed in passing).

## refactor: port push to Context (0.48.0-6)

Commits: [[21]]

Step 6 — last subcommand port before close-out. Biggest module
(`PushArgs` threaded through `push_in` / `run_from` / `run_stage`
/ every `stage_*` / `resolve_message`); uniform plumbing, no
behavior change. One step, not decomposed into `-6.N`.

- `PushParams` + `From<&PushArgs>` in `push.rs`; `from` keeps the
  `Stage` domain type.
- `push(args)` → `push(_ctx: &Context, params: &PushParams)`;
  `ctx` stops at `push()` (`push_in` + stage fns take
  `&PushParams` only, like `sync_repos` / `&SyncParams`). `args`
  → `params` through the threaded fns.
- `main.rs` `Push` arm: `Context::load(cli.log)` +
  `PushParams::from`.
- `push/integration_tests.rs`: `test_args` → `test_params`;
  `push/tests.rs` untouched (clap-parse tests on `PushArgs`).
- Two `PushArgs` quirks the port had to absorb —
  [dual bookmark parameters](#push-dual-bookmark-parameters) and
  the [unimplemented `recheck` flag](#push-unimplemented-recheck-flag)
  — both also queued in `notes/todo.md`.

### push: dual bookmark parameters

`PushArgs` carries two fields for one logical value —
`bookmark_pos` (positional `BOOKMARK`) and `bookmark`
(`--bookmark` flag), mutually `conflicts_with`. `From<&PushArgs>`
collapses them: `bookmark_pos.clone().or_else(|| bookmark.clone())`.
That `or_else` is a smell forced by the CLI shape (a positional
*and* a flag for the same thing). Better: a single positional
with `--bookmark` as a true clap alias, or drop one spelling.
Queued as a todo.

### push: unimplemented `recheck` flag

`PushArgs.recheck` (`--recheck`, "re-run preflight on resume") is
parsed but never read — the stage machine has no
skip-preflight-on-resume path. Mirrored into `PushParams` with
`#[allow(dead_code)]` to keep the conversion total, but a dead
field plus an `allow` is debt. Implement the behavior or remove
the flag. Queued as a todo.

## chore: close Migration A sweep cycle (0.48.0)

Commits: [[22]]

Migration A sweep complete. Every standalone subcommand now runs
the `fn x(&Context, &XxxParams)` shape with a `From`/`TryFrom`
at the binary edge — `init` (0.44.0, worked example), `finalize`
(0.46.0, `TryFrom` + `--log` onto `Context`), and the `0.48.0`
ladder: `symlink` (-1), `clone` (-2), `sync` (-3),
`validate-desc` (-4), `fix-desc` (-5), `push` (-6).

- Remaining: `chid` / `desc` / `list` / `show` — their
  Migration A is bundled into the future "CommonArgs sweep"
  (A + B entangled for the `CommonArgs`-flattening subcommands).
- Two `push` design warts surfaced and queued as todos — see
  [dual bookmark parameters](#push-dual-bookmark-parameters) and
  [unimplemented `recheck` flag](#push-unimplemented-recheck-flag).

## docs: por-dual capture + icr cleanup (0.48.1)

Commits: [[23]]

Docs-only follow-on to the 0.48.0 close-out (same shape as
the 0.42.1 docs cycle). Records the por/dual parity +
bidirectional-conversion goal as a forward-looking design
stub, and clears stale `init-clone-refactor` verbiage now
that the branch has landed in `main` (rebased at 0.42.0-4.7)
and been deleted local + remote. No code change.

## docs: chores edit list → commit message (0.48.2)

Commits: [[24]]

Move the per-file edit list out of `chores-NN.md` sections
into the commit message body — git becomes the source of
truth for what each commit changed mechanically. A chores
section now keeps only the narrative + `###` design
subsections, cites its commit(s) via a `Commits:` ref, and —
when it records a commit, as does a `todo.md > ## Done` entry —
takes its header verbatim from that commit's title
(organizational headings and design subsections don't).
CLAUDE.md and `notes/README.md` codify it; the `0.48.1`
section here and its `todo.md > ## Done` entry are converted
as the worked example, this section is born in the new shape
(its own `Commits:` ref backfilled at the next cycle's start).

### Why git owns the edit record

- **DRY.** The old setup had the commit body mirror a chores
  `### Edits` list — the same content twice, which drifts.
  Git's copy is immutable, `git show`-able, commit-scoped; it
  wins as canonical. Chores keeps what git is bad at — the
  why.
- **`Commits:` ref shape.** Markdown reference-link
  definitions put the destination first, so to keep the
  `[[N]]` citation clickable the `[N]:` definition's
  destination is the URL (`…/commit/<12-hex>` — short, and
  GitHub / GitLab resolve a unique prefix), and the full
  40-hex SHA rides in the title slot: host-agnostic,
  unambiguous, and present in the raw markdown for external
  tooling (a database, say) to scrape.
- **Backfill timing.** A commit's URL/SHA don't exist when
  its section is written, so `Commits:` is filled in at the
  *next* cycle's start — the newest section is briefly
  ref-less, which is fine (the commit itself is the record).
- **Doubled citations.** Reference *citations* are written
  `[[N]]` (not `[N]`) so the brackets render — `[27]` reads
  as a reference, `27` doesn't. `[N]:` definitions and inline
  links stay single-bracketed. `todo.md` is fully retrofitted;
  `chores-01..08` grandfathered (chores-09 retrofitted in
  0.48.3).

## docs: chores-09 → new shape (0.48.3)

Commits: [[25]]

Retroactively convert every commit-recording section in
`chores-09.md` to the 0.48.2 convention: section header =
exact commit title; `Commits: [[N]]` first-line citing the
section's commit; `### Edits` subsections removed (the commit
body is the record). Line 114's bare `[76]` (a
foreign-namespace citation attempt) → a `chores-09`-local ref
to the chores-07 `--scope` design section. Lines 38–40's
renumbering description left code-spanned (literal historical
tokens, no stable current target). `notes/README.md` codifies
the footnote framing — ref numbers are file-local slots; a
`[N]` in a code span is a quoted identifier (data), not a
citation; cross-file pointing uses an inline link.

### Why retrofit one file

- **Surfaced by adding `# References`.** 0.48.2 gave
  `chores-09` a `# References` section; a reader then expects
  it to define the page's `[N]` mentions — but most were
  `todo.md`'s / `done.md`'s numbers written into `chores-09`
  prose, a category error. Half-implemented was the worst of
  both; either the page runs the convention fully or it
  doesn't.
- **Bounded.** One file, 21 historical commit-recording
  sections, ~10 back-refs (`todo.md` `[85]`–`[88]`, `done.md`
  `[79]`/`[81]`/`[84]`, two prose mentions in
  `init-clone-refactor-conflict.md`). `chores-01..08` have
  similar vestigials but no `# References` forcing the issue —
  they stay grandfathered (the convention sunsets their
  `### Edits` lists by attrition, not retrofit).

## docs: renumber todo.md + chores-09 refs (0.48.4)

Commits: [[26]]

Compact two files' `# References` to a contiguous `[1]..[N]` in
first-citation-appearance order — walk the file's prose in
document order (`todo.md`: `## Todo` items then `## Done` items;
`chores-09.md`: top to bottom) and assign `[1]`, `[2]`, … as
each ref's first `[[N]]` citation appears.

- **`todo.md`** — the 0.48.2/0.48.3 retrofits and routine entry
  churn left a sparse namespace (gaps from pruned entries,
  numbers into the 90s); the re-pack makes the next new ref
  `[N+1]`, not `[96]`.
- **`chores-09.md`** — the 0.48.2/0.48.3 bulk retrofit allocated
  slots in *conversion* order, not document order: `[1]` was the
  first section converted (the 0.48.1 section, near the bottom),
  `[2]..[22]` the historical sections top-to-bottom, `[24]` the
  chores-07 design ref (first cited near the top). The re-pack
  puts the numbers back in reading order — `[1]` is the topmost
  citation.

Properties of the renumber:

- **File-local.** Only the file's own `[[N]]` citations and
  `[N]:` definition lines move; every `[N]:` *target* and every
  other note file is untouched (ref numbers are file-local
  slots — `done.md` keeps its own numbering).
- **First-citation order, not definition order.** The number a
  reader meets first is `[1]`; following a citation walks *down*
  the `# References` list, never up.
- **Code spans are not citations.** A `[[N]]` inside a `` ` ``
  span (e.g. this section's `[[9]],[[13]],[[14]]` example) is a
  literal token, not a reference — the script masks fenced and
  inline code before scanning.
- **Cosmetic: multi-ref groups sorted ascending** — a cluster is
  written `[[9]],[[13]],[[14]]`, not in the order the refs
  happened to land.

### When to re-pack a file's references

- A renumber is a whole-file rewrite of citations + defs — do it
  opportunistically (when the namespace has drifted enough to
  annoy) rather than under a standing "keep it dense" rule that
  would force a rewrite on every prune.
- `todo.md` is the live churn surface (entries land and get
  pruned every cycle) so it fragments fastest and is the usual
  candidate. `chores-NN.md` / `done.md` are append-mostly; their
  numbering only drifts out of reading order after an unusual
  event like the 0.48.2/0.48.3 retrofit — which is exactly why
  `chores-09` got re-packed here and the other chores files
  don't need it.

# References

[1]: https://github.com/winksaville/vc-x1/commit/bdec8579c28b "bdec8579c28b76989e52807a9e6bba93ba301c96"
[2]: https://github.com/winksaville/vc-x1/commit/bb27daa86a07 "bb27daa86a078e2e06ebc56e0159e89829fb6356"
[3]: /notes/chores-07.md#--scope-enum-refactor-0420
[4]: https://github.com/winksaville/vc-x1/commit/7880bfe2700d "7880bfe2700d8ce47946e5d9f6bf7e05e34de5c6"
[5]: https://github.com/winksaville/vc-x1/commit/c1525175948c "c1525175948c1a97118996bb0c997c37c81351ff"
[6]: https://github.com/winksaville/vc-x1/commit/bc97a768e643 "bc97a768e6437279599311eb57ee3eb66b549295"
[7]: https://github.com/winksaville/vc-x1/commit/29b2e9c73b2d "29b2e9c73b2d9cb29b7b05f5389b02338b7dab43"
[8]: https://github.com/winksaville/vc-x1/commit/fc7b518a0731 "fc7b518a07319a40b580e149c3900b1b7bae27ad"
[9]: https://github.com/winksaville/vc-x1/commit/3a0fa145fc69 "3a0fa145fc694477180179f58b1fb71043d070a8"
[10]: https://github.com/winksaville/vc-x1/commit/545c7c725b31 "545c7c725b3143d6ed1e92b870e8370362eaafdf"
[11]: https://github.com/winksaville/vc-x1/commit/be7f2502dded "be7f2502ddedbe6413212ef9df6bf93852c1a538"
[12]: https://github.com/winksaville/vc-x1/commit/e966d65feef8 "e966d65feef8b93791dcfef1f3c64a37865ffb32"
[13]: https://github.com/winksaville/vc-x1/commit/d02782e14eff "d02782e14eff44a969b45644bd893ec7bf8e5bb5"
[14]: https://github.com/winksaville/vc-x1/commit/aad07a58e033 "aad07a58e033226734465078966cbfdde7dc48c7"
[15]: https://github.com/winksaville/vc-x1/commit/c4daeef03551 "c4daeef0355187deac2f261572bc71ca3a0ef345"
[16]: https://github.com/winksaville/vc-x1/commit/6a5299ea624d "6a5299ea624dbcbe87e115258656baf5d96010ac"
[17]: https://github.com/winksaville/vc-x1/commit/0529aa50c877 "0529aa50c8777506d65232caa61a33f65d81d11a"
[18]: https://github.com/winksaville/vc-x1/commit/a2a30f7ed1c1 "a2a30f7ed1c1c25cd164a4bcf0713f27737bb006"
[19]: https://github.com/winksaville/vc-x1/commit/aa0ee07c7144 "aa0ee07c714448678651fab8a16b67fcc083ab8d"
[20]: https://github.com/winksaville/vc-x1/commit/88b0e5c3b114 "88b0e5c3b114f799928d1d593aecdad46e1179cd"
[21]: https://github.com/winksaville/vc-x1/commit/c9d89386fb06 "c9d89386fb06876d1ee29e5cf370acc3c83caf66"
[22]: https://github.com/winksaville/vc-x1/commit/e284505c346c "e284505c346ca051e03d36889389e49229252904"
[23]: https://github.com/winksaville/vc-x1/commit/3f176b45235c "3f176b45235c4bc7da4b8de5b18a7f454c464d1e"
[24]: https://github.com/winksaville/vc-x1/commit/79279d112791 "79279d112791c12190a755b8aec9be1ec174ecb8"
[25]: https://github.com/winksaville/vc-x1/commit/47e5e854c922 "47e5e854c9220503ed46eda25ca40293e31bfdb1"
[26]: https://github.com/winksaville/vc-x1/commit/24488a82d6d4 "24488a82d6d4b50d058406589d2850b9ced88e25"
