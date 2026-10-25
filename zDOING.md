# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

(2026-10-25) **feat: vc-x1 commit takes a revision** is closed at 0.85.9 and not landed, its
closing pushed on `feat-vc-x1-commit-takes-a-revision` under wink's waiver, which left Land out.

- Land, on wink's go, from the work-repo with the closing at `@-`:

  ```
  vc-x1 sync -s b
  vc-x1 move-bookmark -s w --bookmark main --target @-
  vc-x1 push -s b main
  vc-x1 delete-bookmark -s w --bookmark feat-vc-x1-commit-takes-a-revision
  vc-x1 install
  ```

- The live acceptance run, wink's since it lands the fixture's `main` on GitHub, in
  `tmp/t-85-8.1-2ecac` with `vc-x1-dev` at 0.85.9, is the three lines of the record's acceptance
  check. Afterwards the fixture, its GitHub repos, `tmp/t-85-8.1+2ecac`, and `tmp/prompt.txt` are
  wink's to remove.
- The waiver covered the reviews and the pushes of the last four rungs and the closing, so their
  diffs and descriptions were not reviewed before they were pushed: a look at them is wink's.
- Filed during the cycle: **sync says one repo when the work @ is dirty**, **commit's dry run
  reports the push as the advance leaves it**, and **commit --land lands a closing already made**.

## In Progress

_No cycle currently in progress._
