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
  the ops-layer architecture, etc.).
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

## Ops layer architecture (forward-looking)

Design target for subsequent cycles: separate clap-aware
parsing from subcommand operation logic so future front-ends
(e.g. a TUI for bot-conversation exploration, or library
embedding) call the same core without retrofitting. Captures
the conclusion of the 0.42.0-4.7 side discussion about
`args.account.account` ergonomics — renaming / accessor
shortcuts were rejected as hiding the architecture mismatch
rather than fixing it.

### Goals

- Subcommand bodies access flat fields (`opts.account`), not
  nested clap shapes (`args.account.account`).
- `src/options_flags/` leaves remain the single source of
  truth for flag definition (parser, help, completer).
- Per-subcommand `-h` stays bundle-specific.
- Tab completion (`vc-x1 init <TAB>`, `--account=<TAB>`)
  keeps working via clap_complete + dynamic completers.
- The ops layer is callable without clap as a dependency
  (today's `plan_init(args: &InitArgs, ...)` is not).

### Two layers, three structures per subcommand

- **CLI layer (clap-aware):** `InitArgs` / `CloneArgs` / etc.
  `#[derive(Args)]` types that flatten leaves from
  `src/options_flags/`. Own clap metadata for `-h` and
  completion. Live at the binary edge.
- **Ops layer (clap-free):** `InitOptions` / `CloneOptions`
  — plain structs, flat fields, `Default`. Plus `Workspace`,
  the shared platform handle (workspace root, loaded
  `UserConfig`, optional progress sink). Entrypoints are
  `ops::init(ws: &Workspace, opts: &InitOptions) -> Result<InitOutcome, InitError>`.
- **Boundary conversion:** `From<&InitArgs> for InitOptions`
  — one contained site per subcommand where leaf nesting is
  unpacked. Op bodies never see `args.xxx.xxx`.

### `Workspace` ("context") vs per-op options

- **`Workspace`:** shared platform every op runs against —
  workspace root, loaded user config, optional progress
  sink. Same shape across subcommands.
- **`InitOptions`** etc.: per-op input; shape differs by
  subcommand.

Two parameters, not a single merged "god context." The
signature `fn init(ws: &Workspace, opts: &InitOptions)`
documents what the op depends on; a merged `Context`
containing every possible field would let any op silently
read any field with no signature-level visibility.

If `Workspace` itself grows, escape valves (defer until
needed):

- Split into `Workspace` (paths + config, read-only, cheap)
  vs `Session` (progress / cancellation, mutable). Most ops
  want `&Workspace`; long-running ops also accept
  `&Session`.
- Trait-based DI (`fn init<W: HasConfig + HasFs>(...)`) for
  multiple front-ends with genuinely different platform
  surfaces. Heavy in Rust; not the default.

### Five rules for the ops layer

1. **Plain options, flat fields, `Default`.** No clap types
   in `InitOptions`. Domain values like `RepoSelector` are
   fine (they're domain types); leaf wrappers like
   `RepoOption` are not.
2. **Typed errors.** `enum InitError` (likely via
   `thiserror`), not `Box<dyn Error>`. A TUI matches
   variants to pick dialogs; CLI formats them.
   `Box<dyn Error>` discards exactly the information a GUI
   most needs.
3. **Returned outcomes, not `println!`.** Each op returns a
   structured result; CLI formats it for stdout, TUI
   populates panels. Library writes nothing to stdout
   itself.
4. **Progress via a sink.** Long-running ops accept an
   optional `&mut dyn ProgressSink` (or
   `mpsc::Sender<Event>`). CLI installs a stderr sink; TUI
   a status-bar sink; tests a recording sink. `log` /
   `tracing` covers diagnostics but is lossy for structured
   progress.
5. **No globals, no implicit cwd/env reads in ops.**
   Everything an op needs goes in `Workspace` or
   `*Options`. CLI resolves cwd/env once at startup and
   builds the handle. This is what lets a multi-window TUI
   drive multiple workspaces without cross-contamination.

### Completion stays at the clap layer

- Static structure (`-h`, subcommand layout, flag presence):
  clap derive on `InitArgs` + leaves. Unchanged from today.
- Dynamic value completion (`--account=<TAB>` against
  accounts in user config): `ArgValueCompleter` attached to
  the leaf in `src/options_flags/`. Already enabled via
  `clap_complete`'s `unstable-dynamic` feature. The leaf is
  the right home — completion is a clap-aware concern with
  no place in `InitOptions`.

Ops layer stays clap-free; leaf layer stays clap-aware.
Completion drives nothing in the ops layer.

### The contained wart

`From<&InitArgs> for InitOptions` walks leaf nesting once
per subcommand:

```rust
impl From<&InitArgs> for InitOptions {
    fn from(a: &InitArgs) -> Self {
        Self {
            target: a.target.clone(),
            account: a.account.account.clone(),
            scope: a.scope.scope,
            private: a.provision.private.private,
            // ...
        }
    }
}
```

Verbose but contained: one site per subcommand. Adding a new
flag becomes a two-edit change (clap struct + options struct
+ one conversion line). Accepted price of decoupling; bodies
are flat thereafter.

### What this is *not*

- **Not a god `Context`.** Two parameters
  (`&Workspace`, `&XOptions`), not one merged blob.
- **Not premature trait-based DI.** Concrete `Workspace` +
  `*Options` structs until a second front-end forces
  generalization.
- **Not a crate split.** Same crate, separate modules
  (e.g. `src/ops/init.rs` vs CLI edge). Promote to a
  workspace only when a second consumer crate appears.
- **Not removing `src/options_flags/`.** Leaves stay; only
  their consumers change shape.

### Migration sketch

The bot thinks the safest first move is converting one
subcommand end-to-end as the worked example before any
sweep. `init` is the largest surface; `sync` or `chid` would
be a lighter proof of concept.

1. Introduce `ops::Workspace` (paths + `UserConfig` only; no
   progress sink yet). Wire it into one subcommand.
2. Introduce `XOptions` for the chosen first subcommand;
   write `From<&XArgs> for XOptions`; port the body to
   `&Workspace, &XOptions`.
3. Add typed errors + returned outcome for that subcommand.
4. Sweep remaining subcommands; each is a contained step.
5. (Later) introduce `ProgressSink` when a TUI need actually
   surfaces, or a long-running op wants structured progress.

The bot thinks the right shape of `Workspace` matters more
than the right shape of `InitError` — defer error-type and
outcome-type design until step 1 has shaken out.

### Open questions

- `Workspace` carries the progress sink from day one even
  with no consumer? The bot's guess is no — leave it out and
  add when a real long-running op forces it; adding later is
  mechanical.
- `XOptions` owns values (`String`) or borrows (`&str`)? The
  bot's guess is owns — simpler, matches the programmatic-
  caller path (build then call). Defer borrow optimization
  until benchmarks call for it.
- Crate split timing — defer until a second consumer crate
  appears.
