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

**docs: apply max review #1 (0.62.0)**

Apply accepted items from
[max-review-1.md](design-cli/max-review-1.md) to the
por/dual parity audit doc and the copying stub.
Concern #1 + #2 Topology were applied in an
abandoned-during-rebase snapshot in 0.61.0
(`604fb9e8`); the rest of the review (concerns #2
Privacy, #3–#6, nits N1–N4, process observation)
remains open.

- 0.62.0-0 Preparation (done)
- 0.62.0-1 seed max-review-1 working list (done)
- 0.62.0-2 Concern #1 runtime `--por` rewrite (done)
- 0.62.0-3 Concern #2 Topology → `--mode` (done)
- 0.62.0-4 Concern #2 Privacy → `--visibility`
  + clone-row fix (done)
- 0.62.0-5 Concern #3 `--init-from*` surface
  halving (done)
- 0.62.0-6 Concern #4 `--repo none` × dual
  interaction (done)
- 0.62.0-7 Concern #5 list-valued CLI-vs-config
  "wins" callout (done)
- 0.62.0-8 gap prereq note + fold todo captures
  (done)
- 0.62.0-9 nits N1–N4 (audit.md footnotes/guide)
- 0.62.0-10 process observation → chores narrative
- 0.62.0 close-out

## Ideas

 Items not yet solid enough for `## Todo` (or surfaced
 during close-out / end-of-day before they're fully
 formed). Triaged at the next Preparation: promote to
 `## Todo` / `notes/todo-backlog.md`, fold into a
 picked-up cycle, or drop.

1. **Codify ochid invariant + bot-repo rules + squash
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

2. **Version-number protocol is fragile — versions are
   baked into titles/bodies/todo/done/chores before the
   change lands.** The cycle protocol embeds an `X.Y.Z-N`
   version in commit titles and bodies, `## Todo` /
   `## Done` entries, and chores headers — all written
   while the work is in progress, i.e. before it lands.
   But version numbers are subject to change: in a public,
   merge-based flow (e.g. Linux), the version a change
   ships under is only fixed when it merges into `main`,
   so the landing version can't be anticipated while the
   work is underway. Pervasive version-in-text is
   therefore fragile for any non-linear / multi-contributor
   workflow.
   - No fix yet — capture the problem; triage at a later
     Preparation.
   - Open question: what identifies a cycle's commits if
     not a pre-assigned version?
     - Needs to be unique within some agreed upon domain.
       A contributors email address would do it, but also
       a UUID (short-version) for a contribution. I could
       imagine a UUID generated from the initial email/issue
       that and then "version number" schema appended to that.

## Priorities

- P1 is highest priority, same priority are grouped equally
- A line with `...` is a prefix-match against a `## Todo` title
  - resolves locally (in this file); should be unique
- Entries without a prefix-match are free-form
  - chores tasks, ad-hoc reminders.

### P1

- `**pre-commit: single rule...**`
- `**vc-x1 push: body starting with `-` breaks `commit-app`.**`
- `**vc-x1 push: support...**`
- `**vc-x1 push --squash...**`

### P2

- `**single-field...**`

### P3

- `**por -> dual...**`

## Todo

 Prioritized entries (referenced by `## Priorities`) lower priority
 todo's are in [todo-backlog.md](todo-backlog.md). In all cases
 we use the [Prose Form in CLAUDE.md](/CLAUDE.md#prose-form). When
 more detail is waranted those reside in `notes/chores/chores-NN.md`.
 Also, we use the  design subsections (link via `[N]` ref). Run
 `vc-x1 fix-todo --no-dry-run notes/todo.md` to renumber.

1. **vc-x1 push: body starting with `-` breaks `commit-app`.**
   Push builds `jj commit -m "<title>" -m "<body>"` in the
   space form, so a body whose first line is a bullet
   (`- file: …`, the normal app-repo body shape) makes jj's
   parser reject the value as a stray flag. Hit on 0.62.0-5.
   The rollback was clean, but the push can't land a
   standard bullet-first body.
   - Fix: invoke `jj commit -m=<body>` (equals form) or set
     `allow_hyphen_values` on the message args.
   - The same defect bites a raw `jj commit -m "<body>"`, so
     the equals form is the general remedy.
   - Workaround until fixed: prepend a non-dash intro line
     to the body (a Prose-Form intro sentence).
2. **pre-commit: single rule (no docs skip) + doc validators.**
   The pre-commit (cargo cycle: fmt/clippy/test/install) only
   checks code, so it's "skip-able for purely-docs commits" —
   but that exception is exactly where checks slip (skipped on
   0.62.0-7/-8 until caught). And `vc-x1 push`'s `preflight`
   stage re-runs the same cycle, which invites treating push as
   the gate rather than a redundant safety-net.
   - Adopt one rule, no exception: the pre-commit runs before
     Work review on every commit; push's `preflight` is a
     safety-net, not the primary gate. (docs: CLAUDE.md Cycle
     Protocol summary + cycle-protocol.md per-commit-flow.)
   - Enrich the pre-commit so it's meaningful on docs commits:
     add `vc-x1 validate-todo` now (and `validate-repo` when it
     exists) to both the documented flow and push's `preflight`
     stage (`push.rs`), with a test. (code)
   - This dissolves the docs exception: with doc validators in
     the pre-commit there's always something to validate, so
     the carve-out stops making sense.
   - Target: its own 0.62.1 cycle (chosen over a 0.61.1 insert
     to avoid rewriting published 0.62.0-x history).
3. **vc-x1 push: support new cycle protocol shape (N:1 code↔bot).**
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
4. **vc-x1 push --squash: symmetric squash on both repos.**
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
5. **single-field `options_flags` leaves → `value` field.**
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
6. **`por → dual` conversion.** Attach a `.claude`
   companion repo + `.vc-config.toml` to an existing por
   workspace; emit cross-links going forward. Manual
   setup on an external por workspace (2026-05-14)
   proved arduous; this should be a routine subcommand.
   Design stub in [[1]] § 2.
7. **`validate-desc` / `fix-desc` por equalization.**
   Replace the `other_repo_from_config` prelude in both
   subcommands (`validate_desc.rs:133`, `fix_desc.rs:152`)
   with a scope-aware resolution that no-ops `Side::Bot`
   when absent. Body unchanged. The 0.61.0 audit/design
   work [[13]] identified this as the cheapest concrete
   equalization and the right prototype for the
   topology-from-config rule (subcommand reads topology
   from `default_scope`, not from a flag). Validates the
   broader design before larger pieces (`push`,
   `--init-from*`) commit to it. The remaining 13
   implementation gaps live in [[13]]'s `## Gap list` for
   future Preparation passes to pick up.

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
- consolidate notes conventions — three notes-file sections (`Todo format`, `Reference numbering`, `Retiring Done entries`) move from notes/README.md into new CLAUDE.md `## Notes file conventions` umbrella alongside existing `## Chores conventions`; `[[N]]` citation duplicate dropped; cargo cycle (`fmt` / `clippy` / `test` / `install`) surfaced at CLAUDE.md `## Cycle Protocol` and notes/README.md (had been buried in cycle-protocol.md since 0.59.0); README.md `## Contributing` rewritten against current anchor homes (0.60.0) [[12]]
- por/dual parity design — eight-commit audit + design cycle producing `notes/design-cli/por-dual-parity-audit.md` as the canonical CLI-design doc (audit + commonality + feature axes + 5-layer resolution chain + subcommand × parameter matrix + per-axis Decisions blocks); new sibling `notes/design-cli/copying.md` stub for the broader file-copy mechanism that subsumes `--config` / `--gitignore` / `--use-template`; `notes/design-cli/` subdir created and three design notes regrouped under it; 14 implementation gaps seeded for 0.62.0+ cycles; one Todo promoted (`validate-desc` / `fix-desc` equalization, cheapest prototype for the topology-from-config rule) (0.61.0) [[13]]

# References

[0]: /CLAUDE.md#prose-form
[1]: /notes/design-cli/por-dual-parity.md
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
[12]: /notes/chores/chores-12.md#docs-consolidate-notes-conventions-0600
[13]: /notes/chores/chores-12.md#docs-pordual-parity-design-0610
