# Notes

This directory contains various notes and documentation related to the project. Each file is
organized by topic for easy reference.

By default there are the chores-NN.md files in [chores/](chores). Chores are general notes about
tasks; short term tasks and their status live at the repo root in [../TODO.md](../TODO.md). The
chores-NN files are numbered in sequence; the highest-numbered file is the active one, older ones
are closed.

Tool architecture lives at the repo root in [`../ARCHITECTURE.md`](../ARCHITECTURE.md): the
CLI-args / `Context`+`Params` split, a generic module map, the subcommand model, and what the
Context+Params port and the options_flags extraction *are* (the *live status* is in the chores
files).

Multi-cycle programs too big for a TODO.md entry get their own dated plan file, e.g.
[refactor-20260716.md](refactor-20260716.md) (typed jj facade -> jj-lib in-process): one `##`
section per stage, so chores and todo entries can reference a stage by anchor. The plan file owns
the forward design; chores still records each shipped cycle.

A rule that governs what the tool does, rather than a record of what was done, gets its own topic
file so it can be found without knowing which cycle produced it, e.g.
[jj-version-policy.md](jj-version-policy.md). The investigation behind such a rule stays where it
happened (a plan file or a chores section) and is linked from the policy, so neither restates the
other.

In the future I expect we may want to create a "notes" database to better manage the information,
TBD.

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

Bot-facing workflow and conventions live in [`../AGENTS.md`](../AGENTS.md) (hard rules + file
map) and its `../agent-data/` satellites:

- [Notes file conventions](../agent-data/notes.md): Todo format, Reference numbering, Notes
  references (`[[N]]` citation style), Markdown anchor links, Retiring Done entries, Chores
  conventions (section headers / Done entries exact-title rule, content rules, the as-built
  ladder and its commit reference format, the chores Table of Contents).
- [Prose and durable text](../agent-data/prose.md): prose form, typeable punctuation,
  conventional-commit shape.
- [Code conventions](../agent-data/code.md): doc comments, `// OK: ...` on `unwrap*` calls.

Per-cycle workflow lives in [`cycle-protocol.md`](../agent-data/cycle-protocol.md):

- [Cycles](../agent-data/cycle-protocol.md#cycles): three-phase shape (Preparation -> Work -> Close-out),
  `X.Y.Z-N` numbering, sub-cycles.
- [Per-commit flow](../agent-data/cycle-protocol.md#per-commit-flow): cargo cycle
  (`fmt` / `clippy` / `test` / `install`), work + commit description review gates.
- [Commit description](../agent-data/cycle-protocol.md#commit-description): Conventional Commits title; body
  shape per work vs bot repo.
- [Pushing](../agent-data/cycle-protocol.md#pushing): push policy, close-out shape, `.claude` cadence.
