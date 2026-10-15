# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: the session lock moves to the README

(wink + agent, 2026-10-15) `custom.md` carries an entry saying the session lock is off and
pointing at [agent-data/session-lock.md](agent-data/session-lock.md#the-instruction). The lock is
off by default, as that file says, so the entry restates the default, and the project layer
should hold only what the project uses. The lock stays in the set, since it may be wanted again,
and it must stay findable: nothing else links the set's file.

#### Deliberation

- The lock is not retired, wink's call: the set's file stays as it is.
- The README carries it, wink's choice over the notes' index: it is where a new reader, or wink
  later, looks first.
  - It says what the lock is, the two edits that turn it on, and how to tell it works.
  - The checks were run 2026-10-15 in one shell: a held lock makes `flock -xn .agent-running
    true` exit 1 and `fuser -v` names the holder, and it exits 0 once the holder ends.
- The set's file still says turning the lock on replaces "the entry that says the lock is off",
  and turning it off puts that entry back. With no such entry here the first reads as a no-op
  and the second is wrong for us, but changing it is an `agent-files` proposal: added to the Todo
  "The set's session-lock file says a killed holder leaves files behind", the same file's cycle.
- A line of `notes/session-lock.md` the last cycle left over the width is rewrapped, the file
  being touched here.
- A Todo is filed, wink's call at review: "sync fetches the messages repo too". The review asked
  whether our own repos need the messaging entry's fetch, and they have it already by
  [Sync](AGENTS.md#sync), while the messages repo, outside the two, is fetched by hand.
- The commit body is one paragraph, no facets, wink's wording at review: a bend of [Commit-body
  form](agent-data/prose.md#commit-body-form), which keeps the intro alone for bookends. It covers
  this commit's body only.
- Single-step, an agent-file line and its documentation.

#### Solution

`custom.md` drops the entry, and the README gains a section on the session lock that turns it on
and checks it, linking the set's instruction and the design notes.

- The README's "The session lock" says what the lock is and what goes without it, how to turn it
  on, two edits and a new session, and how to turn it off, and how to tell it works: the
  session's own words at acquaint, a probe and `fuser` from a shell, a second session stopping,
  and the probe freeing when the session ends. It names the two known faults.
- `notes/session-lock.md` and the notes index point at the README, and the set file's on and off
  steps join the Todo for that file's proposal cycle.
- The agent-files' size row records `custom.md` two lines shorter.

#### Acceptance check

- `custom.md` has no session-lock entry, and `rg -l 'agent-data/session-lock.md'` lists the
  README. Passed.
- The README's section names both edits, the restart, and the checks, and its links resolve:
  `vc-x1 validate-anchors README.md` passes for the same-file ones, and the cross-file anchors
  are checked by hand against their headings. Passed.
- `vc-x1 validate` passes. Passed.

#### Ladder

- docs: the session lock moves to the README (done)

Close-out shape: none to choose, single-step. Land fast-forwards `main` over the one commit.

# References
