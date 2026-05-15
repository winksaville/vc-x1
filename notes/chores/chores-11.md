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

# References

[1]: https://github.com/winksaville/vc-x1/commit/1e7c979e5458 "1e7c979e5458189e4a5f380b18acd81d75ffe68b"
