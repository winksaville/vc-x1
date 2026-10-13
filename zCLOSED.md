# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: vc-x1 takes no lock for a run

(wink + agent, 2026-10-13) Two `vc-x1` runs in one workspace can overlap, and a Todo entry planned
a lock, `.vc-x1-running`, that `vc-x1` would take for every run. Every run would pay for it, and
what it prevents does not come up with one user running one session at a time. The Todo entry
picked is "vc-x1 takes a lock of its own for every run". The entry "The parked planning of
2026-10-12 is run as a cycle" is ended by this cycle too.

#### Deliberation

- The lock is not added, wink's call.
  - Its cost is more than its worth, the reason the session lock was turned off.
  - The one overlap seen was made on purpose, two pushes started in the same second in the tests
    of 2026-10-12, and it lost no data.
- The entry moves to the backlog and is not deleted, wink's call, so the design is not lost.
  - Its text is the parked commit's, where the entry had already been moved to the backlog,
    brought up to date: the decision, what would reopen it, and the session lock being off.
  - The continuation notes had it written fresh and the parked text not taken. Wink reversed
    that at the opening.
- The bookmark `parked-planning-change-20261012` is deleted at Land, locally and on `origin`,
  wink's call. The backlog entry is what was wanted from it.
  - Dropped with it, drafts that were no cycle's rung:
    - `symlink` taking `create` or `remove`, and no `vc-x1 delete`
    - the Ideas entry for a shared work-repo's config, with `clone` taking the agent-repo's
      directory
    - an ochid carrying no path
    - the config's version field.
  - Its commit is `85f87d76`, change `wkytxkqo`, findable by that id in a clone that fetched it
    and nowhere on the remote once the bookmark is gone.
- Two lines of the entry "Improve init and add rename and delete subcommands" leaned on a lock
  and are brought up to date, the agent's placing: what stops delete and rename asks for no
  lock, and the lock's experiment with fresh sessions is gone.
- The third acceptance check of **docs: turn off the session lock** is recorded here, as that
  cycle's record said it would be.
- The commit body is its intro paragraph alone, wink's call at the description review, where
  [Commit-body form](agent-data/prose.md#commit-body-form) asks for the facets and their
  solutions under it. The exception covers this commit's body and nothing else.
- A single-step cycle, with a bare version and no `-dev` name, the cycle being documents alone.

#### Solution

The lock `vc-x1` was to take for a run is recorded as considered and not added, and the planning
parked on 2026-10-12 is ended.

- The backlog holds the entry, [vc-x1 takes a lock of its own for a run that
  writes](notes/todo-backlog.md#vc-x1-takes-a-lock-of-its-own-for-a-run-that-writes), with the
  decision and what would reopen it.
- The two Todo entries are removed, and [session-lock.md](notes/session-lock.md#what-was-decided)
  points at the backlog entry.

#### Acceptance check

- `vc-x1 validate` passes.
- `rg 'vc-x1-running|takes-a-lock-of-its-own-for-every-run|parked-planning' TODO.md notes/`
  finds `.vc-x1-running` in the backlog entry and nothing else.
- The session lock is off, the check **docs: turn off the session lock** left as not run. Passed:
  the session that ran this cycle started after that Land, acquainted without running `flock`,
  and did not say "Lock taken." The probe from another terminal was not run, wink having removed
  `.agent-running`, so there is no file to hold a lock on.

#### Ladder

- docs: vc-x1 takes no lock for a run (done)

# References
