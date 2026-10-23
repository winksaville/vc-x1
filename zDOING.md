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

### feat: vc-x1 closing and land

(wink + agent, 2026-10-21, opened 2026-10-23) Picked from `TODO.md` as **feat: vc-x1 land closes
a cycle and lands it**, retitled at the opening, wink's call, since the closing stays `commit
--closing` and the new command is `land`.

#### The Todo as picked

The close-out and Land are five commands typed in order, and each is a place to skip the sync,
move the wrong bookmark, or push before the review. One command runs the sequence, with the review
between its commit and its pushes, filed at the first closing made by `commit --closing`.

- `vc-x1 land -s b [--from BOOKMARK] [--to BOOKMARK] [--title STR --body STR|@PATH]`, `--from`
  the cycle's bookmark, the line `@-` is on by default, and `--to` the trunk, `main` by default.
- The sequence, each step today's command:
  1. `sync -s b`: `--to` local equals its remote, or stop, since a moved trunk is the Land rule's
     stop.
  2. `commit -s w --closing --title --body`: the merge of `--to` and `--from`'s tip.
  3. The review: `jj show @-`, and the command waits for the go, or `--yes` under a waiver.
  4. `move-bookmark -s w --bookmark <to> --target @-`: the fast-forward, no `--allow-backwards`.
  5. `push -s b <to>`: the agent half commits the journal as the partner, the work half publishes
     `--to`. The closing never reaches `--from`'s remote, so the two pushes of the by-hand Land
     are one.
  6. `build`, a `run -- -V` smoke check, and `install`, since they stage and deploy what `--to`
     now carries, the commands **feat: vc-x1 install stages, deploys, and runs builds** made.
  7. Delete `--from` locally and remotely, once `vc-x1` owns bookmark deletion. Until then it
     prints the two `jj` lines.
- Interactive: with no `--title` and `--body` it opens `$EDITOR`, as `jj describe` and `git gui`
  do, on a buffer pre-filled with the bookend's text, the title from the In Progress block's and
  the body `Closes the cycle "<title>". Its record is the Closed block in zCLOSED.md.`, so the
  common case is save and quit. This is the editor path `commit` left out until asked for.
- `--dry-run` prints the seven steps with what each would do, and does nothing.
- Not `commit -s b`: the journal of the command's own review is what the push commits as the
  partner.
- Wants bookmark deletion in `vc-x1` for step 7, which may land as its own cycle before this one.
  Step 6's commands are there.
- A single-step cycle skips steps 2 and 3: its one commit is on `main` already, titled with the
  bare cycle title, so there is no ladder to merge and `commit --closing` refuses one, by design
  and its test `closing_with_no_ladder_is_refused`. Its Land is the sync, the fast-forward of
  `--to` to the commit, the push, the install, and the deletion, so the command tells the two
  shapes apart, by the commits between `--to` and `--from`'s tip, one for a single-step cycle and
  at least two for a ladder, its opening and a rung, and says which it ran. (wink + agent,
  2026-10-22)

##### Two commands, and their undos

(wink + agent, 2026-10-21) The sequence above splits in two at the review, the names placeholders
until the opening:

- `vc-x1 close-out-commit --bookmark <x>`, `<x>` the current bookmark by default: the closing
  merge, made locally, and reviewed like any rung's commit, by `jj show @-` or gitk.
- `vc-x1 land-it --from <x> --to <y>`, `<x>` the current bookmark and `<y>` `main` by default:
  what follows the go, sync, the fast-forward, the push, the install, and the bookmark's deletion.
- So the review leaves the command. The agent's shell cannot answer a prompt, so a wait inside
  `land` would need `--yes` on every agent run, and the editor path goes with it. The split also
  matches v0.3.1's local commit pair before every push.
- Bookmark deletion is a rung of this cycle, a subcommand or a flag on `move-bookmark`, rather
  than a cycle of its own, since nothing else waits on it.

Defaults invite mistakes, and the two undos differ:

- **close-out-commit's undo is local and wanted now.** The closing never reaches the bookmark's
  remote, so undoing it rewrites nothing anyone has seen. jj's own undo is one repo's operation
  log and the command writes two, so the undo's worth is doing both sides together. The agent side
  is uncommitted, its content moved back into `@`, never restored to an earlier operation, which
  would roll back the session transcript and drop the lines written since. We think the operation
  restore is safe on the work side alone.
- **land-it is guarded rather than undone.** Its push moves `main@origin`, and undoing that is a
  force-move of the trunk backward, so a revert commit is the undo and moving `main` back sits
  behind an explicit flag. The deletion is undone by recreating the bookmark at the commit
  land-it printed, and the install by reinstalling from the previous `main`, which it also prints.
  A run stopped partway wants a resume, or an undo of the local steps not yet pushed, more than a
  finished land wants an undo.
- **Current** is the nearest bookmark on `@-`'s ancestry that is not the trunk. With `@` on `main`
  and no topic bookmark it would resolve to `main`, so both commands refuse that, and land-it
  refuses `--from` equal to `--to`.
- **land-it lands only a close-out**: the commit it lands must be a merge whose parents are `--to`
  and `--from`'s tip, the shape close-out-commit makes, so a cycle not closed out, or closed onto
  the wrong trunk, is refused.
- Both print what their defaults resolved to before acting, and `--dry-run` prints only that.
  land-it stops before its push with the plan printed, the last point where its undo is local.
- The lean: close-out-commit's undo in this cycle, as a flag or a sibling command. land-it's true
  undo, after the push, waits until a real mistake asks for it.

#### Deliberation

- The goal, wink's (2026-10-23): no cycle step needs `jj` typed by hand for its multi-step
  operations, each one easy to get wrong. So the cycle widened at the closing rung's review to every
  such step the agent-files name, the cycle being where the protocol's commands are being defined.
  - Added: reshaping a draft bookmark, the coordinated re-describe, local ladders, and setting a
    rung's edits aside, after the closing and landing path, so that path finishes first.
  - The title and the bookends keep the pushed name, the opening being published.
- `commit` makes and checks, and its bookend flags carry the rest, wink's call (2026-10-23): the
  closing stays `commit --closing`, not a new `close-out-commit`, since it exists, its tests exist,
  and the flag matches the bookend's " closing", and Land is `--land` on it, not a command of its
  own.
- `commit --closing` handles every cycle shape, each tested, wink's call at the opening
  (2026-10-23): no ladder, a single-step cycle; a ladder of one rung, the opening alone, which
  [Cycle shape](AGENTS.md#cycle-shape) says still closes as a ladder; two rungs, the only size
  tested before; and three or more.
  - The Todo's single-step line is mistaken: a single-step cycle's one commit is on its bookmark,
    not on `main`, since `main` advances only at Land. So the commit is made above `main`, and
    the old `has_ladder` would have called that a ladder.
  - Settled at the closing rung: `--closing` with the work `@-` in `::REV` makes the one commit
    plain, under the bare cycle title, and says it is a single-step cycle's, rather than being
    refused.
- The shape is the branch's node count, wink's call (2026-10-23): every cycle, single-step or
  multi-step, is on a branch, and a branch with one node above the trunk is a single-step cycle.
  - A branch whose one node is the opening is single-step, so [Cycle shape](AGENTS.md#cycle-shape)'s
    "a ladder that shrinks to one rung still closes as a ladder" changes: proposal material for
    v0.3.1 beside the closing's shapes, edited at the closing rung.
  - At `--land`, one node that is not a merge is a single-step cycle, fast-forwarded onto, and a tip
    that merges `REV` and the branch, or a linear ladder's closing, is a ladder. Anything else is
    refused, so no title is read to tell the shapes apart.
- The opening anchors every cycle and `--closing` derives the rest from the branch, wink's call
  (2026-10-23), superseding `--opening` with `--closing` on one commit:
  - A single-step cycle starts as `commit --opening <BOOKMARK>`, the work squashed into it, and
    ends as `commit --closing <REV> --land`, with no `--title` or `--body`, since the opening
    already carries them.
  - The cycle title is the first node above `REV`, the opening, its " opening" stripped, so
    `--title` and `--body` are optional at the closing.
  - On a branch of one node `--closing` makes no new commit: it finalizes the one commit in place,
    the bare cycle title replacing the opening's, the body kept with the record pointer added, and
    the record moved. Its change id is kept, so the trailers hold, and a draft already pushed to the
    bookmark is the coordinated re-describe the rules allow, done by the tool, the bookmark deleted
    at Land anyway.
  - On a branch of two nodes or more `--closing` makes the closing commit, titled "<cycle title>
    closing" by default, its body the configured default, its content the record move and whatever
    finalization `@` holds, a merge or linear per its shape.
- The record move is configured, since zDOING.md and zCLOSED.md are a trial and the agent-files
  still name `TODO.md > ## In Progress`, wink's call (2026-10-23), refined from his `[closing]
  zCLOSED-present`:
  - `[record] doing` and `closed` name the two files. When both are set, `--closing` moves the In
    Progress block and its references into the closed file's `## Closed`, replacing what it held,
    and resets `## In Progress` to "_No cycle currently in progress._". When absent, nothing moves.
  - `[closing] body` is the default closing body, its placeholders filled as the install lists'
    are, this workspace's `Closes the cycle "{title}". See details in zCLOSED.md.`, so a project
    without the trial sets its own text.
  - The record's finalization, the acceptance result, the final solution, and the closing
    subsection, stays the agent's writing, reviewed in `@` before `--closing` runs. The tool only
    moves it.
  - A work rung's body points at the record where it is at that commit, and the default closing
    body says zCLOSED.md exactly when the move puts it there.
- `--closing <REV>`, wink's call at the closing rung (2026-10-23): `commit` assumes nothing about
  the current bookmark or branch, so the revision the closing closes onto is named, any one commit,
  `REV` as jj calls it, a bookmark, change id, commit id, or `@` form.
  - `commit` named `main` because the merge needs a first parent and the shape check needs
    something to ask whether `@-` is in. Both are now `REV`, resolved once to a commit id.
  - Weighed: a separate `--trunk`, a config key, and a default found by walking down `@-`'s single
    parents to the nearest bookmark. The walk finds the cycle's own bookmark first, misses a target
    that moved since the fork, cannot tell `main` from the topic bookmark on a single-step cycle's
    one commit, and on a line with no fork runs to the root.
  - Required, with no default. A title ending " closing" without the flag was refused when `@-`
    sat above `main`, and with no revision to ask it is now a warning.
- `commit --opening <BOOKMARK>`, a rung inserted at the closing rung's review and then set ahead
  of it, wink's calls (2026-10-23):
  - Ahead of the closing rung so a test workspace's cycle can be opened by the tool before its
    closing is tried. The closing rung's edits, made and under review, were set aside as a jj node
    above the opening, holding its code and README alone, while the record, the proposal, and the
    version stayed in the working copy for this rung to carry. They return once this rung pushes.
  - The closing's suffixes and `--opening` with `--closing` moved to the closing rung, since each
    needs the `--closing <REV>` code.
  - The bookmark is created at `@-` and pushed, local and remote, before the opening commit is
    made, which stays local until its review and the next `push` publishes it. At `@-` the push
    publishes a name for a commit already public, so nothing unreviewed goes out, which is what
    Opening step 2 does by hand. This replaces the first call, the bookmark created on the opening
    for the next push, since wink wants the bookmark pushed by `--opening`.
  - The line's bookmark advances to each work commit, locally, wink's call at the opening rung
    (2026-10-23), the remote moving only at `push`, resolved as `push` resolves it, `--bookmark`
    naming it and `--no-advance` leaving it.
  - `--skip` names the step left out, settled at the opening rung: `local` skips the creation, so
    the bookmark must be at `@-` already and is pushed, `remote` skips the push, and `both` skips
    both and only checks it is at `@-`.
  - The push inside `commit` is a push, so `commit --opening` takes the approval a push does.
  - `--title` is the bare cycle title for both bookends: `--opening` appends " opening", and
    `--closing` appends " closing" on a ladder and nothing for a single-step cycle, a title already
    carrying the suffix refused so it is never doubled. `--dead-end` with `--closing` appends
    " dead-end closing".
- The closing's shape, trapezoid or linear, wink's calls (2026-10-23): for those who prefer a
  straight history, a linear closing is a plain commit on the ladder, and `--land` fast-forwards
  `REV` to it, the agent-files' keep separate under a clearer name.
  - `--shape <trapezoid|linear>` chooses it, one valued flag rather than `--linear` and
    `--trapezoid`, which could both be given, or abbreviations such as `--clse-lin`, which read as
    typos. The flag's word and the config key's are the same.
  - `[cycle] closing-shape`, `"trapezoid"` or `"linear"`, is the project's default, a preference
    that belongs in the committed `.vc-config.md` with `[validate]` and `[build]`, since adopters
    choose differently. Absent, it is trapezoid, so an adopter who sets nothing sees no change.
  - The flag, then the key, then trapezoid, and `commit` prints which decided. Close-out step 5
    still has the user choose, the key only making the usual answer the default, and a stated
    preference is not the assumption about the current branch that `--closing <REV>` removed.
  - Either converts to the other while the closing is a draft, before Land, by re-parenting the
    same closing, which keeps its change id and trailer, and with the trunk unmoved its tree too,
    `REV` being in the ladder's ancestry. After Land a conversion would rewrite the trunk, so it is
    refused.
  - A linear closing cannot join a trunk that moved: the ladder would need a rebase, a rewrite of
    published commits into a tree no validation saw. `--land` already refuses a moved trunk, so
    the refusal is the whole answer, and the way out is the reshape rung's command, then a
    validation, then the closing. A rebase inside a linear closing would be the hidden multi-step
    operation this cycle removes.
  - The v0.3.1 proposal said keep separate goes, and its section **The closing is a trapezoid or
    linear, and converts** now keeps it as the linear shape and names the flag and the key, edited
    at the closing rung's review when this rung was added, so the proposal leads the flag by one
    rung.
  - Its own rung after the closing's shapes, since `--land` lands both.
- `commit --closing <REV> --land`, wink's call (2026-10-23): Land needs no review of its own, so
  the closing and its Land are one invocation under the one go given at the closing's description
  review.
  - The per-rung flow already reviews everything the closing carries, its working-copy diff at
    step 6 and its description at step 8, and the commit adds only the merge's parents. A conflict
    is found by a check, not by eye.
  - The landed tree differs from the reviewed one only when the trunk moved during the cycle, the
    merge then bringing in commits no validation saw. Land's sync rule already stops that, and
    `--land` enforces it before anything moves: no conflict, `REV` local equal to its remote and
    the ladder forked from it, and `REV` a bookmark, since `--land` moves it.
  - Then the closing, the fast-forward of `REV`, the push of the trunk with the agent journal as
    its partner, and the topic bookmark's deletion, `--skip local|remote|both` keeping what it
    names. Whether the install joins it is settled at the rung, the install not being a `jj` step.
  - Set aside: a separate `land` command, and `push --land`, the agent's proposal, both of which
    kept a review between the closing and its Land that adds nothing.
- The closing's undo is in this cycle, the Todo's lean, for a closing made without `--land`, a flag
  or a sibling command settled at its rung. A Land's undo, after its push, waits for a real mistake.
- Bookmark deletion is a rung of this cycle, not its own cycle, since `--land` and `--skip` use it.
- Multi-step, ten work rungs between the bookends, the closing and landing path first, since each
  of its rungs is used by the next, then the four steps the widening added.

#### Problem

A cycle's multi-step operations are typed as raw `jj` commands in order, and each is a place to
make a mistake: the opening's `jj git push --named` creating and publishing the bookmark, the
closing, Land's sync, fast-forward, push, and two `jj` lines deleting the bookmark, the reshape of
a draft bookmark, the coordinated re-describe, local ladders, and setting a rung's edits aside, any
of them able to skip the sync, move the wrong bookmark, or push before the review. The goal, wink's
(2026-10-23), is that no cycle step needs `jj` typed by hand for these operations. `commit
--closing` assumed `main`, was tested on a ladder of two rungs only, and refused a single-step
cycle, and a closing made by mistake is undone by hand in two repos.

#### Solution

Provisional. Each multi-step operation of a cycle becomes one `vc-x1` invocation that checks before
it acts and prints what it resolved:

- `commit --opening <BOOKMARK>` creates and pushes the bookmark at `@-` and makes the opening,
  appending " opening" to the bare title, and every commit advances the line's bookmark locally.
- `commit --closing <REV>` closes every cycle shape onto the named revision, its title and body
  derived from the opening and the config. A branch of one node is finalized in place, the bare
  title replacing the opening's. A longer one gets a closing commit, titled " closing", or
  " dead-end closing" with `--dead-end`, a trapezoid or linear, from `--shape`, else `[cycle]
  closing-shape`, else trapezoid, converting between them while it is a draft. With `[record]`
  set, the record moves from the doing file to the closed file. A closing can be taken back in
  both repos at once.
- `commit --closing <REV> --land` also lands it: the fast-forward of `REV`, the push, and the
  bookmark's deletion, refused when the trunk moved or the merge conflicts.
- `vc-x1` deletes a bookmark locally and remotely, `--skip` keeping either side.
- `vc-x1` reshapes a draft bookmark, re-describes a pair keeping its trailers, runs a local ladder,
  and sets a rung's edits aside and back.

#### Acceptance check

`cargo test` passes with tests for:

- `commit --closing` finalizing a one-node branch in place, and closing ladders of two and three
  nodes, each shape from the flag, from the key, and by default, the title and body derived, the
  record moved with `[record]` set and left without, the conversion each way, the closing's undo
  restoring both repos, and `--land` landing a single-step cycle, a trapezoid, and a linear ladder
  on fixtures and refusing a moved trunk, a conflict, a revision that is not a bookmark, and a
  commit of none of those shapes.
- `commit --opening` creating and pushing the bookmark, each `--skip`, the title suffix, and the
  line's bookmark advancing.
- Bookmark deletion, the reshape, the re-describe, the local ladder, and the set-aside.

This cycle's own Land is run by the dev build's `commit --closing main --land`, its report showing
`main` and `main@origin` at the closing and the bookmark gone locally and remotely, and the
installed `vc-x1 -V` a bare 0.85.8.

#### Ladder

- [feat: vc-x1 closing and land opening][0] (done)
- [feat: commit --opening makes the opening and its bookmark][1] (done)
- [feat: commit --closing closes every cycle shape][2]
- [feat: commit --closing takes a shape, from the flag or the config][3]
- [feat: the closing can be taken back][4]
- [feat: vc-x1 deletes a bookmark][5]
- [feat: commit --closing --land lands the cycle][6]
- [feat: vc-x1 reshapes a draft bookmark][7]
- [feat: vc-x1 re-describes a pair and keeps its trailers][8]
- [feat: vc-x1 runs a local ladder][9]
- [feat: vc-x1 sets a rung's edits aside][10]
- [feat: vc-x1 closing and land closing][11]

##### feat: vc-x1 closing and land opening

The cycle's setup commit: create and publish the bookmark, move the Todo entry into this block,
and bump the version-of-record.

##### feat: commit --opening makes the opening and its bookmark

A cycle's opening is a commit titled by hand with " opening" and a bookmark created and published
by a separate `jj git push --named`, so `commit --opening <BOOKMARK>` creates and pushes the
bookmark at `@-`, `--skip` keeping it local or taking an existing one, and makes the opening,
appending " opening" to the bare cycle title.

* The bookmark and the opening are made in one invocation, in an order that never publishes what
  was not reviewed.
  - Every check runs before the commit: a bookmark to create must not exist, one taken by `--skip`
    must be at the work `@-`, a push must not move a bookmark published elsewhere, and `@-` must
    already be on a remote, so the push publishes a name and nothing new.
  - The bookmark is created and pushed after the commit, both halves of a `-s b` pair, so a failed
    commit leaves no bookmark. A failed push leaves the opening and the local bookmark, and the
    error says the push after the review publishes both.
* `--skip` names the step it leaves out, a closed set, `local`, `remote`, or `both`, and needs
  `--opening`.
  - `local` pushes a bookmark already at `@-`, `remote` creates one and pushes nothing, for a line
    not ready to be seen, and `both` only checks it.
* The title is the bare cycle title, and " opening" is appended, a title carrying it already
  refused, so a bookend's suffix is never typed by hand.
  - `--opening` and today's `--closing` exclude each other until the closing rung makes the two
    together a single-step cycle's one commit.
* After `--opening` the bookmark sat at the start, not on the opening, found when wink first ran
  it (2026-10-23), so every commit now advances the line's bookmark to the work commit, locally.
  - jj moves a bookmark with its commit through a squash or amend, so the advance holds through the
    review's fixes. Local moves with each commit and the remote only at `push`, the approved step,
    the split git has.
  - The bookmark is resolved as `push` resolves it, the nearest below the work `@-`, so the two
    never disagree, or named by `--bookmark`, or `--opening`'s. None or several is refused before
    anything is made, since `commit` assumes nothing about the branch. `--no-advance` leaves it.
  - The opening's bookmark is created and pushed at the start, then advanced, so the push still
    publishes only a commit already public.
* vc-x1's in-process revsets counted a colocated repo's `@git` refs as remote bookmarks, which the
  jj CLI leaves out, so a bookmark moved only locally made its commit read as published, and
  `squash` and `push`'s restamp refused it. Found by the advance, the first local-only move.
  - The revset parse now ignores the `git` remote as the CLI's does, which also keeps the opening's
    check that `@-` is public from passing a commit only exported to git.
* `commit` and `squash` pointed the review at `jj show @-`, raw `jj` in the step the cycle means to
  rid of it, found when wink first ran `--opening` (2026-10-23).
  - They print `vc-x1 show -s both @- -R <work>` for a pair, and `-s work` for a work half, whose
    agent half push commits later, so `-s both` would show the agent's previous commit.

##### feat: commit --closing closes every cycle shape

`commit --closing` is tested on a ladder of two rungs and refuses a single-step cycle, so it makes
the single-step commit and closes a ladder of one, two, and three or more rungs, each tested.

* `commit` named `main` as the closing's first parent and asked whether `@-` was in it, so it
  assumed the branch a cycle closes onto.
  - `--closing <REV>` names it, any one commit, resolved once to a commit id before anything moves
    and refused when it names none or several. A bookmark is read where it is now, so a target that
    moved during the cycle is still joined by the merge.
  - Without the flag there is no revision to ask, so a title ending " closing" is warned about
    rather than refused.
* A single-step cycle's one commit is made with the work `@-` in `::REV`, so there is no ladder to
  merge.
  - `--closing` there makes the commit plain, paired as any rung is, and says it is a single-step
    cycle's one commit, so the flag can be passed on every cycle's last commit whatever its shape.
  - The title check applies only on a ladder: a single-step cycle's title is the bare cycle title,
    which needs no " closing", and one that happens to end " closing" is just a title.
* The merge never depended on the ladder's length, only on the work `@-` not being in `::REV`.
  - One test now closes a ladder of one rung, the opening alone, of two, and of three, each in its
    own fixture, checking the merge's parents and that its tree carries every rung.
  - Telling a single-step commit already pushed from a ladder of one is `land`'s question, not
    this rung's, since `commit --closing` runs before the last commit exists.
* Still to do, from the opening rung and the plan since, each needing the `--closing <REV>` code:
  - The shapes by the branch's node count: a branch of one node, the opening, finalized in place,
    the bare cycle title replacing the opening's, no new commit, so the ladder-of-one test becomes
    the single-step test. Two nodes or more get the closing commit.
  - The title and body optional, derived from the opening, " closing" appended, or " dead-end
    closing" with `--dead-end`, and the body from `[closing] body`.
  - The record moved from `[record] doing` to `closed` when both are set.
  - The Cycle shape change in the v0.3.1 proposal.

##### feat: commit --closing takes a shape, from the flag or the config

The closing is always the trapezoid, a merge of `REV` and the ladder's tip, so `--shape
<trapezoid|linear>` chooses it, linear a plain commit on the ladder, the shape the agent-files call
keep separate, `[cycle] closing-shape` sets the project's default, trapezoid when absent, and
either shape converts to the other while the closing is a draft.

##### feat: the closing can be taken back

A closing made without `--land` and found wrong is undone by hand in two repos, so one invocation
takes it back, the work side by its operation and the agent side's content moved back into `@`.

##### feat: vc-x1 deletes a bookmark

Deleting a cycle's bookmark is two `jj` lines, so `vc-x1` deletes one locally and remotely,
`--skip` keeping either side, a subcommand or a `move-bookmark` flag settled at the rung.

##### feat: commit --closing --land lands the cycle

Land is typed by hand after the closing, so `--land` makes the closing and lands it in one
invocation: it refuses a moved trunk, a conflict, or a revision that is not a bookmark, then
fast-forwards `REV`, pushes it with the agent journal as its partner, and deletes the topic
bookmark, for both cycle shapes, told apart by the branch's node count.

##### feat: vc-x1 reshapes a draft bookmark

Reshaping a cycle's bookmark while it is a draft is an amend and a force-push typed by hand, so
`vc-x1` does both, refusing a bookmark already landed or one another branch is stacked on.

##### feat: vc-x1 re-describes a pair and keeps its trailers

A coordinated re-describe, as a single-step cycle turning multi-step needs, is a `jj describe` in
each repo that must keep the `ochid:` trailer, so `vc-x1` re-describes both commits of a pair at
once and keeps their trailers.

##### feat: vc-x1 runs a local ladder

A local ladder is `jj commit` for each intermediate and a hand collapse into the rung, so `vc-x1`
makes the intermediates and collapses them, validating each with `validate --fast`.

##### feat: vc-x1 sets a rung's edits aside

A rung inserted while the working copy holds the next rung's edits sets them aside as a patch by
hand and restores them after its push, so `vc-x1` sets them aside and brings them back.

##### feat: vc-x1 closing and land closing

Closing out the cycle.

# References

[0]: #feat-vc-x1-closing-and-land-opening
[1]: #feat-commit---opening-makes-the-opening-and-its-bookmark
[2]: #feat-commit---closing-closes-every-cycle-shape
[3]: #feat-commit---closing-takes-a-shape-from-the-flag-or-the-config
[4]: #feat-the-closing-can-be-taken-back
[5]: #feat-vc-x1-deletes-a-bookmark
[6]: #feat-commit---closing---land-lands-the-cycle
[7]: #feat-vc-x1-reshapes-a-draft-bookmark
[8]: #feat-vc-x1-re-describes-a-pair-and-keeps-its-trailers
[9]: #feat-vc-x1-runs-a-local-ladder
[10]: #feat-vc-x1-sets-a-rungs-edits-aside
[11]: #feat-vc-x1-closing-and-land-closing
