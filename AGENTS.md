# AGENTS.md - Agent Instructions

The universal core of this project's agent instructions: the dual-repo model, the hard rules, and
a map of everything else. This file is one of the [agent-files](#terminology), shared across our
dual repos and carried by every family member.

## Hard rules

The rules whose violation costs the most, numbered so a review can name them
([why](agent-data/rationale.md#hard-rules)). Each links to its detail. The rule as stated here is binding on its own. The rules bind the agent, and none is
absolute: any rule bends when wink says so explicitly at the moment, or in advance as an
explicit scoped delegation (rule 10's stop-and-ask is the path), and a taken exception is
recorded in the cycle's records. No rule bends silently, and no exception is self-granted.

0. **Read [custom.md](custom.md) before acting on anything below**: the project's layer
   (medium, conventions), loaded last, wins conflicts with this file and the
   satellites. Read it every session: only `AGENTS.md` is auto-loaded, and what to read past
   `custom.md` is `custom.md`'s to say.
1. **A cycle rung is committed by `vc-x1 push`, never pre-committed with `jj commit`.** In an
   instruction, "commit", "push", and "commit + push" all mean `vc-x1 push`. A bare `jj commit`
   is asked for by name and is only for work that never publishes.
   [Committing vs pushing](#committing-vs-pushing).
2. **Every push needs that push's explicit approval.** Approval of a plan that includes a push
   does not authorize the push. Ask again at the moment of pushing. Only an explicit scoped
   delegation waives the stops.
   [Before any push](#before-any-push).
3. **Hard stop after the turn's final push or squash-push.** Closing words go before the
   invoke. Afterwards, nothing until the user speaks (a bare acknowledgment if the harness
   forces a token).
   [At rest](#at-rest-push-stop-squash-push).
4. **Never `jj describe` a published or trailer-carrying commit without coordinating first.**
   When a re-describe is agreed, hand-copy the `ochid:` trailers into the new body.
   [Re-describing](agent-data/jj.md#re-describing-coordinate-first-and-keep-the-trailer).
5. **Never hand-write `ochid:` trailers.** `vc-x1 push` stamps them.
   [ochid trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers).
6. **Use jj, not git**, for version-control operations. [jj basics](agent-data/jj.md#jj-basics).
7. **Read the protocol step before the action**: [The per-rung flow](#the-per-rung-flow)
   before commit work and [Before any push](#before-any-push) before any push, from the file,
   not from memory. Validation runs before the push, never after.
8. **Typeable punctuation only** in durable text: no em/en dash, ellipsis, or arrow characters.
   [Typeable punctuation](agent-data/prose.md#typeable-punctuation-only).
9. **One title per step, verbatim in three places**: the ladder rung, the chores `##` header,
   and the commit title line up exactly. The title is a step's only identifier, so it carries no
   number and no version, and it must be unambiguous within its cycle and its chores file. See
   [the shape](agent-data/prose.md#conventional-commit-shape-ladder--chores--commit).
10. **Stop and ask** on ambiguous input, on any deviation from the agreed plan, and when 5+
    minutes on a simple task has produced no progress.
11. **Alert the user when introducing an `unwrap` / `expect` / `unwrap_or*` site**, with its
    `// OK: ...` comment. [code.md](agent-data/code.md).
12. **Intent decides where a rule change is written.** Meant for the family: edit the local copy
    of the pinned file the rule lives in, any time, so the diff against the payload is the
    proposal set. Not meant for the family: it belongs in `custom.md` instead, and has to say why
    it cannot be family-wide. The payload is never edited to experiment.
    [Changing the agent-files](#changing-the-agent-files).
13. **A cycle runs on one topic bookmark in the work repo**, named by the cycle title's slug,
    created at the opening, carrying every step. `main` advances only when the finished cycle
    lands on it, never by pushing commits straight to `main`. Once the bookmark lands on `main`
    the bookmark is deleted, locally and remotely.
    [Cycles run on a bookmark](#cycles-run-on-a-bookmark).

## Terminology

**Repos.** The two repos of [the dual-repo model](#the-dual-repo-model) below. "Work repo" and
"agent repo" are the standard names. Write them as two words, adding a hyphen only when the pair
sits directly in front of another noun ("work-repo commit", "agent-repo side"). Notes:

- `.claude` is the agent repo's *path*, not its name, so commands (`-R .claude`) and ochid paths
  (`/.claude/<chid>`) keep the literal path.
- The vc-x1 CLI's scope names are `work` and `agent` (`--scope=work|agent|work,agent`, and the
  same keywords as `vc-x1 config`'s target). `.vc-config.md` names the same two sides under
  `[repos]` as `work` / `agent`. A config still on the older `bot` spelling or the `[workspace]`
  schema is what `vc-x1 config --validate` reports, with the fix-it.
- Stage names and paths the code still spells with `bot` (`commit-bot`, `squash-push-bot`)
  are quoted as the code has them.
- A commit landing in the work repo is a "work-repo commit", never a bare "work commit".

**Agent-files.** The instruction set an agent reads: `AGENTS.md`, `custom.md`, `agent-data/*`, and
anything `custom.md` points at. The template repository's payload holds the official copies and
every member repo carries its own. How they change is
[Changing the agent-files](#changing-the-agent-files). Notes:

- Always hyphenated, unlike "work repo" above ([why](agent-data/rationale.md#terminology)).
- **Pinned** describes an agent-file whose content is meant to match the payload (`AGENTS.md`,
  `agent-data/*`). `custom.md` is an agent-file but is never pinned, and the same goes for any
  layer below it.

**Project layer.** The project's own agent-files, as against the pinned ones: `custom.md` and
anything it points at. It loads last and wins conflicts.

**Cycle.** One change, run from opening to closing as one commit or a ladder of them, each
commit made by `vc-x1 push`. The protocol is [Cycle protocol](#cycle-protocol). Notes:

- The commit is the unit of change. The cycle is how one change's commits are organized
  and recorded.
- A **single-step** cycle is one commit, which is then also the close-out and carries its
  duties.
- A **multi-step** cycle is a ladder of rungs, minimum two (a commit plus the closing, the
  opening being optional), typically three or more.
- The bookend commits are the cycle title plus " opening" and " closing". The bare title
  names the cycle: the chores `##` header and the `## Done` entry carry it.

**Rationale.** The why behind a rule: why it exists, what it cost to learn, what the
alternatives were. It lives in [rationale.md](agent-data/rationale.md), under a heading that
mirrors the rule's heading here, so a rule reaches its why by one fixed pattern,
`[why](agent-data/rationale.md#<same-slug>)`. This file holds the rule and its boundaries (a
sentence saying what a rule does not cover is the rule), and no rationale.

## The dual-repo model

This project uses **two separate jj-git repos**:

1. **Work repo** (`.`, the project root): the project's generated artifact, whether code,
   prose, image, song, or whatever it produces.
2. **Agent repo** (`.claude`): the agent's session data. The real directory is `<project>/.claude`.
   Claude Code reaches it through a symlink at `~/.claude/projects/<mangled-project-path>`
   pointing *at* that directory, with no further path component. `vc-x1 symlink` creates it.

Both are managed with `jj` (Jujutsu), which coexists with git. Every commit in one repo links
to its counterpart in the other via an `ochid:` trailer. See
[agent-data/jj.md](agent-data/jj.md).

## Cycle protocol

How a [cycle](#terminology) runs, from opening to closing. Its record lives in
`TODO.md > ## In Progress` while it runs and moves to `notes/chores/` when it closes, one home
at a time ([why](agent-data/rationale.md#cycle-protocol)). This section is the whole protocol.
The files it points at hold mechanics (jj commands, prose form, notes conventions, the version
scheme), never a second statement of the flow. The artifact a cycle produces is whatever the
project generates, and the validation commands are the project's, in the work side's
`[validate]` table.

### Cycles run on a bookmark

A cycle runs on one topic bookmark in the work repo, created at the opening and named by the
cycle title's slug ([Markdown anchor links](agent-data/notes.md#markdown-anchor-links)). `main`
advances only when the finished cycle lands on it, and nothing pushes straight to `main`
([why](agent-data/rationale.md#cycles-run-on-a-bookmark)). The agent repo needs no bookmark.
The bookmark is the unit of review: until it lands the line is a draft that can be reshaped
([Topic bookmarks are drafts](#topic-bookmarks-are-drafts)), and landing is the single approval
that makes the cycle permanent. A single-step cycle still gets one. Commands are in
[Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land), and a long-lived program
bookmark is governed by jj.md's [Long-lived bookmarks][llb].

### Opening

The cycle's first commit, when it needs setup (a lightweight cycle omits it and starts at its
first commit, which then carries step 1 below). Before that commit
([why](agent-data/rationale.md#opening)):

1. **Backfill** every as-built ladder whose commits have landed since the last opening
   ([Commits backfill](#commits-backfill)), before anything else. The check is
   `rg '\[\[N\]\]' notes/chores/`: a hit outside a code span is owed work. This is the owner
   of close-out step 7's debt.
2. **Create the cycle's bookmark** and publish it: the create is itself a push and takes push
   approval.
3. **Write the `## In Progress` block**: move the picked-up `## Todo` item in and write the six
   provisional items and the `Ladder details` area, per
   [The In Progress block](agent-data/notes.md#the-in-progress-block).
4. **Sweep `## Done`** per [Retiring Done entries](agent-data/notes.md#retiring-done-entries),
   then **bump the version-of-record** to the opening's version
   ([versioning.md](agent-data/versioning.md#suffix-scheme)).

Rungs are named, not numbered (prose.md's [Steps are named, not numbered][snn]), and a
multi-step cycle's bookend commits are the cycle title plus " opening" and " closing"
(prose.md's [Cycle bookend titles][cbt]).

### The per-rung flow

Every commit (opening, each rung between, closing) goes through these steps, read from here
immediately before acting and never from memory
([why](agent-data/rationale.md#the-per-rung-flow)):

1. **Mark the rung `(current)`** in `TODO.md > ## In Progress`, as the first edit.
2. **Do the work.** On any deviation from the agreed plan, or any question, stop and surface
   it (hard rule 10). The user can interrupt at any point to pull a review forward.
3. **Complete the rung's `Ladder details` subsection** with the conceptual delta. The rung
   stays `(current)`: the flip is step 7's.
4. **Bump the version-of-record** to this commit's version
   ([versioning.md](agent-data/versioning.md#suffix-scheme)). The opening's bump already covers
   an opening commit.
5. **Validate the artifact** with `vc-x1 validate`, at every commit, doc-only ones included.
   No validation while a review iterates: it runs once, on the settled state, after the last
   edit.
6. **Work review.** Stop *before* writing any description and say "please review". The stop is
   its own message and carries no title or body, drafted or final. Iterate until the user says
   "continue" / "go" or equivalent. The review looks at the uncommitted working-copy diff
   (viewing commands in [jj basics](agent-data/jj.md#jj-basics)).
7. **Flip `(current)` to `(done)`**, the moment "done" becomes true, then **write the
   description** ([Commit description](#commit-description)).
8. **Description review.** Show the title + body and stop. Ask permission to commit and push
   without spelling out the invocation. This review covers the push only when the user's go
   explicitly includes it.
9. **Commit + push**, on the user's go: `vc-x1 push <bookmark> --title "..." --body "..."`
   ([Committing vs pushing](#committing-vs-pushing)). Then the
   [at-rest contract](#at-rest-push-stop-squash-push) applies.

### Committing vs pushing

A cycle rung is committed *by* `vc-x1 push`, never pre-committed with `jj commit` (hard rule 1,
[why](agent-data/rationale.md#committing-vs-pushing)). In an instruction, "commit", "push", and
"commit + push" all mean `vc-x1 push`. A bare `jj commit` is asked for by name ("local commit",
"just `jj commit`") and is only for work that never publishes: local-only saves and
[local ladder](#local-ladders) intermediates, with no `ochid:`. What push does and does not do
is in jj.md's [vc-x1 push](agent-data/jj.md#vc-x1-push-what-it-does-and-does-not-do).

### Commit description

The title is a Conventional Commit, each rung's own, sharing a greppable stem across the cycle
and distinct within its cycle and its chores file (hard rule 9). The body is a **problem
statement** then a **solution statement** in
[Commit-body form](agent-data/prose.md#commit-body-form). No version in title or body, no file
list, and no deliberation ([why](agent-data/rationale.md#commit-description)). Details (types,
widths, work-repo versus agent-repo bodies, trailers) are in prose.md's
[Commit description details][cdd].

### Pushing

#### Policy

Push is discretionary during the cycle (backup, progress visibility) and mandatory at close-out.
**Approval is per-push** (hard rule 2): every push, any repo, any kind (rung push, interim
backup, recovery force-push), happens only after the user has reviewed the changes to be
published and explicitly approved that specific push. Approval of a plan that includes a push
does not authorize the push. Ask again at the moment of pushing. "Commit and push" names the
destination, not a waiver: it authorizes the push *after* the work review and the description
review.

**Default is interactive, and only an explicit scoped delegation waives the stops.** The stops
(work review, description review, per-push approval, the hard stop after the final push) yield
when the user explicitly delegates a complete, bounded task and authorizes carrying it through
("do all of X and push each step, don't check in"). Conditions: an explicit grant, never
inferred from a task being well-scoped. A bounded goal, covering the named task only. Each
commit and push still reported as it lands. When in doubt, ask.

**Delegation waives stops, never flow** ([why](agent-data/rationale.md#policy)): a delegated
cycle writes every record and validates every commit exactly as an interactive one. The tiers:
**interactive** (every stop), **delegated cycle** (rungs push to the topic bookmark without
per-push asks, `main` untouched by construction, review at landing), **delegated project**
(landing delegated too, corrections become new cycles). Destructive ops (a force-push over
published history, a history rewrite, deleting a remote branch) pause in every tier, and
landing is its own tier, delegated separately.

#### Before any push

- This specific push has the user's explicit approval, per the policy above.
- Validation ran, and passed, after the last edit.
- Closing words are already written. Nothing follows the turn's final push.

#### At rest: push, stop, squash-push

The contract that keeps both repos clean has three parts, and hard rule 3 is the middle one
([why](agent-data/rationale.md#at-rest-push-stop-squash-push)):

1. **The agent runs `vc-x1 push`**, which commits and publishes both repos: the work rung on its
   bookmark, and the agent repo's session data as one commit on its `main`.
2. **The agent stops for the turn.** Once the turn's final push or squash-push is invoked, no
   further work: no verification, no summary, no next-step offer, no edit, until the user
   speaks. Closing words go *before* the invoke. If the harness forces a visible token after
   the tool returns, a bare acknowledgment ("landed") is all that is allowed. Under a standing
   delegation, an intermediate push is just a step, and the hard stop lands on the turn's
   *final* push.
3. **The user runs `vc-x1 squash-push -R .claude`** after the agent goes quiet, and repeats it
   if new writes land. Only the user can do this.

"Clean" means both repos' `@` empty. A late work-repo tweak after the push is a remote rewrite
and takes approval like any push (jj.md's [vc-x1 push][vpush]).

### Topic bookmarks are drafts

Landing on `main` is publication, and that is the line the rules divide at
([why](agent-data/rationale.md#topic-bookmarks-are-drafts)). Before landing, the series should
be self-consistent when practical: inserting or reordering a rung edits the ladder in the rungs
that already committed an older version of it, not only at the tip. After landing, the commits
are history and are not touched. The reshape moves (amend content, never re-describe, then
force-push under approval, and the named exceptions) are in jj.md's
[Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land). A
[local ladder](#local-ladders) never meets this, since nothing on it is pushed.

### Close-out

The cycle's last commit is bookkeeping only, and its body describes that bookkeeping:

1. **Run the acceptance check** the opening stated, and record what it showed in the block,
   whether or not it passed. A check that was never run is a failed close-out, and a check that
   failed is a finding, not a reason to quietly restate the banner.
2. **Finalize the six items in place**: sync the title if the scope shifted (and every anchor
   back-reference), replace the provisional solution statement with what was done, drop the
   ladder's `(current)` / `(done)` markers since as-built implies done, add any design
   subsections the deliberation grew, and complete the closing rung's subsection.
3. **Move the block** into `notes/chores/chores-NN.md`, which is what creates the section
   ([Chores sections](#chores-sections)), add the title-only `## Table of Contents` entry, and
   **write the `## Done` entry** (notes.md's [The close-out move][tcm]).
4. **Full validation**, mandatory, and `notes/README.md` updated if functionality changed.
5. **Surface the shape** at push time and wait for the user's choice: squash, trapezoid (the
   current default), or keep separate (jj.md's
   [Close-out shapes](agent-data/jj.md#close-out-shapes)).
6. **Land the bookmark** on the user's go
   ([Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land)). Until this, nothing
   the cycle pushed is permanent. Once `main` contains the bookmark, delete it, locally and
   remotely (hard rule 13).
7. **Backfill** the chores as-built ladder for the commits landing just made permanent
   ([Commits backfill](#commits-backfill)). A commit cannot record its own SHA, so the edits
   are the next opening's first step ([Opening](#opening)), never this turn's.

### Chores sections

A **chores section** is a `##` section in `notes/chores/chores-NN.md` recording landed work, and
every commit that lands on the permanent branch should have a rung in some section's as-built
ladder. The section is created at close-out by moving the `## In Progress` block, four
transforms and no rewriting, checked by hand (notes.md's [The close-out move][tcm],
[why](agent-data/rationale.md#chores-sections)). Fuller conventions are in
[Chores conventions](agent-data/notes.md#chores-conventions).

#### Commits backfill

A rung is written with the literal `[[N]]` placeholder and no version, and is backfilled once
the commit is permanent, which is always one push later
([why](agent-data/rationale.md#commits-backfill)). On a topic bookmark the whole cycle waits
for landing. The shape of the fill, and the rewrite and trapezoid-window cautions, are in
[Chores commit references](agent-data/notes.md#chores-commit-references).

### Local ladders

When one rung's work benefits from incremental review, or grows enough to want its own
sub-cycle, it runs as a **local ladder**: a chain of jj commits that never leaves the machine and
collapses into the rung before the cycle continues
([why](agent-data/rationale.md#local-ladders)). Each ladder commit is validated with
`vc-x1 validate --fast` and described once, as scratch. The per-commit contract, the squash,
and the navigation and recovery moves are in jj.md's
[Local ladders](agent-data/jj.md#local-ladders).

[cbt]: agent-data/prose.md#conventional-commit-shape-ladder--chores--commit
[cdd]: agent-data/prose.md#conventional-commit-shape-ladder--chores--commit
[llb]: agent-data/jj.md#long-lived-bookmarks-merge-only-by-default-deletable-once-merged
[snn]: agent-data/prose.md#steps-are-named-not-numbered
[tcm]: agent-data/notes.md#the-close-out-move
[vpush]: agent-data/jj.md#vc-x1-push-what-it-does-and-does-not-do

## Working practices

- **Stay in the project root.** Target other directories with `-R` flags or absolute paths
  rather than `cd` (discuss with the user first if `cd` seems necessary).
- **Shortest unambiguous path** in shell commands (`ls notes/`, not the absolute form).
  Out-of-workspace paths stay absolute, and Read/Edit/Write tool args stay absolute (a
  tool-boundary constraint, not style).
- **One command per shell invocation.** Don't bundle steps (`a && b; c`)
  ([why](agent-data/rationale.md#working-practices)). Exceptions: a genuine pipeline (`grep | sort`) or a tight,
  inseparable pair where the join is the point.
- **Never mask a command's exit status.** What reads the result sees the invocation's status, so
  a command that fails has to make its invocation fail.
  - never pipe a validating command into `tail` / `grep`, and never `&&` after a piped stage.
    `${PIPESTATUS[0]}` is the escape hatch when a pipe is genuinely wanted
  - never trail one with `; echo "exit=$?"`
  - to report and still fail: `cmd || { rc=$?; echo failed=$rc; exit $rc; }`. Leave `failed=$rc`
    unquoted
- **Scratch files go in repo-local `tmp/`** (gitignored, `mkdir -p tmp` on demand, never
  committed). Prefer it over `/tmp` and the harness scratchpad. `/tmp` is for out-of-project
  temporaries.
- **Read the slice you need** from long notes files. The routine acquaint read is `TODO.md`
  `offset=0, limit=60`. [Notes files](agent-data/notes.md).
- **Use https remotes, not ssh.** Unconditional, not "when the agent is sandboxed". An **ssh
  remote is the first thing to check when a push dies at the network leg**, ahead of any theory
  about size or timeouts.
  - **Changing a remote's URL needs the user's go**, like any outward-facing change. Trivially
    reversible, so this is a confirmation and not a prohibition.
- **Delegate mechanical subtasks to lesser models** (Haiku / Sonnet). Reserve the top model for
  design and tricky work.
- **Don't use the per-project memory directory** (`~/.claude/projects/<path>/memory/`). Durable
  context lives in these committed files.
- **Mark speculation** in durable text with "We think ...".
  [Speculation marker](agent-data/prose.md#speculation-marker).
- **End technical explanations in conversation with a plain synopsis**, marked clearly (e.g.
  "The plain version:").
  [Plain synopsis](agent-data/prose.md#plain-synopsis-after-technical-explanations).

## File map

Read every session (`AGENTS.md` is the one file the harness auto-loads, and hard rule 0 covers
the rest of the chain):

- `AGENTS.md`: this file.
- [custom.md](custom.md): the project's layer, and any further file it points at.

Read at the moment of action, immediately before acting, not from memory. The
[Cycle protocol](#cycle-protocol) is in this file. The `agent-data/` files are universal and
pinned:

- [jj.md](agent-data/jj.md): jj usage, revsets, ochid trailers, the re-describe rule, cycle and
  long-lived bookmarks, the trapezoid recipe, local-ladder moves.
- [prose.md](agent-data/prose.md): prose form, punctuation, commit-title identity. Read before
  writing durable text.
- [notes.md](agent-data/notes.md): TODO / chores / done mechanics, references, anchors. Read
  before editing notes files.
- [code.md](agent-data/code.md): doc comments and unwrap discipline. Read before writing code.
- [versioning.md](agent-data/versioning.md): the version scheme and version-of-record.
- [messaging.md](agent-data/messaging.md): the family's notification repo, the acquaint check,
  and what a request becomes. Read at acquaint, when the work side's config has a `[family]`
  table.
- [rationale.md](agent-data/rationale.md): the why behind this file's rules, headings
  mirroring this file's. Read when changing a rule, never needed to apply one.

Project records (`notes/` and the repo root): records only, never universal rules. Anything
normative that outgrows the project belongs in `agent-data/` via
[Changing the agent-files](#changing-the-agent-files):

- `TODO.md`, `notes/todo-backlog.md`, `notes/bugs.md`, `notes/chores/`, `notes/done.md`: the
  project's working records. Conventions are in [agent-data/notes.md](agent-data/notes.md).

## Changing the agent-files

The **agent-files** are `AGENTS.md`, `custom.md`, and `agent-data/*`. The official copies are the
template repository's payload, and every member repo carries its own copy of the same set
([why](agent-data/rationale.md#changing-the-agent-files)).

- **The payload is the read-only copy.** A member never edits it to experiment. The one thing
  that goes straight in is a *correction*: a factual error, a typo, a stale cross-reference.
- **Intent decides the file, and nothing gates the edit.** A member writes a rule change into its
  local copy of the pinned file whenever it means the family to take it, without asking first.
  The review happens at convergence, on the diff. A change the member does *not* mean the family
  to take goes to `custom.md` and must say why it cannot be family-wide.
- **The diff between a member and the payload *is* that member's open proposal set.**
- **An agent-file change is its own commit.**
- **Convention work runs as its own cycle.** A convention itch mid-feature becomes a backlog
  entry or a small dedicated cycle, never an inserted rung in the feature's ladder.
- **A local agent-file may hold an unagreed experiment**, so unlike the payload it does not read
  as family-agreed. Diff against the payload when that distinction matters.
- **At convergence** the family reviews the members' diffs, folds what it accepts into the
  payload, and every member re-syncs. The diff empties, and the history keeps the record.
- **A resolved experiment retires** like a finished Todo, at the beat where it resolves: see
  [Retiring Done entries](agent-data/notes.md#retiring-done-entries). Adopted and rejected retire
  the same way.
- **A rule adopted ahead of its convention cycle lives in the pinned file it belongs to**, as
  the diff against the payload, never in a holding section of the project layer. The diff is
  the holding area, and it needs no section.

## custom.md: the project layer

[custom.md](custom.md) is the project's own layer and, unlike the pinned files, is never pinned:
every project's content differs by construction ([why](agent-data/rationale.md#custommd-the-project-layer)).
It ships from the payload holding nothing but its own shape, and a project adds whatever it
needs: the medium, what a version bump promises this artifact's users, and its conventions.

**`## Project conventions and overrides` is empty at birth and should usually stay that way.** A
rule the project would keep is still a *proposal* until it is rejected, so by default it belongs in
the pinned file where the rule lives (see [Changing the agent-files](#changing-the-agent-files)),
where it shows up as a diff. An empty section stays, with `_None._` under it, rather than being
deleted.

**An entry that only points at a further file is not an override** and owes no "why not
family-wide" justification. A project with a wider context to answer to can hold all of it in
that further file and reach it from one line here. Nothing pinned names the further file or
knows what is in it:
a pinned file asking for something "in custom.md" is answered by following the pointer it finds
there.

Precedence: custom.md is loaded last and wins conflicts with this file and the satellites.
