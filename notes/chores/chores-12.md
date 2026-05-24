# Chores-12

Continuation of `chores-11.md` (which is closed at `0.57.0` —
the `--merge` todo entry cycle). This file covers the `0.58.0`
cycle onward. Reference numbering is file-local — see
[`README.md`](../README.md#reference-numbering); chores-12
starts at `[1]`.

## refactor: notes/todo restructure (0.58.0)

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

# References
