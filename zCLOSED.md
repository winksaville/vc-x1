# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: turn off the session lock

(wink, 2026-10-13) The session lock, `.agent-running`, is an instruction in `custom.md` that every
session follows at acquaint: a probe, a background holder, and a sentence. It guards against two
sessions in one workspace, which one user with one session at a time does not have, so the cost
is paid at every acquaint for a case that does not come up. No Todo entry was picked. The entry
nearest it, the proposal of the lock to the set with the instruction's two faults, is ended by
this cycle.

#### Deliberation

- The lock is turned off and not deleted, wink's call.
  - The instruction is kept word for word in [session-lock.md](notes/session-lock.md), and
    `custom.md` keeps an entry that says the lock is off and where the instruction is, so turning
    it on is moving the text back.
  - The entry says off in its first words. Acquaint has a session do what `custom.md` asks, and
    a bare pointer at an instruction could be read as the instruction.
  - `.gitignore` keeps its `/.agent-running` line, so a lock turned on again needs no second edit,
    and the file a session left on disk stays ignored.
- The cost accepted: nothing keeps a second session out of the workspace.
  - Two sessions lose no data and leave a wrong record, a title that does not say what its commit
    holds and a trailer that names the wrong session, as the tests of 2026-10-12 showed.
  - One session per workspace is then the user's habit and no longer a check.
- The proposal of the lock to the set is dropped, and what it had worked out is kept beside the
  instruction: `flock` being a Linux tool, the thought of a small `vc-x1` command in its place,
  and the instruction's two faults.
  - The faults no longer bite, there being no instruction to forget or to edit.
- The session that makes this cycle keeps the holder it took at its acquaint.
  - The edit reaches the sessions started after it, which is one of the two faults, and here it
    is what is wanted: stopping the holder by hand shows that an agent did as it was told and
    nothing about the instruction.
- The Todo entry for the lock `vc-x1` takes for a run still says the session lock keeps a second
  agent out. It is left as it is, the agent's placing, since the next cycle removes the entry.
- A single-step cycle, with a bare version and no `-dev` name, the cycle being documents alone.

#### Solution

The instruction moves out of `custom.md` into [session-lock.md](notes/session-lock.md), and
`custom.md` says the lock is off and points there.

- The notes file says the lock is off, holds the instruction as it was, and says what turning it
  on again would want settled.
- The Todo entry for the proposal to the set is removed. The scratch its tests left is an entry
  of its own, wink's to delete.
- The Ideas entry for several agents in one workspace no longer says the lock refuses them.

#### Acceptance check

- `vc-x1 validate` passes. Passed.
- `grep -c flock custom.md` prints 0. Passed.
- A session started after Land acquaints without running `flock` and without saying "Lock
  taken.", and `flock -xn .agent-running true` run from another terminal while that session works
  exits 0. Not run: it wants a session started after Land, and the next cycle, which that
  session runs, records it.

#### Ladder

- docs: turn off the session lock (done)

# References
