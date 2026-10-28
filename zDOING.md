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
  - every operation a run commits carries the run's record as operation attributes,
    `Transaction::set_attribute`, the map jj-cli keeps its own `args` in, so `jj op log` shows them
  - the record is typed on our side, the map being strings: its keys an enum, its id a 128-bit
    `RunId` in the UUID v7 layout, its command the subcommand clap parsed, its command line for
    people, and a format version
  - they are set where each transaction starts: `Session::start_tx` and the three that start their
    own, the working-copy snapshot and the git HEAD and git refs imports.
- `op log -s w|a|b|p` shows the op log by run, read with jj-lib (`op_walk::walk_ancestors` and each
  operation's metadata), the display ours since jj's templates and graph are jj-cli's:
  - a line per run, newest first: its id, when it ran, its command, and its operations in each
    scoped repo, a write told apart from a run that only snapshotted, `-s b` showing a run that
    touched both repos as one line
  - `--ops` lists each run's operations, and the untagged ones too, raw jj commands and operations
    from before the recording, and `--limit N` bounds it
  - read-only, with no snapshot, so looking at the log adds nothing to it
  - its reader, the operations walked, grouped by run, and each run's pushes found by comparing the
    remote-tracking bookmarks before and after it, is the one `undo --list` uses.
- `undo --list -s w|a|b|p` shows what undo can take, a run being the unit, so a `commit --closing
  --land` is one line however many operations it made:
  - a line per write, newest first: its id, when it ran, its command, the repos it touched with
    each one's operation count, and what it pushed, `main` from O to N or a bookmark deleted at O
  - a run that also acted outside the repos, an install or files init wrote, is flagged, since undo
    reaches the repos alone
  - `--ops` lists the operations under each line, `-v` being the global verbose flag already.
- `undo -s w|a|b|p` takes back the last write in the scoped repos, `-s b` both together, its
  operations in every repo as one unit:
  - the commits and bookmarks the write changed are reverted, the way `jj op revert` does it, a
    three-way merge of the state after the write's last operation back to the state before its
    first, onto the current one, so the named-run case below reuses it
  - the working copy keeps what is on disk now, `@` made to hold it, so an edit after the write and
    the agent journal the session keeps appending to are never cut back
  - an operation after the write other than a working-copy snapshot, another vc-x1 write or a raw
    jj command, is refused and named, since undoing the write would undo it too.
- undo rolls back what the write pushed: each remote bookmark it moved, from O to N, is shown and
  asked for, then pushed back to O with the lease at N, so a remote someone moved since is reported
  rather than overwritten, and one it deleted is recreated at O, the lease that it is still absent.
- `undo --run <id>` takes back an earlier write, later writes kept, when what followed is
  independent of it:
  - the id is the one `undo --list` shows
  - the revert is computed without being committed, and a result with a conflicted bookmark, a
    divergent change, or a remote bookmark moved since the write is refused, what tangled named
  - a clean result is shown and asked for, its push-backs as above.
- One undo per write: a redo is later, the attributes carrying it.
- The help, the README, and the agent-files name undo as how a mistake is taken back.

#### Acceptance check

`cargo test` passes with tests that:

- every write command's operations carry the run's attributes, in each repo it touches
- `op log` shows a `-s b` run as one line with each repo's operations, a snapshot-only run as no
  write, and with `--ops` the untagged operations, and commits no operation itself
- `undo --list` shows a `commit --closing --land` as one write with both repos' counts and its
  pushes, leaves out snapshot-only runs and raw jj commands, and with `--ops` lists the operations
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
- The run record typed, wink's (2026-10-28): the attribute keys an enum and the id an integer, the
  strings jj stores made and read in one place, and a value that does not parse is no run of ours.
  The id is 128 bits, not 64: a u64 is ample while ids stay in one repo's local op log, but they
  will likely leave it, in a trailer, a message, or another engine, and the UUID v7 layout sorts
  by time at no visible cost, since people type a prefix. A random id needs no counter to
  coordinate, which a sequence would. The command is its own attribute, so undo can reason by
  command without parsing the command line, and the format version lets a later vc-x1 read an
  older record.
- The files on disk are not restored: a plain `jj op restore` checks out the old `@`, which would cut
  the live session's journal back and overwrite edits made since.
- The push-back asks: it is an outward action, so it takes the user's go like any push.
- A mistake noticed late, wink's (2026-10-28): writes made after it may be independent of it, so
  undoing them too loses good work. jj's revert undoes one operation and keeps the later ones, and
  the run attributes say which operations make up the write, so undo is built on revert and takes a
  named run as its own rung. In the squash-push case the later work usually builds on the folded
  `@-`, so the named run helps most when the mistake is caught early or the later work was
  elsewhere.
- `op log`, wink's (2026-10-28): once runs are recorded, vc-x1 should show them itself, vc-x1
  being the interface and jj the engine. jj-lib walks the op log and gives each operation's
  metadata, and only the display is jj-cli's, so ours is shaped by runs, which jj knows nothing
  of. It comes right after the recording so the undo rungs have it while they are built.
- `undo --list` beside `op log`, wink's call (2026-10-28), held open to see how it feels: the run is
  the unit a person thinks in, so `commit --closing --land` is undone whole, its ten-odd operations
  and its pushes, and the pieces stay out of sight until asked for. `undo --list` shows exactly what
  undo can take, and `op log` everything, from one reader. `--ops` opens the pieces, not `-v`, which
  is the global debug log. A trial undo for every listed run was weighed and left out: it costs a
  revert per line and goes stale with the next operation.
- **sync --undo takes back the last sync** may become a case of it, a sync being a vc-x1 run.
- Multi-step: the recording, the op log, the local undo, the push-back, the named run, and the
  docs, each reviewable alone.

#### Ladder

- [feat: undo takes back the last vc-x1 run opening][p0] (done)
- [feat: each vc-x1 write records its run in the op log][p1] (done)
- [feat: op log shows each run and its operations][p2]
- [feat: undo restores the repos to before the last run][p3]
- [feat: undo rolls back the last run's pushes][p4]
- [feat: undo takes back a chosen run when what followed is independent][p5]
- [docs: undo is how a mistake is taken back][p6]
- [feat: undo takes back the last vc-x1 run closing][p7]

##### feat: undo takes back the last vc-x1 run opening

Opens the cycle: the Todo moved here, the version bumped.

##### feat: each vc-x1 write records its run in the op log

Every operation a vc-x1 run commits now says which run made it:

- The run record is its own module, `src/jj/run.rs`, typed on our side of jj's string map:
  - `RunAttribute`, an enum, is the keys, each with its one spelling: `vc-x1-run`,
    `vc-x1-command`, `vc-x1-args`, and `vc-x1-run-format`
  - `RunId` is a `u128` in the UUID v7 layout, 48 bits of Unix milliseconds, the version, the
    variant, and 74 random bits, written as 32 hex digits and read back only from exactly that
  - the command is the subcommand clap parsed, the command line has each argument quoted as a
    shell word where it needs it, and the format is 1.
- `main` begins the run before any repo is opened, and a test, with no `main`, gets one on first
  use with the command `?`.
- Every transaction a session starts is stamped with the record in jj's own attribute map, the one
  jj-cli keeps `args` in, so `jj op log` shows it with no template of ours.
- The stamp goes where each transaction starts: `start_tx`, which every verb uses, and the three
  the snapshot cycle starts on its own, the working-copy snapshot and the git HEAD and git refs
  imports. Every session the process opens shares the run, so `-s b` stamps both repos with one
  id.
- Tests: in-process, a commit, a git branch, and a git HEAD move make an operation at each of the
  four places, and every one carries the whole record. As a command, `commit -s b` leaves the same
  128-bit id in both repos' new operations, with its command, command line, and format. The id's
  layout, its order by time, and its round trip are tested alone.
- Not done: vc-x1's operations still carry no hostname or username, since the settings it builds
  lack the defaults jj-cli adds. The run attributes are what identify them.
- Committed with `vc-x1-dev commit -s b`, the rung's own build, wink's call (2026-10-28), dogfooding
  the work in progress being typical: the rule names `vc-x1 commit`, and the bend covers this
  commit and its push alone. It is the first write whose operations carry the record.

##### feat: op log shows each run and its operations

`op log -s w|a|b|p`: the op log by run, read with jj-lib, `--ops` for each run's operations.

##### feat: undo restores the repos to before the last run

`undo --list` and `undo -s w|a|b|p`: the writes listed, and the last one's commits and bookmarks
reverted as one unit, the files on disk kept, later activity refused.

##### feat: undo rolls back the last run's pushes

Each remote bookmark the run moved is shown, asked for, and pushed back under a lease.

##### feat: undo takes back a chosen run when what followed is independent

`undo --run <id>`, the id from `op log`: an earlier write reverted with the later writes kept, refused
when they built on it.

##### docs: undo is how a mistake is taken back

The help, the README, and the agent-files.

##### feat: undo takes back the last vc-x1 run closing

Closing out the cycle.

# References

[p0]: #feat-undo-takes-back-the-last-vc-x1-run-opening
[p1]: #feat-each-vc-x1-write-records-its-run-in-the-op-log
[p2]: #feat-op-log-shows-each-run-and-its-operations
[p3]: #feat-undo-restores-the-repos-to-before-the-last-run
[p4]: #feat-undo-rolls-back-the-last-runs-pushes
[p5]: #feat-undo-takes-back-a-chosen-run-when-what-followed-is-independent
[p6]: #docs-undo-is-how-a-mistake-is-taken-back
[p7]: #feat-undo-takes-back-the-last-vc-x1-run-closing
