# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

_No cycle currently in progress._

## Todo

 A markdown list of tasks to do in the near future, ordered
 highest-priority first. Keep entries brief — 1-3 lines.
 Detailed motivation, safety requirements, and ordering belong
 in `notes/chores/chores-NN.md` design subsections; link via `[N]` ref.

 Entries carry explicit `1.` `2.` … numbers in the source so
 you can grep, count, and reference them ("let's work on
 #3"). You don't hand-maintain the numbers — insert, delete,
 or reorder entries freely, then `vc-x1 fix-todo --no-dry-run`
 renumbers the list and normalizes continuation-line indent.
 `vc-x1 fix-todo` alone only previews; `vc-x1 validate-todo`
 is the read-only check.

1. **single-field `options_flags` leaves → `value` field.**
   `0.47.0` introduced the convention (single-field leaf names
   its field `value`, declares the flag via `#[arg(long = "…")]`,
   so consumers read `args.<leaf>.value` not `args.<leaf>.<leaf>`)
   on the new `squash` leaf. Sweep the pre-existing single-field
   leaves to match: `repo`, `dry_run`, `private`, `account`,
   `config`, `use_template` + their consumers
   (`init.rs`, tests).

   Note: can a single field be defined as an type or enum instead
   of a struct and maybe eliminate the `args.<leaf>.<leaf>` name
   issue.
2. **Investigate `linkme` for subcommand registration.**
   Distributed-slice registry — each subcommand registers itself
   at link time; `main.rs` discovers them via the slice rather
   than matching a `Commands` enum. Reduces per-subcommand
   touchpoints from 3 (mod decl + enum variant + match arm) to 1
   (registration). Costs: loses compile-time exhaustiveness
   (missing registration = runtime gap); help-output ordering
   depends on link order unless sorted; macro-magic dependency.
   Revisit once the `0.50.0` trait sweep's per-arm cost has been
   felt under real "add a subcommand" load.
   <https://github.com/dtolnay/linkme>
3. **Investigate `inventory` as `linkme` alternative.** Same
   shape as `linkme` — runtime-iterable registry populated by
   `inventory::submit!` per subcommand. Trade-offs mirror
   linkme's. Pick one if/when the trait sweep's match becomes
   the felt bottleneck.
   <https://github.com/dtolnay/inventory>
4. **`por → dual` conversion.** Attach a `.claude`
   companion repo + `.vc-config.toml` to an existing por
   workspace; emit cross-links going forward. Manual
   setup on an external por workspace (2026-05-14)
   proved arduous; this should be a routine subcommand.
   Design stub in [[4]] § 2.
5. **Audit hardcoded `.claude` in diagnostics / logging.**
   `vc-x1 finalize --repo .bot` on `../dicom-rs`
   (2026-05-14) honored `--repo` throughout actual
   operations, but `bm-track` still emitted
   `.claude(main)=no-jj` — diagnostic strings have
   `.claude` baked in. Cosmetic today; load-bearing once
   bot-repo name becomes configurable (see Symmetric
   `.vc-config.toml` schema entry below).
6. **por/dual parity + `dual → por` conversion.** Make
   `por` and `dual` first-class equals (dual is primary
   today, por bolted on); add `dual → por` conversion
   (detach the `.claude` companion). Builds on the
   `--scope` rollout below. Pre-design; goal + open
   questions in the stub. [[4]]
7. **forks-multi-user + bot-data-formats follow-through.**
   Design captured across two notes; concrete work to
   land when a cycle picks it up. Major pieces:
   multi-line `ochid:` parser/emitter; bot-side
   immutability enforcement; URL-shaped ochid (per-user
   / cross-repo); vendor-subdir layout
   (`.bot/<vendor>/<version>/<id>.<ext>`) +
   flat-to-vendor migration; `.claude/` → `.bot/` rename
   (gated behind symmetric `.vc-config.toml` schema).
   Each piece is its own future TODO when the design
   hardens. [[5]],[[6]]
8. **Symmetric `.vc-config.toml` schema.** Add `code = "/"`
   and `bot = "/.claude"` (workspace-root-relative paths) so
   both repos read from the same shape. Side detection walks
   up from cwd via existing `find_workspace_root`, then maps
   cwd-relative-to-root onto the configured `code` / `bot`
   paths. Cwd-basename match is a fast-path shortcut at
   role-root level only — subdirs need the walk-up. Was the
   deferred half of `0.41.1-6`'s title. Migration story for
   existing workspaces TBD at design time. [[7]]
9. **`test_helpers::Fixture` migration + downstream callers.**
   Plus rename `Fixture` → `TestFixtureDual` and `FixturePor`
   → `TestFixturePor` so call sites carry the test-only
   signal that `#[cfg(test)] mod test_helpers` doesn't
   communicate. Was `0.41.1-7`. [[8]]
10. **`vc-x1 finalize --scope` flag.** Replace `--repo`
    with the role vocabulary used elsewhere
    (`code|bot|code,bot`). Carry-over from the 0.42.0
    `--scope` sweep (was 0.42.0-5; deferred at -4.7
    close-out). The paired `Single(_)` dogfood item
    (0.42.0-7) is moot after `0.53.0` — `Single(_)`
    deleted. Design lives in chores-07. [[9]]
11. Cross-file `chores-NN.md` ordering sanity pass.
    `chores-08.md` (the 0.41.1 cycle) landed on `main` via
    the `0.42.0-4.7` rebase; check that section ordering
    across `chores-06`/`-07`/`-08`/`-09` is chronologically
    coherent and normalize if not. Low priority.
12. Add a vc-x1 validate-repo?
13. vc-x1 push: rework the two bookmark parameters.
    `PushArgs` has `bookmark_pos` (positional `BOOKMARK`) +
    `bookmark` (`--bookmark` flag) for one logical value,
    forcing an `or_else` in `From<&PushArgs>`. Collapse to a
    single positional with `--bookmark` as a true clap alias,
    or drop one spelling. [[10]]
14. vc-x1 push: `--recheck` — implement or remove. Parsed by
    `PushArgs`, never read; mirrored into `PushParams` with
    `#[allow(dead_code)]`. Either wire the
    skip-preflight-on-resume behavior or drop the flag. [[11]]
15. vc-x1 push: `--scope=code|bot|code,bot` flag.
    Was 0.42.0-4 (deferred when cycle pivoted to icr
    rebase work; cycle closed at -4.7). State machine
    becomes scope-aware — single-side scope skips
    `commit-claude`/bookmark-claude/`finalize-claude`.
    [[12]],[[13]],[[14]],[[15]]
16. vc-x1 clone: `--scope=code|bot|code,bot` flag.
    Parallel to `init --scope` for role selection;
    topology (`--por` vs dual) is the separate `--por`
    boolean. Was 0.42.0-6 (deferred at -4.7
    close-out). [[13]],[[14]],[[15]]
17. vc-x1 validate-desc / fix-desc:
    `--scope=code|bot|code,bot` flag. Same role vocabulary
    as elsewhere — `code` validates code's commits against
    bot, `bot` reverses, `code,bot` does both (new
    default). [[13]],[[14]],[[15]]
18. Unify `.vc-config.toml` accessors onto Pattern B
    (typed struct + `load_from(path)`, like new
    `config::UserConfig` and `push::resolve_state_layout`).
    Replaces the map-typed helpers in `desc_helpers.rs` /
    `fix_desc.rs` / `validate_desc.rs` with a typed
    `WorkspaceConfig` struct. ~50 LOC, mechanical.
    Candidate for 0.41.2. [[16]]
19. Layered config precedence (user → workspace → CLI)
    once `WorkspaceConfig` is typed. Workspace can
    override `[github].owner` etc. for a specific project;
    init can't use the layer (chicken-and-egg) but
    post-init commands can. Depends on the
    `WorkspaceConfig` typed-struct refactor above.
    Candidate for 0.41.2. [[16]]
20. Help layout: force over-under everywhere. Apply
    `next_line_help(true)` at the root (or via the existing
    `cli_with_banner` walker) so every subcommand's `-h` /
    `--help` uses the same layout. Today clap auto-picks
    per-command based on the widest flag spec, so
    `sync -h` is two-column but `init -h` is over-under —
    visual inconsistency.
21. Replace "Step N" log prefixes with single-word
    `label: body` convention (`bookmark`, `provision`,
    `colocate`, `cross-ref`, `symlink`, …); indent labels
    under per-side `code:` / `bot:` headers in dual.
    Originally planned as 0.41.1-6.7; deferred.
22. Consider renaming the `.vc-config.toml` `[workspace]`
    section. Rust readers expect `[workspace]` to mean a
    Cargo workspace, which a vc-x1 dual-repo isn't.
    Candidates: `[repo-list]`, `[project]`, `[dual-repo]`.
    Breaking change — needs migration story (read both
    names during a transition cycle, or one-shot rewrite
    in `vc-x1 sync`/`init` on first contact). Drives the
    broader "stop saying workspace in user-facing surfaces"
    sweep.
23. Add `status` (alias `st`) subcommand: `jj st` across both
    repos in one shot. Uses `--scope` from day one. This is
    natural home for the working-copy signal called out and
    it needs to include remotes, like remotes/origin/main. [[17]].
24. `vc-x1 init --dry-run` should bypass the
    `--repo-remote` path-existence preflight (currently fires
    before the dry-run early-return; observed dogfooding
    2026-04-24).
25. vc-x1 push: `--squash` flag. Squashes WC into `@-` via
    `--ignore-immutable` and force-pushes; needs
    `--force-with-lease`-equivalent + state-sanity preflight in
    place first. [[12]]
26. vc-x1 push: `--message-file PATH` flag. Git-style commit
    message file (first line = title, blank, rest = body).
    Alternative to `--title` + `--body`. [[18]]
27. Mirror `--check` / `--no-check` onto `vc-x1 push` (forwards
    through to the preflight `vc-x1 sync` invocation).
    0.37.1 hard-codes `--check`; default stays `--check`.
28. Add `validate-repo` subcommand: diagnostic that runs all
    `verify_*` checks (tracking, push state freshness, ochid
    integrity, conflicts, config sanity, working-copy state)
    plus chores↔commit consistency — every `[N]:` anchor
    resolves, and each chores `##` section's recorded title
    matches its `Commits:` commit's title — and reports
    per-check pass/fail. Exit code = number of failed checks.
    Implementation: promote `verify_state_sanity` /
    `verify_completion_sanity` from push.rs to `common.rs`. [[19]]
29. sync: surface working-copy state in the up-to-date summary
    (per-repo pending-files count or compact stat). Wording-only
    fix shipped in 0.37.1; this is the design+impl. [[17]]
30. bm-track silent-when-clean refinement. Print on entry/exit
    only when state isn't fully tracked or when exit state
    differs from entry. [[20]]
31. "Oh shit" revert — post-success undo via `.vc-x1-ops/`
    anchor dir. Idea-stage; every repo-mutating command drops a
    pre-op snapshot, `vc-x1 undo` restores both repos. [[12]]
32. Restructure templates: replace separate `vc-template-x1` +
    `vc-template-x1.claude` repos with a single `vc-template-x1`
    that has `.claude/` as a subdir (covers `LICENSE-*` etc. for
    both sides in one place). Updates to `vc-x1 init` / `clone`
    needed for the new layout.
33. Source-code design ref sweep + CLAUDE.md codification:
    adopt section-name + `blob/main/...` URL pattern for source
    code refs to designs; codify in CLAUDE.md alongside the
    existing markdown ref conventions. Sweep targets:
    src/push.rs lines 4, 121, 645, 1219. [[21]]
34. Richer bookmark enumeration: per-bookmark remote presence + tracking status [[22]]
35. Per-line/per-thread runtime log points (future, maybe) [[23]]
36. Add Windows symlink support via `std::os::windows::fs::symlink_dir` [[24]]
37. Add "::" revision syntax for jj compatibility
38. Add -p, --parents, -c, --children so parent and child counts can be asymmetric
39. Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
40. Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [[25]],[[26]]
41. Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [[27]]
42. Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [[28]],[[29]]
43. Test-tempdir override resolution chain. Both
    `src/test_helpers::unique_base` and
    `tests/common/unique_base` currently use
    `std::env::temp_dir()` (= `$TMPDIR`). Generalize to
    resolve in priority order: explicit env var (e.g.
    `VC_X1_TEST_TMPDIR`) → user config
    (`~/.config/vc-x1/config.toml`) → local
    `.vc-config.toml` → `std::env::temp_dir()` fallback.
    Useful when a developer wants tests on a tmpfs / SSD /
    project-local path without exporting `TMPDIR` globally.
    Open question: do we also expose a CLI parameter
    (e.g. `vc-x1 --workspace-tmp …`)? Test binaries can't
    easily accept arbitrary flags via `cargo test --`, so
    env is the realistic surface for tests; for the
    `vc-x1` binary itself a flag is feasible but unclear
    it adds value over the resolution chain.
44. **`validate-todo` / `fix-todo`: flag malformed lines.**
    A column-0 line inside `## Todo` / `## Bugs` that is
    neither an entry (`N. `) nor a heading is malformed;
    it's currently tolerated silently. Report it so stray
    lines / typos surface.

## Bugs

   Known defects we're aware of but haven't scheduled a fix
   for. Each entry should describe what goes wrong, when,
   and the cost of the failure. Numbering convention same as
   `## Todo` (manual `1.` `2.` …).

1. **`finalize::surface_previous_failures` is racy and
   bounded by "next vc-x1 run".** The current model has
   four gaps:
   - **Stale forever.** Markers sit on disk until the
     next `vc-x1 <anything>` runs. If the user abandons
     the workspace (e.g., CI / scheduled use), failures
     are never surfaced.
   - **Concurrent surface_previous_failures.** Two
     `vc-x1` runs racing: both `read_dir`, both print
     the marker, one deletes (the other's `remove_file`
     silently fails). The user sees the same failure
     printed twice.
   - **Mid-write torn read.** A `finalize --exec` child
     writing a marker while a sibling `vc-x1` is
     surfacing could read partial content. Atomic-rename
     on write would close this.
   - **No notify-at-failure path.** A detached
     `finalize --exec` failure only becomes visible when
     the user next runs *any* `vc-x1` command — fine for
     interactive use, invisible for CI / scheduled use
     where there may be no next run.

   The exec-child gate in `main.rs` (since 0.52.0-3)
   patches one related case — the detached child eating
   its own prior markers before the user can see them —
   but doesn't address any of the above. Holistic fix
   needs locking + atomic writes + maybe a
   notify-at-failure path for hands-off use.

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

_Migrated to [done.md](done.md) on 2026-05-15 (0.44.0–0.50.0 batch)._

- chores subdir reshape — `notes/chores-*.md` → `notes/chores/`; 0.44.0–0.50.0 Done batch migrated to done.md (0.51.0) [[1]]
- `sb_ide` elimination — banner off by default (`-V` toggles), `bm_track` → `debug!`, `sb_ide` + `SubcommandRunner::{is_detached_exec, suppress_banner}` removed (0.52.0) [[2]]
- todo renumber + `notes/fix-todo.py` interim script; cycle re-scoped at close-out, scope CLI cleanup deferred to 0.54.0 (0.53.0) [[3]]
- scope CLI cleanup — `--scope` roles-only, `--por` boolean replaces `ScopeKind`, `Scope` relocated to `options_flags/`, sync gains `-R` (0.54.0) [[30]]
- validate-todo / fix-todo subcommands — check + renumber `## Todo` / `## Bugs` entry numbering, replacing `notes/fix-todo.py` (0.55.0) [[31]]
- refine cycle protocol — one protocol (Preparation/Work-N/Close-out), `.`-separator nested numbering with trailing-`0`=Preparation, push & squash discretionary, `.claude` once per push, two-gate review (work then message, both before commit), CLAUDE.md cycle/commit/push docs consolidated into one linear `## Cycle Protocol` (~39% smaller) (0.56.0) [[32]]

# References

[1]: /notes/chores/chores-11.md#chore-move-chores-under-noteschores-0510
[2]: /notes/chores/chores-11.md#chore-close-sb_ide-elimination-0520
[3]: /notes/chores/chores-11.md#chore-todo-renumber--fix-todopy-0530
[4]: /notes/por-dual-parity.md
[5]: /notes/forks-multi-user.md
[6]: /notes/bot-data-formats.md
[7]: /notes/chores/chores-08.md#operations
[8]: /notes/chores/chores-08.md#cycle-structure--multi-step
[9]: /notes/chores/chores-07.md#--scope-enum-refactor-0420
[10]: /notes/chores/chores-09.md#push-dual-bookmark-parameters
[11]: /notes/chores/chores-09.md#push-unimplemented-recheck-flag
[12]: /notes/chores/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[13]: /notes/chores/chores-06.md#generalize---scope-to-all-commands-design
[14]: /notes/chores/chores-06.md#--scope-continuation-0410
[15]: /notes/chores/chores-08.md#init--clone-redesign-0411
[16]: /notes/chores/chores-08.md#user-config-0411-3
[17]: /notes/chores/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[18]: /notes/chores/chores-05.md#capture---message-file-design-for-push-0375
[19]: /notes/chores/chores-06.md#vc-x1-validate-repo-command-design
[20]: /notes/chores/chores-06.md#bm-track-silent-when-clean-design
[21]: /notes/chores/chores-06.md#source-code-design-ref-convention-design
[22]: /notes/chores/chores-05.md#open-questions--tbd
[23]: /notes/chores/chores-03.md#per-lineper-thread-runtime-log-points-future
[24]: /notes/chores/chores-03.md#windows-symlink-support
[25]: /notes/chores/chores-01.md#refactor-and-add-desc-subcommand
[26]: /notes/chores/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[27]: /notes/chores/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores/chores-02.md#testing-results
[29]: /notes/chores/chores-02.md#shell-completion-discovery
[30]: /notes/chores/chores-11.md#refactor-scope-cli-cleanup-0540
[31]: /notes/chores/chores-11.md#feat-validate-todo--fix-todo-0550
[32]: /notes/chores/chores-11.md#docs-refine-cycle-protocol-0560
