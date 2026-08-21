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
- Retired: "bot repo" (2026-08-21), when the code respelled the side `agent`. Stage names and
  paths the code still spells with `bot` (`commit-bot`, `squash-push-bot`) are quoted as the
  code has them.
- A commit landing in the work repo is a "work-repo commit", never a bare "work commit".

**Agent-files.** The instruction set an agent reads: `AGENTS.md`, `custom.md`, `agent-data/*`, and
anything `custom.md` points at. The template repository's payload holds the official copies and
every member repo carries its own. How they change is
[Changing the agent-files](#changing-the-agent-files). Notes:

- Always hyphenated, unlike "work repo" above ([why](agent-data/rationale.md#terminology)).
- **Pinned** describes an agent-file whose content is meant to match the payload (`AGENTS.md`,
  `agent-data/*`). `custom.md` is an agent-file but is never pinned, and the same goes for any
  layer below it.
- Retired: "instruction files", which named the same set back when `custom.md` was the only
  editable one.

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
scheme), never a second statement of the flow.

The artifact a cycle produces is whatever the project generates: code, prose, an image, a song.
The validation commands are the project's, in the work side's `[validate]` table, and nothing
below names a build tool.

### Cycles run on a bookmark

A cycle runs on one topic bookmark in the work repo, created at the opening and named by the
cycle title's slug ([Markdown anchor links](agent-data/notes.md#markdown-anchor-links)). `main`
advances only when the finished cycle lands on it, and nothing pushes straight to `main`
([why](agent-data/rationale.md#cycles-run-on-a-bookmark)). The agent repo needs no bookmark:
its `main` rides the tip of its linear journal, and no agent-repo bookmark ever mirrors a
work-repo branch.

- **The bookmark is the unit of review.** Everything the cycle does is one line against `main`,
  and until it lands the line is a draft that can be reshaped
  ([Topic bookmarks are drafts](#topic-bookmarks-are-drafts)). Landing is the single approval
  that makes the cycle permanent.
- **A single-step cycle still gets one.**
- **Commands** are in [Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land). A
  long-lived program bookmark is a different animal, governed by jj.md's
  [Long-lived bookmarks][llb].

### Opening

The cycle's first commit, when it needs setup (a lightweight cycle omits it and starts at its
first commit, which then carries step 1 below). Before that commit
([why](agent-data/rationale.md#opening)):

1. **Backfill** every as-built ladder whose commits have landed since the last opening
   ([Commits backfill](#commits-backfill)), before anything else. The check is
   `rg '\[\[N\]\]' notes/chores/`: a hit outside a code span is owed work. This is the owner
   of close-out step 8's debt.
2. **Create the cycle's bookmark** and publish it: the create is itself a push and takes push
   approval.
3. **Move the picked-up `## Todo` item into `## In Progress`** (moved, never copied) and write
   the **six provisional items**, all required, all revised as rungs land, all moved to chores
   at close-out. The title is a heading one level below `## In Progress` and the other five
   are headings one level below the title (a plain cycle: `###` title, `####` items, and under
   a program heading, each one deeper):
   - **title**, which becomes the chores section header at close-out
   - **problem statement**: what is wrong, a sentence or two
   - **solution statement**: what will be done about it, broad. Provisional, and the
     close-out's commit body carries the final one
   - **acceptance check**: the measure of "are you finished?", specific enough that a reader
     can run it. Not the per-commit validation, which asks whether the artifact still works.
     A changed check is one of the things the deliberation exists to justify
   - **ladder**: one rung per step, `- [[N]] [<title>][M]` plus `(current)` / `(done)`, with
     `[M]: #<slug>` in the file's `# References`. The closing rung, `<cycle title> closing`,
     is linked like the rest
   - **deliberation**: how the five above were decided, alternatives weighed, costs accepted.
     `_None._` when there was nothing to deliberate, which is a real answer
   A **`Ladder details`** area follows the six: one subsection per rung, the closing included,
   headed by the rung's exact title. Each opens at laddering with an abstract-sized intent
   statement (the rung's problem and solution in a sentence or two) and completes at the rung's
   landing with the conceptual delta: design points, consequences, deferrals, never a restatement
   of the landed commit body. The closing rung's opens with the stub "Closing out the cycle."
   and completes at close-out with what closing taught, in problem/solution form, or `_None._`.
4. **Sweep `## Done`** per [Retiring Done entries](agent-data/notes.md#retiring-done-entries),
   then **bump the version-of-record** to the opening's version
   ([versioning.md](agent-data/versioning.md#suffix-scheme)).

Nothing is opened in the chores file here. The block is the cycle's only home until close-out
moves it ([Chores sections](#chores-sections)).

**Rungs are named, not numbered.** A rung is `- [[N]] [<title>][M] (marker)` and carries no
detail beyond that: the literal `[[N]]` is the as-built ladder's placeholder, filled only at
backfill after landing, and the title links to the rung's subsection below. A step is
identified by its title, verbatim in the rung, the chores `##` header, and the commit (hard
rule 9), so a title carries no number and no version. The version-of-record still bumps for
every rung and its suffix still encodes the stage, but that encoding belongs to the manifest
and appears nowhere in prose. A multi-step cycle's bookend commits are the cycle title plus
" opening" and " closing" (prose.md's [Cycle bookend titles][cbt]).

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
5. **Validate the artifact** with `vc-x1 validate`, at every commit, doc-only ones included. It
   runs the work side's `[validate] full` table in order, one command per element, stopping at
   the first failure. No validation while a review iterates: it runs once, on the settled
   state, after the last edit.
6. **Work review.** Stop *before* writing any description and say "please review". The stop is
   its own message and carries no title or body, drafted or final. Iterate until the user says
   "continue" / "go" or equivalent.
7. **Flip `(current)` to `(done)`**, the moment "done" becomes true, then **write the
   description** ([Commit description](#commit-description)).
8. **Description review.** Show the title + body and stop. Ask permission to commit and push
   without spelling out the invocation. This review covers the push only when the user's go
   explicitly includes it.
9. **Commit + push**, on the user's go: `vc-x1 push <bookmark> --title "..." --body "..."`
   ([Committing vs pushing](#committing-vs-pushing)). Then the
   [at-rest contract](#at-rest-push-stop-squash-push) applies.

The work review looks at the uncommitted working-copy diff. The user opens diffs in their
editor, and for the terminal `jj diff` is the working copy, `jj diff -r @-` the previous commit,
`jj show -r <X>` one revision's description and diff. Never `jj edit -r @-` to view a past
commit.

### Committing vs pushing

A cycle rung is committed *by* `vc-x1 push`, never pre-committed with `jj commit` (hard rule 1,
[why](agent-data/rationale.md#committing-vs-pushing)). Push's commit stages commit both repos
with the approved title and body and stamp each new commit's `ochid:` trailer
([ochid trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers)). In an instruction,
"commit", "push", and "commit + push" all mean `vc-x1 push`. A bare `jj commit` is asked for by
name ("local commit", "just `jj commit`") and is only for work that never publishes: local-only
saves and [local ladder](#local-ladders) intermediates, with no `ochid:`.

Three push behaviors to keep in mind:

- **No checks of the project's own.** vc-x1 runs no build or tests. Validation is the per-rung
  flow's job, run *before* the push. The one check that remains is `push-work` verifying the
  bookmark's remote refs are tracked.
- **Rerunning is safe.** Push keeps no state and cannot resume: every stage no-ops when its
  work is already done, so a failed run is re-run, not resumed. If push exits after `push-work`
  but before the agent-repo publish, `vc-x1 squash-push -R .claude` by hand is the rest of it.
- **`ochid:` trailers are stamped by push** (hard rule 5), never hand-written into `--title` or
  `--body`.

### Commit description

The title is a [Conventional Commit](https://www.conventionalcommits.org/),
`<type>: <short description>` with an optional `(scope)`, at the width in
[Line widths](agent-data/prose.md#line-widths). Common types: `feat`, `fix`, `refactor`, `test`,
`docs`, `chore`. Each rung gets its own descriptive title, sharing a greppable stem across the
cycle so `git log --grep` collects them, and distinct within its cycle and its chores file, where
it is also an anchor (hard rule 9).

The body is a **problem statement** then a **solution statement** in
[Commit-body form](agent-data/prose.md#commit-body-form): an intro paragraph stating the general
problem and defining any word the title assumes, `*` bullets for its facets, `-` bullets for
solutions, a `-` solving the nearest enclosing problem, wrapped per Line widths. No version in
title or body, no file list, and no deliberation
([why](agent-data/rationale.md#commit-description)). A work-repo body describes the artifact's
or the records' problem. An agent-repo body describes in-session activity. `ochid:` is the
body's last line, stamped by push, and a breaking change uses the hyphenated
`BREAKING-CHANGE:` trailer key.

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
   bookmark, and the agent repo's session data as one commit on its `main`, one push = one
   agent-repo commit paired with every work-repo commit in that push.
2. **The agent stops for the turn.** Once the turn's final push or squash-push is invoked, no
   further work: no verification, no summary, no next-step offer, no edit, until the user
   speaks. Closing words go *before* the invoke. The harness rejects an empty turn, so it may
   force a visible token after the tool returns, and then a bare acknowledgment ("landed") is
   all that is allowed, never a summary. Post-push verification happens next turn at the user's
   direction. Under a standing delegation, an intermediate push is just a step and the tail
   rides into the next rung's agent-repo commit. The hard stop lands on the turn's *final* push.
3. **The user runs `vc-x1 squash-push -R .claude`** after the agent goes quiet. It folds the
   tail into the published agent-repo commit and pushes `main`. Only the user can do this. The
   user repeats it if new writes land.

"Clean" means both repos' `@` empty. A late work-repo tweak after the push (a forgotten edit)
needs `jj squash --ignore-immutable` and a re-push, which is a remote rewrite and takes approval
like any push.

### Topic bookmarks are drafts

Landing on `main` is publication, and that is the line the rules divide at
([why](agent-data/rationale.md#topic-bookmarks-are-drafts)). Before landing, the series should
be self-consistent when practical: inserting or reordering a rung edits the ladder in the rungs
that already committed an older version of it, not only at the tip. After landing, the commits
are history and are not touched.

- **Amend content, never re-describe.** Editing `TODO.md` in a rung and amending is not a
  `jj describe`, so hard rule 4 stays intact.
- **Then force-push the bookmark**, under the same approval as any other push.
- **Exceptions**, named and moved past: the bookmark has already landed, another branch is
  stacked on it, or the ladder is long and only a trailing snapshot disagrees.

A [local ladder](#local-ladders) never meets this, since nothing on it is pushed.

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
   ([Chores sections](#chores-sections)), and add the title-only `## Table of Contents` entry.
4. **Write the `## Done` entry**: the version, then a bold title line with its chores `[N]` ref,
   detail as sub-bullets ([Done entry form](agent-data/notes.md#done-entry-form)). Replace the
   `## In Progress` block with `_No cycle currently in progress._`. Under a program heading this
   retires the cycle's block only: the program heading and its ladder stay, the shipped rung
   flipped `(done)`.
5. **Full validation**, mandatory, and `notes/README.md` updated if functionality changed.
6. **Surface the shape** at push time and wait for the user's choice: **squash** to one commit
   (right for a focused change), **trapezoid** (a merge commit whose first parent is the trunk
   and whose second is the ladder, so `git log --first-parent` reads one commit per cycle while
   every rung stays reachable, the current default), or **keep separate** (one commit per rung
   on `main`, when the decomposition itself is informative). A squash is set up before the
   push. A trapezoid is reshaped between two pushes, by the
   [trapezoid recipe](agent-data/jj.md#trapezoid-close-out-recipe), whose last step is
   `jj git push`, not `vc-x1 push`.
7. **Land the bookmark** on the user's go
   ([Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land)). Until this, nothing
   the cycle pushed is permanent. Once `main` contains the bookmark, delete it, locally and
   remotely (hard rule 13).
8. **Backfill** the chores as-built ladder for the commits landing just made permanent
   ([Commits backfill](#commits-backfill)). A commit cannot record its own SHA, so the edits
   are the next opening's first step ([Opening](#opening)), never this turn's.

### Chores sections

A **chores section** is a `##` section in `notes/chores/chores-NN.md` recording landed work, and
every commit that lands on the permanent branch should have a rung in some section's as-built
ladder. The section is created at close-out by moving the `## In Progress` block, four
transforms and no rewriting ([why](agent-data/rationale.md#chores-sections)):

- **Heading levels shift so the title becomes the section's `##`**, the items shifting with it.
  Anchors survive.
- **Rung refs renumber** into the destination file's `[N]` namespace
  ([Reference numbering](agent-data/notes.md#reference-numbering)).
- **Repo-root-relative links gain `../`**, since the block moves into `notes/chores/`.
- **The block's forward-looking notes are rewritten**, since they described a future that has
  now happened.

Check the renumbered refs and the rebased links by hand. Fuller conventions (content rules,
header sync, the Table of Contents) are in
[Chores conventions](agent-data/notes.md#chores-conventions).

#### Commits backfill

A rung is written with the literal `[[N]]` placeholder and no version, and is backfilled once
the commit is permanent, which is always one push later
([why](agent-data/rationale.md#commits-backfill)). On a topic bookmark the whole cycle waits
for landing. Backfill replaces the placeholder with a file-local `[N]` slot defined as the
commit URL plus 40-hex SHA in the file's `# References`
([Chores commit references](agent-data/notes.md#chores-commit-references)) and writes the
version ahead of the title. A deliberate rewrite of recorded commits invalidates their SHAs:
re-record them once the rewrite is published, on the same timing. Never record a SHA from the
window between a trapezoid's two pushes, or from any commit not on a permanent branch.

### Local ladders

When one rung's work benefits from incremental review, or grows enough to want its own
sub-cycle, it runs as a **local ladder**: a chain of jj commits that never leaves the machine and
collapses into the rung before the cycle continues
([why](agent-data/rationale.md#local-ladders)). Ladder commits are scratch, for review and
bisection only. Per ladder commit:

1. `jj new -R .`: a fresh empty `@`.
2. Do the commit's work.
3. `vc-x1 validate --fast` (the `[validate] fast` table). Non-negotiable.
4. `jj describe -m "..." -R .`: a scratch working title. This first-time authoring is the one
   permitted describe.

At the end, squash the chain into the rung (`jj squash --from "<base>..@-" --into @ -u -R .`,
`<base>` the parent of the first ladder commit) and continue the per-rung flow from step 5.
`vc-x1 push` then publishes the single commit and stamps its one `ochid:`. For a one-commit
loop the squash is a no-op. Navigation and recovery moves (editing an earlier ladder commit,
abandoning one, restoring an op) are in jj.md's [Local ladders](agent-data/jj.md#local-ladders).
A sub-cycle that deserves its own record nests the version suffix
([versioning.md](agent-data/versioning.md#suffix-scheme)) and names its rungs like any other.

[cbt]: agent-data/prose.md#conventional-commit-shape-ladder--chores--commit
[llb]: agent-data/jj.md#long-lived-bookmarks-merge-only-by-default-deletable-once-merged

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
