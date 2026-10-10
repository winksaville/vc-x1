# Doing: the cycle record

What is being done and what was just done: the running cycle's planning and its work's
documentation, and the last finished cycle's. What we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial, with a pointer to it under the
same heading in `TODO.md`, where [Acquaint](AGENTS.md#acquaint) still looks.

Written 2026-10-10 at the end of a long session, and brought up to date 2026-10-11 when the same
session resumed.

- The cycle in flight is **feat: sync is safe by default**, on the bookmark
  `feat-sync-is-safe-by-default`. Four commits are pushed: the opening, "sync fetches only a repo
  with no local work", "sync --dry-run looks and changes nothing", and "sync tells a fast-forward
  from a rewrite". The closing is what remains.
- The `--undo` rung was dropped before any code and is a Todo entry. The working copy still has
  the manifest at `0.84.21-4`, the version that rung would have carried, and the closing takes the
  bare version.
- Parked behind this cycle: `agent-files(proposal): v0.2.8`, one unfinished commit pushed on
  `agent-filesproposal-v028` for safekeeping, the first entry of `## Todo` in `TODO.md`.
- Local only, for wink's `gitk`: the bookmark `0.84.21-1`, on the first work rung's commit.
- This cycle's commit bodies carry a legend line the rule does not have, wink's exception, from
  the second work rung on: ``Legend: `*` is a problem this commit addresses, `-` is a solution to
  it.`` The Deliberation below has the bend.
- How a push goes from a session with no terminal: `vc-x1 push <bookmark> --yes --title "..."
  --body "$(cat tmp/<file>)"`, the body written to a file under `tmp/` first.
- Installed binaries: `vc-x1` is the stable 0.84.20, and `vc-x1-dev` is this cycle's, rebuilt by
  every `vc-x1 validate`.
- Still to do, in no rung: search the jj tracker for a feature request for the remote query the
  `## Waiting` entry in `TODO.md` waits on, and draft one for wink to file if there is none.
- Left as found: three commits with no bookmark, `2fbd6c40`, `b34cb7b3`, and `7694fce7`, the
  heads of remote bookmarks deleted at this session's start.
- The block's order is wink's own edit: the Todo text, the Deliberation, the Solution, the
  Acceptance check, then the Ladder and its details. Leave it as it is. The line at the top of
  this file still says "the Deliberation last" and is stale.
  - Why: the Deliberation is discussed before starting and read oddly at the end. Its cost is
    the Ladder far down the file.
  - The Ladder's list early with its details after the Deliberation was weighed and left: the
    list is then far from the details, and in zed a link works in preview mode alone and "back"
    does not return to the list, so links are no use while working.
  - The order is the cycle-record entry's to settle, in `## Todo` in `TODO.md`.

## In Progress

_No cycle currently in progress._

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening.

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
  - This file holds one cycle, the one being worked on, and what comes next is in `TODO.md`,
    wink's rule. A `## Next` section was tried here for a day, holding the parked cycle and the
    planned one, and went back: planned work in two files was easy to lose track of.
- This cycle runs before agent-files v0.2.8, which is parked on its bookmark: its rule, "every
  cycle start syncs", is to name a sync that is safe to run unasked, and a rule written against
  the sync as it was would be rewritten a cycle later.
- The cycle grew twice before its first push, the shape still being free.
  - It opened as one commit, "feat: sync can look only, and asks on diverged", on a bookmark of
    that name, deleted unused.
  - A trial in scratch repos found the rewritten-remote case below, wink called the acting sync
    dangerous, and the default became "safe".
  - The `--undo` came in as a rung, wink's call over a cycle of its own, and went out again the
    next morning before any code, wink's call too: the cycle had grown long and tangled, the
    routine sync was safe without it, and its design held more than was left to give it. The
    design and its complications are kept in the Todo entry [sync --undo takes back the last
    sync](TODO.md#sync---undo-takes-back-the-last-sync).
  - The `--dry-run` came back as a rung of its own once it could be a look that fetches nothing.
  - A rung came in during the second work rung, "sync tells a fast-forward from a rewrite", from
    wink's trial of the look in iiac-perf.
  - Two rung titles were shortened before their push, having been written over the 50-character
    cap: "sync --dry-run asks the remote and changes nothing" and the new rung's first wording.
  - Five commits at the first push, six since. The plan was four with no separate opening, the
    first work rung carrying the setup, and wink had the opening inserted before the first push:
    the bookend pair exists, the cycle lands as an ordinary trapezoid, and the first work rung's
    diff is its work alone.
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
- The commit of the second work rung carries a legend line the rule does not have,
  ``Legend: `*` is a problem this commit addresses, `-` is a solution to it.``, on wink's word.
  - It bends [Commit-body form](agent-data/prose.md#commit-body-form), for that body and the
    bodies of this cycle's later rungs. The rule and its specimen are unchanged, the bodies
    pushed before it keep their form, and the change to the rule is the Todo entry [A commit
    body carries a legend for its markers](TODO.md#a-commit-body-carries-a-legend-for-its-markers).
  - The commit had been pushed without it, the agent having misread the go, and was
    re-described in both repos, each `ochid:` trailer kept, and force-pushed.
- A Todo entry was rewritten at the third work rung, [Scope is a flag on every
  command](TODO.md#scope-is-a-flag-on-every-command-and-push-resolves-its-own-bookmarks): wink
  tried `sync both` after `status both`, found the two disagree, and chose the flag for every
  command over the positional the entry had first decided on. Nothing in this cycle changes for
  it, `sync` taking its scope by `--scope` already.
- Two entries were added to `TODO.md` at the second rung: under `## Waiting`, the end of the git
  spawn, waiting on jj-lib, and under `## Todo`, [jj-lib moves to
  0.46.0](TODO.md#jj-lib-moves-to-0460), which wink found published while the spawn was weighed.
  - An upstream feature request for the query is wanted and not yet looked for.
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
- Two entries came along from the parked change so they reach `main`, both in `TODO.md`: the
  reworked `zDOING.md` entry and [TODO.md is renamed
  zTODO.md](TODO.md#todomd-is-renamed-ztodomd).
- The `## Waiting` entry's condition is unmet, `vc-x1 closed` not landed, so nothing promotes.

#### Solution

Before it fetches, a sync finds out what the fetch would do. It asks the remote where each of its
bookmarks is, downloads the commits of those that moved with no ref moved, and tells a
fast-forward from a rewrite. A remote that has not moved means nothing to fetch, and one that
only fast-forwarded is fetched with the repo's local work left where it is. A repo is held back,
not fetched and named in a failing exit, in two cases: a bookmark that moved has local commits of
its own, or the remote rewrote a bookmark and the repo holds local work. `--force` syncs it
anyway, `--rebase` being the same flag. With `--dry-run` the same finding out is reported and
nothing is moved, each repo ending with a verdict, what a sync would do.

A way to take back a forced sync, `--undo`, was planned here and deferred to a Todo entry.

#### Acceptance check

Run at the closing with the closing's build, in scratch clones of one bare remote, eight cases,
each a `--dry-run` and then the sync, by `tmp/accept-sync.sh`.

- In a scratch clone that is behind its remote and holds no local work, `vc-x1 sync`
  fast-forwards. Passed.
- In a scratch clone with an unpushed commit on `main` whose remote also moved, `vc-x1 sync`
  exits non-zero naming the repo and does not fetch, `main@origin` staying where it was, and
  `vc-x1 sync --force` rebases it. Passed, the unforced run exiting 1.
- In a scratch clone whose remote rewrote a commit, `vc-x1 sync` follows the rewrite when the repo
  holds no local work, and with an edited file in `@` it does not fetch and leaves the file
  untouched. Passed, the second exiting 1.
- In a scratch clone holding an unpushed bookmark, or uncommitted changes, `vc-x1 sync` succeeds
  when the remote has not moved, and when it fast-forwarded it moves `main` and leaves the local
  work where it was. Passed.
- In the same scratch clones, `vc-x1 sync --dry-run` reports the state and a verdict that the
  sync run after it bears out, and moves no bookmark, `@`, or remote-tracking ref. Passed, eight
  verdicts of eight borne out.
- In this workspace, `vc-x1 sync --scope=both --dry-run` reports both repos up to date with
  nothing to do, the work-repo's uncommitted changes noted. Passed.
- `vc-x1 validate` passes. Passed.

One check the opening stated was withdrawn with its rung: that `vc-x1 sync --undo` puts a forced
sync back and refuses when a commit was made since.

#### Ladder

- [feat: sync is safe by default opening][1] (done)
- [feat: sync fetches only a repo with no local work][2] (done)
- [feat: sync --dry-run looks and changes nothing][3] (done)
- [feat: sync tells a fast-forward from a rewrite][4] (done)
- [feat: sync is safe by default closing][5] (done)

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

##### feat: sync --dry-run looks and changes nothing

A held repo's report cannot say whether the remote moved too, and there is no way to learn what a
sync would do without running one. The look has to change nothing at all, a look that fetches and
restores being the design this cycle backed out of.

- The remote is asked, not fetched: `git ls-remote <remote> refs/heads/<bookmark>` returns the
  commit the remote's bookmark is on, downloading and writing nothing.
- The call is a spawn of `git`, which the project bans, and wink reopened the spawn register for
  it as a sixth entry, temporary.
  - The first draft was written as if a spawn were ordinary here, the project spawning `jj -V`,
    `gh`, and an editor. Those are the register's named exceptions, and clippy stopped the build.
  - jj-lib is the reason the spawn is ours. It uses gix for a repo's local git store and spawns
    git itself for every network leg, keeps that layer `pub(crate)`, and builds gix with no
    network client, in 0.45.1 and in 0.46.0, read from the published source.
  - Turning on gix's network client was the other way to keep the ban whole, and was left: it
    would be the one path to a remote that is not jj-lib's, with its own transport and credential
    handling, so a look and the fetch after it could disagree.
  - The spawn runs the git executable jj's fetch runs, `git.executable-path`, and so takes the
    same config and credentials.
  - Three reminders make the exception findable when it can end: the register entry and the
    comment at the call, a `## Waiting` entry in `TODO.md`, and a note on the jj-lib line of
    `Cargo.toml` for whoever raises it.
- The state comes from three commits, the local bookmark, the remote's answer, and whether this
  repo has the remote's commit.
  - The repo has it: the two ancestry probes a sync makes, exact.
  - The repo does not: the remote moved to commits not yet fetched. A local bookmark that is on
    the remote holds nothing of its own, so a fetch would follow: behind. One that is not:
    diverged.
  - A remote that advanced and one that rewrote history read the same, behind, their commits
    not being here to tell apart. Only a fetch tells rewritten.
- The report is each repo's state, worded as what a sync would do, and the local work the gate
  would hold it back for.
- What it changes: nothing a sync cares about. The plan said "no jj operation, not even a
  snapshot", and that was too strong: "does `@` hold changes" is asked of the files, which takes
  the snapshot any jj command makes, and a repo with an edited file gains that one operation.
- It exits 0 whatever it found, a look that worked not being a failure.
- The report names the bookmark and ends each repo with a verdict, what a sync would do. Wink ran
  the first wording in iiac-perf, read "up-to-date" over an unpushed bookmark as "safe to sync",
  and it was not: the state is the synced bookmark's alone, and the gate would have held the
  repo back.

##### feat: sync tells a fast-forward from a rewrite

The gate runs before any fetch and holds back every repo with local work, the one whose remote
has not moved and the one whose remote only added commits included. One unpushed bookmark
anywhere in a repo then stops every sync of it, wink's case in iiac-perf, and the look's verdict
there read "held back" over a `main` it had just called up to date.

- Sync asks the remote where every one of its bookmarks is, a fetch bringing all of them, and
  compares with the last fetch. The remote where the last fetch left it means nothing to fetch:
  sync says so, notes the local work, and exits 0.
- When a bookmark moved, its commits are downloaded with no ref moved, and the move is classified
  by ancestry: a fast-forward, a rewrite, new, or deleted.
  - Tried in scratch repos, colocated and not: `git fetch --refmap= --no-tags
    --no-write-fetch-head <remote> <ref>` brought the remote's commit into the object store, left
    every git ref as it was, and jj saw nothing, no bookmark, `@`, or remote-tracking ref changed
    and no operation added, a later jj command included. `--refmap=` is what does it: without
    it git moves the remote-tracking ref as a side effect, and jj imports that as a fetch.
  - The ancestry walk is gix's, in-process: `rev_walk` is in the build as it is, where
    `merge_base` wants a feature jj-lib does not turn on. A revset cannot ask, jj not having
    imported the remote's commit.
  - It is a second form of the one spawn, under the same register entry, and both go through one
    function so the `#[allow]` is at one site.
- A fast-forward replaces nothing, so no local work can be moved by it: sync fetches, and says
  what it leaves untouched. Untouched is not brought up to date: an `@` with changes, or an
  unpushed bookmark, stays on the old commit, and moving it is the user's act.
- Two things still hold a repo back, each reported with its bookmark:
  - A bookmark that moved on the remote has local commits of its own, the fetch leaving it
    conflicted. This is any bookmark, not the synced one alone.
  - The remote rewrote or deleted a bookmark and the repo holds local work. Which commits the
    rewrite replaced is not worked out: any local work is taken as at risk, the check erring
    toward holding.
- The look and the sync share the finding out, `look_repos`, so the look's verdict and the sync's
  act cannot disagree. In the scratch scenarios each verdict was borne out by the sync run after
  it.
- A look now downloads, where the second rung's fetched nothing at all: the moved bookmarks'
  commits land in the git store with no ref naming them. They are the commits a fetch would
  bring.
- `--force` is the flag's documented name, wink's call, `--rebase` staying as an alias: past the
  gate there is often nothing to rebase, and "force a sync" is what the flag is reached for.
- Every sync now spawns git once to ask, and once more when the remote moved, where the look
  alone did.

##### feat: sync is safe by default closing

Closing out the cycle. Close-out shape: trapezoid.

- The cycle changed its design five times after it opened, and each change was found by trying
  the last one.
  - A dry run that put a bookmark back, a restore of the whole repo, a gate on local work alone,
    a look that asked the remote, and the gate that tells a fast-forward from a rewrite.
  - What moved it each time was a trial in scratch repos or wink running the built binary in
    another project, never the plan. The acceptance check written at the opening tested the
    first design and would have passed it.
- The plan could have been cheaper to change. The record was rewritten whole four times, and a
  rung's worth of code was written and discarded twice.
  - A trial of the mechanism before the ladder is written would have found the first two faults
    in an hour: a fetch moves a bookmark, and a restore is blind to the remote afterwards.
- A project rule was learned by tripping it: the ban on spawning git, met when clippy stopped
  the build and after the agent had said the spawn added nothing new.
  - The register in `clippy.toml` is where to look before a `Command::new` is written.
- A go was misread once, "option 1" taken to mean the body as it stood, and a published commit
  was re-described in both repos to add the legend line.
  - A choice that changes what is pushed is worth restating in the words of its effect before
    the push.
- The trial of this file is the subject of the Todo entry that makes it the rule, and what the
  trial showed is written there.

# References

[1]: #feat-sync-is-safe-by-default-opening
[2]: #feat-sync-fetches-only-a-repo-with-no-local-work
[3]: #feat-sync---dry-run-looks-and-changes-nothing
[4]: #feat-sync-tells-a-fast-forward-from-a-rewrite
[5]: #feat-sync-is-safe-by-default-closing
