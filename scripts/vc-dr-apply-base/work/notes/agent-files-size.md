# Agent-files size

The line count of the agent-files, one row per landing that changed them, so the set's size is
tracked over time. A cycle that leaves the agent-files untouched adds no row, since a row saying
the count did not move records nothing. Smaller is the quasi-goal: a rule stated once is shorter
than a rule stated three times, and a shrinking count is evidence the set is converging, while a
growing one is a prompt to ask what arrived as a paragraph that should have been a line. The count
is not a rule, and a rule is never cut to move it.

A finding from the proposal cycle (2026-08-27), for whoever next tries a cut: a draft that cut
AGENTS.md from 370 to 320 lines with every rule kept found that the remaining bulk is the rules
themselves, so a further cut has to cut rules, and that is a convention decision, not an edit.

The count is `wc -l` over `AGENTS.md`, `custom.md`, and `agent-data/*.md`, the total leaving out
`rationale.md`, taken at the close-out of a cycle that changed an agent-file and recorded here as
the closing rung's last edit, with the cycle title as the row's label. `rationale.md` is out of the
total because it is the rules' why and grows with every rule that gains one, so a rule gaining a
why does not read as the set growing. Rows before v0.2.5 counted it.

`vc-x1 agent-files size` takes the count and slides the per-file table, a new leftmost column in
and the oldest out. It is dry-run by default and `--no-dry-run` writes the file, `--label=TEXT`
names a column the set's version file cannot, and the `## Counts` row above stays a hand's work,
since its note is prose.

## Counts

| Landed | Cycle | Files | Lines | Note |
|---|---|---|---|---|

Per file for the three most recent rows, newest on the left, the window sliding at each close-out
so the earlier history is in the commits. A column is labeled by the set version it carries, the
landings before the set was versioned relative to the first version (`- v0.1.0` one before it,
`-- v0.1.0` two before), and a landed local change updates the newest column in place and has
its own row above. A cell in angle brackets, `<543>`, is a file the total leaves out, shown so a
reader sees what it costs, and an empty cell is a file that was not in the set that landing.
