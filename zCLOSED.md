# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: the session lock is settled as flock

(wink + agent, 2026-10-11) The Todo as it was picked, **The lock a running agent holds**, less
its plan for `.vc-x1-running`, which stays in `## Todo` as an entry of its own. The file is
`.agent-running` at the workspace root, in `.gitignore`. It is a mutex a running agent holds, one
session per workspace, and a second session that finds it held refuses to work. The entry ended
on an open line: whether the lock is a counter, the `flock`, or a counter guarded by one.

- Wink's first design: the file holds a counter, a session adds one and takes one away, and the
  file is deleted at 0.
  - The update is a read, change, and write that two sessions starting together can interleave,
    so it wants a guard.
  - A session that dies never takes its one away, and a number does not say which holder is dead.
- Linux's advisory lock, `flock`, answers both: the kernel releases it when its holder exits or
  dies. The file then has no contents and is never deleted.
  - The first form was shared: each session held it shared, and rename and delete asked for it
    exclusive without waiting. A second session then started as freely as the first.
  - Wink's call, 2026-10-11: one session per workspace. The session holds it exclusive and asks
    without waiting, so a second session's request fails at once instead of queueing.
  - It is advisory: it binds the processes that ask. Claude Code itself still starts, and the
    refusal is the agent's, by the `custom.md` line.
  - The lock lasts as long as the process holding it, and an agent's commands each exit at once,
    so a session needs one process that lives exactly as long as it does.
- The experiment: the agent takes the lock at acquaint, by a line in this project's `custom.md`,
  which [Acquaint](AGENTS.md#acquaint) already reads and acts on.
  - The holder is a background command the agent starts, `flock -xn .agent-running sleep
    infinity`, so there is no release for the agent to forget.
  - A foreground probe comes first, `flock -xn .agent-running true`, since a background command
    reports its failure seconds later, after the session has moved on.
  - A refusal is one fixed sentence, so the session offers no way around the lock.
  - The check is `flock -xn .agent-running true` from another terminal: it fails while a session
    is open and succeeds after.
- What the experiment was to find out: whether the holder dies with its session, what a resumed
  session does, what a reacquaint does in a session that no longer remembers its holder, how an
  edit to the line reaches a session already open, and how rename and delete check.

#### Deliberation

- The lock is the `flock`, exclusive, and no counter, wink's call once the tests were in.
  - Every way a session ends releases it, so nothing is left for a counter to count.
  - The tests came before the cycle was opened, wink's call, each of them ending the session
    that would have reported it.
- The holder dies with its session, however the session ends.
  - A clean exit: a session was ended with `/exit`, and the next session's probe succeeded.
    Claude Code asks before that exit, the holder being a background command still running, and
    exiting anyway kills it. The question is one more key at every exit.
  - A crash: wink's check from a plain terminal exited 1 with a session open, Claude Code was
    killed with `kill -9`, and the check exited 0 twenty seconds later.
  - A closed terminal: the check exited 1 with the session open and 0 after its terminal
    window was closed.
  - A power loss: wink's pull-the-plug test, 2026-10-11, with a session open. With the machine
    back up the check exited 0, `.agent-running` was still there and empty, and `vc-x1 sync` and
    `vc-x1 st -s both` found both repos up to date and clean.
  - Each session ran its commands in Claude Code's sandbox. One run outside it is untested, and
    we think it is the same, the kernel releasing a lock when the last process holding it dies.
- A resumed session holds no lock, and the line now tells it to take one.
  - The session the closed terminal ended was resumed with `claude --resume`, and the check
    from a plain terminal still exited 0.
  - Claude Code tells the resumed session that its holder was stopped when the earlier process
    exited, and starts nothing in its place.
  - The line as it was gave that case the refusal sentence and the stop, the holder having
    exited, and the sentence would have been untrue, no other session holding the lock.
  - The session ran no command until wink said to take the lock again.
  - "If that command ever exits" became "exits while the session runs", so the two cases do not
    read as one.
- The probe run by the session that holds the lock exits 1, confirmed by running it.
  - A reacquaint in a session that no longer remembers its holder would read that as another
    session and stop. The line's "a session that already holds one does neither" is all that
    stands against it, and the case goes to the `.vc-x1-running` entry with the hook command.
- The link at the end of the lock line is dropped, the entry it named being this record.
- The `.vc-x1-running` plan is an entry of its own, first in `## Todo`, wink's call.
  - It takes what the session lock leaves: the hook command, the reacquaint cases, and the
    proposal of the lock to the set.
  - It answers how rename and delete check, the process that asks being the process that
    holds.
- The cycle is two stages and one work commit, wink's call, as the last two cycles ran.
  - It bends [Cycle shape](AGENTS.md#cycle-shape), [Cycle-record](AGENTS.md#cycle-record),
    [Opening](AGENTS.md#opening), and [Close-out](AGENTS.md#close-out) as the record in the
    landmark `docs: update single-step cycle add zCLOSED.md` says, the record kept in this file
    included.
  - The fold of the closing into the work commit is by hand again, `push --amend` not being
    written, and is a coordinated re-describe with one trailer written by hand.
  - The bend covers this cycle's two pushes and their shape. Land takes its own go.
- The title was counted before the bookmark was published, 42 characters, and both were
  approved by wink at the opening.
- The `## Waiting` conditions are unmet, `vc-x1 closed` not landed and jj-lib offering no remote
  query, so nothing promotes.

#### Solution

The session lock is the `flock`, exclusive, with no counter, and what is left is planned in
`## Todo`.

- The lock line in `custom.md` lost its link and gained the resumed session's case.
- The Todo entry became this record, with its results and `flock`, exclusive, as the answer to
  its open line.
- The `.vc-x1-running` plan stands alone as the first entry in `## Todo`, and carries what the
  session lock leaves.

#### Acceptance check

- `vc-x1 validate` passes. Passed, on the pushed work and again on the closing.
- A session that met the new line cold takes the lock again when resumed: a fresh session is
  started and acquaints, its terminal is closed, it is resumed and sent one message, and its
  transcript shows the probe and the holder as its first two commands. The check from a plain
  terminal then exits 1. Not run, wink's call, to close the cycle the day it opened.
  - The closing session met the line cold at a fresh acquaint, and the probe and the holder
    were its first two commands. That is the line's acquaint case and says nothing of the
    resumed one.
  - The run is a line in the Todo entry [vc-x1 takes a lock of its own for every
    run](TODO.md#vc-x1-takes-a-lock-of-its-own-for-every-run).

#### Ladder

- docs: the session lock is settled as flock (done)

#### Closing

Close-out shape: one commit, the closing folded into the work commit.

- The power loss was tested after the work was pushed, and its result is in the Deliberation
  with the other three ways a session ends.
- The parked proposal was reviewed in the closing session and stays parked, wink's call.
  - Its Todo entry, [agent-files(proposal): v0.2.8](TODO.md#agent-filesproposal-v028), has
    what the review settled: a look at acquaint, a sync at the opening, and a sync at Land.
- One of the closing's validations failed in two tests and passed when run again, nothing
  having changed. The agent re-ran it before looking, and wink's call is that a failure is
  investigated first.
  - It is the Todo entry [A validation failure is investigated before it is run
    again](TODO.md#a-validation-failure-is-investigated-before-it-is-run-again), with the cause
    as far as it was found.
- The size table has one column for each version of the set, wink's call: three columns named
  v0.2.7 read as three versions of a set that had not changed.
  - A local change updates the newest column in place and has its own row above, so the
    `v0.2.7-lock` column the last cycle added is folded back and v0.2.5 returns.

# References
