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
- [[5]] 0.73.0-4 test: jj facade fixture helpers
  - `test_helpers::jj_ok` (which anticipated this migration
    in its doc note) gains `cid` / `chid` / `description`;
    the near-identical private copies in
    `push/integration_tests.rs` and
    `sync/integration_tests.rs` are deleted (push keeps a
    local `desc_first_line` built on `jj_ok`)
  - deviation from the plan bullet: `tests/cli_sync.rs`
    lives in its own test crate and can't reach the
    binary's `test_helpers`, and its variant needs `HOME`
    injection — its `jj` / `cid` move to the crate's shared
    `tests/common/mod.rs` instead, with the why documented
  - the shared helpers spawn directly rather than calling
    `crate::jj`, so test inspection stays independent of
    the facade under test
- [[6]] 0.73.0 refactor: DRY jj facade (close-out)
  - ARCHITECTURE.md: `src/jj.rs` joins the module
    inventory; the stale subcommand enumeration corrected
    (`finalize` dropped; `squash_push`, `revert`,
    `validate_bot`, `config_cmd`, `bot_session` /
    `transcript`, `sync/state` added)
  - per-stage refactor Todos: each remaining program stage
    is a short ranked entry pointing at its stage section
    (por → dual to todo-backlog); the umbrella entry
    shrinks to the program pointer
  - `## Done` entry; `## In Progress` retired; ladder
    commit refs pruned (the As-built ladder here is the
    permanent record)

### Outcome

Every read-only jj query spawn now goes through `src/jj.rs`
(log templates + bookmark listings), with one tracking
parser pair and one ochid trailer parser beside it, and one
jj test helper per crate. The remaining `run(` sites are
mutations (commit / describe / bookmark / push / fetch /
squash / rebase / op), the git/gh spawns owned by the
de-gitify-init stage and the by-design `gh` calls, push's
`jj diff --stat` display passthrough, and the deliberate
direct spawns in test harnesses — the surface the jj-lib
migration stage inherits.

## docs: adopt new template repo names

Commits: [[7]]

The template repos were renamed (`vc-template-x1` →
`vc-x1-work-repo-template`, `vc-template-x1.claude` →
`vc-x1-bot-repo-template`); the template side adopted the new
names 2026-07-21. Sweep vc-x1's live mentions to match and
restore shared-doc-set byte-identity.

- AGENTS.md re-syncs byte-identical with the template's copy
  (the `CargoRust.toml` mention was the one divergent hunk)
- README.md init examples: the real templates no longer
  satisfy the `<CODE>.claude` sibling default, so the sweep
  makes that example an explicit `CODE,BOT` pair; a generic
  `../tmpl` example keeps the default documented
- Historical records keep the old name (`## Done` entries,
  the notes-sync manifest, chores, jj-tips) — GitHub
  redirects renamed repos. `src/init/tests.rs` fixture names
  are arbitrary strings, not repo references, and stay.

## docs: notes rework + config refresh

Commits:

Alignment batch riding the template-side session: vc-x1
adopts what the working pair (vc-x1 + the template) decided.

- `.vc-config.toml` (both sides) now carries init's
  generated optional-keys block, so the hand-seeded configs
  are byte-identical with what `vc-x1 init` emits (the block
  is schema-sourced and cannot drift from the binary).
- jj-tips.md re-syncs with the template under a
  reclassification: its recorded transcripts are pedagogy,
  not history, so example names adopt
  `vc-x1-work-repo-template`. This supersedes the previous
  section's "historical records keep the old name" for this
  one file; chores files and the notes-sync manifest remain
  historical and keep old names.
- The template-restructure design (single template repo +
  fixed bot seed manifest, `<CODE>.claude` sibling default
  dropped) is promoted to Todo #10; the `.bot` default-name
  decision and the symmetric config schema fold into the
  refactor program's facade-owns-topology stage.
- New Idea recorded: chores retire into a session index
  once the provenance viewer can present session + code
  (the template already seeds no chores history — a new
  project's history is its commits + bot session).
- The bot repo gains its seed-manifest files: LICENSE-*
  verbatim from vc-x1-bot-repo-template, README.md adapted
  from it (the "source template" paragraph is
  template-specific, so it becomes a partner-repo pointer).

## refactor: hygiene riders

Commits: see [As-built ladder](#as-built-ladder-1)

Terminology stragglers from the 0.69.0-4 work/bot sweep plus
the single-field `options_flags` leaves that still name their
field after the flag (`args.<leaf>.<leaf>` doubling) — sweeps
that churn the same lines the later facade stages rewrite,
done early so those stages diff cleanly. Second stage of the
refactor program. Decisions at cycle open (2026-07-22):

- Stragglers rename rather than document-as-historical;
  `-s` gains `work` as the canonical scope keyword.
  Amended 2026-07-23: no `code` alias — vc-x1 is
  unreleased, so the keyword renames outright rather than
  carrying compatibility baggage; `code` now errors.
- Single-field leaves keep the struct + `value` shape (the
  `squash` exemplar) — the leaf structs' reason to exist is
  the single-sourced flag definition (one doc comment /
  parser / default shared by every subcommand), which the
  bare-type alternative forfeits; clone.rs's inline
  `dry_run` duplicate is the observed drift case and folds
  onto the leaf.

### As-built ladder

- [[N]] 0.74.0-0 chore: open hygiene riders cycle
  - version 0.74.0-0; hygiene riders picked into
    `## In Progress` with the four-rung ladder; this
    section opened
  - stage decisions above recorded here, in the ladder
    block, and in the stage section of
    [refactor-20260716.md](../refactor-20260716.md#stage-hygiene-riders)
  - rider: repo-local `tmp/` scratch area — gitignored,
    convention in AGENTS.md; jj ignoring it is what parks
    the trapezoidal-commits draft note outside this commit
- [[N]] 0.74.0-1 refactor: hygiene work/bot idents
  - work/bot terminology sweep across `src/`: the three
    stragglers-plus-synonym families — `code_*` → `work_*`,
    `claude_*` / `session_*` → `bot_*` — covering
    identifiers, `Side::Code`→`Side::Work`,
    `ConfigRole::{Code,Session,AppOnly}`→`{Work,Bot,WorkOnly}`,
    the `derive_session_url`/`claude_path` functions, the
    `Fixture.claude` field, `"code"`/`"session"` narration
    labels, test-remote names (`remote-code`/`remote-claude`
    → `remote-work`/`remote-bot`), and doc-comment prose
  - not touched: `.claude` path literals, `.claude`-derived
    URL suffixes, "Claude Code" prose, and genuine words
    (`dead_code`, `encode_*`, `exit_code`, `session_id` from
    the transcript JSON schema) — those aren't terminology
  - `STATE_FORMAT_VERSION` 1→2: the push-state keys `op_app`
    / `op_claude` renamed to `op_work` / `op_bot`; a v1 file
    would parse but silently drop both rollback targets, so
    the bump turns a stale state into a `--restart` prompt
  - decision applied: full sweep of all three families (the
    stage named four identifiers; the actual old-terminology
    surface was ~380 sites)
- [[N]] 0.74.0-2 refactor: hygiene work scope keyword
  - the `-s`/`--scope` keyword `code` → `work`: `parse_scope`
    now accepts `work`, `bot`, `work,bot`, `bot,work`; `code`
    errors (a test pins the rejection)
  - `value_name`s (`work|bot|work,bot`), the sync/revert/
    common-args help, the `--scope=bot` not-in-workspace hint,
    and every `-s code`/`code,bot` test invocation swept
  - AGENTS.md: scope-name note + the `vc-x1 chid -s work,bot`
    change-ID capture command updated (the latter is a live
    command run each cycle); todo-backlog scope-vocabulary
    references (`finalize`/`push`/`clone`/`validate-desc`
    future `--scope` flags) swept to the new keyword
  - amended decision: no `code` alias — vc-x1 is unreleased,
    so the keyword renames outright rather than carrying
    compatibility baggage (recorded here, in the ladder
    block, and in the stage section of the refactor doc)

# References

[1]: https://github.com/winksaville/vc-x1/commit/f761e89092df "f761e89092dfbb82e8ab355d6e5a058e77b07e23"
[2]: https://github.com/winksaville/vc-x1/commit/47e5075b90da "47e5075b90daa5e9b24fa7c93a5814a2eee0f03f"
[3]: https://github.com/winksaville/vc-x1/commit/5a61ebdcbac8 "5a61ebdcbac872eac03d6b70141030217be1f850"
[4]: https://github.com/winksaville/vc-x1/commit/c3a6d258f511 "c3a6d258f511ae4a3a6f0b6e42aba80d5005d4e8"
[5]: https://github.com/winksaville/vc-x1/commit/303d163196ab "303d163196ab4c387428e4bec0fc65430ead4206"
[6]: https://github.com/winksaville/vc-x1/commit/6c93a011a54d "6c93a011a54dd990035733e49f0bfa169ebad609"
[7]: https://github.com/winksaville/vc-x1/commit/007cc5d5a030 "007cc5d5a030a2a0673ed5ef7b425c98dce40a74"
