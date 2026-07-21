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
- [[2]] 0.73.0-1 refactor: jj facade query module
  - new `src/jj.rs`: the `log(repo, rev, template)` primitive
    plus typed helpers — `matches` / `rev_exists`, `chid_of`,
    `cid_of` / `cid_short_of`, `desc_of`, `is_empty` — every
    read-only `jj log -T` spawn now goes through it
  - folded: `squash_push::{jj_rev_exists, jj_commit_id,
    rev_is_empty_undescribed}` (the last two deleted, the
    empty-undescribed check rebuilt on `is_empty` +
    `desc_of`), `push::{get_change_id, jj_log_empty}`, the
    sanity verifiers' inline template blocks, `init::jj_chid`,
    `sync::{commit_id, revset_nonempty}` + the
    `commit_ids_of_bookmark` template, and
    `common::bookmark_publish_state`'s commit-id closure
  - `rev_exists` folds jj's unresolvable-revision errors to
    `false` (the `try_commit_id` patterns); the push
    stale-state checks keep their any-failure-is-stale
    behavior via `.unwrap_or(false)` at the call sites
- [[3]] 0.73.0-2 refactor: jj facade tracking parse
  - the two tracking parsers unify on one listing:
    `jj::bookmark_list` / `bookmark_list_all` join the
    facade, and `common::find_tracked_remote` (positive:
    indented `@remote` entry, synced or divergent form)
    sits beside `find_non_tracking_remote` (negative:
    column-0 ref) over the same `-a` output
  - `main::bm_track_one` drops its raw
    `std::process::Command` spawn + private parse — now a
    thin wrapper; `verify_tracking` and squash-push's
    bookmark-existence check use the facade
  - five `find_tracked_remote` unit tests
  - rider: TODO.md In Progress ladder rungs now carry
    prepended `[[N]]` commit refs (backfilled once pushed,
    like the chores As-built rungs); intro template updated
- [[4]] 0.73.0-3 refactor: jj facade ochid parse
  - `desc_helpers::extract_ochids` (column-0 trailers, all,
    in order, values trimmed) becomes the crate's one
    string-level ochid parser; `extract_ochid_from_desc` is
    now `.pop()` over it (the *last* trailer — matching
    `common::extract_ochid`'s previous reverse scan) and
    `common::extract_ochid` wraps that over
    `commit.description()`
  - `squash_push`'s local `extract_ochids` deleted; its
    three parser tests move to desc_helpers, plus a new
    multi-ochid last-wins test
  - behavior tightened: the single-value accessors are now
    column-0-only — an indented `ochid:` mention is prose,
    not a trailer (previously the trimmed scans matched it)

[1]: https://github.com/winksaville/vc-x1/commit/f761e89092df "f761e89092dfbb82e8ab355d6e5a058e77b07e23"
[2]: https://github.com/winksaville/vc-x1/commit/47e5075b90da "47e5075b90daa5e9b24fa7c93a5814a2eee0f03f"
[3]: https://github.com/winksaville/vc-x1/commit/5a61ebdcbac8 "5a61ebdcbac872eac03d6b70141030217be1f850"
