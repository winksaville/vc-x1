# Chores-15

Continuation of `chores-14.md` (closed at `0.76.0`, the repo-registry close-out). This file covers
`0.76.1` onward, the remainder of the jj refactor program
([refactor-20260716.md](../refactor-20260716.md)), worked on the `refactor-vc-x1` bookmark while
`main` parks at the `0.71.0` tip.

Reference numbering is file-local; see
[`agent-data/notes.md`](../../agent-data/notes.md#reference-numbering); chores-15 starts at `[1]`.

## Table of Contents

- [docs: trapezoid close-out recipe](#docs-trapezoid-close-out-recipe)
- [refactor: stateless push](#refactor-stateless-push)
- [docs: jj-lib design notes + trapezoid recipe](#docs-jj-lib-design-notes--trapezoid-recipe)
- [docs: typeable punctuation](#docs-typeable-punctuation)
- [docs: re-describe rule + defer punctuation sweep](#docs-re-describe-rule--defer-punctuation-sweep)
- [build: bump jj-lib to 0.43](#build-bump-jj-lib-to-043)
- [refactor: jj-lib migration](#refactor-jj-lib-migration)

## docs: trapezoid close-out recipe

Commits: [[2]]

Publishing `0.76.0` as a trapezoid [[1]] needed four steps that no single document described. The
procedure was spread across three places (the cycle-protocol recipe, the refactor doc's trapezoid
stage, and a passing mention in a `0.74.0` As-built rung), and where they overlapped they disagreed.
Two of the three stated mechanisms were wrong, and the one check that would catch a silent failure
appeared nowhere. This interlude consolidates them into a single definitive procedure before the
`--merge` flag is built from it.

- The four corrections, each a claim that survived because nothing was the source of truth:
  - **Base selection.** The recipe glossed the first parent as "the previous cycle's close-out (the
    current `main` tip)"; the design doc said "the parent of the ladder's first commit". They
    coincide only when no interlude sits between cycles. At `0.76.0` they differed (the base was the
    `0.75.1` docs interlude, not the `0.75.0` close-out), and the recipe's gloss would have
    swallowed the interlude into the merge's ladder side.
  - **Why `jj new` is needed.** The recipe attributed it to the rebase leaving `@` *on* the merge.
    It doesn't: `jj rebase -r` re-parents descendants onto the rebased commit's **old** parent, so
    `@` lands beside the merge, not on it. Same step, different reason, and the wrong reason makes a
    load-bearing step read as cosmetic, since `bookmark-set` resolves `@-`.
  - **Immutability.** "Its commits are immutable, the rebase needs `--ignore-immutable`" is true
    only when the close-out is already on `trunk()`. A long-lived topic bookmark isn't, so the
    reshape needs no flag.
  - **Ordering.** The design doc specifies the merge in-flight (after `commit-bot`, before
    `bookmark-set`); the four-step manual procedure reshapes after a completed push. Both are legal,
    they have different consequences, and nothing said so.
- The missing check: jj does **not** collapse the second parent when the base is an ancestor of the
  tip, but nothing said to verify it. A simplified merge is indistinguishable from a correct one in
  `jj log --no-graph`, and the mistake is only visible once published. The recipe now asserts the
  parent count explicitly. Observed intact across `0.74.0`, `0.75.0`, and `0.76.0`.
- Consolidation, one home each:
  - `cycle-protocol.md` owns the procedure: it is the operator's how-to and it outlives the flag
    (post-hoc conversion and non-vc-x1 projects still need hand steps). The jj steps are written in
    jj terms with the `vc-x1 push` invocations called out as this project's binding, so the
    template-family fan-out doesn't carry a tool other projects lack.
  - The refactor doc's [trapezoid stage](../refactor-20260716.md#stage-trapezoid-close-out) keeps
    only what is implementation-specific and links to the recipe. It is ephemeral (it becomes
    history when the flag ships), which is the reason it must not own the steps.
  - `README.md` gets the user-facing description **with** the flag, not before: documenting
    `--merge` while it doesn't exist would be documenting vapor.
- Vocabulary collapsed to one word. The shape was "merge non-ff" in the option list and section
  headers, "trapezoid" in every other paragraph. **Trapezoid** is now primary, with "(merge non-ff)"
  as a one-time git-level gloss, so a grep finds all of it.
- The published-then-reshaped wart is now written down: step 4 moves the bookmark sideways, so step
  1's SHA becomes unreachable and anyone who fetched in between holds a dangling commit. The
  consequence that bites this project is the backfill embargo: a `Commits:` fill must never read a
  SHA from that window, which is exactly when the per-push cadence would otherwise be due.

### As-built riders

- `AGENTS.md` ochid scoping: "more than one occurs on a merge non-ff close-out (one ochid per Work
  commit in the cycle)" overstated the rule. The count is per **push**: `0.76.0`'s trapezoid carried
  exactly one, because its rungs were published 1:1 as they landed. The preceding sentence already
  said this correctly; the trailing gloss now agrees with it.
- Backlog retirements: the trapezoidal-merge diagram entry is satisfied by the diagram in the recipe
  (a better home than the proposed `notes/README.md`; it sits where the base is chosen), and the
  `vc-x1 push --merge` entry is superseded by the ranked `TODO.md` trapezoid-close-out entry.
- `0.76.0` backfill: the close-out `Commits:` ref in chores-14's As-built ladder, the program
  ladder's rung in `TODO.md`, and a note on that rung recording it as the first trapezoid published
  by the four-step procedure.

## refactor: stateless push

Commits: [[3]],[[4]],[[5]],[[6]],[[7]]

Picked up 2026-07-28. `push.rs` (~1.5k lines) holds the `Stage` machine, TOML state persistence,
eight stage bodies, two sanity verifiers, and the interactive gates in one file. The state file is
where the defects come from: bugs.md #3 (the rollback rewinds the *repos* but not the *state*, so
the rerun skipped the commit stages and republished a previous bot commit), and both sanity
verifiers exist largely to defend against that staleness. Retiring it, deriving the resume point
from repo reality as standalone `squash-push` already does, deletes the class rather than patching
it. Fifth cycle of the refactor program; see [split
push.rs](../refactor-20260716.md#stage-split-pushrs) and [stateless
push](../refactor-20260716.md#stage-stateless-push).

Working ladder (greppable stem `push`), detail per rung below:

- [[3]] 0.77.0-0 chore: open stateless push cycle (done)
  [detail](#0770-0-chore-open-stateless-push-cycle)
- [[4]] 0.77.0-1 refactor: extract push/state.rs (done)
  [detail](#0770-1-refactor-extract-pushstaters)
- [[5]] 0.77.0-2 fix: push skips an empty work commit (done)
  [detail](#0770-2-fix-push-skips-an-empty-work-commit)
- [[6]] 0.77.0-3 refactor: drop push state and preflight
  [detail](#0770-3-refactor-drop-push-state-and-preflight)
- [[7]] 0.77.0 refactor: stateless push (close-out)

This section was written as one block in `TODO.md > ## In Progress` while the cycle ran and moved
here wholesale at close-out, the trial of Todo "One home for a cycle's narrative", adopted mid-cycle
at -2. See [Outcome](#outcome-1) for how it went.

### Decisions at cycle open

- **Trapezoid support was folded in and unfolded the same day.** The fold rested on rung -4 deleting
  `--from`, which the manual trapezoid recipe's step 4 uses, but that rung's own docs rider makes
  step 4 a bare `vc-x1 push <bookmark>`, since after the reshape the repos are exactly what
  reality-derived resume recognizes. Nothing was stranded, so the fold bought nothing and cost a
  seven-rung cycle. Trapezoid support returns to 0.79.0.
- **Program order stays as planned** (stateless push -> jj-lib -> trapezoid). A swap was considered
  (jj-lib first, since the index-lock race keeps firing) and rejected: the "we'd write the stage
  bodies twice" argument doesn't hold up. Resume detection is *reads*, which already go through the
  `src/jj.rs` facade; jj-lib rewrites facade internals and the hand-rolled mutations, not call sites
  that already went through it. The overlap is thin. (Worth confirming against the stage bodies when
  rung -3 opens.)
- **Doing this first shortens the exposure to bugs.md #1** even though it can't fix it. The race can
  only be retried once jj-lib owns the lock acquisition, but rung -4 fixes bugs.md #3, so a rollback
  stops leaving a poisoned state file and a plain rerun becomes safe rather than dangerous. The -0
  push hit exactly this: the race fired, both repos rolled back cleanly, and the state file still
  said `stage=bookmark-set`.
- Ordering within the cycle: the empty-`@` fix (bugs.md #4) lands early, at -2, so every later
  rung's dogfood push is protected by it.

### 0.77.0-0 chore: open stateless push cycle

- version 0.77.0-0; the stage picked into `## In Progress` as the program ladder's current `####`
  rung with a five-rung ladder
- rider: 0.76.1 `Commits:` backfill
- rider: `## Done` retirement sweep into done.md
- the push itself hit bugs.md #1 twice before landing, recorded there as the fourth and fifth
  occurrences, and as #3's second occurrence (clean rollback, poisoned state file, `--restart` the
  safe rerun)

### 0.77.0-1 refactor: extract push/state.rs

- `Stage`, `StateLayout` / `resolve_state_layout`, `PushState`, `STATE_FORMAT_VERSION`, the
  state-dir / state-file defaults, and the escape helpers move to `src/push/state.rs`; push.rs 1480
  -> 1101 lines with no behavior change
- the parked 0.72.0-1 extraction was **reference, not base**: `support-trapezoid-commits` turns out
  to be published (`@origin`), so rebasing it would rewrite a pushed commit. Its boundary was reused
  (the same item set, plus `STATE_FORMAT_VERSION` which it left behind at version 1) and the
  extraction redone against the current file. Deleting that bookmark (a remote branch) is still
  outstanding
- `escape_multiline` / `unescape_multiline` stay private, so their round-trip test moves into
  `state.rs`'s own `#[cfg(test)] mod tests` rather than widening visibility for a test's benefit;
  the remaining state tests reach `STATE_FORMAT_VERSION` through push.rs's `#[cfg(test)]` re-export
  beside `DEFAULT_STATE_DIR` / `_FILE` (which `config_schema`'s tests already used)
- the module doc records what the stateless-push rungs delete, so the next reader knows the file is
  scaffolding with a scheduled end

### 0.77.0-2 fix: push skips an empty work commit

- bugs.md #4 fixed: `commit-work` skips an empty `@` the way `commit-bot` always has, and
  `stage_message` resolves the work chid from `@-` when `@` is empty: the unconditional `@` was what
  made the bot's trailer name the duplicate instead of the real commit
- **skip, not error.** The bug report offered erroring loudly as the alternative, but an empty work
  `@` is legitimate in the publish-only case: commits already made, only the bookmark and the remote
  left to advance: the trapezoid recipe's final step, and the shape rung -3 teaches push to
  recognize on its own. Erroring would break the flow rung -4 depends on
- push does not rewrite a description it didn't author, so a hand-made commit keeps its message and
  simply carries no work-side ochid; `validate-desc` / `fix-desc` are the tools for adding one. The
  skip warns rather than informs, because a supplied `--title`/`--body` silently going unused is
  worth noticing
- `push_empty_work_at_skips_commit_work` reproduces the bug's exact scenario and was confirmed to
  fail without the fix (the empty duplicate's title lands on the bookmark) before being kept
- rider: the one-home trial starts here: chores-15's in-flight section (intro, decisions, As-built
  rungs) moved back into this block and its two now-orphaned commit refs were pruned there, since
  TODO.md already carries them

### 0.77.0-3 refactor: drop push state and preflight

Rungs -3 and -4 collapsed into one after the 2026-07-28 design conversation went further than the
ladder assumed. A first attempt at -3 derived the resume point from the repos instead of the state
file; that work is superseded here and was never committed.

The premise: **resume is not recoverable in general.** A recorded position describes a world we may
no longer understand, and we cannot know why a run failed. If something goes wrong, push stops and
the user and bot put the repos right; that is not vc-x1's job and cannot be. What vc-x1 owes them is
that *rerunning is always safe*, which is a property of each stage, not of a record.

- **State file gone**, with `PushState`, `StateLayout` / `resolve_state_layout`,
  `STATE_FORMAT_VERSION`, the escape helpers, `src/push/state.rs`, the `[push]` `state-dir` /
  `state-file` config keys, and the `.gitignore` coherence check that existed only to keep the file
  out of commits. `--restart` / `--from` / `--status` go with it: all three are resume machinery.
- **Per-stage guards instead of a start point.** Each stage checks its own precondition and no-ops
  when its work is already done, so a rerun after any failure is just a run. `commit-work` already
  skips an empty `@` (-2); `bookmark-set` is a set; `push-work` needs no guard of its own (jj's push
  reports nothing to do when the bookmark is already published) and instead picks up preflight's
  tracking check, which is its real precondition; `squash-push-bot` skips an empty `@`; `message`
  doesn't demand a title when neither side will commit. This subsumes the derived-start-point idea:
  nothing needs to decide *where* to begin.
- **Preflight dropped entirely.** Its three checks were vc-x1's own preconditions, not project
  checks (the Rust cargo cycle was always outside vc-x1). Bookmark tracking moves to `push-work`,
  which is what needs it; `sync --check` goes: it was the expensive one, it re-invoked
  `current_exe()` as a subprocess, and that self-spawn is why the integration tests needed `--from
  message` to skip the stage at all. Dropping it takes the workaround with it.
- **The bot-published invariant goes too**, after being argued down twice: jj's op log means local
  content isn't lost, and a bot repo that hasn't been squash-pushed yet is not a broken state: it's
  an unfinished errand with no deadline, and it self-heals on the next push. What we accept is a
  window where a published work trailer names a bot commit no fresh clone can resolve; that only
  matters to a third party, and that world is [forks-multi-user](../forks-multi-user.md)'s to solve.
- **The in-process `jj op` rollback stays.** Persisted state is a guess; a snapshot taken moments
  earlier in the same process is not. It is what makes a failure before the remote boundary cost
  nothing: both of the -0 push's index-lock failures rolled back clean.
- `squash-push` stays repo-agnostic. It knows there is a repo and a bookmark, not whether it is the
  bot side, and it should not consult `.vc-config.toml` to find out. Its exists-and-tracked check
  stays, for a reason unrelated to topology: `jj bookmark set` creates rather than errors and `jj
  git push` publishes a new bookmark without ceremony (confirmed at 0.76.0-5), so a typo'd name
  would create a branch on origin instead of failing.

### Outcome

- Push lost ~940 lines net and gained a property it can state in one sentence: rerunning is always
  safe. `push.rs` went 1480 -> 816 lines across the cycle and now reads top to bottom: no state
  machine, no dispatch loop, no resume.
- What the deletions removed is a *class* of defect, not instances. bugs.md #3 is unrepresentable
  now: there is no record to go stale against the repos. #4 is fixed and pinned. The two sanity
  verifiers that existed to police the state file went with it, leaving only the completion check,
  which verifies reality against what this run just did.
- The scope grew twice mid-cycle, both times because an argument didn't survive contact:
  - the derived-resume-point design (a first attempt at -3, never committed) was superseded by
    per-stage guards: deciding *where to start* is unnecessary if every stage is safe to repeat;
  - preflight, not in the original ladder at all, went once its three checks were examined
    individually: two were preconditions belonging to specific stages, and the third's self-spawn
    was the reason the tests needed a stage-skipping flag.
- Dogfooded end to end: the `-3` push that deleted the state machinery ran on the stateless build,
  and the close-out ran the trapezoid recipe's step 4 as a bare `vc-x1 push`, the flow that used to
  need `--from bookmark-set`.
- Still open: `support-trapezoid-commits` is published, so deleting it deletes a remote branch: it
  needs an explicit go and has not had one. The bookmark's only value was the parked extraction,
  which was reused as reference at -1.

### Outcome: the one-home trial

Todo "One home for a cycle's narrative" was adopted at -2 and ran through close-out. Verdict: keep
it.

- The dual maintenance is gone. Under the old convention this cycle would have had every rung
  written twice and each `Commits:` backfill applied in two files; instead the working ladder
  carried the refs and this section is a move, not a rewrite.
- The migration is mechanical (four transforms, no rewriting):
  - heading levels shift one deeper (`####` -> `##`, `#####` -> `###`);
  - rung refs renumber into the destination file's namespace;
  - repo-root-relative links gain `../`;
  - the block's own note about being migrated is rewritten, since it described a future that has now
    happened.
- Two of those fail *silently*: a mis-renumbered ref and an un-rebased link both render as plain
  text or 404 rather than erroring. That is the case for the proposed `validate-repo` (every `[[N]]`
  resolves, every `[N]:` is cited, every relative link exists) more than for automating the move
  itself.
- The per-rung `[detail](#...)` anchors survived the depth change untouched, because GitHub slugs
  derive from the heading *text*, not its level. Worth knowing: it means the working ladder's links
  keep working after migration with no edit at all.
- The `Commits:` line is retired for sections that carry a working ladder: the ladder gives the same
  refs per rung, with titles, which is strictly more informative. Sections without a ladder (a
  single-commit interlude, say) still need it.
- What we watched for didn't happen: the narrative did *not* thin out from being written in TODO.md
  rather than chores. The per-rung sections were written when each rung landed, which was the
  property the per-commit convention protected.
- Not yet settled: whether `#####` per-rung sections are the right depth in TODO.md (they nest under
  the program `###` and the cycle `####`). They read fine and the anchors are genuinely useful in
  the raw file, which was the argument for keeping them.

## docs: jj-lib design notes + trapezoid recipe

Commits: [[8]]

A trunk-line interlude between `0.77.0` and the punctuation cycle, carrying two threads from the
2026-07-29 session plus the corrections the `0.77.0` close-out earned by attempting the trapezoid
recipe as written.

- The op-store coexistence risk was a stub asking for a spike; it is now answered, and the answer is
  "unenforceable, probably fine". Read out of jj-lib 0.41 source against an installed jj 0.40.0:
  structural change fails closed, the index self-heals by reindexing, the op store serializes with
  protobuf and carries no version stamp, and jj publishes no on-disk compatibility policy. We think
  the residual risk is low-probability, silent, and low-blast-radius.
- The mitigation that looked obvious does not work. A `jj --version` gate samples what `$PATH`
  resolves to, which says nothing about the jj an editor integration or a later session runs against
  the same repo. This workspace already has two.
- Consequence for the ladder: taking the coupling is a decision, not a step 0.78.0 can assume. The
  index-lock prize (bugs.md #1) does not require it, since a retry can wrap the existing spawn
  today.
- The trapezoid recipe's step 4 is `jj git push`, not `vc-x1 push`. Push runs its whole pipeline or
  none of it, and the bot repo is never quiet: by the time the reshape is done, `.claude` holds the
  session writes from steps 1-3, so `commit-bot` wants a title for a work-side publish that needs
  nothing but a moved ref.

## docs: typeable punctuation

Commits: [[9]]

`—`, `–`, `…` and `→` cost nothing to write and are paid on every read: none can be typed at a
terminal, so none can be grepped for, and an em dash next to option syntax reads as another flag.
Nobody chose the 551 sites across the five prose files, they accumulated. AGENTS.md gains the rule
that bans them, and the sweep converts what is already written.

One commit, no ladder: the rule and the sweep are one idea, and per-file rungs would buy no review
value in prose files that have no build and no tests.

It started as a question about whether a ladder rung's title and its detail should share a line.
They should not, and the em dash was what let them: it joins two things without naming their
relationship, which is the failure `### Semicolons inside bullets` already names for `;`. Pulling
that thread reached the character itself, and then the whole class. The rule is `### Typeable
punctuation only` under [Prose form](/agent-data/prose.md#prose-form), a sibling to the semicolon
rule and cross-linked from its intro. It was designed jointly with the iiac-perf project, whose
AGENTS.md carries the same section.

### Three roles a banned character can play

The rule took four rounds to get right, and each round was the same mistake in a different place: a
blanket claim that the next paragraph then contradicted. The settled form sorts by what the
character is *doing*, not where it sits.

- **Naming it** (`` `…` ``) is a specimen and stays. This is how the section names the characters it
  bans.
- **Doing a job** (`` `.expect(…)` ``) is a use and converts. A first draft exempted all code spans,
  which would have forbidden the very conversions the sweep had just made.
- **Transcribed from outside** (tool output, an error message, an already-published commit title)
  keeps its characters, code span or not. Transcription is not authoring.

The third role is why `README.md` still holds four em dashes: its `vc-x1 config` samples transcribe
what the installed binary prints, and `src/config_schema.rs` puts em dashes in the `doc:` strings.
Converting the samples would make the docs claim output that does not exist. They convert whenever
the strings do, in that same commit; the source sweep was deferred out of the `0.77.x` ladder on
2026-07-30 and now sits in `## Todo` as "typeable punctuation: source sweep + rule rewording".

We think the reason this rule needs to be absolute where the semicolon rule is a lean is that the
cost is asymmetric: an em dash is free to write and paid on every read, so a soft rule accumulates
them. 551 sites in five files is the evidence.

### Four banned characters, and an audit that checked three

The rule bans four characters: `—`, `–`, `…`, `→`. Every verification run during the sweep counted
three of them. The en dash was never in the audit command, so it was never in the report, so the
sweep read as complete while two real sites survived: a section range written `1–8` in TODO.md and a
step range written `1–3` in cycle-protocol.md. Both were found by the last check before the commit,
which happened to include the fourth character.

We think the en dash is the easiest of the four to miss for a reason that will recur: it is the only
one with no distinct visual signature at a glance. `…` and `→` are unmistakable, and an em dash is
long enough to notice in running prose, but `–` between two numbers looks exactly like the hyphen it
should have been. It hides in precisely the construct it is used for.

Two consequences worth carrying forward:

- **The audit must enumerate the ban, not a memory of it.** A check that tracks a subset silently
  reports success. This is the same defect class as the `clippy ... | tail -2 && cargo test` gate
  found at the 0.77.0 close-out, where the pipeline status came from `tail` and the `&&` was
  decorative: a mechanism that looks like a guarantee and is not.
- **A future `validate-repo` byte scan should read its character set from one place** shared with
  the rule's text, so the two cannot drift. Enumerating the characters a second time in the checker
  recreates exactly the gap this cycle hit.

The count reported for this cycle, 551 sites, is therefore two low as originally tallied; the
corrected total is 553.

### An em dash in a heading moves its anchor

Found mid-sweep, and it is the one silent failure in the whole change. Stripping the dash leaves the
spaces on both sides, so `## A — B` slugs to `#a--b` while the colon form slugs to `#a-b`.
Converting a heading therefore breaks every inbound link to it.

Three headings in AGENTS.md and one in refactor-20260716.md were affected; a repo-wide grep found
four inbound links, all in the same files. The rule now carries the warning.

## docs: re-describe rule + defer punctuation sweep

Commits: [[10]]

Two unrelated threads, both found by reading the `0.77.x` ladder rather than by doing the work it
planned: a `jj describe` hazard that had been recorded only inside a feature request, and two
planned rungs whose value did not survive scrutiny.

- **Re-describing is coordinate-first.** `jj describe` on a published or already-stamped commit is a
  history rewrite and silently drops the `ochid:` trailer, since it replaces the whole message and
  nothing guards it the way `squash-push` guards a squash. The hazard is not theoretical: it cost
  the `0.77.2` amend its trailer, which survived only by hand-copying. AGENTS.md carries the rule,
  next to the ochid semantics rather than inside the Todo that will eventually fix it.
- **The sub-cycle ladder is the named exception**, and it needed saying from both ends. Ladder
  commits never leave the machine, so they carry no trailer and step 4's `describe` is first-time
  authoring, not a rewrite. cycle-protocol says so at the per-Work-commit contract and again at the
  Close-out squash, which is the ladder's single push and where the trailer is stamped.
- **`typically` became `will`** in the sub-cycle intro, since the local-only claim is flat and a
  hedge in the same section undercut it.
- **Two rungs left the ladder**: the `src` punctuation sweep to `## Todo`, interlude shape to the
  backlog.

### Wording strength should match the deviation it invites

The session hit the same calibration error twice, pointing opposite ways, which is what made it
visible.

- `Banned:` in the punctuation rule was too strong for a rule with legitimate exceptions, so the
  section spent four paragraphs contradicting its own first word.
- `typically` in the sub-cycle intro was too weak for a rule whose exception should be deliberate,
  letting a reader stray without noticing a choice had been made.

We think the useful test is not "is this rule absolute" but "how much hesitation should breaking it
cost". Any rule here can be broken; the wording decides whether breaking it is an act or a default.

### Why the source sweep left the ladder

The rule cannot be enforced at the byte level, and noticing that changed the sweep's shape and its
checker both.

- **`Banned` is not what the rule means.** Transcribed tool output and published commit titles keep
  their characters, so presence in a file is legitimate and the prohibition is on *authoring*. The
  rewording is now the first step of the `## Todo` entry, because it is what bounds the sweep.
- **Only `src/` and `tests/` are converted.** Everything else converts when touched. The chores
  archive is thick with transcription that must not convert, so sweeping it would be judgment calls,
  not a sweep.
- **A `validate-repo` byte scan cannot implement this**, since no scanner separates authored from
  transcribed. The backlog entry now asks for a per-file count baseline that fails on a rise, which
  supersedes the note two sections above asking the checker to read its character set from one
  place.

## build: bump jj-lib to 0.43

Commits: [[11]]

The local `jj` went to 0.43.0, leaving the pin two releases behind at 0.41. Not migration work: this
keeps the existing read-side compiling against the installed jj, and it is correct whichever way the
mutation decision goes, so it lands on the trunk line before the cycle rather than inside it.

- `RevsetParseContext::use_glob_by_default` is gone in 0.43, so the field assignment goes with it.
- `Revset::commit_change_ids()` returns a `LocalBoxStream` rather than an iterator. Consumed with
  `StreamExt::next` driven by the `pollster` `block_on` the file already used for `load_at_head()`.
- `futures` is promoted from a transitive dependency to a direct one. The `Revset` trait offers no
  blocking iterator any more, so consuming any of its three methods needs `StreamExt`; the
  alternative was a hand-rolled poll loop with a no-op waker.

### A silent semantic change under a loud API break

The compiler reported the removed field. It could not report that the behavior the field selected
had also changed, and that is the more interesting half.

- In 0.41 the default string-pattern kind was chosen by the flag: `glob` when true, `substring` when
  false. We passed false.
- In 0.43 `expect_string_expression` builds `StringPattern::glob(value)` unconditionally. There is
  no opt-out, so the default moved to glob.
- A revset like `description(foo)` therefore went from matching any description containing "foo" to
  matching the literal "foo". Nothing of ours changes, since our revsets are `all()`,
  `children(<hex>)` and bare change ids, none of which take string patterns. A user-supplied `-r
  'description(foo)'` does change.

We think this is worth recording because of what it says about the mutation decision this cycle is
deciding. Two releases of a pre-1.0 library produced one break the compiler caught and one it
structurally could not, and the green build that followed the fix was not evidence the bump
preserved behavior. That asymmetry is the treadmill cost, and it is paid on every bump, not only the
ones that touch the op store.

## refactor: jj-lib migration

Facade internals and mutations move in-process, ending jj and
git spawning; see
[the stage](../refactor-20260716.md#stage-jj-lib-migration).
Scope settled 2026-07-30: all three pieces, accepting the
op-store version coupling that the migration introduces. The
version gate at `-5` is what makes that coupling enforceable
rather than merely accepted, which is the change from the
2026-07-29 framing.

- [[12]] 0.78.0-0 refactor: jj-lib migration opening
  [detail](#0780-0-refactor-jj-lib-migration-opening)
  - retitled 2026-08-02 from `chore: open the jj-lib migration cycle` by a coordinated
    re-describe + force-push, adopting the bookend-titles convention
  - rungs `-0..-5` re-recorded after the rewrite
- [[13]] 0.78.0-1 docs: adopt universal AGENTS
  [detail](#0780-1-docs-adopt-universal-agents)
  - inserted 2026-07-30: the AGENTS restructure proposed in
    vc-x1-work-repo-template becomes this repo's live
    instructions, dogfooded for the rest of the cycle; lands
    first so the remaining rungs run under the new rules
- [[14]] 0.78.0-2 feat: report jj-lib and jj-data versions
  (done)
  [detail](#0780-2-feat-report-jj-lib-and-jj-data-versions)
  - split out of the former `-2` on 2026-07-31: the rung had
    grown a `build.rs`, a module, and a CLI behavior change,
    which no `docs:` title covers
  - the measurement lands before the prose that cites it
- [[15]] 0.78.0-3 docs: jj-lib version coupling policy
  (done)
  [detail](#0780-3-docs-jj-lib-version-coupling-policy)
  - the policy proper goes to `notes/`, beside the risk
    section it supersedes; `TODO.md` keeps the narrative that
    moves to chores at close-out
  - retires three recorded conclusions at once, so they move
    together or the notes argue with themselves: the risk
    section's `jj --version` verdict, this ladder's
    write-path-only bullet, and the "Decisions at cycle open"
    claim that one direction is safe
- [[16]] 0.78.0-4 feat: jj-lib version gate
  [detail](#0780-4-feat-jj-lib-version-gate)
  - moved ahead of the reads rung on 2026-07-31: the ladder
    put the gate before the *mutations*, but
    `common::load_repo` has called `load_at_head` since the
    facade moved to jj-lib, so the write-capable read path is
    live now and the hole is open today, not later
  - builds only the gate; both operands ship at `-2` and the
    rule is written down at `-3` in
    [the policy](../jj-version-policy.md), which this rung
    implements rather than re-decides
  - no carve-out: every subcommand gates. `version` is the one
    exception and barely one, reporting the verdict rather than
    acting on it, printing both versions and withholding the
    `jj-data` lines on a mismatch
  - the `.vc-config.toml` pin turns a `$PATH` sample into a
    declaration, but only matters once more than one jj is in
    play; it stays a Todo
- [[17]] 0.78.0-5 refactor: jj-lib reads
  [detail](#0780-5-refactor-jj-lib-reads)
  - `jj log` templates become `Commit` accessors
  - `@`-relative reads stay behind: they need a working-copy
    snapshot, which is an op-store write, so they move with
    the mutations
- [[18]] 0.78.0-6 refactor: jj-lib mutations (done) [detail](#0780-6-refactor-jj-lib-mutations)
  - commit, describe, bookmark set/track, fetch, push, plus the `@`-relative reads deferred
    from `-5`
- [[19]] 0.78.0-7 refactor: context-owned repo sessions
  [detail](#0780-7-refactor-context-owned-repo-sessions)
  - inserted 2026-08-01 at the `-6` review, design settled there: `Context` owns lazily-opened
    `RepoSession`s keyed by repo path, has-a and never is-a, because an invocation touches
    0..N repos and repo-less commands (`version`) must not open one; verbs become session
    methods; the one-shot facade fns stay as wrappers for context-less callers
  - one op per verb stays: sharing a transaction across stages would change the op-log shape
    that push re-run and sync revert rely on
  - per-verb opens are the lifted subprocess lifecycle made visible, not a regression; this
    rung is the improvement over the spawned form, and push / squash-push / sync are its
    consumers today
  - ordered before the retry so the retry lands on the final frame, though it fits either shape
- [[20]] 0.78.0-8 fix: jj-lib index-lock retry (done) [detail](#0780-8-fix-jj-lib-index-lock-retry)
  - renumbered from `-7` by the session insert
  - bugs.md #1, with the `git init --bare` to gix rider
  - the retry classifies by error variant rather than substring, which is the real win:
    `SpawnInPath` and `UnsupportedGitOption` are never retryable, and treating the whole
    `Subprocess` arm as retryable would loop forever on a missing git binary
- [[21]] 0.78.0-9 docs: sync pin set and adopt new conventions
  [detail](#0780-9-docs-sync-pin-set-and-adopt-new-conventions)
  - inserted 2026-08-02 after the iiac-perf TC session's mailbox and conventions arrived
  - lands last so the close-out itself runs under the amended schema
- [[22]] 0.78.0 refactor: jj-lib migration

### Decisions at cycle open

- **All three pieces**, decided 2026-07-30: jj-lib for reads,
  jj-lib for mutations, and the index-lock retry that is the
  headline prize. The 2026-07-29 session ended undecided
  between this and deferring mutations.
- **What changed is the coexistence objection, not the
  evidence.** The risk section concluded the coupling was
  "unenforceable, probably fine" because a `jj --version`
  check cannot answer whether two versions are compatible.
  That evaluates it as a compatibility oracle. As a guard on
  our own writes it fits the actual risk direction: the
  dangerous case is an old jj reading an op written by a
  newer jj-lib, and refusing to write on a mismatch closes
  exactly that. The safe direction, a newer jj reading our
  older op, is something jj must support anyway, since the
  user's own older jj wrote into that repo first.
  - **Half superseded 2026-07-31.** The conclusion stands;
    the "safe direction" sentence does not. See
    [why equality, and why at startup](#why-equality-and-why-at-startup).
    Kept as written because this section records what was
    decided at open, not what we believe now.
- **The gate lands at `-5`, before the mutations at `-6`**, so
  mutations arrive in a repo that already refuses on
  mismatch. The cost is one commit whose check guards nothing
  yet, which reads oddly in isolation; folding it into `-6`
  would avoid that but make one rung do two things.
  - renumbered 2026-07-31 by the `-2` split; the decision is
    unchanged, the gate still lands one rung ahead of the
    mutations
- **Deferring mutations was not free**, which is what settled
  it. The trapezoid reshape at 0.79.0 is a `jj rebase`, so
  "reads only" would leave that cycle either spawning or
  waiting.

### The 0.43 bump was a preview of the cost

The `build: bump jj-lib to 0.43` interlude that immediately
precedes this cycle is worth reading as evidence rather than
housekeeping. Two releases of a pre-1.0 library produced two
breaks of different kinds: `use_glob_by_default` disappearing
from `RevsetParseContext`, which the compiler caught, and the
default revset string-pattern kind moving from substring to
glob, which it structurally could not.

We think the useful lesson is narrow and worth keeping in
front of us for the rest of this cycle: a green build after a
jj-lib bump is not evidence that the bump preserved behavior.
That is the treadmill cost the mutation decision accepts, and
it is paid on every bump, not only on the ones that touch the
op store.

### 0.78.0-0 refactor: jj-lib migration opening

Retitled 2026-08-02 (originally `chore: open the jj-lib migration cycle`): the bookend-titles
convention arrived from iiac-perf mid-cycle, and with the whole ladder still on the unmerged
`refactor-vc-x1` branch the retro-apply cost one coordinated re-describe (trailer hand-copied
per the rule), one force-push, and re-recording rungs `-0..-5`; main was never touched.

Preparation only. The `## Todo` entry moved into
`## In Progress` as this cycle block, carrying the ladder and
the version-gate design this session worked out, and
`fix-todo` renumbered the 19 entries left behind.

Two stale references surfaced while renumbering and were
fixed in the same commit: the `0.79.0` program rung pointed
at "Todo #2", a positional number the renumber invalidated,
and now names the entry by title as the convention requires;
and the program ladder still said `0.78.0` bases on `0.77.2`,
which two interludes had since made wrong.

The narrative lived in `TODO.md > ## In Progress` while the cycle ran, per the one-home
convention adopted at `0.77.0-2`, and moved here wholesale at the `0.78.0` close-out. This
commit first did it the old way, because
[Chores conventions](/agent-data/notes.md#chores-conventions) then still described the
superseded per-commit build-up (overridden for this repo in [custom.md](/custom.md)); the
`## Todo` entry "One home for a cycle's narrative" is what closes that gap template-wide.

### 0.78.0-1 docs: adopt universal AGENTS

Inserted after the cycle opened. The AGENTS restructure
(short universal AGENTS.md + `agent-data/` satellites +
`custom.md` as the one agent-editable instruction file) is
proposed in vc-x1-work-repo-template as
`AGENTS-vc-x1-f5-20260730.md` with a `-notes.md` companion,
and this repo adopts it now to dogfood it:

- the local copy is authoritative during the dogfood window;
  the template snapshot is frozen for discussion
- promotion back to the template happens en masse or
  incrementally as the local copy proves out
- findings land in `custom.md`'s dogfood log; semantic rule
  changes wait for that evidence

Semantics-preserving by design: rules keep their current
meaning, only the organization changes (checklists at the
moment of action, rationale behind them, project specifics
in `custom.md`). We think that keeps any adherence change
attributable to the structure, which is the hypothesis under
test.

### 0.78.0-2 feat: report jj-lib and jj-data versions

`--version` reports three versions that answer different
questions, so the policy at `-3` can be written against
measured output instead of inference:

- ours, `CARGO_PKG_VERSION`, compile time
- jj-lib's, `JJ_LIB_VERSION`, resolved from `Cargo.lock` by a
  new `build.rs`
- the data's, read through jj-lib's public accessors only, one
  `jj-data` line per repo

The report is its own `version` subcommand, answered before
`Context::load` so it still works when the workspace is the
thing that is broken. That is what let the gate at `-4` name it
as its one exception: "the `version` subcommand gathers
`jj-data` lines only after the gate passes" is a sentence the
policy can hold; "the bare invocation with no subcommand" was
not. (Written when the gate was still `-5`.)

Version output now rides along with every run, so any captured
output says which version produced it. Stream by who asked:

- no flag: stderr. Provenance that was not asked for must not
  land in the stream `chid`, `desc`, `list` and `show` emit
  data on, or piping them breaks.
- `-V` / `--version`: the banner on stdout. An explicit request
  makes it data, capturable alongside the command's own output.
- `-VV`: the full report on stdout, then the subcommand runs.
  This is the one thing the subcommand cannot do, and what a
  bug report wants stamped on top of a real command's output.
- `--no-banner` silences the ambient one; `-V` still prints,
  since asking outranks suppressing.

Counted like the `-v` / `-vv` this CLI already teaches, so
version detail scales the way verbosity does and needs no
separate explanation.

We rejected `-V` versus `--version` as the terse/full split.
Those two being aliases is close to a universal CLI
convention, and this project prefers invariants that can be
stated in one line.

The ambient banner uses `eprintln!` rather than the logger,
because `CliLogger` routes by level and puts info on stdout.
The cost is that `--log` does not capture it.

The `build.rs` is `-5`'s mechanism arriving three rungs early,
which was not the plan: jj-lib exports no version constant and
no accessor for one, so printing the version at all requires
resolving it from the lock. `-5` inherits it and adds only the
comparison and the refusal. The lock is read from
`$CARGO_MANIFEST_DIR` rather than by walking ancestors,
because we are not a workspace and a walk can bind a sibling
project's lock, which is worse than failing.

`data_version` stops at `Workspace::load` and never calls
`load_at_head`, since resolving op heads can merge divergent
ones, which is a write. A version report must not mutate what
it reports on.

We think one wrong turn is worth recording. The `build.rs`
parser first shipped with unit tests, which never ran: cargo
does not compile a build script as a test target. They were
the exact defect class the `## Todo` entry "A committed
cycle-check runner" describes, a mechanism that looks like a
guarantee and is not, so they were deleted. The parser is
covered instead by a test asserting the compiled-in version
matches `Cargo.lock`, scanned deliberately unlike `build.rs`
scans it, so a parser that drifted onto the wrong
`[[package]]` block cannot agree with itself.

### 0.78.0-3 docs: jj-lib version coupling policy

The policy proper goes to `notes/`, beside the risk section it
supersedes, and not into chores at close-out. Chores is
append-mostly history organized by when work happened; this
rule governs what the tool does from `-5` onward, and someone
asking "why does vc-x1 refuse to run" should not have to know
which cycle produced the answer. `TODO.md` keeps the
narrative, chores gets it verbatim, and the two cross-link
rather than restate, the same division `notes.md` draws
between a commit body and its chores section.

The rule lands as [jj-version-policy.md](../jj-version-policy.md),
a topic file rather than a section of the plan file: the plan
file becomes historical when the refactor program ends, and the
gate ships in the product. `notes/README.md` gains the general
form of that split, since it is not specific to this rule.

Three recorded conclusions retire together, since leaving any
one would have the notes arguing with themselves. Two were
annotated in place at `-2` as they were found; this rung
finishes the third and links all of them to the policy:

- the risk section's "a `jj --version` check does not work",
  which judged the check as a compatibility oracle. Its
  findings stand and are what the policy rests on; only the
  closing verdict is superseded, so the section is annotated
  rather than rewritten.
- this ladder's "refuse on the write path only" bullet, rewritten
  at `-2` when the startup gate was decided
- the "Decisions at cycle open" claim that a newer jj reading
  our older op is the safe direction, marked half-superseded at
  `-2`

A fourth surfaced while writing the policy: the `-5` carve-out
still listed `--version` among the commands that never open a
repo. That stopped being true at `-2`, when the report grew
`jj-data` lines. `-V` alone still qualifies; `version` and `-VV`
do not, and are ordered around the gate instead.

### 0.78.0-4 feat: jj-lib version gate

Implements [the policy](../jj-version-policy.md) written at
`-3`; the design questions were settled there and this rung
re-decides none of them.

Moved ahead of the reads rung when the ladder's own reason for
its position turned out to be wrong. "The gate lands before the
mutations, so mutations arrive in a repo that already refuses"
assumed the exposure arrives with the mutations. It did not:
`common::load_repo` has called `load_at_head` since the facade
moved to jj-lib, and it backs `chid`, `desc`, `list` and `show`.
Op-head merging and index reindexing have been happening
in-process, ungated, for several cycles. The hole is open now,
and `-5` would have widened it first.

The gate applies to every subcommand, with no list of exempt
ones. It first shipped in this rung's working copy with a
carve-out: an `opens_repo` method, an exhaustive `match` on
`Commands` with no wildcard arm, so a new subcommand would fail
to compile until someone picked a side. That was dropped at
review, unpushed, on an argument that holds:

- the match enforces enumeration, not classification. A new
  subcommand does force a decision; an existing one that grows a
  repo read later stays classified as safe, silently, and nothing
  fails.
- the policy called the carve-out "provably does not open a
  repo". The actual proof was a grep of five modules for
  `load_repo` and friends, which is a point-in-time observation
  wearing the word "provably".
- two of the costs claimed for keeping the list were not real.
  `--help` exits inside clap's `e.exit()` during parse, and
  completion exits inside `CompleteEnv::complete()` on main's
  first line, so neither ever reached the gate.

What remains is one rule with no list to maintain: every
subcommand except `version` refuses on a mismatch. The cost is
that a markdown linter needs a version-matched jj, which is what
the per-invocation override is for.

`jj -V` is spawned once per process and cached, not once per
operation. Ironic in the cycle that ends spawning, and
unavoidable: it is a spawn on their side of the boundary.

Three failures, not one, because the fix differs each time: `jj`
absent from `$PATH`, `jj -V` unreadable, and a genuine mismatch.
Only the third mentions `--allow-jj-mismatch`, since the override
is meaningless for the other two.

A measured over-strictness landed in the policy's known holes on
the way: `0.42` and `0.43` store the same bytes, so a vc-x1
linking `0.42` would refuse against `jj 0.43` while being safe to
run. The route to that finding is worth more than the finding.
The first evidence offered was that the `.proto` files are
identical, and the conclusion drawn from it, that the data cannot
have changed, was wrong: a fixed schema still permits the same
fields meaning different things, non-protobuf state like the
index segments moving, and content hashing changing so the same
data lands under different ids. Ruling those out took a source
diff across four files. Nobody will repeat that on every bump,
which argues *for* the blunt gate, and it demotes the schema
fingerprint from "makes an override provably safe" to "catches
one narrow class at build time".

The refusal path is what the test drives, with a fake `jj` first
on `PATH` reporting `0.99.0`: the real pair matches here, so a
test that only exercised the match would pin nothing. It checks
all four consequences at once, that a repo-opening command
refuses and names the override, that the override works, that a
markdown linter is unaffected, and that `version` still answers
while withholding the `jj-data` lines.

### What the data records about itself

Measured 2026-07-31 against `jj 0.43.0`, both repos identical:
`commit=git op=simple_op_store op-heads=simple_op_heads_store
index=default submodule=default working-copy=local`.

Every value is an identity, never a version. They are the
`.jj/repo/<backend>/type` files that
`RepoLoader::init_from_file_system` reads, and they are all
jj-lib exposes: 0.43 has no public version constant and no
accessor for one. This is the risk section's "the op store has
no version stamp", now observed rather than inferred from
reading jj-lib's source.

Nor can the stamp be recovered from the data itself. A proto3
message serializes to (field number, wire type) keys plus
payload bytes: no message name, no schema id, no field names.
That absence is exactly what makes an unknown field skippable,
so the property that lets jj evolve the format is the same
property that makes the evolution undetectable. Three reasons
sniffing the tags present cannot substitute:

- proto3 has no presence for scalars, so a field the writer
  left unset and a field the writer never heard of are the
  same bytes
- a new tag appears only once a newer jj populates it, so a
  newer jj that has the field but has not used it is
  byte-identical to an older one
- prost does not surface unknown tags at all. The derived
  `merge_field` routes an unrecognized tag to
  `encoding::skip_field`, which advances past it and discards
  it; there is no unknown-field set to inspect

The last point is the one that matters beyond detection, and
it is why a compile-time schema fingerprint would not help
either: equality needs two operands, and the data supplies
none. A stamp we wrote ourselves would record only what we
last wrote, would go stale the moment the user's jj wrote
without updating it, and would never be read by the old jj
that is the endangered party. `jj -V` stays the second operand
not because a version is the right thing to compare, but
because it is the only thing the other side emits. It is a
proxy for schema identity and the policy should say so.

### Why equality, and why at startup

Two findings from the 2026-07-31 session, both of which
retire text written at cycle open.

**The loss is symmetric, so no direction is safe.** Because
prost discards unknown tags and the next writer serializes
from the decoded struct, an old jj and a new jj can each
destroy what the other wrote:

- ours newer than theirs: we write fields their jj skips, and
  their next write drops them
- ours older than theirs: they wrote fields we skip, and our
  next write drops them

Content addressing means the original blob survives under its
own id, so this is loss of current state rather than
destruction of history. The recorded rationale exempted the
second direction on the grounds that jj must support reading
its own older ops. That holds for jj reading. It stops holding
once our writes are in the picture, which they are. So the
test is `!=`, not an ordering comparison, and the reason is
better than the one we first wrote down.

Equality is also the honest response to an unanswerable
question. We cannot compute compatibility, because the data
publishes no schema and jj publishes no stability policy. An
unequal pair is not "incompatible", it is "unknown", and the
only correct response to unknown on a path that writes is
stop.

**Reads write, so the gate cannot be scoped to writes.**
"Read" in jj-lib means "does not write anything the caller
asked for". Three paths we already know:

- `load_at_head` resolves op heads and, when several have
  diverged, merges them and writes a new operation
- the index self-heals: a stale or format-mismatched index
  makes `DefaultIndexStore` reindex and write new segment
  files. `COMMIT_INDEX_SEGMENT_FILE_FORMAT_VERSION` is a
  compile-time constant, so a mismatched pair does not merely
  risk this, it guarantees churn in both directions
- any `@`-relative read needs a working-copy snapshot, which
  writes `tree_state` and can create a commit; this is
  already why `-5` defers those reads to `-6`

So the gate fires at startup and stops before anything opens a
repo. What it guarantees is narrow and should be stated
narrowly: not "no old jj will misread our op", only "we never
run against a jj differing from the one we can see". The
`$PATH` sample objection survives intact, since an editor
integration running a different jj is outside the gate.

Known holes and costs, recorded now so they are not
rediscovered at `-4`:

- `jj -V` prints `jj 0.43.0-<40-hex>`, and we compare triples.
  A jj built from git between releases claims the release
  triple while being arbitrarily far ahead of it, and a hash
  cannot be mapped to a schema, so that hole stays open.
- version equality is coarser than schema equality, so two
  releases with an identical op-store proto still trip the
  gate. Relaxing that safely needs a schema fingerprint:
  hash jj-lib's shipped `.proto` files at build time and fail
  the build when a bump changes them, turning "a green build
  after a bump is not evidence" into a red build for the
  op-store-shape class. It catches nothing semantic, so the
  0.43 glob-versus-substring change would still pass, and
  cargo hands a build script only its own
  `CARGO_MANIFEST_DIR`, so locating a dependency's `.proto`
  needs `cargo metadata` or the registry layout. Wants a
  `## Todo` entry of its own; not this cycle.
- a jj release stops vc-x1 entirely until the lock is bumped
  and revalidated, not just its writes. The override flag is
  what keeps the tool usable that day, so it is load-bearing
  rather than a nicety, and it is a per-invocation flag: a
  config key gets set once during a frustrating afternoon and
  then silently protects nothing.

The pedantry is deliberate and provisional. The measurement
that would let it relax: hash every file under `.jj/` in both
repos, run one command against a deliberately mismatched
jj-lib, hash again, diff. The index case should light up
immediately, which is itself the evidence for keeping the gate
broad; anything genuinely inert becomes a candidate for
narrowing later, backed by a measurement instead of an
assumption.

### 0.78.0-5 refactor: jj-lib reads

The facade's internals flip: in-process through jj-lib is now
the default read path, and spawning is the carve-out rather
than the mechanism. The seam the DRY-facade cycle bought is
what made this a one-module change: every caller kept its
signature, so push, squash-push, sync, init and the registry
checks moved without being edited.

- The routing is per revset, not per call site, because the
  revs are runtime values: squash-push's source/target default
  to `@`/`@-` but are user-overridable, so no static split of
  the call sites exists. `references_working_copy` decides: a
  `@` is working-copy syntax (`@`, `@-`, `ws@`) unless it has
  symbol characters on both sides, the remote-bookmark form
  (`name@remote`).
- Working-copy revsets keep the spawn path on purpose (the
  ladder's standing caveat): the CLI auto-snapshots, so "is
  `@` empty right now?" answers about the filesystem, while an
  in-process `load_at_head` would answer about the last
  snapshot. Those reads move at `-6` with the mutation lift.
- The raw `log(repo, rev, template)` primitive is gone from
  the facade surface: jj-lib has no template engine (templates
  live in jj-cli), so its one caller, sync's bookmark-heads
  probe, became the typed `cids_short_of`.
- `rev_exists` and sync's `try_commit_id` now classify the
  unresolvable-revision error through one helper,
  `is_no_such_revision`: a typed
  `RevsetResolutionError::NoSuchRevision` downcast on the
  in-process path, the old stderr substrings on the spawn
  path. A first taste of the `-8` principle that
  classification is by variant, not wording.
- Parity is pinned by tests: the in-process accessors are
  compared against spawned `jj log` templates on a fixture
  repo, with revs pinned to concrete commit ids so the tests
  stay on the in-process path.
- `bookmark_list` / `bookmark_list_all` still spawn: their
  consumers parse the CLI listing textually. They are not
  `jj log` templates, so not this rung's scope; where they
  land (a typed view query, or a rider on `-6`) is an open
  ladder question.

### 0.78.0-6 refactor: jj-lib mutations

The workspace/transaction/op-store lift. A new `jj::session`
module's `RepoSession` is the CLI's `WorkspaceCommandHelper`
plumbing reduced to what the facade's verbs need, written
against jj 0.43's `cli_util.rs` as the reference; the facade
grows the five publish-path verbs on top, and the `@`-read
carve-out from `-5` closes. Named for what it is (an open ->
mutate -> finish working session with one repo) and
backend-neutral on purpose; "engine", the working title, is
machinery you start, not a thing you open per operation.

- The session is three pieces: a settings loader replicating
  the CLI's config discovery (`/etc/jj`, `$JJ_CONFIG`, user
  files, `.jj/repo/config.toml`, `JJ_USER`/`JJ_EMAIL`), the
  snapshot cycle (git HEAD/refs import around the
  working-copy snapshot, under the CLI's own
  `git_import_export.lock`), and transaction finish (git HEAD
  reset + ref export, op commit, working-copy update).
  Colocation drives the git halves; these repos are
  colocated, so that fidelity is the bulk of the session
  module.
- Verbs: `commit`, `describe`, `bookmark_set`,
  `git_push_bookmark`, `git_fetch`. Call sites swapped in
  push, squash-push, sync, fix-desc, init, and repo_utils.
  The ladder's "bookmark track" had no call site to lift:
  jj-lib's `push_refs` marks pushed bookmarks tracked, which
  is the side effect init's no-`--allow-new` design relied on
  in the spawned form too.
- The `@`-read deferral resolves as predicted at `-5`:
  `references_working_copy` survives as the trigger, now
  routing to snapshot-then-read (`repo_for_read`) instead of
  to a spawn; `log_spawn` is deleted and
  `is_no_such_revision` drops its stderr-wording fallback,
  leaving only the typed `NoSuchRevision` downcast.
- Fetch returns typed changed-bookmark lines; sync's
  stderr-capture wrapper (`fetch_silent`) now just relabels
  them, keeping the clean-case silence it existed for.
- Documented deviations from the CLI, each at its function:
  no immutability preflights (rewrite targets are validated
  by callers), a small defaults layer for three keys whose
  defaults ship in the CLI's config files rather than
  jj-lib's, the auto-track map driven by
  `git.auto-local-bookmark` alone, and the fetch expression
  pinned to all-branches rather than the remote's refspec
  config.
- Still spawning after this rung: `jj squash` (squash-push),
  `jj new` / `jj rebase` / `jj op log` / `jj op restore`
  (sync, revert), `jj git clone` (clone), `jj diff --stat`
  (push preview), init's `gh` / `git init --bare` (the `-8`
  gix rider) / `jj git init --colocate` / `jj git remote
  add`, the facade's two bookmark listings, and the gate's
  `jj -V`, which is a spawn by definition. The migration
  stage's "removes spawning entirely" now reads as this
  cycle's five verbs plus a remainder with named homes.
- Validation is the existing integration suites now running
  entirely through `RepoSession` (init, push, squash-push, sync
  fixtures), plus facade tests pinning colocated-git export:
  after in-process commit / bookmark-set / describe, `git
  rev-parse` sees the same commit ids jj reports.

### 0.78.0-7 refactor: context-owned repo sessions

`Context` grows the session map the `-6` review designed:
lazily-opened `RepoSession`s keyed by canonicalized repo path,
opened on first use by `Context::session` and reused for the
rest of the invocation. Has-a, never is-a: an invocation
touches 0..N repos, and a repo-less `version` never opens one.
The five verbs move from facade fns onto `RepoSession` as
methods; the facade keeps one-shot wrappers for context-less
callers.

- `SubcommandRunner::run` (and `dispatch`) take
  `&mut Context`: the sessions are exclusive mutable state,
  and the borrow checker enforcing one live session borrow at
  a time is what a `RefCell` would trade for runtime panics.
  The fifteen non-consumer subcommands change signature only.
- Verbs as `RepoSession` methods: `commit`, `describe`,
  `bookmark_set`, `git_push_bookmark`, `git_fetch`, plus the
  `one_commit` resolver and `complete_newline`.
  `DebugCallback` goes private to the session module.
- One-shot wrappers stay for the context-less callers:
  fix-desc (`describe`), init (`bookmark_set`,
  `git_push_bookmark`), repo_utils (`commit`, `describe`).
  `git_fetch`'s only caller is sync, context-ful, so its
  wrapper is deleted rather than kept unused.
- `RepoSession::snapshot` now reloads at the op-store head
  for every repo, not only colocated ones: a session outlives
  single verbs in a `Context`, and a spawned `jj` (squash,
  new, rebase, op restore) may commit operations between
  verbs. Reuse skips only the open (settings + workspace
  load); freshness is per-verb, unchanged.
- Consumers: push threads `ctx` through `mutate` and the four
  mutating stages, and its `squash-push-bot` stage passes the
  same `ctx` into squash-push, so one bot-repo session serves
  the whole run; squash-push and sync take `ctx` for their
  verb sites (`fetch_silent`, `act_on_state`). Reads stay
  one-shot facade fns.
- Tests: `test_helpers::test_ctx()` builds a
  default-user-config `Context`; the push / sync /
  squash-push integration tests pass a fresh one per op call,
  matching the production one-context-per-invocation shape.

### 0.78.0-8 fix: jj-lib index-lock retry

The bugs.md #1 fix, landing on the `-7` frame as ordered: gix
gives `.git/index.lock` a single attempt, and a git-aware
watcher can hold it exactly when a mutation resets the index,
so the session retries the colocated git half itself. Plus the
planned rider: init's `git init --bare` becomes a gix call,
the last `git` spawn in init.

- `retry_git_lock` wraps the two colocated git blocks
  (`finish_tx`'s HEAD/index reset + ref export, the snapshot's
  intent-to-add + ref export), both strictly before the
  transaction commit, so a retried closure never doubles an
  op-store write.
- `is_lock_contention` classifies by type, never by message
  substring: walk the source chain, downcast each link to
  `gix::lock::acquire::Error`. Never-retryable failures (a
  missing git binary via `SpawnInPath`, an old git via
  `UnsupportedGitOption`) can never carry that type, so they
  classify false without being named, where a broader "retry
  git errors" rule would loop on them forever.
- Backoff: 5 attempts, 25 ms doubling, about 375 ms of
  waiting in total. The observed holds are watcher-brief;
  anything longer surfaces as the same error as before.
- gix becomes a direct dependency (no features of our own, so
  it resolves to exactly jj-lib's 0.85): the downcast needs
  type identity with the errors jj-lib returns.
- The rider: `init_bare_main` uses
  `ThreadSafeRepository::init_opts` with an in-memory
  `init.defaultBranch=main` override, standing in for the
  spawned form's `--initial-branch=main` so the user's git
  config cannot steer which branch vc-x1 publishes to.
- Tests: classifier and retry-loop units (including the
  give-up-after-budget and pass-through cases), a planted
  `.git/index.lock` released mid-backoff by a thread proving
  a mutation survives transient contention, and a bare-init
  test pinning HEAD to `refs/heads/main`.

### 0.78.0-9 docs: sync pin set and adopt new conventions

The schema-sync rung: the amendments and conventions from the iiac-perf collaboration arrive,
this cycle's records are brought under them before the close-out writes the chores section, and
the dogfooded conventions graduate into the pinned set template-side. Inputs: the template
mailbox (`../vc-x1-template/messages/vc-x1.md`, 2026-07-31) and iiac-perf's custom.md
conventions (2026-08-02).

- tier-1 graduation authored template-side as `AGENTS-vc-x1-f5-20260802-snapshot/`, a new
  directory keeping 0730 frozen (the template repo carries no commits, so an in-place
  amendment would have destroyed the adoption record)
  - graduates: write-to-full-width, cycle bookend titles, the checklist's close-the-records
    step, the mailbox check at acquaint
  - plus two prose.md consistency fixes: the stale chores `Commits:` bullet, and the
    "prohibition is on authoring" rewording
  - `work/` payload, snapshots.md, and both mailboxes updated to match
- pin set re-copied from the 0802 snapshot and verified byte-identical, which also lands the
  0730 amendments this rung originally targeted: rule 0 + hard-rules-first, generic "the
  template repository" pin lines, the chores as-built ladder, the chores `## Table of Contents`
- cycle-protocol.md's "Chores sections" and "Commits backfill" amended to match
  - the satellites defer to the protocol, so until this rung the two disagreed
- custom.md brought to the post-graduation shape
  - the one-home override reconciled with the now-universal rung backfill
  - the graduated conventions replaced by their project parameters (mailbox member/path, the
    0.78.0 bookend adoption boundary)
  - the dogfood entry records the whole exchange
- the bookend retro-apply: `-0` re-described to `refactor: jj-lib migration opening` and the
  branch force-pushed (see the `-0` detail)
  - rungs `-0..-5` re-recorded, `-6..-8` backfilled
- chores-15 sanitized (50 word-level conversions, specimens kept), reflowed to full width, and
  given its ToC
- README.md reflowed (word-identical, its four em dashes are transcribed config samples
  deferred to the source-sweep Todo); notes/README.md sanitized, reflowed, and refreshed
- bugs.md gains the mailbox's init step-order report and iiac-perf's `push --body`
  leading-hyphen find
- the backlog gains the mailbox's init CLI ideas
- tier 2 staged for iiac-perf's read: one-home, cycle-protocol.md into the byte-identical set,
  every-commit-belongs-to-a-cycle, scope-based version advancement

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
[11]: https://github.com/winksaville/vc-x1/commit/0cf200b9b3eb "0cf200b9b3eb2ad652b99e518edcdfe69b657075"
[12]: https://github.com/winksaville/vc-x1/commit/343eb2ed38bc "343eb2ed38bcf3046bfd0a229388ddcccbb90cb9"
[13]: https://github.com/winksaville/vc-x1/commit/307a4f57fd1b "307a4f57fd1b8ec0532d359ec5f54f82fa29847a"
[14]: https://github.com/winksaville/vc-x1/commit/fabcbed27c3e "fabcbed27c3ef3db3d78722a68628589cfa43dc3"
[15]: https://github.com/winksaville/vc-x1/commit/b5ffa439b23f "b5ffa439b23f817325a1dfd67c7bde61b66f39a0"
[16]: https://github.com/winksaville/vc-x1/commit/9b08f953ead8 "9b08f953ead84ebc178df3ce01b5a084bdd3b563"
[17]: https://github.com/winksaville/vc-x1/commit/8254bbdb7e08 "8254bbdb7e08606393f9ccd4a83ffd93e6aa3501"
[18]: https://github.com/winksaville/vc-x1/commit/738f5f219d42 "738f5f219d429393cba84811e426dbd0844a5062"
[19]: https://github.com/winksaville/vc-x1/commit/bbcfd0ea6985 "bbcfd0ea698529b539f681e7243ac5fbae70ab83"
[20]: https://github.com/winksaville/vc-x1/commit/5faf428dd7d2 "5faf428dd7d2f478c60f968d628276ddd049db73"
[21]: https://github.com/winksaville/vc-x1/commit/54897e0919dc "54897e0919dc6f51d1eb64954a838e0623729e9b"
[22]: https://github.com/winksaville/vc-x1/commit/99f45fcb87d9 "99f45fcb87d901c00b0c650e520cb98b30e74208"
