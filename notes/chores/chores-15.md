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

Commits:

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

# References

[1]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[2]: https://github.com/winksaville/vc-x1/commit/2424e14f858d "2424e14f858d010e5c07e8821149a114b3d3dda5"
[3]: https://github.com/winksaville/vc-x1/commit/4898d93e4172 "4898d93e41720070cddb995bfd4e53ffc38ccb88"
[4]: https://github.com/winksaville/vc-x1/commit/ab3a07d4903b "ab3a07d4903bbe6ae7cec5490f5edd622161c72e"
[5]: https://github.com/winksaville/vc-x1/commit/846b5eee5b98 "846b5eee5b988b0cd8887559a0fba3397155ee19"
[6]: https://github.com/winksaville/vc-x1/commit/66aa3f67d4b1 "66aa3f67d4b1308bb08388ccb929fc27967e8259"
[7]: https://github.com/winksaville/vc-x1/commit/9d6f7c0b0f05 "9d6f7c0b0f05ae74dd7100d457b92b72d913404f"
