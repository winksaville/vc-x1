# Cycle protocol

This protocol uses [Prose form](prose.md#prose-form). It
contains instructions on how a commit cycle is accomplished.

Universal file, pinned to the template repository; do not edit
here. Project overrides go in [custom.md](../custom.md).

The artifact a cycle produces is whatever the bot generates from
the conversation: code, prose, an image, a song, a screenplay.
The steps below use a Rust crate as the running example (the
cargo cycle, `Cargo.toml` versioning); substitute your medium's
equivalents. This project's manifest is recorded in
[versioning.md](versioning.md).

## Cycles

A cycle has three phases:

- **[Preparation](#preparation)** (`X.Y.Z-0`): the cycle's
  first commit, when it needs setup (a lightweight cycle omits
  it and starts at `-1`; see
  [versioning.md](versioning.md#step-numbering)). Sets up the
  cycle:
  - Bump the version-of-record to `X.Y.Z-0` (where it lives
    and the suffix scheme are project-specific; see
    [versioning.md](versioning.md)).
  - Pick up a `## Todo` item (typically the top-ranked,
    #1) into `## In Progress` (bold title + succinct problem
    statement + plan ladder).
  - Open the [chores section](#chores-sections).
- **[Work-N](#work-n)** (`X.Y.Z-1`, `X.Y.Z-2`, ...): the
  commits that implement the change. As many as the change
  needs; each runs through the
  [per-commit flow](#per-commit-flow).
- **[Close-out](#close-out)** (bare `X.Y.Z`): the cycle's
  last commit. Bookkeeping only:
  - Move the picked-up item to a one-line entry in
    `## Done`.
  - Move the `## In Progress` block into the
    [chores section](#chores-sections).
  - Optionally update `notes/README.md` if functionality
    changed.

A cycle's commits are published to the project remote
either incrementally or as one batch at close-out; the
result must always be published at close-out. See
[Pushing](#pushing).

**Sub-cycles.** When a Work commit's scope grows enough to
warrant its own ladder, it subdivides: `X.Y.Z-3.0`
Preparation, `X.Y.Z-3.1` / `X.Y.Z-3.2` Work, `X.Y.Z-3`
Close-out. The same three-phase shape applies recursively
at every depth. See [Numbering](#numbering) for the
suffix rule and [Sub-cycle ladders](#sub-cycle-ladders)
for the local-ladder mechanics.

## Chores sections

A **chores section** is a `##` section in `notes/chores/chores-NN.md` recording landed work.
In general, every commit that lands on the permanent branch should have a rung in some
section's as-built ladder in a chores file.

The phrase **"Open" the chores section** means append a `##` header to the current
`notes/chores/chores-NN.md` with the title it records (e.g. `## refactor: foo bar`), add the
file's `## Table of Contents` entry, and start the **as-built ladder**: one
`- [[N]] X.Y.Z[-n] <title>` rung per commit the section records, each opening with the literal
`[[N]]` placeholder. **Each work commit then appends its own rung + narrative note as it
lands**, so the chores record is built up per commit, not held back and written all at once at
close-out; close-out only *finalizes* (title sync, design subsections, retiring the
`## In Progress` block). Rung placeholders are backfilled later, once their commits are
permanent (see [Commits backfill](#commits-backfill) below).

Fuller chores conventions (content rules, header sync, design subsection pattern, ladder and
reference formatting, the Table of Contents) live in
[Chores conventions](notes.md#chores-conventions).

### Commits backfill

An as-built ladder rung cites its commit by SHA, but a SHA isn't stable until the commit lands
on a **permanent branch** (`main`, or a long-lived release/patch branch that won't be
rewritten); a rebase or squash rewrites it on the way. So:

- A rung is **written with the literal `[[N]]` placeholder**.
- **Backfill once the commit is on a permanent branch**, where its SHA is final. A commit
  can't record its own SHA (that would change the hash), so the fill always lands one push
  later: **each push backfills the rungs of the commits the previous push made permanent.** On
  a topic branch the sections instead wait until the branch lands, so no SHA is ever written
  that a later rebase could invalidate.
- A deliberate rewrite of already-recorded commits (a coordinated re-describe, a retroactive
  reshape) invalidates their recorded SHAs. Re-record them once the rewrite is published, on
  the same one-push-later timing.

Each rung carries its own `[N]` ref, with the commit URL + 40-hex SHA in the file's
`# References` (format in
[Chores commit references](notes.md#chores-commit-references)). A section's `##`
title matches its close-out commit title, and each rung's text its own commit title, so a rare
deliberate rewrite of a permanent-branch commit re-syncs via `git log --grep "<title>"`.
Sections that predate the ladder form keep their grandfathered `Commits:` lines.

The per-push cadence is a project choice, not dogma. A **per-close-out** model (recording a
cycle's SHAs at its close-out) is equally valid. The one invariant: a recorded SHA must be
permanent.

## Preparation

The cycle's first commit (`X.Y.Z-0`), when the cycle needs
setup (a lightweight cycle omits it; see
[versioning.md](versioning.md#step-numbering)):

- **Bump the version-of-record** to `X.Y.Z-0`. Where it
  lives, the suffix scheme, and any derived files (a
  lockfile, a sourced manifest version) are
  project-specific; see [versioning.md](versioning.md).
- **Move a `## Todo` item** (if the cycle has one) into
  `## In Progress` and the todo item should have:
  - A **bold title line**, which will be the chores
    section header, minus the `## ` prefix.
  - A **succinct problem statement**; add if one is needed
  - A **plan ladder**.
- **Open the [chores section](#chores-sections)**: append a
  `##` header with the title it records, the cycle's
  anticipated close-out title.

## Work-N

The cycle's work commits (`X.Y.Z-1`, `X.Y.Z-2`, ...)
implement the change. As many as needed:

- Each commit runs through the
  **[per-commit flow](#per-commit-flow)**.
- **Interim pushes** are optional (backup, progress
  visibility).
- Close-out is the only mandatory push (see
  [Pushing](#pushing)).
- **Subdivide into a sub-cycle** if a Work commit's
  scope grows enough (see
  [Sub-cycle ladders](#sub-cycle-ladders)).

## Close-out

The cycle's last commit (bare `X.Y.Z`) does bookkeeping
only, and the commit body describes that bookkeeping, not
what happens post-squash:

- **Move the picked-up item** from `## In Progress` to a
  one-line entry (with a chores `[N]` ref) in `## Done`.
- **Finalize the chores section** (opened during Preparation
  and grown per commit; see [Chores sections](#chores-sections)):
  - The problem statement is already the chores intro (written
    when the section was opened) and the plan ladder is already
    the `### As-built ladder` (a rung appended per commit), so
    close-out only *finalizes*, it does not cut-and-paste.
  - Sync the chores header to the **final** commit title if the
    cycle's scope shifted; update every anchor back-reference.
  - Add any `### design subsections`; optional `### Outcome`
    notes.
  - Replace the `## In Progress` cycle block with
    `_No cycle currently in progress._`. Under a multi-cycle
    program (program `###`, current stage `####`) this retires
    the stage `####` only; the program heading and its ladder
    stay, the shipped rung flipped `(done)`, so the marker
    sits inside the program block until the next stage is
    picked up.
- **Update `notes/README.md`** if functionality changed
  (new flags, new subcommands, changed behavior).

Whether to **squash** the cycle into one commit before the
publishing push, or push as-is, is decided at push time; see
[Pushing](#pushing).

## Numbering

Each commit's phase is encoded in the version suffix: `-0`
Preparation, `-1`/`-2`/... Work, bare `X.Y.Z` Close-out,
recursively for sub-cycles. The full scheme (disambiguation,
nesting, optional Preparation, the project's version-of-record
format, and the per-phase bump) lives in
[versioning.md](versioning.md#step-numbering), which is the
single source of truth for this repo's versioning.

## Per-commit flow

Every commit (Preparation, each Work commit, Close-out) goes
through:

1. **Mark this commit `(current)`** as the first edit in
   `TODO.md > ## In Progress` (`TODO.md` is at the repo
   root).
2. **Do the work** (see [Iterative work](#iterative-work)
   for the loop-and-squash technique).
3. **Flip this commit `(current)` -> `(done)`** in `## In
   Progress`, before the cargo cycle and the commit.
4. **Validate the artifact**, a medium-specific step, skip-able
   for notes-only commits, mandatory at close-out. For the Rust
   example the cargo cycle is:
   1. `cargo fmt`
   2. `cargo clippy --all-targets -- -D warnings`
   3. `cargo test`
   4. `cargo install --path . --locked`
   5. (re-test if anything substantive changed)
5. **Work review.** Stop *before* writing any description;
   tell the user "ready to commit." The user reviews the
   changes and we iterate until complete.
6. **Write the commit description**; see
   [Commit description](#commit-description).
7. **Commit Description review.** Show the title + body
   and stop. The user reviews the description. Iterate.
8. **Commit + push.** Hand the approved title/body to
   `vc-x1 push <bookmark> --title "..." --body "..."`, whose
   commit stages commit both repos and stamp the `ochid:`
   trailers. Never pre-commit the rung with `jj commit`: an
   empty `@` at push mints a stamped empty duplicate
   (bugs.md #6). Push approval is per-push, so step 7's
   review covers it only when the user's go explicitly
   includes the push.

**Two overrides apply:**

- **Deviation or question.** Any time the work deviates
  from the agreed plan, or a question arises, stop and
  surface it; don't push through.
- **ESC-ESC.** The user can interrupt at any point to pull
  a review or question forward.

## Commit description

[Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <short description>
<type>(scope): <short description>   # optional scope
```

Titles carry **no trailing `(<version>)` suffix**. The
version-of-record (where it lives and its bump cadence, see
[versioning.md](versioning.md)) is useful for confirming
you're running the version you're testing, not a per-commit
title marker. We think per-commit version-in-title is
unstable on large projects: many changes are in flight
simultaneously, and a version is only stable once merged into
the main repo, so titles don't carry one.

### Title

- <=72 chars total.
- Common types: `feat`, `fix`, `refactor`, `test`,
  `docs`, `chore`; optional `(scope)` in parentheses after
  the type, per the spec.
- Favor terse phrasings.
- **Distinct per step.** Each of a cycle's commits gets its
  own descriptive title (no shared cycle title with a step
  marker). Share a greppable stem across the cycle's titles
  (e.g. `ring buffer`) so `git log --grep` collects them; the
  chores section header matches the close-out title. See
  [Conventional-commit shape](prose.md#conventional-commit-shape-ladder--chores--commit).

### Body

[Prose form](prose.md#prose-form) (intro + bullets),
wrap ≤72. Bullet content differs per repo:

- **Work-repo body**: file-by-file. One bullet per file
  changed (file plus a one-line gist), in short sentence
  fragments (full sentences are discouraged). Sub-bullets
  for files with multiple distinct changes:

  ```
  - path/to/file1
    - first distinct change
    - second distinct change, wrapping to next line
       with continuation indented 5
  ```

  The list **opens with the version bump** as its first
  bullet, for the Rust example
  `- Cargo.toml, Cargo.lock: vX.Y.Z-xxxx` (generally:
  the medium's version-of-record files); `-xxxx` stands for
  the whole suffix, nested or not (`v0.78.0-1`,
  `v0.78.0-3.1`), per
  [versioning.md](versioning.md#grammar-and-storage). Titles stay
  version-free (above), so this first bullet is the
  version's durable, visible home, and log viewers (e.g.
  gitk's message pane) show it without opening the commit's
  file list. Adopted 2026-07-24.

  The file-by-file list is the source of truth for the
  cycle's mechanical change record. Chores carries the
  narrative + design, not a copy of it. Promote any
  "why" beyond one sentence to a chores `###` subsection.

- **Bot-repo (`.claude`) body**: bullets describe
  in-session activity rather than work-repo changes.

### Trailer

`ochid:` as the last line of the body; see
[Cross-repo linking (ochid trailers)](jj.md#cross-repo-linking-ochid-trailers)
in agent-data/jj.md for the convention.

For breaking changes, use the hyphenated `BREAKING-CHANGE:`
trailer key. `BREAKING CHANGE:` (with a space) is the only
space-separated key the Conventional Commits spec allows; the
hyphenated form is also valid and avoids the space ambiguity.

## Reviewing changes

Work review looks at the **uncommitted working-copy diff**,
on the way to commit. The user opens diffs in their
editor (Zed, VSCode); jj commands are for terminal:

- `jj diff`: working-copy diff (uncommitted)
- `jj diff -r @-`: diff of the previous commit
- `jj diff --from <X> --to <Y>`: any two revisions
- `jj show -r <X>`: description + diff for one rev

Don't `jj edit -r @-` to view a past commit, because that marks
it mutable and shifts `@`; use `jj diff -r @-` or
`jj show -r @-`.

No preflight while a review iterates: `fmt` / `clippy` / `test`
wait until the review settles, since `fmt` mutates files in ways
that interact badly with the user's mid-review edits. Validation
runs once, on the settled state, per the per-commit checklist.

See [Sub-cycle ladders](#sub-cycle-ladders) for the
close-out squash recipe and recovery; revset primitives
are in [`jj.md > Revsets`](jj.md#revsets).

## Pushing

### Policy

Push is **discretionary** during the cycle (backup,
progress visibility) and **mandatory at close-out**, since
the cycle's result must be published.

**Approval is per-push.** Every push (any repo, any kind:
cycle push, interim backup, recovery/surgery force-push)
happens only after the user has reviewed the changes to be
published and explicitly approved that specific push.
Approval of a plan that *includes* a push does not authorize
the push itself; stop and ask again at the moment of pushing.

**Default is interactive; an explicit scoped delegation waives
the gates.** The gates above (per-push approval, the
commit-description review that shows title+body and stops, and
the hard stop after push/squash-push) are the *interactive
default*.
They yield when the user **explicitly** delegates a complete,
bounded task and authorizes carrying it through ("do all of X
and push each step, don't check in"). The bot then proceeds
through that task's commits and pushes without stopping, and
continues past each push to the next step. Conditions:

- **Explicit grant.** Never inferred from a task merely being
  well-scoped; the user's words must authorize unattended
  completion. "Commit and push" (or "then push") names the
  destination, not a waiver: it authorizes the push *after*
  the normal work review and description review, not skipping
  them. Only wording that explicitly waives the stops ("don't
  check in", "no need to review", "carry it through
  unattended") waives them.
- **Bounded goal.** Covers the named task only; does not carry
  to the next task or a vaguer follow-on.
- **Destructive ops still pause.** Delegation covers the task's
  ordinary commits and pushes; it does *not* pre-authorize a
  genuinely irreversible action (force-push over published
  history, history rewrite, deleting a remote branch). Those can
  permanently destroy work and aren't a normal cycle step, so the
  bot flags one before acting. An ordinary delegated cycle never
  reaches this.
- **Still transparent.** Report each commit/push as it lands
  (title + outcome) so the user can catch up.
- **When in doubt, ask.** Ambiguous authorization falls back to
  per-push approval.

### Shape at close-out push

At close-out the cycle's *work* is done; its *published
shape* is the remaining choice, made at push time. Surface
the options and get user approval before pushing. Once on
the target, changing shape is a remote rewrite (force-push,
needs approval), so choose deliberately.

- **Squash to one commit**: single entry on the target.
  Right for straightforward changes where the Work-N is
  focused on one or two files.
- **Trapezoid** (a `git merge --no-ff` merge commit)
  *(current default)*: the target gains a merge commit
  (`X.Y.Z`) whose first parent is the trunk line and whose
  second parent is the cycle's ladder, so
  `git log --first-parent` reads one commit per cycle while
  every rung stays reachable. See
  [Trapezoid close-out recipe](#trapezoid-close-out-recipe)
  for the full sequence.
- **Keep separate**: one commit per cycle entry on
  `main`. Use when the decomposition itself is
  informative. Each chores section keeps its own header /
  ladder refs; no consolidation churn.

A squash is set up before invoking `vc-x1 push`; a trapezoid
is reshaped between two pushes (the recipe below). For any
other shape, drive `jj git push` directly.

### Trapezoid close-out recipe

A [trapezoid](#shape-at-close-out-push) close-out is
published in four steps: an ordinary close-out push, a
two-command reshape, and a second push that re-points the
bookmark at the reshaped commit.

```
  main line   ...──<base>──────────────────<closeout>──
                      \                    /
  ladder             <rung-1>──...──<tip>─┘
```

- `<base>`: the **parent of the ladder's first rung**, which
  is the trunk position when the cycle opened. It becomes the
  first parent.
- `<tip>`: the cycle's last Work commit. It becomes the
  second parent.
- `<closeout>`: the close-out commit, created by step 1.

The steps. Only step 1 is a `vc-x1 push`; the rest is jj,
because after step 1 the commits already exist and all that
remains is reshaping and publishing them:

1. `vc-x1 push <bookmark> --title "..." --body "..."`
   - the ordinary close-out push. It commits both repos, stamps the
   `ochid:` trailers, and publishes `<closeout>` linearly.
2. `jj rebase -r <closeout> --onto <base> --onto <tip>`
   - `<closeout>` becomes the merge. Parent order is the
   argument order.
3. `jj new <closeout>`
   -  puts an empty `@` above the merge.
4. `jj git push --bookmark <bookmark> -R .`
   - publishes the reshaped commit. The bookmark needs no `jj bookmark set`:
   it follows the rewrite in step 2 automatically. The bot
   repo is untouched, and its session tail goes out with a
   separate `vc-x1 squash-push` afterwards.

**Step 4 is not a `vc-x1 push`** (corrected at the `0.77.0`
close-out, which tried it). Push runs its whole pipeline or
none of it, and the bot repo is never quiet for long: by the
time the reshape is done, `.claude` holds the session writes
from steps 1-3, so `commit-bot` wants to run and the message
stage demands a title for it. The result is a bot-side
requirement blocking a work-side publish that needs nothing
but a moved ref. Publishing an already-made commit is a
different operation from committing and publishing, and only
the latter is push's job.

#### Details

- **Verify two parents before step 4.** `jj log -r <closeout>
  -T 'parents.map(|p| p.change_id().short(8))'` must list
  both. jj preserves the second parent even though `<base>`
  is an ancestor of `<tip>` (observed at `0.74.0`, `0.75.0`,
  `0.76.0`), but a collapsed merge is indistinguishable from
  a correct one in `jj log --no-graph` and is only visible
  once published.
- **`<base>` is not always the previous close-out.** A docs
  or planning interlude between cycles sits on the trunk line
  and must stay there; take the parent of the ladder's first
  rung, not the last close-out.
- **Step 3 is about `@`, not the bookmark.** The bookmark
  follows the rewrite in step 2 on its own. What step 2
  leaves misplaced is the working copy: `jj rebase -r`
  re-parents descendants onto the rebased commit's **old**
  parent, so the empty `@` from step 1 lands beside the merge
  on `<tip>`, and the working tree reverts to pre-close-out
  content, which looks alarming and isn't. `jj new` puts `@`
  back on top of the merge so the tree is right and the next
  commit continues from there. Skipping it doesn't break the
  publish; it leaves you working from the wrong parent.
- **Trailers survive.** The reshape changes `<closeout>`'s
  SHA but not its change ID, so the `ochid:` trailers stamped
  in step 1 stay valid in both directions. This is why the
  reshape is safe after the trailers are written.
- **Step 4 moves the bookmark sideways.** Step 1's SHA
  becomes unreachable; anyone who fetched between the two
  pushes holds a dangling commit. Consequently a
  [Commits backfill](#commits-backfill) must never read a
  SHA from that window; wait until step 4 lands.
- **Immutability.** No flag is needed on a long-lived topic
  bookmark. Only when `<closeout>` is already on `trunk()`
  does the rebase need `--ignore-immutable`, and then the
  push force-updates the target.
- **The bot repo is left for afterwards.** Step 4 touches only
  the work repo, so `.claude` still holds every session write
  from the whole procedure. `vc-x1 squash-push` folds that
  tail into the bot commit, and its change id survives the
  squash, so the work-side `ochid:` keeps resolving.

#### Recovery

- **Nothing is published between steps 2 and 3**, so the local
  reshape is undoable with `jj undo` / `jj op restore`.
- **A collapsed or mis-parented merge** (step 2 verification
  fails): undo and redo step 2 with the corrected revisions.
  Do not push a shape you did not intend; after step 4 the
  remote boundary is crossed and recovery is forward-only.
- **Working copy left beside the merge** (step 3 skipped):
  `jj new <closeout>` after the fact. Nothing published is
  affected (the bookmark was never wrong), but any commit
  made in the meantime branches off `<tip>` and needs a
  rebase onto the merge.
- **A wrong bookmark position**, however it arose:
  `jj bookmark set <bookmark> -r <closeout>` before pushing.
  If step 4 already published it, the fix is a second
  sideways move, not a rewrite.

### vc-x1 push wrapper

`vc-x1 push <bookmark>` wraps per-push mechanics. See
`vc-x1 push --help` for current flags. `<bookmark>` names a
work-repo bookmark only; the bot repo is always pinned to
`main` (see [.claude cadence](#claude-cadence)).

**Current limitation**: only fully supports the
[Keep separate](#shape-at-close-out-push) shape; other
shapes need manual jj steps. Planned improvements are
project state, tracked in the project's `TODO.md`;
this protocol describes only the stable mechanism.

### .claude cadence

**Cadence**: one push = one bot-repo commit, paired
with every work-repo commit in that push.

The `.claude` working copy accumulates session data
across the cycle; its change ID stays stable across
snapshots, `jj describe`, and the squash-push fold, so
work-repo `ochid:` trailers resolve.

`.claude` is a linear journal: all session work lives
on `main`, regardless of the work-repo bookmark. **Do
not create or maintain bot-repo bookmarks that mirror
work-repo branches**, which risks the bot steering session
pushes to the wrong remote ref.

Ending a session: if the user runs `/exit` there will be
session information created, which we don't worry about.
The user can close the terminal instead and `@` will
remain empty.

### Bot communication at the reviews

Use plain prose, no insider jargon ("Gate N signal",
"Checkpoint N", etc.):

- **At Work review**, summarize what changed and stop.
  "Work complete. Please review."
- **At Commit Description review**, present `$TITLE`
  and `$BODY` explicitly; ask permission to commit/push.
  Don't spell out the full `vc-x1 push ... --title ...
  --body ...` invocation by default.
- **At Post close-out review**, surface the shape
  options (squash / merge / keep) and the push target;
  wait for the user's choice before any `jj squash` /
  `jj rebase` / `jj git push` invocation.

### After push or squash-push: stop and wait

After a **push** (crossing the remote boundary, by hand or
via the `vc-x1 push` wrapper, whose last stage publishes
the bot repo too) or a manual **squash-push** on the bot
repo, stop for the turn: no next step, edit, tool call, or
text output until the user directs otherwise. **Even when
the next step seems obvious, wait.**

- **Scope**: the stop follows the user's directive, not the
  push. A standing directive covering more work ("finish
  the remaining ladder commits on your own") makes an
  intermediate push just a step; the hard stop lands on the
  turn's *final* push.
- **Why**: the bot repo is a live journal, so everything after
  the invocation (its own record, closing words) lands in
  `@` as a trailing tail. Between delegated pushes the tail
  rides into the next cycle's bot commit; the final push's
  tail has no next commit, and the bot's own squash-push is
  itself session data (`@` refills immediately), so only the
  user, after the turn, can capture it
  (`vc-x1 squash-push -R .claude`).
- **Silence**: put all closing words *before* the final
  push. The harness rejects an empty turn, so it may force a
  visible token after the tool returns; if so, emit a bare
  acknowledgment only (e.g. "landed"), never a summary,
  verification, or next-step offer. There is no "harmless"
  closing line after the push; that is a known slip.
- **Flush**: when the user wants `@` empty (no tail), they
  run `vc-x1 squash-push -R .claude` after the bot goes
  quiet, which flushes all bot session information into the
  published commit. Repeat if new writes land (see
  [Recovery](#recovery)).

### Recovery

- **If push exits before its last stage** (`push-work`
  succeeded but the bot-repo publish (`squash-push-bot`)
  didn't run), run the squash+push by hand:

  ```
  vc-x1 squash-push -R .claude
  ```

  It runs in-process, so a failure is a visible non-zero
  exit, with no log file to chase.
- **Run squash-push again if `@` is non-empty** after a
  pass (also desirable after extra activity by the bot's
  agents).
  - Why: the bot keeps writing session data while the
    command runs, so the invocation's own record plus any
    closing response land after the squash.
  - Safe to repeat: bot session data is append-only, so a
    re-run never conflicts or overwrites. (This could
    change; it is not under the user's control.)
  - No guarantees: events outside the bot's control can leave
    `@` non-empty; e.g. the bot's back end may decide to
    squash/consolidate session data, which can take minutes
    and land after the pass. The remedy is the same: just
    run squash-push again. This is why a single pass is never
    guaranteed to leave `@` empty.
- **Nothing to clear after an out-of-band recovery.** Push
  keeps no saved state, so whatever you do to the repos by
  hand is simply the state the next run sees.
- **Late work-repo tweak after the work-repo push succeeded**
  (e.g. updating AGENTS.md or memory) requires `jj
  squash --ignore-immutable` and a re-push; that is a
  remote rewrite and needs explicit approval like any
  push.

## Iterative work

When work for a single commit (the **target**) benefits
from incremental review, loop:

1. `jj new -R .`: fresh empty `@` on top of the target.
2. Make the next round of changes.
3. User reviews the round (see
   [Reviewing changes](#reviewing-changes)).
4. `jj squash -R .` folds into the target and creates a
   new empty `@`.
5. If not done, go to step 2.

Same jj mechanics as a
[sub-cycle ladder](#sub-cycle-ladders), but at
single-commit scope, so the version doesn't change.

## Sub-cycle ladders

When a Work commit subdivides into a sub-cycle (see
[Numbering](#numbering) for suffix nesting), its Work
commits will live as a local jj `@` chain and
**collapse into the sub-cycle's Close-out** before the
parent cycle continues. Ladder commits are scratch, for
review and bisection only.

### Per-Work-commit contract within a ladder

For each Work commit in the ladder:

1. `jj new -R .`: create a fresh empty `@`.
2. Do the commit's work.
3. Run the fast validation (Rust example: `cargo test
   --bins`). **Non-negotiable**, because for code, build and
   clippy alone miss regressions until a later commit runs the
   full suite, raising bisection cost.
4. `jj describe -m "..." -m "..." -R .`: working title
   only; the sub-cycle Close-out collects everything
   into one final commit.

**Nothing here is pushed.** The ladder is local until the
[Close-out squash](#close-out-squash-the-ladder), so no ladder
commit ever carries an `ochid:` trailer: the trailer is stamped
once, by `vc-x1 push`, on the squashed commit. Step 4 is
therefore first-time authoring of a scratch description, not a
rewrite of a published or stamped one, and is the named
exception to
[Re-describing](jj.md#re-describing-coordinate-first-and-keep-the-trailer).

### Navigating the ladder

Common moves:

- `jj log -r '<base>::' -R .`: see the whole ladder
  from its base.
- `jj edit -r <prefix> -R .`: jump `@` to any ladder
  commit by chid prefix; useful for bisection.
- `jj edit @-- -R .`: quick-jump back two commits.
- `jj diff -r <chid> -R .`: review one commit in
  isolation.

Modifications to any ladder commit rewrite it in place;
descendants auto-rebase.

### Close-out: squash the ladder

When all ladder Work commits are done and tests pass:

```
jj squash --from "<base>..@-" --into @ -u -R .
```

`<base>` is the parent of the first ladder commit; `-u`
keeps `@`'s description and discards the sources'.
After squash, history is linear: `<base> -> @`;
intermediate commits are auto-abandoned.

Then `vc-x1 push <bookmark>` as for any other commit. This is
the ladder's first and only publish, and where its `ochid:`
trailer is stamped. The scratch descriptions written at step 4
of the
[per-Work-commit contract](#per-work-commit-contract-within-a-ladder)
never left the machine and never carried one, which is why
describing them freely is safe.

For N = 1 the squash is a no-op (`<base>..@-` is empty
when `@-` is `<base>`); push the single commit directly.

### Recovery

If a ladder commit goes wrong, back out without losing
prior commits:

- **Discard the current commit.** `jj abandon @ -R .`
  drops it; you get a fresh empty `@` on the same
  parent.
- **Edit an earlier commit.** `jj edit -r <chid> -R .`,
  make corrections, then `jj edit -r <last-ladder-chid>`
  to return. Descendants auto-rebase.
- **Discard the entire ladder.** `jj op log -R .` shows
  the op history; `jj op restore <op-id> -R .` reverts
  to that point. Full undo: removes *all* ladder work
  after the chosen op. Use only to start over.

# References

- [`jj.md > Revsets`](jj.md#revsets): revset primitives
  (chid/cid, `@`/`@-`/`@+`, `..`/`::` ranges, prefix matching).
- The per-commit `cargo test --bins` gate exists because a
  regression introduced in an early ladder commit can go
  uncaught until a later commit runs the full suite, raising
  bisection cost.
