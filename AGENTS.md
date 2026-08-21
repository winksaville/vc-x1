# AGENTS.md - Agent Instructions

The universal core of the agent instructions: the dual-repo model, the hard rules, the cycle
protocol, and a map of the rest. One of the [agent-files](#terminology), carried by every family
member.

## Hard rules

The rules whose violation costs the most, numbered so a review can name them
([why](agent-data/rationale.md#hard-rules)). Each is binding as stated and links to its detail. None
is absolute: a rule bends when wink says so explicitly, at the moment or in advance as a scoped
delegation (rule 10 is the path), and the exception is recorded in the cycle's records. No rule
bends silently, and no exception is self-granted.

0. Read custom.md first: read [custom.md](custom.md) before acting on anything below, every session.
   It is the project's layer, loaded last, and wins conflicts. What to read past it is its to say.
1. Push commits: a cycle rung is committed by `vc-x1 push`, never pre-committed with `jj commit`.
   "Commit", "push", and "commit + push" all mean `vc-x1 push`. A bare `jj commit` is asked for by
   name and is only for work that never publishes. [Committing vs pushing](#committing-vs-pushing).
2. Approval per push: every push needs that push's explicit approval, asked at the moment of
   pushing. Approving a plan that includes a push does not approve the push. Only an explicit scoped
   delegation waives the stops. [Before any push](#before-any-push).
3. Hard stop after the final push: after the turn's final push or squash-push, nothing until the
   user speaks (a bare acknowledgment if the harness forces a token). Closing words go before the
   invoke. [At rest](#at-rest-push-stop-squash-push).
4. No re-describe without coordinating: never `jj describe` a published or trailer-carrying commit
   without coordinating first. When agreed, hand-copy the `ochid:` trailers into the new body.
   [Re-describing](agent-data/jj.md#re-describing-coordinate-first-and-keep-the-trailer).
5. No hand-written trailers: `vc-x1 push` stamps `ochid:` trailers; never write one by hand.
   [ochid trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers).
6. jj, not git: version-control operations use jj. [jj basics](agent-data/jj.md#jj-basics).
7. Read the step before the action: [The per-rung flow](#the-per-rung-flow) before commit work,
   [Before any push](#before-any-push) before any push, from the file, not from memory. Validation
   runs before the push, never after.
8. Typeable punctuation: no em/en dash, ellipsis, or arrow characters in durable text.
   [Typeable punctuation](agent-data/prose.md#typeable-punctuation-only).
9. One title per step: the ladder rung, the chores `##` header, and the commit title are verbatim
   identical. The title is the step's only identifier: no number, no version, unambiguous within its
   cycle and its chores file.
   [The shape](agent-data/prose.md#conventional-commit-shape-ladder--chores--commit).
10. Stop and ask: on ambiguous input, on any deviation from the agreed plan, and when 5+ minutes on
    a simple task has produced no progress.
11. Alert on unwrap: say so when introducing an `unwrap` / `expect` / `unwrap_or*` site, with its
    `// OK: ...` comment. [code.md](agent-data/code.md).
12. Intent picks the file: a rule change meant for the family is edited into the local copy of the
    pinned file it lives in, any time, so the diff against the payload is the proposal set. One not
    meant for the family goes in `custom.md` and says why it cannot be family-wide. The payload is
    never edited to experiment. [Changing the agent-files](#changing-the-agent-files).
13. One bookmark per cycle: a cycle runs on one topic bookmark in the work repo, named by the cycle
    title's slug, created at the opening, carrying every step. `main` advances only when the
    finished cycle lands on it, never by a direct push. A landed bookmark is deleted, locally and
    remotely. [Cycles run on a bookmark](#cycles-run-on-a-bookmark).

## Terminology

Repos: the two repos of [the dual-repo model](#the-dual-repo-model), "work repo" and "agent repo".
Two words, hyphenated only directly before another noun ("work-repo commit", "agent-repo side").
Notes:

- `.claude` is the agent repo's *path*, not its name: commands (`-R .claude`) and ochid paths
  (`/.claude/<chid>`) keep the literal path.
- The CLI's scope names are `work` and `agent` (`--scope=work|agent|work,agent`, and the same
  keywords for `vc-x1 config`'s target), and `.vc-config.md` names the sides `[repos]` `work` /
  `agent`. A config on the older `bot` spelling or `[workspace]` schema is what
  `vc-x1 config --validate` reports, with the fix-it.
- Stage names the code still spells with `bot` (`commit-bot`, `squash-push-bot`) are quoted as the
  code has them.
- A commit landing in the work repo is a "work-repo commit", never a bare "work commit".

Agent-files: the instruction set an agent reads: `AGENTS.md`, `custom.md`, `agent-data/*`, and
anything `custom.md` points at. The template repository's payload holds the official copies and
every member repo carries its own ([Changing the agent-files](#changing-the-agent-files)). Always
hyphenated ([why](agent-data/rationale.md#terminology)). Pinned: an agent-file whose content is
meant to match the payload (`AGENTS.md`, `agent-data/*`). `custom.md` and any layer below it are
never pinned.

Project layer: the project's own agent-files, `custom.md` and anything it points at. Loaded last,
wins conflicts.

Cycle: one change, run from opening to closing as one commit or a ladder of them, each made by
`vc-x1 push` ([Cycle protocol](#cycle-protocol)). The commit is the unit of change; the cycle is how
one change's commits are organized and recorded. A single-step cycle is one commit, which is also
the close-out and carries its duties. A multi-step cycle is a ladder of two or more rungs (a commit
plus the closing; the opening is optional). The bookend commits are the cycle title plus " opening"
and " closing", and the bare title names the cycle in the chores `##` header and the `## Done`
entry.

Rationale: a rule's why: why it exists, what it cost to learn, what the alternatives were. It lives
in [rationale.md](agent-data/rationale.md) under the heading that mirrors the rule's here, reached
by one pattern, `[why](agent-data/rationale.md#<same-slug>)`. This file holds the rule and its
boundaries (a sentence saying what a rule does not cover is the rule), and no rationale.

## The dual-repo model

Two separate jj-git repos:

1. Work repo: `.`, the project root, holding the project's artifact, whether code, prose, image,
   song, or whatever it produces.
2. Agent repo: `.claude`, the agent's session data. Claude Code reaches `<project>/.claude` through
   a symlink at `~/.claude/projects/<mangled-project-path>` pointing at that directory, with no
   further path component. `vc-x1 symlink` creates it.

Both are managed with `jj`, which coexists with git, and every commit in one links to its
counterpart in the other by an `ochid:` trailer ([jj.md](agent-data/jj.md)).

## Cycle protocol

How a [cycle](#terminology) runs. Its record lives in `TODO.md > ## In Progress` while it runs and
moves to `notes/chores/` when it closes, one home at a time
([why](agent-data/rationale.md#cycle-protocol)). This section is the whole protocol; the files it
points at hold mechanics, never a second statement of the flow. The artifact is whatever the project
generates, and the validation commands are the work side's `[validate]` table.

### Cycles run on a bookmark

A cycle runs on one topic bookmark in the work repo, created at the opening and named by the cycle
title's slug ([Markdown anchor links](agent-data/notes.md#markdown-anchor-links)). `main` advances
only when the finished cycle lands on it ([why](agent-data/rationale.md#cycles-run-on-a-bookmark)).
The agent repo needs no bookmark. The bookmark is the unit of review: until it lands the line is a
draft ([Topic bookmarks are drafts](#topic-bookmarks-are-drafts)), and landing is the one approval
that makes the cycle permanent. A single-step cycle still gets one. Commands are in
[Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land); a long-lived program bookmark
is governed by [Long-lived bookmarks][llb].

### Opening

The cycle's first commit, when it needs setup (a lightweight cycle starts at its first commit, which
then carries step 1). Before that commit ([why](agent-data/rationale.md#opening)):

1. Backfill: fill every as-built ladder whose commits have landed since the last opening
   ([Commits backfill](#commits-backfill)). The check is `rg '\[\[N\]\]' notes/chores/`: a hit
   outside a code span is owed work. This pays close-out step 7's debt.
2. Bookmark: create the cycle's bookmark and publish it. The create is a push and takes push
   approval.
3. In Progress block: move the picked-up `## Todo` item in and write the six provisional items and
   the `Ladder details` area ([The In Progress block](agent-data/notes.md#the-in-progress-block)).
4. Sweep and bump: sweep `## Done` ([Retiring Done entries][rde]), then bump the version-of-record
   to the opening's version ([versioning.md](agent-data/versioning.md#suffix-scheme)).

Rungs are named, not numbered ([Steps are named, not numbered][snn]), and a multi-step cycle's
bookend commits are the cycle title plus " opening" and " closing" ([Cycle bookend titles][cbt]).

### The per-rung flow

Every commit (opening, each rung between, closing) goes through these steps, read from here
immediately before acting ([why](agent-data/rationale.md#the-per-rung-flow)):

1. Mark current: mark the rung `(current)` in `TODO.md > ## In Progress`, as the first edit.
2. Work: do the work. On any deviation from the agreed plan, or any question, stop and surface it
   (hard rule 10). The user can interrupt at any point to pull a review forward.
3. Ladder details: complete the rung's subsection with the conceptual delta. The rung stays
   `(current)` until step 7.
4. Bump: bump the version-of-record to this commit's version
   ([versioning.md](agent-data/versioning.md#suffix-scheme)). The opening's bump covers an opening
   commit.
5. Validate: run `vc-x1 validate`, at every commit, doc-only ones included, once, on the settled
   state, after the last edit. Never while a review iterates.
6. Work review: stop *before* writing any description and say "please review". The stop is its own
   message and carries no title or body, drafted or final. Iterate until the user says "continue" /
   "go" or equivalent. The review is of the uncommitted working-copy diff (viewing commands in
   [jj basics](agent-data/jj.md#jj-basics)).
7. Flip and describe: flip `(current)` to `(done)` the moment "done" is true, then write the
   description ([Commit description](#commit-description)).
8. Description review: show the title + body and stop. Ask permission to commit and push without
   spelling out the invocation. The user's go covers the push only when it says so.
9. Commit + push: on the go, `vc-x1 push <bookmark> --title "..." --body "..."`
   ([Committing vs pushing](#committing-vs-pushing)). Then the
   [at-rest contract](#at-rest-push-stop-squash-push) applies.

### Committing vs pushing

A cycle rung is committed *by* `vc-x1 push`, never pre-committed with `jj commit` (hard rule 1,
[why](agent-data/rationale.md#committing-vs-pushing)). "Commit", "push", and "commit + push" all
mean `vc-x1 push`. A bare `jj commit` is asked for by name ("local commit", "just `jj commit`") and
is only for work that never publishes: local-only saves and [local ladder](#local-ladders)
intermediates, with no `ochid:`. What push does and does not do is in [vc-x1 push][vpush].

### Commit description

The title is a Conventional Commit, each rung's own, sharing a greppable stem across the cycle and
distinct within its cycle and its chores file (hard rule 9). The body is a problem statement then a
solution statement in [Commit-body form](agent-data/prose.md#commit-body-form). No version, no file
list, no deliberation ([why](agent-data/rationale.md#commit-description)). Details are in
[Commit description details][cdd].

### Pushing

#### Policy

Push is discretionary during the cycle and mandatory at close-out. Approval per push (hard rule 2):
every push, any repo, any kind (rung, interim backup, recovery force-push), happens only after the
user has reviewed the changes and approved that specific push. Approving a plan that includes a push
does not approve the push; ask at the moment. "Commit and push" names the destination, not a waiver:
it approves the push *after* the work review and the description review.

Interactive by default: only an explicit scoped delegation waives the stops (work review,
description review, per-push approval, the hard stop). It takes an explicit grant of a complete,
bounded task ("do all of X and push each step, don't check in"), never inferred from a task being
well-scoped, covering the named task only, with each commit and push still reported as it lands.
When in doubt, ask.

Stops, never flow: a delegated cycle writes every record and validates every commit exactly as an
interactive one ([why](agent-data/rationale.md#policy)). Tiers: interactive (every stop), delegated
cycle (rungs push to the topic bookmark without per-push asks, review at landing), delegated project
(landing delegated too, corrections become new cycles). Destructive ops (a force-push over published
history, a history rewrite, deleting a remote branch) pause in every tier, and landing is delegated
separately.

#### Before any push

- This specific push has the user's explicit approval.
- Validation ran, and passed, after the last edit.
- Closing words are written. Nothing follows the turn's final push.

#### At rest: push, stop, squash-push

The contract that keeps both repos clean, hard rule 3 its middle part
([why](agent-data/rationale.md#at-rest-push-stop-squash-push)):

1. Agent pushes: `vc-x1 push` commits and publishes both repos, the work rung on its bookmark and
   the session data as one commit on the agent repo's `main`.
2. Agent stops: once the turn's final push or squash-push is invoked, no further work of any kind
   until the user speaks. Closing words go *before* the invoke. If the harness forces a token, a
   bare acknowledgment ("landed") is all that is allowed. Under a standing delegation an
   intermediate push is a step, and the hard stop lands on the turn's *final* push.
3. User squash-pushes: the user runs `vc-x1 squash-push -R .claude` after the agent goes quiet, and
   again if new writes land. Only the user can do this.

"Clean" means both repos' `@` empty. A late work-repo tweak after the push is a remote rewrite and
takes approval like any push ([vc-x1 push][vpush]).

### Topic bookmarks are drafts

Landing on `main` is publication ([why](agent-data/rationale.md#topic-bookmarks-are-drafts)).
Before landing, the series stays self-consistent when practical: inserting or reordering a rung
edits the ladder in the rungs that committed an older version of it, not only at the tip. After
landing, the commits are history and are not touched. The reshape moves (amend content, never
re-describe, then force-push under approval, and the exceptions) are in
[Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land). A
[local ladder](#local-ladders) never meets this, since nothing on it is pushed.

### Close-out

The cycle's last commit is bookkeeping only, and its body describes that bookkeeping:

1. Acceptance check: run the check the opening stated and record what it showed, pass or fail. A
   check never run is a failed close-out; a check that failed is a finding, not a reason to restate
   the banner.
2. Finalize the block: sync the title if the scope shifted (and every anchor back-reference),
   replace the provisional solution statement with what was done, drop the `(current)` / `(done)`
   markers, add any design subsections the deliberation grew, and complete the closing rung's
   subsection.
3. Move and record: move the block into `notes/chores/chores-NN.md`, which creates the section
   ([Chores sections](#chores-sections)), add the title-only `## Table of Contents` entry, and write
   the `## Done` entry ([The close-out move][tcm]).
4. Validate: full validation, and `notes/README.md` updated if functionality changed.
5. Shape: surface the shape at push time and wait for the user's choice: squash, trapezoid (the
   default), or keep separate ([Close-out shapes](agent-data/jj.md#close-out-shapes)).
6. Land: land the bookmark on the user's go
   ([Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land)). Until then nothing the
   cycle pushed is permanent. Once `main` contains it, delete it, locally and remotely (hard rule
   13).
7. Backfill: fill the chores as-built ladder for the commits landing made permanent
   ([Commits backfill](#commits-backfill)). A commit cannot record its own SHA, so this is the next
   opening's first step, never this turn's.

### Chores sections

A chores section is a `##` section in `notes/chores/chores-NN.md` recording landed work, and every
commit on the permanent branch should have a rung in some section's as-built ladder. It is created
at close-out by moving the `## In Progress` block, four transforms and no rewriting, checked by hand
([The close-out move][tcm], [why](agent-data/rationale.md#chores-sections)). Conventions are in
[Chores conventions](agent-data/notes.md#chores-conventions).

#### Commits backfill

A rung is written with the literal `[[N]]` placeholder and no version, and is backfilled once the
commit is permanent, always one push later ([why](agent-data/rationale.md#commits-backfill)). On a
topic bookmark the whole cycle waits for landing. The fill's shape and its cautions are in
[Chores commit references](agent-data/notes.md#chores-commit-references).

### Local ladders

A rung that wants incremental review, or its own sub-cycle, runs as a local ladder: a chain of jj
commits that never leaves the machine and collapses into the rung before the cycle continues
([why](agent-data/rationale.md#local-ladders)). Each ladder commit is validated with
`vc-x1 validate --fast` and described once, as scratch. The contract, the squash, and the navigation
and recovery moves are in [Local ladders](agent-data/jj.md#local-ladders).

[cbt]: agent-data/prose.md#conventional-commit-shape-ladder--chores--commit
[cdd]: agent-data/prose.md#conventional-commit-shape-ladder--chores--commit
[llb]: agent-data/jj.md#long-lived-bookmarks-merge-only-by-default-deletable-once-merged
[rde]: agent-data/notes.md#retiring-done-entries
[snn]: agent-data/prose.md#steps-are-named-not-numbered
[tcm]: agent-data/notes.md#the-close-out-move
[vpush]: agent-data/jj.md#vc-x1-push-what-it-does-and-does-not-do

## Working practices

- Project root: stay in it. Target other directories with `-R` or absolute paths rather than `cd`
  (ask first if `cd` seems necessary).
- Short paths: the shortest unambiguous path in shell commands (`ls notes/`). Out-of-workspace paths
  and Read/Edit/Write tool args stay absolute.
- One command per invocation: no bundled steps (`a && b; c`)
  ([why](agent-data/rationale.md#working-practices)). Exceptions: a genuine pipeline, or a tight
  pair where the join is the point.
- Exit status: never mask a command's exit status; a command that fails makes its invocation fail.
  - never pipe a validating command into `tail` / `grep`, and never `&&` after a piped stage.
    `${PIPESTATUS[0]}` is the escape hatch when a pipe is wanted
  - never trail one with `; echo "exit=$?"`
  - to report and still fail: `cmd || { rc=$?; echo failed=$rc; exit $rc; }`, `failed=$rc` unquoted
- Scratch files: repo-local `tmp/` (gitignored, `mkdir -p tmp` on demand), over `/tmp` and the
  harness scratchpad. `/tmp` is for out-of-project temporaries.
- Slice reads: read the slice you need from long notes files. The acquaint read is `TODO.md`
  `offset=0, limit=60` ([notes.md](agent-data/notes.md)).
- https remotes: use https remotes, not ssh, unconditionally. An ssh remote is the first thing to
  check when a push dies at the network leg, ahead of any theory about size or timeouts.
  - Remote URL changes: need the user's go, like any outward-facing change. A confirmation, not a
    prohibition.
- Delegate: mechanical subtasks go to lesser models (Haiku / Sonnet); the top model is for design
  and tricky work.
- No memory directory: `~/.claude/projects/<path>/memory/` is unused. Durable context lives in these
  committed files.
- Speculation: mark it in durable text with "We think ..."
  ([Speculation marker](agent-data/prose.md#speculation-marker)).
- Plain synopsis: end a technical explanation in conversation with one, marked "The plain version:"
  ([Plain synopsis](agent-data/prose.md#plain-synopsis-after-technical-explanations)).

## File map

Read every session (`AGENTS.md` is the one file the harness auto-loads; hard rule 0 covers the rest
of the chain):

- `AGENTS.md`: this file.
- [custom.md](custom.md): the project's layer, and any further file it points at.

Read at the moment of action, not from memory. The [Cycle protocol](#cycle-protocol) is in this
file. The `agent-data/` files are universal and pinned:

- [jj.md](agent-data/jj.md): jj usage, revsets, ochid trailers, re-describing, vc-x1 push, cycle
  and long-lived bookmarks, close-out shapes and the trapezoid recipe, local-ladder moves.
- [prose.md](agent-data/prose.md): prose form, punctuation, commit-title identity. Before writing
  durable text.
- [notes.md](agent-data/notes.md): TODO / In Progress / chores / done mechanics, references,
  anchors. Before editing notes files.
- [code.md](agent-data/code.md): doc comments and unwrap discipline. Before writing code.
- [versioning.md](agent-data/versioning.md): the version scheme and version-of-record.
- [messaging.md](agent-data/messaging.md): the family's notification repo, the acquaint check, and
  what a request becomes. At acquaint, when the work side's config has a `[family]` table.
- [rationale.md](agent-data/rationale.md): the why behind this file's rules, headings mirroring
  this file's. When changing a rule, never needed to apply one.

Project records (`notes/` and the repo root) are records only, never universal rules; anything
normative that outgrows the project moves to `agent-data/` via
[Changing the agent-files](#changing-the-agent-files). `TODO.md`, `notes/todo-backlog.md`,
`notes/bugs.md`, `notes/chores/`, `notes/done.md` are the working records, conventions in
[notes.md](agent-data/notes.md).

## Changing the agent-files

The agent-files are `AGENTS.md`, `custom.md`, and `agent-data/*`. The official copies are the
template repository's payload, and every member repo carries its own copy
([why](agent-data/rationale.md#changing-the-agent-files)).

- Payload read-only: a member never edits the payload to experiment. Only a *correction* goes
  straight in: a factual error, a typo, a stale cross-reference.
- Intent picks the file: a rule change meant for the family goes into the local copy of the pinned
  file, without asking first; the review is at convergence, on the diff. One not meant for the
  family goes to `custom.md` and says why it cannot be family-wide.
- Diff is the proposal: the diff between a member and the payload *is* its open proposal set.
- Own commit: an agent-file change is its own commit.
- Own cycle: convention work runs as its own cycle. A convention itch mid-feature becomes a backlog
  entry or a small dedicated cycle, never an inserted rung.
- Local experiments: a local agent-file may hold an unagreed experiment, so it does not read as
  family-agreed. Diff against the payload when that matters.
- Convergence: the family reviews the members' diffs, folds what it accepts into the payload, and
  every member re-syncs. The diff empties; the history keeps the record.
- Retirement: a resolved experiment retires like a finished Todo, at the beat where it resolves
  ([Retiring Done entries][rde]), adopted and rejected alike.
- Adopted ahead: a rule adopted ahead of its convention cycle lives in the pinned file it belongs
  to, as the diff against the payload, never in a holding section of the project layer.

## custom.md: the project layer

[custom.md](custom.md) is the project's own layer and is never pinned
([why](agent-data/rationale.md#custommd-the-project-layer)). It ships from the payload holding only
its own shape, and a project adds what it needs: the medium, what a version bump promises, its
conventions.

Overrides section: `## Project conventions and overrides` is empty at birth and usually stays so. A
rule the project would keep is still a *proposal* until rejected, so it belongs in the pinned file
where the rule lives ([Changing the agent-files](#changing-the-agent-files)), as a diff. An empty
section stays, with `_None._` under it.

Pointer entries: an entry that only points at a further file is not an override and owes no
justification. A project with a wider context can hold all of it in that file and reach it from one
line here. Nothing pinned names the further file: a pinned file asking for something "in custom.md"
is answered by following the pointer found there.

Precedence: custom.md is loaded last and wins conflicts with this file and the satellites.
