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

### feat: push, commit, and squash-push take flags only

Picked from `## Todo` as **push takes a revision, and the remote's name for it** (wink + agent,
2026-10-24), widened at its planning to every argument a flag (wink + agent, 2026-10-27), and split
in two: this cycle the three commands that publish, the rest **feat: the other commands take flags
only**.

#### Problem

`push`'s positional is a bookmark, so `push -s b @-` is refused, though git needs only a name on
the remote: `git push origin HEAD:refs/heads/<name>` publishes any commit under a new name, and `jj
git push --named <name>=<REV>` does the same and creates the local bookmark that tracks it. Filed
from the live testing of **feat: vc-x1 closing and land**, whose `[9]` made the refusal say to
leave the revision out.

A positional means something different from command to command: a bookmark to `push` and
`squash-push`, a revision to `commit` and the read commands. So `push -s b feature` has two
readings, the bookmark to move or the commit it is at, and `push` takes its bookmark both as the
positional and as `--bookmark`, one value under two spellings.

#### Solution

- Every argument is a flag, the command names aside, and this cycle does `push`, `commit`, and
  `squash-push`.
- `push -s b|w|a|p [-r REV] [--bookmark NAME]`:
  - REV is the commit to publish, `@-` by default. At `@-` it is the pair's path, unchanged.
    Elsewhere a name alone is published, no pair, today's off-line case, the working copy and the
    root refused.
  - NAME is the remote's name for it, the bookmark at or nearest below REV by default. With no
    bookmark there NAME is required, created at REV and pushed tracked, as `--named` does.
  - A REV that resolves to more than one commit is refused with the count.
- `commit -r REV`, `@` by default, its `--bookmark` unchanged.
- `squash-push --bookmark NAME`.
- An old positional is kept hidden and refused with the flag that replaces it: `push -s b @-`
  names `-r @-`, and `push -s b feature` with `feature` a bookmark names `--bookmark feature`.
- The README's sections for the three, and the agent-files' invocations of them.

#### Acceptance check

`cargo test` passes with a test per rule: `push -r` publishes the commit named, `push --bookmark`
with a new name creates it tracked on origin, a REV of several commits is refused, and each
command's old positional is refused naming its flag. Live, from the rung after push's on: `vc-x1-dev
push -s b @-` fails naming `-r @-`, and a rung committed with `--no-push` is published by
`vc-x1-dev push -s b -r @-`.

#### Deliberation

- Flags only, the command names aside, wink's call (2026-10-27).
  - A positional's meaning differs by command, so the same word reads two ways, and a flag has one
    reading on every command, the reason [Scope is a flag on every command, and push resolves its
    own bookmarks](TODO.md#scope-is-a-flag-on-every-command-and-push-resolves-its-own-bookmarks)
    gave for `-s`, and `land`'s for naming `--source` and `--onto`.
  - jj splits the two the same way: a bookmark name is a revision symbol, and a command that moves
    a bookmark takes its name apart from the revision, `jj bookmark set NAME -r REV` and `jj git
    push --named NAME=REV`.
  - The command line is what the user approves, and a flag says what each word is.
  - `run` keeps its trailing arguments, which are the built binary's.
- `-r, --revision` and `--bookmark`: the names the read commands, `lookup`, and `commit` already
  use, and jj's.
- An old positional is refused, not dropped, wink's call (2026-10-27): a hidden argument whose error
  names the flag, so a stale doc or a habit is told its own fix.
- Two cycles, wink's call (2026-10-27): this one the three commands that publish, which every rung
  runs and the push Todo is about, and the rest in **feat: the other commands take flags only**,
  with **build, install, and run take --dev** folded into it, which keeps `install-dev`, run at
  every rung here, unchanged until this cycle lands, and **init and clone take --scope por**,
  whose commands it touches.
- A REV other than `@-` publishes a name alone, provisional: the pair's checks, the trailer, the
  partner, and the restamp, assume `@-`, and a pair's commit below `@-` was published by its own
  rung.
- The README states the rule in the other cycle, once it holds for every command.
- Waiver, wink's "Continue and complete this cycle except for landing on main" (2026-10-27), after
  the opening's push.
  - Covers every rung from **feat: push takes --revision and --bookmark** through the closing: the
    work review, the description review, the per-push approval, and the hard stop after each push.
  - Does not cover Land: the closing is pushed without `--land`, and the close-out shape is the
    default, trapezoid, until wink chooses at the Land.
  - Validation still runs before every push.

#### Ladder

- [feat: push, commit, and squash-push take flags only opening][f0] (done)
- [feat: push takes --revision and --bookmark][f1] (done)
- [feat: commit and squash-push take flags only][f2] (done)
- [docs: the agent-files name the flags][f3]
- [feat: push, commit, and squash-push take flags only closing][f4]

##### feat: push, commit, and squash-push take flags only opening

Opens the cycle: the Todo moved here and widened, the other commands' Todo filed, the version
bumped.

##### feat: push takes --revision and --bookmark

`push`'s positional goes hidden and refused, `-r` names the commit and `--bookmark` the remote's
name, a new name created tracked, and the refusal helper the other commands reuse.

- `options_flags::retired_positional::refusal`, the one wording every retired positional gets: the
  command, "the positional is retired, every argument is a flag", and the flag form with the value
  in place, so the next two rungs and the other cycle share it.
- `push`'s refusal names `--bookmark` when the value is a local bookmark in the repo `-R` names,
  else `-r`, the lookup made in `to_params` and a failed one only losing the hint.
- Two paths, split on whether REV is the work `@-` by commit id, so `-r @-`, `-r <its chid>`, and
  `-r <its bookmark>` all take the pair's:
  - The pair's path is unchanged but for `--bookmark` taking a new name, set at `@-` by the work
    half's own `bookmark_set` and tracked by its push, the call the opening's bookmark uses.
  - `publish_elsewhere`, which replaces `publish_name`, publishes a name alone: the one bookmark at
    REV, or `--bookmark`'s, at REV or new. A bookmark elsewhere is refused naming `move-bookmark`,
    since push moves a bookmark only along the pair's line.
- `-s p` resolves its bookmark the pair's way, so a named bookmark off the line is refused there too,
  where it was moved sideways before.
- `-r` with `-s a` is refused, and `--bookmark` given a revision names `-r`.
- The backlog's **vc-x1 push: rework the two bookmark parameters** retires, the two spellings gone.
- `tests/cli_sync.rs` called `push -s w main`, and the full validation found it, the one CLI caller
  of the old form.

##### feat: commit and squash-push take flags only

`commit`'s `[REV]` becomes `-r`, and `squash-push`'s `[BOOKMARK]` becomes `--bookmark`, each old
positional refused by the helper.

- `commit -r, --revision`: the field stays `rev`, so nothing past the parse changes, and its errors
  say `-r` where they said `REV`. The refusal always names `-r`, the positional having been a
  revision only.
- `squash-push --bookmark`: the one flag, the positional having had no flag form, and the refusal
  always names `--bookmark`.
- Each refusal is made where the command builds its params, `CommitParams::try_from` and
  `SquashPushArgs::to_params`, before any repo is touched.
- No CLI test and no code called either with its positional. The README's examples and tables did,
  and `support/fixtures/build-dr-1.py`'s `vc-x1 push {bookmark} --title` is left: it is a
  transcript line the builder synthesizes as history, never run.

##### docs: the agent-files name the flags

The invocations of the three in `AGENTS.md` and `agent-data/`, with the stale `vc-x1 push <bookmark>
--title` forms brought to what `commit` does now.

##### feat: push, commit, and squash-push take flags only closing

Closing out the cycle.

# References

[f0]: #feat-push-commit-and-squash-push-take-flags-only-opening
[f1]: #feat-push-takes---revision-and---bookmark
[f2]: #feat-commit-and-squash-push-take-flags-only
[f3]: #docs-the-agent-files-name-the-flags
[f4]: #feat-push-commit-and-squash-push-take-flags-only-closing
