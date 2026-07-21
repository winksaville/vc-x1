# Chores-14

Continuation of `chores-13.md` (closed at `0.72.x`, parked on
the `support-trapezoid-commits` bookmark). This file covers
the jj refactor program
([refactor-20260716.md](../refactor-20260716.md)) from
`0.73.0` onward, worked on the `refactor-vc-x1` bookmark
while `main` parks at the `0.71.0` tip.

Reference numbering is file-local — see
[`AGENTS.md`](../../AGENTS.md#reference-numbering); chores-14
starts at `[1]`.

## refactor: DRY jj facade

Commits: see [As-built ladder](#as-built-ladder)

~30 call sites hand-roll `run("jj", ["log", "-r", <rev>,
"--no-graph", "-T", <template>, "-R", <repo>])` and each
module has quietly grown a private wrapper — spotted at
0.69.0-2, where `jj_rev_exists` read as "first of its kind"
but was the Nth reinvention. One typed facade module ends
the reinvention; first stage of the refactor program.

### As-built ladder

- [[1]] 0.73.0-0 chore: open jj facade cycle
  - restore the five notes files from
    `support-trapezoid-commits` (TODO.md, todo-backlog,
    forks-multi-user, refactor doc, chores-13) so the shared
    bookkeeping rides the refactor line
  - re-plan after the pivot: `## In Progress` → the DRY
    facade ladder; the trapezoid `--merge [<base>]` design
    folded into the refactor doc's trapezoid stage; new
    "stateless push" stage (retire the push state file —
    from the extraction review's "why does `Stage` live in
    a file?")
  - shared-doc-sync todos updated: vc-template-x1 synced
    2026-07-20; family grows to five (adds zc-msg-x1,
    tprobe)
  - version 0.73.0-0; backfill the 0.72.0-1 ref [57] in
    chores-13; open this file
