# Todo list

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

The other files are listed in [zTOC.md](zTOC.md).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record, read first at
acquaint, acted on, and reset to `_None._` by the reader.

- **feat: improve vc-x1 cycle flow** is open, its opening pushed as a draft on
  `feat-improve-vc-x1-cycle-flow` (2026-11-11). wink then delegated the work rungs, and all
  thirteen are committed as pairs, unreviewed, twelve pushed as a draft and the last local, so
  `zTOC.md`'s ladder reads `(done)` to the closing, which is `(current)`. `vc-x1-dev cycle --status` says the same.
  - Each rung's subsection under `#### Rungs` says what it did and what it left for wink, the
    lines starting "For wink" and "Not done" among them.
  - The closing, the Land, and every push wait for wink. The closing owes the acceptance check,
    `scripts/cycle-rehearsal.sh`, at both ends, the record's finalizing, a row in
    `notes/agent-files-size.md`, and `cargo xtask bump bare`.
  - The installed `vc-x1` is 0.85.21 and has the old commands. `vc-x1-dev` is this branch's, and
    is the one that has `cycle`, so the closing is `vc-x1-dev cycle --close -s b --trapezoid main`.
- `~/tmp` holds wink's test workspaces of the last cycle, `wt-21-6` and `remotes`, disposable now
  that `scripts/cycle-rehearsal.sh` makes its own under `~/tmp/cycle-rehearsal`.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's shape is
the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in [The In Progress
block](agent-data/notes.md#the-in-progress-block).

### feat: improve vc-x1 cycle flow

#### Problem

(wink, 2026-11-11) I want to improve the vc-x1 cycle flow so the user or agents can effectively and
consistently follow the rules of [Agents.md Cycle Protocol](AGENTS.md#cycle-protocol). While doing
this I expect us to modify the current Cycle Protocol so it will be as good as possible based on
our current usage model.

(wink + agent, 2026-11-10) wink, running the single-step flow by hand in `~/tmp/wt-21-6`, found
three faults in a row, fixed in **feat: open makes the cycle's bookmark first**, and vc-x2 is
postponed until vc-x1's flow behaves predictably.

#### Solution

Provisional: the cycle's acts move out of `commit`, `open`, and `land` into one command, `cycle`,
whose mode flags are the flow's stages, `--open`, `--commit`, `--close`, `--land`, and `--status`.
The tool keeps the record's mechanical parts, the scaffold, the ladder, and its markers, and the
user and the agent keep the judgment. The version is the project's, bumped by an xtask and only
checked by the tool. A rehearsal script runs the flow end to end, and the Cycle Protocol is revised
to what the commands then do.

#### Acceptance check

Provisional: `scripts/cycle-rehearsal.sh` runs two single-step cycles and a ladder end to end by
`cycle`'s flags, open to Land, in a `~/tmp` workspace that holds no cargo project, every command
exiting zero and the script checking what each left: the bookmark, the record, the ladder's
markers, the commits' titles and trailers, and the destination where the Land put it. It exits
zero. It runs as the first thing in closing and again after it.

#### Deliberation

- The ladder was completed by the agent with wink away (wink, 2026-11-11): "go, commit the
  opening, and then permission to complete the ladder pushing but leaving it on the branch".
  - The agent read "complete the ladder" as planning it, the rungs written into this record and
    `zTOC.md` and folded into the opening by `vc-x1 squash`, and not as doing the rungs' work.
    wink corrected the reading the same day: "I meant and should have said was complete the
    ladder up to closing but don't close. So proceed with the permission to do that and commit
    locally both work and agent repo's".
  - So the work rungs are delegated: the agent does each, validates it, and commits it locally as
    a pair by `vc-x1 commit -s b`, with no "please review" stop, the review being wink's of the
    whole ladder after. The delegation covers the per-rung review alone. It does not cover the
    closing, a Land, or any push, the draft push below having been made before it.
  - The waiver covers pushing the cycle's bookmark, a draft, in that session. It does not cover a
    Land, or a push of any destination.
  - The ladder is the agent's, unreviewed, for wink to change: its order, its split, and what it
    leaves out.
- What the ladder leaves out, each staying in `## Todo`: **push only publishes**, **validate-desc
  shows a note as part of its row**, and the two faults found at the last closing, filed as
  **validate-desc -r takes a revset without a word, and clone leaves a symlink**. None is in the
  flow this cycle changes, and the ladder is long without them.
- "commit moves only the bookmark it is asked to", a candidate below, is the rung **feat: cycle
  --commit makes a rung**: the cycle's bookmark is the slug of the title in `## In Progress`, and
  `cycle --commit` advances that one alone.
- The rehearsal is not first, as the Todo entry had it, but after the four stages exist, so it is
  written once against `cycle`'s flags and not rewritten when the old forms go.
- The old forms are removed in one rung, after the rehearsal, and not as each replacement lands,
  so the cycle's own commits are made by commands that work until the new ones are rehearsed.

- Long output is a rule and a rung: the agent-files name the commands whose output the Bash tool
  cuts off and the form they run in, and `validate` goes quiet as the first work rung (wink,
  2026-11-11).
  - The names are the rule and "in doubt" its fallback, since a judgment is skipped when the agent
    is confident and wrong.
  - A `bash-guard.sh` deny of a named command run bare was weighed and held until the rule has
    been tried, a wrong deny list costing more than a wrong rule.
  - The rung was to be first, since every later rung runs validation several times, and was
    dropped later the same day, under Terms and scope below. The rule stands.
- The agent-files change inside this cycle, as drafts of the open v0.3.1 proposal: the opening is
  `v0.3.1-8` (wink, 2026-11-11), and the one later rung that edits them, the docs rung, is `-9`.

The candidates the Todo entry carried, each to be decided in the deliberation after the opening,
kept, changed, laddered as a rung, or filed back into `## Todo`:

- The rehearsal script, first, since it turns the testing by hand into a repeatable run and shows
  what else is broken.
- commit moves only the bookmark it is asked to: it advances the line's nearest bookmark, a guess
  that advanced `main` in `wt-21-6`. We think `vc-x1 open` records the cycle's bookmark, commit
  advances that one alone, and with none recorded nothing moves and commit says so.
- The closing says what it joins, its shape, and where it lands, wink's design (2026-11-10):
  `commit -s <b|w|p> --closing [REV,...] [--trapezoid <T>] [--land <T>] [--push] [--title ..]
  [--body ..]`.
  - `--closing` lists the heads joined, `@` when none is listed. The closing is always made from
    `@`, each head listed another parent, so a bushy tree closes in one merge. A tip elsewhere is
    reached by moving `@` there first, and the detour goes.
  - `--trapezoid <T>` makes it a trapezoid, `T` the first parent, and without it the closing is
    linear, a plain commit or a merge of the heads alone, which builds the linear shape and retires
    `--shape`. "Trapezoid" is Terminology's word, the flag naming what it makes.
  - `--land <T>` lands on `T` once everything is made, taking a value again, since a linear closing
    has no target of its own. `--trapezoid main --land main` repeats the name, explicit, and a
    mismatch is the Land's fast-forward check's to refuse. `--onto` leaves `commit`, kept by `vc-x1
    land --onto`, the destination, as `--land` is.
  - `--push` publishes the target and deletes the cycle's bookmark on `origin` with `--land`, and
    without it the closing on the cycle's bookmark, a draft. `-s`, `--title`, and `--body` are as
    now, the title and body read from an opening when there is one.
  - To decide: a single-step cycle whose opening is committed, finalized in place today, keeps that
    or gets a closing commit above it, and a linear closing whose target moved is refused before
    anything is made, naming `--trapezoid` as the way to join it.
  - It replaces the last cycle's required `--onto`, its refusal of `--land <value>`, and the
    questions of a bare `--closing` and of keeping `commit --land`.
- **push only publishes** and **validate-desc shows a note as part of its row**, in `## Todo`, as
  rungs if they fit.
- Found at the closing of **chore: complete vc-x1, ready for vc-x2**: `validate-desc -r` takes
  `::main` and scans one commit without a word, and `clone` into a scratch directory leaves a
  Claude Code symlink in `~/.claude/projects`.

#### Flow

(wink + agent, 2026-11-11) The cycle as a list of operations, wink's, with every sub-step we could
name, so each can be sorted into what `vc-x1` does and what is done by hand. The goal is simplicity
for users and agents, with the steps still separable, so we or others can change the flow. A draft
the deliberation revises, the questions it raises being in the list after it.

The mark on a sub-step says who does it:

- (tool): a `vc-x1` command does it today.
- (tool?): mechanical and done by hand today, a candidate for `vc-x1`.
- (hand): a judgment, the user's or the agent's, which stays by hand.

1. Open:
   - (tool) Sync both repos, `vc-x1 sync -s b`, and stop on a repo held back.
   - (hand) Fetch and read the messages repo, where a workspace has one.
   - (hand) Choose the `## Todo` entry and its cycle title, a Conventional Commit.
   - (tool) Create the bookmark on the work `@`, `vc-x1 open --title`.
   - (hand) Check each `## Waiting` condition and promote what is met.
   - (tool?) Delete what `## Closed` holds, and the reference definitions only it used.
   - (tool?) Move the `## Todo` entry into `## In Progress` as a scaffold: the title heading, the
     item headings, the entry's text under Problem, and the opening and closing rung subsections.
   - (tool?) Replace `zTOC.md`'s ladder with the opening, marked `(current)`, and the closing.
   - (tool?) Bump the version-of-record to the opening's suffix, and the lock file with it.
2. Deliberate and plan the cycle:
   - (hand) Write the problem, the solution, the acceptance check, and the deliberation.
   - (hand) Ladder the work rungs, each a title in `zTOC.md` and a subsection of intent.
3. Commit the cycle opening:
   - (tool) Validate, `vc-x1 validate`.
   - (tool) Install the dev build when the artifact changed, `vc-x1 install --profile dev`.
   - (hand) Review of the working-copy diff, "please review" and the user's go.
   - (tool?) Flip the opening's marker to `(done)`.
   - (tool?) Title and body: the cycle title plus " opening" and a pointer body, defaults the
     tool could supply as the closing's are, an explicit `--title` or `--body` overriding.
   - (tool) Commit the pair, `vc-x1 commit -s b --opening`, the trailers stamped.
   - (tool) Install the dev build again, from the committed tree.
4. Do the next work rung, the head of the loop:
   - (tool) Sync, optional between the bookends.
   - (tool?) Mark the rung `(current)` with its suffix, and bump the version-of-record to it.
   - (hand) Do the work.
   - (hand) Write the rung's subsection, what changed conceptually.
5. Test and review the work rung:
   - (tool) Validate, `--fast` while a review iterates and full before the commit.
   - (tool) Install the dev build.
   - (hand) Test, by the user, the agent, and others, each running the dev build.
   - (hand) Review, and back to step 4 until the user's go.
6. Commit the work rung:
   - (tool?) Flip the marker to `(done)`.
   - (hand) Title and body, the rung's own problem and solution.
   - (tool) Commit the pair, `vc-x1 commit -s b`.
   - (tool) Install the dev build again.
7. Optionally push the rung, a draft on the cycle's bookmark:
   - (hand) The user's approval of this push.
   - (tool) `vc-x1 push -s b`.
8. Loop to step 4 while a rung is left. Unplanned work enters here:
   - (hand) The user picks a rung inserted into the ladder or a `## Todo` entry.
   - (hand) Edits already in `@` are set aside as a jj commit with a local bookmark.
9. Commit the cycle closing:
   - (tool) Sync, and stop when a `main` moved.
   - (hand) Run the acceptance check and record pass or fail.
   - (hand) Finalize the record: the title, the solution as done, the design subsections, the
     closing rung's subsection, and what must outlive the cycle written into `notes/`.
   - (hand) Write the continuation notes.
   - (hand) Update `notes/README.md` when functionality changed.
   - (tool) Record the agent-files' size when one changed, `vc-x1 agent-files size`, its note by
     hand.
   - (hand) Decide the parents: the heads the closing joins, and the target as first parent when
     it is a trapezoid, a linear closing having one parent.
   - (tool?) Mark the closing `(current)` and bump the version-of-record to the bare version.
   - (tool) Validate, full.
   - (hand) Review.
   - (tool?) Flip the marker to `(done)`.
   - (tool) Title and body: read from the opening, " opening" changed to " closing", the body
     from `[closing] body`, when `--title` and `--body` are not given.
   - (tool) Move the block to `## Closed` and make the closing commit, `vc-x1 commit -s b
     --closing <BM|REV> --onto <BOOKMARK>`.
10. Test the cycle:
    - (tool) Validate the closed tree, and install its dev build.
    - (hand) Test, by the user, the agent, and others.
    - (hand) A fault found here goes back to step 4, and the closing is made again.
11. Optionally push the cycle, still a draft on its bookmark:
    - (hand) The user's approval, then (tool) `vc-x1 push -s b`.
12. Land on the target, `main` or another bookmark:
    - (hand) The user's go.
    - (tool) Sync, and stop when the target moved, the closing then made again and tested again.
    - (tool) Commit the agent `@` as the partner of the work commits not yet covered.
    - (tool) Fast-forward the target and delete the cycle's local bookmark, `vc-x1 land -s b
      --onto <BOOKMARK>`.
13. Optionally push the target, which publishes the cycle:
    - (hand) The user's approval, the closing words said first.
    - (tool) `vc-x1 push -s b --bookmark <BOOKMARK>`, or by the user after the session,
      `vc-x1 squash-push -s b`.
    - (tool) Delete the cycle's bookmark on `origin` when it was ever pushed, `vc-x1
      delete-bookmark`.
14. After the Land:
    - (tool) Build, smoke check, and install the release build, `vc-x1 install`.
    - (hand) Stop, and the user restarts the agent.

A dead end leaves the loop at any rung for step 9, its closing `--dead-end`.

What the list raises, each to be decided:

- The way back from step 10: a fault found after the closing has nowhere to go in wink's list.
  Decided (wink, 2026-11-11): the user chooses, at the fault, and the flow prescribes none.
  - Reopen: the closing is dropped, a fix rung added, and the cycle closed and tested again.
  - Fix on top: a rung above the closing, then a second closing.
  - Fix inside: the fix folded into the closing commit.
  - Something else, the user's: the test itself may be what is wrong.
  - So the rule is to stop and ask, and the tool's part is to refuse none of them. The rehearsal
    tries each, and a reopen by `vc-x1`, in place of `jj abandon` by hand, is a candidate.
- Step 10 tests what lands only when the Land is a fast-forward. Decided (wink, 2026-11-11): a
  fast-forward is the Land's default, and a merge at the Land is allowed when the user asks for it.
  - By default a destination that moved is refused, the refusal naming the ways on: a merge at
    the Land, the closing made again, a rebase, or the user's own.
  - A merge at the Land makes a commit that step 10 did not test, which the user accepts by
    asking for it. Whether to test again is theirs to decide.
- The parents are one decision, settled by wink's N and M under Terms and scope below: "other
  commits to incorporate" and "the list of parents" are both the parent list, each head joined a
  parent, which is wink's `--closing [REV,...] --trapezoid <T>`.
- The acceptance check and step 10 are one test at two moments. Decided (wink, 2026-11-11): the
  acceptance check runs as the first thing in closing, and again as the last thing after it.
  - Every work rung has its own test and no more, the acceptance check being the closing's. It
    replaces the wording first decided, the last work rung's test including the check, since one
    rung was then unlike the others.
  - The first run gives the closing commit a result to record. A check that runs only mid-ladder
    is still run by a rung and recorded by the closing.
  - The second run is step 10, wink's "test cycle", on the closed tree: the first run on that
    tree when a merge or a rebase brought the destination's changes in, and otherwise a
    confirmation the user may skip.
- How much of step 1 `vc-x1 open` does. Decided (wink, 2026-11-11): the three record steps, by
  default, beside the bookmark it makes today.
  - The three: what `## Closed` holds is deleted, the `## Todo` entry titled as `--title` is moved
    into an `## In Progress` scaffold, an empty one when none matches, and `zTOC.md`'s ladder is
    replaced with the opening and the closing.
  - They are the cycle's record, whose shape the agent-files define, so the tool knows them
    without knowing the work. We think a flag turns them off and a workspace without the files
    gets the bookmark alone, and the dry run shows the edits, `open` editing files where it made
    only a bookmark.
- The version bump is never the tool's. Decided (wink, 2026-11-11): `vc-x1`, and `vc` after it,
  does not bump the version, now or later, the version being the project's and out of our scope.
  - It replaces the agent's split, the tool computing the version and a configured command
    writing it, which had the tool doing the bump by another hand.
  - The project automates its own. Ours is an xtask, a candidate rung: `toml_edit` keeps a
    manifest's comments, layout, and key order and changes the one value, as cargo's own `cargo
    add` does, where the usual `toml` crate loses them. It is in our build through jj-lib, so it
    adds no download. The xtask refreshes `Cargo.lock` in the same step, the build being
    `--locked`. **Build config moves to an xtask** in `## Todo` is the same crate.
  - The tool notices: a rung committed with the version unchanged is refused unless the user says
    so, wink's idea. Decided (wink, 2026-11-11): the config names a file and a pattern, ours
    `Cargo.toml` and its `version = ` line, and `commit` refuses a work commit whose diff changes
    no line the pattern matches.
    - A `version.md` of its own, wink's first form, was set aside: cargo wants the version in
      `Cargo.toml`, so the check would be on a copy something must keep equal. A project with no
      manifest names its `version.md` as the file, the same key.
    - A named flag commits anyway, `--same-version` or the like, and not `--yes`, which on `push`
      is the user's approval of a forced push and would become a switch for any question.
    - The check is the work commit's alone, the agent-repo having no version, and without the
      key nothing is checked.
- Deliberation comes before the opening commit. Decided (wink, 2026-11-11): by default, as in
  this cycle, so the opening carries the plan, and the user is free to commit the setup first.
  - The opening's review is then of the plan, before any work, where a wrong direction costs the
    least. The plan stays provisional, later rungs revising it.
  - The cost is a working copy uncommitted while the plan is talked through, the gap **Work in
    progress is durable and continues on any machine** in `## Todo` names. The opening is
    committed when the plan is good enough to start.
  - A small cycle's Deliberation reads `_None._`, as the format allows.
  - The Cycle Protocol's Opening gains the step between the scaffold and the commit, and stops
    calling the opening setup.
- One shape or two. Decided (wink, 2026-11-11): every cycle has an opening and a closing, and in a
  single-step cycle the closing is squashed into the opening, the one commit's title losing its
  trailing " opening" and gaining no " closing".
  - The tool knows a single-step cycle from a multi-step one and does the right thing for each,
    the user giving `--title`, `--body`, both, or neither.
  - It keeps today's finalize in place, and sets aside the agent's proposal that a closing is
    always a new commit. The agent had called the finalize in place the case that surprised wink
    in `wt-21-6`, which nothing in the record says, the three faults there being fixed by **feat:
    open makes the cycle's bookmark first**.
  - The tool's test is the count, as Cycle shape has it: no commit or one above the destination
    is single-step, the work in `@` folded in, and more is a ladder, whose closing is its own
    commit titled " closing". Decided (wink, 2026-11-11), taken at the "go" for the opening.
- Two kinds of push: steps 7 and 11 publish the cycle's bookmark, a draft, and step 13 publishes
  the destination and removes the draft. The names "draft push" and "publish" are reasonable to
  wink (2026-11-11), who was thinking of the naming the next bullet holds.
- A `cycle` command, wink's (2026-11-11): `vc-x1 commit` is very overloaded, and `cycle --open`,
  `cycle --opening`, `cycle --closing`, and `cycle --land` take the cycle's acts out of it. The
  agent's thoughts, none decided:
  - It is the right direction: `commit` makes a rung, an opening, a closing that is a merge or a
    rewrite in place, moves the record, lands, pushes, and describes an older commit, and most of
    the flow's surprises are in how those flags combine. `commit` goes back to committing the
    pair with its trailers.
  - Subcommands and not flags, `cycle open`, `cycle close`, `cycle land`: verbs that exclude each
    other are subcommands, each with its own options and help, the closing's parents `close`'s
    alone.
  - Composition is "this stage and onward", the stages having one order: `cycle close --land
    --push`.
  - No "opening" verb: `--open` and `--opening` differ by three letters and mean two things. The
    tool sees that a commit is the cycle's first, so `cycle commit` makes a rung and the first is
    the opening, as it knows a single-step cycle.
  - The verbs are then `open`, `commit`, `close`, `land`, and `status`, the last saying where the
    cycle stands and what may come next.
  - The two pushes are named by it: a push of the cycle's bookmark is the draft push, and the
    Land's push is the publish.
  - It is **vc-x2: a cycle command, open, close, and land** in `## Todo`. Two spellings of one act
    being worse than either, it is done here whole, the old forms removed in this cycle, or left
    to vc-x2. The agent leans to here: the rehearsal is written once, and the Cycle Protocol is
    revised once.
  - Decided (wink, 2026-11-11), the agent's recommendations taken with one change:
    - No subcommands in vc-x1: every command is `command --options`, the form the commands were
      brought to earlier for consistency, and vc-x2 reevaluates it. So the verbs are mode flags,
      `cycle --open`, `--commit`, `--close`, `--land`, and `--status`.
    - The opening is inferred, the first commit of a cycle, and there is no `--opening`.
    - It is built in this cycle, and the old forms go in it: `open` and `land` are removed, and
      `commit` loses `--opening`, `--closing`, and `--land`, staying the plain commit of the pair
      for work outside a cycle.
    - The parser declares which flags exclude each other and which option needs which flag, so
      no refusal is written by hand. Consecutive stages chain in their order, `cycle --close
      --land --push`, a stage's flag carrying its value, `--close` the heads joined and `--land`
      the destination. The help is one text, its options grouped under a heading per stage.
    - It ports to vc-x2 as it is, `cycle --close` becoming `cycle close` if subcommands return.
    - Noticed: `vc-x1 agent-files copy`, `diff`, and `size` are subcommands today, an exception
      to the form still standing, left as it is by this cycle.
- The markers, at steps 3, 4, 6, and 9. Decided (wink, 2026-11-11): the tool manages `(done)` and
  `(current)`, and nobody types or flips one. The bump beside them in the list is the project's,
  decided above.
  - The ladder stays markdown: `zTOC.md`'s purpose is that it is clickable, wink using it to jump
    to a rung in `TODO.md`, so a `toml` block, the agent's proposal, is out.
  - The line's grammar is strict and the tool enforces it: under `## Ladder`, to the next `## `
    heading, each `- ` at the left margin is a rung, an optional number and `(current)` or
    `(done)`, then the linked title, a single-step cycle's one rung unlinked. A line that does not
    fit is reported and nothing is edited.
  - A rung is written once, as its heading under `#### Rungs` in `TODO.md`, by the user or the
    agent, the planning being theirs. The tool makes the ladder from the headings, in their
    order, each link the title's slug, so the two places cannot differ. `open` writes the first
    two, the opening and the closing.
  - The markers are computed and written: a rung is `(done)` when a commit of its title is on the
    cycle's branch, and `(current)` is the first without one. `(done)` stays written on the line,
    so a finished ladder, the one the landmark keeps, says so, and a ladder not started does not
    read the same. Position is a check and not the record: a done rung below the current one is
    reported.
  - The tool rewrites the ladder as part of each cycle commit, so `zTOC.md` at a commit shows the
    cycle as of it.
  - Every commit in the cycle matches a rung in `TODO.md`, or the tool errors (wink): a coding
    requirement is a higher bar than a naming protocol, and the naming is now one. It replaces
    the agent's "reported and committed anyway" for a title not in the ladder, so an unplanned
    rung gets its heading before its commit. To check: a local ladder's intermediate commits,
    which are on the branch until they collapse.
  - The rung's number is its position, which needs no configuration and no version, and the
    cycle's destination is named on the command each time, as today, and not stored, the bookmark
    being the title's slug. Decided (wink, 2026-11-11), the agent's answers taken at the "go" for
    the opening.
- The opening's title and body default as the closing's do. Decided (wink, 2026-11-11), each
  default giving way to a `--title` or `--body` given:
  - The opening: the cycle title plus " opening", and a body from the config, as the closing's
    `[closing] body` is.
  - A ladder's closing: the cycle title plus " closing", and `[closing] body`, as today.
  - A single-step cycle's one commit: the bare cycle title, and a body that is required, the
    commit holding the work, which a pointer body says nothing about. A named waiver commits
    without one.
  - A work rung: both required.
  - The cycle title is read from the `### ` heading under `## In Progress`, which `open` writes,
    where the closing reads the opening commit's title today, so a single-step cycle with no
    opening commit has one too.

Terms and scope (wink + agent, 2026-11-11), raised before the questions above were gone through,
each to be decided, and the list's marks are revised by what is:

- Land is a fast-forward: `vc-x1 land` is jj and git's fast-forward merge, wink's point, and a
  regular merge is the more common case where several commits join at once.
  - A regular merge is made today, at the closing: a trapezoid closing is a merge commit of the
    target and the ladder. Git's `merge` makes the merge commit and advances the branch in one act,
    and we split them, the closing making the merge and the Land advancing the destination to it,
    which is what lets the merged tree be tested before the destination moves.
  - Every integration ends in a fast-forward of the destination, whether the commits were prepared
    by a merge, a rebase, or nothing. "Fast-forward" names the bookmark's move, and merge or rebase
    names how the commits were prepared.
  - We think "Land" stays the name of the moment a cycle becomes permanent, its mechanism said
    plainly to be a fast-forward, and the general case is "prepare, then advance".
- The destination of a Land is any bookmark (wink): `main` is the usual one and never assumed, in
  the commands, the rules, or this record. The agent's thoughts above read as if a trunk were the
  destination, and are to be read with any bookmark in its place.
- A closing merges a list of commits (wink): in the general case the merge has N direct parents,
  the commits merged, and a set of M original parents, which defines the trapezoid.
  - The N are the heads the closing joins, the cycle's tip and any other line taken in, and the M
    are the commits those lines started from on the destination's side, one when every line
    forked from the destination's tip, so the figure is the M below, the N lines between, and the
    merge above. Confirmed by wink, and the closing's flags follow from it.
- Rebase is supported (wink): unmet so far in our single-change, linear use. It has two places,
  mid-cycle to bring a cycle up to a destination that moved, and at the closing as how a linear
  shape takes one. Decided (wink, 2026-11-11): not in vc-x1, it is vc-x2's, filed as **vc-x2:
  cycle --rebase moves a cycle onto a destination that moved** in `## Todo`. In vc-x1 a rebase
  is `jj rebase` by hand.
  - The three preparations: nothing, linear as committed and refused when the destination moved,
    merge, the trapezoid, which joins a destination that moved, and rebase, linear, the ladder
    replayed on the destination.
  - A rebase rewrites every rung, so the cycle is tested again. The `ochid:` trailers survive,
    being change ids.
  - A draft that was pushed needs a forced push, under the approval any push takes.
- The flow makes its own conflicts: every cycle edits `TODO.md`, `zTOC.md`, and the version in
  `Cargo.toml`, so two cycles in flight conflict in all three at any merge or rebase, whatever the
  code did. One cycle at a time has hidden it. We think the record and the version being in shared
  files hurts before the choice of merge or rebase does. Decided (wink, 2026-11-11), taken at the
  "go" for the opening: not this cycle's, vc-x1 running one cycle at a time, and filed as **Two
  cycles in flight conflict in the record and the version** in `## Todo`.
- The work is not ours (wink): validate, test, build, run, and install are specific to a language
  or to the work, and we manage cycles, not the work. In vc-x2 they are not in `vc`, which knows
  nothing about the work and helps get it done, organized, and its history kept.
  - The Flow needs a fourth mark, (work), the project's: validation, the dev install, testing, and
    the release install. The version bump is one too, since doing it in the tool puts Cargo
    knowledge there. The ladder's markers are ours, and the version is the project's.
  - We think the boundary is a hook: the tool knows that a gate is at a point in the cycle, not
    what it runs, runs the command the project configured, and reads its exit status. `validate`
    is nearly that, running lists from the config, and build, install, run, the profiles, and the
    dev suffix are what is specific to the work.
  - The rung **feat: validate prints only what failed** adds to a command that is to go. Decided
    (wink, 2026-11-11): `validate` is kept as it is for now and removed in vc-x2, so the rung is
    dropped. The Long output rule covers its output, and `[validate]` in the config is left as
    it is, `-q` on its cargo lines not taken up.
  - In this cycle nothing is removed from vc-x1: those commands gain nothing, and the flow is
    designed so their steps plug in. A rehearsal workspace in `~/tmp` holding no cargo project
    tests "knows nothing about the work" directly.

#### Rungs

The rung subsections, in the ladder's order. The ladder itself is in `zTOC.md > ## Ladder`.

##### feat: improve vc-x1 cycle flow opening

The cycle's setup commit: create the bookmark, delete `## Closed`'s contents, move the Todo entry
into this block, and bump the version-of-record. The work rungs are laddered by the deliberation
that follows it.

- The opening's own validation was run twice, its output cut off before the line that said it
  passed, so the opening carries the answer wink asked for, the agent-files at `v0.3.1-8`.
  - Working practices gains Long output: a named command keeps its output in `tmp/last.log` and
    shows its tail, a pass is the exit status, and a failure is read from the file.
  - `custom.md` names this workspace's commands, the set naming no project.
  - A first work rung, **feat: validate prints only what failed**, was laddered and then dropped
    by the deliberation, `validate` being kept as it is until vc-x2 removes it.

##### feat: cycle --open makes the bookmark and record

Opening a cycle is a bookmark by `vc-x1 open` and three edits by hand, two of them made this cycle
by deleting line ranges by number. `cycle --open --title` makes the bookmark and, by default, the
record: what `## Closed` holds is deleted, the `## Todo` entry of that title is moved into an `##
In Progress` scaffold, an empty one when none matches, and the opening and closing rung headings
are written. A flag turns the record steps off, a workspace without the files gets the bookmark
alone, and the dry run shows the edits.

- `cycle` is a new command beside the old ones, which stay until **feat: the old cycle commands
  are removed**, so `vc-x1 open` and `cycle --open` both make the bookmark, by the same code.
  - The stage is a required flag, the group every later stage joins, and an option that is one
    stage's alone requires that stage's flag, so the parser makes the refusal.
- The record's edits are text in and text out, in `src/cycle/record.rs`, each file's new text
  planned before the bookmark is made.
  - A cycle in progress, a `TODO.md` with no `## In Progress`, and a `zTOC.md` with no `## Ladder`
    are refused with nothing made, and a name in use is refused before a file is written.
  - `--no-record` is the flag, and `--bookmark` with no `--title` writes none either, there
    being no title to write it under.
- Left by hand, each said in the README: the reference definitions only `## Closed` cited, which
  `validate-anchors` names, and an entry's own `####` headings, which arrive under Problem as
  they were.
- The scaffold's unwritten items read `_To write._`, the agent's choice, since a blank item
  reads as an omission.
- An entry is found by its exact title, so a `## Todo` entry is retitled as a Conventional Commit
  before the open, as this cycle's was, or the scaffold is empty and the entry stays.

##### feat: the ladder is made from the rung headings

A rung is typed twice, a line in `zTOC.md` and a heading in `TODO.md`, and its markers are flipped
by hand. The tool makes `zTOC.md > ## Ladder` from the headings under `#### Rungs`, in their order,
each link the title's slug, each number its position, and each marker computed, `(done)` when a
commit of that title is on the cycle's branch and `(current)` the first without one. The line's
grammar is strict: a left-margin bullet that does not fit is reported and nothing is edited, and a
done rung below the current one is reported.

- `cycle --ladder` is a stage the deliberation did not name, the agent's addition: the planner
  writes the rung headings and then needs the ladder made before any commit rewrites it. For wink
  to keep or fold into `--status`.
- The cycle's branch needs no destination: it is the commits the cycle's bookmark reaches, and
  those above it up to the work `@-`, that no other local bookmark reaches.
  - A second local bookmark at or above the cycle's first commit hides the commits below it, the
    cost of naming no destination. `--bookmark` names the cycle's when it is not the title's slug.
  - A working copy not yet described is not a commit of the cycle.
- A rung not started has neither number nor marker, as the format had it, the number being the
  position only once the rung is current.
- Refused, nothing edited: a commit that is no rung's, a rung written twice, a ladder without its
  bookends, and a line already in the ladder that the grammar does not fit, any line but a blank
  one and the skeleton's `_None._`. So a note typed into the ladder is named and kept, where a
  rewrite would have lost it.
- The check that every commit is a rung's reads the branch up to `@-`, so a local ladder's
  intermediates refuse `--ladder` until they collapse, which is when the rung commits. That
  settles "To check" in the Flow above, by the agent: the collapse comes before `cycle --commit`.
- Tried on this cycle's own ladder, which it made as it was by hand, one marker aside.

##### feat: cycle --commit makes a rung

`commit` makes a rung, an opening, a closing, a Land, and a describe, and guesses the bookmark to
advance. `cycle --commit` makes a rung and nothing else: the first commit of a cycle is its
opening, with the title and body defaulted, a work rung gives both, the ladder is rewritten into
the commit, and the cycle's own bookmark, the slug of the title in `## In Progress`, is the one
advanced. A commit whose title matches no rung heading is an error, a local ladder's intermediates
being the case to settle.

- The commit is `vc-x1 commit`'s own code, called with the title, the body, and the bookmark to
  advance named, so the pair, the trailers, and the rollback are one implementation.
- The opening is the commit made when the branch has none, and a `--title` given then is refused
  unless it is the opening's, since a first commit under a work rung's title would leave the
  opening never done.
- The opening's body is a new config key, `[opening] body`, as the closing's is `[closing] body`,
  and a workspace without it gets a built-in body, so a workspace needs no key to run a cycle.
- `-s` is defaulted, the agent's choice: `b` in a dual workspace and `p` in a plain repo, since
  the flow commits a pair by default and the workspace says which it is. For wink to keep or
  make required, as it is on `commit`.
- The ladder is written before the commit and put back when the commit is not made, and an
  empty work `@` is refused first, since the ladder's own edit would otherwise make an empty rung
  look like a change.
- `--push` is the draft push of the rung, in the same invocation.
- A local ladder's intermediates were settled in the last rung: they collapse before `cycle
  --commit`, which refuses a branch holding a commit that is no rung's.
- This rung's own commit was made by `cycle --commit`, from the dev build.

##### feat: cycle --close makes the closing

The closing is `commit --closing <BM|REV> --onto <BOOKMARK>`, its parents and its shape implied.
`cycle --close [REV,...]` lists the heads joined, `@` when none is listed, and `--trapezoid <T>`
makes the destination the first parent. A single-step cycle's closing is squashed into its
opening, the title losing " opening" and the body required. A ladder's closing is its own commit,
its title and body defaulted. It moves the block to `## Closed`, and the acceptance check is the
first thing in closing, by hand.

- The closing is always made from the work `@`, which must be on the cycle's tip, so the detour
  `commit --closing` made around a tip elsewhere is gone: `jj new <bookmark>` puts `@` there.
- The count is of the cycle's branch, read as the ladder reads it, so the shape needs no
  destination, and only a trapezoid names one.
- A ladder's closing is linear unless `--trapezoid` or a head listed makes it a merge, which is
  the linear shape built, where `commit --closing` always merged. The agent-files still say a
  ladder lands as a trapezoid, so until **docs: the Cycle Protocol follows the cycle command**
  decides the default in the rules, this repo's closings pass `--trapezoid`.
  - The parents are the destination first, then `@`'s own, then each head listed, none twice.
- The single-step waiver is `--default-body`, the agent's name for it: the body is `[closing]
  body`, the pointer. `[closing] body` absent is no longer a refusal, a built-in body taking its
  place, as the opening's does.
- A `--title` given is the commit's title as written, not the bare cycle title, so no suffix is
  appended to it and none is refused.
- `--dead-end` is kept, always a closing commit.
- The ladder and the record move are written before the commit and put back when it is not made.
- After the closing `## In Progress` is empty, so `cycle --ladder` and `cycle --commit` have no
  cycle to read, and the Land reads the branch and not the record.
- Not done: the old closing's `--land` in the same invocation, which is the next rung's chain.

##### feat: cycle --land advances the destination

The Land is `vc-x1 land -s b --onto <BOOKMARK>`, a fast-forward that refuses a destination that
moved and says only that. `cycle --land <DEST>` fast-forwards by default, and its refusal names
the ways on: a merge at the Land, asked for by a flag, the closing made again, or a rebase by `jj`.
`--push` publishes the destination and deletes the cycle's bookmark on `origin`, and the stages
chain, `cycle --close --land --push`.

- The Land is `vc-x1 land`'s own code, called with the source and the destination named, so the
  agent commit, the trailers' update, and the bookmark's deletion are one implementation.
- The cycle landed is found without a flag: the title `## Closed` holds names its bookmark, the
  closing having moved the record there, then `--bookmark`, then the one local bookmark besides
  the destination. A cycle in `## In Progress` is refused, a Land being of a cycle closed.
- `--merge` is the flag. The merge is a pair's commit, made by the commit code from an empty
  work `@`, titled the cycle title plus " merge", the agent's name for it, and the cycle's
  bookmark is advanced to it, so the Land that follows is the ordinary fast-forward.
  - A conflicted merge is undone and refused. With the record and the version in shared files,
    **Two cycles in flight conflict in the record and the version** in `## Todo`, we think most
    merges at a Land will be, so `--merge` is of use mainly once that is settled.
  - `commit` gained an internal `allow_empty`, since a clean merge changes nothing its parents
    do not hold, and the closing's dry run uses it too, writing no record.
- In the chain the Land's checks of the destination are made before the closing, and the push
  is the Land's, the closing making none.
- `-s w` is refused with `--land`, as `land` refuses it.
- Not done: `--delete-intermediate-bookmarks`, which `vc-x1 land` has for a stack of cycles
  landing together, and a Land by a revision. Both are `vc-x1 land`'s until it is removed, and
  the removal rung decides whether `cycle --land` takes them.

##### test: a rehearsal runs both cycle shapes

The flow is tested by hand, in `~/tmp` workspaces wink builds. A script makes a dual workspace
with local remotes and no cargo project, and runs a single-step and a ladder cycle through
`cycle`'s flags, open to Land, checking what each command left. It tries each way back from a
fault after the closing, reopen, fix on top, and fix inside, and reports the ones the tool
refuses. It is the cycle's acceptance check.

- `scripts/cycle-rehearsal.sh [DIR]`, `~/tmp/cycle-rehearsal` by default, the binary `VC`,
  `vc-x1-dev` by default. The workspace is `vc-x1 init --test-repo`'s, so nothing leaves the
  machine, and the script removes the Claude Code symlink `init` makes for it.
- Three cycles, each landed and published to the local remotes:
  - A single-step cycle with nothing committed, closed and landed in one invocation, `cycle
    --close --body --land main --push`.
  - A single-step cycle whose opening is committed first, its closing squashed into it by
    `--default-body`, then landed.
  - A ladder of two work rungs, its `## Todo` entry moved by the open, a draft push between,
    closed as a trapezoid, and landed.
- 48 checks, all passing on its first run (2026-11-11): the bookmarks on both sides, the scaffold
  and the ladder's markers, each title, the closing's parents, the trailers, both `main`s
  published, and the work `@` empty.
- The ways back from a fault after the closing, as found:
  - Fix inside: `vc-x1 squash -s b` folds the fix into the closing, which stays a merge of two.
  - Reopen: by `jj`, the bookmark set back to the last rung, the closing abandoned, and `@` put
    on the tip. The record is then in progress again, the abandoned commit having carried the
    move, so a fix rung commits by `cycle --commit` and `cycle --close` closes again. It leaves
    the abandoned closing's agent commit in the agent-repo, harmless and unreferenced. A `cycle
    --reopen` in place of the three `jj` commands is a candidate, not built.
  - Fix on top: refused. After the closing nothing is in progress, so `cycle --commit` has no
    cycle to commit a rung of. A plain `vc-x1 commit` above the closing works, and a second
    closing has no command. For wink: whether fix on top is a way the tool should serve.
- Also refused, as it should be: a commit whose title is no rung's.
- Found by it: `cycle --close` prints `commit`'s warning that a title ends " closing", meant for
  the old flags, which goes with them in **feat: the old cycle commands are removed**.
- Not in `vc-x1 validate`: it takes about ten seconds and writes under `~/tmp`, so it is run by
  hand, at the closing's two ends.

##### feat: cycle --status says where the cycle stands

Nothing says where a cycle stands or what may come next. `cycle --status` prints the cycle's
title, its bookmark, the ladder with its markers, the commits not yet pushed, and the stages open
from here.

- It reads and changes nothing, so what another stage refuses, a commit that is no rung's or a
  bookmark not found, is a line here and no failure.
- The ladder shown is the one the rungs and the commits make, not `zTOC.md`'s, with a note when
  the two differ, so `--status` is also the check that `--ladder` would change something.
- `next` names one stage, the one the cycle is at, with the current rung's title filled in, the
  agent's reading of "the stages open from here": a list of every stage allowed was longer and
  said less.
- Between cycles it says whether the cycle `## Closed` holds is landed, by whether its bookmark
  is still in the repo.
- `--ladder` stays its own stage, since `--status` writes nothing.
- Not shown: the agent-repo's side, and the destination, which is named on a command and not
  stored.

##### feat: commit refuses an unchanged version

A rung committed without its version bump is found by reading, if at all. The config names a file
and a pattern, ours `Cargo.toml` and its `version = ` line, and a work commit whose diff changes no
line the pattern matches is refused. A named flag commits anyway, and without the key nothing is
checked. The tool reads the version's line and never writes it.

- The keys are `[version] file` and `[version] pattern`, both or neither, in the work side's
  config, and this workspace's config has them.
- The pattern is what the line starts with, not a regular expression: a line is the version's
  when it starts with the pattern at the left margin, so `clap = { version = "4" }` is not, and no
  regex is added to the config's vocabulary.
- The check is of the commit's own diff, the lines it removes and adds in the file, so a bump
  made and then undone inside one working copy is no bump.
- It is in `vc-x1 commit` for every work commit, a draft described by `-r` included, and so in
  `cycle --commit` and `cycle --close`, which also check before the ladder or the record is
  written. The flag is `--same-version` on both commands.
- The agent-repo's commit is not checked, having no version, and a merge at the Land is not,
  being no rung.
- The closing is checked like a rung, since the closing bumps to the bare version.

##### feat: an xtask bumps the version

The version is bumped by hand, in `Cargo.toml`, with `Cargo.lock` following at the next build. An
xtask sets it, `toml_edit` keeping the manifest's comments and layout, and refreshes `Cargo.lock`
in the same step, the build being `--locked`. It is the project's own tool, outside `vc-x1`, and
the repo becomes a cargo workspace for it.

- `cargo xtask bump <next|bare|VERSION>` and `cargo xtask version`, the alias in
  `.cargo/config.toml`, the package in `xtask/`.
  - `next` adds one to the suffix's last number and gives a bare version `-0`, `bare` drops the
    suffix, and a version named is taken as written when it has one `-` at most and no `+`, the
    suffix scheme's shape.
  - A sub-cycle's suffix, `-3.1`, bumps its last number, `-3.2`.
- `Cargo.lock` is refreshed by `cargo update --workspace --offline`, which touches the
  workspace's own packages alone, so no dependency moves. This rung's own first build failed on
  the stale lock before the xtask existed, a `--locked` build after a bump by hand.
- `toml_edit` is at the version jj-lib already brings into the lock file, so the bump added no
  download.
- The workspace's default members are both packages, so `vc-x1 validate` formats, lints, and
  tests the xtask with the rest, and a release build compiles it too, a second or so.
- The root's `clippy.toml` bans spawning a process, for `vc-x1`. The xtask's one spawn, of
  cargo, carries an `allow` with its reason.
- Not done: the bump as a step some stage runs. The tool never bumps, so the order is the
  user's or the agent's: `cargo xtask bump next`, then the work, then `cargo xtask` is not asked
  again until the next rung. We think a `[cycle]` hook, a command the project names for the
  tool to run before a commit, is the vc-x2 form.
- **Build config moves to an xtask** in `## Todo` now has its crate to move into.

##### feat: the old cycle commands are removed

Two spellings of one act are worse than either. `open` and `land` are removed, and `commit` loses
`--opening`, `--closing`, and `--land`, staying the plain commit of the pair for work outside a
cycle. The README and the tests follow, and **vc-x2: a cycle command, open, close, and land** in
`## Todo` retires at the close-out.

- `open` and `land` are no longer commands, and each still parses, hidden, so its refusal names
  the stage that took its place, `cycle --open` and `cycle --land <BOOKMARK>`. `commit`'s three
  flags are refused the same way. `--onto`, `--dead-end`, and `--skip` on `commit` are gone
  outright, each having needed one of the three.
- What was removed from `commit` is the closing's own code, which `cycle --close` replaced and
  does not call: the detour around a tip that is not `@-`, the closing's plan and its three
  shapes, the title read from the opening commit, and the Land run from inside a commit. The
  file went from 3911 lines to 1933.
  - Kept, as `cycle`'s: the pair's commit, the squash into an opening, the record's move, and the
    version check. `title` and `body` are required again, no closing deriving them.
- `src/open.rs` and `src/land.rs` stay as the code `cycle` calls, less their argument parsers.
- `cycle --land` took `--delete-intermediate-bookmarks`, which the last rung left with `vc-x1
  land`. A Land by a revision, `land --source @-`, has no flag: `--bookmark` names the cycle's
  bookmark, and a cycle with none is found as the one local bookmark besides the destination.
- Tests: 29 of `commit`'s tests of the old flags went with them, their behavior being `cycle`'s
  tests' and the rehearsal's, and `land`'s and `open`'s build their params without a parser.
  - Not carried over: the detour's tests, there being no detour, and
    `the_fixture_lands_by_three_lines`, which ran the old three-command Land on the fixture.
- `commit`'s warning of a title ending " closing", which `cycle --close` set off, went with the
  flag it pointed at.
- The agent-files and the skeleton `TODO.md` that `vc-dr-apply.sh` ships still name the old
  commands, and are the next rung's.
- The installed `vc-x1`, 0.85.21, still has the old commands until this cycle lands and its
  release build is installed, so the rungs here are committed by `vc-x1-dev cycle --commit`.

##### docs: the Cycle Protocol follows the cycle command

The agent-files describe the flow the old commands made. The Cycle Protocol and `jj.md` are
revised to `cycle`'s flags and to the deliberation's decisions: deliberation before the opening
commit, the acceptance check at both ends of the closing, the ways back from a fault as the
user's choice, the two kinds of push, the Land's default and its merge, the ladder as the tool's,
and the version as the project's. The agent-files version moves with it.

- The agent-files are `v0.3.1-9`, a draft of the open v0.3.1 proposal, adopted locally by
  `custom.md`. Only this rung edited them after the opening, so the draft moved once, which
  settles "To decide" in the Deliberation.
- `AGENTS.md`, by section:
  - Terminology: a cycle's commits are `vc-x1 cycle`'s, the target is named on the command each
    time and stored nowhere, and "draft push" and "publish" are defined.
  - Rules: two new, the ladder is the tool's and the version is the project's, and Commit by the
    tool names `cycle --commit` and `--close`.
  - Cycle shape: every cycle has an opening and a closing, the count saying whether they are one
    commit, and the defaults of each title and body.
  - Opening: `cycle --open` writes the record, and a step is added, deliberate and plan, before
    the bump, the opening carrying the plan.
  - The per-rung flow: Mark current and Flip are gone, the tool's, so the steps are sync, bump,
    work, subsection, validate, review, describe, commit, and the review iterates over 2 to 5.
  - Close-out: the acceptance check first, a new step, test the closed cycle, where a fault's
    way back is the user's choice, and the Land's refusal of a target that moved with its ways on.
- A ladder still closes as a trapezoid in the rules, by `cycle --close --trapezoid <target>`,
  and linear is said to be the user's to ask for, the agent's reading of wink's flag: the tool's
  default is linear, and the rules' default is the trapezoid. For wink to confirm or turn over.
- `jj.md`: Create, Close, and Land name the stages, the Land's sequence keeps its five steps, and
  the linear shape is described as built.
- `notes.md`: a rung's number is its position, no longer its version suffix, the markers are
  computed, and the ladder's grammar is said to be strict.
- `rationale.md` gains the why of each change under the mirrored heading, dated 2026-11-11, and
  retires `--onto` under Terminology.
- The skeleton `TODO.md` that `vc-dr-apply.sh` ships no longer names `commit --closing`.
- Not done, the closing's: the row in `notes/agent-files-size.md`. `AGENTS.md` grew, 526 lines
  to 560, against the quasi-goal.
- Found: `vc-x1 validate-anchors` reports four problems, none of this cycle's making, one of
  them a citation it misses, filed as **validate-anchors misses a citation after a wrapped code
  span** in `## Todo`.

##### feat: cycle requires -s on commit, close, land

`-s` is required on every `vc-x1` command that writes, and `cycle` alone defaulted it, to `b` in a
dual workspace and `p` in a plain repo. It is required with `--commit`, `--close`, and `--land`,
and still refused with the stages that take none.

- wink's call (2026-11-11), turning over the agent's default from **feat: cycle --commit makes a
  rung**: every command that writes names its scope, `commit`, `push`, `squash`, `squash-push`,
  `sync`, `merge`, `move-bookmark`, and `delete-bookmark` among them, so a rule with no exception
  is worth more than the two characters saved, and the tool no longer reads the workspace to guess.
- The parser requires it unless the stage is `--open`, `--ladder`, or `--status`, which refuse
  it. A scope the workspace does not fit is refused, `p` in a dual workspace and `b` or `w` in a
  plain repo, and now for `--land` too, which took the workspace's own before.
- In a chain one `-s` serves the closing and the Land, and `-s w` is refused with `--land`.
- `cycle --status`'s `next` line names the flag, `-s <SCOPE>`.
- The agent-files' command lines gained `-s b`, so the set is `v0.3.1-10`, a second draft after
  the opening where the last rung's subsection says the draft moved once.
- The rung is appended after the docs rung, wink's call, so both repos stay in the order things
  happened. Inserting it before the docs rung was talked through and not done, and what that
  found is filed as three `## Todo` entries, **feat: add a capability in vc-x1 to insert rung
  with -sw|a|b|p**, **A rung is removed from a local ladder**, and **A local ladder's rungs are
  reordered**, to be tried in one exhaustive experiment.
- The rehearsal passes with `-s b` on each of its commands, 48 checks.

##### feat: the xtask completes its tasks

`cargo xtask <tab>` offers file names, and `cargo xtask -h` is a usage error. The xtask's
arguments move to clap, so it has help, and it answers completion requests for its own tasks,
through cargo's hand-off to `cargo-xtask` and for a shell alias such as `cx`.

- wink's ask (2026-11-12), after finding that `cargo xtask` has no completion and that the
  cargo team's own `xtask-*` crates have none either.
- The xtask is on clap, at `vc-x1`'s versions, and calls clap's `CompleteEnv` first, so with
  `COMPLETE` set it answers a request and exits. `bump` offers `next` and `bare`.
- How `cargo xtask <tab>` gets there, found by reading cargo's script and tried here: since
  cargo 1.87 its bash completion hands a subcommand it has no options for, an alias among them,
  to the completion registered for `cargo-<name>`, and zsh has a like hook. No wrapper around
  cargo's own function is needed.
- `cargo xtask completions [NAME...]` prints the function to register. It is clap's own
  registration script with one change: where that calls one binary by name, this runs `cargo
  xtask`, since an xtask is a different program in every repo and all would register as
  `cargo-xtask`. So one copy serves every repo, the answer coming from the repo the shell is in.
  - A `NAME` is a shell alias, wink's `cx`, which bash completes by its own name and never by
    what it expands to.
  - Outside a repo, or where `cargo xtask` fails, it completes nothing.
  - A request took 55 ms here with the xtask built.
  - The candidates keep the order the xtask declares them in, `-o nosort` where bash has it,
    4.4 and later, as clap's script does. Left off at first, they came out alphabetical.
- Caveats, each in the README or to know:
  - It needs cargo's script completing `cargo`. wink's shell had `carapace` registered for
    `cargo`, with 797 other commands, which completes the alias's name and hands off nothing.
  - An xtask in another repo that does not answer `COMPLETE` is run with the typed words as
    its arguments on each tab. One that acts on arguments it does not know would act.
  - bash alone. clap prints the other shells' scripts, and the change is the same one line.
- `-h` and `--help` are help now, where both were the usage error with a failure status.
- The two `## Todo` entries filed since the last rung, **The completion script calls the name
  it was typed as** and **lookup takes seven seconds for one line**, are in this commit.
- This is one project's xtask. Whether validate, build, and install move into an xtask or into
  a runner with its own name, installed once and reading each repo's config, is vc-x2's to
  decide, and this rung is a trial of the first.

##### fix: cycle --land names push for a main ahead

`cycle --land` refused a target whose local and `origin` positions differ with "`vc-x1 sync`
brings it up to date" whichever way they differ, so a local `main` ahead of `origin` was told to
sync, which changes nothing. And with no cycle-record the Land named its source by `--source`, a
flag of the retired `vc-x1 land` and none of `cycle`'s.

- Found by wink (2026-11-13) in `~/tmp/wt-21-6`, step 0 of the test plan: its local `main`
  carries an unpublished closing, and the dry run said to sync.
- The refusal now names the way on by how the two stand, read from the commits: ahead, a push,
  `vc-x1 push -s b --bookmark <target>`, `-s p` in a plain repo; behind, a sync; absent on
  `origin`, a push; diverged, neither, the two to be reconciled first.
  - One function, `land::not_at_origin`, serves `cycle --land`'s own check and the Land's plan,
    which said "sync first" the same way.
- The Land's messages are labelled `cycle --land` and name `--bookmark`, since `vc-x1 land` is
  retired and `cycle --land` is the only caller of its code. The one other bookmark found with
  no record is "the cycle's bookmark", and several ask for `--bookmark <NAME>`.

##### fix: cycle --land and push name -r for a target

The last rung's refusal of a target ahead of `origin` named `vc-x1 push -s b --bookmark <target>`,
which `push` refuses when the target is not at the work `@-`, and `wt-21-6`'s `main` is not:
`single-step-2` is above it. `push`'s own refusal then said "Name one of them" to a bookmark that
was named. Both name `-r <target>` now, which publishes a bookmark where it is.

- Found by wink (2026-11-13) running the last rung's advice in `~/tmp/wt-21-6`. `push --bookmark`
  sets the bookmark at `@-` and publishes that, so naming one elsewhere would move it, and `push`
  refused rightly. Only the wording and the advice were wrong.
- `push`'s refusal now says the bookmark is not on the work `@-`, that pushing it there would move
  it, and gives both ways on: `vc-x1 push -s <scope> -r <bookmark>` to publish it where it is, or
  the line's own bookmark, or a new name, to publish `@-`.
- The Land's refusal names `vc-x1 push -s b -r <target>`, right whether or not the target is at
  `@-`. With `-r` the push sends the work bookmark alone, so a closing's agent commits wait for
  the Land, which publishes the agent `main`.
- The test now runs the command the refusal names, in a workspace whose `@-` is not the target,
  and checks the target is then where `origin` is, so the advice is tried and not only matched.

##### feat: cycle --close names the cycle's shape

`cycle --close` took the cycle's shape from its branch's commit count, which is a guess over one
commit: a single-step cycle whose opening was committed early and a ladder with only its opening
made are the same branch, and the guess squashed the second's closing into its opening. The
closing names the shape, `--single-step` or `--multi-step`, one required, and the count checks it.

- wink's call (2026-11-13), testing a single-step closing in `~/tmp/wt-21-6`, before the
  multi-step cycle there, so each flag gets its first real use there in turn.
- The table: no commit, single-step makes the one commit and multi-step is refused, there being
  no opening. One commit, single-step squashes into the opening and multi-step makes a closing
  commit above it. Two or more, single-step is refused, a ladder never squashed.
  - Multi-step over the opening alone is new: a ladder of its bookends, which only `--dead-end`
    could make before.
- `--dead-end` takes `--multi-step` and is refused with `--single-step`, a dead end being a
  closing commit of its own. The agent had said it would work with either, and chose the refusal
  writing it, since a single-step dead end has no closing to title.
- The flags exclude each other in the parser, and the one required is a refusal of the params,
  clap having no "one of these, with that flag" of its own.
- The rules' Cycle shape and its rule change from the count being the shape to the count checking
  it, the set now `v0.3.1-11`, and `rationale.md` says why, `cycle --status`'s next names the flag.
- Every test that closes names its shape, and the rehearsal's three closings do.

##### feat: improve vc-x1 cycle flow closing

Closing out the cycle.

## Waiting

Important work that cannot start yet. Each entry names what it waits on, in a form that can be
checked, and the rank it takes in `## Todo` once unblocked. Every opening checks each condition
and promotes what is met ([Opening](AGENTS.md#opening)).

- **Retire the frozen history: `notes/chores/` and `notes/done.md`.** (wink, 2026-08-27) They
  are frozen and no longer grow, and the agent-repo transcript plus git history hold what they
  hold. Delete both in one cycle, after a sweep turns every link into them (Todo entries, the
  backlog, rationale.md, the other members' messages) into a permalink at the SHA before the
  deletion.
  - Waits on: **`vc-x1 closed "<title>"`** landed, and the session viewer good enough to read
    a cycle's record from the transcript.
  - Place when unblocked: first.
- **Sync's remote look stops spawning git.** (wink + agent, 2026-10-09) Sync learns where a
  remote's bookmarks are by spawning `git ls-remote`, and downloads the commits of the ones that
  moved by spawning a `git fetch` that moves no ref, the sixth entry of the spawn register in
  `clippy.toml` and the one spawn of `git`. Replace `git_command` and its two callers in
  `src/jj/session.rs` with jj-lib's query, delete the `#[allow]`, and close the entry.
  - Waits on: jj-lib's public API offering that query. Neither 0.45.1 nor 0.46.0 has one:
    `rg -i 'ls.remote' src/` in the jj-lib source prints nothing, and its git subprocess layer is
    `pub(crate)`. An upstream feature request, once filed or found, is named here.
  - Place when unblocked: first, being one function.
- **lookup matches a line to its write by content.** (wink + agent, 2026-10-26) lookup resolves a
  line through blame to its commit and the commit's `ochid:` trailer to its agent partner, with
  the agent commits inside a time tolerance as candidates when there is no trailer. In one
  person's workspace the trailer is exact. In a shared repo it fails: a squash or rebase merge
  leaves blame on a commit no trailer names, and a trailer that survives may point into an
  agent-repo the reader cannot reach. Time fails too, many contributors' commits inside any
  tolerance.
  - The key that survives squash, rebase, and cherry-pick is the text: each transcript records
    what every Write and Edit put into a file, so a line is matched to the call that wrote it
    with no commit between. **dr1_every_committed_line_finds_its_transcript_write** checks a
    part of it. The layering: the trailer when it resolves, the content when it does not, time
    last.
  - What makes it hard: a line changed after the write, by a person or `cargo fmt`, needs a
    fuzzy match, a common line (`}`, `Ok(())`) needs its hunk for context, and every
    contributor's transcripts must be reachable and indexed, a privacy question as much as a
    technical one.
  - Waits on: **Cycles in a shared repo, by fork and pull request** opening, since the need
    arises there.
  - Place when unblocked: directly after it.
- **vc-x2 is seeded from vc-x1.** (wink + agent, 2026-11-10) A test vc-x2, seeded from a rehearsal
  without history, its first commit naming vc-x1's final SHA by hand, with `lookup` tried in it and
  what vc-x2 needs filed as `## Todo` entries: a `seeded-from:` trailer and `lookup` following it,
  adopt taking a renamed config, an agent across several work-repos, the push-time check, and the
  xtask move. It was the rung **chore: a test vc-x2 reaches vc-x1's history**, cut at the closing.
  Then vc-x2 itself: a `git archive` of vc-x1's landed `main` made the new
  root of `winksaville/vc-x2`, with a fresh `vc-x2.agent-session`, both adopted, and a continuation
  note in vc-x2's `TODO.md` handing the work to vc-x2's own session. The decisions toward it are in
  **chore: complete vc-x1, ready for vc-x2**'s record, found by `git log --grep` on its title, its
  landmark's `## Closed` holding them. **vc-x2: a cycle command, open, close, and land** and
  **vc-x2: the agent-repo stays on main, agent branches set aside** are vc-x2's Todos.
  - Waits on: **feat: improve vc-x1 cycle flow** closed, its rehearsal of a single-step and a
    ladder cycle running clean.
  - Place when unblocked: first.

## Todo

Entries are in priority order, the first highest, and reprioritizing is moving an entry. Each is a
`###` heading, so a citation is a link to its anchor. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the [Prose form](agent-data/prose.md#prose-form).
Deeper detail goes in a `notes/` design file (link via `[N]` ref).

### Work in progress is durable and continues on any machine

(wink, 2026-11-10) A cycle stopped for the night was made durable by a work and an agent commit
titled `wip`, pushed, and removing them took two abandons and two backward pushes by `jj`, since
`vc-x1 push` would fold the agent `@` into the last rung's agent commit. A scheme is wanted that
makes work in progress durable, on the remote, and easy to continue on this machine or another,
then leaves no trace in the cycle's history once the rung it belongs to is committed.

- We think a separate ref, a `wip/<bookmark>` on the remote holding both `@`s, is the shape: the
  cycle's bookmark and the agent `main` never move for it, and a `continue` fetches it into `@`.
- What the other machine's session reads is part of it: the continuation notes ride in the work
  `@` and the transcript in the agent `@`.

### push only publishes

(wink + agent, 2026-11-10) `push`'s agent half folds the journal into an unpublished partner, or
commits it as the partner, before it pushes `main`, so a push changes a commit. A push should only
publish, the fold being `squash-push`'s. The agent `@` is always dirty: Claude Code writes after
the agent's last command, so a tail is left whatever the agent runs, and it goes into the next
agent commit, of the next rung, cycle, or session. Spawning a delayed capture was tried and dropped,
its races and hand-off files costing more than housekeeping is worth.

- `push` loses its fold and its commit of the journal, and refuses a pair whose agent half is not
  committed, naming `commit -s a`.
- Whether the agent publishes by `push` or by `squash-push` is decided by measurement, wink leaning
  to `squash-push` to keep each rung's housekeeping in its own commit. Over a few rungs, record the
  agent `@`'s entries and bytes before each publish, what `squash-push` folds and `push` leaves
  forward, and at the next acquaint, the tail neither catches, each against its rung's agent commit.
  The first sample: rung 4's agent commit is 470 entries, 813 KB, and what followed it to the
  session's end 16 entries, 18 KB, mostly the records Claude Code writes after the agent stops.
- The agent-files: At rest's "clean" is the work `@` alone, the agent `@` is never clean and
  carries forward, and rung 4's "push folds the journal while unpublished" goes.

### validate-desc shows a note as part of its row

(wink + agent, 2026-11-10) `validate-desc` prints a note, a remark on a commit whose verdict is
still `ok`, as a row of its own before the verdict, keyed by the same change id, so one commit reads
as two: `note qrsqqzlyzzsm the agent commit rnnmsmxwtzol is not on a remote yet` above `ok
qrsqqzlyzzsm feat: no trunk, every target is named`. Notes are common now that a rung commits as a
pair locally, its agent partner unpushed until a push.

- A note belongs to its row. A succinct mark is wanted, a `*` on the verdict or the like, TBD, with
  the text said once below the table or under `-v`. Another option is there is a column of "other"
  information that could be several characters wide with single character info. 
- The two kinds: an agent partner committed and not yet on a remote, and an older 8 or 32 character
  `ochid:` naming one commit. README's status list names neither, the second documented only in
  the code, and both are written up there.

### close, land, and push compose with testing, to re-deliberate

(wink + agent, 2026-11-02, moved here 2026-11-05) The plan of the cycle that opened as **feat:
close, land, and push compose with testing** and closed as **feat: test repos, vc-dr-apply, and the
v0.3.1-(0,1,2) agent-files**, whose landmark `df633e81` holds it whole: that cycle built what the
plan needs and left its rungs below unstarted. Wink's call (2026-11-05): it goes first and is
re-deliberated before any of it runs, since it was laddered before **The Land is one command,
agent-files v0.3.1-3**, **commit's --body is optional**, and **vc-x1 do runs a recipe of vc-x1
commands**, which overlap it, the last proposing stage flags and recipes for what its three steps
compose.

#### Its problem

(wink + agent, 2026-11-02) Closing a cycle should be obvious, and to the user's liking, and `vc-x1`
should help them do it properly. It does not:

- A local cycle's closing holds `ocid: .../xxxx` until `vc-x1 land` commits the agent `@`, and that
  rewrites all of the cycle's commits, 12 last time, so what was tested is not by SHA what lands.
- The way through took five steps, which wink called a non-starter, and the one-command answer
  gives up the test pause.
- `land` needs the agent repo, so a work-only land, a merge or rebase, or a share for others to test
  drags in a permanent link it does not need.
- Nothing says where a cycle stands or what may come next.
- The commands are already separate, so that is not the problem, wink's (2026-11-02). They do not
  compose: the user wants to test between any two of them, passively by `validate`, and actively by
  you, me, and others running a build named by `profile.<name>.suffix`. Today that is by hand and on
  the real repos, and full `validate` runs `cargo fmt`, which can dirty the `@` the next step needs
  empty.

#### Its solution

Provisional. Three steps, each doing one thing, that compose with testing:

- Every step leaves a state that can be tested, nothing half-done.
- A step may repeat, so the user can test, tweak, and close again.
- Testing does not disturb the cycle or block the next step.
- Each tester's binary says what it is, its suffix the build and its version the commit.
- Exit codes can be trusted, so the steps and the tests chain in a plain shell, and `vc-x1` does not
  run the tests itself.

The steps:

- close makes the cycle's final shape, single-step or a trapezoid, local, with `--scope` defaulting
  to work. It may be run again, and the agent and the user keep iterating on it.
- land merges or fast-forwards onto any bookmark, local, and it may repeat. `--scope` defaults to
  work, and it takes any scope that applies.
- push to any other bookmark shares the work for backup or testing, with `xxxx` still in it. Push to
  a release bookmark with `-s b` seals: it commits the agent `@`, fills the `ocid:`, refreshes any
  stale `pcid:`, checks each tree unchanged, and publishes the agent repo first.
- A release bookmark is one the user treats as permanent: `[release] bookmarks`, or `main` and
  `master` when that key is absent. Work alone with an unsealed agent commit is an error there,
  unless `--yes`.

The links follow one general rule, for any mix of work and agent commits:

- `ochid:` is one line per related commit, on either side.
- Every `ocid:` is the SHA of an agent commit the same commit's `ochid:` lines name, matched by
  resolving it and not by line order.
- A work commit's `ocid:` is real only when its agent commit was made before it, wink's
  (2026-11-02), and otherwise stays `xxxx`. So most are `xxxx`, rungs included and on a release
  bookmark, and are never rewritten. In an N:1 cycle only the closing has a real one, the seal
  committing the agent repo before the closing's final form. In 1:1 pairs every commit has a real
  one.
- Each agent commit's SHA is carried by exactly one work commit, the last it names.

#### Its acceptance check

By `vc-x1-dev`, on repos made by `init --test-repo <path>/<repo-name>`, in a table-driven test:

- An N:1 cycle: close leaves every `ocid:` at `xxxx`. A tweak and a second close work. `land` with
  no `--scope` leaves the agent repo alone, and a push to any other bookmark keeps `xxxx`, at any
  scope.
- A push to a release bookmark with `-s b` commits the agent `@`, fills only the closing's `ocid:`,
  leaves every rung's `xxxx` and SHA unchanged and the closing's tree identical, and pushes the
  agent repo first. Pushed to a second release bookmark, the commits are already sealed and nothing
  changes.
- A push to a release bookmark of work alone, with an agent commit no commit carries the SHA of, is
  an error naming `-s b` and `--yes`, and with `--yes` it publishes and warns. An unsealed closing
  naming another agent `@` is refused, and a release bookmark that moved is refused before anything
  is committed.
- The release bookmarks: with no key, `main` and `master`, so an old config works, and with
  `[release] bookmarks`, that list alone, a project's `main` then being an ordinary bookmark.
- A test between any two steps, `validate` or a suffixed build, changes nothing the next step
  depends on, and the next step runs.
- Two clones of one local remote give a moved trunk and another's commits, with no network.
- 1:1 pairs carry a real `ocid:` on every commit and never `xxxx`.
- `validate-desc` and `lookup` accept 1:1, N:1, 1:M, N:M, and mixed, and this repo's `main`, where
  every rung of **feat: improve trailers** carries an `ocid:`, reports what it reports today.
- close, land, and push each end by printing where the cycle stands and what may come next.
- `vc-x1 validate` passes.

#### Its deliberation

- Close makes no agent commit, wink's (2026-11-02), reversing the agent's proposal: the agent and
  the user iterate after close, the conversation belongs in the cycle's own agent commit, and rework
  stays free since nothing on the agent side names a rung yet.
- The seal is the push to a release bookmark, wink's (2026-11-02), reversing the agent's
  land-as-seal: land may be a merge or a rebase on any bookmark, and sharing to test needs no
  permanent link. Only the release line is permanent.
  - The cost: no test at the final SHA before the release, only at an identical tree, and the
    binary's embedded SHA differs. A tester's SHA for the closing never reaches `main`, the rungs'
    do.
  - It bends **Only push publishes**, one push rewriting before it publishes, so it says so plainly
    and `--dry-run` shows every rewrite.
  - We think sealing is callable on its own too, since a pull request's merge is GitHub's and runs
    no `vc-x1 push`. It is built as a step of its own, whether or not it gets a flag now.
- `ocid:` is real only when the agent commit came first, wink's (2026-11-02), so a rung's stays
  `xxxx` rather than being absent or rewritten, and the commit that carries an agent commit's SHA is
  the last it names, wink's and the agent's, generalizing closing-only. A reader goes from a rung to
  its agent commit by `ochid:`. Wink's "next commit" is the fallback once the last is on the release
  line, a one-way link. Rejected as the norm: on a closing, "next" is the next cycle's opening, so
  `main`'s landmark would have no link to its conversation.
- Any mix is valid, wink's (2026-11-02): N:M, so readers and validators accept all of it and stop
  requiring links to be mutual. Writers stay at 1:1 and N:1 here, as **commit writes every mix of
  work and agent commits** files, and the agent's mid-cycle commit is **An agent commit mid-cycle
  backs up the transcript**.
- `--scope` defaults to work, always, wink's (2026-11-02), and any scope that applies may be named.
  - It reverses **commit works at every scope, and --local keeps it on the machine**'s `--land`
    requiring `--scope`, since a work-only land moves no agent commit.
  - This cycle does it for close, land, and push, and the rest of the commands stay with that entry.
- The tests run outside `vc-x1`, wink's and the agent's (2026-11-02): composing is the steps' job
  and a shell's, and `validate takes any profile` stays its own Todo.
- The agent repo may have branches someday, merged at a final Land, wink's heads-up (2026-11-02), so
  nothing here assumes the agent repo is linear, and the readers rely on the trailers alone.
- `xxxx` is an error only on a release bookmark, wink's (2026-11-02): it is fine on a test branch
  and probably a mistake on `main`, so the user defines the set, with `main` and `master` hard-coded
  for existing configs.
  - The key is `[release] bookmarks`, the agent's: a property of the repo and not a cycle, and the
    rules say bookmark. `[cycle] trunk` stays, where a cycle lands by default and not what is
    protected.
  - A key that is present replaces the two names rather than adding to them, the agent's: otherwise
    a project developing on `main` and releasing on `stable` could never take `main` off.
  - The check is per agent commit, the agent's: rungs keep `xxxx`, so a search for it would refuse
    every push. Each group of commits naming one agent commit needs one real `ocid:`.
  - Other commands read the same set, `trailers --update`'s refusal first, which names the trunk
    today.
  - `--yes` is no human check, an agent typing it as easily, so the agent passes it only when wink
    says so for that push. **Only push publishes** has the mechanism as open.
  - It catches a mistake made through `vc-x1` and no more: a raw `jj git push` or a pull request's
    merge goes around it, and branch protection is the tool for that.
- `vc-x1 close` is a subcommand, the agent's, to confirm: it pairs with `land` and `push`, and
  `commit --closing` shares its code.

#### Its rungs

The rungs as laddered, not started, in their order.

##### feat: the readers take any mix of ochid and ocid

The readers and validators assume one `ochid:` on a work commit and a link both ways. They learn the
general rule first, so no build writes a form it cannot read.

##### feat: ocid is real only when the agent commit came first

A Land rewrites every rung to fill its `ocid:`. It stops, so a rung's stays `xxxx` and only the
commit carrying the agent commit's SHA is written, and `validate-desc` accepts `xxxx` on the others.

##### fix: commit --closing reads the nearest opening and keeps the cycle's base

The two folded `commit --closing` Todos, wink asking if they are one cycle and the answer yes. A
stack's closing reads its own opening, and its first parent is the opening's parent when that
descends from the trunk. We think it refuses when a cycle below is not closed, which answers wink's
question on dangers.

##### feat: land takes @- and the work scope by default

`land` requires `--scope`, refuses work alone, and finds its source from every bookmark. Its
`--scope` defaults to work, leaving the agent repo alone, any scope that applies may be named, and
it takes the folded Todo's `@-` default.

##### feat: a push to a release bookmark seals the cycle

Nothing makes the link permanent once `land` stops. `[release] bookmarks` names the set, in
`vc-config.md` and the generated model too. A push to one with `-s b` commits the agent `@`, fills
the `ocid:`, checks each tree unchanged, and publishes, and it refuses a bookmark that moved and an
`xxxx` naming another agent `@`. Work alone leaving an agent commit unsealed is an error unless
`--yes`.

##### fix: testing leaves a cycle able to continue

A test between the steps can leave the next one refused, full `validate`'s `cargo fmt` dirtying the
`@` that `land` needs empty. A test changes nothing the next step depends on, or the step says what
it found and how to go on.

##### feat: close, land, and push say where the cycle stands

Nothing says what state a cycle is in. Each command ends by printing it and what may come next, and
refuses what may not, and `vc-x1 close` arrives.

##### docs: the rules and README name the three steps

The README and **agent-files(proposal): v0.3.1**: **A cycle stays local until its Land**, **Land is
the sequence, and `vc-x1 land` its integrate step**, **Only push publishes**, and **Work commits
link by URL** are rewritten, and the Terminology gets a word for what makes a cycle permanent, we
think "release". The rules themselves change in that proposal's cycle.

#### The Todos it folds

##### commit --closing reads the nearest opening's title

(wink 2026-11-1: this these 2 `commit --closing` should be one cycle?)

(wink + agent, 2026-10-26) Without `--title`, `commit --closing` takes the cycle title from the
opening above the trunk, and with cycles stacked on one line it reads the oldest: the closing of
**feat: vc-x1 land** came out as **feat: vc-x1 commit takes a revision closing**, the stack's first
cycle, and was made with `--title`.

- The opening read is the nearest below the tip, the cycle's own, so a stack closes its top cycle.
- A test closes a stack of two cycles without `--title`.

##### commit --closing merges onto the cycle's base when it was opened above the trunk

(wink 2026-11-1: this seem convient, but not sure the why,when,downsides,dangers)

(wink + agent, 2026-10-26) `commit --closing` merges the trunk with the tip, so a cycle opened
above the trunk, on an unlanded series as **feat: vc-x1 land** was, gets a closing whose first
parent is the trunk, and the cycles below it leave `main`'s first parents for the second parent's
ladder. That closing was re-parented by hand onto the fix commit its opening sat on.

- The first parent is the opening's parent, where the cycle's bookmark was created, when that
  descends from the trunk, and the trunk otherwise, so `git log --first-parent` reads one commit
  per cycle.
- A test closes a cycle opened above an unlanded one and checks both parents.

##### land lands @- by default

(wink + agent, 2026-10-29) `land`'s `--source` defaults to "the one local bookmark besides
`--onto`", which depends on every bookmark in the repo and fails once there are two. The rest of
vc-x1 works from the line's `@-`, `push` moving its bookmark there and `commit --closing` naming it.

- `--source` left out is `@-`, with the work `@` empty: what lands is then the tree validation ran
  on, and nothing uncommitted is left behind. `commit --closing` without `--land` leaves exactly
  this, the closing at `@-` and an empty `@`.
- A non-empty work `@` is refused, naming `--source`, rather than warned about, since a land is
  pushed and permanent on the trunk. A merge `@`, whose `@-` is ambiguous, is refused too.
- The empty check is the work-repo's alone: the agent `@` holds the session and is never empty.
- An explicit `--source` lands what it names, with no check of `@`.
- The cycle's bookmark is found from `@-` as today: its own, else its nearest parent's above
  `--onto`, several refused with the list.
- The help: `--source <BM|REV>`, "The cycle's last commit, a bookmark or a revision [default: @-,
  with @ empty]".


### The Land is one command, agent-files v0.3.1-3

(wink + agent, 2026-11-04) The Land of **feat: test repos, vc-dr-apply, and the v0.3.1-(0,1,2)
agent-files** took three commands, `land`, `push -s b --bookmark main`, and `delete-bookmark`, where
`land -s b --onto main --push` does all three, and its close-out four more, `commit --closing`,
`commit -s a --covers`, `trailers --update`, and `push -s b`, where `commit --closing … --land
--push` does them and the Land too. The agent followed `jj.md`'s numbered Land list, which names
`land --push` only as "steps 2 and 3 as one" and not its deleting the topic on `origin`. The
agent-files draft `v0.3.1-3`, wink's call.

- `jj.md`'s Land section leads with the one-command forms: `commit --closing <BM|REV> --land --push`
  to land at the closing, and `land -s b --onto <trunk> --push` to land later, each publishing the
  agent repo first and deleting the cycle's bookmark on `origin`. The numbered steps stay as what
  they do.
- The install is part of the Land, after its push, so At rest's hard stop follows the install and
  not the push: today the two disagree, `jj.md` putting the install last and At rest stopping at the
  final push.
- `land`'s hint for a topic still on `origin` names `--push` first, `delete-bookmark` after.

### commit's --body is optional

(wink + agent, 2026-11-04) `commit` refuses a commit with no `--body`, "--title and --body are
required without --closing", and refuses `--body ""` as blank, so a title-only commit, ordinary in
git and enough for a one-line fix or a test repo's rung, cannot be made. The body's form is a rule
of the agent-files, Commit description, and the tool enforcing it makes every user carry the
agent's convention.

- No `--body` commits the title alone and warns that the body is missing.
- `--body ""` commits the title alone with no warning, the empty body asked for.
- The closing's defaults are unchanged: the opening's title, and `[closing] body` or the opening's
  body.
- The change is `description`'s match and the blank check in `src/commit.rs`, and
  `scope_title_and_body_are_required` becomes a test of the three cases.

### vc-x1 do runs a recipe of vc-x1 commands

(wink + agent, 2026-11-04, reworked 2026-11-05) commit, close, trailers, land, push, and install are
operations, and today they compose only where a flag was wired for it: `commit --push`, `commit
--closing --land --push`, and `land --push` work, while `commit --land` without `--closing`, `commit
--push NAME`, and a `land` with no `--onto` do not, so which forms work is a matter of history. A
chain typed on one line, `commit ... + push + land`, was weighed and set aside, wink's call after
sleeping on it: it is error-prone, shell completion cannot follow its operations' different flags,
and a long line is cumbersome to type and to edit. Two ways in replace it, on one model of the
operations.

- The model: each operation is a step from a state to the next, with a precondition, so any order is
  free where the state is valid for the step. A step's outcome is valid, refused, or a nop, nothing
  for it to do: `commit` with an empty `@`, `push` with nothing new, `land` with the trunk at the
  source already. A nop is reported and the run goes on, and `--strict` makes it an error. Whether a
  plain `commit` keeps today's "nothing to commit" refusal or only `--strict` does is open, and we
  think the command alone and the step in a recipe behave the same.
- Stage flags: a command takes its later stages as flags in a fixed order, commit, close, land,
  push, install, `commit -s b --title ... --body ... --close --land --push`, every combination
  working and checked by the state, not only the pairings wired today. One command and one flag set,
  so completion works, and a nop stage is skipped.
- Recipes: `.vc-config.md`'s `[recipes]` names lists of `vc-x1` invocations, read as `[validate]`
  reads its lists, split on whitespace with no shell, globs, or pipes, each line a command that
  could be typed and completed alone. `vc-x1 do NAME` runs one, stopping at the first failure and
  naming the line. `{title}`-style placeholders are filled by `--set title="..."`, as
  `vc-dr-apply.sh` fills `{name}`, after the split, so a value with spaces stays one argument.

```toml
[recipes]
land = [
  "vc-x1 commit -s b --closing @- --land --push",
  "vc-x1 install",
]
```

- A recipe's lines each plan themselves, so a refusal on a later line comes after the earlier ones
  ran. `do` refuses a recipe with a line that does not publish after one that does, and on a failure
  names the `jj op restore` point taken before its first line. We think planning the whole recipe
  against a model of the state, before anything moves, comes later, the runner calling `vc-x1` lines
  in-process.
- `[validate]` is a recipe too, so the two may share the runner.
- One report at the end says where the cycle stands, the carried rung **feat: close, land, and push
  say where the cycle stands**.
- The existing combined flags become stage flags, or retire. It subsumes the composing rungs of
  **close, land, and push compose with testing, to re-deliberate**, **feat: land takes @- and the
  work scope by default**, **feat: a push to a release bookmark seals the cycle**, and **fix:
  testing leaves a cycle able to continue**, and the one-command half of **The Land is one command,
  agent-files v0.3.1-3**, and **A hyphenated subcommand becomes a command and a flag** names its
  verbs, `close` for `commit --closing`.

### commit works at every scope, and --local keeps it on the machine

(wink 2026-11-01, I want to discuss have most/all vc-x1 command not require `--scope` and default
to `--scope work` if not "dangerous".)

(wink + agent, 2026-10-30) Wink's desire: every variation of `vc-x1 commit` works at every scope,
and a dangerous combination is an error unless `--yes` is given, what is dangerous still to be
decided. Found trying a cycle held local, every commit by `commit -s w --no-push` with one agent
commit made at the Land. Folds in **commit and land take the same scopes**, which planned to drop
`commit -s a`, the opposite.

- `--local` and `--no-push` mean nothing leaves the machine, the default since **feat: ochid trailers
  are URLs**, where a bare `commit` stays local and `--push` publishes.
- `--scope` is optional and defaults to `work`, `por` in a plain repo, wink's (2026-10-30), since
  `-s w --local` is a plain repo's commit with a link back, and a plain `vc-x1 commit` is then a
  rung. This holds for `--opening`, `--closing`, and `--push`, and `vc-x1 push`, the same as `commit
  --push`.
- `--land` requires `--scope`, and `vc-x1 land` the same, dropping its default of `both`, wink's
  (2026-10-30): a Land has a new destination, so it is assumed dangerous for now.
  - Reversed by wink (2026-11-02), `--scope` defaulting to work, always. **feat: close,  land, and push
    compose with testing** does it for close, land, and push, and the rest stay here.
- A pushed work commit whose `ochid:` names an unpushed agent commit is not dangerous, wink's
  (2026-10-30). The link is dead until the agent side is pushed, which is obvious to the user. The
  commit is still work, to modify, squash, fork, or merge, and `--push` makes it durable while it
  stays fluid. Whether a Land's result is durable is the user's to decide.
- `--closing --squash` folds a ladder into one commit with the bare cycle title, no " opening" and no
  " closing", a single-step cycle by its commit count. Considered, not decided, wink + agent
  (2026-10-30). We think it refuses a merge in the range, a bottom title with no " opening" and no
  `--title`, `-s a`, and any range whose commits carry different `ochid:` values, a `--local -s b`
  ladder among them, since its rungs each have an agent commit. Whether a pushed range refuses is
  open, since a pushed commit is still fluid by the entry above.
- A matrix of command against scope, each cell what it does or why it refuses, in the README's `###
  Write scopes` table and checked by a table-driven test, so a new flag leaves no gap. Goes with
  **docs: each command's -s help names the scopes it takes**, which checks the help against that
  table. Today, by trial on 0.85.17, the `-s a` refusals read from the code and `land`'s help:
  - `b` and `w` take a rung, `--opening`, `--closing`, and `--closing --land`, and `-s w` with
    `--land` or a push publishes both repos.
  - `a` takes a rung, unpaired, and refuses both bookends.
  - `p` takes all of them in a plain repo, and refuses everything in a dual workspace.
  - `land` takes `b` and `p`, and refuses `w` and `a`.
- `-s p` in a dual workspace: `p` stays "no partner", so it works there as an explicit opt-out with
  no trailer. A trailer would make it `-s w --local`, which exists, and a plain repo has no agent
  repo to name.
- A Land is local since **feat: improve trailers**, `--push` publishing it, and a later
  `vc-x1 push -s b` does the same. Open: the name of `--land`'s target beside `--bookmark <BM|REV>`,
  and `push` deleting a published cycle's bookmark on `origin`, which a bare Land leaves there.
- `--closing` at `-s a` does what `--closing -s b` does on the agent side, wink's (2026-10-30): a
  regular commit, no trapezoid, the merge being the work-repo's. We think `--opening` at `-s a` is
  the same, with no bookmark. Open: its title with no opening to read it from in the agent-repo,
  and its trailer, since `-s b`'s names the work closing and `-s a` makes none, today's `-s a`
  commit being unpaired.
- `--yes` passes the danger guard and nothing else. The description review stays the agent's, in
  conversation, at [The per-rung flow](AGENTS.md#the-per-rung-flow) step 8, with no gate in the tool,
  so `--yes` does not excuse it and a waiver is the user's to say. `push`'s `--yes` passes its
  `--step` gates, so that one is reconciled or renamed, and **squash-push shows what it takes, and
  push follows jj** retires `squash --yes`, so the two entries agree first.
- Candidates for dangerous, to be decided:
  - landing a trailer that names an unpublished agent commit, a work-only Land
  - landing a range with a trailer naming an agent change that no longer exists
  - rewriting a commit on the trunk's remote, which `squash` already guards
  - `-s p` in a dual workspace, giving up the rung's link to the transcript.
- The worst danger found is outside `commit`: `squash-push -s a --yes` between `-s w` rungs folds
  the agent `@` the earlier trailers name, so they dangle, and commit and land both still succeed.
  Only `validate-desc` over the range reports it, `[not found]`. A check at the Land or in
  `squash-push` closes it, and until then a cycle held local runs no `squash-push -s a`.
- From the folded entry:
  - the rule that a scope is valid when what it publishes names nothing it leaves unpublished,
    which binds publishing and not committing, so every local cell can work
  - `merge` keeps `w` and `p`, a trapezoid being the work-repo's
  - the repo commands keep `a`: `sync`, `squash`, `squash-push`, `push`, `status`, `move-bookmark`,
    `delete-bookmark`, and `validate-desc` act on the agent-repo as a repo of its own and link
    nothing.
- Open: a held-local cycle's agent commit names only the closing, so the transcript reaches the
  rungs only through their own trailers. **vc-x1 push: record uncovered work commits (N:1
  work<->agent)** plans one `ochid:` line per commit. We think `lookup` from a rung gets the whole
  cycle's agent commit, not yet tried.

### init defaults to --local

(wink, 2026-10-30) `vc-x1 init` makes the workspace on the machine and creates and publishes nothing
unless `--push` asks, as `commit` does in **Only push publishes**. Today it creates the remotes and
pushes, public on GitHub unless `--private`, so a bare `init` publishes a repo.

- `--push` is what creates the remotes and publishes, and the approval it needs is the rest of that
  section's. `--private` and `--push-retries` take effect with it.
- Publishing later needs a way: a workspace with no `origin` has nothing for `vc-x1 push` to push to,
  so `push`, or a command of its own, creates the remotes.
- The trailers: `init`'s cross-links then have no URL to stamp, so they take the path form **feat:
  ochid trailers are URLs** defines, `/.agent-session/<chid>`, and they stay valid after a remote
  appears, the URL being checked only on a URL form.
- The name: `--repo local` is already a category, a local directory of bare repos, which is a remote
  of a kind and not this `--local`, so one of the two is renamed or explained. `clone` is to be
  checked for the same default.
  - **feat: close, land, and push compose with testing** makes `init --test-repo <path>/<repo-name>`,
    with local bare remotes, so that name is not taken by `--local`. What a bare `init` does stays here.

### squash-push shows what it takes, and push follows jj

(wink + agent, 2026-10-28) The rules for rewriting what is published, from the opening that became
**feat: undo takes back the last vc-x1 run**. `squash` refuses folding into a commit on the trunk's
remote without `--yes`, a guard on a local act that undo takes back, while `squash-push -s b` folds a
work `@` it expects empty without showing it, and pushes it. The mistake that matters is a command
taking more than was meant and publishing it unseen, not a bookmark moving sideways. After the undo
cycle and before **squash takes --include**, wink's order (2026-10-28).

- Local acts are not refused, since undo takes them back: squash's trunk refusal becomes its
  existing rewrite warning, and its `--yes` retires as `--dev` did, hidden and refused, freeing it
  for **squash takes --include**.
- A command shows what it takes when that is more than the routine: `squash-push -s b` at rest
  expects an empty work `@`, and a non-empty one is listed, each file with `+n -m`, and asked for,
  `squash --include`'s approval. `squash-push.yes` skips the routine case alone, the agent tail or
  an empty work side.
- Pushes are force-with-lease and nothing more: a backward or sideways move goes, undo reversing
  it, and the lease stops a push over a remote someone else moved. No bookmark is special to vc-x1,
  and a shared repo's guard on its trunk is the server's, GitHub's branch protection refusing a
  force push from every client.
- The agent-repo stays clean in one command, `squash-push -s a` folding the tail into its published
  tip, a local rewrite and a leased push.
- Push's flags are a subset of `jj git push`'s where they overlap, with its meanings: `--remote`,
  defaulting to `git.push`, else `origin`, else the only remote, `-b/--bookmark`, `-r`, `--named
  NAME=REV` for a new bookmark, and `--dry-run`. vc-x1 adds the scope and the pair, the trailers,
  and moving the line's bookmark to `@-`. What jj does is information, not a requirement (wink,
  2026-10-28).
- Push's help and module doc say what each scope publishes: the pair only for `-s b`, since `-s a`,
  `-s w`, and `-s p` each publish one repo.
- Open: the trunk a closing merges onto and `--land` lands on, `[cycle] trunk` else `main` today.
  The candidates are jj's `trunk()`, the latest of `main`, `master`, and `trunk` on `origin` or
  `upstream` unless a repo overrides it, and the remote's default branch, `origin`'s `HEAD`, the key
  retired either way.
- **push --from --to moves a bookmark and publishes it** may fold in.

### commit's dry run reports the push as the advance leaves it

(agent, 2026-10-25) `commit --dry-run` plans the bookmark advance, then runs push's dry run, which
reads the bookmark where it is, since the dry run moved nothing, so a commit that advances a
bookmark reports publishing it at its old commit. Found at the acceptance check of **feat: vc-x1
commit takes a revision**, where the fixture's cycle bookmark was reported at the opening rather
than the draft.

- Push's dry run takes the target the advance would leave, or commit's dry run says the push
  without asking push.

### A bookmark set to where it is commits no operation

(agent, 2026-10-29) `Session::bookmark_set` commits a transaction whether or not the bookmark moves,
so a `commit -s b` leaves two identical "point bookmark ... to commit ..." operations, the commit's
advance and push's, 3 s apart. Found in `jj op log` at the rung **feat: op lists each run and its
operations** of **feat: undo takes back the last vc-x1 run**.

- A transaction with no change is dropped rather than committed, `repo_mut().has_changes()`, in
  `bookmark_set` or in `finish_tx` for every caller.
- Each wasted operation is one more line in the op log and one more step an undo walks back.

### The run record misses workspace init

(agent, 2026-10-29) A `clone` run's first operation in each repo, jj-lib's `add workspace
'default'`, carries no run record, since it is made before vc-x1 opens a transaction. The record is
set at four places, `Session::start_tx` and the snapshot and the two git imports, and workspace
init is a fifth. Seen in a fresh `vc-x1-dev clone`, whose other three operations per repo carry
the run.

- Tag it, or treat the operation before a run's first as the run's base, which is what an undo
  restores to either way.
- `init` makes the same operation and is the other case to check.

### vc-x1 stamps no user, host, or workspace on its operations

(agent, 2026-10-29) In `jj op log` raw jj's operations read `wink@3900x default@` and vc-x1's read
`@`, with no user or host, and its snapshots carry no workspace either. In this work-repo 543
operations are raw jj's, 427 vc-x1's with a workspace, and 389 vc-x1's without.

- Set the user and host through jj-lib's settings, as jj-cli does, so vc-x1's operations read as
  jj's own, the run record saying which run made them.
- The snapshot path sets the workspace as the transactions do.
- A missing stamp tells vc-x1's operations from raw jj's today, and is how a stable vc-x1's
  untagged ones were recognized. The run record replaces that test.
- The op log is local to `.jj` and never pushed, so these stamps and the run record stay in the
  clone that made them: a fresh `vc-x1-dev clone` of this pair begins both op logs at the clone,
  none of the original's operations carried. Commits carry their change ids, `c594f1c0`,
  `c1e9e653`, and `de2e9af7` identical in both, so `ochid:` trailers resolve in a clone (confirmed
  2026-10-29).

### docs: each command's -s help names the scopes it takes

(wink + agent, 2026-10-26) Every command's `--scope` help writes the scopes out, `w|work`,
`a|agent`, `b|both`, `p|por`, and clap then repeats the long names as `[possible values: ...]`,
since the parser declares them for completion. A command taking a subset reads worse:
`validate-desc` lists four, says "`p` refused", and clap prints three. `delete-bookmark --skip`
names its two values and clap repeats them. Found at **feat: vc-x1 land**, whose `land` and
`merge` `-s` help it fixed, `-h` naming only the scopes taken and `--help` why the others are
refused.

- The README's `### Write scopes` table, written by **feat: vc-x1 land**'s docs rung, is the list
  the help is checked against.
- `hide_possible_values` on every `--scope` and on `delete-bookmark --skip`, the help text the one
  list, being the one with the short forms.
- `validate-desc`'s `-h` names `w|work`, `a|agent`, or `b|both`, and `--help` says why `p` is
  refused, in `land`'s and `merge`'s form.
- `tests/cli_complete.rs` still offers each command's scopes, since the values are hidden from
  help alone.

### A cycle should dogfood its own changes continuously

(wink + agent, 2026-10-26) Every rung on a cycle should use `vc-x1-dev` for all operations that
aren't broken.

### agent-files(proposal): v0.3.1

(wink + agent, 2026-10-18, restructured 2026-10-20) Small rule fixes learned by running v0.3.0,
dead-end cycles defined, and the end of every rung and of Land reshaped, so that what the user
reviews is what is published. Follows **feat: vc-x1 commit, install, and run-dev**, whose commands
the rules below name. Each change is a `####` heading holding what is done, and its `#####` holds
why.

Adopted locally as the draft `v0.3.1-0`, vc-x1's agent-files since **chore: vc-x1 adopts the
agent-files v0.3.1-0 draft** (2026-11-03), and `v0.3.1-1` since **chore: vc-x1 adopts the
agent-files v0.3.1-1 draft** (2026-11-04), which wrote in what came after the first draft: **New
repos start from an adopter's rules**, **At rest leaves the tail for the next push**,
**Every unpublished rung's ocid is real before it is published**, **The user decides when each side
commits and publishes**, **Step 8 names the bookends**, and **The user chooses the dev build**.
`v0.3.1-2`, since **feat: the cycle record returns to TODO.md, indexed by zTOC.md, v0.3.1-2 draft**
(2026-11-04), adds **The record stays in TODO.md, and zTOC.md indexes it**, and **feat: vc-dr-apply
takes the active version and offers language skeletons** makes the working copy the default source
in **New repos start from an adopter's rules**, and **feat: the ladder lives in zTOC.md alone** adds
**The ladder lives in zTOC.md alone**, the one draft spanning those rungs by wink's call
(2026-11-04). The entry stays the proposal.

#### Never rename

- The package keeps its plain name throughout a cycle: Opening step 6 and Land step 1 go, and
  `versioning.md` Dev artifact name is rewritten around the `[install]` table and the dev root.
- Land step 4 becomes `vc-x1 build`, a `vc-x1 run -- -V` smoke check, and `vc-x1 install`.
- Fast and full then differ only by `-- --include-ignored`, the fourteen `dr1_*` tests, and are
  identical once **dr-1 is built in Rust** drops their `#[ignore]`. What fast becomes is decided
  then.

##### Why: never rename

The rename's restore at Land changes the closing after its review, by a squash into a pushed
commit, and Land step 4 names no command.

#### A local commit pair before every push

- Per-rung flow steps 8 and 9 merge, since `commit` pushes what it commits: on the description
  review's go, `vc-x1 commit -s b --title --body` makes the pair and publishes it. With `--no-push`,
  or `[commit] push = false`, it keeps the pair local for a look at the commit itself, description
  and diff together, by `vc-x1 show -s both @-` or gitk, and `vc-x1 push -s b` publishes it after.
  A fix is edited and folded by `vc-x1 squash -s b`, a description fix by `squash -s b --title
  --body`.
- A commit made by hand, a draft below `@` with no description, is described in place by `vc-x1
  commit -s b -r REV --title --body`, its trailer stamped and its partner made, never by `jj
  describe`, and `--bookmark` moves the cycle's bookmark onto it past any nearer one.
- Step 6's work review stays ahead of it.
- Committing vs pushing is rewritten, `vc-x1 commit` committing both repos, `squash` continuing
  a rung, and `vc-x1 push` publishing, and `jj.md`'s vc-x1 push section follows.
- Re-describing: the rule becomes never by hand and never on a published commit, since the pair
  carries trailers from the start and `squash --title --body` is the sanctioned re-describe.

##### Why: a local commit pair

The last closing's review came after its push, so a change meant a remote rewrite and a
squash-push in both repos. Made as a pair, both trailers are known at commit and push carries no
description, so the reviewed commit is the published one. The work review stays first so a
description is not rewritten while the work still moves.

#### The scope flags in the rules

- Every `vc-x1` invocation in the agent-files carries its scope: `sync -s b` at the sync points,
  `commit -s b`, `squash -s b`, `push -s b` in the per-rung flow, and `squash-push -s a` for the
  user's squash-push, replacing `--scope=both` and `-R <agent-dir>`.
- Every argument is a flag, no positional: `commit -r REV`, `push -r REV --bookmark NAME`, and
  `squash-push --bookmark NAME`. Step 9's `vc-x1 commit -s b --title --body`, At rest's `squash-push
  -s a`, and `jj.md`'s vc-x1 push section's opening were brought current in this copy by **feat:
  push, commit, and squash-push take flags only**, the invocations alone, the Committing vs pushing
  rewrite still this proposal's.
- The rest of the commands too, by **feat: the other commands take flags only**: `versioning.md`'s
  adoption names `agent-files copy --from` and `diff --from`, and `hooks.sh`'s guard test names
  `install --dev`, `build-dev`, `install-dev`, and `run-dev` being `build`, `install`, and `run`
  with `--dev` now. A rule that says `install-dev` says `install --dev`.
- And then by profile, by **feat: build, install, and run take --profile**: `--dev` retired to
  `--profile dev`, so `hooks.sh`'s guard test names `install --profile dev`, and a rule that says
  `install-dev` or `install --dev` says `install --profile dev`. The `[build]` and `[install]` keys a
  rule might name are `commands`, `dir`, and the `[profile.<name>]` tables.

##### Why: the scope flags

`-s` is required on the write commands and `sync` once **feat: vc-x1 commit, install, and run-dev**
lands, so the rules' invocations must carry it or fail.

#### The closing is a trapezoid or linear, and converts

- A multi-step cycle closes as a trapezoid by default or as linear on request, the closing commit
  carrying the shape, so nothing is reshaped at Land.
  - `vc-x1 commit --closing <BM|REV>`, naming the cycle's tip, onto the trunk, `--land`'s bookmark
    or `[cycle] trunk`, `main` when absent, makes the closing in the shape `--shape
    <trapezoid|linear>` names, else the `.vc-config.md` key `[cycle] closing-shape`, else
    trapezoid, and prints which decided.
  - Trapezoid is the merge of the trunk and the ladder's tip. Linear is a plain commit on the
    ladder, so the trunk carries one commit per rung, the shape the rules called keep separate,
    renamed.
  - The key is the project's preference, so each adopter sets its own, and one that sets nothing
    keeps the trapezoid.
  - Either converts to the other while the closing is a draft, before Land, by re-parenting the
    same closing: its change id, trailer, and, with the trunk unmoved, its tree are kept.
- Land is sync, then `commit --closing <BM|REV> --land [TRUNK]`, the fast-forward and the push,
  then `vc-x1 install`. To test the closing before it is published, it is made without `--land`,
  its release build, the candidate, installed off the PATH by `vc-x1 install --dir <DIR>` and
  tested by its path, and the candidate lands it, `<DIR>/vc-x1 land -s b --onto <TRUNK>`, and
  installs itself, `<DIR>/vc-x1 install`. The fast-forward lands the tree tested, so `cmp
  <DIR>/vc-x1` with the installed binary finds no difference, and the plain `vc-x1` is replaced
  only by what is on the trunk. The bookmark deleted is the one `--closing` names, or for a revision its
  current or nearest parent bookmark above the trunk, several refused with the list, and the
  branch's other bookmarks are left and named as merged. The trapezoid recipe goes,
  Close-out step 5 keeps the user's choice, the key its default, and carries it out by the flag,
  and `jj.md` Close-out shapes names the two shapes and why squash is not a shape.

- The shape's name: with one rung the figure is a triangle, wink's observation (2026-10-20), so
  "trapezoid" names one case. The tool calls the operation `merge` and the result the merge
  close-out, and the rules' text could follow. `commit --closing` merges into the trunk as it is
  now, a bookmark read where it points, so a `main` that moved during the cycle no longer stops Land
  for a rebase. A linear closing cannot join a moved trunk without rewriting the reviewed ladder, so
  its Land refuses one.

##### Why: a trapezoid by default, linear on request

The trapezoid keeps every rung reachable while `git log --first-parent` reads one commit per cycle,
and it joins a moved trunk. Some prefer a straight history, which linear gives, and the choice is
made at the closing, so the shape is in the commit that was reviewed rather than a reshape at Land
made after it. Converting is cheap while the closing is a draft, so the choice is never final
before Land.

#### A cycle's shape is its branch's commit count

- [Cycle shape](AGENTS.md#cycle-shape)'s "A ladder that shrinks to one rung still closes as a
  ladder" goes: a branch with one commit above the trunk, the opening alone, is a single-step cycle,
  and `commit --closing <BM|REV>` finalizes that commit in place, the bare cycle title replacing the
  opening's, while more commits get the closing commit. Every closing names its tip, by the cycle's
  bookmark or a revision, `@-` the common one, and the count reads the branch, so there is no bare
  form, and `--dead-end` always makes the closing commit.
- Any commit on any line closes: a tip that is not `@-` is closed by a detour, `@` moving onto it
  and returning after, and the commits above it stay as they are.
- A parent wrong after the fact is fixed with `vc-x1 reparent`, never by closing again.
- Every cycle starts with `commit --opening`, a single-step cycle included: its bookmark is created
  and pushed at the trunk, and its one commit begins as the opening, the work squashed into it. So
  Opening step 2, the bookmark's creation and publication by `jj git push`, is `commit --opening`'s,
  and the step names it.
- The closing's title and body may be left out: the cycle title is read from the opening, and the
  body comes from `.vc-config.md`'s `[closing] body`.

##### Why: the commit count

The tool tells the shapes apart by counting, with no title to read, and a cycle need not decide its
shape before its first commit: it is single-step until a second commit makes it a ladder.

#### A commit body is two plain paragraphs

- The problem, then the solution opening "Solution:", ending with the pointer to the record where
  it is at that commit, "See details in TODO.md.", no bullets, and the rung subsection the full
  account: the entry **A commit body is an intro and a pointer to the record** in `## Todo`, its
  rule, specimens, and
  subsection change. The bookends stay pointers, "Opens the cycle ..." and "Closes the cycle ...",
  with the same pointer after them.

##### Why: two paragraphs

A `git log` reader gets what was wrong and what was done in two sentences, and the breakdown is
written once, in the record, rather than twice.

#### Ladder labels from `[0]`

- `notes.md` and the `cycle-model.md` specimen number the ladder's reference labels from `[0]`.

##### Why: labels from zero

Each label then equals its rung's version suffix.

#### A rung may run the acceptance check

- Close-out step 1 says a rung may run it and the closing records the result.

##### Why: a rung may run it

Some checks run only mid-ladder, as in **fix: a missing fixture fails instead of passing**.

#### Dead-end cycles defined

- A cycle abandoned after its opening pushed still closes and lands. Its closing, titled with the
  cycle title plus " dead-end closing", keeps what is worth keeping, reverts the rest, and records
  what was tried, why it stopped, and what replaces it, then lands as a trapezoid like any closing.
- The title cap does not bind the suffix (wink, 2026-10-20).
- In `AGENTS.md` Close-out, `jj.md` Close-out shapes, and `prose.md`'s bookend titles.
- Open: whether a dead end's Land releases its bare version or reverts to the last release.
- First run: **test: the tests build the dr-1 fixture**, closed 2026-10-20.

##### Why: dead ends land

The cycle's agent-repo commits are on `main` and their `ochid:` trailers name its work-repo rungs,
so dropping the bookmark would leave them naming nothing, and landing keeps every rung reachable.

#### Only push publishes

- A bare `vc-x1 commit`, with or without `--land`, and `vc-x1 land`, which is exactly like it, are
  local and need no approval. `vc-x1 push`, exactly like `commit --push`, publishes and needs approval
  unless `--yes`, wink's (2026-10-30). `squash-push` publishes too.
- `[commit] push` goes, so no workspace setting changes what a bare `commit` does.
- `land --push` and `commit --land --push` publish in one step, as `commit --push` does, and delete
  the published bookmark on `origin`, built in **feat: improve trailers**.
- Open: the approval's mechanism. We think it is the Bash hook sending what publishes to the
  permission prompt, since an agent can type `--yes` as easily as anything else. And a bare Land's
  deletion of a published bookmark, which is itself publishing, so `land` deletes the local one and
  `delete-bookmark` the remote one today, wink's preference being that `push` does.

##### Why: only push publishes

A local commit can be reworked and a push cannot easily be taken back, so the safe act is the one
done by doing nothing, and publishing has one place to gate.

#### A cycle stays local until its Land

On trial, from **feat: improve trailers**, wink's (2026-10-30), and the cycle of record if
that cycle's Land works. A new cycle type, for every cycle and not that one alone.

- Every commit is the work half alone, made and kept local: `commit -s w`, the opening's bookmark
  created and not pushed. The agent `@` is neither committed nor pushed before the Land, so
  its change id holds for the whole cycle.
- The work review stays on every commit. The description review goes, wink's (2026-10-30), since a
  local description can still be changed before the Land. Approval per push reduces to one go, the
  Land's.
- The Land commits the agent `@` first, one `ochid:` per work commit, sets each work commit's `ocid:`
  to its SHA and each `pcid:` to its parent's new SHA, trees unchanged, all on the machine. `push -s b --bookmark <TRUNK>` then pushes
  the agent repo first and the work repo after. It folds nothing into the agent commit, so the SHA
  every `ocid:` names holds, and what arrives after it rides the next cycle's agent commit.
- Each rung's `ochid:` is the same agent change, so it stays right when the Land commits it, and only
  `ocid:` and `pcid:` are written then.
- No `squash-push -s a`: At rest's item 2 goes, and "clean" means the work `@` is empty, the agent's
  `@` being the session's data and no finding, as [Acquaint](AGENTS.md#acquaint) says.
- A single-step cycle is the same with one commit.
- A rung that changes the code ends with `vc-x1 install --profile dev`, after validate, wink's
  (2026-10-30), so the next rung runs the code just reviewed.
- The sections it rewrites: Opening step 2, the per-rung flow's step 9, Committing vs pushing, At
  rest, Before any push, and Land. It replaces **A local commit pair before every push** above, a
  rung no longer making a pair, and Every cycle starts with `commit --opening` holds with the
  bookmark local.
- Open:
  - One machine: a local cycle cannot move between machines or survive a lost disk. We think an
    explicit backup push of the work bookmark answers it, and that runs into the rule that what is
    published names nothing unpublished, and into the Land rewriting those commits.
  - Shared repos: **Cycles in a shared repo, by fork and pull request** needs a branch pushed before
    anything lands, so stamping and publishing are separate steps of the Land.
  - One large agent commit: a cycle's whole transcript in one commit nears the limit **fix: snapshot
    fails on a file the size limit refused** dealt with, and `lookup` gets less precise per rung.
  - The published form: publishing per rung stays for a cycle someone wants visible while it runs,
    so this is the default and not the only form.

##### Why: local until the Land

The Land is where the decision is, so one approval belongs there. Nothing published can name a rung,
so reshaping a ladder costs nothing, and the agent commit made first lets every commit link to it, a
SHA included. The first cycle run this way changed its ladder several times at no cost.

#### Work commits link by URL

From **feat: improve trailers**, wink's (2026-11-01). In [ochid
trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers), and **No hand-written trailers** with
it, which now names `vc-x1 commit`:

- `ochid: <chid>` is the partner's change id in the other repo, a bare id on both sides. Published
  `/<chid>`, `/.claude/<chid>`, and `<repo-url>#<chid>` stay valid.
- A work commit also links by URL: `ocid: <agent-repo-url>/commit/<sha>` the agent commit, and `pcid:
  <work-repo-url>/commit/<parent-sha>` each parent, one line per parent in parent order, the trunk first
  on a trapezoid. An agent commit carries `ochid:` alone, so the work-repo is the root of the record.
- The URL is from `origin`, normalized to https, or `file:///<path>` for a local one. `--no-url-trailer`
  leaves the links out.
- The agent commit is made first. `commit -s b` then stamps its real SHA, and `commit -s w` stamps
  `.../commit/xxxx` until a push or a Land makes it. Nothing folds into a commit an `ocid:` names, so `squash` and `push`
  leave it, and the journal rides the next agent commit.
- The count is per agent commit, one `ochid:` per work commit it covers, so a local cycle's Land makes
  one agent commit naming them all.
- A trailer `vc-x1` does not stamp is kept on a rewrite, so `custom.md` may add its own.
- `vc-x1 trailers --update` moves a range of commits to today's form, and a Land refreshes the links
  by the same pass.

##### Why: work commits link by URL

An `ochid:` always means the other repo, so a label or URL in it told a tool nothing, and a change id
survives a rewrite where a URL can go stale. Keeping the URLs on the work commit makes the work-repo
the durable root: lose the agent-repo and only context goes. A commit cannot hold its own SHA, so a
parent's is what names the work-repo at a real commit, and it still says where a forked commit came
from.

#### A local cycle's record may be reworked

- [Cycle-record](AGENTS.md#cycle-record) gains: while a cycle's work commits are local and its agent
  `@` is neither committed nor pushed, an earlier commit's record may be reworked. By default it is
  unnecessary, a later rung revising the record, and it is permissible when the error is significant,
  a word left to the judgment of whoever is reading, with no test fixed, wink's (2026-10-30).
- A reworked commit's tree is validated again, since no validation saw it.
- Once published, Never amended and Re-describing hold as they do. No backfill keeps its own subject,
  that a rung carries no `[[N]]` placeholder, no SHA, and no version.

##### Why: reworking is permitted and not expected

Nothing published names a local commit, so a rework breaks no link. Each commit's ladder is the plan
as it stood then, and seeing it change has value, so reworking stays the exception.

#### Warn rather than refuse

- Open, asked and unanswered (2026-10-24): wink's principle "warn rather than refuse", for a check
  whose failure may be a legitimate shape rather than a mistake, possibly a rule in
  `agent-data/code.md` beside the unwrap alert.
- Its uses so far, in `commit`: a title ending " closing" without `--closing` is warned about, a
  draft below `@-` and a closing with no bookmark are kept local with a line saying why, and a
  closing is never refused for an empty `@`.

##### Why: warn rather than refuse

We think a refusal of a legitimate shape sends the user to raw `jj`, which the tool exists to
retire, where a warning names what was done and leaves it reviewable before the push.

#### Continuation notes go in the closing

- Close-out step 7's notes are written before the closing commit, which carries them, so the
  work-repo is clean from the Land on and At rest's "both `@` empty" holds at the restart. A fact
  known only after the Land is not written: the note names what the next session checks, `main`
  at the closing, the bookmarks gone, the installed version's commit, and machine-local evidence,
  a `cmp` or the Land's output, lives in the agent-repo, published by the user's `squash-push -s
  a` after `/exit`. Provisional, on trial from **feat: vc-x1 land**.

##### Why: notes in the closing

Notes written after the Land have no clean home: a commit after it puts a non-cycle commit on the
trunk after the hard stop, and a squash into the closing rewrites the landmark just landed and
verified. The cost, notes written before the Land they describe, suits a close-out.

#### The exit-status rule is one form, and hooks guard it

- Working practices' Exit status becomes one form, the command under test bare and last, `set -o
  pipefail` first when piped, and the Rules index lists it and One command per invocation.
- The set carries its Claude Code hooks, `agent-data/bash-guard.sh`, `bash-allow.sh`, and
  `inhibit-sleep.sh`, with `agent-data/hooks.sh check|install|test`. Acquaint gains a step 2,
  Hooks, the check, the install the user's.
- `bash-allow.sh` names this family's commands and `inhibit-sleep.sh` needs systemd, so an adopter
  may carry other hooks, and `hooks.sh`'s lists are its own. From **feat: a hook enforces the
  shell rules**.

##### Why: hooks guard the rule

The rule was broken through a whole session with nothing failing, and a deny at the moment of the
slip is the one fix that does not rely on remembering it.

#### Land is the sequence, and `vc-x1 land` its integrate step

- Land stays the rules' word for the whole sequence that makes a cycle permanent: sync, integrate,
  install. `vc-x1 land` names the command alone, its integrate step: `--onto` fast-forwarded and
  pushed and the cycle's bookmark deleted, with no sync and no install. A rule writes "Land" for the
  sequence and `vc-x1 land` for the command, and never one for the other.
- Land is onto the trunk, `[cycle] trunk`, `main` when absent, where the rules now assume `main`:
  Terminology, Sync's "Land syncs", Close-out step 6, and `jj.md`'s Land.
- `vc-x1 land --onto` a bookmark that is not the trunk, a stack of cycles landing together, is the
  command and not the sequence: nothing is installed, and the cycle is permanent only when it
  reaches the trunk.

##### Why: two names for two things

`vc-x1 land` integrates onto any `--onto` and leaves the install to `vc-x1 install`, while the rules'
Land also installs and assumes `main`, so a reader of either takes the word to mean what the other
does. Installing only from the trunk keeps the rule above, that the plain `vc-x1` is replaced only
by what is on the trunk.

#### Scratch files have a durable sibling

Wink's (2026-11-02). Where a thing goes is decided by whether it can be reproduced:

- `tmp/`: disposable, cleared at any time.
- `tmp-durable/`, gitignored beside it: what we would rather not lose and can reproduce, the index of
  **lookup builds an index that correlates the two repos** being the example.
- `notes/`: what must last, committed and pushed. It holds no pointer into `tmp-durable/`.
- Code set aside: a jj commit with a local bookmark, so either of us can address it, never a patch
  file. Unplanned work's "sets those edits aside as a patch" changes to this, and the bookmark is not
  pushed.
- Working practices' Scratch files says so, and the set's `.gitignore` guidance gains the line.

##### Why: a durable sibling

`rm ./tmp/*`, wink's cleanup, took two set-aside patches a continuation note still named, we think on
2026-10-30, and another took three trial scripts on 2026-11-02. A commit survives a cleanup and rebases
with the work, where a patch's context moves. An index costs a rebuild to lose, so it stays out of
`tmp/`, and being reproducible it needs no commit.

#### New repos start from an adopter's rules

- This cycle writes `scripts/vc-dr-apply.sh`, which applies vc-x1's set and skeletons to a bare
  dual workspace, **vc-dr-apply.sh applies vc-x1's rules to a dual workspace**, and the
  Terminology's "template repository is still to come" becomes that: a new repo starts from an
  adopter's set.
- The source is the adopter's working copy by default, what it has not committed included, and
  `--landed` takes its trunk's, wink's (2026-11-04) in `v0.3.1-2`, where `-1` said committed and
  landed.
- Its acceptance check includes making the repo exploring the io_uring interface with the script,
  from vc-x1's `main` after the Land, so the first repo made from v0.3.1 is its test.

##### Why: an adopter's rules

vc-x1 carries the set already, so a template repo is a second copy to keep in step. The working
copy by default, since the rules being worked on are what a test repo is made to try, and `wt6`
got `v0.3.0` from `main` when `v0.3.1-2` was the one in use, wink's finding (2026-11-04).

#### At rest leaves the tail for the next push

- Once a work commit's `ocid:` names the agent commit's SHA, that commit is frozen: squashing the
  session's tail into it changes its SHA, and every `ocid:` naming it dangles. So At rest's second
  step, the user's `vc-x1 squash-push -s a`, goes. What the transcript gains after the final push,
  the push's tool result, the "Published", a recap, stays in the agent `@`, and the next push commits
  it as a commit of its own.
- push's agent half never folds the journal into a commit a work commit's `ocid:` names.

##### Why: the tail stays

A squash after the push rewrites the commit the links point at, wink's finding (2026-11-03).

#### Every unpublished rung's ocid is real before it is published

- `vc-x1 push -s b` commits the agent `@` and pushes it, then rewrites every unpublished work commit's
  `ocid:` to that commit's SHA and its `pcid:` to its parent's, oldest first, each tree checked
  unchanged, as `vc-x1 trailers --update` does, and then publishes the bookmark, one command doing all
  of it. It replaces "each agent commit's SHA is carried by exactly one work commit, the last it
  names" in the cycle **feat: close, land, and push compose with testing**.
- `v0.3.1-1` writes it as the three commands that do it today, `commit -s a --covers`, `trailers
  --update`, and `push -s b`, and one command doing all three, its own name and not an overloaded
  `push`, wink's preference (2026-11-03), as proposed and not built.

##### Why: every link resolves

A rung left at `xxxx` links nothing, and rewriting commits no one else has, their trees unchanged,
costs only their SHAs, wink's (2026-11-03).

#### The user decides when each side commits and publishes

- **A cycle stays local until its Land** becomes the default and not a rule: the user decides when
  the agent side is committed and when either side is published, at any rung, for a backup, a move
  between machines, or someone else to see it, and every push still takes its own approval.
- Any mix of work and agent commits is valid, N:M, wink's (2026-11-02), 1:1 and N:1 the common ones:
  `commit -s b` makes a 1:1 pair, `commit -s w` rungs then `commit -s a --covers <REVSET>` or the Land
  make N:1, and several agent commits naming one work commit make 1:M. The rule is the links, not the
  ratio: a work commit's `ochid:` names each agent change it pairs with, an agent commit's names each
  work commit it covers, and an agent commit whose SHA an `ocid:` names is never folded into.
- Its "no `squash-push -s a` or `commit -s b` before the Land" goes, and "approval per push reduces
  to one go, the Land's" with it.

##### Why: the user decides

The user is always in control, wink's (2026-11-03): the per-rung pairs came first, the local cycle
after, and neither is the only shape, so the rules say what each shape needs rather than choosing one.

#### Step 8 names the bookends

- The per-rung flow's step 8 names `--opening` for the opening and `--closing <BM|REV>` for the
  closing beside the plain `commit -s w`.

##### Why: the bookends

The `wt5` agent opened and closed by the step without tripping, wink's finding (2026-11-03), but a
step that names one form leaves the other two to be found elsewhere.

#### The user chooses the dev build

- `versioning.md`'s **Run your own build** becomes **Which build runs**: when the dev build is newer
  than the plain name, the session asks whether to run it, and otherwise runs the plain name.

##### Why: the user chooses

A dev build left from an earlier cycle is older than the plain name installed at its Land, and
running it by rule runs stale code, wink's (2026-11-04). In the agent-files rather than `custom.md`,
since a default is simpler than an exception.

#### The record stays in TODO.md, and zTOC.md indexes it

- The trial's files retire, [Retired record files](notes/README.md#retired-record-files), and the
  cycle-record and continuation notes stay in `TODO.md`, as the agent-files said all along.
- `zTOC.md` is new, in [zTOC format](agent-data/notes.md#ztoc-format), which `notes.md` adds. The
  Terminology, the Cycle-record's Life, and the Close-out name it, and `vc-dr-apply.sh` ships its
  skeleton.
- In `TODO.md` the ladder's references were `[0]` to `[49]` and the rest started at `[50]`, which
  **The ladder lives in zTOC.md alone** made moot, its references leaving `TODO.md`.
- `vc-x1 commit --closing` moves the block within `TODO.md` and resets the ladder, with no
  `[record]` table.

##### Why: one record file and an index

Why the trial's files retire is in their entry, wink's (2026-11-04). A default is simpler than a
table naming files, and a ladder that owns its numbers moves without renumbering.

#### The ladder lives in zTOC.md alone

- The ladder, each rung with its marker, is `zTOC.md`'s one copy, linking across to the rung
  subsections, which a `#### Rungs` heading holds in the In Progress block, in [The In Progress
  block](agent-data/notes.md#the-in-progress-block) and [zTOC
  format](agent-data/notes.md#ztoc-format).
- Its links are inline, `- N (marker) [<title>](TODO.md#<slug>)`, the version suffix and the marker
  first so they read down their columns, wink's (2026-11-04), and it is one list with no `#
  References` beside it, and **Ladder labels from `[0]`** no longer applies.
- The per-rung flow marks and flips it there, the opening writes it in place of the last cycle's,
  the closing keeps it so the landmark holds it, and acquaint reports from it. `vc-x1 commit
  --closing` no longer touches `zTOC.md`.

##### Why: one ladder that can be clicked through

Two copies kept in step by hand drift, and zed's markdown preview follows a link to another file but
not one within the same file, so only `zTOC.md`'s could be followed, wink's (2026-11-04).

#### Not in it

Each run on its own:

- A rung inserted before pushed rungs: a `jj.md` recipe, once **lookup is tested on a reordered
  ladder** has its results.

### Partner::Landed is any linked partner, and push says a Land made it

(wink + agent, 2026-11-03) Found by the test repo `~/tmp/wt5`: `push -s b` after a plain `commit -s b`
printed "publish main at the partner @- qltqpvvvmryo, which a Land made". `pair::partner` takes any
agent commit naming several work commits, or partnering one that carries an `ocid:`, as `Landed`, and
since the URL trailers every `commit -s b` pair carries one, as will `commit -s a --covers`. The
behavior is right, the agent commit published as it is and the journal left in `@`, since a fold
changes the SHA the `ocid:` names, and the name and the message are left from when only a Land made
such a commit.

- `Partner::Landed` becomes a name for what it is, `Linked` perhaps, and the message says "whose SHA
  the work commit links, leaving the journal in @".
- `Partner::Committed`'s doc says what reaches it now, a pair made with `--no-url-trailer` alone.

### push names a local remote it cannot write

(wink + agent, 2026-11-03) Found by `~/tmp/wt5` with the Claude Code sandbox on: the agent half's push
to the bare repo `~/tmp/remotes/wt5.agent-session.git` was refused with git's "unpacker error", the
sandbox not letting the session write there. Nothing was published, the agent half failing first, but
the message does not say why. Push, on a refused push to a path remote, names the path and says the
write was refused, permissions or a sandbox the likely cause.

### The skeleton's first Todo makes validation run

(wink + agent, 2026-11-03) Found by `~/tmp/wt5`: its first push came before **Configure this
workspace** added `[validate]`, so validation could not run, and the agent said so and pushed on
wink's go, a bend of [Before any push](AGENTS.md#before-any-push) that went unrecorded. The skeleton in
`scripts/vc-dr-apply-base/work/TODO.md` gives a `[validate]` that runs, `full = ["true"]` and `fast =
["true"]` until the project has checks, or `vc-dr-apply.sh` appends it, so a new workspace validates
from its first commit.

- `vc-dr-apply.sh --lang NAME` appends one since **feat: the cycle record returns to TODO.md,
  indexed by zTOC.md, v0.3.1-2 draft** (2026-11-04), `--lang none` the doc checks alone, so what is
  left is an apply with no `--lang`.

### status and validate-desc count the working copies as findings

(wink + agent, 2026-11-03) Found by the test repo `~/tmp/wt5` after its first pair was pushed, both
repos otherwise right:

- `vc-x1-dev st -sb` ends "status: dirty: .agent-session @ has changes", where the rules say clean
  is the work `@` empty, the agent's `@` being the session's data and no finding
  ([Acquaint](AGENTS.md#acquaint), [At rest](AGENTS.md#at-rest)). The verdict says clean, the agent
  `@` named as the session's data.
- `vc-x1-dev validate-desc -sb` reports "1 missing (of 3 total)" on each side, the one being `@`,
  which never has a description. The working copy is left out of the count, or listed as the working
  copy and not as missing.
- The summary counts the links still to be made, wink's (2026-11-03): "N temp", the work commits
  whose `ocid:` is the placeholder `xxxx`, right while their partner is the uncommitted agent `@`. In
  vc-x1 before its backup, six rungs carried it and the summary read "4 valid" with nothing saying
  their links waited on the agent commit.

### Drop the retired flags, subcommands, and positionals

(wink + agent, 2026-11-03) A retired spelling is kept as a hidden argument so it is refused naming
its replacement, and clap's completion offers a hidden argument once its prefix matches nothing
visible: `vc-x1-dev init --use<TAB>` completed the retired `--use-template`. vc-x1 has no users but
wink, so a fix-it no longer pays for the argument carrying it, and every one goes, after
**agent-files(proposal): v0.3.1**, wink's call. `--use-template` went this way in **feat: init drops
--use-template**.

- Flags: `init --por`, `build --dev`, `install --dev`, `run --dev`, and `config --validate`.
- Subcommands: `bot-session`, `validate-bot`, `build-dev`, `install-dev`, and `run-dev`.
- Positionals: the hidden `RETIRED` one in `init`, `clone`, `push`, `squash-push`, `commit`,
  `symlink`, `lookup`, `config`, `validate-anchors`, `validate-desc`, `fix-desc`, `validate-todo`,
  `fix-todo`, `agent-session`, and `agent-files copy`, `diff`, and `size`, and the rev and count
  pair `list`, `desc`, `chid`, and `show` share through `options_flags/common_args.rs`, with
  `options_flags/retired_positional.rs` once nothing calls it.
- The `bot` value of `--scope`, refused as `agent`'s old name.
- Each goes with its refusal, its tests, and what the help and README say of it, and a typed one
  gets clap's "unexpected argument".
- Not retired, so they stay: `agent-session`'s `--no-<item>` negations, hidden to keep its help
  short, and the `--scope` values a command does not take, hidden per command.

### lookup builds an index that correlates the two repos

(wink + agent, 2026-11-02) The hypothesis, wink's: with an index, one agent commit per cycle and one
per rung make no difference to correlating the two repos, so `lookup` builds the index in
`<path>/agent-repo-indexes/`, and it is tested with no UI. A lookup from a transcript line today
returns every work commit its agent commit names, 12 commits and 6,372 lines of output for line 1200
of `520a2a50-1afc-4282-a824-bc78c27b7cc5.jsonl`.

- What it maps: transcript spans to work commits, not agent commits to work commits.
- The evidence, strongest first:
  - the trailers, when they resolve
  - the commit calls in the transcript, for rung boundaries
  - the content each `Write` and `Edit` put into a file, matched against each commit's lines
  - time, last, since a Land rewrites committer time and we think only author time holds
- What it is: durable and rebuildable, wink's (2026-11-02). It survives a cleanup, since we think a
  rebuild is costly, and it is never the source of truth, so losing it costs only the rebuild.
  Transcripts only grow, so we think it updates incrementally.
- Disposable is the first implementation, not a principle, wink's (2026-11-05): the obvious step
  from where we are, kept in `tmp-durable/` because it is expensive to build. A purpose-built
  database is something we will probably need, and what it holds as its own, anchored comments being
  authored rather than derived, decides what is the source of truth. The trial of Zed's Delta, in
  **An interactive app for the todo and the cycle-record**, informs it.
- Staleness: it records what it was built from, so a stale entry is noticed and rebuilt, never served.
- Where it lives: `tmp-durable/agent-repo-indexes/`, the agent's, to confirm, so `<path>` is
  `tmp-durable`. Anything inside the agent-repo's tree gets committed with the session, so not there.
  A workspace's own directory rather than a per-user cache, since each index belongs to one workspace.
- What a rebuild costs is measured by this entry's test, as is the index's size.
- The rung boundaries are not a plain search. All 12 of **feat: improve trailers**'s commit calls are
  in its transcript, among dry runs, retries, a removed first closing at line 3897, an opening under
  its old title, and one inside a compound command at line 4145. A commit run outside the session,
  as that cycle's `land --push` was, is not in the transcript at all.
- The test, on real cycles:
  - A cycle committed per rung is the ground truth: build its index with its trailers ignored and
    compare each span's work commit with what the trailer says, **feat: build, install, and run take
    --profile** one candidate.
  - **feat: improve trailers**, one agent commit for 12 work commits, then gets the same queries, and
    each answer is checked against its rung's diff.
  - A count of wrong, ambiguous, and missing answers is the result, and it says whether the
    hypothesis holds.
- Outside the hypothesis: an index does not change what one agent commit means for durability, since
  a cycle's transcript and rungs still sit on one machine until the Land.
- What it unblocks: the TUI or GUI, wink's thinking (2026-11-02), at least two windows, the work and
  the agent, each linear and correlated by this index, with links that cross where the conversation's
  order and the commits' order differ. Related:
  - **lookup matches a line to its write by content** in `## Waiting`, whose matching is the third
    layer here, its fuzzy matching and privacy questions still open
  - **lookup --history steps a region through the commits that changed it**
  - **Chores retire into a session index (post-viewer)**

### An agent commit mid-cycle backs up the transcript

(wink + agent, 2026-11-02) A local cycle's transcript sits on one machine until the agent commit, and
pushing the work bookmark backs up only the rungs. Any mix of work and agent commits is now valid, so
the agent repo can be committed and pushed mid-cycle, each commit one more group of the relation.

- `land` and the seal stop assuming one agent `@` for a cycle.
- A rung a published agent commit names may later be squashed or dropped, and that commit keeps naming
  a change that is gone. `validate-desc` already reports such an id as `lost`, and when that is
  accepted and when it is a fault is to be decided.
- We think it is a flag on `commit -s a`, or a command of its own, and it needs the approval any push
  takes.

### What a session writes after its final push, sandbox on and off

(wink + agent, 2026-11-04) After the final push the transcript keeps growing, and those lines ride
the next cycle's agent commit, as **At rest leaves the tail for the next push** has it. What is at
stake is what the last agent commit does not hold, and whether any of it is lost. The hypothesis was
that Claude Code's sandbox off writes less after going quiet and at `/exit`. Two samples, about the
same:

- `~/tmp/wt5`, sandbox on: 15 lines after the push, 4 more at `/exit`.
- vc-x1's session `40cfc16f`, sandbox off: 15 lines after going quiet, 3 more at `/exit`, 18 in all.

The samples are in `tmp-durable/session-tail-2026-11-03/`: the `jj diff --stat` runs, the 18-line
tail, and the first 20 lines of the next session, `129195c1`.

- Compare the two tails line by line: which records they are, and whether the next session's first
  lines repeat any of them.
- Decide what, if anything, is lost when the agent repo is not committed after `/exit`.

### The agent repo's size

(wink + agent, 2026-11-02) Size has been an ongoing problem with jj, wink's, and something will probably
have to be done. Measured the same day:

- This agent repo's `.git` is 931 MB, and **feat: improve trailers**'s one session file is 11.6 MB.
- One short `acquaint` session in a test repo wrote 542 KB, so jj's default 1 MiB limit on a new file is
  close from the start. Wink's config turns that limit off, and an adopter's does not. `lookup`, the
  index, and the transcript record all want the whole file, so trimming it is not free.
- A run on the default limit, in a test repo by `init --test-repo`, shows when it is hit.

### commit writes every mix of work and agent commits

(wink + agent, 2026-11-02) **feat: close, land, and push compose with testing** teaches the readers and
validators any N:M mix and leaves `commit` writing 1:1 pairs and N:1 cycles. A work commit naming
several agent commits needs a way to be written, for a rung that outlives an agent commit.

- A link added after a work commit is on the release line can only be one-way, the agent commit naming
  the work commit, since that commit is no longer edited. The fallback for carrying the agent commit's
  SHA is then the next work commit.

### vc-x2: a cycle command, open, close, and land

(wink + agent, 2026-11-10) The cycle protocol's commands are spread over `open`, `commit
--opening`, `commit --closing`, and `land`, so `commit` does both committing and the cycle's
bookends. One `cycle` command holding open, close, and land gives the protocol one home, and
`commit` only commits. For vc-x2, a rename across the agent-files, README, the help, and the tests.

- `cycle open` is `open`, the bookmark made first. It cannot make the opening commit, which comes
  after the record, the ladder, and the bump, so the opening keeps two moments, and `--opening`
  shrinks to the bookend title and its check, or goes.
- `cycle close` is `commit --closing`, still a pair commit with a title, a body, `-s`, and the
  record move, so it calls `commit`'s machinery. `cycle land` is `land`.
- The shape is open: verbs as subcommands, `cycle open`, as **bookmark takes create, delete, and
  move** chose, or flags, `cycle --open`, as **A hyphenated subcommand becomes a command and a
  flag** prefers and the leaning against several positionals.
- Decided with **vc-x1 do runs a recipe of vc-x1 commands** and **close, land, and push compose
  with testing, to re-deliberate**, which cover the same commands.

### The completion script calls the name it was typed as

(wink + agent, 2026-11-12) `vc-x1 cy<tab>` completed to `cycle` with the installed `vc-x1` at
0.85.21, which has no such command. The bash scripts `COMPLETE=bash vc-x1` and `COMPLETE=bash
vc-x1-dev` print define one function, `_clap_complete_vc_x1`, named for the package, and each
hard-codes the binary it calls, `"vc-x1"` or `"vc-x1-dev"`. Sourced one after the other, the
second replaces the first, so both names complete by whichever build was sourced last.

- wink's `.bashrc` works around it with a `sed` on each script, the call rewritten to
  `"${words[0]}"`, the name typed. The fix is that the script needs no `sed`.
- We think either does it: the function named for the binary it was invoked as, as `main.rs`
  already sets the bin name, so two builds define two functions, or the script calling
  `"${words[0]}"` itself. The second also serves a copy under a third name.
- `run_cmd::forward_completion` and `tests/cli_complete.rs` are where completion is handled and
  tested. The other shells' scripts are to check for the same collision.
- Found in **feat: improve vc-x1 cycle flow**, which is not its cause: any two builds whose
  commands differ show it.

### lookup takes seven seconds for one line

(wink + agent, 2026-11-12) `vc-x1 lookup --line src/cycle.rs:1` takes 7.3 s in a release build
and 1 m 11 s in the dev build, `vc-x1-dev`, which is unoptimized, the same code in both. Profiled
in the release build, 2994 samples, the time is in three places, none of them the answer's own
work:

- About 45% is building a substring searcher, `memchr`'s `twoway::Finder::new` and
  `build_forward_with_ranker`. It is blame's reach back, `reach_back` in `src/lookup/blame.rs`,
  whose revset is `roots(diff_lines(substring:"<the line>") & ::<commit>)`: jj-lib diffs every
  ancestor, 708 commits here, and its substring match builds the searcher again for every line
  of every diff. Reading those commits is another 5%, in `zlib`'s inflate.
  - We think a pattern compiled once, `regex:` with the line escaped, removes most of it, and a
    file argument to `diff_lines` narrows the diffs read, at the cost of a line that moved
    between files. The rebuild per line is jj-lib's, and worth reporting there.
- About 20% is the timeline, `Timeline::load` in `src/lookup/window.rs`, which reads and parses
  every `.jsonl` at the agent-repo's root on every run: 251 files, 610 MB, 252 thousand lines
  here, each line a full `serde_json::Value`. It grows with every session.
  - We think it loads only the sessions at or after the window's first, parses only the fields
    it places an entry by, or keeps an index in `tmp-durable/`, which the scratch-file rule
    already names as the place for one.
- Under a second, as a trace timed it, is freeing the parsed timeline after the answer is
  printed, which an exit without the drop saves.

How it was measured, to measure again: `perf record -e cpu-clock:u --call-graph dwarf,16384
--user-callchains -F 499 -o tmp/perf.data <binary> lookup --line src/cycle.rs:1`, then `perf
report -i tmp/perf.data --no-children --sort symbol --stdio`. With `perf_event_paranoid` at 2 a
plain `-g` is refused, since it asks for the kernel's call chains too, and `--user-callchains` is
what it needs. The release binary carries no debug info, so the callers were read from the
source and not from the chains. Found in **feat: improve vc-x1 cycle flow**, which did not
change `lookup`.

### vc-x1 list shows the graph as jj log does

(wink, 2026-11-13) `vc-x1 list` prints one line per commit, its change id, its partner's change
id from the `ochid:` trailer, and its title, newest first. Reading a workspace's state, as in the
test of `~/tmp/wt-21-6` that day, took `jj log -r 'all()'` instead, since only it shows what
`list` leaves out:

- The graph: the lines, a merge's two parents, a trapezoid's figure, and which commits are
  immutable.
- The bookmarks with their remote state, `main*` for a local `main` ahead of `origin` and
  `main@origin` where the remote is, which is what showed that `wt-21-6`'s `main` was unpublished.
- Author, time, and commit id, and `(empty)`.

`list` keeps what `jj log` lacks, the partner's change id, and should keep the two repos side by
side under `-s both`.

- We think the graph is jj-lib's: jj's own `jj log` draws it with the `renderdag` crate jj-cli
  uses, so `list` would take the same, or a template, the partner column added to it.
- `-r` takes a revision with its own `..` notation, and `-r '::@'` is an error, the `::` doubled
  into `::::@`, so `-r 'all()'` may be refused the same way. Its revset handling is to check first.
- Its default is `@` and the commits below it to `-n`. `all()`, or the cycle's branch with
  `main`, may be the better default for reading a workspace's state.
- Found in **feat: improve vc-x1 cycle flow**'s testing, not caused by it.

### validate-anchors misses a citation after a wrapped code span

`vc-config-model.md` cites `[[30]]` right after a code span that wraps across two lines, `cargo`
and `test`, and `vc-x1 validate-anchors` reports `[30]` as defined and never cited, on `main` too
(2026-11-11). We think the code spans are blanked a line at a time, so the span's second half
opens one that swallows the citation. The model is generated, so either the check or the wrap
changes. Found by **feat: improve vc-x1 cycle flow**.

### validate-desc -r takes a revset without a word, and clone leaves a symlink

(wink + agent, 2026-11-10) Found at the closing of **chore: complete vc-x1, ready for vc-x2**, and
carried by **feat: improve vc-x1 cycle flow**, whose ladder left them out.

- `validate-desc -r '::main'` scans one commit, `main`, without a word, a revset form the flag's
  help does not promise. `-r main -n 5000` scans the history.
- `clone` into a scratch directory leaves a Claude Code symlink for it in `~/.claude/projects`,
  removed by hand.

### Two cycles in flight conflict in the record and the version

(wink + agent, 2026-11-11) Every cycle edits `TODO.md`, `zTOC.md`, and the version in `Cargo.toml`,
so two cycles in flight conflict in all three at any merge or rebase, whatever the code did. One
cycle at a time has hidden it, and **Parallel cycles** and **vc-x2: cycle --rebase moves a cycle
onto a destination that moved** both meet it. Found in **feat: improve vc-x1 cycle flow**.

- We think the record wants a file per cycle, or a home outside the shared files, and the version
  a scheme two cycles can each advance.

### feat: add a capability in vc-x1 to insert rung with -sw|a|b|p

(wink + agent, 2026-11-11) A rung found to belong earlier in a local ladder can only be appended:
`cycle --commit` commits the work `@` at the tip. An insert puts a rung between two that are made,
in the work-repo, the agent-repo, or both, and it is the first of three ways to reshape a local
ladder, with **A rung is removed from a local ladder** and **A local ladder's rungs are
reordered**. None is built. They wait for one exhaustive experiment, run on a throwaway workspace
as `scripts/cycle-rehearsal.sh` makes one, to find the gotchas before a flag is designed. Raised
in **feat: improve vc-x1 cycle flow**, whose own late rung was appended and not inserted.

What is known going in:

- Work-repo: `jj new --insert-after <rung>` makes the node and rebases the commits above it,
  which keep their change ids, so every `ochid:` naming them stays right.
  - The commits above conflict in what every rung edits: the version's line, `Cargo.lock`,
    `zTOC.md`, and `TODO.md`. It is **Two cycles in flight conflict in the record and the
    version** met inside one cycle.
  - `cycle --commit` cannot commit the inserted rung, since the cycle's bookmark is above it and
    the bookmark advanced must be at or below the commit. `commit -r <rev> --no-advance` stamps a
    commit made by hand, the way that works today.
  - Each rung above is a tree no validation saw, so each is validated again.
- Agent-repo: the node goes in first, so the session's record of the work accrues in it.
  - The agent `@` is the live session. Moving it checks out an older tree, which rewrites the
    transcript file on disk while the harness appends to it. Untried.
  - One session is one transcript file, so a node inserted during a session appends to the file
    the commits above also append to, and they conflict. We think one session per rung, the
    node inserted and the agent restarted, makes it clean, a new session writing a new file.
  - `jj commit` on an `@` that is not the head leaves the new `@` beside the commits above, so it
    is put back on the tip after.
  - The Land reads the agent `@-` in places, and has not been tried with an agent `@` mid-line.
- The scopes: `w` and `p` insert a work rung, `b` the pair, and `a` an agent node alone, a side
  conversation. It may be about an earlier rung, linked by `--covers`, or off topic, with no
  `ochid:`. The link is by change id, so an agent commit appended at the tip is already tied to
  its rung, and the experiment decides whether `a` needs an insert at all.
- Not dangerous: while the commits above are local an insert only rebases them, and `jj undo`
  takes it back. We think it follows `squash`, warning of a published draft and refusing one in
  jj's `immutable()`, and takes no `--yes`, which is the approval of a forced push.

### A rung is removed from a local ladder

(wink + agent, 2026-11-11) A rung that should not have been made is removed by `jj abandon` by
hand, with its heading and its ladder line. A remove does the three together. It is the
dangerous one of the three reshapes, since it discards work where an insert and a reorder move
it. Part of the experiment in **feat: add a capability in vc-x1 to insert rung with -sw|a|b|p**.

- We think it is dry-run by default, as `fix-desc` and `fix-todo` are, showing what it would
  drop and dropping it only when told, and that it takes no `--yes`.
- The agent side: the journal is never deleted, so the rung's agent commit stays, its `ochid:`
  naming a commit that is gone. What its trailer then says is to decide, `trailers` knowing a
  `lost` state already.
- A published rung's removal is a forced push, under the approval any push takes.

### A local ladder's rungs are reordered

(wink + agent, 2026-11-11) Two rungs committed in the wrong order are swapped by `jj rebase` by
hand, and the ladder then disagrees with the record until the headings are moved too. A reorder
moves the commits and the headings together and makes the ladder again. Part of the experiment
in **feat: add a capability in vc-x1 to insert rung with -sw|a|b|p**.

- It meets the insert's conflicts, in the version's line and the record, at every rung moved.
- The agent-repo's order is the order things happened in, so we think a reorder is the
  work-repo's alone, the pairs still linked by change id.

### vc-x2: cycle --rebase moves a cycle onto a destination that moved

(wink + agent, 2026-11-11) A cycle whose destination moved is joined by a merge, at the closing or
at the Land, or rebased, and vc-x1 leaves the rebase to `jj rebase` by hand, decided in **feat:
improve vc-x1 cycle flow**. We think it is one mode, `cycle --rebase <DEST>`, usable at any point
before the Land, so it serves a cycle brought up to date mid-way and a linear close alike, the two
chaining as `cycle --rebase <DEST> --close`.

- It refuses a cycle whose bookmark was pushed unless the user asks, a forced push following.
- It stops at a conflict and reports it, and says the rebased commits are new and not tested.
- The `ochid:` trailers are change ids, which a rebase keeps, so the agent-repo needs no rewrite.
- Every rung edits `TODO.md`, `zTOC.md`, and the version, so a rebase onto a destination where
  another cycle landed may conflict at each rung, whatever the code did.

### vc-x2: the agent-repo stays on main, agent branches set aside

(wink + agent, 2026-11-10) A decision for vc-x2 to inherit, not work. Only the work-repo has a
cycle bookmark, the agent `main` moving with every push. A cycle bookmark in the agent-repo too,
landed with `--onto main`, would make the two sides alike in `push`, `squash`, and `land`, and it
was weighed and set aside, wink's call: the agent-repo stays on `main`.

- The agent `@` holds the live transcript, so it cannot follow the work-repo between branches: a
  move rewrites the files on disk, truncating the live `.jsonl` to the other branch's copy, or,
  with the files kept, carries one branch's lines into the other's commits. A session works on more
  than one branch, so a rule of one branch per session is not feasible.
- Two branches appending to one `.jsonl` conflict at merge, though every line differs, both being
  inserts at the file's end, and separate files merge clean, a test of 2026-11-10. A resumed session
  appends to its original file. The resolution is always both sides by each entry's time, git's
  `merge=union`, which we think jj does not honour, so an auto-merge is feasible, and it was not
  enough, the live file being the obstacle.
- On `main` alone the agent `@` never moves, every agent commit lands on one line in time order,
  nothing is merged, and the `ochid:` trailers pair the repos by change id, so an agent commit needs
  no branch to be found from its work commit.
- One session per workspace stays a rule, not enforced,
  [session-lock.md](agent-data/session-lock.md) the tried `flock` should that change.

### Build config moves to an xtask

(wink + agent, 2026-11-03) vc-x1 does two things. It manages a dual repo, toward its real goal of
showing why and how the work was made, and it validates, builds, installs, and runs. The second half
lives in `.vc-config.md` as `[validate]`, `[build]`, `[install]`, and `[profile.*]`, and `[test]
fixtures` is vc-x1's own test suite's setting, none of which a dual repo needs. We think they got
there because vc-x1 tests itself, the tool that commits being the tool that changes, which most
adopters are not.

- The build half becomes a `cargo xtask`, a binary crate in the workspace run through the alias
  `xtask = "run --package xtask --"`: `cargo xtask validate [--fast]`, `build`, `install`, and
  `run`, with what the tables say today as its code. It builds and installs vc-x1 without being
  vc-x1, so nothing replaces itself.
- `.vc-config.md` keeps hooks: the command that validates and its fast form, and the one Land runs to
  install, each named by the project, `cargo xtask validate` here and `just check` or `make test` for
  an adopter in another language. vc-x1 runs the hook and records that it passed, which is provenance,
  and is the half of **push refuses when no full validate passed on the current tree** that stays in
  vc-x1.
- `[profile.dev] suffix`: `main` moves into the library behind two thin binaries,
  `src/bin/vc-x1.rs` and `src/bin/vc-x1-dev.rs`, so `cargo install --path . --bin vc-x1-dev --debug`
  puts the dev build beside the release. Whether cargo's install replaces a running binary as safely
  as today's `cp` then `mv` is to be checked.
- `[test] fixtures` becomes a constant or an environment variable in `src/test_helpers.rs`.
- Not `build.rs`: it runs before the crate compiles, so it cannot see the binary it would install, a
  cargo it spawns waits on the build's lock on `target/`, and an install from it is a side effect
  outside `OUT_DIR`. It keeps the version stamp, and could add `-dev` to it, its `PROFILE` naming only
  `debug` or `release`.
- Not a separate product binary: a `vc-build` or `cargo-vcx` competes with `just`, `make`, and
  `cargo-make`, and an adopter outside Rust uses its own. It can be extracted later, if several Rust
  adopters want the same flow.
- What it retires or rewrites: **validate takes any profile** and **install has a role and a
  profile**, which reshape the tables this removes, the flag that copies the language tables, agreed
  for the next `init` rung, and the tables **vc-x1 config and init write a simple config** has `init`
  show. **The vc-config program: finish the surface, then shrink it** shrinks by them. The schema in
  `vc-config.md` is compiled in, so `validate-config` and every workspace's `.vc-config.md` change
  with it, only wink's today.

### vc-dr-apply.sh applies vc-x1's rules to a dual workspace

(wink + agent, 2026-11-03) `vc-x1 init` makes a bare dual workspace and stops: the two repos and
their remotes, on GitHub or local and bare with `--test-repo`, `.vc-config.md` with `[repos]` and
`[remote]`, the `~/.claude/projects` symlink, and the first commit. What goes in it is not vc-x1's
problem. A script fills it, `scripts/vc-dr-apply.sh [--from <vc-x1>] [--rev <rev>] [<workspace>]`,
written in the cycle of **agent-files(proposal): v0.3.1**, wink's call, and vc-x1 is its source, the
adopter whose rules a new repo starts from, its working copy's by default and `--landed` its
trunk's. The first use is the repo exploring the
io_uring interface.

- One script for both uses, since they differ only in the `init` before it, which the script never
  runs. A new project is `vc-x1 init --target foo`, then the script on `foo`. Converting an existing
  repo is `vc-x1 init --target . --adopt`, then the script on `.`.
- It lives in vc-x1, a tool shipped beside it and not part of the binary, and moves to a repo of its
  own if it grows or becomes an app. "Apply" names what it does to the workspace's files, and the
  `vc-dr-` prefix still reads once it is on the PATH.
- What a new repo takes, little of it copied:
  - Copied from vc-x1 at the commit: the set, `AGENTS.md` and `agent-data/`, and the generic files,
    `LICENSE-*` on both sides and the agent repo's `memory/MEMORY.md`.
  - Written from skeletons in the source's `scripts/vc-dr-apply-base/`, so they are reviewed
    like the rules: `TODO.md` in [Todo format](agent-data/notes.md#todo-format), since vc-x1's holds
    its own records; `custom.md`, since vc-x1's is gitignored and local; and the agent repo's
    `README.md`, named for the new repo, since vc-x1's says `vc-x1.claude` and "bot repo".
  - `.gitignore` lines, the generic ones alone, `/tmp`, `/tmp-durable`, and `/custom.md`, never
    vc-x1's `/target` or `/.fixtures`.
  - `.vc-config.md` is `init`'s, never copied.
- From a commit, never the working tree: `vc-x1 agent-files copy` reads its source's working tree,
  which in a cycle changing the rules holds them half changed. The script reads vc-x1 at `--rev`,
  `main` by default, `git -C <vc-x1> archive <sha> AGENTS.md agent-data` and `git show <sha>:<file>`,
  with the ref resolved to a SHA first, and the commit that takes the applied files records "vc-x1 at
  `<sha>`" in each repo, provenance of the project's start, so a later look at what the rules changed
  since is a diff from that SHA. `--from` defaults to the vc-x1 the script sits in.
- Overwriting, the hazard of a convert, guarded in order of strength:
  - A clean start: the script refuses unless both repos' `@` are empty, so every write is a diff and
    `jj restore` undoes them all. The backstop the rest keep from being needed.
  - A staging directory under `tmp/`, never an extraction into the workspace. A staged entry that is
    absolute, holds `..`, or is a symlink is refused, and so is a write through a symlink in the
    workspace.
  - A plan before any write: each path the script would write is new, identical, or differs, a file
    meeting a directory included, judged by what is on disk, so an untracked or ignored file counts.
    New is written, identical skipped.
  - All or nothing: any path that differs writes nothing, prints the plan, and exits nonzero. Nothing
    is ever half applied. `--dry-run` prints the plan and stops.
  - Each way past a difference is explicit: `--keep-existing` skips them, `--beside` writes
    `<file>.vc-dr` beside each for a hand merge, and `--overwrite <path>` takes one named file at a
    time. No blanket `--force`.
  - One automatic merge, `.gitignore`, which takes the lines it lacks, a union of lines being the one
    merge that loses nothing. `README.md`, `LICENSE-*`, `AGENTS.md`, `CLAUDE.md`, and `TODO.md`, the
    likely collisions in a convert, are never merged, `TODO.md` least of all, since the agent-files
    require their own shape of it.
  - No commit: the script ends by printing each repo's `jj diff --stat` and the `vc-x1 commit` to run
    after review.
- `init --adopt` already holds to the same rule, checked in **feat: init drops --use-template**: an
  existing `.gitignore` is given only the lines it lacks, and nothing else is written.
- A shell script, not an xtask: it is language-neutral, as the repos it fills need not be Rust.
- Deferred: applying a whole template repo, every committed file of its work side and a fixed
  manifest of its agent side, from a local path or by GitHub's tarball and contents endpoints, until
  wink wants a template beyond the rules. The design is in this entry's history at **feat: init
  --test-repo makes a test repo**.
- Not a separate app yet: one earns its place by updating a project that drifted from its source, as
  `copier` does with a three-way merge, where `cargo-generate` only creates. vc-x1 already updates
  the rules by `agent-files diff` and `copy` and the messages repo.
- What `init` owes the script: it never prompts, it does one thing, and `[project] name` makes the
  project name one key rather than a `sed`.
- What it retires: the template repos beside vc-x1, `vc-x1-template`, `vc-x1-template-2`,
  `vc-x1-work-repo-template`, `vc-x1-bot-repo-template`, and `vc-x1-codex-template`, each on wink's
  go after checking which siblings name it, and `[family] template`, today the default source of
  `agent-files copy` and `diff`, which needs another default.

### validate takes any profile

(wink, 2026-11-02) The user tests between close, land, and push, and that test may be `validate` under
any profile, where `[validate]` has only `full` and `fast`, neither tied to a profile. Wink's model of
the other half: install under any profile, named by `profile.<name>.suffix`. That is the opposite of
**install has a role and a profile**, which moves `suffix` out of `[profile.*]`, so that entry is to be
retired or rewritten.

### The exit code says what failed

(wink, 2026-10-30) `vc-x1 <...>`'s exit code should indicate what failed as precisely as possible.
Today nearly every failure exits 1 by the runner's Ok-or-1 mapping, clap's usage errors 2, and only
`agent-files diff` returns a code of its own.

- A documented table, stable once published, with an error type that carries its code, so a command
  returns one by what happened and not by a string. The message stays the explanation, and the code
  is for what acts on the result: a script, a hook, an actor's reply, an agent.
- What the code most needs to say, a first cut:
  - which step of a composed command failed, so `land -s b` says the agent closing, the work closing,
    the agent push, or the work push, and a retry knows what was done
  - refused before anything moved, as against failed partway
  - a guard `--yes` would pass, as against an error it would not
  - a push rejected by a lease, as against the network.
- The shape, wink's (2026-10-30): read in hex, the upper digit is the stage, broader, and the lower
  digit the type, narrower.
  - The upper digit runs 0 to 7, since the shell owns 0x7E and up: 126, 127, and 128 plus a signal.
    So a set top bit always means killed, never one of ours.
  - Stage 0 is before anything moved, so `refused` needs no type of its own and today's 1 and clap's
    2 keep their meanings. Stages are a command's own, `land -s b`'s being the agent closing, the
    work closing, the agent push, the work push, and so on, and each is documented with it.
  - Types are global, the same digit meaning the same thing in every command: a guard `--yes` would
    pass, a rejected lease, the network, a conflict. A lower digit of 0 is unspecified.
- Open: **status prints a verdict per repo and exits with a bit per side** uses 1, 2, and 3 for a
  verdict, which collides with stage 0's types, so one of the two moves. Eight stages may be short
  for a long composed command.

### bookmark takes create, delete, and move

(wink + agent, 2026-10-30) `move-bookmark` and `delete-bookmark` become `vc-x1 bookmark
create|delete|move`, and `commit --no-advance` becomes `--no-move`, the same word as the command.

- Subcommands and not flags: each verb takes different options, so each gets help of its own and clap
  refuses what does not apply. It matches `jj bookmark`, **Nest the validate and fix commands**, and
  `agent-files copy|diff`. "Every argument is a flag" binds a command's arguments, not its verbs.
- The retired names are refused, naming the new ones, as the retired flags are.
- Only push publishes, in **agent-files(proposal): v0.3.1**: the verbs are local, so `delete` stops
  removing the bookmark on `origin` and the remote side takes `--push`. `move-bookmark` is to be
  checked for the same. This answers a Land's deletion of a published bookmark.

### install has a role and a profile

(wink + agent, 2026-10-30) `[profile.dev]` holds both how a binary is built and what it is installed
as, so `vc-x1-dev` reads as the `dev` profile. Wink's model: they are two choices. The profile is how
it is compiled, `target/<profile>/<name>`, the name from `Cargo.toml`. The role is what it is: bare
`vc-x1` the released version and `vc-x1-dev` the development version, either from any profile.
`build` and `run` take a profile alone, and `install` alone has a role. Historically the install was
`cargo install --path . --locked`, sometimes with `--root` and a copy, before profiles existed.

A proposal, the agent's, wink's to try (2026-10-30):

- `[role.released]`, `default-profile = "release"`, and `[role.development]`, `suffix = "-dev"` and
  `default-profile = "dev"`. `suffix` leaves `[profile.*]`, and `[install] commands` says
  `{role.suffix}`.
- `install --as released|development`, the default the running binary's role by its name, so
  `vc-x1-dev install` stays development, and `--profile` overrides the role's default.
- A released install refuses a dirty build, a commit not on `main`, and a suffixed version unless
  `--force`, where today it only reports them. A development install reports them.
- The copy stays configured: `[install] commands` holds `cp {profile.bin}` and `mv`, or a project may
  list `cargo install --path . --locked --profile {profile.cargo} --root <dir>` and copy from
  `<dir>/bin`, which needs no path rule and, by cargo's help, shares the workspace's target directory.
  `bin` stays either way, since `run` needs it and `[build] commands` is not cargo-specific.
- A config with `suffix` under a profile is refused naming the move, or read as the role's for a
  while. Open.

### desc takes a title, body, and trailers

(wink + agent, 2026-10-30) Overwriting a commit's description takes three commands and leaves gaps:
`squash --title --body` on `@-` alone, needing both and folding `@`, `fix-desc --title` with no body
and only on a commit whose `ochid:` has an issue, and a hand-run `jj describe` for a trailer or a title. Wink's: `vc-x1 desc` gains optional `--title`, `--body`,
`--ochid`, and `--ocid`, in any combination, what is left out kept. With none it reads as it does
today. The agent's, to be agreed:

- A write requires `-r`, one commit in one repo, so a default of `@` or a range writes nothing by
  accident, and `-s both` is refused, each repo's trailers being its own.
- `--ochid` repeats and replaces every `ochid:` line, since a Land's agent commit carries several, and
  `--ocid ""` removes the line.
- Only the description changes, nothing folds, and the tree is checked unchanged.
- A published commit is refused unless `--yes`, and `--dry-run` prints the result.
- `squash --title --body` and `fix-desc --title` then retire, and `commit -r`'s refusal names `desc`.

### abandon and revert set a mistake aside

(wink + agent, 2026-10-29) The dead end of **feat: undo takes back the last vc-x1 run**: what the
agent made, or a trailer names, is never deleted, so a mistake is set aside going forward rather
than taken back.

- Abandon: a line's tail is kept under a permanent `dead-end-<id>` bookmark, pushed so every
  `ochid:` still resolves, the cycle's bookmark moved back to the chosen commit with a leased push,
  and a `commit -s b` on top saying what was dropped and why. The `<id>` sorts by time.
- Revert: a change in the middle is taken back by a new commit at the tip applying its inverse,
  never by removing it and rebasing what followed, which rewrites published commits into trees no
  validation saw.
- Draft lines only: nothing on the trunk's remote is abandoned or reverted by these, the trunk
  fixed forward by a new cycle, and a bad release marked rather than renamed.
- The agent-repo is never rewound while the agent runs. After `/exit` a rewind is possible, and it
  truncates the transcript, so it is a deliberate act, never a side effect.
- Work only a person touched and nothing references is theirs, raw `jj undo` included.
- Open: the name, `dead-end-<id>` beside the dead-end closing's own term, the `<id>`'s form, and
  whether abandon discards when nothing references the tail.
- **squash-push shows what it takes, and push follows jj** leaned on undo, its "local acts are not
  refused, since undo takes them back", and is to be reread against this.

### vc-x1 is an actor

(wink + agent, 2026-10-30) `vc-x1` becomes a service the user installs, and a client sends it
asynchronous messages. From **feat: improve trailers**, where a Land is to run after the agent
goes quiet, so the transcript it commits is whole: first as a detached process, wink's technique
from the olden days, and then as a message to the actor.

- Its lifetime is the user's: the service is installed, started, and restarted by them, systemd or
  the like, and no agent manages it.
- One actor handles one message at a time, so writes through it queue rather than collide. Raw `jj`
  never goes through it, so that stays a rule.
- A reply is the outcome, so "land finished or failed" has a defined form and place. A reply does not
  wake the agent, since waking it writes to the transcript the Land is waiting to go quiet.
- The client:
  - no service running fails at once and says so, rather than queueing a message nothing reads
  - client and service compare versions and report a mismatch, since a Land installs a new binary
    and a running service keeps the old code until restarted
  - the Land stays an ordinary function, so it runs in the foreground with no service.
- Authority: any local process can send a message, so the user's go still comes through the agent and
  the rules.
- What a finished Land can start, by another message:
  - an agent that checks `main` in both repos, the deleted bookmarks, and the installed version,
    read-only and outside the dual-repo protocol
  - a restart of the agent for the next step, automating [Close-out](AGENTS.md#close-out) step 7,
    which begins with an acquaint and inherits no approval, so a restart is no unattended chain of
    pushes.
- Open: the transport, we think a file mailbox in a gitignored directory, which survives a crash and
  works on every platform **vc-x1 runs on Windows and macOS** names.

### squash takes --include

(wink + agent, 2026-10-24, from **squash takes paths**, reshaped 2026-10-28) `squash -s w` folds all
of the work `@` into `@-`, so folding one file's change while `@` holds the next rung's edits means
`jj squash <path>` typed, or the edits parked first. Filed from the live testing of **feat: vc-x1
closing and land**. After **feat: undo takes back the last vc-x1 run**, which its testing uses, and
**squash-push shows what it takes, and push follows jj**, which frees `--yes`, wink's order
(2026-10-28). The rest of the selection is **squash and commit take --exclude**.

- Selection: `--include <path>`, repeatable, a file or everything under a directory, absolute or
  relative to the current directory. What is not included stays in `@`.
- A path's repo is the innermost repo of the pair holding it, so one list may mix work and agent
  files.
  - The repo must agree with `-s`, a mismatch refused naming the scope that takes it.
  - A path in neither repo is refused. Content not yet in the tree comes in first by **import
    brings content into the tree**.
- A path matching nothing in `@` is warned about, by name, which catches a typo.
- Approval by default: squash lists what folds and what stays, grouped by repo, each file with its
  status and `+n -m`, and asks. `--yes` folds without asking.
  - The mechanism is squash-push's: `--ask` and `--yes`, a config default, `squash.yes` false where
    `squash-push.yes` is true, a non-terminal stdin and a declined prompt each an error.
  - With no terminal, nothing changes and the exit names `--yes`, so the agent shows the list and
    reruns on the user's go. A code of its own for "needs approval", apart from a failure, is open.
  - The approval is of paths, not content. The agent `@` grows with the session between the list
    and the rerun, so the folded list is printed again after the fold.
- A squash without `--include` folds all of `@` without asking, as today.
- Open: the folded `@-` is a tree no validation ran on, since validation runs on the working copy.
  Until a revision can be validated, a fold by paths before a push needs `@-` checked.

### sync says one repo when the work @ is dirty

(agent, 2026-10-25) `vc-x1 sync -s b` with uncommitted changes in the work `@` prints the work
repo's lines and then "1 repo is up to date, nothing to sync", the agent repo unmentioned, where
`--dry-run` at the same state reports both. Seen at the Land of **feat: vc-x1 closing and land**
and reproduced at the opening sync of **feat: vc-x1 commit takes a revision**. Whether the agent
repo was checked and only its line dropped, or was skipped, is the first thing to find.

### Dry runs interleave: commit, push, and sync

(wink + agent, 2026-10-26) `commit`, `push`, `sync`, and `squash-push` each run their dry run as a
branch of its own, an `if dry` that says the steps apart from the code that does them, so a check
or a step added to the real run is missed by the dry run unless someone remembers it. Found in
`land::run` while moving it out of `commit` in **feat: vc-x1 land**, whose dry run skipped the
conflict check, and fixed there.

- Each brought to the form `land::run` has: every check in both modes, and each step its line and
  then the step behind `if !dry`, no early return. A step that depends on a result a dry run does
  not make says it from the planned value and calls nothing.
- Takes **commit's dry run reports the push as the advance leaves it** with it, the same fault.

### agent-files(proposal): dry runs interleave

(wink + agent, 2026-10-26) The dry-run form, wink's call over plan-then-run (2026-10-26), is a
convention of this codebase written nowhere, so a new command's dry run is shaped by taste. A
proposal adds it to `agent-data/code.md`:

- Checks never depend on the dry run, so it refuses what the real run refuses.
- Each change to a repo or remote sits under its own line, `if !dry { ... }`, so a step added
  brings its dry-run line, and no `if dry { ...; return }` precedes the steps.
- Why interleaving rather than a plan of steps run or printed: `sync` and `commit` decide later
  steps from earlier results, which a plan made up front would have to simulate, and one form
  everywhere outweighs a neater one in `land` alone.

### squash and commit take --exclude

(wink + agent, 2026-10-28) The rest of the selection, after **squash takes --include**: `commit` has
the same gap in `jj commit <paths>`, and a fold of everything but the next rung's files is often the
shorter list.

- Selection: everything in `@`, narrowed by `--include` when given, less `--exclude`, exclude
  winning, with `--include`'s path forms, repo routing, and approval.
- `commit` takes `--include` and `--exclude` too, the flags shared in `options_flags`, so the rung's
  files are committed and the rest left in `@`.
- Globs, opt-in, as jj's `glob:`, so a plain path never surprises.
- Output: text, one file per line, by default, and JSON by a general flag, `--json` or `--format
  json`, with a versioned schema, for every command rather than squash alone.
- Paths select whole files: a file holding both the fix and the next rung's edits cannot be split.

### push --from --to moves a bookmark and publishes it

(wink + agent, 2026-10-28) Publishing a bookmark somewhere other than the line's `@-` is two
commands, `move-bookmark --bookmark NAME --target REV` and then `push -r REV --bookmark NAME`, and
push's `-r` and `--bookmark` read as a revision and a name rather than a move. Filed at the
opening of **feat: undo takes back the last vc-x1 run**, while it was still the push cycle.

- `push --from <BM|REV> --to <BM|REV>`: `--from` names the bookmark to move, by name or by the
  commit it is at, the line's nearest when left out, and `--to` where it goes, a revision or another
  bookmark's commit, `@-` when left out. "Or some such": the names are open.
- Forward unless `--rewrite`, as every push.
- Whether it replaces `-r` and `--bookmark`, and whether `move-bookmark` stays as the local half,
  is open.

### import brings content into the tree

(wink + agent, 2026-10-28) Code from an unrelated source, a path outside the pair or a URL, has no
command: it is copied in by hand, and nothing records where it came from or which version it was.

- `--import <path|URL>` with a destination writes it into `@`, and `--include` then takes it into a
  commit like any file in the tree, so a fold never fetches. Its home, a command of its own or a
  flag on `commit` and `squash`, is open.
- Pinned: a git source records the commit SHA it got, any other source a content hash, never a
  moving branch.
- Provenance: an `imported-from: <url>@<sha>` trailer, so `git log --grep imported-from` finds every
  import, and a later check can re-fetch and verify it.
- Trust: content from a URL is untrusted, so the approval list shows sizes and marks binaries, and
  licensing is a human's question.
- Sources: a plain file over https first, then a git URL with a ref and a subpath, through jj-lib
  with no `git` spawn, then tarballs and crates.
- Re-import after local edits is a three-way merge against the last import, the problem git subtree
  and Copybara solve. A first form refuses to overwrite a locally edited file.
- Precedent: `agent-files copy --from` already imports files from the template repo.

### agent-files(proposal): vc-x1 is the interface, jj the engine

(wink + agent, 2026-10-28) The rule **jj, not git** says version-control operations use jj, but in
a cycle the user does not and should not use jj. vc-x1 carries the model jj does not know, the
pair, the scope, the opening and closing, the trailers, and Land, with jj-lib as its engine.

- The rule becomes: in a cycle, version-control operations use vc-x1, and a reach for raw jj is a
  vc-x1 gap, filed as a Todo rather than written into the protocol as a workaround.
- Already said in parts: v0.3.1's **Warn rather than refuse** names raw jj as what "the tool exists
  to retire", and its re-describe rule retires `jj describe`. The bookmark moves by `jj git push`
  are what is left.
- Open: whether read-only jj, `jj log`, `jj st`, and `jj op log` for recovery, stays allowed as
  inspection and the escape hatch. The agent's suggestion is that it does.
- Its own proposal, after **agent-files(proposal): v0.3.1** (wink, 2026-10-28).

### Parallel cycles

(wink + agent, 2026-10-28) The model runs one cycle at a time, and the work must scale to several
changes at once, as Anthropic's Scion does for its agents. A design entry, its findings for a
`notes/` file.

- Sources: [Redesigning source
  control](https://www.anthropic.com/research/redesigning-source-control), [The Pulse on
  Scion](https://blog.pragmaticengineer.com/the-pulse-anthropic-builds-its-own-source-control-system/),
  and the prototypes [CarterT27/scion](https://github.com/CarterT27/scion) and
  [tinyhumansai/openscion](https://github.com/tinyhumansai/openscion).
- What carries over from Scion:
  - making a change never touches a ref, and landing is a rebase onto the trunk as it is now and a
    compare-and-swap on the ref, retried, with no global lock
  - conflicts are expected and fixed after the merge, and humans review intent rather than every
    line
  - v0.3.1's `commit --closing` already merges onto the trunk as it is.
- What assumes one cycle, each a design point:
  - the working copy, one `@` per repo: a jj workspace per cycle, each on its bookmark
  - the record, one `## In Progress` and one `## Closed` in `TODO.md`: a record file per cycle
  - the version, bumped at each opening, so two cycles claim the same next one: assigned at Land
  - the agent-repo, one session's journal per `@` with trailers pairing commits: a workspace or a
    directory per session
  - the build and install, one `target/` and one `vc-x1-dev`: per workspace, a profile per cycle
  - the messages repo's single-clone mutex, its README's "No second machine": member-scoped ids.
- The human gates come first: per-push approval and the hard stop are what make the protocol
  trusted now, and the decision is which of them move to Land or to a review of a cycle's intent.
  We think the rest is mechanical.

### An interactive app for the todo and the cycle-record

(wink, 2026-11-04) `TODO.md` and `zTOC.md` are read in an editor's markdown preview, and zed's
cannot follow a link within the same file, which is why the ladder moved to `zTOC.md`. An
interactive app over them would show a cycle's ladder beside its rung subsections, and the Todos by
rank, and follow every link, for whoever reads the record, agent or person, wink's hope that it is
easier for everyone.

- It reads what is there: `TODO.md` in [Todo format](agent-data/notes.md#todo-format) and `zTOC.md`
  in [zTOC format](agent-data/notes.md#ztoc-format), so it needs no new store.
- We think it starts as a viewer, and that editing, reprioritizing a Todo or marking a rung, comes
  after, since the agent-files say who writes what and when.
- With it, `zTOC.md` may no longer be needed, its index being the app's view.
- It shows the history too, wink's goal (2026-11-05): a cycle's ladder, its rungs, and each work
  commit beside its agent session through the `ochid:` links, which no jj viewer reads.
- Zed is not where it lives: its extension API, WASM, has languages, themes, slash commands, and
  context servers but no UI panels, wink's reading (2026-11-05). We think the routes are a TUI,
  ratatui in zed's terminal panel or any terminal, and a language server for `TODO.md` and `zTOC.md`
  giving go-to-definition and hover on anchors, which follows the same-file links the preview
  cannot.
- Try Zed's Delta before building, wink's find (2026-11-05): https://zed.dev/blog/introducing-delta,
  private beta 2026-08-12 and public beta 2026-09-16. Its goal is vc-x1's, code kept with the
  conversations that made it, and its means the opposite: DeltaDB, a database replicated in real
  time, capturing every edit between commits, where vc-x1 links each commit to an agent-repo commit
  by `ochid:`. The git repo stays plain for those who never open it, transcripts stay whole, and it
  syncs Claude Code sessions. What it has and vc-x1 has not: the history view, multiplayer, and
  comments anchored to lines, diffs, plan steps, and thinking blocks that follow the code as it
  changes. What vc-x1 has: history that is plain git, on any host and offline, and the protocol,
  cycles, approvals, and agent-files. Whether DeltaDB can be self-hosted is unchecked. We think
  vc-x1 keeps the record and the protocol, and Delta or our own viewer is a window onto it, which a
  trial on a test repo like `t1` decides. We think an anchor of a change id and a line, which jj
  keeps through a rewrite, gets part of the way to anchored comments.
- Anthropic's Scion, the inspiration of [Parallel cycles](#parallel-cycles), whose sources it
  shares: changesets in a write-ahead log with content-addressed blobs, refs moved by a
  compare-and-swap on a Spanner row, and reviewers examining a change's intent rather than every
  line. Wink suspects its database couples the conversation and the work more tightly, anchored
  comments one example, and `Redesigning source control` names no stored conversation and no
  anchored comment (read 2026-11-05).

### A hyphenated subcommand becomes a command and a flag

(wink, 2026-11-04) Every hyphenated subcommand becomes `<cmd> --<option>`, wink's preference, so
`validate-config` is `validate --config`, and the `validate-*` commands are one `validate`. 0.80.6
went the other way, `config --validate` to `validate-config`, a mistake by wink's word.

- Today's: `validate-agent`, `validate-desc`, `validate-config`, `validate-anchors`,
  `validate-todo`, `fix-desc`, `fix-todo`, `squash-push`, `move-bookmark`, `delete-bookmark`,
  `agent-files`, and `agent-session`.
- `validate` runs the `[validate]` lists today, `--fast` choosing one, so the checks join it as
  flags beside that, and what bare `validate` runs is decided with **validate takes any profile**.
- `move-bookmark` and `delete-bookmark` are **bookmark takes create, delete, and move**, and
  `squash-push` and the two nouns, `agent-files` and `agent-session`, are asked about first.
- The old names go outright, as **Drop the retired flags, subcommands, and positionals** says.
- Every doc naming one changes with it, among them `agent-data/cycle-model.md` and `vc-config.md`,
  which still name `config --validate`, and the `wt5` agent took it from there (2026-11-03).

### The help lists the subcommands in alphabetical order

(wink, 2026-10-22) `vc-x1 --help` lists the subcommands in the order of the `Commands` enum in
`src/main.rs`, which is the order they were added: `agent-files`, `version`, `chid`, ... `config`
after `squash-push`, `commit` after `push`. A reader looking for one scans the whole list.

- Alphabetical order keeps the pairs together, `build` and `build-dev`, `install` and
  `install-dev`, the `validate-*` commands, and `fix-desc` beside `fix-todo`, and parts the cycle's
  `commit`, `push`, and `squash`.
- The alternative is groups by purpose, reading, checking, building, the cycle's commands, which
  clap draws only with a heading per group.
- We think clap sorts subcommands alphabetically with `next_display_order = None` on the `Cli`
  command, which leaves the enum's order alone.

### Error messages fit the terminal

(wink, 2026-10-22) An error is one line however long it is, so a terminal wraps it wherever it
runs out, mid-word and mid-path. Found by `vc-x1 build` in iiac-perf, which has no `[build]`
table, at 0.85.7-6:

```text
error: build: /home/wink/data/prgs/rust/iiac-perf/.vc-config.md: no `build.release` table. Add one, a TOML array with one command per element (see `vc-x1 config work`)
```

That line is 167 characters. It is a general problem: errors across the code are built by
`format!` as one sentence, the file, the fault, and the remedy together.

- A shape: the fault on the first line, and the file, the remedy, and the reference each on an
  indented line below it, so the first line says what went wrong and fits.
- Or the logger wraps an error at the terminal's width, indenting the continuation, which fixes
  every message at once and changes none. The two compose.
- The tests that match an error's text, `contains` on a phrase, keep passing under either, since
  neither changes the words.

### Any config string references any key

(wink + agent, 2026-10-22) The install lists take `{build.release-bin}` and
`{install.release-dir}`, or the `dev` pair, filled with the values resolved for the run, and no
other string in the config takes a placeholder. Raised at the review of the rung **feat: vc-x1
install-dev puts the dev build on the PATH**, in the cycle **feat: vc-x1 install stages, deploys,
and runs builds**: if some strings may reference keys, any string should, and any key should be
referable, so `[repos]`, `[remote]`, and `[family]` are not left out. The install lists are a
subset of the rule, so nothing written for them changes.

- One meaning everywhere: a `{key}` is the key's resolved value, a path absolute and `~` expanded,
  any other value its own text. A resolved path is a valid value for a path key, so
  `dev-dir = "{install.release-dir}"` works, and resolving it again changes nothing.
- A flag is an override of its key: `install --dir tmp/bin` sets `install.release-dir` for the run
  before references are filled, so a list and a plain value naming the key see the same value.
- Which keys are paths: a `path` kind in the schema, so the loader knows what to resolve. Each
  reader resolves its own today.
- One pass in the loader: references are filled once at load, so every command reads finished
  values. A cycle (`a = "{b}"`, `b = "{a}"`) and an unknown name are errors naming the keys, and a
  brace that is not a placeholder, `{}` or `{a b}`, stays literal, as in the install lists.
- Scope: a reference stays inside its own file, the workspace config or the user config, which
  keep separate homes for their keys.
- Every reader sees filled values at once, `repos`, `family`, and `agent-session` among them, so
  it is a change to config loading and wants its own cycle.
- With it, the schema could mark the command lists `kind = "command-list"` beside `path`, so
  `config --validate` checks a list's placeholders without running it.

### Improve init and add rename and delete subcommands

(wink + agent, 2026-10-11) A workspace's name is written into six places, and nothing renames or
removes them together: the work-repo's directory, the agent-repo's directory when it is not
nested, the two repos on the remote, each repo's `origin` URL, the `.vc-config.{toml|md}` of both
sides, and the Claude Code symlink. And `init` makes a workspace it cannot always finish, or that
Claude Code does not find.

- The order is wink's: `init` is fixed first, then `vc-x1 delete`, then `vc-x1 rename`. Each is a
  cycle of its own.
- The `init` work is more than one cycle, wink's order: the path over 200 characters, then the
  name GitHub would change, the order the two parts below are in. The first part, the agent
  template `--use-template` looked for, retired with the flag (2026-11-03).
- Delete is ahead of rename: its "find every part of a workspace from its config" is what rename
  reuses, and it gives `init` the cleanup it lacks.
- The parts are `####` headings, where [Todo format](agent-data/notes.md#todo-format) asks for
  bullets with a bold title, since a heading is an anchor a link can name.
- Delete leans on two entries far down this list: the clean verdict of [status prints a verdict
  per repo and exits with a bit per
  side](#status-prints-a-verdict-per-repo-and-exits-with-a-bit-per-side), and [status reports the
  commits no bookmark reaches](#status-reports-the-commits-no-bookmark-reaches). Whether they move
  up ahead of it, or delete's cycle builds the verdict as its first rung, is open.
- One experiment wants wink starting a fresh Claude Code session, the symlink's below.

#### symlink names a path over 200 characters as Claude Code does

(2026-10-09) Claude Code cuts a project name longer than 200 characters to 200 and appends `-` and
a base-36 hash of the path, and `encode_path` returns the uncut name, so `vc-x1 symlink` makes a
link Claude Code never reads for such a path. Found in the installed Claude Code (2.1.295) during
the cycle "fix: symlink names a project as Claude Code does", which left it out: the hash function
has to be copied exactly, and no path met so far is near 200.

Whether the hash has to match bit for bit is not known, and the opening tests it before any code is
written. On a scratch project whose full path is over 200 characters, wink starting a fresh Claude
Code session there for each pass:

- No link: the name Claude Code creates under `~/.claude/projects` is the ground truth, and a test
  vector for the hash if one is needed.
- The link as `vc-x1 symlink` makes it today, the uncut name: expected to be ignored, Claude Code
  creating its own directory beside it as it did for the `io_uring` project, whose link was wrong
  within the first 200 characters.
- A link made by hand, the first 200 characters and a made-up suffix: used or ignored, and this
  pass decides the design.
  - Used: the fix is the cut and a fixed suffix, with no hash to copy.
  - Ignored: the hash is reproduced exactly, and the first pass's name is its test.

We think the third pass may be used: Claude Code's hash is recalled, not read this session, as
differing between its native and Node installs, with a match on the first 200 characters as the
fallback.

#### init refuses a name GitHub would change

(wink + agent, 2026-10-09) GitHub allows letters, digits, `.`, `-`, and `_` in a repo name and
turns any other character into `-` without saying so. `init` asks for the repo under the name as
typed and pushes to the URL built from that name, which does not exist when GitHub changed it:

```
$ vc-x1 init test_underscore=equal+plus-dash__two-unscores
vc-x1 0.84.18
...
Step 8: Set main on the agent repo's initial commit, create GitHub repo winksaville/test_underscore=equal+plus-dash__two-unscores.agent-session --public, and push to https://github.com/winksaville/test_underscore=equal+plus-dash__two-unscores.agent-session.git
error: failed after 5 attempts: Git process failed: External git program failed:
fatal: repository 'https://github.com/winksaville/test_underscore=equal+plus-dash__two-unscores.agent-session.git/' not found
```

GitHub had created `test_underscore-equal-plus-dash__two-unscores.agent-session`, so the run left
an empty public repo there and a local workspace with no remote.

- Preflight refuses such a name when the target provisions a GitHub repo, and says what GitHub
  would make of it. `github_slug` already refuses a name ending in `.git`, and this check sits
  beside it.
- No silent rename: taking GitHub's version of the name would work, and the directory and the
  remote would then carry different names nobody chose.
- A local-only name is left alone, since the name is a fine directory.
- A push that fails with "not found" is not retried, as the five attempts here were.
- `init` has no undo, so a late failure leaves the earlier steps' work behind. The preflight check
  avoids this case and not the general one, which is its own entry if it is wanted.

#### delete removes a workspace from this machine

- It removes the directories and the symlink, and leaves the remote alone. A repo on the remote
  is deleted by hand.
- Its prompt lists what goes and the commits that are nowhere else.
- What stops it is below.

#### rename renames a workspace, its remote included

- It renames the repos on the remote too. The remote cannot be lost by it: the remote's rename
  happens or is refused, so the repo is under the old name or the new one.
- The remote goes first, so a refusal (the name taken, no permission) leaves the local side as it
  was. The order after it: both `origin` URLs, the configs, a commit and push in both repos, the
  directory, the symlink.
- The new name is checked as [init refuses a name GitHub would
  change](#init-refuses-a-name-github-would-change) checks one, so the URL written is the name
  the remote took.
- A run that stopped between the two repos is run again: it finds which repo has the new name
  and goes on from there.
- A workspace with no remote skips the remote steps. A remote that is not GitHub's is refused.
- Left out, and listed by the command: the name as the family's other repos hold it, the
  template, the messages, and the other members' configs.
- Open: whether it takes the workspace whole or one side at a time. One side alone overlaps
  [repos.agent becomes repos.agent-dir, and a command brings a config up to
  date](#reposagent-becomes-reposagent-dir-and-a-command-brings-a-config-up-to-date).
- Open: whether `.vc-config.toml` is still a form the commands handle.

#### What stops delete and rename

- An agent running in the workspace is not looked for. No lock is asked for, the session's being
  off and the one `vc-x1` was to take for a run not added ([vc-x1 takes a lock of its own for a
  run that writes](notes/todo-backlog.md#vc-x1-takes-a-lock-of-its-own-for-a-run-that-writes)).
- Uncommitted work, or unpushed work on a bookmark, refuses. This is the status verdict, called
  and not written again. The agent-repo's `@` is clean only after the session exits and a
  `squash-push`.
- Two findings warn and need a yes, `--yes` on the command line or `Y` at the prompt, an agent
  with no terminal using the flag as it does for push:
  - Commits no bookmark reaches.
  - A half-made workspace, a repo with no remote at all, as an `init` that failed before
    publishing leaves. Everything in it is unpushed, and it is the cleanup delete is wanted for.
- We think the two warnings are delete's alone: such commits move with the directory in a rename.

### Every repo's .gitignore ignores .claude, and settings stay out of agent-repos

(wink + agent, 2026-10-21) The sandbox Claude Code runs commands in blocks writes to the project's
`.claude/settings.json`, `skills`, `hooks`, and more by mounting `/dev/null` over each, and for a
name that does not exist it first creates an empty read-only file on the host to mount over, seen
from outside as files that come and go during a sandboxed command. jj inside the sandbox skips
them as devices, but a `jj` run outside meanwhile, or after a sandboxed command died, would
snapshot them as new files. vc-x1 shows them since its work repo ignores only
`/.claude/settings.local.json`, narrowed when the agent-repo moved to `.agent-session`, where
`/.claude` had been ignored whole. `.agent-session/.claude` exists too, made by a session run from
inside the agent-repo, with `.cc-writes`.

The rule, wink's calls (2026-10-21):

- Every repo has a `.gitignore` with `/.claude`, the work repo, the agent-repo, a single repo.
  Nothing under vc-x1's `.claude/` is tracked, so nothing is lost, and `!/.claude/settings.json`
  restores one shared file if ever wanted. It reverses init's narrow ignore, whose comment kept
  the rest of `.claude/` visible to `jj st`, since the placeholders make that a hazard.
- An agent-repo also ignores the four names whose meaning is stable, `/settings.json`,
  `/settings.local.json`, `/.cc-writes`, and `/.claude`, which matter when the agent-repo is itself
  named `.claude` and Claude Code's settings sit at its root, and match nothing otherwise.
  - Not the sandbox's other protected names, `agents`, `hooks`, `skills`, and the rest: they are
    not ours to list and changed on 2026-10-21, so their placeholders may still show briefly.
  - Not an allowlist of session data: Anthropic can rename a session directory at any time, and
    an allowlist would then drop the session data silently, where an ignore list fails toward a
    visible extra file.
- `.git` and `.jj` leave every template: tested 2026-10-21 with a scratch jj repo and no ignore
  file, jj skips `.git` and `.jj` at any depth and a nested jj repo, and git never tracks its own
  `.git` while a colocated `.jj/` carries its own `.gitignore`. The agent-repo's directory stays in
  the work repo's, `/.agent-session`, since git lists a nested repo and `git add -A` would add it
  as an embedded repo.
- A project `settings.json` is not kept: wink's user settings, `~/.claude/settings.json`, carry the
  common grants, and the sandbox's auto-allow and auto mode cover the rest, so a project-specific
  grant goes in that project's `settings.local.json`.
- The agent-repo's name stays free, `.claude` included: iiac-perf and zc-ring-x1 keep theirs as the
  live test of that layout, no rename.

The work:

- In vc-x1: `/.claude` in the work repo's `.gitignore` replacing `/.claude/settings.local.json`,
  the four names in `.agent-session/.gitignore`, and `.git` and `.jj` dropped from both.
- In `vc-x1 init`: the work-repo, agent-repo, and single-repo templates the same, with tests,
  one of them an agent-repo named `.claude`.
- A message to iiac-perf and zc-ring-x1 once vc-x1 lands it: add `/.claude` and the four names to
  their agent-repo's `.gitignore`, iiac-perf's being new, delete the tracked `settings.json`, and
  untrack `settings.local.json`, both pushed with the session history today. Their own agents do
  it, a repo with a live agent not being this session's to write.
- Place: a cycle of its own, and the message after it lands. It was planned as a rung of **feat:
  vc-x1 install stages, deploys, and runs builds** and left that ladder at the review of its
  `install-dev` rung, wink's call (2026-10-22).

### Shell completion installs itself and asks the binary typed

(wink + agent, 2026-10-21) Completion takes a line in the shell's startup file today,
`source <(COMPLETE=bash vc-x1)`, which the README asks for, and the script that line sources
asks one fixed binary, the one named when it was generated. So a second copy cannot answer for
itself: wink's `~/.bashrc` sourced the script for `vc-x1`, `tmp/dev/bin/vc-x1`, and
`tmp/rel/bin/vc-x1`, each binding the same function to the same name, the last winning, so
`./tmp/dev/bin/vc-x1 commit -s <tab>` was answered by the staged release, built before the
command's parser, and completed nothing. Found at the status rung of **feat: vc-x1 install
stages, deploys, and runs builds**. The relative paths also worked only in a shell started in the
workspace. Needing to edit a startup file at all is the larger fault, in wink's words a terrible
story.

- The script asks the binary typed: where it runs `"vc-x1" -- "${words[@]}"`, it runs
  `"${words[0]}"`, the command word as typed, so `./tmp/dev/bin/vc-x1` asks the dev build,
  `tmp/rel/bin/vc-x1` the staged release, and `vc-x1` the one on the PATH. Bash already finds the
  binding for any path ending in `vc-x1`, so one binding covers every copy. The workaround in use
  meanwhile is that edit made by `sed` on the sourced script, tested by a simulated completion.
- No startup-file edit: bash-completion loads a command's completion on first use from
  `~/.local/share/bash-completion/completions/<name>`, where this machine already keeps `rustup`'s
  and `copilot`'s, so vc-x1 writes its script there. zsh has its `fpath` directories and fish
  `~/.config/fish/completions/vc-x1.fish`, the same idea per shell.
  - Written by `vc-x1 install`, the deploy at Land, beside the binary it puts on the PATH, and by
    a command of its own for a hand install, a name to choose then, `vc-x1 completions` or a flag
    on `install`.
  - Since the script asks the binary typed and generates nothing itself, it never goes stale, and
    rewriting it at each install is only for a script whose form changes.
- The README's Shell completion section follows: the startup-file line becomes the fallback for a
  shell without bash-completion.
- vc-x1 prints its own script: clap_complete 4.6.9's `CompleteEnv::completer` overrides the
  binary the script calls but shell-quotes the value, so `${words[0]}` would be written literal
  and never expand, and its default is `args_os()[0]` when the script was generated, the baked-in
  binary. Cobra's scripts, `kubectl` and `gh` among them, call the command word typed, so the
  design is common and only clap's default differs. The binary still answers every completion,
  the script being only the shell's call into it, which is what makes per-copy answers right.
- No mainstream shell asks the binary typed without a per-command registration: bash, zsh, fish,
  and elvish bind completion by command name, and nushell's one global external completer could
  call the typed binary for any command speaking a protocol, at the price of a shell change.

### every repo command takes -s, required

(wink + agent, 2026-10-20) **feat: vc-x1 commit, install, and run-dev** gives the write commands
and `sync` one resolver, `-s w|a|b|p` required and `-R` optional, and the rest of the commands
still guess: the read commands default to `work`, the validate family takes a target or a path
each in its own way, and `clone` and `init` take `--por`. The sweep applies the rule to every
command that touches a repo, with the commands that touch none, `version`, `config`,
`agent-session` on a file, `validate-anchors`, `validate-todo`, and `fix-todo` on a file, exempt.

#### The read commands

- `status`, `lookup`, `show`, `list`, `desc`, `chid` lose their default scope. `status` is the
  one with a cost today: a `status` reporting one repo while the other has unpushed work is the
  miss the acquaint check exists to catch.

#### The validate family

- `validate -s w|a|b` runs each scoped side's `[validate]` table. `validate.full` and
  `validate.fast` gain `workspace-agent` in their homes, so the agent-repo's `.vc-config.md`
  carries its own table, and `validate -s b` is the whole workspace in one command.
- `validate-config -s` replaces its `TARGET`, and `b` runs the cross-side check.
- `validate-desc -s` is whose commits are checked against the other repo, `b` both directions.
- `validate-anchors`, `validate-todo`, and `fix-todo` take `-s` as the tree their files are in,
  and with no file the tree to sweep, so `validate-todo -s w TODO.md` works from anywhere.
- `validate-agent` retires: its check, `main` at `main@origin` with tracked refs, stays as the
  function push's preflight calls, and its verdict moves to `status`, which the entry [status
  prints a verdict per repo and exits with a bit per
  side](#status-prints-a-verdict-per-repo-and-exits-with-a-bit-per-side) gives an exit status.
  The agent side's table is then `full = ["vc-x1 status -s a"]`, so every per-rung validation
  catches a lost publish. The exit bit lands first, then the retirement, so nothing is without a
  check between. The README's section and the pre-0.80.0 name's rejection go with it.

#### clone and init

- `-s b` makes a dual workspace and `-s p` a plain repo, the scope naming the layout made rather
  than touched. `-s w|a` are errors, since neither makes half a workspace. `--por` and `PorFlag`
  retire.

#### What the sweep touches

- The `tests/cli_*.rs` subprocess tests, the fixture's recorded command lines, every `vc-x1`
  invocation in the README and the agent-files, and the lookup's recognition of push lines in
  transcripts, which must keep reading the old form.

##### Why: one rule

A command never guesses which repos it acts on, `-s` says which and `-R` says where, and one rule
with a short exemption list is easier to hold than a list of defaults. The ceremony of `-s w` on
`validate` is accepted for the uniformity, and the error on `validate -s a` without a table teaches
the model.

### A config file says what and its constraints, and the why is in the cycle record

(wink + agent, 2026-10-21) `.vc-config.md`'s prose around each table carries history: who decided
and when, measurements, and earlier trials, as `[validate] fast`'s "wink's call 2026-10-20" with
its 8 s timings and the 2026-09-21 trial, `[validate] full`'s "wink's call 2026-10-21", and
`[build] dev`'s timings and the case for a debug build. Each cycle adds its own, so the file grows
without end and the facts a reader comes for get harder to find. Found at the install rung of
**feat: vc-x1 install stages, deploys, and runs builds**.

The rule, wink's call (2026-10-21):

- A config file states what each key does and its constraints. A constraint stated with its
  reason stays, such as "owned by the work side because it builds the artifact", since the reason
  is what tells a reader the limit is real.
- The constraints that make things fail when broken matter most, and belong beside the key:
  `[build] release-bin` and `dev-bin` must name where the build's commands leave the binary,
  since vc-x1 never works it out. A wrong `release-bin` fails `build` when nothing is there, and
  when a stale binary is there `install` copies it. A wrong `dev-bin` has
  `run-dev` run a stale binary or none.
- The why, who decided, when, the measurements and the trials, is in the cycle record,
  `TODO.md > ## In Progress` while the cycle runs and its `## Closed` in the landmark's tree after.

The work:

- `.vc-config.md`: drop the history from the three places above, keeping what and the
  constraints, and add the `release-bin` and `dev-bin` constraint. Each dropped fact is first
  checked to be in its cycle's record, cited there by the landmark, and one that is not is
  written into a `notes/` file rather than lost.
- `vc-config.md`, the key reference, and the `.vc-config.md` that `vc-x1 init` writes, read the
  same way.
- A sentence in the agent-files saying a config file carries what and constraints and the why
  lives in the cycle record, tried in the local copy and proposed. It may fold into
  **agent-files(proposal): v0.3.1**.

### dr-1 is built in Rust

(wink + agent, 2026-10-20) A fresh clone's full test run fails all fourteen `dr1_*` tests, each
with the same panic, because the `dr-1` fixture ([fixture-dr-1.md](notes/fixture-dr-1.md)) is
built only by `support/fixtures/build-dr-1.py`. That needs `python3`, a dependency the project does
not otherwise have, and drives the installed `vc-x1` rather than the code under test. The fix:
the tests build the fixture in Rust, and the Python builder goes.

- Useful history: **test: the tests build the dr-1 fixture** tried this as one cycle and closed as
  a dead end. Its dead-end closing commit, the landmark on `main`, holds the record in
  [`zCLOSED.md`](notes/README.md#zclosedmd): the problem, the plan as filed, and the deliberation behind it, among them the jj
  CLI kept for the record, a test-only jj settings override per thread, an unseeded build, and a
  failed build stopping the run with one banner. The agent-repo's transcript of that cycle holds
  its unpushed plan rung, the port mapped against the Python builder.
- Suggested plan, bottom-up, each layer tested before the next starts, as rungs of one cycle or as
  cycles of their own, decided when this opens:
  - The transcript writer: the fixture's clock and session-file writer, the user, assistant,
    tool-call, and tool-result lines in the transcripts' shape, as test-only code under
    `#[cfg(test)]`. Checked by a test that writes one line of each kind, and by a throwaway
    comparison with a session file from the Python builder.
  - jj settings pinned per thread in tests: a test-only override of one thread's jj settings, and
    a `jj_ok` that sets a spawned jj's environment, so a commit made in-process or by the CLI
    takes a chosen time and committer and is not signed. A test may not write the process's
    environment, and in-process jj-lib otherwise reads the developer's jj config, signing key
    included. Checked by a test of each that reads the commit back.
  - The agent runs its first cycle: the scripted agent, its writes, edits, Bash lines, and pushes
    with their windows, and the fast-forward land, run through the fixture's first cycle alone,
    calling the crate's own init and push in-process. Checked against the same cycle from the
    Python builder.
  - The whole fixture: the other cycles and the record. A test checks it is whole: both repos
    load, the nine pushes are in `relationships.json`, and each push's two change ids resolve.
  - The tests build their fixture: the fixture helper builds it once per test process and stops
    the run with one banner when the build fails, the fourteen `dr1_*` tests lose their
    `#[ignore]`, and the Python builder goes, with the `[test] fixtures` key, `.fixtures/`, the
    README's stale fixture section, and `tests/cli_fixture_builder.rs`, which guards the Python
    builder only until then (**fix: the fixture builder drives commit**).
- Acceptance: in a fresh `vc-x1 clone` with `python3` off the PATH, `cargo test` runs the
  fourteen `dr1_*` tests, none ignored, and they pass, and a forced build failure prints the
  banner once and stops.

### lookup is tested on a reordered ladder

(wink + agent, 2026-10-18) The cycle **fix: a missing fixture fails instead of passing** inserted
a rung between its plan rung and its pushed fix and tests rungs by moving the bookmark back and
rebasing the two onto the new rung, then amended their `Cargo.toml` versions. The agent-repo's
history stays linear in time while the work-repo's ladder is out of time order, so three
hypotheses to test soon, on this ladder, before it lands and the specimen is gone:

- The `ochid:` pairing holds, since the trailers are change ids and a rebase keeps them, so
  `lookup` pairs each rung with its agent-repo commit as it did before. A rebased rung is a
  rewritten partner with a predecessor, the shape dr-1's amend case covers, and the two amended
  rungs have two predecessors each, which it does not.
- Timestamps split: jj keeps the author time across a rebase and resets the committer time, so
  the new rung, a parent, is authored after its children. Author time still matches the agent-repo
  and the transcript writes. Whichever of author time, committer time, and topological order
  `lookup` leans on when it builds windows and candidates is what the dr-1 window tests, run
  against this workspace, will show. We think author time holds and a parent-before-child
  assumption breaks.
- A line changed by the version amendments has no transcript write, since the rewrite happened
  outside a push, so `lookup` should report it as such and not attribute it to a neighbor.

Before the reorder, in UTC, each work-repo rung then its agent-repo pair, as
`change commit author/committer`:

- opening: `lzvovkulykur ebefe3b22dd4 04:29:53/04:31:17`, pair `nlkoltkttsqo 1fbb8b1e2d1d 04:28:39`
- plan: `stqtnqmlpnws 422abade5eed 04:33:27/04:33:28`, pair `qltllyttorvo d89842b60354 04:33:28`
- fix: `oskrwppmyouv 46e59c49aebb 04:35:44/04:35:44`, pair `nwyysysuknzr 7bf5994495a0 04:35:44`
- tests: `tqspunzumpru 58f3bbe9ebd4 04:38:16/04:38:16`, pair `qqoupsuztovt 88b87b336121 04:38:16`
- all on 2026-10-19 UTC, the evening of 2026-10-18 local.

After the reorder, the same form, `author/committer` both 2026-10-19 UTC:

- opening: `lzvovkulykur ebefe3b22dd4 04:29:53/04:31:17`, unchanged
- plan: `stqtnqmlpnws 422abade5eed 04:33:27/04:33:28`, unchanged
- fixture workspace, the inserted `-2`: `sznlvtzvxsvt 86a51e4eeb4e 05:49:42/05:57:28`, pair
  `umkkskkrtstp 056959ed929a 05:44:33`, the pair authored before the work commit, since the
  session's writes began before the rung's push
- fix, now `-3`: `oskrwppmyouv d9e0159de298 04:35:44/05:58:33`, same change id, the pair unchanged
- tests, now `-4`: `tqspunzumpru e80abb4352c0 04:38:16/05:58:53`, same change id, the pair unchanged
- So the inserted parent is authored after both children, the rebased two keep their author times
  and carry committer times from the rebase, and each was rewritten twice, the rebase and the
  version amendment, so each has two predecessors.
- Acceptance: the `dr1_*` tests, pointed at this workspace and its `.claude` as the fixture, with
  a relationships file written from the pairs above, pass or fail per hypothesis, and each result
  is recorded.

### The family leaves the committed config

(wink + agent, 2026-10-17) `.vc-config.md`'s `[family]` table names this repo's member name and two
sibling paths, `template` and `messages`, which hold only where the repos sit on this machine, and
`agent-files diff` and `copy` fall back to `family.template`. With the family local, its shared
`custom.md` layer tells the agent where the template and the messages repo are, and the tool needs
no family at all.

- `[family]` leaves `.vc-config.md`, `vc-config.md`, and the schema, and the code reading it goes,
  `agent-files diff` and `copy` taking their directory from the argument or `[agent-files.*] dir`.
- `vc-x1 agent-files` stops treating `custom.md` as part of the set, as v0.3.0's text does: `size`
  counts it when present, a local file in the set's total, and `diff` and `copy` compare and copy
  it under `-c` and `agent-files.diff.custom`.
- The entries that find the messages repo through `[family] messages`, [sync fetches the messages
  repo too](#sync-fetches-the-messages-repo-too) and [feat: vc-x1 msg, the messages protocol as
  code](#feat-vc-x1-msg-the-messages-protocol-as-code), are revisited, since that key goes.
- The family itself moves to `../vc-x1-family`, its own repo beside the members, outside this one.
- The cycle **feat: the family directory** (2026-10-16) moved the family into vc-x1 as `family/`,
  the opposite direction, and its bookmark was abandoned unlanded (2026-10-17). Its two unrelated
  changes, the touched files' prose losing its semicolons and `[validate] fast` becoming
  `cargo test --bins`, are redone when wanted.

### init and clone take --custom

(wink + agent, 2026-10-17) With `custom.md` uncommitted, a fresh `init` or `clone` has none, and its
first session starts without the family's layer.

- `--custom <path>` copies the file at `<path>` into the new workspace as `custom.md`.
- We think a member's `custom.md` is mostly a pointer into the family's shared layer, so a copy
  rarely drifts, and an `update` subcommand that copies it again waits for a case that needs it.
- Follows the cycle **agent-files(proposal): v0.3.0**, which made `custom.md` optional.

### The CLI smoke tests need no jj repo

(wink + agent, 2026-10-18) Three tests in `tests/cli_smoke.rs`,
`cli_version_gate_stops_every_subcommand`, `cli_vv_reports_then_runs_the_subcommand`, and
`cli_ambient_banner_is_stderr_only`, run `vc-x1` in the source tree, so they need it to be a jj
repo. A plain `git clone` and `cargo test` fails them with `There is no Jujutsu repo in .`, found
by the cycle **chore: custom.md is untracked and gitignored** in a copy of the tree.

- The tests run `vc-x1` in a jj repo of their own, a temporary one, never the source tree.
- Beside the cycle **fix: a missing fixture fails instead of passing**, both making a stranger's
  `cargo test` true.

### vc-x1 clone of a local jj workspace tracks no main

(wink + agent, 2026-10-18) `vc-x1 clone /path/to/vc-x1 <name>` fails with `bookmark 'main' has
non-tracking remote 'main@origin'`, found by the cycle **chore: custom.md is untracked and
gitignored**. The source is a colocated jj workspace, whose git `HEAD` is detached, so the clone
has no default branch and jj tracks none of the remote bookmarks. `clone_one` in `src/clone.rs`
then fails `verify_tracking(target_dir, "main")`. The help lists local paths as sources.

- A clone tracks `main@origin` when the source has a `main`, whatever its `HEAD`.
- The half-made clone is left behind, `.git` and `.jj` alone, and a rerun refuses since the
  directory exists. The fix decides whether a failed clone removes it.

### init, clone, and the others take --bookmark

(wink + agent, 2026-10-18) `vc-x1 clone` always lands on `main`, so checking a cycle's bookmark
from a fresh clone takes a `jj new <bookmark>@origin` after it, and `init` has the same `main`
built in.

- `clone --bookmark <name>` tracks `<name>@origin` and puts the working copy on it.
- `init --bookmark <name>` names the bookmark the new workspace starts on.
- The other subcommands that assume `main` are surveyed, and those that would take a bookmark
  name it the same way, `--bookmark`, as `sync` already does.
- We think the agent-repo stays on its `main`, having no cycle bookmark.

### Fixtures are named by their source

(wink + agent, 2026-10-17) `[test] fixtures` names a directory on this machine,
`../vc-x1-fixtures`, in a committed file. The `dr-1` fixture is built reproducibly by
`support/fixtures/build-dr-1.py` and published as `vc-x1-fixtures-dr-1-work` and `-agent` on
GitHub, so where it comes from is the portable fact, and where it sits is not.

- The committed config names each fixture by its source, a name and a clone URL, pinned to a
  commit.
- A fixture is fetched into a fixed, gitignored directory in the workspace, `.fixtures/<name>` say,
  set by convention, never by config. The cycle **fix: a missing fixture fails instead of passing**
  made `.fixtures/` that directory, still named by `[test] fixtures` until the source-named config
  replaces the key.
- A `vc-x1 fixtures` subcommand clones the listed fixtures there, as `vc-x1 clone` lays a dual
  workspace out, so any repo can declare fixtures and vc-x1 fetches them.
- Built for vc-x1's need first, the config's shape general enough for another repo to use
  unchanged.

### Tests leave a per-repo config entry under the user's config dir

(wink + agent, 2026-10-15) jj keeps a repo's config under the user's config dir,
`~/.config/jj/repos/<config-id>/`, and creates the entry the first time a command touches the
repo, so every test fixture repo a `jj` child or `vc-x1 init` touches leaves one behind, never
removed. A full `vc-x1 validate` adds 19, and 16,965 had built up by 2026-10-15, removed then at
wink's go. Found in the cycle `fix: a snapshot failing is reported to the user and in Result`.

- The fix: tests give jj a config dir under the test's temporary root, on the spawned child's
  environment and in the in-process sessions alike, never the test process's own environment,
  per `fix: no test sets HOME`.
- The measure: the entry count under `~/.config/jj/repos` is unchanged by a full validation.
- Until it lands, `jj config gc` removes the entries for repos that are gone, asking first
  ([Garbage collection](https://docs.jj-vcs.dev/latest/design/secure-config/#garbage-collection)).

### The set's session-lock file says a killed holder leaves files behind

(wink + agent, 2026-10-14) [agent-data/session-lock.md](agent-data/session-lock.md#what-is-known)
says the session's background commands end when the session does, and so the lock is never
stale. What it does not say: a holder ended that way skips the sandbox's cleanup, and leaves an
empty read-only file on disk for every path the sandbox protects, `.git/config.lock` among them
([session-lock.md](notes/session-lock.md#the-sandboxs-placeholder-files)). An adopter turning the
lock on should know. An `agent-files` proposal cycle, being a change to the set.

- The same cycle fixes [The instruction](agent-data/session-lock.md#the-instruction)'s on and off
  steps (wink + agent, 2026-10-15). Turning on replaces "the entry that says the lock is off",
  and turning off puts that entry back, but an adopter need not carry the entry, and vc-x1 has not
  since the cycle `docs: the session lock moves to the README`. Turning on is adding the entry,
  and off is taking it out.

### push --amend folds a stage into the pushed work commit

(wink + agent, 2026-10-11) A later stage of a cycle, its closing most often, adds to the work
commit already pushed, and `vc-x1 push` can only make a new commit on each side. So the one
work-repo commit needs to reference several agent-repo commits. Today that is done by hand:
a `jj squash`, a `jj describe` with a trailer written by hand, and then the push.

- The work-repo side: `push --amend` squashes the working copy into the target commit, the top
  commit of the bookmark, where a plain push would make a new commit of it.
  - The target keeps its change id and gets a new commit id, so the bookmark is rewritten and
    pushed.
  - The title and body are optional. Given, they replace the target's, if left out the target
    keeps its own.
  - The target's current `ochid:` trailers are kept as is and the new one is added at the bottom
    of the list, so the list is oldest first.
- The agent-repo side: a new commit with the same title and body as the work-repo commit. Its
  `ochid:` trailer names the work-repo commit, and its own change id is the last `ochid:` on
  that commit.
- Approval: it rewrites a published commit, so it takes the approval any push takes and is
  refused on a bookmark that has landed.

### validate-desc and fix-desc read every ochid trailer

(wink + agent, 2026-10-11) A work commit that names two agent-repo commits carries two `ochid:`
trailers, and the two checkers read the last one alone, read from the code on that date.

- `vc-x1 validate-desc` checks the last trailer, so a first trailer that resolves to nothing
  passes.
- `vc-x1 fix-desc` reads the last trailer, and when it repairs with a resolved id it rewrites
  every `ochid:` line to that one id, which would make two trailers one.
- The `list` and `show` columns print the last trailer alone.
- Reading all of them already: the partner search of `lookup`, and the guard `squash-push` has
  against dropping a trailer.
- Each trailer is checked and repaired as its own, and a commit is reported once with every
  finding.
- It follows the entry above, as a cycle of its own.
- It overlaps [vc-x1 push: record uncovered work commits (N:1
  work<->agent)](#vc-x1-push-record-uncovered-work-commits-n1-work-agent).

### validate-anchors has four known issues

(wink + agent, 2026-10-13) `vc-x1 validate-anchors` over the whole work-repo exits 1 on three
problems, two of them false, and it skips every cross-file link, so it is not in
`.vc-config.md`'s `[validate]` table and a rung's broken link fails nothing. The issues are one
entry, each a bullet, and we think each a rung of one cycle that ends with the command in the
table.

- The whole-repo run at 0.84.26, 47 files, 260 links checked, 275 cross-file skipped, 3 failed:
  - `notes/design-cli/por-dual-parity-audit.md:489`, a duplicate `#summary` that no link names,
    the third issue below
  - `notes/design-cli/por-dual-parity-audit.md:1194`, `[1]` defined but never cited, the second
    issue below
  - `vc-config-model.md:121`, `[17]` defined but never cited, the second issue below.
- **validate-anchors keeps a heading's code-span text when it slugs.** (wink + agent, 2026-10-11)
  A heading holding a code span is slugged as though the span's text were absent, so a correct
  link to it is reported as broken and the "did you mean" line offers the slug that breaks it. It
  is bugs 15 and 19 in [bugs.md](notes/bugs.md).
  - The case: ``### Support POR workspaces in `push` `` is `#support-por-workspaces-in-push` on
    GitHub, and the command computes `#support-por-workspaces-in`.
    - It was live in this file until 2026-10-13, when the heading lost its backticks, wink's
      edit, and the link was changed to match. That stepped around the fault and fixed nothing.
    - A scratch file with that heading and its link still fails at 0.84.26, as does ``## A in
      `mid` word`` with `#a-in-mid-word`.
  - The fix is bug 19's: strip the backticks and keep the span's text.
  - Bugs 15 and 19 are one fault found twice, and one retires into the other.
  - `vc-x1 validate-anchors TODO.md` runs to zero today because no link here names a heading
    holding a code span, eleven headings having one.
- **validate-anchors finds a citation that follows a wrapped code span.** (wink + agent,
  2026-10-13) A code span that wraps onto a second line hides a `[[N]]` citation after it on that
  line, so the definition is reported as defined but never cited.
  - The live cases are the two in the run above: `vc-config-model.md:97` cites `[[17]]` after
    the tail of a span opened on line 96, and `por-dual-parity-audit.md:9` cites `[[1]]` the
    same way.
  - A scratch file fails at 0.84.26 with the span wrapped and passes with it on one line.
  - The cause, we think, not read from the code: backticks are paired one line at a time, so the
    wrapped span's closing backtick reads as an opening one and the rest of the line is taken
    for code.
  - `vc-config-model.md` is generated, so until the fix its wrap is the generator's to avoid.
- **validate-anchors warns on a duplicate anchor only when a link names it.** (wink + agent,
  2026-10-11) Two headings that slug alike are reported as a duplicate anchor whether or not
  anything links to either, and a duplicate is a fault only for a link, which reaches the first
  heading alone.
  - A duplicated anchor is reported when a link in the checked files names it, and the report
    says which heading the link reaches.
  - An unlinked duplicate is counted in the summary and not reported as a problem.
  - The live case is the `#summary` in the run above. An earlier one, four duplicates in
    `zDOING.md` with two cycle blocks in one file, went when that layout was left for a file per
    block on 2026-10-11.
  - Weighed and left: making a block's item headings bold labels, which have no anchor, and
    which an editor's outline then no longer shows.
- **validate-anchors resolves a cross-file link.** (iiac-perf, 2026-09-02, widened wink + agent,
  2026-09-22) A cross-file markdown link is never checked. The command recognizes a
  `](other.md#slug)` target and a `[N]: path#slug` definition, counts them, and skips them, so a
  link to a file that is not there, or to a heading that file does not have, passes.
  - `validate-config` resolves one target, a `vc-config.md#<anchor>` fragment, against the
    schema, and nothing else does more.
  - The concrete case: zc-ring-x1's `.vc-config.md` links `vc-config.md` and
    `vc-config-test.md`, neither in that repo, and `validate-config` (0.82.0) on a copy reports
    six problems with neither missing file among them.
  - A fresh adopter sees the gap as the report's tally: uis-x1, 86 links checked and 132
    cross-file skipped, so most of what its records link is unverified.
  - The fix resolves the target instead of skipping it. The file half is a path relative to the
    file holding the link, and the fragment half is a heading in that file, slugged by the rule
    the same-file check already applies, so it needs no new machinery, only the other file's
    headings.
    - File: fail a target whose file does not exist, with the path as the linking file resolved
      it.
    - Fragment: fail a target whose file exists and has no heading slugging to the fragment,
      naming the nearest heading as the same-file check does.
    - Root-absolute paths, the `/notes/<file>.md` form most of this repo's `[N]:` definitions
      still carry, resolve against the workspace root, the directory holding `.vc-config.md`,
      which is how GitHub reads them. A target the check could not place is still counted as
      skipped, and the summary says so.
    - A target outside the workspace, `../vc-x1-messages/...` from a sibling, resolves when the
      file is there and counts as skipped when it is not, since a sibling's presence is the
      machine's, not the record's.
    - Only `.md` targets are resolved. A URL, an `.rs` path, or a `.txt` stays skipped.
    - `validate-config` keeps its schema lookup for `vc-config.md#<anchor>`, since the schema is
      the authority on those anchors and the file may not be in the adopter's tree.
    - The report's cross-file tally becomes two numbers, resolved and skipped, so a pass says
      what it covered.
  - It unblocks **Reference defs: go file-relative, with anchors** in the backlog rather than
    waiting on it: the sweep to file-relative paths becomes a change the check verifies, and the
    backlog entry loses its "after validate-anchors grows the cross-file check" sequencing.
  - Reported by iiac-perf's message **2026-09-02T17:26:18.543Z Cross-file links go unchecked** in
    `../vc-x1-messages`, which asks for a reply naming it and linking where this landed, so the
    reply goes out once this entry's cycle is pushed.

### A single-step cycle is two stages and one commit

(wink + agent, 2026-10-11) Tried by the cycle **docs: update single-step cycle add zCLOSED.md**,
whose record has the reasons, and wink liked how it read. A single-step cycle is one commit that
records its own acceptance check as passed before the commit exists, and an opening deletes the
last finished record, so none is readable while a cycle runs. An `agent-files` proposal makes the
trial the rule or drops it.

- Two stages, each validated, reviewed, and pushed: the first carries the opening's duties and
  the work, under the dev name at the `-1` suffix. The closing runs the acceptance check against
  the pushed work, sets the bare version, and moves the record.
- One work commit: the closing is folded into the pushed work commit, which keeps its change id
  and gains a second `ochid:` trailer, and the agent-repo carries a commit for each stage, both
  naming the one work commit. Done by hand in the trial, and [push --amend folds a stage into the
  pushed work commit](#push---amend-folds-a-stage-into-the-pushed-work-commit) is the tool for it.
- One way of running: a single-step cycle and a multi-step one then open and close alike, and
  [Cycle shape](AGENTS.md#cycle-shape) loses its rule about the first push and the way out it
  gives a single-step cycle that grows. Whether a multi-step closing folds too is open.
- Land restores the plain name in both, as a multi-step cycle's does today.
- The one commit lands as it is, with no merge commit, and `main` is pushed once.
- The bookmark's create and publish needs no approval. The pushes and the Land keep theirs.
- The ladder is one unlinked rung with the bare title, as today, and no title carries
  " closing". What the closing taught goes under a `#### Closing` heading in the block.

### TODO.md is renamed zTODO.md

(wink + agent, 2026-10-09) `zTOC.md` is named so it sorts last, and `TODO.md` beside it
is not. Under `ls` in `en_US.UTF-8` the two are entries apart,
and in byte order, as git, jj, and GitHub sort, they are at opposite ends of the root's files.

- With `TODO.md` renamed `zTODO.md` the pair is adjacent and last in both orders. The pair
  `_DOING.md` and `_TODO.md` is adjacent in byte order only, since that locale ignores the
  underscore.
- Every adopter carries a `TODO.md`, so the rename is an `agent-files` proposal the family takes at
  a re-sync, and the template payload's skeleton is renamed at convergence.
- About 120 mentions outside `notes/chores/` name the file in this repo, and `TODO_FILE` in
  `src/todo_helpers.rs` is the default the `validate-todo` and `fix-todo` commands read.
  - A fallback to `TODO.md` when only that exists keeps an adopter on the new binary working
    before its re-sync.
- The price is the name itself: `TODO.md` is recognised on sight and `zTODO.md` is not.

### sync fetches the messages repo too

(wink + agent, 2026-10-15) `custom.md`'s messaging entry has a session fetch `../vc-x1-messages`
by hand, at an acquaint and at every cycle start, because `vc-x1 sync` knows only the workspace's
two repos. Those are exactly the moments [Sync](AGENTS.md#sync) already runs, and the workspace
config names the repo, `[family] messages`, so `sync` could fetch it beside the two.

- `sync` fetches the `[family] messages` repo when the config names one, and reports it like the
  others. `--dry-run` fetches too, since a fetch moves no local bookmark, so the acquaint look
  stays read-only.
  - The messages repo is a jj repo colocated with git, so the fetch is
    `jj git fetch --ignore-working-copy`, the form its README's Fetch action names, since a bare
    jj command snapshots the working copy, and that is a write.
- `custom.md`'s Fetch sub-bullet shrinks to reading what is pending, the fetch being `sync`'s, a
  change to the project layer in the same cycle.
- The README's guards stay the session's: `sync` only fetches, and a fetch takes no ownership.

### sync --undo takes back the last sync

(wink + agent, 2026-10-11) A sync that was forced with `--force` and is regretted, or that stopped
part-way on a conflict, has no easy way back. `vc-x1 sync --undo` would put a repo back the way
the last sync found it: its bookmarks, `@`, its files, and what jj believes the remote holds.

Deferred from the cycle **feat: sync is safe by default**, where it was a rung and was dropped
before any code was written. That cycle made the routine sync safe by itself, so undo serves the
forced sync alone, and its design turned out to hold more than the cycle had room for.

The complications, the plain version:

- jj's own undo is not enough. `jj op restore` puts jj's records back and leaves git's own note
  of where the remote is at the new value. jj then believes it has already seen the remote's
  change and never follows it again, so the repo says "up to date" when it is not.
  - A real undo puts git's note back as well, so the next sync sees the remote's move and does
    its work again.
  - This part was written and tested once, as `restore_as_found` with its tests, and the code is
    gone: it was a draft under `tmp/`, which is not committed, and was deleted 2026-10-11. It is
    rewritten from this description when the entry is picked up.
- Undo has to know where the last sync began. The plan is for sync to label each jj operation it
  makes, and for undo to read jj's operation log back to the latest labelled one.
  - Nothing is saved in a file of our own. A saved snapshot that went stale is what had the old
    `revert` command removed at 0.78.3.
  - Every step of a sync then has to run through one place that adds the label. Today moving
    `@` at the end of a sync does not.
- Work done after the sync must not be thrown away. If anything but plain saves of the working
  copy came after it, a commit, a bookmark move, another sync, an earlier undo, then going back
  would discard that work. Undo refuses and lists what is in the way.
- The conversation is the hard case. In the agent-repo a running session writes its transcript
  all the time, and putting files back to an earlier state would wipe the lines written since.
  - So undo carries later writes forward: it works out what changed since the sync and puts
    that on top of the restored state.
  - A file the sync never touched is then not rewritten at all, which is what keeps a live
    transcript safe.
  - A file that both the sync and later work changed has no clean answer, and undo refuses
    before it changes anything.
- A second undo in a row refuses, the first being work done after the sync.

Not yet decided, wink's to say:

- Whether "only saves of the working copy may follow" is the right rule for refusing, or an
  `--undo --force` should exist. Past that point `jj op restore` may be the honest tool.
- Whether later writes are carried forward as above, or undo refuses whenever any file changed
  since. Refusing is simpler and makes undo nearly unusable in the agent-repo during a session.
- Whether each repo is undone by itself from its own log, or both repos of one sync run go back
  together or not at all.

Smaller parts of the design:

- `--undo --dry-run` says what an undo would do, or why it would refuse, and changes nothing.
- A sync that changed something ends by naming `vc-x1 sync --undo`, and so does the report of one
  that stopped part-way, in place of the bare `jj op restore` it prints today.
- The scope is `-R` and `--scope`, as a sync takes them.
- What jj-lib offers was checked at 0.45.1: attributes on an operation, a marker on a snapshot
  operation, and a three-way merge of trees.

### A commit body is an intro and a pointer to the record

(wink + agent, 2026-10-11) A commit body grows long, a continuing problem in wink's words, and
the cause is that [Commit-body form](agent-data/prose.md#commit-body-form) asks it for the
problem and solution breakdown the cycle-record already holds: the specimen body of a work rung
and the specimen of that rung's subsection are nearly the same text. The body for the cycle
**docs: update single-step cycle add zCLOSED.md** was drafted at 34 lines and pushed at six.

- The body is two plain paragraphs, wink's sharpening (2026-10-20): the first the problem, the
  second the solution, opening "Solution:", each a sentence or two, and no `*` or `-` bullets.
  The solution paragraph ends with the pointer. First used by the rung **feat: vc-x1 squash
  folds work into the pair** of **feat: vc-x1 commit, install, and run-dev**, ahead of the rule.
- The `*` and `-` breakdown stays where it is already written, the record, and leaves the body.
- The pointer is one fixed sentence ending the solution paragraph, so it is greppable: "See
  details in TODO.md.", "See details in zCLOSED.md." while the record was on trial there, wink's
  wording (2026-10-21), replacing the longer "Details in the cycle's record, zCLOSED.md at its
  closing commit, under this title." `## Closed` is replaced at every closing and at `HEAD`
  usually describes another cycle, so the sentence means the file at the cycle's closing commit, which `git log --grep` on the title finds. The rung's
  subsection is headed by the commit's exact title.
- In use ahead of the rule from the build rung of **feat: vc-x1 install stages, deploys, and runs
  builds** on, a bend of Commit-body form recorded in that cycle's deliberation.
- The rung subsection becomes the full account: the facets and their solutions in `*` and `-`
  form, which [The In Progress block](agent-data/notes.md#the-in-progress-block) says is never a
  restatement of the body, now that the body carries none. **A commit body carries a legend for
  its markers** then applies to the record, not the log, and may retire.
- A bookend's body is this form already.
- The cost: the log alone no longer explains a commit, and the record has to hold everything the
  body did.
- In **agent-files(proposal): v0.3.1**: the rule in `prose.md`, the specimens in
  `commit-model.md`, and the rung subsection in `notes.md` change together, and bodies already
  published keep their form. It was to be part of `agent-files(proposal): v0.2.8` (wink,
  2026-10-11), which landed as the sync rules alone (2026-10-13).

### A commit body carries a legend for its markers

(wink, 2026-10-09) A commit body's `*` is a problem the commit addresses and each `-` under it a
solution, by [Commit-body form](agent-data/prose.md#commit-body-form). The rule is in the
agent-files, which a reader of `git log` or of GitHub never sees, so to them the body is a nested
list and the pairing goes unread. Noticed reading the body of "feat: sync --dry-run looks and
changes nothing".

- One line, after the intro paragraph and before the first `*`, a blank line either side:
  ``Legend: `*` is a problem this commit addresses, `-` is a solution to it.``
- A bookend body has no list and carries no legend.
- An `agent-files` proposal: the rule in `prose.md` and the specimen in `commit-model.md` change
  together, and bodies already published keep the form they shipped with.
- Overtaken if [A commit body is an intro and a pointer to the
  record](#a-commit-body-is-an-intro-and-a-pointer-to-the-record) is adopted: a body with no
  markers needs no legend.

### The docs and the code say .claude where they mean the agent-repo directory

(wink, 2026-10-09) The agent-repo's directory is whatever `repos.agent` names, `.agent-session` by
default, and `.claude` is still written as if it were the name: 57 mentions in `README.md` and
about 170 lines in `src/`, counted and not yet read one by one. Surfaced at the review of the cycle
"fix: symlink names a project as Claude Code does", which fixed only the README's `symlink`
paragraph. A multi-step cycle, one rung per group:

- Prose: the README and `vc-config.md` text. `~/.claude/projects` and Claude Code's own `.claude`
  are correct as written and stay.
- User-visible strings: help text and log lines, push's `.claude (...)` heading and status's
  `.claude` label among them, print the configured directory rather than a literal.
- Fallbacks: `symlink` and `validate-bot` fall back to `.claude` when no config resolves, and
  whether that becomes `.agent-session` or an error is a decision.
- The ochid label: goes to **Work commits link by URL**.
- Tests and fixtures: `.claude` as an arbitrary directory name is harmless and mostly stays.
- Overlap: [The code says agent where it still says
  bot](#the-code-says-agent-where-it-still-says-bot) and [repos.agent becomes repos.agent-dir, and
  a command brings a config up to
  date](#reposagent-becomes-reposagent-dir-and-a-command-brings-a-config-up-to-date) touch the same
  lines, so the three are ordered or folded into one program before any opens.

### Every command finds the repos from the configs, and none assumes nesting

(wink + agent, 2026-10-09) The two roots are meant to be found from a `.vc-config.*` and the paths
in it, the agent-repo sitting anywhere. The lookup already does that: the walk up takes the nearest
config's `repos.work` as the work root, `repos.agent` may be relative or absolute, and a directory
is the agent side when the config resolves to it. Read from the code, and never run against a
workspace whose agent-repo is outside the work-repo. What still assumes nesting or a name:

- `init` cannot create the layout: `--agent-dir` takes one name inside the project, so a non-nested
  workspace is made by moving the agent-repo and editing both configs by hand.
- `symlink` and `validate-bot` fall back to a literal `.claude` when no config resolves, and
  `validate-anchors` skips a fixed list of directories that holds `.claude` and not what
  `repos.agent` names. These overlap [The docs and the code say .claude where they mean the
  agent-repo
  directory](#the-docs-and-the-code-say-claude-where-they-mean-the-agent-repo-directory).
- No test runs the commands against a non-nested workspace, so the first rung is a fixture with
  the agent-repo beside the work-repo and each command run from both roots, which turns "read from
  the code" into a result and finds what this list missed.
- `clone` and the generated `.gitignore`, which names the agent directory inside the work-repo,
  are not yet read for the assumption.

### One spelling for a workspace's two shapes

(wink + agent, 2026-10-09) The tree names a workspace's shape five ways: dual-repo, dual workspace,
POR, single-repo workspace, and `is_work_only()` in the code, which [One spelling per
term](agent-data/prose.md#one-spelling-per-term) forbids. "Dual-repo workspace" becomes the one
spelling for the two-repo shape, defined in `README.md`, and the opening picks its single-repo
counterpart and decides whether the code's names follow the prose.

### vc-config.md states that a settable key's leaf is its flag's long name

(wink + agent, 2026-10-09) A settable key's leaf is its flag's long name, in nine pairs
(`--account`, `--debug`, `--repo`, `--result-lines`, `--col-width`, `--custom` twice, `--yes`, and
the `[DIR]` operand), with no exception among the keys that have a flag. The rule is written
nowhere, which is how `remote.agent-name` came to be proposed and corrected. One line in the
"Shape:" prose of `vc-config.md` states it.

### Scope is a flag on every command, and push resolves its own bookmarks

(wink, 2026-09-17, reworked 2026-10-10) `squash-push -R .agent-session` bakes in a path the config
already knows, and the path differs by project, `.agent-session` here and `.claude` historically,
so the invocation goes stale when it moves. `vc-x1 squash-push -s agent` cannot go stale and reads
as what it means. Beyond `squash-push`, the commands disagree on how a scope is given and on what
none means, and one rule is wanted for all of them.

- Scope is always `-s` / `--scope`, never a positional, on every command that takes one (wink,
  2026-10-10, reversing this entry's first decision, which made it a positional as `status` has).
  - A positional cannot be had everywhere. On a command whose positional is a revision, `chid`,
    `desc`, `list`, `show`, a leading `work`, `agent`, or `both` would have to be a reserved
    word, and each is a plausible bookmark name, so `vc-x1 list work` would have two readings.
  - A flag has one reading on every command, and `-s both` is three characters more than `both`.
  - So `status both` and `lookup`'s scope-then-target form go, each a breaking change, and `sync`,
    found without the positional at the cycle "feat: sync is safe by default", stays as it is.
- What no scope means follows what the command is about.
  - A command about one repo acts on `.`, the repo it is run in: `sync`, `chid`, `desc`, `list`,
    `show`. `status` joins them, where today it answers for the work-repo from anywhere.
  - A command about the pair acts on both, one side alone saying little: `validate-desc` and
    `fix-desc`, which compare the two repos' commits, and `push`, which writes both.
- A pair command prints a column per side, work and agent, wink's ask of `validate-desc`.
- On `squash-push`, `BOOKMARK` moves to `-b`/`--bookmark`, since once the default is the line's own
  bookmark, naming one is the rare override and the rare thing belongs on a flag. This is a
  breaking CLI change, and the callers are the user and `push`'s stage, which builds params
  directly and never parses argv.
- `-R`'s meaning shifts on `squash-push` from "the repo to operate on" to "the workspace root to
  resolve the scope against", which is what `status`'s `-R` already means. A second behavior change
  in one flag, so it wants saying out loud in the docs.
- `push` takes no scope: it is always both repos, as its own help says, and a flag with one legal
  value is documentation pretending to be an option. What `push` loses instead is `[BOOKMARK]`,
  resolving each repo's bookmark the way `squash-push` now does. That collapses its hardcoded
  asymmetry, the work repo's `BOOKMARK` against the agent repo's literal `main`, into one rule, and
  retires the warning in [jj.md](agent-data/jj.md#vc-x1-push-what-it-does-and-does-not-do) against
  mirroring a work bookmark into the agent repo.
- A `trunk()` tie-break is needed before `push` can resolve. At a cycle's opening the topic bookmark
  is created at `main`'s commit, so the first push's nearest bookmarked ancestor is one commit
  carrying two bookmarks, which today's resolution refuses. Dropping candidates that are `trunk()`
  when others remain reads as "the trunk sitting coincidentally at your commit is not your line", and
  leaves a genuine tie of two topic bookmarks still refused.
- `-s both` on `squash-push` raises what `status` never had to answer: with the work side pushed and
  the agent side failing, what the exit code is and whether anything is undone. The instinct is no
  rollback and a bit per side, the shape the status entry already proposes.

Sequenced with **status prints a verdict per repo and exits with a bit per side**, which is already
about scope handling and per-side exit codes, so the scope resolution is written once rather than
twice. Either one cycle covering both, or two adjacent with status first.

### A squash-push test reads the live repo's bookmarks

(wink, 2026-09-20) `squash_push::tests::the_default_is_to_act_without_asking` resolves a bare
`squash-push`'s default bookmark against the vc-x1 repo the tests run in, not a fixture. At a
cycle's opening the topic bookmark sits on `main`'s commit, the resolution refuses the two-bookmark
tie, and the test fails until the cycle's first push moves the bookmark off `main`. The test should
resolve against a fixture, or check only the `yes` default without resolving a bookmark. The
`trunk()` tie-break in [Scope is a flag on every
command](#scope-is-a-flag-on-every-command-and-push-resolves-its-own-bookmarks) would also clear
it, but a test should not depend on the live repo's state either way.

### Commit titles carry no scope, the declared types aside

(wink, 2026-09-17) [Conventional commit scopes](agent-data/prose.md#conventional-commit-scopes)
asks a title's scope for a component name, and [Conventional-commit
shape](agent-data/prose.md#conventional-commit-shape-ladder--commit) offers `feat(push): ...` as
its example. A scope on a plain type repeats what the description already says, and a cycle is
collected by the greppable stem its titles share rather than by the scope. The slot earns its place
only in a declared type, where it carries the declaration's own vocabulary, today
`agent-files(proposal)` and `agent-files(adoption)`. The change: a plain type carries no scope, a
declared type keeps its, and the shape section's example loses one. `prose.md` is a universal file,
so this runs as an `agent-files(proposal)` cycle that bumps the set version.

### lookup marks a write's own lines, labels each line number, and flags a renamed partner

(wink, 2026-09-12) Found trying **feat: vc-x1 lookup for dual repos** by hand before its Land.
`lookup agent` on a transcript write narrows the work window to the files the call wrote, and
that is too wide: session line 846 of `2b15dd23` wrote TODO.md and notes/bugs.md, and the window
printed nine regions, 188 added lines, while the call's own text carries about 67 of them. Most
regions were written partly by that call and partly by other calls in the same rung, so dropping
whole regions removes only two of nine.

Met again (wink + agent, 2026-11-03): `lookup --line src/init.rs:105 -r 4a827490` found
`vorzlzzx src/init.rs:90` and the write at `222bc372-8990-4566-b73e-25023e62a7aa.jsonl:1150`, and
the inverse, `lookup --line <that jsonl>:1150 -r tqtrxzvx` in the agent repo, printed every hunk of
`vorzlzzx` in the two files the call wrote, with no `>` on the lines it wrote. The two numbers, 105
and 90, are one line in two commits, the snapshot read and the commit that wrote it, and the output
named neither, so they read as a disagreement.

- Both directions answer in the commit where the write happened, so they round-trip: the forward
  lookup's `commit vorzlzzx src/init.rs:90` is what the inverse marks, never the line at today's
  snapshot.
- Mark the call's own lines: among the partner commit's hunks, the added lines the call's text
  carries, a heredoc body, a Write's content, an Edit's new string, print with `>` in place of `+`,
  and every other hunk stays, so the whole change is in view and the call's part of it is marked.
  This replaces narrowing by dropping, whose two-of-nine result is above.
- Label every line number with its commit: `work src/init.rs:105 at lpkmkxrk`, and on the commit
  row, `vorzlzzx src/init.rs:90`, the line where it was written.
- Optional `--work-rev <rev>` on the inverse: where the marked lines are at that revision, by blame
  there, a line still blamed on the partner commit with the same text being the same line, and which
  were changed or removed since.
- A renamed partner: `lookup TODO.md:100` printed a commit and its partner with different titles,
  "fix: clone says the right dir and stops on a rejected config" and "fix: clone's dry run and
  warning name the right files", though each trailer names the other. A push gives both the same
  title, so one side was re-described after its push. Print one line saying so when the titles
  differ, so the pair does not read as a wrong link.

### lookup --history steps a region through the commits that changed it

(wink, 2026-09-12) The time dimension of a lookup: from a line, step older and newer through the
commits that changed its region, each step showing the region there, its partner, and the
transcript write, as `git log -L` does for one side. The interactive form, clicking a line and
arrowing through time, is the session viewer's, the one the `## Waiting` entry names, so vc-x1
computes the chain and the viewer draws it.

- Two axes. The work axis is the region's commit history. The agent axis is the session timeline,
  stepping between writes to the same file or between push windows, and is the easier, since the
  timeline and windows exist.
- Older: blame names the last commit to touch the lines, and mapping the range through that
  commit's line diff gives the region in its parent, repeated to the region's first appearance.
  The reach back covers a moved line.
- Newer: a commit can have several descendants, and a trapezoid is that shape. Follow the line
  that reaches `main`, with a ladder's rungs a detour to step into.
- An amended step can show its predecessor, the first reader for the partner module's evolution
  walk.
- Shape: `lookup --history FILE:LINE` prints the chain oldest first, commit, range, partner, and
  write per step, with a JSON form for the viewer. Each step uses the line narrowing of the entry
  above. The `dr-1` fixture has a line edited across two cycles, a real move, and an amend to test
  it over.

### feat: vc-x1 msg, the messages protocol as code

(wink, 2026-09-11) The README's write actions run by hand, take, edit, release, commit, and the
title rule has a gap the hand cannot close: a commit that closes a thread and also carries lines
in other threads, or closes two threads, has no title under v0.3.2, since the close form names
one thread. The title is a convenience, not a store, so the gap costs a skim of `git log`, but
the answer is the subcommand the messages-rules note names under Further out: `vc-x1 msg`, with
open, reply, done, close, pending, and status, the repo found through `[family] messages` in
`.vc-config.md`, so the id allocation, the guards, the pending query, and the commit are code
and the by-hand steps go. We think a tool commits at each release, so a commit carries what one
take wrote and a batch never crosses threads. Found at **chore: update vc-x1-messages to
v0.3.2**, asking what a batch across threads is titled, and **chore: update vc-x1-messages to
v0.3.3** answered it, the ids of the threads the commit touches, so the tool titles by that rule
and its own question is whether it commits at each release. Its own cycle, multi-step, and next.

### sync clones a declared but absent agent-repo

(wink, 2026-09-06) A dual clone that stops because the cloned config declares no agent side
leaves a POR, and today the recovery after fixing the field is a by-hand clone of the derived
`.claude` remote into the directory the field names, then `vc-x1 symlink`. Sync already resolves
its repo set from the config on disk, so a declared `repos.agent` whose directory does not exist
is a fourth state beside up-to-date, behind, ahead and diverged, "absent", whose act is the
second half of `clone_dual`: a colocated clone from the work remote's derived URL, then the
symlink. The config need not be committed, since sync reads the file, so the user fixes the
field, runs `vc-x1 sync .`, and the agent that then starts has both sides and commits the fix
as a cycle. Two guards: the work repo must have a remote to derive from, a clear error otherwise,
and "absent" means the directory is missing, not present and not a repo, since a stray
directory at that name is the user's. When this lands, the clone error's second line says "then
`vc-x1 sync .`" instead of leaving the clone to the hand. The scenario is a new vc-x1 meeting an
old repo, and #18 in [bugs.md](notes/bugs.md) is where it was found.

### clone takes --agent for the agent-repo's source

(iiac-perf, 2026-09-06) The clone's local location now follows the cloned config, and what is
still fixed by convention is the remote: `derive_bot_url` appends `.claude` to the work-repo's
source, so clone can only fetch an agent-repo that sits in the same namespace under the same
owner. That is the piece the multi-contributor case breaks, since a second contributor's
agent-repo lives under their own owner, and the URL cannot come from the work-repo's config for
the reason the design note gives. The general shape is a second source, `vc-x1 clone <work-url>
--agent <url>`, with the derivation as the default when the flag is absent: one flag, and the
onboarding path from the note, cloning someone else's work-repo with your own agent-repo, is one
command. After the sync entry above, since sync cloning an absent agent-repo needs the same URL
and decides where it is stored, a flag or a per-user config.

### clone from a path refuses a dirty source

(wink, 2026-09-06) A clone fetches the source's bookmarks and checks out `main`, never the
source's working copy, which is right for a URL and a surprise for a path: the user is looking
at the directory, edits `.vc-config.md` there, clones, and gets the old key, with the tool
blamed. Found running the #18 fixture. For a path source, a working copy with changes is an
error, not a warning, since the path form is rare and deliberate, the cost is one commit, and
`jj git clone` is there for anyone who wants the draft left behind. The message names what
would be checked out and where it sits against `@-`, so the user knows whether committing is
enough or `main` must move too:

```
error: ../src/work has uncommitted changes, and a clone checks out main at bc7a4366, which is @-.
Commit and move main, or use jj git clone to check out main as it is.
```

with "two commits behind @-" when they differ. A clean working copy with `main` behind `@-` is
a feature bookmark in progress and is not refused.

Rides with this cycle, from iiac-perf's review of the landed clone fix: a test that a config
declaring the agent-repo outside the work-repo's tree, `../name.claude`, clones there, since the
resolution already follows the config but the symlink then points outside the target directory
and nothing exercises it.

### clone takes -b for the work-repo's bookmark

(wink, 2026-09-06) `jj git clone -b` picks which fetched bookmark the working copy sits on, and
vc-x1 clone always picks `main`. Reviewing a cycle branch on another machine, which is how the
stale binary on `7600x` was found, wants the branch checked out. `-b <bookmark>` on the work
side, `main` by default, the agent side pinned to `main` as always, since it is a linear
journal. The cloned config is then read from the chosen bookmark's tree, which is right: a
branch that renamed the agent directory clones to the renamed one. The other `jj git clone`
flags stay out: `--depth` breaks the `ochid:` cross-links, which need history on both sides,
and `--remote`, `--tag` and `--object-hash` are not choices a workspace should differ on. The
dirty-source error above names the chosen bookmark in place of `main`.

### Continuation notes leave the work-repo dirty after Land

(wink, 2026-09-01) Close-out step 7 has the agent write `## Continuation notes` before the exit,
and Land's last push has already gone, so the session ends with `TODO.md` modified in the
work-repo while the agent-repo is clean, and At rest's "both `@` empty" does not hold at the one
moment it is checked by eye. Seen at the first Land under the rule. Think about where the notes
belong: committed by a push of their own, which the hard stop after the final push forbids as
written, written before the closing rung so the closing carries them, kept in the agent-repo
whose session data is the same kind of ephemera, or accepted as the one dirt a restart is
allowed to leave, and say so in At rest.

- Tried at the **docs: check the transcript join on two landed trapezoids** close-out
  (2026-09-03), the third candidate: the notes went into the closing commit before its push, and
  the work-repo ended clean and stays clean through Land. The cost is notes written before the
  Land they describe, which suits a close-out and would not suit a mid-cycle stop.
- The design from the Land of **feat: vc-x1 land** (wink + agent, 2026-10-27), the third
  candidate again: the notes go in the closing. A fact known only after the Land is not written:
  the note names what the next session checks, `main` at the closing in both repos and on origin,
  the cycle's bookmarks gone, and the installed version's commit, and machine-local evidence, a
  `cmp` or the Land's output, lives in the agent-repo's session, published by the user's
  `squash-push -s a` after `/exit`. A commit after the Land puts a non-cycle commit on the trunk
  after the hard stop, and a squash into the closing rewrites the landmark just landed and
  verified. Carried as **Continuation notes go in the closing** in **agent-files(proposal):
  v0.3.1**.

### Land validates and installs before the main push

(wink, 2026-09-01) The first Land under the rule ran the full validation, with its install, after
the name restore and before the `main` push, rather than jj.md's step 4 after it, so the plain
binary came from the commit `main` was about to carry and the pre-push window with a stale `vc-x1`
closed. Write that order into Land, with the note that a single-step draft's validation installs
under the plain name and a later conversion to multi-step leaves that install behind until Land. A
convention change, paired with the entry above.

### push refuses when no full validate passed on the current tree

(wink, 2026-09-03) Two rules in [Before any push](AGENTS.md#before-any-push) have no check and both
fail in silence: that validation ran and passed after the last edit, and that the full run, the one
that installs, is what a review is owed. This cycle broke the second for three rungs. The installed
`vc-x1-dev` reported the opening's `0.83.1-0` while the manifest had reached `0.83.1-4`, and nothing
said so until wink ran `vc-x1-dev -V` by hand. A doc-only rung makes it likeliest, since `--fast`
looks sufficient there and the install looks like waste.

- The proposal: a successful full `vc-x1 validate` writes a stamp holding the working copy's tree
  identity and the manifest version, and `vc-x1 push` refuses when the current tree does not match
  it. The message names the version gap, which is the legible form of the failure.
- The artifact needs no inspection: `cargo install --path . --locked` is the last element of
  `[validate] full`, so a full run that passed on this tree installed by construction. Recording
  that the run passed is stronger than checking what it installed, and it carries to an adopter
  whose artifact has no `-V` and no cargo.
- Rejected, `~/.cargo/.crates2.json` (wink, 2026-09-03): it holds the name, version, and source path
  such a check would want, and it is a cargo internal. It succeeded the `[v1]` `.crates.toml`, which
  cargo still writes beside it, so the format has churned once already, and the newer file carries
  no version marker of its own, so the next change misparses rather than announces itself.
- Rejected, the running binary's own version: the flow invokes `vc-x1 push`, the stable binary an
  earlier Land installed from this same path, whose name and version differ from a mid-cycle
  manifest's every time. The check would need a paired rule that the flow runs `vc-x1-dev`, and an
  escape hatch for a dev build too broken to push itself.
- The counter-evidence, and why this is an entry and not a decision (wink, 2026-09-03): a tracking
  file has burned us here before, in [bugs.md](notes/bugs.md) item 8, where `push` adopted a stale
  `.vc-x1/push-state.toml` from an earlier invocation, resumed at its final stage, squashed a new
  session's transcript into an already-published bot commit, and force-pushed it sideways, leaving
  permanent residue in iiac-perf's repo. Whoever takes this weighs that first. A stamp holds facts
  that are compared rather than stages that are resumed, which is the smaller thing, and the same
  hazard still applies: a stamp that cannot be trusted has to fail closed and demand a validate.
- wink's reading (2026-09-03): the root problem may be that the instructions are too complex, in
  which case a gate patches over the complexity and the cheaper fix is fewer or clearer rules. [The
  per-rung flow](AGENTS.md#the-per-rung-flow) step 5 is one clause requiring the full run and one
  granting `--fast` for iteration, and the grant is the half that got read.
- Pairs with **Land validates and installs before the main push**, the same install-currency
  question at the other end of a cycle.

### At rest names vc-x1 status as the verdict's printer

(wink, 2026-09-02) AGENTS.md's At rest defines "clean" as both `@` empty and names no command
that answers it. The **feat: the status and agent-files commands** cycle gives the word a home,
`vc-x1 status`, and the one-line pointer in At rest is an agent-file change, so it runs as its
own cycle after that one lands, with the two close-out entries above if they are ready.

### The cycle-record's items are an intro and bullets, and a pronoun names its noun

(wink, 2026-09-03) A cycle-record's Problem came out as three sentences with thirteen commas, a
wall of prose, with an "it" whose referent was four words from a different "its". The Prose form
already asks for a short intro and bullets and names the cycle-record as a surface, but the
specimen every block copies shows paragraphs, so the paragraph wins. Fix and unify four places
in one `agent-files` proposal cycle:

- cycle-model.md: rewrite the specimen's Problem, Solution, and Acceptance check as an intro
  sentence and bullets, the shape the **docs: check the transcript join on two landed trapezoids**
  opening settled on, since "copy the shape, not the words" is the specimen's own instruction.
- notes.md, The In Progress block: each item is in the Prose form, an intro sentence that states
  the claim and bullets that carry the detail, and the acceptance check is one runnable check per
  bullet. Replaces "a sentence or two".
- prose.md, a pronoun rule beside Semicolons: a pronoun whose referent is not the sentence's
  subject is replaced by its noun, and two referents in one sentence are both named.
- prose.md, a density heuristic a reader can run: a sentence with more than three commas, or a
  paragraph with more than three sentences of detail, becomes an intro and bullets.
- prose.md, a term rule: use the Terminology section's term when one exists, since the family
  shares words across projects and "records" already means iiac-perf's `--records` option to its
  owner, where the cycle-record was meant.
- AGENTS.md and jj.md, "Land" as a proper noun: the Terminology entry and jj.md's heading make a
  verb into a name, and a reader is left to guess whether it means the push or the arrival on
  `main`. Say "land on `main`", or on whichever bookmark, wherever the name is used, and retire the
  entry.

Evidence, 2026-09-20: rung 2 of **feat: init adopts an existing tree** took four review rounds on
its `Ladder details`, and every round found a record-prose defect rather than a code one. The `*`
facets stated answers where problems go, "the name" left its referent a paragraph away, the text
assumed the reader knew `.vc-config.md` and the markdown-as-config carrier, and "this workspace
records" gave a workspace agency it has not got.

### repos.agent becomes repos.agent-dir, and a command brings a config up to date

(wink, 2026-09-02, 2026-09-21) Two changes that need each other: a key rename every adopter must
apply, and a command that applies it.

- `repos.agent` becomes `repos.agent-dir`. It holds the agent-repo's directory while `[remote]
  agent-repo` holds its name on the remote, and the flag that sets the directory is `--agent-dir`.
  A key's leaf is its flag's long name everywhere else.
  - Breaking, unlike the `[remote]` key: every adopter's config carries `repos.agent`, and
    workspace-root discovery reads it. The old key is rejected with a fix-it, as `repos.bot` was
    at 0.80.0 (`reject_old_agent_keys` in `src/legacy_vc_config.rs`), never read as an alias.
- A command brings a config up to date, and the fix-it names it. Today `vc-x1 config` prints and
  validates, and a workspace whose config predates a change learns of it only by reading the model.
  - It renames every old key found, across both sides' config files, from one old-to-new table that
    also carries `repos.bot`.
  - It adds the model's tables and keys the file lacks, as commented lines with their default or
    example, and leaves what the file holds untouched.
  - It edits in place and keeps the prose: in a `.vc-config.md` only the key lines inside the `toml`
    fences change, and prose that still names an old key is reported rather than rewritten.
  - A dry run by default, as `fix-desc` and `fix-todo` are, and the result is reviewed in the
    working copy.
  - First use: this repo's `.vc-config.md`, which the **feat: the status and agent-files commands**
    cycle left without the `[agent-files.*]` tables on purpose.
- The name is undecided. iiac-perf has `update-config FILE`, which does the same job by
  regenerating the file from its template, keeping the values and losing the prose, and fails on a
  key it no longer knows rather than renaming it. The same spelling across the family would be
  nice, not required. The candidates are `update-config`, `config --update`, and `config update`
  under **Nest the validate and fix commands**.

### --repo takes a URL, and a separate flag takes the path for local remotes

(wink, 2026-09-21) `--repo local=<dir>` reads as the local work repo, where it names the directory
init creates the bare origins in, standing in for GitHub. The two categories of one flag also
overlap, since `remote=` takes a path prefix too, meaning bare repos someone else created. Split
them by what they take:

- `--repo <url>` names a remote on a server, a URL and never a path.
- A separate flag, `--repo-local <path>` or `--repo-test <path>`, the name for this cycle to settle,
  takes a path and only a path, and init creates the bare repos there. The two conflict.
- Init creates the path when it is missing. Today a missing one fails with jj's "Could not open
  data at", and only at the publish step, after an adopt has already committed the work repo, so
  preflight checks it before anything is written.
- The bares are named after the project, `<path>/<name>.git` and `<path>/<name>.agent-session.git`,
  not the fixed `remote-work.git`, so one path holds several projects' remotes as one GitHub account
  holds several repos. The fixtures and `tests/cli_sync.rs` are built on the fixed names.
- The user config's account keys, `repo.default` and `repo.category.<cat>`, were shaped for one
  flag with categories (0.41.1-4). Two flags want a key each, named by their long names, and the
  old keys rejected with a fix-it, which is work for [repos.agent becomes repos.agent-dir, and a
  command brings a config up to
  date](#reposagent-becomes-reposagent-dir-and-a-command-brings-a-config-up-to-date).
- `local=` and `remote=` as category values are rejected with a fix-it, not kept as aliases.

### The code says agent where it still says bot

(wink, 2026-09-21) 0.80.0 renamed the bot side to the agent side in the config and on the CLI, and
the code kept the old word: about 314 `bot_` and `Side::Bot` identifiers across `src`. A reader
meets both words for one repo, and output a user reads still carries the old one.

- Output first, since a user reads it:
  - `clone`'s summary prints `Bot repo:` (`src/clone.rs`).
  - `push`'s stages are `commit-bot` and `squash-push-bot`, and it prints `restored bot repo`.
  - `push` prints `.claude had no pending changes` whatever the agent repo's directory is, which is
    wrong as well as old.
  - `fix-desc` says `no bot side`.
- Then the identifiers: `Side::Bot`, `is_bot_dir`, `write_bot_config`, `bot_dir` and its kin, and
  the `bot_session` and `validate_bot` modules.
- Out of scope: the old-config code in `src/legacy_vc_config.rs`, whose `bot` spellings are the old
  names it rejects, and the `--scope bot` rejection, which names the old value on purpose.
- `create_dual` in `src/init.rs` was renamed by **feat: init defaults to .agent-session**, as the
  first function its rung rewrote.

### The opening deletes the closed block's reference definitions with it

(wink, 2026-09-04) A closed block's ladder links its rungs as `- [<title>][N]`, and the `[N]:
#<slug>` definitions live outside the block, in the file's `# References`, pointing at its
rung subsections. The opening's "delete whatever `## Closed` holds" takes the block
and leaves the definitions, so every opening since the ladder form arrived left dead definitions
behind, seven at the `agent-files(adoption): v0.2.0` opening. Nothing said so, because
`validate-anchors` is not in `.vc-config.md`'s `[validate]` table. Two edits: [The In Progress
block](agent-data/notes.md#the-in-progress-block) says the definitions go with the block at the
next opening, and `validate-anchors` joins the validate table once its three known failures are
fixed, so a dead definition fails the push that made it.

### Global -R anchors the workspace for every command

(wink, 2026-09-01) `vc-x1 version -R vc-x1` and `vc-x1 -V -R vc-x1` from a parent directory are
refused, since `-R` is not global: seven subcommands declare their own, with their own defaults
and meanings, and the workspace root finder is anchored on the cwd. One global `-R <path>` on the
root command, as jj has it, anchors the root search for every subcommand, `version`, `-V`, and
the `agent-files` group included, so the report and the banner follow it, and the per-subcommand
flags retire or become its aliases. CLI-surface consolidation, paired with the nesting entry
below.

### Nest the validate and fix commands

(wink, 2026-09-01) Six flat `validate-*` commands, their `-old` variants, and two `fix-*` are a
namespace asking for `validate {bot|desc|config|anchors|todo}` and `fix {desc|todo}`, bare
`validate` staying the full run. The flat names stay as hidden aliases for a while, the validate
table and typing habits using them, and retire in a later cycle.

### Support POR workspaces in vc-x1 push

(2026-08-31) `vc-x1 push` refuses a POR workspace at its first stage (`require_bot_dir`: "this
operation requires a dual workspace"), confirmed by probe at 0.80.7, and the July audit records it
as dual-only ([audit](notes/design-cli/por-dual-parity-audit.md)). Support is auto-detected, not a
flag: `init` needs `--por` because it creates the topology, while `push` reads an existing one, so
`bot_repo_path()` returning `None` is the POR signal. On a POR the bot stages skip and the
`ochid:` trailer is omitted, there being no other side to name. The opening rung is a test pinning
today's refusal, the error names a dual workspace and nothing is mutated, which inverts into the
POR-success test when support lands.

### Get defaults from .vc-config in cli processing

When processing the default parameters for a vc-x1 subcommand look in .vc-config.
For instance `vc-x1 validate-desc` should have a default that is both repos
and their locations are in vc-config::repos.*. This is also a way to determine
if a repo is a dual repo or not.

- Overlaps [Support POR workspaces in `vc-x1 push`](#support-por-workspaces-in-vc-x1-push): `bot_repo_path()`
  reading `repos.agent` is the same dual-or-POR signal that entry names.

### status prints a verdict per repo and exits with a bit per side

(wink, 2026-09-03) `vc-x1 status` prints each scoped repo's `jj st` block and a summary line,
more than the At rest check needs. The redesign: the default output is one line per scoped repo,
`<label>: clean` or `<label>: dirty: <why>`, the why being the `@ has changes` and `@ is
described` the verdict already names, and the global `-v` restores today's blocks. What `-vv`
adds is left open until a use shows up. The exit code is a bit per side, `work` 1 and
`.agent-session` 2, so `both` exits 0 clean, 1, 2, or 3, and the code means the same repo
whatever the scope, since the scope is one keyword and `both` runs work then agent. Errors exit
outside 0 to 3, so `$?` is never ambiguous, which means the command returns its own exit code
rather than the runner's Ok-or-1 mapping, as `agent-files diff` does. The per-repo verdict is
exposed as a function, since **Enhance squash-push** calls it for its precheck and after-check,
and it carries the bookmark's publish state beside the working-copy verdict, since that entry's
"clean" needs both. The docs follow: the command's help, the README's status section, and the
At rest pointer entry above, whose wording describes the output.

### status reports the commits no bookmark reaches

(wink, 2026-10-11) A commit that no bookmark reaches, local or remote, is shown by no vc-x1
command: `status` reports `@` alone, and `sync --dry-run` leaves such commits out by design, since
no sync moves them. They are found today by a jj revset typed by hand. Have `vc-x1 status` report
them for each scoped repo, as information.

- The heads alone: `visible_heads() ~ ::(bookmarks() | remote_bookmarks()) ~ @`.
- Every such commit: `mutable() ~ ::(bookmarks() | remote_bookmarks()) ~ @`.
- Which of the two is reported, or both, the second perhaps behind `-v`, is open.
- They are information and not a fault: they change neither the verdict nor the exit code of
  [status prints a verdict per repo and exits with a bit per
  side](#status-prints-a-verdict-per-repo-and-exits-with-a-bit-per-side).
- The work-repo holds three at this writing, `2fbd6c40`, `b34cb7b3`, and `7694fce7`, the heads of
  remote bookmarks deleted during the cycle **feat: sync is safe by default**, left in place as a
  case to try it on.

### Write up who owns a config file's prose

(2026-08-28) The cycle **feat: finish the vc-config surface** reversed half the 2026-08-10
ownership model and left the reversal in its closed record, where it is found only by knowing which
cycle produced it. The rule now is that a fence interior and the prose around it are both the
workspace's own, and the tool checks a config file rather than regenerating it, because a renderer
owning every adopter's prose would cost each adopter the ability to explain its own config. That is
a rule about what the tool does rather than a record of what was done, so it wants its own topic
file ([notes/README.md](notes/README.md)), say `notes/config-ownership.md`.

- Ranked first (wink, 2026-08-28). History holds the reversal either way, and the risk it guards
  against is a regenerating config surface being proposed again before the rule is findable.
- The retired **feat: add config --refresh** and the `--output` question still open in **Fix
  `vc-x1 config`'s rendering** are the two places the reversal changed a plan, so the file should
  name them.

### `validate`: enforce the record shapes the agent-files ask for

(2026-08-25) The shapes the agent-files state in prose are missed at the point of action and
invisible on reread, so each checkable one becomes a validate element and `vc-x1 push` refuses what
fails. First set:
- every ladder rung title is `<type>(<scope>)?: <desc>` with a type from the conventional-commit set
  or a project-declared type per [Project-declared
  types](agent-data/prose.md#project-declared-types) (nine untyped rungs went unnoticed through
  five rereads on 2026-08-25)
- a `--body` is an intro paragraph then `*` facets each with at least one `-` under it, or the intro
  alone for a bookend
- an unfilled `[[N]]` whose commit is on `main` (the backlog entry **`validate`: fail on an unfilled
  `[[N]]` on `main`** folds in here)
- every Todo and backlog entry is a `###` heading, its title unique within its file
- every `](path#anchor)` and `[N]: path#anchor` in the agent-files resolves against the headings
  (the check that found two dead links on 2026-08-27, run by hand)
- `TODO.md` has the pinned shape: `## Continuation notes`, `## In Progress`, `## Closed`, `##
  Waiting`, `## Todo`, `## Ideas`, `## Bugs`, `# References`, in that order, and the In Progress
  block per cycle-model.md (2026-08-27)
- `## Todo` entries are `###` headings, no numbered entries anywhere in the file, and
  `validate-todo` / `fix-todo` retire, the numbered form they served being gone (2026-08-27)
- a per-file punctuation baseline, since a byte scan cannot be the check: the rule forbids
  *authoring* the four characters rather than their presence, and transcribed tool output and
  published commit titles keep theirs, so record a per-file count and fail when one rises (carried
  2026-08-28 from the retired backlog entry **Add `validate-repo` subcommand**, and it supersedes
  the chores-15 note asking the checker to read its character set from one place, which assumed a
  checkable zero)
- wrapper-level tests for `validate-desc` / `fix-desc` ride along: the analyze cores are covered,
  the wrappers (file I/O, output, exit codes) are not

### The vc-config program: finish the surface, then shrink it

The markdown carrier landed and the rest of the config subcommand's work was spread across six
entries. They share one surface, one schema and one set of tests, so they run as one program rather
than six rankings. Sub-entries keep their bold titles, so a citation by title still resolves.
- **Fix `vc-x1 config`'s rendering: print once, and write with `--output`.** (wink, 2026-08-21) Bare
  `vc-x1 config` prints the schema once per side of the default `work,agent` target, and since every
  remaining key has both workspace homes the two blocks are identical apart from the header, so the
  reader sees the same ~40 lines twice. In a workspace with no agent side the second block is still
  printed, under the `<root>/<agent-dir>/...` fallback hint, for a side that does not exist.
  - print the schema once by default, grouped per side only when the sides' key sets differ (they
    will again once `[family]` and `[validate]` land as `workspace-code`-only)
  - add `--output <scope>:<path>[,<scope>:<path>]` or some such, writing each side's rendering to a
    file instead of stdout, so a side's config can be (re)generated in place. This overlaps `config
    --refresh` in "Finish the vc-config surface" and should be designed with it, one verb or two
  - skip a side the workspace does not have, rather than rendering its fallback hint
  - accept a directory as a path target (wink, 2026-08-21): `config --validate .claude` or
    `../iiac-perf` resolves through the carrier lookup (`config_md::vc_config_path`) to that side's
    config file, the both-carriers error included, and the report labels the side by the directory.
    Today a path target must name the file itself
  - an explicit path must exist (wink, 2026-08-21). Today `config xyz` prints the whole schema for
    "any home" with the path as a label and never opens it, and `config --validate xyz` reports the
    file "not found, skipping" and passes with zero problems, so a typo'd path validates clean. The
    skip is right for a keyword side the workspace lacks, wrong for a path the user typed: error by
    name, and say which file was read
  - the rendered hints still say `.vc-config.toml` (the `VC_CONFIG_FILE` constant), and the md
    carrier rename is the "regenerate configs in md format" rung's

- **`config --toml`: print the TOML a markdown carrier yields** (iiac-perf + bot, 2026-08-12). The
  md carrier costs a config file the toml-aware editors and formatters a `.toml` gets, and nothing
  answers "what do these fences actually concatenate to?", which is also the question a parse
  diagnostic raises. Outside the "docs: freshen vc-config and config subcmd" ladder, whose
  acceptance items do not need it, but ranked here because a format's debugger is worth most while
  the format is new.
  - run the `md_fence` filter over the target file and print the result verbatim, blanks included,
    so the printed line numbers are the source's and a diagnostic's line lands
  - **not `--resolved`**, iiac-perf's word: this subcommand already spends "resolved" on
    effective-after-layering (the `[repos]` resolved-agreement invariant, `resolved_hint`'s
    which-carrier-exists answer), and this is the far end of that, one file's raw extraction before
    any parse or layering
  - it has no existing surface to join: `config` with no flag prints the *schema*, not a workspace's
    values, so nothing today shows a config file's own contents at all
  - decide there: the name (`--toml`, `--as-toml`, `--fences`), and whether it composes with
    `--validate` or excludes it

- **Drop the global config and the account notion.** vc-x1 loads a user-level
  `~/.config/vc-x1/config.toml` whose whole remaining job, once the unread keys go, is expanding an
  `init` shorthand that the `owner/name` and path target forms already cover without it (wink,
  2026-08-11: he passes the full url in practice and a local name only when testing). A config tier
  nothing needs is the same rot as the fossil `[push]` block, so it goes, and the schema drops from
  eleven keys to five.
  - out: `src/config.rs` entire (loader, `UserConfig`, the account map, the `--account` ->
    `[default].account` resolution chain), the `--account` flag, `Context.user_config` and its disk
    read at every subcommand entry, `Home::User`
  - out of the schema: `default.account`, `default.debug` (parsed, logged, never consumed),
    `repo.default`, `repo.category.<cat>`, and both `account.<name>.*` families
  - what remains is five keys in two files: `[repos]` on both sides, `[bot-session]` on the work
    side
  - `homes` becomes `files` with values naming the two sides only, so "user" leaves the vocabulary
    and stops colliding with `account` (wink: a human reading "user" and "account" connects them,
    and here they were unrelated axes)
  - removing `--account` breaks an invocation, so it errors by name and points at this entry's
    record rather than reporting an unknown flag
  - decided 2026-08-21 (wink): `init` takes a URL or a path, nothing else. No bare name, since a
    bare name has no host-neutral meaning and delegating it to `gh` would make the convenience
    GitHub-only. No user tier of any kind: identity is jj's config, credentials are git's helpers (a
    GitLab token in the helper makes `vc-x1 clone` work unchanged), and the remote is the URL. The
    `owner/name` shorthand goes too, since it hardcodes an ssh remote. bugs.md #10 (a pre-created
    GitHub repo rejected at preflight) rides this entry
  - provisioning stays host-keyed from the URL: `gh` for github.com as today, and a `glab repo
    create` arm for gitlab.com is the next one worth adding. Measured 2026-08-21: an authenticated
    push to a nonexistent gitlab.com path is refused ("could not be found or you don't have
    permission"), so push-to-create is not something init can lean on (we think a token with `api`
    scope might allow it, but an instance setting and a token scope are not a foundation). Until an
    arm exists, non-GitHub hosts pre-create both remotes
  - the account model is worth resurrecting if a second repo host ever matters: a backlog entry
    names the cycle that removed it and lets the diff carry the design, rather than restating it in
    prose that can rot
  - runs after the vc-config cycle on purpose: `--refresh --check` makes a schema shrink mechanical,
    so this is the first real customer of the machinery that cycle builds

- **Tiered exit status for `config --validate`** (wink, 2026-08-12). Today every failure is
  `ExitCode::FAILURE`: a misspelled key and a config the tool could not read exit alike, so a caller
  can branch on "clean or not" and nothing finer. Proposed: **0** all tables and keys known and
  their values reasonable, **1** unknown or otherwise non-fatal findings, **2** a fatal situation.
  The convention is grep's and diff's, so it needs no teaching.
  - the fatal cases already exist and are the subject of bugs.md's **`config --validate` reports "I
    gave up" as a finding** (#9): malformed TOML, an unclosed fence, a side holding both carriers, a
    legacy `[workspace]` schema. Every one of them means the check could not be performed rather
    than that it failed
  - **sequenced after that bug**, which draws the "found something" / "could not check" distinction
    as a local fix. Once drawn, the exit status is a rendering of it, and doing the tiering first
    would mean inventing the classification twice
  - the cost is not in `config`: `main` maps every subcommand error to `ExitCode::FAILURE`
    (`main.rs:477`, `:507`, `:514`), so a distinct code needs the error path every subcommand
    shares. Cheapest to take while that path is open for another reason
  - tier 0's "values reasonable" describes a capability that does not exist: `key_known` compares
    key paths only and no value is ever inspected. Read tier 0 as "keys known" at the start, and
    value checks land later as ordinary tier-1 findings
  - decide there: whether `--refresh --check`'s difference exit joins this scheme (a difference is a
    finding, not a fatal) or keeps its own

- **config: extract flag-backed key descriptions from Clap.** `config`'s key descriptions live in
  `config_schema.rs` (`doc`/`used_by`). For the handful of keys that map 1:1 to a CLI flag
  (`bot-session.col-width` <-> `--col-width`, `--result-lines`), the description could instead be
  pulled from the Clap arg's help via `Cli::command()` introspection, so `vc-x1 config` and `--help`
  share one source and can't disagree.
  - Only ~2 keys map cleanly (most are config-only, flag-sets, or value-providers), so it's a
    partial source and the schema stays authoritative for the rest.
  - Defaults still come from the schema/consts (the args dropped `default_value_t`, so Clap no
    longer holds them).
  - Output format is unchanged, only the text source, so no rework of the 0.71.0-9 rendering.

- **init distributes vc-config.md, reference-base at the member.** (wink + agent, 2026-08-10, folded
  in from the backlog 2026-08-28) `vc-x1 init` seeds a new member with a copy of `vc-config.md` from
  the template payload and stamps `[vc-config] reference-base` with the member's own repo url, so
  every generated doc-reference web link in the member's `.vc-config.md` lands on a copy the member
  owns.
  - the copy is a pinned family file: a member's edits diff against the payload and are its doc
    proposals, folded back at convergence
  - folded here because **chore: regenerate configs in md format** already teaches init to emit
    `.vc-config.md` from the generated model and already owns `reference-base` as the key that
    survives a refresh, so the two are the same code one rung apart

- **Config provenance names the schema, not just the binary.** (iiac-perf + agent, 2026-08-12,
  folded in from the backlog 2026-08-28) The schema is generated at build time from `vc-config.md`,
  so an installed binary validates against its build's prototype rather than the workspace's, and a
  key added after that build is reported unknown with the config blamed. Member repos run a binary
  built from this one, so the exposure is the family's.
  - provenance already prints, keyed to the binary: `--validate` opens with the version banner and
    `print_schema` with its "settable config keys (from ...)" line, so this is one field on two
    lines that already exist rather than a new flag
  - the gap is that a version identifies the *build*, while the question behind an unknown-key
    complaint is whether that build's `vc-config.md` equals the workspace's. A content hash of the
    prototype, baked by build.rs beside the schema and printed next to the version, answers it
    exactly
  - not covered by **Tiered exit status for `config --validate`**, which was asked and is worth
    recording: an unknown key is tier 1 whether it is a typo or a stale binary, so the exit status
    is the same either way. That entry carries severity (could the check run at all) and this one
    attribution (whose fault the unknown key is), and only the second tells a reader whether to fix
    a spelling or to rebuild
  - decide there: a hash or a schema version. A hash is free, exact, and unreadable, while a version
    is readable and someone has to remember to bump it
  - folded here because build.rs is already open at **chore: regenerate configs in md format**,
    which generates the model from the same prototype

### validate-repo-data

Golden ids for a fixture repo, so a jj-lib bump that moves the on-disk data fails loudly instead of
building green. The gate at `0.78.0-4` refuses on a version mismatch precisely because we cannot
tell whether the data moved. This is the check that could eventually tell us, and the route to
relaxing the gate's coarseness. See [the
policy](notes/jj-version-policy.md#how-this-could-be-relaxed). Two modes over one fixture and one id
extractor:

- **Ratchet**, in `cargo test`. Record ids under the current jj-lib, commit them, and let the *next*
  bump re-run them. Zero standing cost, catches drift the moment we take a new version.
- **Live pair**, a `support/` script, not a `#[test]`, so `cargo test` never pays for it. Build a
  probe binary twice, against N-1 and N, run both over the same fixture, diff the reported ids.
  Generate a throwaway manifest in a temp dir for each version rather than adding a crate to our
  lock.
- **Trigger the live pair on the jj-lib bump, not on our release cycle.** Our cycles run faster than
  jj's releases, so per-cycle mostly re-compares the same pair. The bump is when the answer can
  change, and it is also when the answer is most useful: "should we take 0.44?" is a question the
  probe can answer *before* we commit to the bump.
- The probe needs only the storage-facing API: load a workspace, read operation / view / commit /
  change ids, create a commit. That is jj-lib's stable surface. The 0.43 break that motivated this
  whole cycle (`use_glob_by_default` leaving `RevsetParseContext`) was in revset *parsing*, which
  the probe never touches, so keeping it compiling against N-1 should stay cheap.
- **What it does not cover.** It compares two versions *on our fixture*. A change touching a path
  the fixture does not exercise reports "same" and is wrong. A sample, like `jj -V` is a sample, so
  say so where it is documented rather than letting it read as proof.
- **Watch operation ids and view ids first.** Those are jj's own content-addressed op-store hashes,
  so they move if hashing, serialization, or a stored field's meaning moves. Commit SHAs are gix's,
  computed from commit content, so they mostly pin git rather than jj and are the weaker signal.
- **Change ids are goldenable, and are the best canary in the set.** Three cases:
  - a commit authored in jj gets a random chid (`JJRng::new_change_id`)
  - a git commit carrying a `change-id` extra header keeps the original
  - a git commit without one gets a *deterministic* chid, the commit id's bytes `4..20` reversed and
    bit-reversed (`git_backend.rs`, `synthetic_change_id_from_git_commit_id`)
  Build the fixture by importing git commits and every chid is reproducible with no seeding at all.
- That function's doc says "the exact algorithm for the computation should not be relied upon", so
  jj reserves the right to change it. That is a documented instance of the schema-invisible drift
  the gate exists for, and this test is what would catch it: the algorithm moving changes every
  synthetic chid at once.
- **Determinism for the rest.** Operation ids embed timestamps and commit ids embed author and
  committer time, so those still need a pinned clock. Random chids, if the fixture needs any, are
  pinned by the `debug.randomness-seed` config key (`settings.rs`), which arrives through
  `StackedConfig` and so is reachable from jj-lib without going near the CLI.
- **A committed fixture, not this repo.** Using vc-x1's own repo as the guinea pig was the original
  sketch, but its history grows every commit, so the goldens would churn and stop meaning anything.
  A small fixture stays stable and fast, but this repo can still be a manual proving ground.
- Read-only commands get the complementary assertion: hash every file under `.jj/` before and after,
  and record which ones are genuinely inert. That is the measurement the policy names as the way to
  narrow the gate from "every subcommand" to something smaller, backed by evidence.

### refactor: trapezoid-push + body-intro validation

`vc-x1 trapezoid-push`, a **subcommand** rather than a flag on `push` (decided 2026-07-28),
publishes a close-out as a non-fast-forward merge, and body-intro validation rides as the first
rung. See [trapezoid close-out](notes/refactor-20260716.md#stage-trapezoid-close-out) and [push
body-intro validation](notes/refactor-20260716.md#stage-push-body-intro-validation). After jj-lib,
so the reshape is built in-process.
- `push` keeps a stateable invariant: it never produces a merge. A mode flag that rewires the stage
  sequence would cost that.
- Shared implementation, not a second copy: the common pipeline (preflight, both gates, message,
  commit-work, commit-bot, bookmark-set, push-work, bot squash) moves into its own module that both
  subcommands call, with the reshape as the one inserted step. The stateless-push cycle shrinks that
  pipeline first, which is what makes the extraction cheap.
- A backend `trait` (jj today, git or another VCS later) is the natural next abstraction if a second
  backend ever appears. Worth converting these concepts to traits then, not now: we are committed to
  jj, and a one-implementation trait buys nothing but indirection.
- The last stage of the retired jj facade refactor program (its as-built ladder is in
  [refactor-20260716.md](notes/refactor-20260716.md#as-built-trunk-ladder-program-retired-2026-08-18)).
  Parked state at the 2026-08-18 retirement: the published `trapezoid-push-vc-x1` bookmark holds a
  stale opening commit forked off `0.78.2`, with `support-trapezoid-commits` its support line.
  Rebase or restart is decided at pickup.
- At its merge: reconcile with the 0.78.3 single-name convention (chores-16). The branch manifest
  still says package `vc-x1-dev`, which under the convention is a legitimate dev name for its rungs,
  and the merge commit's manifest says `vc-x1`. custom.md's resolution keeps the branch's filled
  copy, with the version-bump line's `cargo update -p` phrased against the manifest's current name,
  and gains the open/close rename step beside the version bump (custom.md on `main` is the bare
  skeleton, so neither has a home until that merge).

### The validate family: umbrella, runner, and `validate-work`

(wink + agent, 2026-08-21) The 0.80.0 cycle shipped bare `vc-x1 validate` running the `[validate]`
table, beside `validate-agent`, `validate-desc`, and `validate-todo`, which are at-rest checks of
repo state. Read as a family the bare name looks like their parent and is not: it runs cargo while
its siblings check bookmarks and records. Supersedes "A committed cycle-check runner" (resolved by
`vc-x1 validate`, whose "not a vc-x1 subcommand" line was decided the other way at that cycle: the
commands live in config, so the tool assumes nothing about the medium) and absorbs the backlog's
"Add `validate-repo` subcommand", retired into this entry 2026-08-28, whose "runs all" is this
umbrella under a name that no longer fits the family.
- rename the runner to `validate-artifact` (`--fast` kept), `validate` rejecting the old meaning the
  way `bot-session` does, and the per-rung flow saying `validate-artifact` per rung and plain
  `validate` at close-out
- with the rename, tighten jj.md's Land step 4: it names the act ("promote the artifact from
  `main`") but no command, so a raw `cargo install` was improvised there (2026-08-31). The step
  should name the runner, whose full table ends in the install, so the config stays the single
  source of the commands and the landed tree is re-proved at the commit being promoted. A
  set-level edit, so it rides the next agent-files proposal
- add `validate-work`: the work side at rest, the cycle bookmark tracked and at origin,
  `validate-config` clean, mostly the push preflight exposed read-only
- landed early, 2026-08-28: `validate-config` is out of `config --validate` and into this family,
  with the old flag rejected by name. What is left here is the runner rename and the umbrella
- bare `validate` runs everything that applies to the workspace (artifact, work, agent, desc, todo),
  each reported by name, exit status the worst of them, a side the workspace lacks skipped by name
- the `[validate]` config key stays as it is: it is the artifact's validation, and the umbrella
  reads it
- implementation, carried from the retired entry: promote `verify_state_sanity` /
  `verify_completion_sanity` from `push.rs` to `common.rs`, which is the surface `validate-work`
  reads the push preflight through, and the sketch is [the validate-repo
  design](notes/chores/chores-06.md#vc-x1-validate-repo-command-design)
- two of that entry's items did not survive it: the chores-to-commit consistency check, whose
  `Commits:` lines retired with the chores record form, and the exit code as a count of failed
  checks, which the umbrella's "the worst of them" above replaces

### `squash-push --title` / `--body`

`squash-push` amends content only: it folds the working copy into the last commit and force-updates
the remote, but the commit keeps its existing message. Fixing a published commit's *message* is
therefore two steps (`jj describe -r @-`, then `squash-push`). Accepting `--title` / `--body` makes
it one.
- No new risk: squash-push already rewrites a published commit and force-updates the remote. This
  only changes which part of the commit it edits.
- **ochid handling: tell, don't force.** A user-supplied body drops the `ochid:` trailer unless it
  repeats it, which silently breaks the cross-repo link. vc-x1 should *not* inject the trailer
  (unlike `push`, which authors the message and stamps it, but here the user authors it and the tool
  shouldn't rewrite their text). It should error when the new message loses a trailer the commit
  had, naming what would be lost, with an explicit override flag for the case where dropping it is
  intended.
- The content-side guard is the precedent: squash-push already refuses a squash that would drop
  source-only trailers (the 0.65.1 ochid-loss incident). Same check, new input.
- **The guard has a hole the flags would close.** Today the two-step workaround routes around the
  very check that protects the trailer: `squash-push` guards the squash path, `jj describe` guards
  nothing, so the workaround is strictly less safe than the feature. Hit at the 0.77.2 amend
  (2026-07-29), where fixing that commit's own close-out bookkeeping meant editing content *and*
  message, and the trailer survived only by hand-copying it. `vc-x1 fix-desc` can repair a dropped
  ochid by title match, so the failure is recoverable, not silent-forever.
- Amending a just-pushed commit is a real workflow, not a rare one: the cycle-record cites no SHA by
  design, so a rewrite costs nothing. Message fixes naturally cluster there, which is exactly where
  the two-step shape bites.

### sync follow-up: extract `move-bookmark` command

The "put the bookmark / `@` where it belongs" step at the end of sync (reposition logic) is useful
standalone (e.g. the t1B scenario where `main` is right but `@` isn't on it) and deserves an
honestly-named command instead of a mode.
- `vc-x1 move-bookmark` (name open): no fetch, and move `@` (and optionally the bookmark) onto a
  target under the same safety rules as sync's reposition step.
- Sync's final step becomes a call to the same logic.
- Follow-up to the 0.67.0 single-mode sync cycle.

### sync follow-up: retire `--check`, revisit push's auto-rollback

The first half of this entry (push shelling out to `vc-x1 sync --check`, which was racy and not
actually read-only) is done: 0.77.0-3 deleted preflight outright, taking the shell-out and its PATH
dependency with it. What survives:
- Remove sync's deprecated hidden `--check` alias. Nothing invokes it now except
  `tests/cli_sync.rs`'s alias test, so this became actionable the moment preflight went.
- Push's commit-stage rollback auto-runs `jj op restore`, which hides the evidence of what failed.
  This cycle deliberately kept it, since an in-process snapshot taken moments earlier is knowledge,
  not a guess, and both index-lock failures during 0.77.0 cost nothing because of it. Revisit only
  with a concrete case where the hidden evidence mattered.

### vc-x1 push: record uncovered work commits (N:1 work<->agent)

Today push assumes 1:1 symmetric WC commits with shared title/body. The interop / adoption scenario
breaks that: the work side is worked single-repo style (commit + `jj git push` / `git push`, no
`vc-x1 push` in the loop), so no agent pairings exist. One agent commit then records every work
commit not yet covered by a prior `ochid:`, via a multi-line `ochid:` per the design in [[50]].
- Out of scope: the trapezoid close-out, handled natively by the in-progress "feat: push merge
  close-out (trapezoid)" cycle, whose N-ochid stamping also covers a cycle held local and published
  all at once. This Todo is only the no-agent-pairings interop case, and the stamping step's
  multi-line `ochid:` emit is shared groundwork.
- Teach push to:
  - detect the shape (work WC empty, uncovered commits at the bookmark)
  - skip `commit-work`
  - compose a `.claude`-specific message
  - emit one `ochid:` line per uncovered commit
- Open: computing "uncovered", likely a revset from the work bookmark back to the newest commit
  referenced by the agent journal's ochids.

### Run validate-agent at every vc-x1 invocation (config-gated)

The check is one jj spawn (`jj bookmark list main --all-remotes`), cheap enough to run at every
execution, noted 2026-07-15 as a "could, not should". Design points:
- locate the agent-repo (`<cwd>/.claude` or config, which shares the lookup with the refactor
  program's [facade-owns-topology stage](notes/refactor-20260716.md#stage-facade-owns-topology)) and
  silently skip when absent
- severity knob in `.vc-config.md` (`warn|error|off`): unrelated commands (`desc`) warn at most,
  while push / squash-push / validate-agent already have their own handling from 0.69.0-3

### CLI reference lives in `--help`, and README owns concepts

Each command is described in three places (clap's `long_about`, a README section with a flag table,
and sometimes AGENTS.md) and only the flag *descriptions* self-sync, because those come from the
field doc comments. Every hand-written block drifts silently: 0.69.0-4 found the init section
documenting retired `--owner` / `--dir` / `--repo-local`, and 0.77.0-3 found push's `long_about`
still advertising a state machine that had just been deleted. The fix is removing the duplication,
not auditing it on a schedule.
- `--help` becomes the reference: what a command does, its stages, its flags, its invariants. It
  ships with the binary, so it always matches the binary being run.
- README keeps workflows and concepts (the dual-repo model, the cycle, testing recipes, worked
  examples) and points at `--help` instead of restating flag tables. Delete the tables. That is the
  drift source. The `## Usage` block is the same species: its trailing `#` comments have drifted
  into three columns (40, 43, 44) as commands were added, because the alignment is hand-maintained
  and invisible. Left unaligned at 0.77.2 deliberately, since this entry deletes the block.
- Clap reflows prose and collapses bullets unless a field carries `verbatim_doc_comment`, so help
  owns the reference, not the explanations. `long_about` does preserve explicit newlines (0.77.0-3's
  push stage list renders as an aligned two-column list).
- Optional enforcement, cheapest first:
  - assert README has no flag-table rows
  - snapshot-test `--help` output so unintended changes surface in review
  - generate the reference from clap and assert the committed file matches. The third rhymes with
    "config: extract flag-backed key descriptions from Clap", the same single-sourcing shape.
- Sweep each section against `vc-x1 <cmd> -h`.
- Consider regenerating transcripts via support scripts (the gen-exmpl pattern) so examples stay
  reproducible.

### Stale `/.vc-x1` gitignore line: report it, maybe revert

The 0.78.3 residue. Existing workspaces keep their `/.vc-x1` `.gitignore` line: never edit the
user's file automatically. Report that the line is no longer needed and leave the removal to them
(which surface runs the check is TBD, and `config --validate` and the proposed `validate-work` are
the candidates). Separately, any `revert` reintroduction first needs the op-log-derived design:
identifiable sync operations, target the parent of the run's earliest op, preview and confirm,
refuse on intervening non-sync operations. Background in
[chores-16](notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert).

### `vc-x1 validate --full`: accept the default by name

(wink, 2026-08-21) `full` is the `[validate]` table `vc-x1 validate` runs and `--fast` names the
other, so `--full` should be accepted too, unnecessary but allowed, so a reader of a command sees
which table ran.

### `vc-x1 closed "<title>"`: print a landed cycle's block

(2026-08-27) A landed cycle's record is its `## Closed` block in the landmark commit's `TODO.md`,
and reading it back is two awkward commands (`git log --first-parent main --grep`, then `git show
<sha>:TODO.md`). One verb that finds the landmark by title and prints the block.

### docs: no single-owner assumption in the agent-files

(wink, 2026-08-27) The rules assume the user owns the trunk: Land fast-forwards `main` and deletes
the bookmark, and "`main` advances only when the finished cycle lands on it" states it as a rule.
Everything else, the per-rung flow, approval per push, the cycle-record, the bookmark per cycle,
holds whether the bookmark ends in a local merge or a review request to another owner. Rewrite Land
as "hand the bookmark to the trunk's owner" with two endings: the owner is the user (today's
sequence) or someone else (push the bookmark, open the review request, close the cycle as a `##
Waiting` entry on the merge, delete the bookmark once merged, the long-lived case in jj.md). The
close-out shape then follows the owner's merge policy. Also drop any wording that assumes a single
user, so the agent-files fit anyone's repo as well as wink's. With it, rename the `[family]` config
table and its `family.member` key (2026-08-27: the words left the agent-files' prose for set /
adopter / maintainer, and the key is the last holder), a schema change with the usual fix-it
rejection of the old spelling.

### Probe a full re-describe of a landed ladder, or retire the question

(wink, 2026-09-03) Deferred from the **docs: check the transcript join on two landed trapezoids**
cycle, where it was a rung and was dropped at the amend probe's review. The case is a re-describe
of every rung of a pushed ladder, titles and bodies only, with the bookmark renamed and the line
force-published: the heaviest rewrite the work-to-transcript lookup can meet.

- Its four predictions were written before the deferral: every `ochid:` trailer still resolves,
  since change ids do not move, every work committer time moves to the redo so the push-call key
  fails for every rung, every rung's title diverges from its partner's, since agent-repo commits
  on `main` are not rewritten, and the transcript writes stay in their old time-windows.
- Rank it low or retire it: the proposal cycle's opening was re-described after its push and
  evidences all four at one rung, and the amend probe evidences the committer-time half again. The
  open question is only whether eight rungs behave as one did.
- If it runs, it wants a cycle whose ladder is expendable, not a cycle about to land. Re-describing
  pushed commits, renaming a bookmark, and force-publishing just before Land was judged the wrong
  risk for a confirmation.

### push checks the description's widths

(wink + agent, 2026-10-19) `vc-x1 push` takes `--title` and `--body` as given, so a description
that skipped the description review ships whatever it holds. **chore: next time starts with the
settings.local.json Todo** (`qwsqmksw`, on `main` through `vprsqqml`) carries an unwrapped body,
each paragraph one line, and a 57-character title, past the 50-character cap. It is published, an
agent-repo trailer names it, and it stays as it is.

- The proposal: `push` refuses a title over the cap and a body line over the body width, both
  from [Line widths](agent-data/prose.md#line-widths), naming the line and its width.
- A long line that reads better long, an URL or a literal report row, needs a way through, a flag
  or an allowance for lines without spaces.
- The widths live in prose.md, their one home, so the check reads them from there or the two
  copies must be kept in step.

### vc-x1 runs on Windows and macOS

(wink + agent, 2026-10-22, macOS and the hooks added 2026-10-27) vc-x1 is built and run on Linux,
and wink recalls no run on macOS (2026-10-27), so it is untested there. Weighed at the review of
the rung **feat: vc-x1 install-dev puts the dev build on the PATH**, in the cycle **feat: vc-x1
install stages, deploys, and runs builds**, when the install lists were written with `cp` and
`mv`. On Windows the binary is `vc-x1.exe`, run as `vc-x1`, and what follows from that is
unwritten.

- macOS: the build, `vc-x1 validate`, and a cycle run there first, and what fails is named here.
  We think the Unix code paths carry over, `symlink` and the install lists included.
- The hooks, landed by **feat: a hook enforces the shell rules**, were written and tested on Linux
  alone, and the set carries them to every adopter:
  - `inhibit-sleep.sh` needs `systemd-inhibit` and `setsid`, so on macOS, and on Windows outside
    WSL, it fails at every prompt, turn end, and subagent. A hook exiting other than 0 or 2 is a
    non-blocking error, so nothing is stopped, but `hooks.sh install` registers it
    unconditionally. Register it only where `systemd-inhibit` exists, or ship a `caffeinate -i`
    form for macOS.
  - `bash-guard.sh`, `bash-allow.sh`, and `hooks.sh` need `bash` and `jq`. We think they run on
    macOS's bash 3.2 and BSD grep, and `jq` may be missing there. `hooks.sh` checks for `jq` first
    and says so plainly.
  - Native Windows: which shell Claude Code runs a hook's command with is unchecked. WSL behaves
    as Linux.
- The binaries: cargo leaves `target/release/vc-x1.exe`, so `[build] release-bin` and `dev-bin`
  name the `.exe`, or vc-x1 adds `std::env::consts::EXE_SUFFIX` itself, a decision.
- The `-dev` name goes before the extension, `vc-x1-dev.exe`. `dest_name` in
  `src/install_cmd.rs` appends it to the whole file name, `vc-x1.exe-dev`, which Windows will not
  run.
- The built-in copy: a running `.exe` cannot be replaced in place, even by a rename over it, so
  the copy renames the running one aside and moves the new one in.
- The install lists: `cp` and `mv` exist on Windows only where something installed them, Git for
  Windows or MSYS2, and `copy` and `move` are `cmd.exe` builtins, not programs a list can run. A
  portable form, a `{cp}` placeholder, would be additive, since an unknown placeholder is an error
  today, and it resolves to a real program, perhaps vc-x1's own copy.
- The code: `symlink` is `#[cfg(unix)]`, and `install_cmd`'s tests use `std::os::unix`
  unconditionally, so the crate's tests do not build on Windows.

### vc-x1 config and init write a simple config

(wink + agent, 2026-10-22) `vc-x1 config work` names the workspace's file and then prints the
schema, every key commented at its default or example, so it never shows what the file says, and
its form is the old TOML carrier's, `#` lines under bare `[table]` headers, which a `.vc-config.md`
cannot take without a fence around it. Each key is six lines, its doc, `used by`, `default`, its
reference url, the line itself, and a blank, 187 lines at 0.85.7-6, most of them `default:
(none)` and urls. `vc-x1 init` writes `[repos]` and `[remote]` and only the keys with a default,
commented, so a new workspace has no `[validate]`, `[build]`, or `[install]` to start from.
Found at the review of the rung **feat: vc-x1 install-dev puts the dev build on the PATH**, in
the cycle **feat: vc-x1 install stages, deploys, and runs builds**.

- The simple form is iiac-perf's `.vc-config.md`: a short intro, then per table one line of prose
  saying what it is for and a `toml` fence holding its values, no per-key bullets, defaults lines,
  or urls. The documentation stays in `vc-config.md`, where the schema keeps it.
- `vc-x1 config work` prints the workspace's config in effect in that form: the tables and the
  values the file sets, a key at its default marked so, and a key a command needs but the file
  lacks named with the command, `build.release` needed by `build`.
- `vc-x1 config --model` prints every table in the same form with its default or example, for
  copying from. `vc-config-model.md` is the same text, so its generator, `#[cfg(test)] mod model`
  in `src/config_md.rs` today, ships with the binary.
- `--toml` prints either in pure TOML, for a `.vc-config.toml`: the same tables and values, each
  table's line of prose as a `#` comment above it.
- `vc-x1 init` writes the simple form, from the same renderer, with `[repos]` and `[remote]` active
  and the tables it cannot fill, `[validate]`, `[build]`, and `[install]` among them, shown with
  their examples to edit. Whether those are commented or left out with a pointer to `config
  --model` is a decision, since a commented table in a fence reads as configuration.
- The current output, the six lines per key, goes. `config --validate` is unchanged.

### Cycles in a shared repo, by fork and pull request

(wink + agent, 2026-10-26) The cycle assumes its author lands on the trunk and is the one writer
of the files every cycle edits. In a shared repo, work is pushed to a fork and reaches upstream's
`main` by a pull request a maintainer merges, so neither holds. Raised while opening **feat: vc-x1
land**.

- Land splits in two: the handoff, the bookmark pushed to the fork and the pull request opened,
  and the cleanup after the merge, `main` synced from upstream, the bookmark deleted locally and on
  the fork, and the install.
- The project allows merge commits only. GitHub's merge commit is the trapezoid, upstream `main`
  its first parent and the ladder its second, the commits untouched so every `ochid:` trailer
  holds, and it may serve as the closing. Squash merging discards the change ids, which the rules
  forbid, and we think rebase merging drops jj's change-id header, leaving the trailers dangling.
- The trunk takes a remote as well as a name, a key such as `[cycle] trunk-remote = "upstream"`,
  read by sync and by `land`'s at-origin check, since `main` follows upstream and the work goes to
  `origin`.
- The files every cycle edits, the version in `Cargo.toml`, `TODO.md`, and `zTOC.md`, conflict
  between any two pull requests. The version is set at release or merge, the cycle's record lives in the pull request or the contributor's agent-repo, and `TODO.md` is
  the maintainer's, changed by proposals.
- The agent-repo stays each contributor's own, never in the pull request, so a trailer in upstream
  may point where a reader cannot follow, and the pull request's record stands alone.
- lookup's half of this is in `## Waiting`: **lookup matches a line to its write by content**.

### feat: vc-x1 repairs and reshapes without jj

(wink + agent, 2026-10-23) Cut from **feat: vc-x1 closing and land** at its closing rung's review,
the ladder having grown to sixteen rungs: that cycle keeps the path from opening to Land with no
`jj` typed, and the repairs and alternative paths it planned are this cycle, each a step the
agent-files still do with raw `jj`. Each was a rung there, named and with its intent, and its
deliberation moved here whole.

- **feat: commit --closing takes a shape, from the flag or the config**: Taken by **vc-x1's cycle
  flow is in good shape**, `--trapezoid <T>` or linear without it. A project default and converting
  a draft closing between the shapes remain here.
- **feat: the closing can be taken back**: A closing made without `--land` and found wrong is undone
  by hand in two repos, so one invocation takes it back, the work side by its operation and the
  agent side's content moved back into `@`.
- **feat: vc-x1 reshapes a draft bookmark**: Reshaping a cycle's bookmark while it is a draft is an
  amend and a force-push typed by hand, so `vc-x1` does both, refusing a bookmark already landed or
  one another branch is stacked on.
- **feat: vc-x1 reparent edits a commit's parents**: A commit's parents are edited by `merge`, which
  only adds, and by a raw `jj rebase -r` to remove or replace one, so `vc-x1 reparent --target
  <REV>` takes `--add`, `--remove`, or `--set`, on any committed node, its description, change id,
  and trailer kept.
- **feat: vc-x1 re-describes a pair and keeps its trailers**: A coordinated re-describe, as a
  single-step cycle turning multi-step needs, is a `jj describe` in each repo that must keep the
  `ochid:` trailer, so `vc-x1` re-describes both commits of a pair at once and keeps their trailers.
- **feat: vc-x1 runs a local ladder**: A local ladder is `jj commit` for each intermediate and a
  hand collapse into the rung, so `vc-x1` makes the intermediates and collapses them, validating
  each with `validate --fast`.
- **feat: vc-x1 sets a rung's edits aside**: A rung inserted while the working copy holds the next
  rung's edits sets them aside as a patch by hand and restores them after its push, so `vc-x1` sets
  them aside and brings them back.

#### Deliberation carried over

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
    operation the cycle removes.
  - The v0.3.1 proposal said keep separate goes, and its section **The closing is a trapezoid or
    linear, and converts** now keeps it as the linear shape and names the flag and the key, edited
    at the closing rung's review when this rung was added, so the proposal leads the flag by one
    rung.
  - Its own rung after the closing's shapes, since `--land` lands both.
- `vc-x1 reparent`, wink's call at the closing rung's review (2026-10-23): parent fix-ups get their
  own command, since `merge` is the wrong verb for removing one.
  - `--target <REV>`, any committed node, with `--add`, `--remove`, or `--set`, one jj-lib rebase
    of it onto its new parent list, its descendants rebased, the list printed before and after,
    and a failure restored from a snapshot. `merge`'s guards carry over: no cycle, no parent at or
    below the target, `--ignore-immutable` for a jj-immutable one.
  - `--remove` changes the tree: the target keeps its own diff, and what the removed parent brought
    is gone from it, so it prints what was dropped. The last parent is never removed, `--set`
    being the way to replace it, and a conflicted result is reported and left.
  - `merge` stays as the short form of `--add`, its tests and `commit --closing`'s use of its
    primitive unchanged, and may retire into `reparent` later.
- The closing's undo, the Todo's lean, for a closing made without `--land`, a flag
  or a sibling command settled at its rung. A Land's undo, after its push, waits for a real mistake.

### A tool rewrites a repo's history to the current trailers

(wink + agent, 2026-11-01) Every commit on `main` in both repos could be rewritten to the trailer
protocol of the day, from **feat: improve trailers**, and a renamed repo's stale URLs corrected
the same way. Whether to run it on `main` is decided after a trial on a clone.

- It works because a rebase or re-describe keeps a change id and changes a SHA, so `ochid:` holds and
  `ocid:` and `pcid:` are recomputed. The agent repo goes first, since its commits name work commits by
  change id alone, then the work repo oldest first, with the new agent SHAs and parents' SHAs.
- One jj-lib transaction, since a commit at a time rebases every descendant each time. Trees checked
  unchanged, a SHA map written, a dry run, and run from outside any session, the agent-repo never being
  rewound while one writes to it.
- To try on a clone first: whether a fresh clone keeps the change ids of old commits, which we think
  depends on a stored change-id header.
- What it costs on `main`: a force-push of both repos, every clone and fork broken, and every old SHA
  citation stale. Bodies can be remapped, and `notes/`, the closed records, and the transcripts cannot without
  changing trees, `notes/chores/` being frozen. It also goes against the rule that published trailers
  are never rewritten, so a run needs that rule changed.
- Committer dates are kept, since `lookup` finds an untrailered partner by them, and signatures are
  lost.
- A `lost`, `none`, or unresolvable trailer gets no `ocid:` and is left as it is, with no guessing:
  `fix-desc` proposes one agent commit for all of `0.62.0-*`'s 8-character ids, which looks wrong.
- The engine is the one a cycle's trailers are updated by, the range being what differs.

## Ideas

Items not yet solid enough for `## Todo` (or surfaced during close-out / end-of-day before they are
fully formed). Triaged at the next opening: promote to `## Todo` / `notes/todo-backlog.md`, fold
into a picked-up cycle, or drop.

### Several agents work in one workspace

(wink, 2026-10-12) More than one agent session working together in one workspace. Nothing refuses
it since the session lock, `.agent-running`, was turned off 2026-10-13, and nothing makes it work
either. It could be useful where the sessions work on separate projects that share a workspace.

- A first try, the same day, two sessions in a scratch workspace with no lock: nothing either
  wrote was lost, and the record came out wrong, titles, trailers, and one failed push
  ([session-lock.md](notes/session-lock.md)).
- The sessions share one `@` and a push takes the working copy whole, so one session's edits
  land in the other's commit. We think it wants a jj workspace for each session, or a push that
  takes the paths it is given.
- Until then one session per workspace is the user's to keep, and a second agent gets a second
  clone.

### Tool-results land in the agent-repo, tmp/ for the non-durable

(wink, 2026-08-31) The harness persists oversized command output to
`<session>/tool-results/*.txt`, which resolves through the projects symlink into
`./.agent-session`, so those blobs are committed and pushed with the session record at the next
squash-push (session `07191fe5`'s is already tracked). The agent can use, or be directed to use,
repo-local `tmp/` (gitignored) by redirecting chatty commands (`cmd > tmp/<name>.log 2>&1`) when
the output does not need to be durable: both parties inspect the same file and the agent-repo's
history stays lean. Open choices: pin the redirect practice as a `custom.md` line, and whether
the agent-repo should gitignore `*/tool-results/` at the cost of transcript references dangling.

### `vc` as a code+conversation provenance tool (grander ambition)

Today `vc-x1` manages a dual repo (code + `.claude`) cross-linked by `ochid:`. The larger aim is to
*surface* that link: view history with the conversation and the code side by side, giving
provenance, the *why* of a change, not just the *what*. The dual-repo + `ochid` design is already
the substrate, and the cross-links make code<->conversation navigable, so the viewer is UI over an
already-solved data link.
- Build direction: keep resolution/assembly in `vc`, an editor-agnostic Rust engine/lib extending
  the `show` / `chid` / `desc` family ("given a commit, resolve its ochid and assemble the paired
  diff + conversation slice"), and the editor add-on is a thin presentation layer over it.
- Front-end leans a Zed add-on (Rust, preferred), maybe VSCode / other. Verify Zed's extension API
  can host a rich side-by-side panel before committing, and an editor-agnostic core hedges the bet.
- `vc-x2`? A rewrite is unwarranted: the audit's Commonality pass found the architecture sound (por
  is bolted on where an existing good pattern wasn't applied), so equalize incrementally. "vc-x2"
  only makes sense if the viewer changes the *core* architecture (an index / daemon / data model).
  Separate engine-rewrite (no) from product-reposition (open).
- Possible artifact: a top-level `notes/design-cli/vision.md` framing the direction, with the parity
  and conversion docs as sub-designs.

### Restructure the design-cli parity docs (target 0.63.0)

`por-dual-parity-audit.md` (~1200 lines) fuses a *frozen* audit (the `## 1`-`## 8` snapshot
evidence) with a *living* design (axes, decisions, matrix, gap list). The "audit" name undersells it
and the halves have different lifecycles. And `por-dual-parity.md` (the stub) overlaps on parity but
uniquely holds the `por <-> dual` conversion design.
- Split the audit doc into a frozen audit snapshot + a living design doc (names TBD, and could
  reclaim `por-dual-parity.md` for the design).
- Refocus the stub to conversion-only and rename (e.g. `por-dual-conversion.md`), and drop its
  redundant parity half.
- Repoint refs (`todo.md` `[1]` + the `por -> dual` Todo, `copying.md`, the audit's internal anchors
  + Reading guide) and validate. `chores-10/11/12` mentions are historical and stay.
- Promote the Gap-list items to anchored `#### Gap N: <title>` sub-headings so cross-cycle citations
  can deep-link a specific gap (markdown anchors headings, not list items). Trade-off: stable
  anchors, but the ordinal lives in the heading text (manual renumber on reorder), fine for a
  consumed backlog. The 3 `Gap #N` links in the `0.62.0` close-out chores narrative resolve only to
  the section until this lands.
- Deferred from the 0.62.0 close-out: close-out is bookkeeping-only, and the split is substantive,
  anchor-heavy work warranting its own cycle.

### Chores retire into a session index (post-viewer)

Once the provenance viewer ("`vc` as a code+conversation provenance tool" above) can present a
commit's session and code side by side, the hand-written chores narrative is a distillation of a
conversation the agent-repo already records verbatim, so the DRY argument that removed edit lists
from chores (git owns the mechanics) then applies to the narrative too (the session owns it). Chores
collapses to an index into the session.
- The `ochid:` trailer links a work commit to a session *commit*, and the index adds within-session
  granularity: which conversation span produced the commit, where the design argument happened. We
  think it can be generated (the transcript records when pushes happen), making it drift-proof where
  hand-written chores never were.
- What survives: the curated design layer (the refactor-20260716.md pattern). Sessions are an
  immutable journal, good as record and poor to cite into, so live design references keep pointing
  at curated docs, not per-cycle narrative sections.
- The template side already points this way: chores files are not seeded, and a new project's
  history is its own commits + agent session from day one.

### A workspace anchor, so `[repos]` is shareable

(wink, 2026-09-04) `[repos]` values resolve against the config file's own directory, which is why
the two sides' blocks must differ in every entry, and why a `.vc-config.md` copied between repos
carries values that are wrong in its new home. Define `workspace` as the offset from a config to
the workspace root and anchor `[repos] work` and `[repos] agent` to that instead, so the table is
shareable and only the one `workspace` line is per-clone. It also collapses two rules into one:
root-finding (the nearest config's `[repos] work`) and side detection (the entry resolving to its
own directory) are separate traversals over the same three values today, and four defects in
**agent-files(proposal): v0.2.2** came from prose describing that pair and confusing it.

- `workspace` must be readable with no other knowledge, since it is what tells a reader which side
  it is on, so it cannot depend on side detection. We think that means it replaces `[repos] work`
  rather than sitting beside it, keeping both being a way to keep both rules under new names.
- Ship it with a `[repos] work` fallback as a migration window, converting all three family repos
  in the same cycle, and delete the fallback in a second one. A fallback left standing makes three
  rules where there were two, which is the surface the change exists to shrink.
- The gain is not identical configs: a file has to know where it sits, so one line always differs.
  The gain is that it is one named line and the rest of the table is shared.
- Related: [Work commits link by URL](#work-commits-link-by-url). An anchored `[repos]` would let a
  trailer prefix be derived rather than the constant it is now.

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)).

# References

[50]: /notes/forks-multi-user.md
