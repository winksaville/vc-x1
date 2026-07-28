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

Commits:

`push.rs` (~1.5k lines) holds the `Stage` machine, TOML state
persistence, eight stage bodies, two sanity verifiers, and the
interactive gates in one file. The state file is where the
defects come from: bugs.md #3 — the rollback rewinds the
*repos* but not the *state*, so the rerun skipped the commit
stages and republished a previous bot commit — and both sanity
verifiers exist largely to defend against that staleness.
Retiring it, deriving the resume point from repo reality as
standalone `squash-push` already does, deletes the class
rather than patching it. Fifth cycle of the refactor program.
Decisions at cycle open (2026-07-28):

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

### As-built ladder

- [[3]] 0.77.0-0 chore: open stateless push cycle
  - version 0.77.0-0; the stage picked into `## In Progress`
    as the program ladder's current `####` rung with a
    five-rung ladder; this section opened
  - rider: 0.76.1 `Commits:` backfill ([[2]])
  - rider: `## Done` retirement sweep into done.md
  - the push itself hit bugs.md #1 twice before landing —
    recorded there as the fourth and fifth occurrences, and
    as #3's second occurrence (clean rollback, poisoned
    state file, `--restart` the safe rerun)
- [[N]] 0.77.0-1 refactor: extract push/state.rs
  - `Stage`, `StateLayout` / `resolve_state_layout`,
    `PushState`, `STATE_FORMAT_VERSION`, the state-dir /
    state-file defaults, and the escape helpers move to
    `src/push/state.rs`; push.rs 1480 → 1101 lines with no
    behavior change
  - the parked 0.72.0-1 extraction was **reference, not
    base**: `support-trapezoid-commits` turns out to be
    published (`@origin`), so rebasing it would rewrite a
    pushed commit. Its boundary was reused (the same item
    set, plus `STATE_FORMAT_VERSION` which it left behind at
    version 1) and the extraction redone against the current
    file
  - `escape_multiline` / `unescape_multiline` stay private,
    so their round-trip test moves into `state.rs`'s own
    `#[cfg(test)] mod tests` rather than widening visibility
    for a test's benefit; the remaining state tests reach
    `STATE_FORMAT_VERSION` through push.rs's `#[cfg(test)]`
    re-export beside `DEFAULT_STATE_DIR` / `_FILE` (which
    `config_schema`'s tests already used)
  - the module doc records what the stateless-push rungs
    delete, so the next reader knows the file is scaffolding
    with a scheduled end

# References

[1]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[2]: https://github.com/winksaville/vc-x1/commit/2424e14f858d "2424e14f858d010e5c07e8821149a114b3d3dda5"
[3]: https://github.com/winksaville/vc-x1/commit/4898d93e4172 "4898d93e41720070cddb995bfd4e53ffc38ccb88"
