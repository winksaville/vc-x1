# Chores-15

Continuation of `chores-14.md` (closed at `0.76.0`, the
repo-registry close-out). This file covers `0.76.1` onward —
the remainder of the jj refactor program
([refactor-20260716.md](../refactor-20260716.md)) — worked on
the `refactor-vc-x1` bookmark while `main` parks at the
`0.71.0` tip.

Reference numbering is file-local — see
[`AGENTS.md`](../../AGENTS.md#reference-numbering); chores-15
starts at `[1]`.

## docs: trapezoid close-out recipe

Commits: [[2]]

Publishing `0.76.0` as a trapezoid [[1]] needed four steps that
no single document described. The procedure was spread across
three places — the cycle-protocol recipe, the refactor doc's
trapezoid stage, and a passing mention in a `0.74.0` As-built
rung — and where they overlapped they disagreed. Two of the
three stated mechanisms were wrong, and the one check that
would catch a silent failure appeared nowhere. This interlude
consolidates them into a single definitive procedure before
the `--merge` flag is built from it.

- The four corrections, each a claim that survived because
  nothing was the source of truth:
  - **Base selection.** The recipe glossed the first parent
    as "the previous cycle's close-out (the current `main`
    tip)"; the design doc said "the parent of the ladder's
    first commit". They coincide only when no interlude sits
    between cycles. At `0.76.0` they differed — the base was
    the `0.75.1` docs interlude, not the `0.75.0` close-out —
    and the recipe's gloss would have swallowed the interlude
    into the merge's ladder side.
  - **Why `jj new` is needed.** The recipe attributed it to
    the rebase leaving `@` *on* the merge. It doesn't:
    `jj rebase -r` re-parents descendants onto the rebased
    commit's **old** parent, so `@` lands beside the merge,
    not on it. Same step, different reason — and the wrong
    reason makes a load-bearing step read as cosmetic, since
    `bookmark-set` resolves `@-`.
  - **Immutability.** "Its commits are immutable, the rebase
    needs `--ignore-immutable`" is true only when the
    close-out is already on `trunk()`. A long-lived topic
    bookmark isn't, so the reshape needs no flag.
  - **Ordering.** The design doc specifies the merge in-flight
    (after `commit-bot`, before `bookmark-set`); the four-step
    manual procedure reshapes after a completed push. Both are
    legal, they have different consequences, and nothing said
    so.
- The missing check: jj does **not** collapse the second
  parent when the base is an ancestor of the tip, but nothing
  said to verify it. A simplified merge is indistinguishable
  from a correct one in `jj log --no-graph`, and the mistake
  is only visible once published. The recipe now asserts the
  parent count explicitly. Observed intact across `0.74.0`,
  `0.75.0`, and `0.76.0`.
- Consolidation, one home each:
  - `cycle-protocol.md` owns the procedure — it is the
    operator's how-to and it outlives the flag (post-hoc
    conversion and non-vc-x1 projects still need hand steps).
    The jj steps are written in jj terms with the `vc-x1 push`
    invocations called out as this project's binding, so the
    template-family fan-out doesn't carry a tool other
    projects lack.
  - The refactor doc's
    [trapezoid stage](../refactor-20260716.md#stage-trapezoid-close-out)
    keeps only what is implementation-specific and links to
    the recipe. It is ephemeral — it becomes history when the
    flag ships — which is the reason it must not own the
    steps.
  - `README.md` gets the user-facing description **with** the
    flag, not before: documenting `--merge` while it doesn't
    exist would be documenting vapor.
- Vocabulary collapsed to one word. The shape was "merge
  non-ff" in the option list and section headers, "trapezoid"
  in every other paragraph. **Trapezoid** is now primary, with
  "(merge non-ff)" as a one-time git-level gloss, so a grep
  finds all of it.
- The published-then-reshaped wart is now written down: step
  4 moves the bookmark sideways, so step 1's SHA becomes
  unreachable and anyone who fetched in between holds a
  dangling commit. The consequence that bites this project is
  the backfill embargo — a `Commits:` fill must never read a
  SHA from that window, which is exactly when the per-push
  cadence would otherwise be due.

### As-built riders

- `AGENTS.md` ochid scoping: "more than one occurs on a merge
  non-ff close-out (one ochid per Work commit in the cycle)"
  overstated the rule. The count is per **push** — `0.76.0`'s
  trapezoid carried exactly one, because its rungs were
  published 1:1 as they landed. The preceding sentence already
  said this correctly; the trailing gloss now agrees with it.
- Backlog retirements: the trapezoidal-merge diagram entry is
  satisfied by the diagram in the recipe (a better home than
  the proposed `notes/README.md` — it sits where the base is
  chosen), and the `vc-x1 push --merge` entry is superseded by
  the ranked `TODO.md` trapezoid-close-out entry.
- `0.76.0` backfill: the close-out `Commits:` ref in
  chores-14's As-built ladder, the program ladder's rung in
  `TODO.md`, and a note on that rung recording it as the first
  trapezoid published by the four-step procedure.

## refactor: stateless push

Commits: [[3]],[[4]],[[5]],[[6]],[[7]]

Picked up 2026-07-28. `push.rs` (~1.5k lines) holds the
`Stage` machine, TOML state persistence, eight stage bodies,
two sanity verifiers, and the interactive gates in one file.
The state file is where the defects come from: bugs.md #3 —
the rollback rewinds the *repos* but not the *state*, so the
rerun skipped the commit stages and republished a previous bot
commit — and both sanity verifiers exist largely to defend
against that staleness. Retiring it, deriving the resume point
from repo reality as standalone `squash-push` already does,
deletes the class rather than patching it. Fifth cycle of the
refactor program; see
[split push.rs](../refactor-20260716.md#stage-split-pushrs)
and
[stateless push](../refactor-20260716.md#stage-stateless-push).

Working ladder (greppable stem `push`) — detail per rung
below:

- [[3]] 0.77.0-0 chore: open stateless push cycle (done)
  [detail](#0770-0-chore-open-stateless-push-cycle)
- [[4]] 0.77.0-1 refactor: extract push/state.rs (done)
  [detail](#0770-1-refactor-extract-pushstaters)
- [[5]] 0.77.0-2 fix: push skips an empty work commit (done)
  [detail](#0770-2-fix-push-skips-an-empty-work-commit)
- [[6]] 0.77.0-3 refactor: drop push state and preflight
  [detail](#0770-3-refactor-drop-push-state-and-preflight)
- [[7]] 0.77.0 refactor: stateless push — close-out

This section was written as one block in `TODO.md > ## In
Progress` while the cycle ran and moved here wholesale at
close-out — the trial of Todo "One home for a cycle's
narrative", adopted mid-cycle at -2. See
[Outcome](#outcome-1) for how it went.

### Decisions at cycle open

- **Trapezoid support was folded in and unfolded the same
  day.** The fold rested on rung -4 deleting `--from`, which
  the manual trapezoid recipe's step 4 uses — but that rung's
  own docs rider makes step 4 a bare `vc-x1 push <bookmark>`,
  since after the reshape the repos are exactly what
  reality-derived resume recognizes. Nothing was stranded, so
  the fold bought nothing and cost a seven-rung cycle.
  Trapezoid support returns to 0.79.0.
- **Program order stays as planned** (stateless push → jj-lib
  → trapezoid). A swap was considered — jj-lib first, since
  the index-lock race keeps firing — and rejected: the
  "we'd write the stage bodies twice" argument doesn't hold
  up. Resume detection is *reads*, which already go through
  the `src/jj.rs` facade; jj-lib rewrites facade internals
  and the hand-rolled mutations, not call sites that already
  went through it. The overlap is thin. (Worth confirming
  against the stage bodies when rung -3 opens.)
- **Doing this first shortens the exposure to bugs.md #1**
  even though it can't fix it. The race can only be retried
  once jj-lib owns the lock acquisition, but rung -4 fixes
  bugs.md #3 — so a rollback stops leaving a poisoned state
  file and a plain rerun becomes safe rather than dangerous.
  The -0 push hit exactly this: the race fired, both repos
  rolled back cleanly, and the state file still said
  `stage=bookmark-set`.
- Ordering within the cycle: the empty-`@` fix (bugs.md #4)
  lands early, at -2, so every later rung's dogfood push is
  protected by it.

### 0.77.0-0 chore: open stateless push cycle

- version 0.77.0-0; the stage picked into `## In Progress`
  as the program ladder's current `####` rung with a
  five-rung ladder
- rider: 0.76.1 `Commits:` backfill
- rider: `## Done` retirement sweep into done.md
- the push itself hit bugs.md #1 twice before landing —
  recorded there as the fourth and fifth occurrences, and as
  #3's second occurrence (clean rollback, poisoned state
  file, `--restart` the safe rerun)

### 0.77.0-1 refactor: extract push/state.rs

- `Stage`, `StateLayout` / `resolve_state_layout`,
  `PushState`, `STATE_FORMAT_VERSION`, the state-dir /
  state-file defaults, and the escape helpers move to
  `src/push/state.rs`; push.rs 1480 → 1101 lines with no
  behavior change
- the parked 0.72.0-1 extraction was **reference, not
  base**: `support-trapezoid-commits` turns out to be
  published (`@origin`), so rebasing it would rewrite a
  pushed commit. Its boundary was reused (the same item set,
  plus `STATE_FORMAT_VERSION` which it left behind at
  version 1) and the extraction redone against the current
  file. Deleting that bookmark — a remote branch — is still
  outstanding
- `escape_multiline` / `unescape_multiline` stay private, so
  their round-trip test moves into `state.rs`'s own
  `#[cfg(test)] mod tests` rather than widening visibility
  for a test's benefit; the remaining state tests reach
  `STATE_FORMAT_VERSION` through push.rs's `#[cfg(test)]`
  re-export beside `DEFAULT_STATE_DIR` / `_FILE` (which
  `config_schema`'s tests already used)
- the module doc records what the stateless-push rungs
  delete, so the next reader knows the file is scaffolding
  with a scheduled end

### 0.77.0-2 fix: push skips an empty work commit

- bugs.md #4 fixed: `commit-work` skips an empty `@` the way
  `commit-bot` always has, and `stage_message` resolves the
  work chid from `@-` when `@` is empty — the unconditional
  `@` was what made the bot's trailer name the duplicate
  instead of the real commit
- **skip, not error.** The bug report offered erroring
  loudly as the alternative, but an empty work `@` is
  legitimate in the publish-only case: commits already made,
  only the bookmark and the remote left to advance — the
  trapezoid recipe's final step, and the shape rung -3
  teaches push to recognize on its own. Erroring would break
  the flow rung -4 depends on
- push does not rewrite a description it didn't author, so a
  hand-made commit keeps its message and simply carries no
  work-side ochid; `validate-desc` / `fix-desc` are the tools
  for adding one. The skip warns rather than informs, because
  a supplied `--title`/`--body` silently going unused is
  worth noticing
- `push_empty_work_at_skips_commit_work` reproduces the bug's
  exact scenario and was confirmed to fail without the fix
  (the empty duplicate's title lands on the bookmark) before
  being kept
- rider: the one-home trial starts here — chores-15's
  in-flight section (intro, decisions, As-built rungs) moved
  back into this block and its two now-orphaned commit refs
  were pruned there, since TODO.md already carries them

### 0.77.0-3 refactor: drop push state and preflight

Rungs -3 and -4 collapsed into one after the 2026-07-28
design conversation went further than the ladder assumed. A
first attempt at -3 derived the resume point from the repos
instead of the state file; that work is superseded here and
was never committed.

The premise: **resume is not recoverable in general.** A
recorded position describes a world we may no longer
understand, and we cannot know why a run failed. If something
goes wrong, push stops and the user and bot put the repos
right — that is not vc-x1's job and cannot be. What vc-x1
owes them is that *rerunning is always safe*, which is a
property of each stage, not of a record.

- **State file gone**, with `PushState`, `StateLayout` /
  `resolve_state_layout`, `STATE_FORMAT_VERSION`, the escape
  helpers, `src/push/state.rs`, the `[push]` `state-dir` /
  `state-file` config keys, and the `.gitignore` coherence
  check that existed only to keep the file out of commits.
  `--restart` / `--from` / `--status` go with it — all three
  are resume machinery.
- **Per-stage guards instead of a start point.** Each stage
  checks its own precondition and no-ops when its work is
  already done, so a rerun after any failure is just a run.
  `commit-work` already skips an empty `@` (-2);
  `bookmark-set` is a set; `push-work` needs no guard of its
  own — jj's push reports nothing to do when the bookmark is
  already published — and instead picks up preflight's
  tracking check, which is its real precondition;
  `squash-push-bot` skips an empty `@`; `message` doesn't
  demand a title when neither side will commit. This
  subsumes the derived-start-point idea: nothing needs to
  decide *where* to begin.
- **Preflight dropped entirely.** Its three checks were
  vc-x1's own preconditions, not project checks (the Rust
  cargo cycle was always outside vc-x1). Bookmark tracking
  moves to `push-work`, which is what needs it; `sync
  --check` goes — it was the expensive one, it re-invoked
  `current_exe()` as a subprocess, and that self-spawn is
  why the integration tests needed `--from message` to skip
  the stage at all. Dropping it takes the workaround with
  it.
- **The bot-published invariant goes too**, after being
  argued down twice: jj's op log means local content isn't
  lost, and a bot repo that hasn't been squash-pushed yet is
  not a broken state — it's an unfinished errand with no
  deadline, and it self-heals on the next push. What we
  accept is a window where a published work trailer names a
  bot commit no fresh clone can resolve; that only matters
  to a third party, and that world is
  [forks-multi-user](../forks-multi-user.md)'s to solve.
- **The in-process `jj op` rollback stays.** Persisted state
  is a guess; a snapshot taken moments earlier in the same
  process is not. It is what makes a failure before the
  remote boundary cost nothing — both of the -0 push's
  index-lock failures rolled back clean.
- `squash-push` stays repo-agnostic. It knows there is a
  repo and a bookmark, not whether it is the bot side, and
  it should not consult `.vc-config.toml` to find out. Its
  exists-and-tracked check stays, for a reason unrelated to
  topology: `jj bookmark set` creates rather than errors and
  `jj git push` publishes a new bookmark without ceremony
  (confirmed at 0.76.0-5), so a typo'd name would create a
  branch on origin instead of failing.

### Outcome

- Push lost ~940 lines net and gained a property it can state
  in one sentence: rerunning is always safe. `push.rs` went
  1480 → 816 lines across the cycle and now reads top to
  bottom — no state machine, no dispatch loop, no resume.
- What the deletions removed is a *class* of defect, not
  instances. bugs.md #3 is unrepresentable now: there is no
  record to go stale against the repos. #4 is fixed and
  pinned. The two sanity verifiers that existed to police the
  state file went with it, leaving only the completion check,
  which verifies reality against what this run just did.
- The scope grew twice mid-cycle, both times because an
  argument didn't survive contact:
  - the derived-resume-point design (a first attempt at -3,
    never committed) was superseded by per-stage guards —
    deciding *where to start* is unnecessary if every stage
    is safe to repeat;
  - preflight, not in the original ladder at all, went once
    its three checks were examined individually: two were
    preconditions belonging to specific stages, and the
    third's self-spawn was the reason the tests needed a
    stage-skipping flag.
- Dogfooded end to end: the `-3` push that deleted the state
  machinery ran on the stateless build, and the close-out ran
  the trapezoid recipe's step 4 as a bare `vc-x1 push` — the
  flow that used to need `--from bookmark-set`.
- Still open: `support-trapezoid-commits` is published, so
  deleting it deletes a remote branch — it needs an explicit
  go and has not had one. The bookmark's only value was the
  parked extraction, which was reused as reference at -1.

### Outcome: the one-home trial

Todo "One home for a cycle's narrative" was adopted at -2 and
ran through close-out. Verdict: keep it.

- The dual maintenance is gone. Under the old convention this
  cycle would have had every rung written twice and each
  `Commits:` backfill applied in two files; instead the
  working ladder carried the refs and this section is a move,
  not a rewrite.
- The migration is mechanical — four transforms, no rewriting:
  - heading levels shift one deeper (`####`→`##`,
    `#####`→`###`);
  - rung refs renumber into the destination file's namespace;
  - repo-root-relative links gain `../`;
  - the block's own note about being migrated is rewritten,
    since it described a future that has now happened.
- Two of those fail *silently* — a mis-renumbered ref and an
  un-rebased link both render as plain text or 404 rather
  than erroring. That is the case for the proposed
  `validate-repo` (every `[[N]]` resolves, every `[N]:` is
  cited, every relative link exists) more than for automating
  the move itself.
- The per-rung `[detail](#…)` anchors survived the depth
  change untouched, because GitHub slugs derive from the
  heading *text*, not its level. Worth knowing: it means the
  working ladder's links keep working after migration with no
  edit at all.
- The `Commits:` line is retired for sections that carry a
  working ladder — the ladder gives the same refs per rung,
  with titles, which is strictly more informative. Sections
  without a ladder (a single-commit interlude, say) still
  need it.
- What we watched for didn't happen: the narrative did *not*
  thin out from being written in TODO.md rather than chores.
  The per-rung sections were written when each rung landed,
  which was the property the per-commit convention protected.
- Not yet settled: whether `#####` per-rung sections are the
  right depth in TODO.md (they nest under the program `###`
  and the cycle `####`). They read fine and the anchors are
  genuinely useful in the raw file, which was the argument for
  keeping them.

## docs: jj-lib design notes + trapezoid recipe

Commits: [[8]]

A trunk-line interlude between `0.77.0` and the punctuation
cycle, carrying two threads from the 2026-07-29 session plus
the corrections the `0.77.0` close-out earned by attempting
the trapezoid recipe as written.

- The op-store coexistence risk was a stub asking for a spike;
  it is now answered, and the answer is "unenforceable,
  probably fine". Read out of jj-lib 0.41 source against an
  installed jj 0.40.0: structural change fails closed, the
  index self-heals by reindexing, the op store serializes with
  protobuf and carries no version stamp, and jj publishes no
  on-disk compatibility policy. We think the residual risk is
  low-probability, silent, and low-blast-radius.
- The mitigation that looked obvious does not work. A
  `jj --version` gate samples what `$PATH` resolves to, which
  says nothing about the jj an editor integration or a later
  session runs against the same repo. This workspace already
  has two.
- Consequence for the ladder: taking the coupling is a
  decision, not a step 0.78.0 can assume. The index-lock prize
  (bugs.md #1) does not require it, since a retry can wrap the
  existing spawn today.
- The trapezoid recipe's step 4 is `jj git push`, not
  `vc-x1 push`. Push runs its whole pipeline or none of it,
  and the bot repo is never quiet: by the time the reshape is
  done, `.claude` holds the session writes from steps 1-3, so
  `commit-bot` wants a title for a work-side publish that
  needs nothing but a moved ref.

## docs: typeable punctuation

Commits: [[9]]

`—`, `–`, `…` and `→` cost nothing to write and are paid on
every read: none can be typed at a terminal, so none can be
grepped for, and an em dash next to option syntax reads as
another flag. Nobody chose the 551 sites across the five prose
files, they accumulated. AGENTS.md gains the rule that bans
them, and the sweep converts what is already written.

One commit, no ladder: the rule and the sweep are one idea,
and per-file rungs would buy no review value in prose files
that have no build and no tests.

It started as a question about whether a ladder rung's title
and its detail should share a line. They should not, and the
em dash was what let them: it joins two things without naming
their relationship, which is the failure `### Semicolons
inside bullets` already names for `;`. Pulling that thread
reached the character itself, and then the whole class. The
rule is `### Typeable punctuation only` under
[Prose form](/AGENTS.md#prose-form), a sibling to the semicolon
rule and cross-linked from its intro. It was designed jointly
with the iiac-perf project, whose AGENTS.md carries the same
section.

### Three roles a banned character can play

The rule took four rounds to get right, and each round was the
same mistake in a different place: a blanket claim that the
next paragraph then contradicted. The settled form sorts by
what the character is *doing*, not where it sits.

- **Naming it** (`` `…` ``) is a specimen and stays. This is
  how the section names the characters it bans.
- **Doing a job** (`` `.expect(…)` ``) is a use and converts.
  A first draft exempted all code spans, which would have
  forbidden the very conversions the sweep had just made.
- **Transcribed from outside** (tool output, an error message,
  an already-published commit title) keeps its characters,
  code span or not. Transcription is not authoring.

The third role is why `README.md` still holds four em dashes:
its `vc-x1 config` samples transcribe what the installed binary
prints, and `src/config_schema.rs` puts em dashes in the `doc:`
strings. Converting the samples would make the docs claim
output that does not exist. They convert whenever the strings
do, in that same commit; the source sweep was deferred out of
the `0.77.x` ladder on 2026-07-30 and now sits in `## Todo` as
"typeable punctuation: source sweep + rule rewording".

We think the reason this rule needs to be absolute where the
semicolon rule is a lean is that the cost is asymmetric: an em
dash is free to write and paid on every read, so a soft rule
accumulates them. 551 sites in five files is the evidence.

### Four banned characters, and an audit that checked three

The rule bans four characters: `—`, `–`, `…`, `→`. Every
verification run during the sweep counted three of them. The en
dash was never in the audit command, so it was never in the
report, so the sweep read as complete while two real sites
survived: a section range written `1–8` in TODO.md and a step
range written `1–3` in cycle-protocol.md. Both were found by
the last check
before the commit, which happened to include the fourth
character.

We think the en dash is the easiest of the four to miss for a
reason that will recur: it is the only one with no distinct
visual signature at a glance. `…` and `→` are unmistakable, and
an em dash is long enough to notice in running prose, but `–`
between two numbers looks exactly like the hyphen it should
have been. It hides in precisely the construct it is used for.

Two consequences worth carrying forward:

- **The audit must enumerate the ban, not a memory of it.** A
  check that tracks a subset silently reports success. This is
  the same defect class as the `clippy ... | tail -2 && cargo
  test` gate found at the 0.77.0 close-out, where the pipeline
  status came from `tail` and the `&&` was decorative: a
  mechanism that looks like a guarantee and is not.
- **A future `validate-repo` byte scan should read its
  character set from one place** shared with the rule's text,
  so the two cannot drift. Enumerating the characters a second
  time in the checker recreates exactly the gap this cycle hit.

The count reported for this cycle, 551 sites, is therefore two
low as originally tallied; the corrected total is 553.

### An em dash in a heading moves its anchor

Found mid-sweep, and it is the one silent failure in the whole
change. Stripping the dash leaves the spaces on both sides, so
`## A — B` slugs to `#a--b` while the colon form slugs to
`#a-b`. Converting a heading therefore breaks every inbound
link to it.

Three headings in AGENTS.md and one in refactor-20260716.md
were affected; a repo-wide grep found four inbound links, all
in the same files. The rule now carries the warning.

## docs: re-describe rule + defer punctuation sweep

Commits: [[10]]

Two unrelated threads, both found by reading the `0.77.x`
ladder rather than by doing the work it planned: a `jj describe`
hazard that had been recorded only inside a feature request,
and two planned rungs whose value did not survive scrutiny.

- **Re-describing is coordinate-first.** `jj describe` on a
  published or already-stamped commit is a history rewrite and
  silently drops the `ochid:` trailer, since it replaces the
  whole message and nothing guards it the way `squash-push`
  guards a squash. The hazard is not theoretical: it cost the
  `0.77.2` amend its trailer, which survived only by
  hand-copying. AGENTS.md carries the rule, next to the ochid
  semantics rather than inside the Todo that will eventually
  fix it.
- **The sub-cycle ladder is the named exception**, and it
  needed saying from both ends. Ladder commits never leave the
  machine, so they carry no trailer and step 4's `describe` is
  first-time authoring, not a rewrite. cycle-protocol says so
  at the per-Work-commit contract and again at the Close-out
  squash, which is the ladder's single push and where the
  trailer is stamped.
- **`typically` became `will`** in the sub-cycle intro, since
  the local-only claim is flat and a hedge in the same section
  undercut it.
- **Two rungs left the ladder**: the `src` punctuation sweep to
  `## Todo`, interlude shape to the backlog.

### Wording strength should match the deviation it invites

The session hit the same calibration error twice, pointing
opposite ways, which is what made it visible.

- `Banned:` in the punctuation rule was too strong for a rule
  with legitimate exceptions, so the section spent four
  paragraphs contradicting its own first word.
- `typically` in the sub-cycle intro was too weak for a rule
  whose exception should be deliberate, letting a reader stray
  without noticing a choice had been made.

We think the useful test is not "is this rule absolute" but
"how much hesitation should breaking it cost". Any rule here
can be broken; the wording decides whether breaking it is an
act or a default.

### Why the source sweep left the ladder

The rule cannot be enforced at the byte level, and noticing
that changed the sweep's shape and its checker both.

- **`Banned` is not what the rule means.** Transcribed tool
  output and published commit titles keep their characters,
  so presence in a file is legitimate and the prohibition is
  on *authoring*. The rewording is now the first step of the
  `## Todo` entry, because it is what bounds the sweep.
- **Only `src/` and `tests/` are converted.** Everything else
  converts when touched. The chores archive is thick with
  transcription that must not convert, so sweeping it would be
  judgment calls, not a sweep.
- **A `validate-repo` byte scan cannot implement this**, since
  no scanner separates authored from transcribed. The backlog
  entry now asks for a per-file count baseline that fails on a
  rise, which supersedes the note two sections above asking
  the checker to read its character set from one place.

## build: bump jj-lib to 0.43

Commits:

The local `jj` went to 0.43.0, leaving the pin two releases
behind at 0.41. Not migration work: this keeps the existing
read-side compiling against the installed jj, and it is
correct whichever way the mutation decision goes, so it lands
on the trunk line before the cycle rather than inside it.

- `RevsetParseContext::use_glob_by_default` is gone in 0.43,
  so the field assignment goes with it.
- `Revset::commit_change_ids()` returns a `LocalBoxStream`
  rather than an iterator. Consumed with `StreamExt::next`
  driven by the `pollster` `block_on` the file already used
  for `load_at_head()`.
- `futures` is promoted from a transitive dependency to a
  direct one. The `Revset` trait offers no blocking iterator
  any more, so consuming any of its three methods needs
  `StreamExt`; the alternative was a hand-rolled poll loop
  with a no-op waker.

### A silent semantic change under a loud API break

The compiler reported the removed field. It could not report
that the behavior the field selected had also changed, and
that is the more interesting half.

- In 0.41 the default string-pattern kind was chosen by the
  flag: `glob` when true, `substring` when false. We passed
  false.
- In 0.43 `expect_string_expression` builds
  `StringPattern::glob(value)` unconditionally. There is no
  opt-out, so the default moved to glob.
- A revset like `description(foo)` therefore went from
  matching any description containing "foo" to matching the
  literal "foo". Nothing of ours changes, since our revsets
  are `all()`, `children(<hex>)` and bare change ids, none of
  which take string patterns. A user-supplied
  `-r 'description(foo)'` does change.

We think this is worth recording because of what it says
about the mutation decision this cycle is deciding. Two
releases of a pre-1.0 library produced one break the compiler
caught and one it structurally could not, and the green build
that followed the fix was not evidence the bump preserved
behavior. That asymmetry is the treadmill cost, and it is
paid on every bump, not only the ones that touch the op
store.

# References

[1]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[2]: https://github.com/winksaville/vc-x1/commit/2424e14f858d "2424e14f858d010e5c07e8821149a114b3d3dda5"
[3]: https://github.com/winksaville/vc-x1/commit/4898d93e4172 "4898d93e41720070cddb995bfd4e53ffc38ccb88"
[4]: https://github.com/winksaville/vc-x1/commit/ab3a07d4903b "ab3a07d4903bbe6ae7cec5490f5edd622161c72e"
[5]: https://github.com/winksaville/vc-x1/commit/846b5eee5b98 "846b5eee5b988b0cd8887559a0fba3397155ee19"
[6]: https://github.com/winksaville/vc-x1/commit/66aa3f67d4b1 "66aa3f67d4b1308bb08388ccb929fc27967e8259"
[7]: https://github.com/winksaville/vc-x1/commit/9d6f7c0b0f05 "9d6f7c0b0f05ae74dd7100d457b92b72d913404f"
[8]: https://github.com/winksaville/vc-x1/commit/3be698fcde83 "3be698fcde831b09949077e1ce934839ee01f4ea"
[9]: https://github.com/winksaville/vc-x1/commit/62d71818d78b "62d71818d78bc06ae8f5cc17ca060d30a08b6ea1"
[10]: https://github.com/winksaville/vc-x1/commit/03df811a72fe "03df811a72fe61bdd013e34961e72aecd671c126"
