# Notes

This directory contains various notes and documentation related to the project.
Each file is organized by topic for easy reference.

By default there are todo.md and the chores-NN.md files in
[chores/](chores). Chores are general notes about tasks and
todo.md contains short term tasks and their status. The
chores-NN files are numbered in sequence; the highest-numbered
file is the active one, older ones are closed.

App architecture — the CLI-args / `Context`+`Params` split, a
generic module map, the subcommand model, and what the
Context+Params port and the options_flags extraction *are* (the
*live status* is in the chores files) — lives at the repo root in
[`../ARCHITECTURE.md`](../ARCHITECTURE.md).

In the future we I expect we may want to create a "notes"
database to better manage the information, TBD.

Examples chore file:
```
# Chores-01.md
 
General maintenance tasks and considerations for the project see other files for
more specific topics. A chore in a chores file provides quick information on the
how and why of a particular chore.

## Create a binary that lists jj info 

This binary should list the changeID, commitID, and description title
and using `jj-lib`
```

## Workflow and conventions

Bot-facing workflow and conventions live in
[`../AGENTS.md`](../AGENTS.md):

- [Notes file conventions](../AGENTS.md#notes-file-conventions)
  — Todo format, Reference numbering, Notes references
  (`[[N]]` citation style), Markdown anchor links, Retiring
  Done entries.
- [Chores conventions](../AGENTS.md#chores-conventions) —
  section headers / Done entries exact-title rule, content
  rules, `Commits:` line format.
- [Code Conventions](../AGENTS.md#code-conventions) — doc
  comments, `// OK: …` on `unwrap*` calls, ask-on-ambiguity,
  stuck detection.

Per-cycle workflow lives in
[`cycle-protocol.md`](cycle-protocol.md):

- [Cycles](cycle-protocol.md#cycles) — three-phase shape
  (Preparation → Work → Close-out), `X.Y.Z-N` numbering,
  sub-cycles.
- [Per-commit flow](cycle-protocol.md#per-commit-flow) —
  cargo cycle (`fmt` / `clippy` / `test` / `install`),
  work + commit description review gates.
- [Commit description](cycle-protocol.md#commit-description)
  — Conventional Commits + `(version)` suffix; body shape
  per app vs `.claude` repo.
- [Pushing](cycle-protocol.md#pushing) — push policy,
  close-out shape, `.claude` cadence.
