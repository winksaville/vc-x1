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

## Priorities

- P1 is highest priority, same priority are grouped equally
- A line with `...` is a prefix-match against a `## Todo` title
  - resolves locally (in this file); should be unique
- Entries without a prefix-match are free-form
  - chores tasks, ad-hoc reminders.

### P1

- chores-11 is "full" > 1,000 lines, start chores-12.md

### P2

- `**single-field...**`

### P3

- `**por -> dual...**`
- `**por/dual parity...**`

## Todo

 Prioritized entries (referenced by `## Priorities`).
 Full backlog in [todo-backlog.md](todo-backlog.md). Keep
 entries brief — 1-3 lines; detail in
 `notes/chores/chores-NN.md` design subsections (link via
 `[N]` ref). Run `vc-x1 fix-todo --no-dry-run notes/todo.md`
 to renumber.

1. **single-field `options_flags` leaves → `value` field.**
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
2. **`por → dual` conversion.** Attach a `.claude`
   companion repo + `.vc-config.toml` to an existing por
   workspace; emit cross-links going forward. Manual
   setup on an external por workspace (2026-05-14)
   proved arduous; this should be a routine subcommand.
   Design stub in [[1]] § 2.
3. **por/dual parity + `dual → por` conversion.** Make
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

# References

[1]: /notes/por-dual-parity.md
[2]: /notes/chores/chores-11.md#chore-move-chores-under-noteschores-0510
[3]: /notes/chores/chores-11.md#chore-close-sb_ide-elimination-0520
[4]: /notes/chores/chores-11.md#chore-todo-renumber--fix-todopy-0530
[5]: /notes/chores/chores-11.md#refactor-scope-cli-cleanup-0540
[6]: /notes/chores/chores-11.md#feat-validate-todo--fix-todo-0550
[7]: /notes/chores/chores-11.md#docs-refine-cycle-protocol-0560
[8]: /notes/chores/chores-11.md#docs-add---merge-todo-entry-0570
[9]: /notes/chores/chores-12.md#refactor-notestodo-restructure-0580
