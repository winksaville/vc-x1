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

# References

[1]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[2]: https://github.com/winksaville/vc-x1/commit/2424e14f858d "2424e14f858d010e5c07e8821149a114b3d3dda5"
