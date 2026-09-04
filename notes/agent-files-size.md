# Agent-files size

The line count of the agent-files, one row per landing, so the set's size is tracked over time.
Smaller is the quasi-goal: a rule stated once is shorter than a rule stated three times, and a
shrinking count is evidence the set is converging, while a growing one is a prompt to ask what
arrived as a paragraph that should have been a line. The count is not a rule, and a rule is never
cut to move it.

A finding from the proposal cycle (2026-08-27), for whoever next tries a cut: a draft that cut
AGENTS.md from 370 to 320 lines with every rule kept found that the remaining bulk is the rules
themselves, so a further cut has to cut rules, and that is a convention decision, not an edit.

The count is `wc -l AGENTS.md custom.md agent-data/*.md`, taken at close-out and recorded here as
the closing rung's last edit, with the cycle title as the row's label.

## Counts

| Landed | Cycle | Files | Lines | Note |
|---|---|---|---|---|
| 2026-08-25 | docs: the family agent-files proposal opening | 10 | 2205 | vc-x1's set |
| 2026-08-27 | docs: sync the agent-files to zc-ring-x1's set | 12 | 2822 | two files back, 863 |
| 2026-08-28 | docs: the family agent-files proposal | 11 | 2126 | cycle-checklists and cycle-protocol folded into AGENTS.md, commit-model added |
| 2026-08-31 | docs: point custom.md at the messages repo | 11 | 2127 | the messaging line replaces custom.md's `_None._` |
| 2026-08-31 | agent-files(adoption): from iiac-perf, zc-ring-x1, 2026-08-31 | 10 | 2109 | iiac-perf's set verbatim, messaging.md out |
| 2026-08-31 | agent-files(proposal): to iiac-perf, zc-ring-x1, 2026-08-31 | 10 | 2158 | the commit-vocabulary section and its rationale, declared types in |
| 2026-09-01 | agent-files(proposal): v0.1.0 | 10 | 2230 | the set's version and its rationale, titles by version, the empty version file uncounted |
| 2026-09-04 | agent-files(adoption): v0.2.0 | 10 | 2231 | iiac-perf's set verbatim: `## Closed` last in the Todo format list, `# References` named, the re-pack parenthetical dropped |

Per file at the last row, replaced at each close-out, the history being in the commits:

```
   369 AGENTS.md
    12 custom.md
    92 agent-data/code.md
    42 agent-data/commit-model.md
    76 agent-data/cycle-model.md
   376 agent-data/jj.md
   171 agent-data/notes.md
   401 agent-data/prose.md
   488 agent-data/rationale.md
   204 agent-data/versioning.md
```
