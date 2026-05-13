# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

A bulleted list of the in-progress task's development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

**chid/desc/list/show — CommonArgs sweep** (options_flags
extraction + `--scope` + Context+Params port). Re-scoped at
0.49.0-1 from the original "finish Migration A" plan, B-first so
the Context+Params ports land once against the final `CommonArgs`
shape.

Design:
[chores-10.md](chores-10.md#chore-open-0490--finish-migration-a-0490-0)
+ the 0.49.0-1 re-scope subsection; prior `--scope` design in
[chores-06](chores-06.md#generalize---scope-across-commands-0400),
[chores-07](chores-07.md#--scope-enum-refactor-0420).

   - 0.49.0-0 plan + version bump + chores section + ladder (done)
   - 0.49.0-1 options_flags extraction — relocate `CommonArgs`
     → `options_flags/common_args.rs`. Kept separate; no `-1`
     close-out commit.
     - 0.49.0-1.1 the relocation + all four importers (done)
     - 0.49.0-1.2 docs: slim ARCHITECTURE.md; start chores-10 (done)
   - 0.49.0-2 `-R`/`--repo` → `-s`/`--scope` for `chid` /
     `desc` / `list` / `show`.
     - 0.49.0-2.1 open the step (done)
       - tidy `## Todo` (drop the two now-in-progress items).
       - add CLAUDE.md rule — a picked-up `## Todo` item is
         deleted when it goes `## In Progress`.
       - backfill `-1.1` / `-1.2` chores `Commits:` refs.
     - 0.49.0-2.2 the rollout — code (done)
       - kept `-R` as single path (`Option<PathBuf>`); added
         `-s`/`--scope` (`Option<Scope>` via
         `parse_scope_roles`, keyword-only — `-s <path>` is a
         future Todo).
       - the two compose (no `conflicts_with`): `-R` overrides
         workspace root, `-s` selects sides within it. New
         `common::resolve_repos(repo, scope)` does the match;
         `for_each_repo` takes a resolved `Vec<PathBuf>`.
       - defaults preserve today: no flag → `[.]`, `-R foo`
         alone → `[foo]`.
       - scope: four subcommand bodies + `--help`; tests;
         CLAUDE.md `chid -R .,.claude -L` → `chid -s code,bot -L`.
     - 0.49.0-2.3 docs (done)
       - `README.md` `### Multi-repo queries` rewritten to lead
         with `-s code,bot`; every example updated. The prior
         `-R .,.claude` / `-R . -R .claude` forms no longer
         parse since `-R` is now single-path.
       - `-R PATH` retained for single-path use and as the
         workspace-root override that composes with `-s`
         (`-R ../other -s code,bot`).
       - `ARCHITECTURE.md`: `resolve_repos` added to the
         `common.rs` helper list with a one-line note on what
         it composes.
       - `notes/` swept; no stale `-R .,.claude` for these four.
     - 0.49.0-2.4 unify prose form in CLAUDE.md (current)
       - new top-level `## Prose form` section as the single
         source of truth for the intro+bullets shape across
         commit bodies / chores / todo / done / doc comments.
       - slim `## Commit Message Style`, `### Chores section
         content`, and `### Doc comments…` to reference it.
       - surfaced as process drift while writing the `-2.3`
         chores section; deferred from -2.3 to keep that
         commit scoped to the `-s/--scope` docs sweep.
   - 0.49.0-3 chid Context+Params port + introduce `CommonParams`
   - 0.49.0-4 desc Context+Params port
   - 0.49.0-5 list Context+Params port
   - 0.49.0-6 show Context+Params port (`TryFrom`, `FileLimit` parse)
   - 0.49.0 close-out — drop suffix, todo→Done (Context+Params
     port 12/12 + CommonArgs sweep), README + ARCHITECTURE.md

## Todo

A markdown list of tasks to do in the near future, ordered
highest-priority first. Keep entries brief — 1-3 lines.
Detailed motivation, safety requirements, and ordering belong
in `notes/chores-NN.md` design subsections; link via `[N]` ref.

Items use lazy numbering — every entry begins with `1. `; the
markdown renderer auto-numbers them, so reorder/insert without
renumbering. Reference by displayed number ("let's work on #3").
1. **single-field `options_flags` leaves → `value` field.**
   `0.47.0` introduced the convention (single-field leaf names
   its field `value`, declares the flag via `#[arg(long = "…")]`,
   so consumers read `args.<leaf>.value` not `args.<leaf>.<leaf>`)
   on the new `squash` leaf. Sweep the pre-existing single-field
   leaves to match: `scope`, `repo`, `dry_run`, `private`,
   `account`, `config`, `use_template` + their consumers
   (`init.rs`, tests).
1. **Drop `-R`/`--repo` from `CommonArgs` once `-s`/`--scope` is
   established.**
   - `0.49.0-2.2` kept `-R` (single path) alongside the new `-s`
     (roles, composes with `-R` as workspace root) as a
     backwards-compat alias for the migration period.
   - Once users / scripts have moved, drop `-R` — `--scope` then
     covers both path and role forms (`parse_scope` already
     handles both; only `parse_scope_roles` would go away).
1. **`-s <path>` and `-s <path>,roles` workspace-root override.**
   - Today `-s` is keyword-only (`parse_scope_roles` rejects
     paths, pointing users at `-R`).
   - Future: have `parse_scope_roles` accept a comma list mixing
     one path + role keywords, where the path is the workspace
     root.
   - `vc-x1 chid -s ../foo,bot,code` would resolve to `[../foo,
     ../foo/.claude]` (today `-R ../foo -s code,bot`).
   - Decisions to make at design time: comma-list syntax, error
     cases (multiple paths, path without roles vs
     path-as-`Scope::Single`), and whether to also accept bare
     `-s <path>` as a synonym for `-R <path>`.
1. **por/dual parity + bidirectional conversion.** Make
   `por` and `dual` first-class equals (dual is primary
   today, por bolted on); add `por → dual` / `dual → por`
   conversion. Builds on the `--scope` rollout below.
   Pre-design; goal + open questions in the stub. [[4]]
1. **forks-multi-user + bot-data-formats follow-through.**
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
1. **Symmetric `.vc-config.toml` schema.** Add `code = "/"`
   and `bot = "/.claude"` (workspace-root-relative paths) so
   both repos read from the same shape. Side detection walks
   up from cwd via existing `find_workspace_root`, then maps
   cwd-relative-to-root onto the configured `code` / `bot`
   paths. Cwd-basename match is a fast-path shortcut at
   role-root level only — subdirs need the walk-up. Was the
   deferred half of `0.41.1-6`'s title. Migration story for
   existing workspaces TBD at design time. [[7]]
1. **`test_helpers::Fixture` migration + downstream callers.**
   Plus rename `Fixture` → `TestFixtureDual` and `FixturePor`
   → `TestFixturePor` so call sites carry the test-only
   signal that `#[cfg(test)] mod test_helpers` doesn't
   communicate. Was `0.41.1-7`. [[8]]
1. **`--scope` sweep continuation: finalize + Single(_)
   dogfood.** Carry-over from 0.42.0 cycle (closed at -4.7;
   -5/-7 deferred). Two pieces: `vc-x1 finalize --scope`
   replacing `--repo` (was 0.42.0-5), and `Single(_)`
   end-to-end dogfood validation (was 0.42.0-7). The third
   originally-paired item, `vc-x1 clone --scope` (was
   0.42.0-6), is tracked in its own entry below. Design
   lives in chores-07. [[9]]
1. **init dual|por arg split.** Via `#[command(flatten)]` of
   `ProvisionOptionFlagBundle` (built in -6.7) +
   `provision_side(role, …)` shared helper. CLI surface
   decision (subcommands `init dual|por` vs preserved
   `--scope` flag with manual two-pass parse) deferred to
   design time. Was `0.41.1-6.9`; may or may not happen.
1. Cross-file `chores-NN.md` ordering sanity pass.
   `chores-08.md` (the 0.41.1 cycle) landed on `main` via
   the `0.42.0-4.7` rebase; check that section ordering
   across `chores-06`/`-07`/`-08`/`-09` is chronologically
   coherent and normalize if not. Low priority.
1. Add a vc-x1 validate-repo?
1. vc-x1 push: rework the two bookmark parameters.
   `PushArgs` has `bookmark_pos` (positional `BOOKMARK`) +
   `bookmark` (`--bookmark` flag) for one logical value,
   forcing an `or_else` in `From<&PushArgs>`. Collapse to a
   single positional with `--bookmark` as a true clap alias,
   or drop one spelling. [[10]]
1. vc-x1 push: `--recheck` — implement or remove. Parsed by
   `PushArgs`, never read; mirrored into `PushParams` with
   `#[allow(dead_code)]`. Either wire the
   skip-preflight-on-resume behavior or drop the flag. [[11]]
1. vc-x1 push: `--scope=code|bot|code,bot|<path>` flag.
   Was 0.42.0-4 (deferred when cycle pivoted to icr
   rebase work; cycle closed at -4.7). State machine
   becomes scope-aware (single-side path skips
   `commit-claude`/bookmark-claude/`finalize-claude`;
   `Single(_)` is single-repo mode).
   [[12]],[[13]],[[14]],[[15]]
1. vc-x1 clone: `--scope=code|bot|code,bot|<path>` flag.
   Parallel to `init --scope`; single-repo clone target
   via the path form. Was 0.42.0-6 (deferred at -4.7
   close-out). [[13]],[[14]],[[15]]
1. vc-x1 validate-desc / fix-desc:
   `--scope=code|bot|code,bot` flag. Same role vocabulary
   as elsewhere — `code` validates code's commits against
   bot, `bot` reverses, `code,bot` does both (new
   default). `Single(_)` errors here (validate compares
   two repos by definition). [[13]],[[14]],[[15]]
1. Unify `.vc-config.toml` accessors onto Pattern B
   (typed struct + `load_from(path)`, like new
   `config::UserConfig` and `push::resolve_state_layout`).
   Replaces the map-typed helpers in `desc_helpers.rs` /
   `fix_desc.rs` / `validate_desc.rs` with a typed
   `WorkspaceConfig` struct. ~50 LOC, mechanical.
   Candidate for 0.41.2. [[16]]
1. Layered config precedence (user → workspace → CLI)
   once `WorkspaceConfig` is typed. Workspace can
   override `[github].owner` etc. for a specific project;
   init can't use the layer (chicken-and-egg) but
   post-init commands can. Depends on the
   `WorkspaceConfig` typed-struct refactor above.
   Candidate for 0.41.2. [[16]]
1. Help layout: force over-under everywhere. Apply
   `next_line_help(true)` at the root (or via the existing
   `cli_with_banner` walker) so every subcommand's `-h` /
   `--help` uses the same layout. Today clap auto-picks
   per-command based on the widest flag spec, so
   `sync -h` is two-column but `init -h` is over-under —
   visual inconsistency.
1. Replace "Step N" log prefixes with single-word
   `label: body` convention (`bookmark`, `provision`,
   `colocate`, `cross-ref`, `symlink`, …); indent labels
   under per-side `code:` / `bot:` headers in dual.
   Originally planned as 0.41.1-6.7; deferred.
1. Consider renaming the `.vc-config.toml` `[workspace]`
   section. Rust readers expect `[workspace]` to mean a
   Cargo workspace, which a vc-x1 dual-repo isn't.
   Candidates: `[repo-list]`, `[project]`, `[dual-repo]`.
   Breaking change — needs migration story (read both
   names during a transition cycle, or one-shot rewrite
   in `vc-x1 sync`/`init` on first contact). Drives the
   broader "stop saying workspace in user-facing surfaces"
   sweep.
1. Add `status` (alias `st`) subcommand: `jj st` across both
   repos in one shot. Uses `--scope` from day one. This is
   natural home for the working-copy signal called out and
   it needs to include remotes, like remotes/origin/main. [[17]].
1. `vc-x1 init --dry-run` should bypass the
   `--repo-remote` path-existence preflight (currently fires
   before the dry-run early-return; observed dogfooding
   2026-04-24).
1. vc-x1 push: `--squash` flag. Squashes WC into `@-` via
   `--ignore-immutable` and force-pushes; needs
   `--force-with-lease`-equivalent + state-sanity preflight in
   place first. [[12]]
1. vc-x1 push: `--message-file PATH` flag. Git-style commit
   message file (first line = title, blank, rest = body).
   Alternative to `--title` + `--body`. [[18]]
1. Mirror `--check` / `--no-check` onto `vc-x1 push` (forwards
   through to the preflight `vc-x1 sync` invocation).
   0.37.1 hard-codes `--check`; default stays `--check`.
1. Add `validate-repo` subcommand: diagnostic that runs all
   `verify_*` checks (tracking, push state freshness, ochid
   integrity, conflicts, config sanity, working-copy state)
   plus chores↔commit consistency — every `[N]:` anchor
   resolves, and each chores `##` section's recorded title
   matches its `Commits:` commit's title — and reports
   per-check pass/fail. Exit code = number of failed checks.
   Implementation: promote `verify_state_sanity` /
   `verify_completion_sanity` from push.rs to `common.rs`. [[19]]
1. sync: surface working-copy state in the up-to-date summary
   (per-repo pending-files count or compact stat). Wording-only
   fix shipped in 0.37.1; this is the design+impl. [[17]]
1. bm-track silent-when-clean refinement. Print on entry/exit
   only when state isn't fully tracked or when exit state
   differs from entry. [[20]]
1. "Oh shit" revert — post-success undo via `.vc-x1-ops/`
   anchor dir. Idea-stage; every repo-mutating command drops a
   pre-op snapshot, `vc-x1 undo` restores both repos. [[12]]
1. Restructure templates: replace separate `vc-template-x1` +
   `vc-template-x1.claude` repos with a single `vc-template-x1`
   that has `.claude/` as a subdir (covers `LICENSE-*` etc. for
   both sides in one place). Updates to `vc-x1 init` / `clone`
   needed for the new layout.
1. Source-code design ref sweep + CLAUDE.md codification:
   adopt section-name + `blob/main/...` URL pattern for source
   code refs to designs; codify in CLAUDE.md alongside the
   existing markdown ref conventions. Sweep targets:
   src/push.rs lines 4, 121, 645, 1219. [[21]]
1. Richer bookmark enumeration: per-bookmark remote presence + tracking status [[22]]
1. Per-line/per-thread runtime log points (future, maybe) [[23]]
1. Add Windows symlink support via `std::os::windows::fs::symlink_dir` [[24]]
1. Add "::" revision syntax for jj compatibility
1. Add -p, --parents, -c, --children so parent and child counts can be asymmetric
1. Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
1. Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [[25]],[[26]]
1. Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [[27]]
1. Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [[28]],[[29]]
1. Test-tempdir override resolution chain. Both
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

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- InitParams + Context — init subcommand-layer decoupling worked example (0.44.0) [[1]]
- ARCHITECTURE.md + subcommand-layer terminology reconciliation (0.45.0) [[30]]
- finalize subcommand-layer migration — Context.log + TryFrom (0.46.0) [[31]]
- finalize Migration B — squash options_flags leaf (0.47.0) [[3]]
- Migration A sweep — subcommand-layer ports: symlink/clone/sync/validate-desc/fix-desc/push (0.48.0) [[2]]
- docs: por-dual capture + icr cleanup (0.48.1) [[32]]
- docs: chores edit list → commit message (0.48.2) [[33]]
- docs: chores-09 → new shape (0.48.3) [[34]]
- docs: renumber todo.md + chores-09 refs (0.48.4) [[35]]

# References

[1]: /ARCHITECTURE.md
[2]: /notes/chores-09.md#chore-open-0480-cycle--migration-a-sweep-0480-0
[3]: /notes/chores-09.md#refactor-finalize---squash--options_flags-leaf-0470
[4]: /notes/por-dual-parity.md
[5]: /notes/forks-multi-user.md
[6]: /notes/bot-data-formats.md
[7]: /notes/chores-08.md#operations
[8]: /notes/chores-08.md#cycle-structure--multi-step
[9]: /notes/chores-07.md#--scope-enum-refactor-0420
[10]: /notes/chores-09.md#push-dual-bookmark-parameters
[11]: /notes/chores-09.md#push-unimplemented-recheck-flag
[12]: /notes/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[13]: /notes/chores-06.md#generalize---scope-to-all-commands-design
[14]: /notes/chores-06.md#--scope-continuation-0410
[15]: /notes/chores-08.md#init--clone-redesign-0411
[16]: /notes/chores-08.md#user-config-0411-3
[17]: /notes/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[18]: /notes/chores-05.md#capture---message-file-design-for-push-0375
[19]: /notes/chores-06.md#vc-x1-validate-repo-command-design
[20]: /notes/chores-06.md#bm-track-silent-when-clean-design
[21]: /notes/chores-06.md#source-code-design-ref-convention-design
[22]: /notes/chores-05.md#open-questions--tbd
[23]: /notes/chores-03.md#per-lineper-thread-runtime-log-points-future
[24]: /notes/chores-03.md#windows-symlink-support
[25]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[26]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[27]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores-02.md#testing-results
[29]: /notes/chores-02.md#shell-completion-discovery
[30]: /notes/chores-09.md#docs-architecture-md--subcommand-layer-naming-0450
[31]: /notes/chores-09.md#refactor-finalize--context--finalizeparams-0460
[32]: /notes/chores-09.md#docs-por-dual-capture--icr-cleanup-0481
[33]: /notes/chores-09.md#docs-chores-edit-list--commit-message-0482
[34]: /notes/chores-09.md#docs-chores-09--new-shape-0483
[35]: /notes/chores-09.md#docs-renumber-todomd--chores-09-refs-0484
