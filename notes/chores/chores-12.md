# Chores-12

Continuation of `chores-11.md` (which is closed at `0.57.0` —
the `--merge` todo entry cycle). This file covers the `0.58.0`
cycle onward. Reference numbering is file-local — see
[`CLAUDE.md`](../../CLAUDE.md#reference-numbering); chores-12
starts at `[1]`.

## refactor: notes/todo restructure (0.58.0)

Commits: [[1]]

`notes/todo.md` is too large for routine reads and its
`## Todo` intro duplicates `notes/README.md`.

### As-built ladder

- 0.58.0-0 Preparation
  - backfill `0.57.0` `Commits:` ref in chores-11
  - bump Cargo.toml to `0.58.0-0`
  - open chores-12 with bare section header
  - add minimal `## File reads` to CLAUDE.md
  - populate `## In Progress`
- 0.58.0-1 add `## Priorities` + trim `## Todo` intro
- 0.58.0-2 move `## Bugs` → `notes/bugs.md`; pointer in todo.md
- 0.58.0-3 extract `notes/todo-backlog.md` (prioritized entries
  stay in todo.md; references re-packed per file)
- 0.58.0-4 expand CLAUDE.md `## File reads` to bugs.md / chores
- 0.58.0-5 codify in CLAUDE.md: chores title-only during cycle,
  In Progress moves into chores at close-out, problem+plan
  shape for In Progress / chores intros / Todo entries
- 0.58.0 close-out: move In Progress block into chores-12;
  `## Done` entry; update `notes/README.md`

### Outcome

Cycle grew from 3 planned commits (intro trim + Priorities +
bugs split) to 6 as the work surfaced two related concerns:
the bot's routine read cost (handled by extracting
`todo-backlog.md` and the CLAUDE.md `## File reads` rule),
and the chores/In-Progress duplication during cycles (handled
by codifying the new "chores title-only during cycle, move at
close-out" mechanic and the problem+plan shape).

`notes/todo.md` is now sized for cheap routine reads — the
head limit=60 covers intro + `## In Progress` + `## Priorities`,
the live surfaces. The split infrastructure (`notes/bugs.md`,
`notes/todo-backlog.md`, re-packed per-file refs) makes
future similar splits cheap.

## docs: extract cycle protocol (0.59.0)

Commits: [[2]]

`notes/cycle-protocol.md` becomes the canonical,
self-contained home for the cycle workflow. CLAUDE.md
keeps a 10-line pointer. Originally scoped as "verbatim
relocate"; opened up into a top-to-bottom iterative
tightening of the extracted protocol.

Late-stage design (ochid invariant + bot-repo rules +
squash gating + cross-repo migration + Ideas-aware
Preparation/Close-out) was planned for `-2` but
deferred at close-out. The rules were exercised
manually via Option F (app squash + bot-side trailer
rewrite + force-push), establishing the recipe that
`vc-x1 push --squash` (Todo #2, P1) will automate.

### As-built ladder

- 0.59.0-0 Preparation — backfill 0.58.0 chores
  `Commits:` ref; bump Cargo.toml; capture 0.58.0
  follow-up Todo (`vc-x1 push` N:1 P1); refresh
  Priorities.
- 0.59.0-0.1 capture design notes for `-1` as `###`
  subsections (see existing subsections below — they're
  the early-stage design that got deferred when 0.59.0
  pivoted to squash close-out).
- 0.59.0-1 Extract + tighten cycle protocol —
  `notes/cycle-protocol.md` (504 lines after iterative
  tightening); `notes/substep-protocol.md` recipes
  folded as `## Sub-cycle ladders`; CLAUDE.md `## Cycle
  Protocol` (~390 lines) → 10-line pointer;
  `init-clone-refactor-conflict.md` cross-ref updated;
  2 entries added to `notes/todo-backlog.md` + 42
  renumbered.
- 0.59.0 Close-out via squash + Option F —
  `## Ideas` section added to `notes/todo.md`
  (chores-convention-relocation entry #1; deferred
  codification entry #2); symmetric-squash captured as
  Todo #2 (P1); close-out bookkeeping (In Progress →
  chores; `## Done` entry); app squash; bot-side
  `jj describe` of `af60f979` (replace two `ochid:`
  trailers with one to squashed chid + rewrite note);
  force-push `.claude/main`; push app `main`.

### Outcome

cycle-protocol.md landed at significantly higher quality
than originally scoped — the "verbatim relocate" plan
opened the door to a top-to-bottom tightening pass that
improved structure throughout (phase-first overview;
`###` subsections; Chores sections + Commit description
extracted as their own `##` sections; Iterative work
pattern captured; chid/ochid defined locally).

First squash close-out via the manual Option F recipe;
first cycle to rewrite a published bot-side commit's
description. The codification follow-up that would make
the recipe binding (and surface the ochid invariant in
cycle-protocol.md) is captured as `## Ideas` entry #2.

### Squash carve-out (for CLAUDE.md `### Pushing`)

Default = merge for any multi-commit cycle. Squash = explicit
opt-in for cycles whose sub-steps are intermediate validation
points rather than distinct landings. Every cycle is ≥3
commits (Preparation + ≥1 Work + Close-out), so the
"merge of one parent" edge case is moot.

### Close-out follow-up capture (for CLAUDE.md `### Close-out`)

At close-out, write any follow-up Todos surfaced during the
cycle to `notes/todo.md > ## Todo` (if prioritized) or
`notes/todo-backlog.md` (if long-tail). List them in the
close-out commit body so the additions are reviewable.

### Close-out surfaces follow-ups (meta-note, for `## Cycle Protocol`)

Close-out often surfaces follow-up items that became visible
only by doing the cycle's work. Capture them at close-out
(per the rule above) so they survive `/exit` between cycles
— sessions don't carry conversational context across `/exit`.

## docs: consolidate notes conventions (0.60.0)

Commits: TBD (backfills at next cycle's Preparation)

Bot is the primary writer of notes files, so the
file-format conventions belong in CLAUDE.md as the single
source of truth. Three notes-file sections (`Todo format`,
`Reference numbering`, `Retiring Done entries`) move from
notes/README.md into a new `## Notes file conventions`
umbrella in CLAUDE.md, alongside the existing
`## Chores conventions` umbrella; the `[[N]]` citation
rule duplicated between CLAUDE.md `## Notes references`
and notes/README.md `## Todo format` collapses into one
place.

### Cargo cycle discoverability

Mid-cycle the bot lost track of where the per-commit
cargo cycle (`fmt` / `clippy` / `test` / `install`)
lived. It had been buried inside `notes/cycle-protocol.md
> ## Per-commit flow > step 4` since the 0.59.0
extraction, with no direct surface in CLAUDE.md or
notes/README.md — and README.md `## Contributing` still
listed seven `CLAUDE.md#…` anchors that had been removed
in 0.59.0 (only `#code-conventions` still resolved).
Cycle scope expanded to fix all three: CLAUDE.md
`## Cycle Protocol` now names the cargo cycle inline;
notes/README.md `## Workflow and conventions` breaks
cycle-protocol.md into explicit pointers (Cycles /
Per-commit flow / Commit description / Pushing);
README.md `## Contributing` is rewritten against current
anchor homes.

### As-built ladder

- 0.60.0-0 Preparation — backfill 0.59.0 chores
  `Commits:` ref; bump Cargo.toml; pick up the "notes
  conventions → CLAUDE.md" Idea into `## In Progress`;
  open chores section.
- 0.60.0-1 Consolidate notes-file conventions in
  CLAUDE.md — three sections moved into the new
  umbrella; `[[N]]` duplicate dropped; chores files'
  reference-numbering pointer redirected to CLAUDE.md;
  `cycle-protocol.md` broken anchor fixed; cargo cycle
  surfaced at CLAUDE.md + notes/README.md; README.md
  `## Contributing` rewritten.
- 0.60.0 Close-out — chores narrative; `## Done` entry +
  `[12]:` ref; `## In Progress` reset.

# References

[1]: https://github.com/winksaville/vc-x1/commit/a199d062ff6e "a199d062ff6e88b5e2d87d57551d1c60e75b073b"
[2]: https://github.com/winksaville/vc-x1/commit/e67e44b8e1c5 "e67e44b8e1c55b8e7c33087b8f2184df87181885"
