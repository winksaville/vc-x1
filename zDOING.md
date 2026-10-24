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
  - Cut at the closing rung's review, wink's call (2026-10-23), the ladder having grown to sixteen
    rungs: this cycle keeps the path from opening to Land with no `jj` typed, its title's subject,
    and the repairs and alternative paths, the closing's shape, its undo, the reshape, `reparent`,
    the re-describe, local ladders, and the set-aside, moved with their deliberation to the Todo
    **feat: vc-x1 repairs and reshapes without jj**.
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
  - Refined at the closing rung's review, wink's call (2026-10-23): a single-step closing has no
    parent, so it takes no `REV`, and a multi-step one needs a parent, so it takes one. A bare
    `--closing` finalizes `@-` in place, which must be an opening unless `--title` is given, and
    a rung there is refused, asking for `--closing <REV>`. `--closing <REV>` makes the closing
    commit, `REV` any one node as its first parent, and with only the opening above `REV` it is
    refused, pointing at the bare form, unless `--dead-end`, whose closing keeps the opening as it
    was. The count validates what was typed rather than choosing the shape.
  - `commit --closing` adds the cycle's last commit and is no repair tool. A parent wrong after the
    fact is fixed on the committed node with `reparent`, its own rung.
  - At `--land`, one node that is not a merge is a single-step cycle, fast-forwarded onto, and a tip
    that merges `REV` and the branch is a ladder. Anything else is
    refused, so no title is read to tell the shapes apart.
- The opening anchors every cycle and `--closing` derives the rest from the branch, wink's call
  (2026-10-23), superseding `--opening` with `--closing` on one commit:
  - A single-step cycle starts as `commit --opening <BOOKMARK>`, the work squashed into it, and
    ends as `commit --closing --land main`, with no `--title` or `--body`, since the opening
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
    finalization `@` holds, the merge.
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
  - The bookmark's name defaults to the bare title's slug, the rules' own name for it, wink's call
    (2026-10-23), a rung inserted after a live test left it off.
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
- `commit` pushes what it commits, wink's model at the closing rung (2026-10-23): closing a cycle
  means its local and remote topic bookmarks both end on the closing, and every commit pushed at
  once is durable and is what the cycles have done all along, so pushing is the default, one rule.
  - The push is today's `push -s b` run after the commit in the same invocation: the agent half
    committed or folded as the partner, an empty one included, and the topic bookmark set and
    pushed. With `--land` the topic push is skipped, `main` moving instead.
  - `--no-push` keeps one commit local and `--push` forces the push, over `[commit] push`, `true`
    or `false`, `true` when absent, so a project or a person choosing the local pause sets it
    once. The flag, then the key, then `true`, printed as every default is.
  - The pause v0.3.1 put between the commit and its push, a look at the commit itself, goes by
    default, the argument `--land` accepted: the diff is reviewed at step 6 and the description at
    step 8, and the commit adds only the trailer and a closing's parents. The per-push approval
    moves onto the commit, the go at step 8 covering the commit and its push, steps 8 and 9 one.
    A `--no-push` commit keeps the pause, so the rules name both, proposal material for v0.3.1's
    per-rung flow.
- `push <bookmark>` publishes a bookmark whose commit is committed, the working copy being the one
  node it refuses, pointing at `commit`, wink's call (2026-10-23).
  - A committed node is pushable whether or not jj calls it immutable, so a `--no-push` commit is
    published later by `push`, and a pair's work half still has its agent partner completed as
    today. The docs and errors say "committed", since jj's "immutable", a commit it refuses to
    rewrite, is narrower.
- `commit -s a`, wink's call (2026-10-23): the agent repo is committed alone when asked, an
  unpaired commit with the given title and body and no trailer, `--push` publishing its `main`.
  - The refusal it replaces guarded a paired agent commit made before its work half. An unpaired
    one names no partner, so nothing mismatches. An agent `@` that is a pending partner keeps its
    change id when committed, so the work commit's trailer still resolves, and a warning says the
    back-trailer is missing.
  - The principle, wink's (2026-10-23): do what the user asks unless it is dangerous, and warn
    rather than refuse, so a fix-up such as a repaired change id stays inside `vc-x1`.
- A waiver, wink's (2026-10-23): "Permission granted to complete this cycle up to but not including
  the closing." It covers rungs `[3]` through `[7]`, their work reviews, description reviews, and
  per-push approvals, so each rung is validated, described, committed, and pushed without a stop.
  It does not cover the closing, `[8]`, or Land, which take their reviews and goes as usual, and the
  live testing comes before the closing.
- Bookmark deletion is a rung of this cycle, not its own cycle, since `--land` and `--skip` use it.
- The closing's shape stays the trapezoid in this cycle, `--shape` and the linear closing moving to
  the Todo with the rest, though the v0.3.1 proposal already names them.
- Multi-step, seven work rungs between the bookends after the cut, in the order each is used by
  the next: the opening and its bookmark, the closing, the push, the record move, the deletion, and
  `--land`.

#### Problem

A cycle's multi-step operations are typed as raw `jj` commands in order, and each is a place to
make a mistake: the opening's `jj git push --named` creating and publishing the bookmark, the
closing, and Land's sync, fast-forward, push, and two `jj` lines deleting the bookmark, any of them
able to skip the sync, move the wrong bookmark, or push before the review. The goal, wink's
(2026-10-23), is that no cycle step needs `jj` typed by hand for these operations. `commit
--closing` assumed `main`, was tested on a ladder of two rungs only, and refused a single-step
cycle.

#### Solution

Provisional. Each step from opening to Land becomes one `vc-x1` invocation that checks before it
acts and prints what it resolved:

- `commit --opening [BOOKMARK]` creates and pushes the bookmark at `@-` and makes the opening,
  appending " opening" to the bare title, and every commit advances the line's bookmark locally.
- `commit` pushes what it commits by default, `--no-push` or `[commit] push = false` keeping it
  local, and `push <bookmark>` publishes a bookmark on a committed node.
- A bare `commit --closing` finalizes a single-step cycle's one commit in place, the bare title
  replacing the opening's. `commit --closing <REV>` makes a multi-step cycle's closing commit, the
  merge of `REV` and the last rung, titled " closing", or " dead-end closing" with `--dead-end`,
  its body from `[closing] body`. With `[record]` set, the record moves from the doing file to the
  closed file.
- `commit --closing --land main` also lands it: the fast-forward of the trunk, the push, and the
  bookmark's deletion, refused when the trunk moved or the merge conflicts.
- `vc-x1` deletes a bookmark locally and remotely, `--skip` keeping either side.

#### Acceptance check

`cargo test` passes with tests for:

- `commit --opening` creating and pushing the bookmark, each `--skip`, the title suffix, the
  bookmark named from the title, and the line's bookmark advancing.
- `commit --closing` finalizing a single-step cycle's commit in place, making the closing commit on
  ladders of two and three, refusing a `REV` with only the opening above it and a bare form on a
  rung, the title and body derived, and the record moved with `[record]` set and left without.
- `commit` pushing by default, `--no-push` and `[commit] push` keeping it local, `-s a` committing
  the agent repo alone, and `push <bookmark>` publishing a committed node and refusing the working
  copy.
- Bookmark deletion locally and remotely, and `--land` landing a single-step cycle and a ladder on
  fixtures and refusing a moved trunk, a conflict, and a revision that is not a bookmark.

This cycle's own Land is run by the dev build's `commit --closing main --land`, its report showing
`main` and `main@origin` at the closing and the bookmark gone locally and remotely, and the
installed `vc-x1 -V` a bare 0.85.8.

#### Ladder

- [feat: vc-x1 closing and land opening][0] (done)
- [feat: commit --opening makes the opening and its bookmark][1] (done)
- [feat: commit --opening names the bookmark from the title][2] (done)
- [feat: commit --closing closes every cycle shape][3] (done)
- [feat: commit pushes what it commits][4] (done)
- [feat: commit --closing moves the record][5]
- [feat: vc-x1 deletes a bookmark][6]
- [feat: commit --closing --land lands the cycle][7]
- [feat: vc-x1 closing and land closing][8]

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

##### feat: commit --opening names the bookmark from the title

`--opening` takes the bookmark's name as a value, though the rules name it by the cycle title's
slug, so the value becomes optional and a bookmark left unnamed is the slug of the bare `--title`.

* The bookmark's name was typed a second time, though the title determines it, found when wink
  left it off in a live test (2026-10-23).
  - `--opening [BOOKMARK]` takes the name or none, and none is the bare title's slug by
    `validate_anchors::anchor_slug`, the algorithm [Cycle
    bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land) names, printed since it is a
    default. A `--title` right after `--opening` is the next flag, not its value.
  - A title with nothing to slug is refused, asking for the name.

##### feat: commit --closing closes every cycle shape

`commit --closing` assumed `main`, was tested on a ladder of two rungs only, and refused a
single-step cycle, so it takes the revision it closes onto, tells the shapes apart by the branch's
commit count, and reads the title and body from the opening and the config.

* `commit` named `main` as the closing's first parent and asked whether `@-` was in it, so it
  assumed the branch a cycle closes onto.
  - `--closing <REV>` names it, any one commit, resolved once to a commit id before anything moves
    and refused when it names none or several. A bookmark is read where it is now, so a target that
    moved during the cycle is still joined by the merge.
* The shape is what was typed, a single-step closing naming no parent and a multi-step one naming
  its `REV`, the count of commits in `REV..@-` checking it, wink's call at the rung's review.
  - A bare `--closing` is a single-step cycle: `@-`, which must be an opening unless `--title` is
    given, is finalized in place, no new commit. The work `@` is
    folded into it and it is re-described under the bare cycle title, its body and `ochid:`
    trailer kept, so its change id and partner stand. The agent `@` is left for push, since the
    commit's partner is the opening's, and the line's bookmark follows the rewrite.
  - `--closing <REV>` is a multi-step cycle's closing commit, the merge of `REV` and the last rung,
    the line's bookmark advanced to it as with any commit.
  - `--closing <REV>` with none above it is refused, a cycle starting with `--opening`, and with the
    opening alone, pointing at the bare form. `--dead-end` makes a closing commit even then, since a
    dead end keeps the opening as it was, and so needs a `REV`.
* A bookend's title was typed by hand, its suffix included.
  - `--title` and `--body` are optional with `--closing`. The cycle title is read from the
    opening, the branch's one first commit, " opening" removed, refused with "pass --title" when
    there is no such commit or it is not an opening. A given title is the bare cycle title, and one
    carrying " opening" or " closing" already is refused, for `--opening` too.
  - " closing" is appended for the closing commit, " dead-end closing" with `--dead-end`.
  - The closing commit's body defaults to `[closing] body`, a new key, its `{title}` the cycle
    title, this workspace's `Closes the cycle "{title}". See details in zCLOSED.md.`, and with
    neither `--body` is required. A finalized commit keeps the opening's body unless one is given.
  - Without `--closing` a title ending " closing" is warned about, never refused, a title like
    "fix: the door stops closing" being just a title.
* The v0.3.1 proposal gains **A cycle's shape is its branch's commit count**, retiring Cycle
  shape's "a ladder that shrinks to one rung still closes as a ladder".
* The record move, `[record] doing` and `closed`, split out at this rung into the next, so the
  shapes could be tried on a live repo sooner, wink's leave (2026-10-23).
* The set-aside node's code was restacked onto the slug rung, its `commit.rs` taken from the slug
  rung and the closing re-applied on top, since most of it was superseded by the commit count.

##### feat: commit pushes what it commits

A commit is published by a second command after it, though every rung and closing has been pushed
as soon as it was made, so `commit` pushes what it committed in the same invocation by default,
`--no-push` or `[commit] push = false` keeping it local, `push <bookmark>` publishes a bookmark on
a committed node, and `-s a` commits the agent repo alone.

* A commit was published by a second command after it.
  - `commit` runs `push` after the commit in the same invocation, `-s b` for a pair whatever the
    commit's scope, so a `-s w` commit's agent half is completed as its partner, `-s p` for a plain
    repo, and the bookmark advanced is the one pushed, or the line's when none was. A dry run shows
    the push's dry run too.
  - `--no-push` keeps one commit local, `--push` pushes over `[commit] push = false`, a new key,
    true when absent, the flag, then the key, then true. A `--no-advance` commit pushes only with
    `--push`, since a push would move the bookmark it left.
  - A single-step finalize pushes too, the bookmark following the rewrite.
* `commit -s a` was refused.
  - It commits the agent `@` alone with the title and body and no trailer, and when it pushes, sets
    the agent `main` to it and publishes it. A pending partner is committed with a warning, its
    change id kept so the work commit's trailer still resolves. A bookend is refused there, being
    the work repo's.
* `push <bookmark>` refused a bookmark off the line.
  - A named bookmark off the work `@-`'s line, on a commit already made, is published where it is,
    a name and no pair. One on the working copy is refused, pointing at `commit`, and one on the
    root has nothing to publish. A bookmark on the line goes the pair's way, as before.
* The tests' `args` helper passes `--no-push`, so a test sees the commit and pushes only when it
  asks, and the hand-built params carry `push: Some(false)`.

##### feat: commit --closing moves the record

A closing's record is moved from `zDOING.md` into `zCLOSED.md` by hand, so `--closing` moves the In
Progress block and its references into the closed file's `## Closed` and resets `## In Progress`,
when `[record] doing` and `closed` name the two files, and a finalized commit's body gains the
pointer.

##### feat: vc-x1 deletes a bookmark

Deleting a cycle's bookmark is two `jj` lines, so `vc-x1` deletes one locally and remotely,
`--skip` keeping either side, a subcommand or a `move-bookmark` flag settled at the rung.

##### feat: commit --closing --land lands the cycle

Land is typed by hand after the closing, so `--land` makes the closing and lands it in one
invocation: it refuses a moved trunk, a conflict, or a revision that is not a bookmark, then
fast-forwards `REV`, pushes it with the agent journal as its partner, and deletes the topic
bookmark, for both cycle shapes, told apart by the branch's node count.

##### feat: vc-x1 closing and land closing

Closing out the cycle.

# References

[0]: #feat-vc-x1-closing-and-land-opening
[1]: #feat-commit---opening-makes-the-opening-and-its-bookmark
[2]: #feat-commit---opening-names-the-bookmark-from-the-title
[3]: #feat-commit---closing-closes-every-cycle-shape
[4]: #feat-commit-pushes-what-it-commits
[5]: #feat-commit---closing-moves-the-record
[6]: #feat-vc-x1-deletes-a-bookmark
[7]: #feat-commit---closing---land-lands-the-cycle
[8]: #feat-vc-x1-closing-and-land-closing
