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
on, and reset to `_None._` by the reader. Kept here by the trial, with a pointer to it under the
same heading in `TODO.md`, where [Acquaint](AGENTS.md#acquaint) still looks.

(agent, 2026-10-12) The session ended with the cycle below one rung short of closed.

- The cycle `docs: the session lock` is in flight on the bookmark `docs-the-session-lock`, its
  third rung pushed. The next step is the closing rung, at version 0.84.26.
  - The closing replaces the record in `zCLOSED.md` with the block below.
  - Its Land is a trapezoid, wink's call, on a go of its own: the closing gets two parents,
    `docs: update single-step cycle add zCLOSED.md` first and the third rung second, and `main`
    moves forward onto it.
- The bookmark `parked-planning-change-20261012` is published and is one commit on
  `docs: the session lock is settled as flock`, a draft of `TODO.md` planning and no cycle's
  rung. It is to be rebased after Land and run as a cycle of its own. It holds:
  - the run lock moved to the backlog, which the agent now advises against, the same-second
    test being that lock's case
  - `symlink` taking `create` or `remove`, and what stops rename
  - the Ideas entry for a shared work-repo's config, with `clone` taking the agent-repo's
    directory
  - an ochid carrying no path, and the config's version field.
- The bookmark `agent-filesproposal-v028` does not descend from `main`, reported at the acquaint
  of 2026-10-12 and not yet answered.
- Scratch that wink may delete: `/tmp/hook-probe`, `/tmp/vc-x1-two-sessions`, and the workspace
  `../agent-tests`.

## In Progress

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
- Every rung has a bare version, 0.84.25 for the rung made today and 0.84.26 for the closing.
  - The first two landed bare, 0.84.23 and 0.84.24, and are published.
  - A suffixed version wants the `-dev` name beside it, and the cycle is documents alone.

#### Solution

The session lock stays as it is. What a hook could do and what two sessions do without a lock are
written down in a notes file, three Todo entries carry what is left, and a closing merge draws
the trapezoid around the lock's commits.

#### Acceptance check

- `vc-x1 validate` passes.
- `custom.md` is unchanged, and no file under `src/` is changed.
- [session-lock.md](notes/session-lock.md) holds the probe's findings and the three tests, and
  each of the three Todo entries links it.
- After Land, `main` is a merge of two parents, `docs: update single-step cycle add zCLOSED.md`
  first and this cycle's third rung second, and `jj log -r 'main@origin..main'` prints nothing.

#### Ladder

- [docs: a session takes the workspace lock at acquaint][1] (done)
- [docs: the session lock is settled as flock][2] (done)
- [docs: two sessions in one workspace, tested][3] (done)
- [docs: the session lock closing][4]

##### docs: a session takes the workspace lock at acquaint

Landed 2026-10-11 as a single-step cycle. A line in `custom.md` has a session probe the lock at
acquaint and hold it with a background command, or refuse in one fixed sentence.

##### docs: the session lock is settled as flock

Landed 2026-10-11 as a single-step cycle, and its record is in [zCLOSED.md](zCLOSED.md) until this
cycle's closing replaces it. Every way a session ends releases the lock, so it is the `flock`
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

##### docs: the session lock closing

Closing out the cycle.

# References

[1]: #docs-a-session-takes-the-workspace-lock-at-acquaint
[2]: #docs-the-session-lock-is-settled-as-flock
[3]: #docs-two-sessions-in-one-workspace-tested
[4]: #docs-the-session-lock-closing
