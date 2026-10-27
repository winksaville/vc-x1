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
| 2026-08-25 | docs: the family agent-files proposal opening | 10 | 2205 | vc-x1's set |
| 2026-08-27 | docs: sync the agent-files to zc-ring-x1's set | 12 | 2822 | two files back, 863 |
| 2026-08-28 | docs: the family agent-files proposal | 11 | 2126 | cycle-checklists and cycle-protocol folded into AGENTS.md, commit-model added |
| 2026-08-31 | docs: point custom.md at the messages repo | 11 | 2127 | the messaging line replaces custom.md's `_None._` |
| 2026-08-31 | agent-files(adoption): from iiac-perf, zc-ring-x1, 2026-08-31 | 10 | 2109 | iiac-perf's set verbatim, messaging.md out |
| 2026-08-31 | agent-files(proposal): to iiac-perf, zc-ring-x1, 2026-08-31 | 10 | 2158 | the commit-vocabulary section and its rationale, declared types in |
| 2026-09-01 | agent-files(proposal): v0.1.0 | 10 | 2230 | the set's version and its rationale, titles by version, the empty version file uncounted |
| 2026-09-04 | agent-files(adoption): v0.2.0 | 10 | 2231 | iiac-perf's set verbatim: `## Closed` last in the Todo format list, `# References` named, the re-pack parenthetical dropped |
| 2026-09-04 | agent-files(proposal): v0.2.1 | 10 | 2238 | the agent-repo's directory comes from `.vc-config.md` at six sites, and the `ochid:` trailer's `/.claude` says at first use that it is a label and not one |
| 2026-09-04 | agent-files(proposal): v0.2.2 | 10 | 2255 | iiac-perf's notes on v0.2.1 plus wink's: the citation names the work-repo's config, the label rule keeps one home, the `[repos]` specimen says which cell the project chooses, and the `ochid:` prefix names a side rather than a path, and both repos are located by the `[repos]` entry that resolves home rather than by a literal directory |
| 2026-09-07 | agent-files(adoption): v0.2.3 | 10 | 2315 | iiac-perf's set verbatim by `vc-x1 agent-files copy`: eight rules corrected where they live with a why each in `rationale.md`, the dual-repo model as two definitions, the agent-files version tending to the patch, and the heading and term renamed "Agent-files version" |
| 2026-09-11 | agent-files(proposal): v0.2.4 | 10 | 2315 | the messaging pointer in `custom.md` names Pending instead of an inbox, one clause reworded on one line, so the count holds |
| 2026-09-16 | agent-files(adoption): v0.2.5 | 9 | 1774 | zc-ring-x1's proposal verbatim: a size row only when an agent-file changed, with its why, and `rationale.md` out of the count, so the files drop to 9 and the total loses its 543 lines |
| 2026-09-27 | agent-files(proposal): v0.2.6 | 9 | 1773 | the In Progress block's ladder moves below the Deliberation and heads the rung subsections, so the `Ladder details` heading goes and the specimen loses two lines |
| 2026-10-09 | agent-files(proposal): v0.2.7 | 9 | 1802 | `## Acquaint` in AGENTS.md, a session's first steps in order with a repo check of four findings, and its line in the rules index |
| 2026-10-11 | docs: a session takes the workspace lock at acquaint | 9 | 1810 | the lock line in `custom.md`, first in its list, eight lines and no change to the set |
| 2026-10-11 | docs: the session lock is settled as flock | 9 | 1811 | the lock line in `custom.md` loses its link and gains the resumed session's case, one line more and no change to the set |
| 2026-10-13 | docs: turn off the session lock | 9 | 1804 | the lock line in `custom.md` becomes an entry saying the lock is off and where the instruction is kept, seven lines fewer and no change to the set |
| 2026-10-13 | agent-files(proposal): v0.2.8 | 10 | 1873 | `## Sync` in AGENTS.md, one command at acquaint, opening, and Land, with its line in the rules index and a first step in Opening, an adoption as a copy and a check in `versioning.md`, the messages repo's fetch in `custom.md`, and a tenth file, `session-lock.md`, the lock's instruction for the family to decide on, 45 lines |
| 2026-10-15 | docs: the session lock moves to the README | 10 | 1871 | the entry saying the lock is off leaves `custom.md`, the README pointing at the set's instruction instead, two lines fewer and no change to the set |
| 2026-10-15 | agent-files(proposal): v0.2.9 | 10 | 1879 | the per-rung flow gains a step 0 sync, required at the opening and the closing, and the Validate step's rule that a failure is read before validation runs again, with Sync's moments and the rules index line to match, 8 lines |
| 2026-10-27 | feat: a hook enforces the shell rules | 10 | 1854 | from v0.3.0's 1841: Acquaint's Hooks step, Exit status as one form, a Hooks bullet, and two lines in the rules index, 13 lines, the four `.sh` files in `agent-data/` uncounted as the count is of `.md` |
| 2026-10-27 | feat: push, commit, and squash-push take flags only | 10 | 1855 | the per-rung flow's commit step and At rest's squash-push in AGENTS.md, and jj.md's push section, brought to `commit -s b`, `squash-push -s a`, and the flags, 1 line |
| 2026-10-27 | feat: the other commands take flags only | 10 | 1855 | versioning.md's adoption names `agent-files copy --from` and `diff --from`, rewrapped to the same length, and hooks.sh's guard test `install --dev`, uncounted as the count is of `.md`, no change |

Per file for the three most recent rows, newest on the left, the window sliding at each close-out
so the earlier history is in the commits. A column is labeled by the set version it carries, the
landings before the set was versioned relative to the first version (`- v0.1.0` one before it,
`-- v0.1.0` two before), and a landed local change updates the newest column in place and has
its own row above. A cell in angle brackets, `<543>`, is a file the total leaves out, shown so a
reader sees what it costs, and an empty cell is a file that was not in the set that landing.

| File | feat: the other commands take flags only | feat: push, commit, and squash-push take flags only | feat: a hook enforces the shell rules |
|---|---:|---:|---:|
| AGENTS.md | 436 | 436 | 436 |
| custom.md | 15 | 15 | 15 |
| agent-data/code.md | 90 | 90 | 90 |
| agent-data/commit-model.md | 42 | 42 | 42 |
| agent-data/cycle-model.md | 74 | 74 | 74 |
| agent-data/jj.md | 389 | 389 | 388 |
| agent-data/notes.md | 162 | 162 | 162 |
| agent-data/prose.md | 401 | 401 | 401 |
| agent-data/rationale.md | <653> | <653> | <653> |
| agent-data/session-lock.md | 43 | 43 | 43 |
| agent-data/versioning.md | 203 | 203 | 203 |
| total | 1855 | 1855 | 1854 |
