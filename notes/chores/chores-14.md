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

Commits: [[11]]

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

- [[8]] 0.74.0-0 chore: open hygiene riders cycle
  - version 0.74.0-0; hygiene riders picked into
    `## In Progress` with the four-rung ladder; this
    section opened
  - stage decisions above recorded here, in the ladder
    block, and in the stage section of
    [refactor-20260716.md](../refactor-20260716.md#stage-hygiene-riders)
  - rider: repo-local `tmp/` scratch area — gitignored,
    convention in AGENTS.md; jj ignoring it is what parks
    the trapezoidal-commits draft note outside this commit
- [[9]] 0.74.0-1 refactor: hygiene work/bot idents
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
- [[10]] 0.74.0-2 refactor: hygiene work scope keyword
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
- [[12]] 0.74.0-3 refactor: hygiene OF value fields
  - "OF" is `options_flags` — the `src/options_flags/` leaf
    structs (the title is immutable in the pushed commit;
    this gloss is the decoder)
  - the six single-field `options_flags` leaves adopt the
    `value` field shape (the 0.47.0 `squash` convention):
    `DryRunFlag.dry_run`, `PrivateFlag.private`,
    `AccountOption.account`, `ConfigOption.raw`,
    `RepoOption.repo`, `UseTemplateOption.use_template` →
    `.value`; consumers swept (init params/tests,
    test_helpers fixtures)
  - clone.rs's inline `dry_run: bool` (the observed drifted
    duplicate of the leaf) folds onto `DryRunFlag` via
    `#[command(flatten)]`, matching its `por` sibling
  - wrinkle: clap derives each arg's id from the Rust field
    name, so six `value` fields flattened into one command
    (`InitArgs`) collide at runtime ("Argument names must be
    unique"). Each leaf now pins `id = "<flag-name>"` (and an
    explicit `long`), decoupling the CLI surface from the
    field name. The `squash`/`por` exemplars never surfaced
    this — they don't coexist with another `value` leaf in
    one command.
  - rider: ladder-ref backfill — chores-14 `[6]`/`[7]`
    re-pointed at the published post-rebase SHAs (the
    trapezoid close-out rewrote them; the old SHAs are
    branch-unreachable), notes-rework `Commits:` filled
    ([[11]]), the `-0`/`-1`/`-2` As-built and TODO.md ladder
    rungs backfilled
- [[13]] 0.74.0 refactor: hygiene riders (close-out)
  - `## Done` entry; `## In Progress` retired; TODO.md
    ladder commit refs pruned (this As-built ladder is the
    permanent record)
  - refactor doc: stage marked shipped at 0.74.0
  - version 0.74.0; Merge non-ff close-out (trapezoid) onto
    `refactor-vc-x1` — set up manually, published via
    `vc-x1 push --from bookmark-set` (the pre-`--merge`
    procedure; the native flag is the program's trapezoid
    stage)

### Outcome

The old-terminology surface in `src/` is gone: no
`code_*`/`claude_*`/`session_*` identifiers (the survivors —
`session_id`, `bot_session`, `encode_*`, `dead_code`,
`exit_code` — are genuine words, not repo-role terms), the
CLI speaks `work`/`bot`, and every single-field
`options_flags` leaf is a uniform `value` struct with its
clap id pinned. What still says `.claude` is the *path* —
the literal directory name, ochid prefixes, URL-suffix
derivation — owned by the facade-owns-topology stage's
configurable bot-dir work.

## refactor: facade owns topology

Commits: see [As-built ladder](#as-built-ladder-2)

Repo resolution is a per-command prelude (each command
re-derives "where is the other repo" from
`path`/`other-repo`); it becomes facade state. Third stage of
the refactor program. Decisions at cycle open (2026-07-23):

- `.vc-config.toml` `[workspace]` adopts the symmetric
  `work = "/"` / `bot = "/.bot"` pair, *replacing*
  `path`/`other-repo` — the block is identical on both
  sides, so there is nothing to keep mutually consistent;
  side detection comes from location (walk up from cwd via
  `find_workspace_root`: at the root → work side, at
  `<root>/<bot-dir>` → bot side).
- Presence of `bot` is the dual-repo signal; POR is
  `work = "/"` alone. One vocabulary across surfaces:
  `-s work,bot`, config `work`/`bot`, `Side::Work`/`Side::Bot`.
- `.bot` is the new-init default only — a recommendation.
  Existing workspaces keep their recorded dir (vc-x1 stays
  `bot = "/.claude"`), old `/.claude/<chid>` trailers keep
  resolving verbatim; no migration.

### As-built ladder

- [[14]] 0.75.0-0 chore: open facade owns topology cycle
  - version 0.75.0-0; stage picked into `## In Progress`
    with the five-rung ladder; this section opened; stage
    decisions recorded here, in the ladder block, and in the
    stage section of
    [refactor-20260716.md](../refactor-20260716.md#stage-facade-owns-topology)
  - rider: 0.74.0 ladder-ref backfill (`-3` + close-out
    rungs → [[12]],[[13]])
  - rider: `## Done` retirement sweep — the 0.69.1–0.71.0
    batch (eight entries) migrated to done.md; TODO.md
    references re-packed to `[1]..[5]`
  - rider: the DRY facade and hygiene stages' shipped-status
    links verified in the refactor doc (added at 0.74.0)
- [[15]] 0.75.0-1 refactor: topology por equalization
  - `validate-desc` / `fix-desc` lose their dual-required
    `other_repo_from_config` prelude: a single-repo / POR
    workspace now no-ops with a clear message instead of
    erroring; a dual workspace resolves as before, and
    `--other-repo` still overrides
  - the replacement is `common::bot_repo_path(root)` —
    `default_scope` gates (has the workspace a bot side?),
    `scope_to_repos` resolves; `Ok(None)` is the no-op
    signal. This is the prototype of the
    topology-from-config rule the rest of the stage extends.
  - `desc_helpers::other_repo_from_config` deleted — the
    helper's last two callers were these preludes
  - tests: three `bot_repo_path` units (dual / single-repo /
    no-config) + a `FixturePor` no-op end-to-end per command
- [[16]] 0.75.0-2 feat: topology work/bot config schema
  - `.vc-config.toml` `[workspace]` flips to the symmetric
    `work = "/"` / `bot = "/.claude"` pair; `path` /
    `other-repo` are gone (unreleased — replaced outright,
    like the scope keyword). init emits one identical dual
    block for both sides (`ConfigRole::{Work,Bot}` collapse
    into `Dual`); this workspace's two committed configs
    flipped in the same commit (the running binary is
    installed before the push that uses them)
  - side detection becomes location-based:
    `common::is_bot_dir(dir)` — true iff the parent dir's
    config names `dir` as its `bot`. `find_workspace_root`
    keys on the `work` key and steps up when standing in the
    bot repo; `sync::is_bot_repo` and the new
    `desc_helpers::ochid_prefix_for(repo)` (replacing the
    content-based `ochid_prefix_from_config`) both ride it
  - `default_scope` / `scope_to_repos` read `bot` (presence =
    dual signal; value is root-relative with leading `/`)
  - legacy files error fast with the solution (decided
    2026-07-23 mid-rung): a config with `path`/`other-repo`
    but no `work` is still *found* by the root walk, and
    every resolver rejects it via
    `common::reject_legacy_config` — the message shows the
    replacement block. Silent degradation to POR was the
    alternative and a foot-gun. A file with both old and new
    keys passes (new keys drive; `config --validate` flags
    the strays as unknown).
  - riders: README caught up — the stale "historical
    wrinkle" paragraph (scope keyword renamed at 0.74.0-2),
    `-s code` examples, `remote-code`/`remote-claude`
    fixture names (missed by the 0.74.0-1 sweep), and the
    layout/schema snippets; AGENTS.md `.vc-config.toml`
    section rewritten to the symmetric form
  - incident while landing this rung: the Bugs #3 lock race
    fired at bookmark-set, and the `--yes` rerun resumed
    from the stale state stage, skipping the commit stages —
    session data squashed into the published `-1` bot
    commit. Recorded as Bugs #5 (resume-after-rollback
    replays from the wrong stage); no data loss, and the
    strongest evidence yet for the stateless-push stage
- [[17]] 0.75.0-3 refactor: topology bot-dir sweep
  - every reader of the bot-repo location now resolves it
    from `[workspace] bot`; `.claude` as a *choice* survives
    only in init's `DEFAULT_BOT_DIR` (plus the `Dual` render
    / `GITIGNORE_CODE` literals its doc comment binds to it)
    and the remote-name suffix, which belongs to the
    deferred `.bot`-flip decision set
  - swept: push's `bot_path` (now `require_bot_dir`) and its
    commit-work ochid prefix (via `ochid_prefix_for`),
    `repo_utils::cross_ref_ochids` (prefix from the created
    dir's name), `bm_track`'s probe + label, clone's local
    destination (from the *cloned* work repo's config),
    `validate-bot`'s `-R` default, `symlink`'s default
    target (both fall back to `.claude` when unresolvable)
  - dual-mode entry preflight (the 2026-07-23 principle):
    `bot_repo_path` verifies coherence before anything acts —
    bot dir exists, its config loads, and the two sides'
    `[workspace]` blocks are identical — erroring with both
    paths and both blocks, changing nothing.
    `configured_bot_dir` is the pure-config-read half for
    pre-existence callers (clone)
  - field report, same day: a `vc-x1 push` in another
    workspace (you.h2hist) hit the legacy-config error from
    `-2`; the bot there applied the fix printed in the
    message and the push went through — the
    error-fast-with-the-solution loop working as designed
  - path grammar pinned *and enforced* (pulled forward from
    the `-4` plan during review — no downside):
    `check_workspace_grammar` at the same resolver
    chokepoints rejects `work` ≠ `"/"` (a name tag every
    reader ignores — the last silent-lie case) and a `bot`
    that isn't `/` + one component (an unanchored value
    corrupts ochid trailers)
  - stage-doc prose riders: the grammar definition, the
    dual-preflight principle, and the bisect-skew note
    (the two repos rewind independently; old binaries fail
    loudly-but-cryptically against the new bot-side config)
- [[18]] 0.75.0-4 feat: topology config target
  - the `config` command reshape (decided at the `-2`
    review): positional `work|bot|work,bot|<path>` target
    (default `work,bot`) replaces `--home`; print stays the
    default, one schema group per resolved side, the divider
    naming the target keyword + resolved file path; the user
    config has no keyword — reachable only via its explicit
    path
  - a path target gets the *whole* schema (decided during
    this rung's review): a path carries no side information,
    so nothing is guessed from it — the old `--home` group
    labels (User / Workspace) died with the flag, and the
    earlier sniff-the-user-config-path detection was dropped
    as imprecise. Slightly laxer validation for a path
    target (any home's key accepted) traded for exact
    semantics: keywords = side-filtered, path = full
    registry
  - `--validate` now covers the target file(s)' unknown keys
    *and* the topology invariants: the `[workspace]` grammar
    (via `reject_legacy_config` at the root) and the
    identical-`[workspace]`-block coherence check (riding
    `bot_repo_path`'s dual preflight). A coherence failure is
    a counted finding, not a hard error, so the work-side
    report still lands; a single-repo workspace skips the bot
    side with a note
  - rider: `package.name` → `vc-x1-dev` (decided this
    session): the per-commit `cargo install` no longer
    clobbers the `vc-x1` binary concurrent workspaces run;
    promotion to plain `vc-x1` is an explicit act — see
    versioning.md's new
    [Dev artifact name](../versioning.md#dev-artifact-name)
- [[19]] 0.75.0 refactor: facade owns topology
  - close-out bookkeeping: version 0.75.0; the
    `## In Progress` block retired (decisions live in this
    section's intro and the stage doc); Done entry added;
    ladder and Outcome finalized

### Outcome

- The stage shipped in four Work rungs on `refactor-vc-x1`;
  repo resolution is facade state — topology comes from
  `.vc-config.toml`'s symmetric `[workspace]` block, side
  from location, and every `.claude` literal from config.
- Spun off during the `-4` review: the
  [repo registry stage](../refactor-20260716.md#stage-repo-registry)
  (file-relative/absolute paths, resolved agreement
  replacing identical blocks, ochid prefixes as registry
  labels) — ranked Todo #2, so the schema settles in one
  migration wave before de-gitify init.
- Still open (recorded in the stage doc): the
  harness-controlled symlink location and the vendor's
  `<project>/.claude` settings dir when a workspace picks a
  non-`.claude` bot dir.
- Field-tested mid-cycle: another workspace (you.h2hist)
  hit the legacy-config error and self-served the printed
  fix; and the `-2` lock-race incident became Bugs #5, more
  evidence for the stateless-push stage.

## docs: refactor program ladder + conventions

Commits: [[20]]

The 0.75.1 planning interlude between the facade-owns-topology
and repo-registry cycles: the refactor program's remaining
stages consolidate into four cycles, `TODO.md > ## In
Progress` adopts a heading shape, and the version-visibility
convention lands.

- Program restructure: the program entry moves to
  `## In Progress` as a `###` heading (picked-up tasks are
  `###`, a program's current stage a `####`) carrying a
  program ladder — one rung per cycle, shipped rungs
  backfilled with real commit refs, provisional versions on
  the rest. Stages consolidated pairwise (repo registry +
  de-gitify init, split push.rs + stateless push, jj-lib,
  body-intro validation + trapezoid close-out) — small
  stages ride as rungs of a neighbor. Rung / Todo titles
  drop "stage": they are the anticipated close-out commit
  titles.
- Parked 0.72.0 disposition: `support-trapezoid-commits` is
  quarry, not base — rebase or redo its extraction when the
  split-push cycle opens, then delete the bookmark; 0.72.0
  is an accepted version gap, recorded in the refactor doc's
  split-push stage (versioning.md is shared byte-identical
  across the template family, so the record stays
  project-local). Logged as a live example on the Todo
  "Version-number protocol is fragile".
- Version-visibility convention (cycle-protocol.md Body):
  the work-repo body's file list opens with
  `- Cargo.toml, Cargo.lock: version X.Y.Z-N`, so log
  viewers show the version without opening the commit's
  file list. Evolves the shared protocol; template fan-out
  after the refactor series (with the pending shared-doc
  syncs).
- Backfills: the 0.75.0 As-built rungs ([[14]]–[[19]]) and
  the program ladder's shipped rungs — commit refs on
  `refactor-vc-x1`, treated as permanent (long-lived
  branch, lands on main merge-only, never rebased).

## refactor: repo registry

Commits: see [As-built ladder](#as-built-ladder-3)

`.vc-config.toml`'s `[workspace]` block is a root-anchored
path grammar (leading `/` = workspace root, pinned at
0.75.0-3) kept coherent by a literal identical-block
invariant. It becomes a **repo registry**: stable labels
mapped to locations. Fourth cycle of the refactor program;
de-gitify init rides as its last rung. Decisions at cycle
open (2026-07-24):

- The section is named `[repos]` — "workspace" is overloaded
  (cargo, VS Code, and jj itself has `jj workspace`) and
  vc-x1 is medium-agnostic; config-key paths become
  `repos.work` / `repos.bot`.
- `work` / `bot` values become ordinary paths: absolute, or
  relative to the directory containing the config file that
  states them (the standard file-relative rule — no
  leading-`/`-means-root special case). Docs recommend
  relative; absolute is allowed but discouraged, since it
  commits one machine's layout.
- The identical-block invariant cannot survive
  file-relative values (the same bytes mean different dirs
  on each side), so it is replaced by **resolved
  agreement** — canonicalize each side's declared pair and
  require both files to name the same two directories. The
  0.75.0-3 coherence preflight keeps its role, comparing
  reality instead of spelling.
- Side detection: the entry resolving to the config's own
  directory names the side (`work = "."` → work repo).
  Repos need not be nested.
- ochid trailer prefixes become opaque registry labels
  resolved through `[repos]`, not filesystem spellings: `/`
  and `/.claude` stay valid as historical labels for
  work/bot, so published trailers keep resolving. Labels are
  intended to grow into URLs — the local-path half of the
  Todo "ochid: bot-repo location qualifier", whose published
  half lives in
  [forks-multi-user.md](../forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid).
- Sequencing: this runs immediately after facade owns
  topology because the schema is unreleased and freshly
  adopted (one external workspace migrated at 0.75.0-2) —
  one migration wave instead of two, and the init/clone
  rework lands on the final schema. See the
  [stage](../refactor-20260716.md#stage-repo-registry).

### As-built ladder

- [[21]] 0.76.0-0 chore: open repo registry cycle
  - version 0.76.0-0; the stage picked into
    `## In Progress` as the program ladder's current
    `####` rung with a seven-rung ladder; this section
    opened; the `[repos]` name settled and recorded here,
    in the ladder block, and in the stage section of
    [refactor-20260716.md](../refactor-20260716.md#stage-repo-registry)
  - rider: 0.75.1 `Commits:` backfill ([[20]])
  - rider: `## Done` retirement sweep into done.md
- [[22]] 0.76.0-1 refactor: registry schema + side detection
  - `[workspace]` → `[repos]` across the topology core,
    schema registry, init renders, fixtures, and both live
    configs (atomic flip, the 0.75.0-2 precedent)
  - values are ordinary file-relative paths; side detection
    by self-resolution (the entry resolving to the config's
    own dir names the side), so the root walk needs no
    nesting assumption
  - rung-boundary decision (approved): the identical-block
    invariant can't survive asymmetric per-side blocks, so
    this rung seeds a minimal resolved-agreement coherence
    check; 0.76.0-2 finishes the preflight (validate
    wiring, error detail, edge cases, dedicated tests)
  - the 0.75.x root-anchored schema joins pre-0.75.0
    `path`/`other-repo` as a rejected legacy generation
    (found by the root walk, rejected with the per-side
    rewrite)
  - interim ochid prefix: bot side rebuilt as
    `/<dir name>/` from the canonicalized bot dir
    (`repos.bot` is `"."` in its own config); 0.76.0-3
    decouples labels from filesystem spelling
  - rider (bug, found in review): clone of a legacy-schema
    repo swallowed the rejection and guessed `.claude`.
    Backward-compat read `legacy_configured_bot_dir` honors
    both legacy generations at the bootstrap surfaces —
    clone completes with the declared bot dir + a
    warn-to-update, the symlink default follows it, and the
    fix-it rewrite echoes the workspace's actual bot dir
    name; every other resolver still hard-rejects
- [[23]] 0.76.0-2 refactor: registry resolved-agreement
  preflight
  - finishes the -1 seed: **self-identification** joins
    resolved agreement — each config's own directory must
    sit at its side's key (root's `work`, bot's `bot`),
    since agreement alone can't catch both sides naming the
    same wrong pair
  - dedicated edge tests: missing `repos.bot`, unresolvable
    declared dir, self-identification violation, and
    absolute/relative spellings agreeing on resolved reality
  - `config --validate` reports these through the bot-side
    resolution's coherence check (wired at -1); no separate
    validate path needed
  - validate legacy dedupe (user report, iiac-perf): a
    legacy schema printed the rejection twice (root call +
    bot-side resolution) plus its own keys as unknown — now
    one warn, one finding, remaining checks skipped as
    redundant
  - rider: incremental `Commits:` backfill of the pushed -0
    / -1 rungs ([[21]],[[22]] here; [[8]],[[9]] in TODO.md's
    ladder) — the per-push cadence the protocol implies,
    rather than saving backfills for cycle boundaries
  - rider: bugs.md #1/#2 annotated as scheduled with the
    de-gitify init rung; #6 (push empty commit-work
    duplicate) recorded from the -1 push incident
  - rider: commit procedure clarified after the #6 incident
    — AGENTS.md's Committing-vs-pushing condensed to "a
    cycle rung is committed *by* `vc-x1 push`, never
    pre-committed" (the stale non-top hand-written-ochid
    rule deleted), and the per-commit flow's step 8 now
    invokes `vc-x1 push --title --body` (the `jj commit`
    example deleted); net smaller
- [[24]] 0.76.0-3 refactor: registry ochid labels
  - trailer prefixes are now canonical side labels —
    `OCHID_WORK_LABEL` (`/`) and `OCHID_BOT_LABEL`
    (`/.claude`) — chosen by side detection, never from the
    directory's spelling; a non-`.claude` bot dir reads and
    writes the `/.claude` label (the -1 interim dir-name
    derivation retired)
  - `/` and `/.claude` are the historical spellings, so
    every published trailer stays valid; the constants are
    the seam where URL labels grow in later
    ([forks-multi-user](../forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid))
  - legacy fallback (user review, verified on iiac-perf):
    `is_bot_dir` falls back to the legacy location rule —
    parent's `workspace.bot` *or* pre-0.75.0
    `workspace.other-repo` — so the explicit `--other-repo`
    escape hatch in validate-desc / fix-desc (which bypasses
    the resolvers' legacy rejection) still sides a legacy
    bot dir correctly instead of flagging (or rewriting!)
    every valid `/.claude/` trailer as wrong-prefix
  - the whole legacy surface consolidated into
    `src/legacy_vc_config.rs` (side detection, bot-dir read,
    root marker, the fix-it rejection) for easy retirement
    once every workspace is migrated: delete the module and
    simplify the call sites `grep -rn 'legacy_vc_config::'
    src/` lists
- [[N]] 0.76.0-4 docs: registry docs sweep
  - AGENTS.md's `.vc-config.toml` section rewritten for the
    `[repos]` registry: per-side blocks, file-relative
    semantics, resolved agreement, side labels
  - README: validate-desc default read, `config --validate`
    checks (legacy rejection + resolved agreement), the
    printed-schema sample, and sync's repo-set resolution
    all speak `[repos]`
  - refactor-20260716.md stage status: rungs -1..-3 recorded
    shipped with the legacy-consolidation note
  - todo-backlog #20 rewritten: the `[workspace]` rename
    itself shipped this cycle; the surviving remainder is
    the broader "stop saying workspace" wording sweep
  - ARCHITECTURE.md checked — no schema-level detail, no
    edit needed
  - incremental backfill rider: -2/-3 refs ([[23]],[[24]]
    here; [[10]],[[11]] in TODO.md's ladder)
- [[N]] 0.76.0-5 refactor: de-gitify init
  - init's publish path is jj-only: jj stays colocated
    throughout, `jj git remote add` + `jj git push` replace
    the strip-jj → git-push → re-colocate dance (`git clean
    -xdf`, `git checkout`, `git remote add`, `git push -u`,
    `jj git init --colocate`), and the push establishes
    tracking as a side effect (no `--allow-new`, no
    follow-up `jj bookmark track`)
  - `prepare_local_repo` drops its explicit `git init` —
    `jj git init --colocate` creates the git repo itself
  - the strip-jj step is what forced `push_repo`'s
    `clean_exclude` parameter (preserving the nested bot
    repo through `git clean -xdf`); with no clean, the
    parameter is gone
  - bugs.md #1 fixed: local bares are created with
    `--initial-branch=main`, so a clone of one has a default
    branch to track
  - bugs.md #2 fixed, both halves: init's bot bare is now
    `derive_bot_url` of the work bare
    (`remote-work.claude.git`), the same rule clone uses —
    one naming convention instead of two; and the bot-side
    clone runs from the invoking cwd, so a relative
    local-path TARGET resolves on both sides
  - `tests/cli_sync.rs` drops both workarounds (HEAD
    symbolic-ref, bot-remote symlink) and now clones by
    relative path, so the fixes are pinned by the test that
    originally documented the bugs
  - consequence worth knowing — where the identity check
    happens moved, and it caught a latent defect. See
    [Identity enforcement: git commits, jj pushes](#identity-enforcement-git-commits-jj-pushes)

### Identity enforcement: git commits, jj pushes

Swapping init's publish path from `git push` to `jj git push`
moved *when* a missing user identity is caught, which surfaced
as two CLI-fixture tests failing at the new push step. The two
tools enforce at opposite ends:

- **git — at commit.** `git commit` with no `user.name` /
  `user.email` tries to synthesize one from
  `username@hostname`; if that isn't usable it refuses
  outright (`fatal: unable to auto-detect email address (got
  'wink@3900x.(none)')`). Nothing is checked at push time —
  `git push` never inspects an existing commit's author.
- **jj — at push.** `jj commit` succeeds with an explicit
  *empty identity*, warning `Name and email not configured.
  Until configured, your commits will be created with the
  empty identity, and can't be pushed to remotes.` The
  enforcement lands at `jj git push`: `Error: Won't push
  commit <id> since it has no author and/or committer set`.

We think jj's split follows from its model: local commits are
cheap and rewritable, so it defers the check to the boundary
where history becomes shared and permanent.

The latent defect: init already used `jj commit` for the
initial commit, so under the old flow the CLI fixtures — whose
`HOME` is an isolated tempdir with no user config — produced
empty-identity commits and `git push` shipped them to the bare
remotes without complaint. Nothing verified authorship, so it
went unnoticed. The jj-only path fails loudly instead, which is
why `CliFixture::new` now seeds a `[user]` identity: not test
scaffolding for its own sake, but the fixture finally supplying
what a real workspace always has.

Not documented upstream: jj's config, git-compatibility, and
FAQ pages don't mention the empty identity or the push-time
refusal (checked 2026-07-27) — jj's runtime warning and error
text are the authoritative statement.

# References

[1]: https://github.com/winksaville/vc-x1/commit/f761e89092df "f761e89092dfbb82e8ab355d6e5a058e77b07e23"
[2]: https://github.com/winksaville/vc-x1/commit/47e5075b90da "47e5075b90daa5e9b24fa7c93a5814a2eee0f03f"
[3]: https://github.com/winksaville/vc-x1/commit/5a61ebdcbac8 "5a61ebdcbac872eac03d6b70141030217be1f850"
[4]: https://github.com/winksaville/vc-x1/commit/c3a6d258f511 "c3a6d258f511ae4a3a6f0b6e42aba80d5005d4e8"
[5]: https://github.com/winksaville/vc-x1/commit/303d163196ab "303d163196ab4c387428e4bec0fc65430ead4206"
[6]: https://github.com/winksaville/vc-x1/commit/b5e40e7458b8 "b5e40e7458b8506574b2ae01f52f7ccae9023418"
[7]: https://github.com/winksaville/vc-x1/commit/e5d1aae2985a "e5d1aae2985af8408f20fbd63bef7f172ec2dc59"
[8]: https://github.com/winksaville/vc-x1/commit/1e4cfa5a04b2 "1e4cfa5a04b2961ec5c158b8baeb1aef677c5bdc"
[9]: https://github.com/winksaville/vc-x1/commit/efb66f992890 "efb66f992890e8f7f6434010b79f97c282b5bdd4"
[10]: https://github.com/winksaville/vc-x1/commit/143743ec3bb4 "143743ec3bb4b067b7d53ce42e6b8f5c316ab5ec"
[11]: https://github.com/winksaville/vc-x1/commit/f2a042452176 "f2a0424521765c72151c5d663e35b69d8b21fef7"
[12]: https://github.com/winksaville/vc-x1/commit/31e8d95816ba "31e8d95816baad7dd7e1e5c66618de5070ba1b03"
[13]: https://github.com/winksaville/vc-x1/commit/946dc964b75c "946dc964b75ca29e2cc4b6c59f03aec2c364feee"
[14]: https://github.com/winksaville/vc-x1/commit/160cff19a7dc "160cff19a7dca70272998d970e5a6bc7c5c29b45"
[15]: https://github.com/winksaville/vc-x1/commit/bd415793e0c4 "bd415793e0c443f3ee0b5694833b8ae8ab8b8fdc"
[16]: https://github.com/winksaville/vc-x1/commit/49395a3a6a11 "49395a3a6a116f889b6a388f02680c6cc852abff"
[17]: https://github.com/winksaville/vc-x1/commit/f896b8e67e0b "f896b8e67e0b224e3abbe938199951916059198a"
[18]: https://github.com/winksaville/vc-x1/commit/3c0d15ea2fca "3c0d15ea2fca3db36135fa38d40687fdb923c239"
[19]: https://github.com/winksaville/vc-x1/commit/dc14a421d850 "dc14a421d8509e58fa05741fd1a868329540731e"
[20]: https://github.com/winksaville/vc-x1/commit/eb4a12eb3b56 "eb4a12eb3b561234d176953d3773960fb9f4cdaa"
[21]: https://github.com/winksaville/vc-x1/commit/da932d2293c8 "da932d2293c8de28cf2290d14502a991bdee5861"
[22]: https://github.com/winksaville/vc-x1/commit/ff4a8d624cf3 "ff4a8d624cf3979ebad246f9c3961b7fed2dcba1"
[23]: https://github.com/winksaville/vc-x1/commit/f8885f1e411f "f8885f1e411f0d2e8f6e0c26aa1b226b7a93a556"
[24]: https://github.com/winksaville/vc-x1/commit/669e9fbb7fb8 "669e9fbb7fb816c3c584fa6f525128ee7ebb63bd"
