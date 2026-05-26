# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

_No cycle currently in progress._

## Ideas

 Items not yet solid enough for `## Todo` (or surfaced
 during close-out / end-of-day before they're fully
 formed). Triaged at the next Preparation: promote to
 `## Todo` / `notes/todo-backlog.md`, fold into a
 picked-up cycle, or drop.

1. **Relocate CLAUDE.md chores conventions into
   `notes/cycle-protocol.md`.** Makes cycle-protocol.md
   fully self-contained on the cycle/chores workflow for
   README.md and other consumers.
   - Identify cross-refs in CLAUDE.md and elsewhere
     pointing at the three subsections being moved.
   - Move `### Headings and entries that record a
     commit` from CLAUDE.md `## Pre-commit Requirements`
     into cycle-protocol.md.
   - Move `### Chores section content` from CLAUDE.md
     into cycle-protocol.md.
   - Move `### Chores commit references` from CLAUDE.md
     into cycle-protocol.md.
   - Restructure CLAUDE.md `## Pre-commit Requirements`
     after the moves — it then holds only general
     writing conventions (`### Notes references`,
     `### Markdown anchor links`); consider renaming.
   - Update all cross-refs to point at the new
     locations.
2. **Codify ochid invariant + bot-repo rules + squash
   gating + cross-repo migration in
   `notes/cycle-protocol.md`.** Was planned for
   `0.59.0-2` but deferred when 0.59.0 closed out via
   squash + Option F (manual bot-side rewrite). The
   rules were exercised manually for the close-out;
   this Idea formally codifies them.
   - Codify the **ochid invariant** in `## ochid
     trailers`: every public ochid must resolve in the
     public graph.
   - Codify bot-repo rules: never squashed; descriptions
     editable (`jj describe` preserves chid).
   - Codify squash gating: until `vc-x1 push --squash`
     exists, manual symmetric squash (Option F: app
     squash + bot-side trailer rewrite + force-push) is
     the standard recipe; merge non-ff is the default
     shape for multi-commit cycles.
   - Sketch cross-repo migration: ochids change at
     every merge until the change reaches the canonical
     repo's `main`.
   - Apply Ideas-aware Preparation/Close-out updates:
     Preparation triages `## Ideas` (promote / fold /
     drop) before declaring the plan; Close-out
     captures unresolved follow-ups into `## Ideas`.

## Priorities

- P1 is highest priority, same priority are grouped equally
- A line with `...` is a prefix-match against a `## Todo` title
  - resolves locally (in this file); should be unique
- Entries without a prefix-match are free-form
  - chores tasks, ad-hoc reminders.

### P1

- `**vc-x1 push: support...**`
- `**vc-x1 push --squash...**`

### P2

- `**single-field...**`

### P3

- `**por -> dual...**`
- `**por/dual parity...**`

## Todo

 Prioritized entries (referenced by `## Priorities`) lower priority
 todo's are in [todo-backlog.md](todo-backlog.md). In all cases
 we use the [Prose Form in CLAUDE.md](/CLAUDE.md#prose-form). When
 more detail is waranted those reside in `notes/chores/chores-NN.md`.
 Also, we use the  design subsections (link via `[N]` ref). Run
 `vc-x1 fix-todo --no-dry-run notes/todo.md` to renumber.

1. **vc-x1 push: support new cycle protocol shape (N:1 code↔bot).**
   Today push assumes 1:1 symmetric WC commits with shared
   title/body. The new cycle protocol has a different shape on
   each side:
   - code side: fully committed before push (N commits via
     merge), nothing to commit at push time
   - `.claude` side: one commit with its own message (distinct
     from any code commit) and a multi-line `ochid:` listing
     all N code commits

   Teach push to:
   - detect this shape
   - skip `commit-app` when code WC is empty
   - compose a `.claude`-specific message
   - emit multi-line `ochid:` per the design in [[10]]

   Today's workaround: pre-commit `.claude` manually, then
   `vc-x1 push <bm> --from bookmark-both --yes`.
2. **vc-x1 push --squash: symmetric squash on both repos.**
   Automate Option F (manually exercised in 0.59.0
   close-out): app-side squash + bot-side description
   rewrite + force-push, atomically. Without this,
   squash is a manual recipe that future cycles must
   follow each time.
   - App side: squash cycle commits into one new commit;
     capture the squashed chid.
   - Bot side: rewrite the prior push commit's
     description — replace its per-commit `ochid:`
     trailers with one pointing at the squashed chid;
     add a rewrite-note acknowledging the change
     (preserves historical truth for future readers).
   - Force-push bot `main` (rewrites the published
     commit; chid preserved via `jj describe`).
   - Push app `main`. The new bot commit paired with
     this push receives `ochid: /<squashed-chid>` as
     normal — produces a 2:1 bot→code pattern that's
     already in the protocol's design space.
   - Gates `Squash to one commit` as a routine
     close-out shape vs. the current manual recipe.
3. **single-field `options_flags` leaves → `value` field.**
   `0.47.0` introduced the convention (single-field leaf names
   its field `value`, declares the flag via `#[arg(long = "…")]`,
   so consumers read `args.<leaf>.value` not `args.<leaf>.<leaf>`)
   on the new `squash` leaf. Sweep the pre-existing single-field
   leaves to match: `repo`, `dry_run`, `private`, `account`,
   `config`, `use_template` + their consumers
   (`init.rs`, tests).

   Note: can a single field be defined as an type or enum instead
   of a struct and maybe eliminate the `args.<leaf>.<leaf>` name
   issue.
4. **`por → dual` conversion.** Attach a `.claude`
   companion repo + `.vc-config.toml` to an existing por
   workspace; emit cross-links going forward. Manual
   setup on an external por workspace (2026-05-14)
   proved arduous; this should be a routine subcommand.
   Design stub in [[1]] § 2.
5. **por/dual parity + `dual → por` conversion.** Make
   `por` and `dual` first-class equals (dual is primary
   today, por bolted on); add `dual → por` conversion
   (detach the `.claude` companion). Builds on the
   `--scope` rollout below. Pre-design; goal + open
   questions in the stub. [[1]]

## Bugs

_See [bugs.md](bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

_Migrated to [done.md](done.md) on 2026-05-15 (0.44.0–0.50.0 batch)._

- chores subdir reshape — `notes/chores-*.md` → `notes/chores/`; 0.44.0–0.50.0 Done batch migrated to done.md (0.51.0) [[2]]
- `sb_ide` elimination — banner off by default (`-V` toggles), `bm_track` → `debug!`, `sb_ide` + `SubcommandRunner::{is_detached_exec, suppress_banner}` removed (0.52.0) [[3]]
- todo renumber + `notes/fix-todo.py` interim script; cycle re-scoped at close-out, scope CLI cleanup deferred to 0.54.0 (0.53.0) [[4]]
- scope CLI cleanup — `--scope` roles-only, `--por` boolean replaces `ScopeKind`, `Scope` relocated to `options_flags/`, sync gains `-R` (0.54.0) [[5]]
- validate-todo / fix-todo subcommands — check + renumber `## Todo` / `## Bugs` entry numbering, replacing `notes/fix-todo.py` (0.55.0) [[6]]
- refine cycle protocol — one protocol (Preparation/Work-N/Close-out), `.`-separator nested numbering with trailing-`0`=Preparation, push & squash discretionary, `.claude` once per push, two-gate review (work then message, both before commit), CLAUDE.md cycle/commit/push docs consolidated into one linear `## Cycle Protocol` (~39% smaller) (0.56.0) [[7]]
- add `--merge` todo entry — Todo #1 records future `vc-x1 push --merge` flag (close-out shape, sibling to planned `--squash`); dogfoods the Preparation/Work-N/Close-out protocol on a deliberately small docs cycle (0.57.0) [[8]]
- notes/todo restructure — split `## Bugs` → `bugs.md` and the long-tail `## Todo` → `todo-backlog.md`; `## Priorities` with tier sub-headings (`### P1`/`### P2`/`### P3`); CLAUDE.md `## File reads` rule + protocol codification (chores title-only during cycle, In Progress moves into chores at close-out, problem+plan shape) (0.58.0) [[9]]
- extract cycle protocol — `notes/cycle-protocol.md` becomes the canonical self-contained home for the cycle workflow (504 lines, extensively tightened from the CLAUDE.md extract); CLAUDE.md keeps a 10-line pointer; `notes/substep-protocol.md` folded in as `## Sub-cycle ladders`; `## Ideas` section added to `notes/todo.md`; first squash close-out via manual Option F (app squash + bot-side `af60f979` trailer rewrite + force-push) (0.59.0) [[11]]

# References

[0]: /CLAUDE.md#prose-form
[1]: /notes/por-dual-parity.md
[2]: /notes/chores/chores-11.md#chore-move-chores-under-noteschores-0510
[3]: /notes/chores/chores-11.md#chore-close-sb_ide-elimination-0520
[4]: /notes/chores/chores-11.md#chore-todo-renumber--fix-todopy-0530
[5]: /notes/chores/chores-11.md#refactor-scope-cli-cleanup-0540
[6]: /notes/chores/chores-11.md#feat-validate-todo--fix-todo-0550
[7]: /notes/chores/chores-11.md#docs-refine-cycle-protocol-0560
[8]: /notes/chores/chores-11.md#docs-add---merge-todo-entry-0570
[9]: /notes/chores/chores-12.md#refactor-notestodo-restructure-0580
[10]: /notes/forks-multi-user.md
[11]: /notes/chores/chores-12.md#docs-extract-cycle-protocol-0590
