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

- Two commands, split at the review, the Todo's own lean: the closing is made, reviewed like any
  rung, and pushed on its own go, and `land` is everything after the Land go.
  - The agent's shell cannot answer a prompt, so a review inside one command needs `--yes` on
    every agent run, and the editor path goes with it.
- The closing stays `commit --closing`, wink's call at the opening (2026-10-23), not a new
  `close-out-commit`: it exists, its tests exist, and `--closing` matches the bookend's
  " closing". The second command is `land`, the protocol's own word, and the cycle is retitled to
  match.
- `commit --closing` handles every cycle shape, each tested, wink's call at the opening
  (2026-10-23): no ladder, a single-step cycle; a ladder of one rung, the opening alone, which
  [Cycle shape](AGENTS.md#cycle-shape) says still closes as a ladder; two rungs, the only size
  tested today; and three or more.
  - The Todo's single-step line is mistaken: a single-step cycle's one commit is on its bookmark,
    not on `main`, since `main` advances only at Land. So the commit is made above `main`, and
    today's `has_ladder` would call that a ladder, which the closing rung settles.
  - The provisional single-step behavior: `--closing` with the work `@-` on `main` makes the one
    commit plain, under the bare cycle title, and says it is a single-step cycle's, rather than
    being refused. We think the title is the way to tell a one-rung ladder from a single-step
    commit already pushed, the opening's ending " opening", since both are one commit above
    `main`.
- The closing's undo is in this cycle, the Todo's lean, a flag or a sibling command settled at its
  rung. `land`'s undo, after its push, waits for a real mistake.
- Bookmark deletion is a rung of this cycle, not its own cycle, since only `land` waits on it.
- `land` lands two shapes and refuses anything else: a closing merge whose parents are `--to` and
  the ladder's tip, or a single-step cycle's one commit whose parent is `--to`.
- Multi-step, five rungs between the bookends' sides, the agent's proposal: the closing's shapes
  first since `land` reads them, then its undo, then deletion, then `land`, which uses all three.

#### Problem

Closing a cycle and landing it are five commands typed in order, sync, the closing commit, the
fast-forward, the push, and the install, then two `jj` lines to delete the bookmark, each a place to
skip the sync, move the wrong bookmark, or push before the review. `commit --closing` is tested on a
ladder of two rungs only and refuses a single-step cycle, and a closing made by mistake is undone by
hand in two repos.

#### Solution

Provisional. `commit --closing` closes every cycle shape and can be taken back in both repos at
once, `vc-x1` deletes a bookmark locally and remotely, and `vc-x1 land` runs what follows the Land
go: the sync, the fast-forward of the trunk to the closing, the push, the build, the smoke check, the
install, and the bookmark's deletion, printing what its defaults resolved to first and what undoes
each step after.

#### Acceptance check

`cargo test` passes with `commit --closing` tests for a single-step cycle, a ladder of one rung, of
two, and of three, a test of the closing's undo restoring both repos, and `land` tests landing a
single-step cycle and a ladder on fixtures and refusing a commit that is neither shape. This
cycle's own Land is run by the dev build's `vc-x1 land`, its report showing `main` and
`main@origin` at the closing, the bookmark gone locally and remotely, and the installed `vc-x1 -V`
a bare 0.85.8.

#### Ladder

- [feat: vc-x1 closing and land opening][0] (done)
- [feat: commit --closing closes every cycle shape][1]
- [feat: the closing can be taken back][2]
- [feat: vc-x1 deletes a bookmark][3]
- [feat: vc-x1 land lands a closed cycle][4]
- [feat: vc-x1 closing and land closing][5]

##### feat: vc-x1 closing and land opening

The cycle's setup commit: create and publish the bookmark, move the Todo entry into this block,
and bump the version-of-record.

##### feat: commit --closing closes every cycle shape

`commit --closing` is tested on a ladder of two rungs and refuses a single-step cycle, so it makes
the single-step commit and closes a ladder of one, two, and three or more rungs, each tested.

##### feat: the closing can be taken back

A closing made by mistake is undone by hand in two repos, so one invocation takes it back, the
work side by its operation and the agent side's content moved back into `@`.

##### feat: vc-x1 deletes a bookmark

Deleting a cycle's bookmark is two `jj` lines, so `vc-x1` deletes one locally and remotely, a
subcommand or a `move-bookmark` flag settled at the rung.

##### feat: vc-x1 land lands a closed cycle

Land is typed by hand, so `vc-x1 land` checks the shape, syncs, fast-forwards the trunk, pushes,
builds, smoke-checks, installs, and deletes the bookmark, for both cycle shapes.

##### feat: vc-x1 closing and land closing

Closing out the cycle.

# References

[0]: #feat-vc-x1-closing-and-land-opening
[1]: #feat-commit---closing-closes-every-cycle-shape
[2]: #feat-the-closing-can-be-taken-back
[3]: #feat-vc-x1-deletes-a-bookmark
[4]: #feat-vc-x1-land-lands-a-closed-cycle
[5]: #feat-vc-x1-closing-and-land-closing
