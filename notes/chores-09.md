# Chores-09.md

General chores notes — design captures (forward-looking) and
post-implementation chore entries. Same shape as chores-01..08.md;
09 starts here because chores-07 (0.42.0 cycle, 600+ lines) and
chores-08 (0.41.1 cycle, 1500+ lines) are both already large; the
init-clone-refactor rebase landing is a natural new-file
boundary.

Subsection headers use the trailing-version format from CLAUDE.md
when they correspond to a release: `## Description (X.Y.Z)`.

## init-clone-refactor rebase landing (0.42.0-4.7)

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

### Edits

- `notes/chores-09.md`: this file (new).
- `notes/todo.md`: ladder marker flips
  (-4.6 (current)→(done), add -4.7 (current)); add `-4.7`
  Done entry referenced as `[79]`; add `[79]:` ref target;
  move pre-0.42.0 Done entries to `notes/done.md`.
- `notes/done.md`: append migrated 0.40/0.41 Done entries.
- `Cargo.toml`: bump 0.42.0-4.6 → 0.42.0-4.7.

## 0.42.0 close-out

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
at chores-07 [76]:

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

### Edits

- `notes/todo.md`: 0.42.0 ladder removed from `## In
  Progress`; consolidated `--scope` continuation TODO
  entry added; four existing scope-related entries
  updated to drop "0.42.0 cycle" claims; `0.42.0 cycle
  close-out` Done line added; `[81]` reference target
  added.
- `notes/chores-09.md`: this subsection (new).
- `Cargo.toml`: bump 0.42.0-4.7 → 0.42.0.

## Design notes: bot-data + multi-user updates (0.42.1)

Documentation-only follow-on to the 0.42.0 close-out.
Forward-looking design captures for multi-user
collaboration, multi-bot vendor support, and bot-repo
scaling thresholds. No code change.

### Edits

- `notes/bot-data-formats.md`: new file. Format-agnostic
  principle, dual-repo merits-based defense,
  vendor-subdir layout
  (`.bot/<vendor>/<version>/<id>.<ext>`), multi-bot in
  one repo, format versioning, flat-to-vendor migration,
  `.claude` → `.bot` rename, viewer layer, open
  questions.
- `notes/forks-multi-user.md`: four new subsections
  (bot-repo size and scaling thresholds;
  monotonic-growth asymmetry; mitigation menu; tracking
  trigger). One new subsection on URL-shaped ochid for
  per-user repos (link-rot mitigations: project-side
  mirroring, cryptographic stapling, CI-enforced live
  ochid). Cross-ref to `bot-data-formats.md` added in
  intro.
- `notes/todo.md`: replace multi-user TODO entry with
  `forks-multi-user + bot-data-formats follow-through`;
  add `[82]` / `[83]` refs to the two design docs
  (slight extension of the existing
  `chores-NN.md#anchor` reference style for whole-file
  pointers).
- `README.md`: TOC entry +
  `## Thoughts for the future` section pointing at
  `notes/forks-multi-user.md`. Reader chain: README →
  forks-multi-user → bot-data-formats.
- `notes/chores-09.md`: this subsection (new).
- `Cargo.toml`: bump 0.42.0 → 0.42.1.

## Test-module extraction (0.43.0)

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

### Edits (this sub-step, 0.43.0-0)

- `Cargo.toml`: bump 0.42.1 → 0.43.0-0.
- `notes/chores-09.md`: this section (new).
- `notes/todo.md`: open `## In Progress` ladder for the
  cycle; mark -0 `(current)`; add `[84]` ref pointing
  here; add `[84]` to the existing Test-module
  extraction TODO entry.

## init test extraction (0.43.0-1)

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

### Edits

- `src/init.rs`: production code retained at the original
  path; trailing `mod tests;` forward declaration added.
- `src/init/tests.rs`: de-indented test body with new
  `//!` header.
- `Cargo.toml`: bump 0.43.0-0 → 0.43.0-1.
- `notes/chores-09.md`: this section (new).
- `notes/todo.md`: ladder marker flips (-0 (current) →
  (done); -1 → (done)).

## push test extraction (0.43.0-2)

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

### Edits

- `src/push.rs`: production code retained at the
  original path; trailing test bodies replaced with
  `mod tests;` + `mod integration_tests;` forward
  declarations.
- `src/push/tests.rs`: de-indented unit test body with
  new `//!` header.
- `src/push/integration_tests.rs`: de-indented
  integration test body; existing file-level `//!`
  block preserved (single new top-line header added).
- `Cargo.toml`: bump 0.43.0-1 → 0.43.0-2.
- `notes/chores-09.md`: this section (new).
- `notes/todo.md`: ladder marker flips (-2 (current) →
  (done)).

## sync test extraction (0.43.0-3)

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

### Edits

- `src/sync.rs`: production code retained at the
  original path; trailing test bodies replaced with
  `mod tests;` + `mod integration_tests;` forward
  declarations; the test-only
  `#[cfg(test)] use crate::scope::Side;` lines
  removed.
- `src/sync/tests.rs`: de-indented unit test body with
  new `//!` header; explicit `use crate::scope::Side;`
  added under `use super::*;`.
- `src/sync/integration_tests.rs`: de-indented
  integration test body; existing file-level `//!`
  block preserved (single new top-line header added);
  explicit `use crate::scope::Side;` added with the
  rest of the imports.
- `Cargo.toml`: bump 0.43.0-2 → 0.43.0-3.
- `notes/chores-09.md`: this section (new).
- `notes/todo.md`: ladder marker flips (-3 (current) →
  (done)).

## common test extraction (0.43.0-4)

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

### Edits

- `src/common.rs`: production code retained at the
  original path; trailing test body replaced with
  `mod tests;` forward declaration.
- `src/common/tests.rs`: de-indented unit test body
  with new `//!` header.
- `Cargo.toml`: bump 0.43.0-3 → 0.43.0-4.
- `notes/chores-09.md`: this section (new).
- `notes/todo.md`: ladder marker flips (-4 (current) →
  (done)).

## 0.43.0 close-out

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

### Edits (this commit)

- `Cargo.toml`: bump 0.43.0-4 → 0.43.0.
- `notes/chores-09.md`: this section (new).
- `notes/todo.md`: drop the cycle's `## In Progress`
  ladder; add a `## Done` entry; remove the
  Test-module extraction `## Todo` item.

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

## InitParams implementation (0.44.0)

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

### Edits

- `Cargo.toml`: bump 0.43.0 → 0.44.0.
- `src/context.rs` (new): `Context` struct holding
  `user_config: UserConfig` + `Context::load()` ctor;
  declared `mod context;` in `src/main.rs`.
- `src/init/params.rs` (new): `InitParams` flat struct
  + `impl From<&InitArgs> for InitParams` (production
  default sets `create_symlink: true`); declared
  `mod params; pub use params::InitParams;` in
  `src/init.rs`.
- `src/init.rs`: `pub fn init` signature changed
  from `init(args: &InitArgs, create_symlink: bool)`
  to `init(ctx: &Context, params: &InitParams)`
  (`create_symlink` folded into `InitParams`).
  Internal helpers (`plan_init`, `plan_from_url`,
  `plan_from_path`, `plan_from_bare_name`,
  `plan_remote`, `plan_local`, `build_plan`,
  `create_por`, `create_dual`, `push_repo`,
  `run_remote_step`) all switched from
  `args: &InitArgs` to `params: &InitParams`. Body
  reads `params.account` instead of
  `args.account.account` (and similarly across all
  leaf nestings). `args.config.resolve(ConfigKind::None)`
  call sites collapse to `&params.config` since the
  resolve runs at the boundary now. `cfg` inside
  `pub fn init` switched from `config::load()?` to
  `&ctx.user_config`.
- `src/main.rs`: `mod context;` added; init dispatch
  builds `Context::load()` then converts
  `InitParams::from(&init_args)` and calls
  `init::init(&ctx, &params)`.
- `src/test_helpers.rs`: `Fixture` and `FixturePor`
  builders updated — load `Context`, build
  `InitParams::from(&args)`, override
  `params.create_symlink = false`, then call
  `init(&ctx, &params)`. Imports updated to bring
  `Context` and `InitParams` into scope.
- `src/init/tests.rs`: every `plan_init(&args, ...)`
  call site rewritten to
  `plan_init(&InitParams::from(&args), ...)`. The two
  `plan_init(&args_for(...), ...)` sites split across
  multiple lines for readability. Tests of `InitArgs`
  (parse/defaults) untouched — those exercise the
  clap edge.
- `notes/chores-09.md`: this section + the
  `## InitParams implementation (0.44.0)` plan
  section.
- `notes/todo.md`: drop the cycle's `## In Progress`
  entry; add `## Done` entry; rewrite the
  `Ops layer / CLI decoupling` `## Todo` item to
  reflect "init done, sweep remaining."

## Architecture doc and terminology reconciliation (0.45.0)

Single-step docs cycle. Promotes the forward-looking
"Ops layer architecture" capture (this file) into a living
`ARCHITECTURE.md` at the repo root, and reconciles the
terminology: the design capture said `Workspace` / `XOptions`
and "ops layer"; the 0.44.0 implementation shipped `Context`
/ `XxxParams`. Settled on "subcommand layer" everywhere. No
code change beyond a doc comment.

### Edits

- `Cargo.toml`: bump 0.44.0 → 0.45.0.
- `ARCHITECTURE.md` (new): module map (CLI layer +
  subcommand-layer scaffolding + subcommand modules), the
  two-layer split (`XxxArgs` ↔ `Context` + `XxxParams`,
  boundary `From<&XxxArgs>`), the subcommand model, and
  Migrations A (args → Context/Params) / B (per-subcommand
  flags → `src/options_flags/`) with status tables; plus a
  `### Naming` subsection.
- `README.md`: TOC entry + `## Contributing` opening
  paragraph pointing at `ARCHITECTURE.md`.
- `notes/README.md`: intro paragraph pointing at
  `../ARCHITECTURE.md`.
- `src/options_flags/README.md`: intro note — this module
  is the "Migration B" leaf store; points at
  `../../ARCHITECTURE.md`.
- `notes/chores-09.md`: "Ops layer architecture
  (forward-looking)" section trimmed to a stub pointing at
  `ARCHITECTURE.md` (keeps the load-bearing conclusions);
  the InitParams section's `### Naming differs …`
  subsection collapsed to a brief pointer; the InitParams
  cross-ref + the 0.43.0 Goals' "ops-layer" mention renamed
  to "subcommand layer"; this subsection.
- `notes/todo.md`: `## Todo` #1 renamed `Ops layer / CLI
  decoupling` → `Subcommand layer / CLI decoupling` (and
  "op body" → "subcommand body"); `## Done` entry added;
  `[80]` repointed to `/ARCHITECTURE.md`; `[85]` ref added
  for this subsection.
- `notes/bot-data-formats.md`: "ops-layer / CLI decoupling"
  → "subcommand-layer / CLI decoupling"; cross-ref
  repointed from `chores-09.md > Ops layer architecture` to
  `ARCHITECTURE.md`; `Workspace` handle → `Context` handle.
- `src/context.rs`: module doc — "op layer" → "subcommand
  layer"; pointer repointed from `notes/chores-09.md > ##
  Ops layer architecture …` to `ARCHITECTURE.md`.

## finalize subcommand-layer migration (0.46.0)

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

### Edits

- `Cargo.toml`: bump 0.45.0 → 0.46.0.
- `src/context.rs`: `Context` gains `pub log: Option<PathBuf>`;
  `Context::load()` → `Context::load(log: Option<PathBuf>)`;
  doc comments updated.
- `src/finalize.rs`: added module `//!` docstring (had none);
  `FinalizeOpts` → `FinalizeParams` (dropped its `log` field —
  moved to `Context`); `FinalizeArgs::into_opts(self, log)`
  replaced by `impl TryFrom<&FinalizeArgs> for FinalizeParams`;
  `finalize` / `detach` now `(ctx: &Context, params:
  &FinalizeParams)` and read `ctx.log`; `build_exec_args(params,
  log: Option<&Path>)`; `preflight` / `finalize_exec` /
  `log_plan` / `write_failure_marker` param renamed
  `opts` → `params`; tests updated, +2
  (`try_from_canonicalizes_repo`, `try_from_bad_squash`).
- `src/main.rs`: init arm passes `cli.log` to `Context::load`;
  finalize arm builds `Context::load(cli.log)` +
  `FinalizeParams::try_from(&finalize_args)` +
  `finalize::finalize(&ctx, &params)`.
- `src/test_helpers.rs`: `Context::load()` → `Context::load(None)`
  (the two fixture builders).
- `ARCHITECTURE.md`: Migration A table — `finalize` `partial` →
  `done (0.46.0)`; "Boundary conversion" bullet now states the
  `From` (total) vs `TryFrom` (fallible) rule; `Context` bullet
  mentions the `--log` path; subcommand-modules table updated;
  Migration A intro notes the planned `0.47.0-N` multi-step
  cycle for the remaining nine.
- `notes/todo.md`: `## Todo` #1 updated (finalize done; remaining
  nine → `0.47.0-N` plan); new `## Todo` item for Migration B on
  finalize (`--squash` → shared `options_flags` leaf, since
  `vc-x1 push --squash` will reuse it); `## Done` entry added;
  `[86]` ref added.
- `notes/chores-09.md`: this subsection.

## finalize Migration B — squash options_flags leaf (0.47.0)

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

## Migration A sweep: subcommand-layer ports (0.48.0)

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

### Edits (this sub-step, 0.48.0-0)

- `Cargo.toml`: 0.47.0 → 0.48.0-0.
- `notes/chores-09.md`: this section.
- `notes/todo.md`: cycle ladder opened in `## In Progress`
  (-0 `(current)`); `[88]` ref added; `[88]` added to the
  "Subcommand layer / CLI decoupling" TODO entry.

## symlink → Context + SymlinkParams (0.48.0-1)

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

## clone → Context + CloneParams (0.48.0-2)

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

## sync → Context + SyncParams (0.48.0-3)

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
