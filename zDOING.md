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

_None._

## In Progress

### feat: undo takes back the last vc-x1 run

Picked from `## Todo` as **undo takes back the last operation** (wink + agent, 2026-10-28), at the
opening of **push refuses rewriting the work trunk**, which this opening first carried and gave up
for this, wink's call (2026-10-28): undo is the safety net the push and squash rules lean on, so it
comes first. Run in the v0.3.1 form, wink's call (2026-10-28): every cycle from now on.

#### Problem

A vc-x1 write is local and so can be taken back, but only by raw jj, `jj op restore` in each repo,
two op logs for `-s b`, the agent-repo's moving with every session write. A `squash-push -s b` that
folds hours of uncommitted work into `@-` by mistake and pushes it leaves the work recoverable from
the op log and the remote rewritten, and getting both back means finding the right operation in each
repo by hand, restoring without clobbering the files on disk, and pushing the bookmark back past a
lease that the restore has made stale.

#### Solution

- Each vc-x1 run records itself in the op log jj already keeps, with no state file to disagree
  with it:
  - a run is one vc-x1 invocation, and a write is a run that commits an operation other than a
    working-copy snapshot or a git import, the operations nearly every run makes first
  - every operation a run commits carries the run's id and command as operation attributes,
    `Transaction::set_attribute`, the map jj-cli keeps its own `args` in, so `jj op log` shows them
  - they are set where each transaction starts: `Session::start_tx` and the three that start their
    own, the working-copy snapshot and the git HEAD and git refs imports.
- `undo -s w|a|b|p` takes back the last write in the scoped repos, `-s b` both together:
  - the commits and bookmarks the write changed are reverted, the way `jj op revert` does it, a
    three-way merge of the state after the write's last operation back to the state before its
    first, onto the current one, so the named-run case below reuses it
  - the working copy keeps what is on disk now, `@` made to hold it, so an edit after the write and
    the agent journal the session keeps appending to are never cut back
  - an operation after the write other than a working-copy snapshot, another vc-x1 write or a raw
    jj command, is refused and named, since undoing the write would undo it too.
- undo rolls back what the write pushed: each remote bookmark it moved, from O to N, is shown and
  asked for, then pushed back to O with the lease at N, so a remote someone moved since is reported
  rather than overwritten.
- `undo --run <id>` takes back an earlier write, later writes kept, when what followed is
  independent of it:
  - `undo --list` shows the recent writes, each with its id and command
  - the revert is computed without being committed, and a result with a conflicted bookmark, a
    divergent change, or a remote bookmark moved since the write is refused, what tangled named
  - a clean result is shown and asked for, its push-backs as above.
- One undo per write: a redo is later, the attributes carrying it.
- The help, the README, and the agent-files name undo as how a mistake is taken back.

#### Acceptance check

`cargo test` passes with tests that:

- every write command's operations carry the run's attributes, in each repo it touches
- the mistake: hours of work in the work `@`, `squash-push -s b`, then `undo -s b` puts the work back
  in `@`, `@-` as it was, and the work bookmark on `origin` back at O, and an agent journal written
  in between survives
- `undo -s w`, `-s a`, and `-s p` take back their own repo's run alone
- an operation after the run is refused and named, a declined push-back leaves the remote as it is,
  and a remote moved since the run fails its lease and is reported
- `undo --run` takes back an earlier write with an independent write after it, the later one kept,
  and refuses one whose later writes built on it or moved its bookmark, naming them.

#### Deliberation

- Undo first, wink's call (2026-10-28): the rules that stop refusing rewrites rest on everything
  local being undoable, so the net is built before the rules change. The push material went back
  to `## Todo` as **squash-push shows what it takes, and push follows jj**.
- The scenario that decided it, wink's (2026-10-28): a `squash-push -s b` folding hours of work by
  mistake and pushing it. jj keeps every earlier state, so nothing is lost locally, and the push is
  the part with no one-command way back.
- The op log is the record, not a file: the attributes travel with the operations, so the last run
  is found by reading the log, and what it pushed by comparing the remote-tracking bookmarks before
  and after it.
- The files on disk are not restored: a plain `jj op restore` checks out the old `@`, which would cut
  the live session's journal back and overwrite edits made since.
- The push-back asks: it is an outward action, so it takes the user's go like any push.
- A mistake noticed late, wink's (2026-10-28): writes made after it may be independent of it, so
  undoing them too loses good work. jj's revert undoes one operation and keeps the later ones, and
  the run attributes say which operations make up the write, so undo is built on revert and takes a
  named run as its own rung. In the squash-push case the later work usually builds on the folded
  `@-`, so the named run helps most when the mistake is caught early or the later work was
  elsewhere.
- **sync --undo takes back the last sync** may become a case of it, a sync being a vc-x1 run.
- Multi-step: the recording, the local undo, the push-back, the named run, and the docs, each
  reviewable alone.

#### Ladder

- [feat: undo takes back the last vc-x1 run opening][p0] (done)
- [feat: each vc-x1 write records its run in the op log][p1]
- [feat: undo restores the repos to before the last run][p2]
- [feat: undo rolls back the last run's pushes][p3]
- [feat: undo takes back a chosen run when what followed is independent][p4]
- [docs: undo is how a mistake is taken back][p5]
- [feat: undo takes back the last vc-x1 run closing][p6]

##### feat: undo takes back the last vc-x1 run opening

Opens the cycle: the Todo moved here, the version bumped.

##### feat: each vc-x1 write records its run in the op log

Every transaction carries the run's id and command as operation attributes.

##### feat: undo restores the repos to before the last run

`undo -s w|a|b|p` reverts the last write's commits and bookmarks, keeps the files on disk, and
refuses later activity.

##### feat: undo rolls back the last run's pushes

Each remote bookmark the run moved is shown, asked for, and pushed back under a lease.

##### feat: undo takes back a chosen run when what followed is independent

`undo --list` and `undo --run <id>`: an earlier write reverted with the later writes kept, refused
when they built on it.

##### docs: undo is how a mistake is taken back

The help, the README, and the agent-files.

##### feat: undo takes back the last vc-x1 run closing

Closing out the cycle.

# References

[p0]: #feat-undo-takes-back-the-last-vc-x1-run-opening
[p1]: #feat-each-vc-x1-write-records-its-run-in-the-op-log
[p2]: #feat-undo-restores-the-repos-to-before-the-last-run
[p3]: #feat-undo-rolls-back-the-last-runs-pushes
[p4]: #feat-undo-takes-back-a-chosen-run-when-what-followed-is-independent
[p5]: #docs-undo-is-how-a-mistake-is-taken-back
[p6]: #feat-undo-takes-back-the-last-vc-x1-run-closing
