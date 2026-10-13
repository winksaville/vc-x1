# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: the session lock

(wink + agent, 2026-10-12) No Todo entry was picked for managing a session lock. The first entry
held two plans, the lock `vc-x1` takes for a run and what the session lock, `.agent-running`,
left undone, and the cycle began as a rewrite of the second. The rewrite asked for a hook that
takes the lock in the agent's place, and a day of looking into that ended in a decision not to
build one.

The session lock was added as two single-step cycles, each landing on `main`. This cycle gathers
those two with what was found today into one ladder and closes it as a trapezoid.

#### Deliberation

- The lock stays as the instruction in `custom.md`, wink's call.
  - Two sessions were tested with no lock, wink's thought, to see what the lock is worth:
    pushes minutes apart, pushes in the same second, and one session resumed in two terminals.
  - Nothing either session wrote was lost, and every transcript line parses, but the commit
    title and bodies might be missing if they happened at the same time with the first one
    committing all the work that both did.
  - One line of instruction prevents that. What was found is in
    [session-lock.md](notes/session-lock.md), not here.
- The agent keeps taking the lock itself. Two other takers were considered, a Claude Code hook
  and `vc-x1`, and neither is built, wink's call.
  - A hook is a command Claude Code runs on its own when a session starts, so the lock would
    not depend on an agent reading the instruction and following it. A trial showed a hook can
    take the lock and hold it for as long as the session runs.
  - The hook is not built because it needs `CLAUDE_PID`, the id of the Claude Code process, to
    know when the session has ended. Claude Code's documents do not name that variable, so any
    release could drop it.
  - `vc-x1` taking the lock was thought to close a gap between the agent's probe and its hold.
    It closes none, wink's point: one asker wins a `flock`, whichever program asks.
- The current top two commits on main are being converted from being single-step
  cycles into a multi-step trapezoid, because in retrospect it should have been multi-step
  from the start, wink's call 2026-10-12.
  - Nothing published is rewritten. The two keep their ids, titles, and trailers, and the
    closing descends from both, so `main` moves forward onto it.
  - It is an exception to [Cycle shape](AGENTS.md#cycle-shape), where a single-step cycle
    lands as it is. It covers these two commits and the versions below, and Land takes its
    own go.
- Every rung has a bare version, 0.84.25 and 0.84.26 for the two rungs made in the cycle and
  0.84.27 for the closing.
  - The first two landed bare, 0.84.23 and 0.84.24, and are published.
  - A suffixed version wants the `-dev` name beside it, and the cycle is documents alone.
- A fourth rung is inserted ahead of the closing, wink's call, 2026-10-13
  ([Unplanned work](AGENTS.md#unplanned-work)).
  - The closing's review grew Todo planning that is no bookkeeping: `TODO.md` lost the headings
    the record left behind, and two entries were reworked.
  - The closing's own edits were set aside while the rung was made, so each commit has its own
    change alone.
  - It is outside the cycle's subject and is a rung all the same, the edits being made and
    reviewed, and a cycle of its own costing an opening, a closing, and a Land.

#### Solution

The session lock stayed as it was, an instruction in `custom.md` that the agent follows.

- What a hook could do and what two sessions do without a lock are in
  [session-lock.md](notes/session-lock.md).
- Three Todo entries carry what is left: the proposal of the lock to the set with the
  instruction's two faults, the lock `vc-x1` takes for a run, and `init` writing the agent
  sandbox's names into `.gitignore`.
- `TODO.md` is the todo list alone, by a rung the closing's review grew.
- The lock's two landed commits and the three made in this cycle are one ladder, and the closing
  is the merge that draws the trapezoid around them at Land.

#### Acceptance check

- `vc-x1 validate` passes. Passed, on the closing.
- `custom.md` is unchanged, and no file under `src/` is changed. Failed for `custom.md` and
  passed for `src/`, compared with the trapezoid's first parent,
  `docs: update single-step cycle add zCLOSED.md`.
  - `custom.md` gained the lock line, nine lines, which the first rung added and the second
    reworded.
  - Why it failed: the check was written for the work of 2026-10-12, before the first two rungs
    were counted as this cycle's, and as a trapezoid the cycle is everything since its first
    parent. The line is the cycle's subject, so the change is the intended one and the check
    was wrong.
- [session-lock.md](notes/session-lock.md) holds the probe's findings and the three tests, and
  each of the three Todo entries links it. Passed: the findings are under "What a hook could do",
  the tests under "What two sessions do with no lock", and the three entries, the second to
  the fourth of `## Todo`, each link the file.
- After Land, `main` is a merge of two parents, `docs: update single-step cycle add zCLOSED.md`
  first and this cycle's third rung second, and `jj log -r 'main@origin..main'` prints nothing.
  Not run: Land comes after this commit, on a go of its own, and the check is made then. The
  second parent is the fourth rung, one having been inserted after the check was written.

#### Ladder

- [docs: a session takes the workspace lock at acquaint][1] (done)
- [docs: the session lock is settled as flock][2] (done)
- [docs: two sessions in one workspace, tested][3] (done)
- [docs: TODO.md is the todo list, its split completed][5] (done)
- [docs: the session lock closing][4] (done)

##### docs: a session takes the workspace lock at acquaint

Landed 2026-10-11 as a single-step cycle. A line in `custom.md` has a session probe the lock at
acquaint and hold it with a background command, or refuse in one fixed sentence.

##### docs: the session lock is settled as flock

Landed 2026-10-11 as a single-step cycle, and its record is that commit's copy of `zCLOSED.md`,
which this cycle's closing replaced. Every way a session ends releases the lock, so it is the `flock`
alone, and a resumed session takes it again.

##### docs: two sessions in one workspace, tested

The lock rested on a guess that two sessions in one workspace do harm, and on an instruction an
agent has to follow. Both were looked into, and the lock is kept as it is.

- A notes file holds what a hook could do in the agent's place and what three tests of two
  sessions with no lock showed.
- The first Todo entry, which held two plans, is two entries.
  - The session lock's is first: the proposal to the set, and the instruction's two faults.
  - The run lock's is second, with the push that died as its first evidence. Two `vc-x1` runs
    in the same second is that lock's case and not the session lock's.
- A third entry is new: `init` writes the agent sandbox's names into `.gitignore`, a test
  having committed 23 empty files the sandbox put on disk.
- The Ideas entry for several agents in one workspace has the tests as its first try.

##### docs: TODO.md is the todo list, its split completed

`TODO.md` still had the headings of the records that moved out of it, each over a pointer, and
its `validate-anchors` and `push --amend` entries had grown hard to read. The headings go and the
entries are reworked.

- `TODO.md` is what we might do and nothing else: its intro, `## Waiting`, `## Todo`, and
  `## Ideas`.
  - `## Continuation notes`, `## In Progress`, `## Closed`, and `## Bugs` are removed, and the
    intro names `zDOING.md`, `zCLOSED.md`, and `bugs.md` in their place.
  - The split is completed for the file and not for the agent-files, which still name
    `TODO.md` for all four. That is left to the entry for the record's move.
- The three `validate-anchors` entries are one, a bullet for each known issue.
  - A fourth issue is new: a code span that wraps onto a second line hides a citation after it.
    Two of the three failures of a whole-repo run are that, and both are false.
  - A heading in `TODO.md` lost its backticks, which stepped around the code-span slug fault
    and fixed nothing.
- The `push --amend` entry is grouped by side: what the work-repo commit keeps and takes, what
  the new agent-repo commit is, and the approval.
  - The title and body are optional, and the new trailer goes at the bottom of the list.

##### docs: the session lock closing

The trapezoid is created, wink's call for the close-out shape, and `vc-x1 validate` passed.

- This commit is the trapezoid's merge. Land gives it its two parents,
  `docs: update single-step cycle add zCLOSED.md` first and the fourth rung second, and moves
  `main` forward onto it.

# References

[1]: #docs-a-session-takes-the-workspace-lock-at-acquaint
[2]: #docs-the-session-lock-is-settled-as-flock
[3]: #docs-two-sessions-in-one-workspace-tested
[4]: #docs-the-session-lock-closing
[5]: #docs-todomd-is-the-todo-list-its-split-completed
