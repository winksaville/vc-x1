# Chores-11

Continuation of `chores-10.md` (which is closed at `0.50.0` —
the Subcommand trait sweep). This file covers the `0.51.0`
cycle onward. Reference numbering is file-local — see
[`README.md`](../README.md#reference-numbering); chores-11
starts at `[1]`.

## chore: move chores under notes/chores/ (0.51.0)

Commits: [[1]]

Single-step. `notes/` was dominated by ten chores files plus
the freshly opened `chores-11.md`; the eleven files now move
under `notes/chores/`, leaving `notes/` for the live notes
(`todo.md`, `done.md`, `README.md`, design docs). Filenames
stay `chores-NN.md` — preserving them keeps section-anchor
slugs unchanged and reduces the cross-file ref update to a
single mechanical substitution.

Done.md hygiene rides along: the eleven `0.44.0`–`0.50.0`
entries that had been sitting in `todo.md > ## Done` migrate
into `done.md` (renumbered `[85]`–`[95]`), since the 0.50.0
close-out marks a natural batching point and keeps `todo.md`
focused on what's actually in flight.

### Substitutions applied

External callers (everything outside `notes/chores/`):

- `/notes/chores-NN.md` → `/notes/chores/chores-NN.md` —
  absolute-path refs in `notes/todo.md`, `notes/done.md`,
  `notes/README.md`, the sibling design notes, `CLAUDE.md`,
  `ARCHITECTURE.md`, the root `README.md`, and the
  GitHub-URL fragments in `src/clone.rs` /
  `src/config.rs` / `src/init.rs` / `src/init/params.rs`
  / `src/push.rs` / `src/scope.rs` / `src/subcommand.rs` /
  `src/sync.rs`.

Within-chores siblings (now under `notes/chores/`):

- `(../ARCHITECTURE.md` → `(../../ARCHITECTURE.md` (the
  root `ARCHITECTURE.md` is two `..` away now).
- `(../src/…` → `(../../src/…`.
- `(README.md` → `(../README.md` (sibling
  `notes/README.md`).
- `(forks-multi-user.md` → `(../forks-multi-user.md` and
  the `[1]: forks-multi-user.md` ref def in
  `chores-08.md` (similar fixups for
  `vc-x1-init.md` / `cargo-locked-issue.md`).

In-chores cross-refs to other chores files (now siblings
in the same `chores/` dir) — `[chores-NN.md](chores-NN.md#…)`
style — are unchanged.

### Why this shape

`/notes/chores/` (under notes/) rather than top-level
`/chores/` keeps the notes umbrella intact —
`todo.md` / `done.md` / `README.md` are conceptually
adjacent to chores and benefit from sharing the
directory. The `chores-` filename prefix is technically
redundant once the files live in `chores/` (could become
`01.md` / `02.md` / …), but the rename was rejected to
keep the diff focused: section-anchor slugs are
filename-independent, so the rename would buy no
ref-resolution simplification and would force renumbering
of every `[N]:` URL slug in `todo.md`, `done.md`, and the
src `.rs` GitHub fragments.

## chore: open sb_ide elimination (0.52.0-0)

Commits: [[2]]

Multi-step. The `0.50.0` cycle landed every subcommand on
`SubcommandRunner::dispatch` with two trait peek methods
(`suppress_banner` / `is_detached_exec`) and one free
function (`sb_ide`) carrying per-run "session chrome" —
the `vc-x1 X.Y.Z` banner, `finalize::surface_previous_failures`,
and the `bm_track` enter/exit pair. This cycle eliminates
all three by relocating each behavior to where it actually
belongs:

- The `vc-x1 X.Y.Z` banner already lands on `--help` via
  clap's `before_help`; printing it again at the top of
  every normal run is duplicate chatter the user can
  recover from `vc-x1 -V`. Drop the on-every-run banner.
- `finalize::surface_previous_failures` is a finalize-side
  concern; moving its trigger into `finalize` itself
  (rather than into a generic dispatch hook) cleans up the
  gate.
- `bm_track` enter/exit is a diagnostic; demoting it to
  `debug!` matches the "permanent sanity check, but quiet
  by default" intent already recorded in its module doc.

After these relocations the trait's `suppress_banner` /
`is_detached_exec` methods have no remaining consumers,
and `sb_ide` has no body left to keep — all three
disappear. Plan-only opener; the work lands in the
substeps below.

### What goes away — and the reasoning

- **`bm_track` info-level emission.** Currently emits
  `bm-track enter vc-x1 X: app(main)=tracked, …` on every
  run. The module doc already notes "a future refinement
  would remove the steady-state noise without losing
  detection value" — `debug!` accomplishes exactly that.
  Detection value is preserved (run with `-v` when
  investigating); steady-state output gets cleaner.
- **`sb_ide`'s banner-on-every-run.** `clap::Command::before_help`
  already shows the banner on `--help`. The on-every-run
  emission is duplicate; users who want it explicitly have
  `vc-x1 -V`. Removing it shrinks the default output by one
  line per command.
- **`finalize::surface_previous_failures` gated in `sb_ide`.**
  This is finalize machinery — it reads
  `.vc-x1/finalize-fail-*` markers and surfaces them. The
  "don't surface from the detached child" gate is
  finalize's concern, not the trait's; moving the trigger
  into `finalize` itself lets the gate live where the
  marker logic lives.
- **`SubcommandRunner::is_detached_exec`.** With `bm_track`
  at `debug!` and `surface_previous_failures` relocated,
  no remaining consumer needs the trait's detached-exec
  peek. Drop the method; `finalize` reads `params.exec`
  directly as a finalize-private concern.
- **`SubcommandRunner::suppress_banner`.** With the banner
  gone from `sb_ide`, the `-L` flag still suppresses
  bookmark labels in the listing subcommands' output
  (chid/desc/list/show via `CommonParams::header`) — that
  already lives in the op layer, not the chrome layer.
  The trait peek has no remaining consumer; drop it,
  along with the `suppress_banner` field on `ChidParams`
  / `DescParams` / `ListParams` / `ShowParams`.

### Ladder

- 0.52.0-0 plan + version bump + this section + todo
  ladder (current)
- 0.52.0-1 `bm_track` → `debug!`; drop the bm_track gates
  in `SubcommandRunner::dispatch`
- 0.52.0-2..N eliminate `sb_ide`:
  - Drop banner-on-every-run from `sb_ide`
  - Move `finalize::surface_previous_failures` trigger
    into `finalize` itself
  - Delete `sb_ide` once both bodies have moved
- 0.52.0-K drop `SubcommandRunner::is_detached_exec` and
  `SubcommandRunner::suppress_banner` (no remaining
  consumers); drop `suppress_banner` fields from the
  four `Params` types
- 0.52.0 close-out

### Per-step evaluation

Effectiveness is evaluated after each substep. Possible
outcomes at any step boundary:

- Continue as planned.
- Significantly modify the shape (e.g. keep the banner
  somewhere else; preserve `is_detached_exec` for a use
  case discovered mid-cycle) — recorded in a new chores
  subsection at the next step.
- Abandon: revert the cycle to a non-eliminated
  baseline, closing as a no-op with an `### Outcome` note
  capturing what didn't fit.

## refactor: bm_track → debug! (0.52.0-1)

Commits: [[3]]

First substep. `bm_track` enter/exit lines were
emitted at `log::info!`, which lit up on every
command run as `bm-track enter vc-x1 X: app(main)=tracked,
.claude(main)=tracked` plus its matching exit line.
The doc comment recorded the intent to silence the
steady-state noise "at which point a 'silent when
clean' refinement would remove the noise without
losing detection value" — `log::debug!` accomplishes
exactly that: default runs go quiet, `-v` brings the
signal back when investigating.

With the demotion, the `if !is_detached { bm_track(…) }`
gates inside `SubcommandRunner::dispatch` are
unnecessary — the detached `finalize --exec` child
runs at default verbosity, so debug-level output
silently no-ops in that path too. The gates collapse;
`is_detached_exec` survives the substep purely for
`sb_ide`'s remaining banner / `surface_previous_failures`
gate, which the next substeps will dismantle.

- `Cargo.toml`: `0.52.0-0` → `0.52.0-1`.
- `src/main.rs`: `bm_track`'s two `log::info!` calls
  → `log::debug!`; module doc-comment reworded
  (drop the "future refinement" paragraph, note the
  new debug-level emission + why the gate isn't
  needed at the call site).
- `src/subcommand.rs`: `dispatch` drops the two `if
  !is_detached { crate::bm_track(…) }` gates around
  enter/exit; the local `let is_detached = …`
  binding disappears (its remaining consumer,
  `sb_ide`, reads the trait peek inline);
  doc-comment updated accordingly.
- `notes/chores/chores-11.md`: backfilled
  `Commits: [[2]]` on the 0.52.0-0 opener.
- `notes/todo.md`: 0.52.0-0 → `(done)`; 0.52.0-1
  marked `(current)`.

## feat: -V toggles version banner (0.52.0-2)

Commits: [[4]]

Second substep. `sb_ide` no longer emits the
`vc-x1 X.Y.Z` banner on every command run, and `-V`
becomes a banner-toggle rather than print-and-exit:

- `vc-x1 chid -s code,bot` — runs silently (no banner).
- `vc-x1 chid -s code,bot -V` — prints
  `vc-x1 0.52.0-2` as the first line, then runs
  `chid` normally.
- `vc-x1 -V` — prints `vc-x1 0.52.0-2` and exits
  success (no subcommand to run).
- `vc-x1` (no flags, no subcommand) — prints help and
  exits non-zero (mirroring clap's old
  required-subcommand error path).

Replacing clap's auto-version means scripts can capture
the version *and* the command's output in one
invocation. The banner is the uniform `vc-x1 X.Y.Z`
regardless of subcommand — the version is the binary's
either way, and `propagate_version`'s
`vc-x1-<sub> X.Y.Z` form added noise without
information.

With the banner gone from the on-every-run path,
`sb_ide`'s `suppress_banner` parameter loses its only
consumer; the `SubcommandRunner::suppress_banner`
trait method and the `suppress_banner` field on the
four listing `Params` types (`ChidParams`,
`DescParams`, `ListParams`, `ShowParams`) cascade out
— they have no remaining callers once `sb_ide` stops
asking.

`-L` / `--no-label` still works — it suppresses the
per-repo header label in `chid` / `desc` / `list` /
`show` output, a concern that lives in the op layer
(`CommonParams::header`) and was always separate from
banner suppression. The trait peek was a duplicate
spelling of `-L` that survived only as long as
`sb_ide` asked for it.

`sb_ide` retains one remaining responsibility: gating
`finalize::surface_previous_failures` on
`!is_detached_exec`. The next substep relocates that
trigger into finalize itself, after which `sb_ide`
and `SubcommandRunner::is_detached_exec` both
disappear.

- `Cargo.toml`: `0.52.0-1` → `0.52.0-2`.
- `src/main.rs`: `Cli` drops `version,
  propagate_version = true` from `#[command(...)]`
  and adds a global `pub version: bool` arg (with
  `short = 'V'` / `long = "version"`);
  `command: Commands` → `command: Option<Commands>`;
  `fn main` emits the banner ahead of dispatch when
  `cli.version` is set, and handles the no-subcommand
  paths (print + exit on `-V`, print help + exit on
  no flags). `sb_ide` body drops the banner emission
  and its `if !suppress_banner` wrapper; signature
  drops the `suppress_banner: bool` parameter; doc
  rewritten.
- `src/subcommand.rs`: `dispatch` call updated to
  `crate::sb_ide(Self::is_detached_exec(&params))`;
  trait method `SubcommandRunner::suppress_banner`
  removed (no remaining consumer).
- `src/chid.rs` / `src/desc.rs` / `src/list.rs` /
  `src/show.rs`: `suppress_banner: bool` field on
  each `Params` struct removed; `TryFrom` impls drop
  the `suppress_banner: a.common.no_label,`
  assignment; trait `suppress_banner` overrides
  removed; doc-comments updated.
- `src/clone.rs` / `src/finalize.rs` / `src/init/tests.rs`:
  test helpers updated for the `Option<Commands>`
  shape (`match cli.command { Commands::X(a) => … }`
  → `Some(Commands::X(a)) => …`).
- `notes/chores/chores-11.md`: backfilled
  `Commits: [[3]]` on the 0.52.0-1 section.
- `notes/todo.md`: 0.52.0-1 → `(done)`; 0.52.0-2
  marked `(current)`.

## refactor: remove is_detached_exec from trait (0.52.0-3)

Commits: [[5]]

Third substep. The `SubcommandRunner::is_detached_exec`
trait method goes away, taking `sb_ide` with it. The
detached-exec gate is finalize-specific machinery —
`finalize::surface_previous_failures` is the only
behavior it ever protected, and `FinalizeArgs.exec` is
the only field that ever set it. Lifting the gate into
`main` (one `matches!` against `Commands::Finalize`
with `args.exec`) cuts the trait peek + its sole
override + the free function that consumed it.

After this substep:

- The `SubcommandRunner` trait surface is the
  load-bearing minimum: `to_params`, `run`, and the
  default `dispatch`. No more peek methods.
- `main` owns the surface-previous-failures call,
  with the exec-child skip inline at the call site.
  Single home for the gate; no indirection through
  `sb_ide` or the trait.
- `dispatch` shrinks to "build params, bracket with
  `bm_track`, run, map exit code." No session-chrome
  responsibility at all.

The cycle's stated goal — eliminate `sb_ide` and both
trait peeks — is met after this substep. Close-out
lands separately as `0.52.0` (todo→done, no code).

- `Cargo.toml`: `0.52.0-2` → `0.52.0-3`.
- `src/subcommand.rs`: `SubcommandRunner::is_detached_exec`
  removed; `dispatch` drops its `crate::sb_ide(…)`
  call; doc rewritten.
- `src/finalize.rs`: `is_detached_exec` override on
  `FinalizeArgs` removed.
- `src/main.rs`: `pub fn sb_ide` deleted (no
  callers); `main` adds an inline
  `matches!(&cmd, Commands::Finalize(args) if args.exec)`
  peek and calls `finalize::surface_previous_failures`
  before loading `Context`.
- `notes/chores/chores-11.md`: backfilled
  `Commits: [[4]]` on the 0.52.0-2 section; new
  0.52.0-3 section.
- `notes/todo.md`: 0.52.0-2 → `(done)`; 0.52.0-3
  marked `(current)`.

## chore: close sb_ide elimination (0.52.0)

Cycle close-out. The `0.50.0` Subcommand trait sweep
left three pieces of per-run "session chrome" hanging
off `SubcommandRunner` — `sb_ide` (free function) and
the trait peek methods `suppress_banner` /
`is_detached_exec`. All three are gone after this
cycle:

- The on-every-run `vc-x1 X.Y.Z` banner is replaced by
  an opt-in `-V` flag that prints the banner as the
  first line and continues (rather than clap's
  exit-on-print default). Scripts can capture version
  *and* output in one invocation.
- `bm_track` enter/exit emit at `log::debug!` —
  default runs stay quiet, the signal is available
  under `-v`, and the detached `finalize --exec`
  child stays silent without needing a per-call gate.
- `finalize::surface_previous_failures` moved to a
  single inline call in `main` with an exec-child
  skip; `sb_ide` had no body left to keep and is
  deleted; the `is_detached_exec` trait peek (its
  only consumer) goes with it.
- `SubcommandRunner::suppress_banner` and the
  `suppress_banner: bool` field on the four listing
  `Params` types cascaded out during 0.52.0-2 when
  the banner emission left.

`SubcommandRunner`'s trait surface is now the
load-bearing minimum: `to_params`, `run`, and the
default `dispatch` (which builds params, brackets
`bm_track` enter/exit, runs, maps exit code). No
peek methods, no chrome responsibility.

### As-built ladder

- 0.52.0-0 plan + version bump + chores-11 opener
  section + todo ladder
- 0.52.0-1 `bm_track` → `debug!`; drop bm_track
  gates in `dispatch`
- 0.52.0-2 `-V` toggles version banner (replaces
  clap's auto-version); banner-on-every-run gone;
  `suppress_banner` trait method + `Params` fields
  cascade out
- 0.52.0-3 remove `is_detached_exec` from trait;
  gate moves to `main` inline; `sb_ide` deleted
- 0.52.0 close-out

### Outcome

Three subtractive substeps after the opener; no
evaluation-gate detours. The cycle's stated goal — a
trait whose surface only carries what every
subcommand actually needs — landed cleanly. The
`-V`-toggles-banner shape (instead of the original
"drop banner entirely" sketch) emerged mid-cycle when
the user pointed out that opting into the version on
demand is more useful than removing the signal
altogether.

While we were at it, the broader
`surface_previous_failures` design gaps (stale-forever
markers, concurrent surfacing double-print, mid-write
torn reads, no notify-at-failure path) got captured
as a new `## Bugs` section in `notes/todo.md`. The
exec-child gate covers one race; the rest is queued.

- `Cargo.toml`: `0.52.0-3` → `0.52.0` (suffix
  dropped — cycle close marker).
- `notes/chores/chores-11.md`: backfilled
  `Commits: [[5]]` on the 0.52.0-3 section; new
  close-out section with the as-built ladder +
  outcome notes.
- `notes/todo.md`: deleted the In Progress ladder
  block; added `sb_ide elimination — banner off by
  default (-V toggles), bm_track → debug!, sb_ide +
  SubcommandRunner::{is_detached_exec,
  suppress_banner} removed (0.52.0)` to `## Done`
  with `[[2]]` ref.

# References

[1]: https://github.com/winksaville/vc-x1/commit/1e7c979e5458 "1e7c979e5458189e4a5f380b18acd81d75ffe68b"
[2]: https://github.com/winksaville/vc-x1/commit/48b79876ef3f "48b79876ef3f8f421eee81a63bb9937611558734"
[3]: https://github.com/winksaville/vc-x1/commit/9a3e1605d453 "9a3e1605d453eaff6f7a45e50174fdfaee9f7b48"
[4]: https://github.com/winksaville/vc-x1/commit/61454c56229a "61454c56229ac37afd89ab8bbcb7d2947eb9465c"
[5]: https://github.com/winksaville/vc-x1/commit/90584bfbd171 "90584bfbd1710d9c4a5db6b93902b57c33875f6b"
