# Notes

This directory contains various notes and documentation related to the project. Each file is
organized by topic for easy reference.

The chores-NN.md files in [chores/](chores) and [done.md](done.md) are frozen history: the records
of cycles that closed before the cycle-record rule ([Cycle-record](../AGENTS.md#cycle-record)).
Nothing is appended to them. They stay until the agent-repo transcripts are comfortable to read a
cycle from, then both retire in one cycle after a permalink sweep of the links into them (the `##
Waiting` entry in [../TODO.md](../TODO.md)). A cycle's live record is [../TODO.md](../TODO.md)'s `##
In Progress` block, and a closed cycle's is its `## Closed` block in the landmark commit's tree.

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

The agent-files' line count over time is in [agent-files-size.md](agent-files-size.md), one row
per landing, smaller being the quasi-goal.

How a change to a line in any work-repo file is connected to its discussion in the agent-repo,
the partner commit, its time-window, and the transcript write, is in
[transcript-write.md](transcript-write.md). It carries the objective and the terms, the by-hand
procedure, the two decisions the probes settled, the `vc-x1 lookup` command's requirements in
their one home, and the probes themselves, three cycles of them.

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

Per-cycle workflow is AGENTS.md's [Cycle protocol](../AGENTS.md#cycle-protocol):

- [Opening](../AGENTS.md#opening): the bookmark, the six provisional items, the ladder.
- [The per-rung flow](../AGENTS.md#the-per-rung-flow): validation, the work and description
  review stops, `vc-x1 push`.
- [Commit description](../AGENTS.md#commit-description): Conventional Commits title, body shape
  per work vs agent repo.
- [Pushing](../AGENTS.md#pushing): push policy, the at-rest contract.
- [Close-out](../AGENTS.md#close-out): acceptance check, the chores move, shape, landing.
