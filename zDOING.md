# Doing: the cycle record

What is being done and what was just done: the running cycle's planning and its work's
documentation, and the last finished cycle's. What we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
plan, Solution and Acceptance check, then the Ladder and its details, and the Deliberation last.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## In Progress

### feat: sync is safe by default

(wink, 2026-10-09) A `vc-x1 sync` fetches and then acts: a `main` that is behind is fast-forwarded,
one that diverged is rebased, and `@` is repositioned. An acquaint and a cycle start want to know a
repo is level without anything of theirs being put at risk, which today takes a `jj git fetch` and
a look.

- A sync is dangerous enough that its default guarantees, as best it can, that it messes nothing
  up, and that a neophyte can put things back the way they were before it.
- The agent runs it mid-conversation, and the user runs it with no agent to help, so the routine
  command neither rewinds a working copy nor leaves a state that needs interpreting.
- The parked agent-files v0.2.8 follows it, naming the look in its two steps.

#### Solution

A sync fetches a repo only when the repo holds no local work for the fetch to collide with:
uncommitted changes in `@`, an `@-` that is not on the remote, or a bookmark with commits the
remote does not have. A repo that holds some is not fetched, and sync says what it found, syncs
the other repos, and fails naming it. `--rebase` is the go to sync such a repo anyway. With
`--dry-run` sync asks the remote where its bookmark is and reports, fetching and changing nothing,
and `--undo` takes back a sync that went ahead and is regretted.

#### Acceptance check

- In a scratch clone that is behind its remote and holds no local work, `vc-x1 sync`
  fast-forwards. Passed at the first rung.
- In a scratch clone with an unpushed commit on `main`, `vc-x1 sync` exits non-zero naming the
  repo and does not fetch, `main@origin` staying where it was, and `vc-x1 sync --rebase` rebases
  it. Passed at the first rung.
- In a scratch clone whose remote rewrote a commit, `vc-x1 sync` follows the rewrite when the repo
  holds no local work, and with an edited file in `@` it does not fetch and leaves the file
  untouched. Passed at the first rung.
- In the same scratch clones, `vc-x1 sync --dry-run` reports behind, diverged, and up to date
  correctly, and no operation is added to any repo's `jj op log` by it.
- After a `vc-x1 sync --rebase` that rebased, `vc-x1 sync --undo` puts the repo back, and it
  refuses when a commit was made since.
- In this workspace, `vc-x1 sync --scope=both` reports the agent-repo up to date and holds the
  work-repo back for its uncommitted changes. Passed at the first rung, exit 1.
- `vc-x1 validate` passes. Passed at the first rung.

#### Ladder

- [feat: sync is safe by default opening][1] (done)
- [feat: sync fetches only a repo with no local work][2] (done)
- [feat: sync --dry-run asks the remote and changes nothing][3]
- [feat: sync --undo takes back the last sync][4]
- [feat: sync is safe by default closing][5]

##### feat: sync is safe by default opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, write
this block, bump the version-of-record, and rename the artifact to its dev name. It is also where
the trial begins: this file is created, and `TODO.md > ## In Progress` points at it.

##### feat: sync fetches only a repo with no local work

A sync fetches whatever it is pointed at, and a fetch is where the damage is done: it moves a
bookmark that is behind, conflicts one that diverged, and rebases `@` onto a rewritten remote.

- The gate is wink's: `@` empty, `@-` a commit on the remote, and every bookmark fast-forward
  only. Each names one way local work meets a fetch, and with none of them every commit a fetch
  can touch is the remote's own, so nothing can be lost and the way back is the old commit.
  - The agent-repo's `@` is not asked to be empty: it holds the running session's writes and
    never is. What that leaves open is a remote rewrite that touched the same session file, and
    there the sync stops with the state in place and rewinds nothing.
  - Local-only commits no bookmark names are left out. The work-repo holds three, the heads of
    bookmarks deleted this morning, and a check that counted them would hold the repo back for
    good over commits no sync moves.
- A held repo is not fetched at all, so there is nothing to restore: its remote-tracking refs are
  where they were, and the report lists what was found. The other repos still sync, and the run
  exits non-zero.
- `--rebase` is the go, widened from "rebase a non-empty `@` without asking" to "sync a repo that
  holds local work". The prompt on a terminal goes with it: the flag is the one way to say yes.
- The state is classified from the bookmark as it was before the fetch, since after it a repo
  that was behind reads as up to date.
- A fifth state, rewritten: the remote's commit replaced the local one and the fetch followed it,
  told from diverged by the bookmark already being on the remote's commit.
- An ahead bookmark is reported, where it was folded into "up to date, nothing to sync".
- What a held repo's report does not say is whether the remote moved too, sync not having asked
  it. Asking a remote for its head without fetching would add that, and is left for later.

##### feat: sync --dry-run asks the remote and changes nothing

A held repo's report cannot say whether the remote moved too, and there is no way to learn what a
sync would do without running one. The look has to change nothing at all, a look that fetches and
restores being the design this cycle backed out of.

- The remote is asked, not fetched: `git ls-remote <remote> refs/heads/<bookmark>` returns the
  commit the remote's bookmark is on, downloading and writing nothing. Tried here against
  `origin`, it returned the ids jj has recorded for `main` and for the parked bookmark.
  - It is run with the git executable jj's own fetch uses, `git.executable-path`, so it takes the
    same credentials and config.
  - The other way, gix's network client, is not among the features jj-lib builds gix with, and
    would bring its own transport and credential handling.
- The report is the gate's findings and one line of state per repo, from three commits: the local
  bookmark, the remote-tracking ref as last fetched, and the remote's answer.
  - The same commit: up to date. The remote unchanged since the last fetch: ahead, when there are
    local commits.
  - The remote moved and the bookmark holds no local commits: behind, a sync would follow the
    remote. Whether it advanced or rewrote cannot be told without its commits.
  - The remote moved and the bookmark holds local commits: diverged, a sync needs `--rebase`.
- No jj operation is made, not even a snapshot, so the look is safe in the agent-repo under a
  running session.
- It exits 0 whatever it found, a look that worked not being a failure.

##### feat: sync --undo takes back the last sync

A sync that went ahead on the go and is regretted, or that stopped part-way on a conflict, has no
way back but `jj op restore` and an operation id, and that restore leaves jj unable to see the
remote's change at the next fetch. Wink wants the way back, and its design is agreed here before
it is written:

- The restore itself is drafted and tested: it puts back the view and git's own remote-tracking
  refs, so the next fetch does all of its work again.
- Finding the last sync: to be settled, how a sync marks its operations so `--undo` knows where
  the last run began.
- Refusing: to be settled. The starting point is to refuse when an operation after the sync
  changed a bookmark or a commit, and to tolerate plain working-copy snapshots, which a live
  session makes constantly.
- Work in `@` now: carried forward onto the restored state, never rewound, a restore being what
  could lose a conversation's lines from the agent-repo.
- Scope: the repo set `-R` and `--scope` name, as a sync takes them.

##### feat: sync is safe by default closing

Closing out the cycle.

#### Deliberation

- The record is in this file: a trial, on wink's word, of the planning and the work's
  documentation living apart from the backlog.
  - It bends [Cycle-record](AGENTS.md#cycle-record), which names `TODO.md > ## In Progress`. The
    bend covers where this cycle's block is written and its order, and nothing else: every other
    step of the protocol runs as the agent-files say.
  - The block opens with the Todo as picked, then the plan, the Ladder and its details, and the
    Deliberation last, the shape wink described: pick the Todo, move it here, and everything
    about the cycle is written under it.
  - `## Continuation notes` stays in `TODO.md` for now, since [Acquaint](AGENTS.md#acquaint)
    reads it there, and `TODO.md` keeps its `## In Progress` and `## Closed` headings, each
    pointing here.
  - `## Next` is a section the agreed shape did not have: wink expected the `zDOING.md` entry in
    this file, and a parked cycle and a planned one are both things being done. It holds those two
    while three things are in flight and is not part of the shape.
- This cycle runs before agent-files v0.2.8, which is parked on its bookmark: its rule, "every
  cycle start syncs", is to name a sync that is safe to run unasked, and a rule written against
  the sync as it was would be rewritten a cycle later.
- The cycle grew twice before its first push, the shape still being free.
  - It opened as one commit, "feat: sync can look only, and asks on diverged", on a bookmark of
    that name, deleted unused.
  - A trial in scratch repos found the rewritten-remote case below, wink called the acting sync
    dangerous, and the default became "safe".
  - The `--undo` came in as a rung, wink's call over a cycle of its own: the cycle is complete
    when a regretted sync can be taken back, and nothing had been pushed.
  - The `--dry-run` came back as a rung of its own once it could be a look that fetches nothing.
  - Five commits. The plan was four with no separate opening, the first work rung carrying the
    setup, and wink had the opening inserted before the first push: the bookend pair exists, the
    cycle lands as an ordinary trapezoid, and the first work rung's diff is its work alone.
- A fetch is not a look, found by trial in scratch repos.
  - A tracked `main` that is behind is moved to the remote's, and one that diverged is left
    conflicted.
  - When the remote rewrote a commit the clone has, as a `squash-push` or an amended cycle
    bookmark does from another clone, the fetch rebases `@` onto the rewrite, and an edit in `@`
    to the same lines becomes conflict markers in the file on disk.
- The gate replaced three designs that each tried to undo a fetch after making it.
  - A `--dry-run` that put the bookmark back with a `bookmark set` missed the rewritten case: `@`
    had moved and the file held markers.
  - A restore to the pre-fetch operation that kept the remote-tracking refs left jj reading a
    rewritten remote as diverged from its own replacement, jj having already made its one follow.
  - A restore that also reset git's remote-tracking refs worked, and was dropped as the default
    on two points of wink's: a restore resets files on disk, and in the agent-repo those files are
    the conversation, and the user runs sync with no agent beside them to read a failure.
  - Not fetching needs none of it. The check is three reads of the repo, and what it refuses is
    exactly the work a fetch could hurt.
- No dry-run default: the bare command acts only where acting loses nothing. `--dry-run` is
  still wanted, wink's call, for what the gate cannot say: what the remote holds, and what a sync
  would do. It asks the remote and never fetches.
- No marker bookmarks, wink's first thought for a way back: which commits a fetch will touch is
  not known before it runs, a bookmark pins a commit and not where the bookmarks, `@`, and the
  files were, and jj keeps every replaced commit in its operation log regardless.
- A held repo makes the run exit non-zero: the repo was not synced, and a `--quiet` caller has
  only the exit status to learn that from.
  - So an acquaint mid-cycle, the work-repo's `@` holding a rung's edits, reads a non-zero exit
    as the finding it already reports, uncommitted work.
  - It is a breaking change, and a patch, no minor having been named: a repo that is ahead, or
    that holds uncommitted changes, was synced before and is held back now.
- Stop-on-error stays as it was for the `--rebase` path: the state is left where the failing step
  stopped, with the operation ids printed.
- Two entries came along from the parked change so they reach `main`. The reworked `zDOING.md`
  entry is under `## Next` here, and [TODO.md is renamed
  zTODO.md](TODO.md#todomd-is-renamed-ztodomd) is in the backlog.
- The `## Waiting` entry's condition is unmet, `vc-x1 closed` not landed, so nothing promotes.

## Next

Cycles planned or parked behind the one in progress, in the order they run. An entry is moved here
from `TODO.md` when it stops being something we might do and becomes something we are doing.

### agent-files(proposal): v0.2.8

Parked 2026-10-09 on its bookmark, `agent-filesproposal-v028`, as one unfinished commit pushed for
safekeeping. Its block is in that commit's `TODO.md > ## Closed`: every cycle start syncs, and the
acquaint's repo check with it. It resumes when the cycle in progress lands, to name
`vc-x1 sync --scope=both --dry-run` for the look in both steps, and its `TODO.md` is reconciled
with this file then.

### The cycle-record moves out of TODO.md into zDOING.md

(wink, 2026-10-09) What is being done is a different thing from what is to be done, and one file
holds both: the live record at the top of `TODO.md`, the backlog in the middle, and `## Closed` at
the bottom. A new file at the work-repo root, `zDOING.md`, takes the record, and `TODO.md` keeps its
name, a small step to see how the doing and done notion reads.

- Shape: an `agent-files` proposal and single-step, wink's call. One commit creates the file, moves
  the sections, and re-points the agent-files that name the old place:
  [Cycle-record](AGENTS.md#cycle-record), [Todo format](agent-data/notes.md#todo-format), [The In
  Progress block](agent-data/notes.md#the-in-progress-block), and the specimen.
- Name: `zDOING.md` is wink's choice over `DOING.md`, `LADDER.md`, and `DOING_LADDER.md`.
  - The `z` puts it last in every listing. Under `ls` in `en_US.UTF-8`, wink's daily view, that is
    three entries below `TODO.md`, where `DOING.md` would sit ten above it.
  - The cost is in byte order, as git, jj, and GitHub sort, where `DOING.md` would sit three
    entries above `TODO.md` and `zDOING.md` is at the far end of the list from it.
- Sections that move: `## In Progress` and `## Closed`, each block whole, the planning and the
  work's documentation, so `TODO.md` is what we might do and nothing else (wink, 2026-10-09).
  `## Continuation notes` goes with them so the acquaint read is the whole of a short file.
- Tried by hand first: the cycle **feat: sync is safe by default** kept its
  record in `zDOING.md` while the agent-files still named `TODO.md`, and what the trial showed is
  in that block.
- The ladder's place in the block: ahead of the Deliberation, which goes last as the part read only
  in doubt, the Ladder still heading its rung subsections. The placement below the Deliberation
  came from a miscommunication at agent-files v0.2.6. Whether this cycle or one of its own makes the
  change is settled at the opening.
- Heading names are open: `## Done` for `## Closed` pairs with the file's name, but `(done)` is
  already the rung marker and `notes/done.md` the frozen history, and it asks `## Doing` of
  `## In Progress`, a term the agent-files use throughout.
- No `vc-x1` code reads the block. A fixture script, `support/fixtures/build-dr-1.py`, and a comment
  in `src/lookup/blame.rs` name the old place.
- The template payload's `TODO.md` skeleton is the maintainer's, and this project's diff from it is
  the proposal.

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening.

_None yet: this file's first cycle is the one in progress._

# References

[1]: #feat-sync-is-safe-by-default-opening
[2]: #feat-sync-fetches-only-a-repo-with-no-local-work
[3]: #feat-sync---dry-run-asks-the-remote-and-changes-nothing
[4]: #feat-sync---undo-takes-back-the-last-sync
[5]: #feat-sync-is-safe-by-default-closing
