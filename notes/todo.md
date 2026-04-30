# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

A bulleted list of the in-progress task's development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

- 0.41.1-0 plan + chores-08 + forks-multi-user + draft-reviews + vc-x1-init forward (done) [72]
- 0.41.1-1 shared repo_url module + clone/init migrate (done) [73]
- 0.41.1-2 clone reshape: <TARGET> + [NAME] + --scope code,bot|por (done) [73]
- 0.41.1-3 user config: ~/.config/vc-x1/config.toml + [default]/[github] (done) [74]
- 0.41.1-4 user config rewrite: account/category schema + literal values (done) [74]
- 0.41.1-5 init reshape: drop old flags + <TARGET> + [NAME] + --account + --repo (done) [73]
- 0.41.1-6 init refactor + symmetric .vc-config.toml schema [75]
  - -6.0 POR baseline integration tests + Fixture::new_por (done)
  - -6.1 literal lift: extract init_one / init_dual from init_with_symlink (done)
  - -6.2 extract create_repo + module reshape (repo_url → url, init_dual → create_dual) (done)
  - -6.3 extract push_repo (steps 7-9) + rename create_repo → create_local_repo (done)
  - -6.4 CLI subprocess integration tests (true `vc-x1` invocations) — add tests/ crate + harness (done)
  - -6.5 extract cross_ref_ochids + eliminate init_one + extract config-writing from create_local_repo + final create_dual collapse (done)
    - (1) drop config/gitignore params from create_local_repo; add write_{por,code,session}_config helpers in init.rs (done)
    - (2) extract cross_ref_ochids into repo_utils.rs (step 6 placeholder rewrite) (done)
    - (3) eliminate init_one — inline into init_with_symlink's POR branch (done)
    - (4) final create_dual collapse — drop stale step-N comments, tighten doc (done)
    - (5) fix: split create_local_repo into prepare_local_repo + commit_initial so role-config lands in the initial commit (regression from (1)) (done)
  - -6.6 --config=none|<path> flag (POR) + symmetric .vc-config.toml schema (dual) + dual-format reader for back-compat
  - -6.7 replace "Step N" log prefixes with single-word `label: body` convention (`bookmark`, `provision`, `colocate`, `cross-ref`, `symlink`, …); indent labels under per-side `code:` / `bot:` headers in dual
- 0.41.1-7 test_helpers::Fixture migration + downstream callers [73]
- 0.41.1 close-out [72]

## Todo

A markdown list of tasks to do in the near future, ordered
highest-priority first. Keep entries brief — 1-3 lines.
Detailed motivation, safety requirements, and ordering belong
in `notes/chores-NN.md` design subsections; link via `[N]` ref.

Items use lazy numbering — every entry begins with `1. `; the
markdown renderer auto-numbers them, so reorder/insert without
renumbering. Reference by displayed number ("let's work on #3").
1. **Rebase note — CLAUDE.md `### Per-file review checkpoints`.**
   Both `main` (0.42.0 work) and `init-clone-refactor`
   (0.41.1) authored this subsection independently —
   same intent, different wording. When 0.42.0 rebases on
   top of 0.41.1 at close-out, resolve CAREFULLY: don't
   take either side wholesale, reconcile to preserve the
   best of both. Likely conflict surface is the bullet
   list under "How to apply".
1. vc-x1 push: `--scope=code|bot|code,bot|<path>` flag.
   Lands in the 0.42.0 cycle alongside the sum-type
   refactor; state machine becomes scope-aware (single-
   side path skips `commit-claude`/bookmark-claude/
   `finalize-claude`; `Single(_)` is single-repo mode).
   [57],[60],[71]
1. vc-x1 clone: `--scope=code|bot|code,bot|<path>` flag.
   Parallel to `init --scope`; single-repo clone target
   via the path form. 0.42.0 cycle. [60],[71]
1. vc-x1 validate-desc / fix-desc:
   `--scope=code|bot|code,bot` flag. Same role vocabulary
   as elsewhere — `code` validates code's commits against
   bot, `bot` reverses, `code,bot` does both (new
   default). `Single(_)` errors here (validate compares
   two repos by definition). 0.42.0 cycle. [60],[71]
1. CommonArgs sweep — add `--scope=code|bot|code,bot|<path>`
   to `chid`/`desc`/`list`/`show` in one cycle (single
   shared `CommonArgs` change picks all four up). Drops
   the existing `-R`/`--repo` repeatable flag in favor of
   the new path form. 0.42.0 cycle. [60],[71]
1. Unify `.vc-config.toml` accessors onto Pattern B
   (typed struct + `load_from(path)`, like new
   `config::UserConfig` and `push::resolve_state_layout`).
   Replaces the map-typed helpers in `desc_helpers.rs` /
   `fix_desc.rs` / `validate_desc.rs` with a typed
   `WorkspaceConfig` struct. ~50 LOC, mechanical.
   Candidate for 0.41.2. [74]
1. Layered config precedence (user → workspace → CLI)
   once `WorkspaceConfig` is typed. Workspace can
   override `[github].owner` etc. for a specific project;
   init can't use the layer (chicken-and-egg) but
   post-init commands can. Depends on the
   `WorkspaceConfig` typed-struct refactor above.
   Candidate for 0.41.2. [74]
1. Help layout: force over-under everywhere. Apply
   `next_line_help(true)` at the root (or via the existing
   `cli_with_banner` walker) so every subcommand's `-h` /
   `--help` uses the same layout. Today clap auto-picks
   per-command based on the widest flag spec, so
   `sync -h` is two-column but `init -h` is over-under —
   visual inconsistency.
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
   it needs to include remotes, like remotes/origin/main. [54].
1. `vc-x1 init --dry-run` should bypass the
   `--repo-remote` path-existence preflight (currently fires
   before the dry-run early-return; observed dogfooding
   2026-04-24).
1. vc-x1 push: `--squash` flag. Squashes WC into `@-` via
   `--ignore-immutable` and force-pushes; needs
   `--force-with-lease`-equivalent + state-sanity preflight in
   place first. [57]
1. vc-x1 push: `--message-file PATH` flag. Git-style commit
   message file (first line = title, blank, rest = body).
   Alternative to `--title` + `--body`. [58]
1. Mirror `--check` / `--no-check` onto `vc-x1 push` (forwards
   through to the preflight `vc-x1 sync` invocation).
   0.37.1 hard-codes `--check`; default stays `--check`.
1. Add `validate-repo` subcommand: diagnostic that runs all
   `verify_*` checks (tracking, push state freshness, ochid
   integrity, conflicts, config sanity, working-copy state)
   and reports per-check pass/fail. Exit code = number of
   failed checks. Implementation: promote
   `verify_state_sanity` / `verify_completion_sanity` from
   push.rs to `common.rs`. [69]
1. sync: surface working-copy state in the up-to-date summary
   (per-repo pending-files count or compact stat). Wording-only
   fix shipped in 0.37.1; this is the design+impl. [54]
1. bm-track silent-when-clean refinement. Print on entry/exit
   only when state isn't fully tracked or when exit state
   differs from entry. [62]
1. "Oh shit" revert — post-success undo via `.vc-x1-ops/`
   anchor dir. Idea-stage; every repo-mutating command drops a
   pre-op snapshot, `vc-x1 undo` restores both repos. [57]
1. Restructure templates: replace separate `vc-template-x1` +
   `vc-template-x1.claude` repos with a single `vc-template-x1`
   that has `.claude/` as a subdir (covers `LICENSE-*` etc. for
   both sides in one place). Updates to `vc-x1 init` / `clone`
   needed for the new layout.
1. Source-code design ref sweep + CLAUDE.md codification:
   adopt section-name + `blob/main/...` URL pattern for source
   code refs to designs; codify in CLAUDE.md alongside the
   existing markdown ref conventions. Sweep targets:
   src/push.rs lines 4, 121, 645, 1219. [68]
1. Richer bookmark enumeration: per-bookmark remote presence + tracking status [52]
1. Per-line/per-thread runtime log points (future, maybe) [36]
1. Add Windows symlink support via `std::os::windows::fs::symlink_dir` [37]
1. Add "::" revision syntax for jj compatibility
1. Add -p, --parents, -c, --children so parent and child counts can be asymmetric
1. Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
1. Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]
1. Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [27]
1. Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [28],[29]
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

- CLAUDE.md refresh + memory migration (0.36.1) [49]
- Lift sync's inline test harness into shared `test_helpers` module (0.36.2) [51]
- Sync improvements: -R flag + quieter dry-run + sync-before-work discipline (0.36.3) [50]
- push subcommand scaffolding: flag surface, Stage enum, stub (0.37.0-0) [48]
- push state machine: state file, --status/--restart/--from, stage stubs (0.37.0-1) [48]
- push real stage bodies + jj-op snapshot rollback (0.37.0-2) [48]
- push integration tests + workspace-root refactor (0.37.0-3) [48]
- push interactivity: review prompt, $EDITOR, message persistence (0.37.0-4) [48]
- push polish: --dry-run, --step, non-tty detection, gitignore warning (0.37.0-5) [48]
- push docs + workflow migration — CLAUDE.md rewrite + README section (0.37.0) [48]
- First-dogfood polish for push: editor template, gitignore-fatal, sync --check, log prefix, quieter subprocess (0.37.1) [53]
- Temporary bookmark-tracking diagnostic probe on command entry/exit (0.37.2) [55]
- Fix bm-track bugs + rename + promote to permanent (0.37.3) [56]
- Capture squash-mode + scope design for push (0.37.4) [57]
- Capture --message-file design for push (0.37.5) [58]
- CLAUDE.md polish: markdown-anchor rule, shell-path brevity, state-file clearing, late-changes recipe trimmed (0.37.6) [59]
- Notes restructure: chores-06 + trim long todo entries (0.37.7) [64]
- Scope design refinements (0.37.8) [65]
- Bookmark tracking verification: shared helper + tests (0.38.0-0) [66]
- Bookmark tracking verification: wire into setup commands (0.38.0-1) [66]
- Bookmark tracking verification: wire into preflight commands (0.38.0-2) [66]
- Bookmark tracking verification: cycle close-out + dogfood validation (0.38.0) [66]
- Push hardening: state-sanity preflight on resume (0.39.0-0) [67]
- Push hardening: honest completion via post-completion verification (0.39.0-1) [67]
- Push hardening: cycle close-out, 0.39.0-2 skipped (0.39.0) [67]
- Scope generalization: init --repo-local + --repo-remote (0.40.0-1) [70]
- Scope generalization: init --scope=code|bot|code,bot (0.40.0-2) [70]
- Scope generalization: integration tests migrate onto init --repo-local (0.40.0-3) [70]
- Scope generalization: cycle close-out, init --scope foundation shipped (0.40.0) [70]
- Pre-commit checklist requires `--locked` for `cargo install` (0.41.0-1) [71]
- Scope continuation: sync --scope (0.41.0-2) [71]
- Scope continuation: capture --scope-everywhere direction (0.41.0-3) [71]
- Scope continuation: capture --scope sum-type vocabulary (0.41.0-4) [71]
- Scope continuation: cycle close-out — push/finalize work deferred to 0.42.0 (0.41.0) [71]

# References

[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[27]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores-02.md#testing-results
[29]: /notes/chores-02.md#shell-completion-discovery
[36]: /notes/chores-03.md#per-lineper-thread-runtime-log-points-future
[37]: /notes/chores-03.md#windows-symlink-support
[48]: /notes/chores-05.md#add-push-subcommand-0370
[49]: /notes/chores-05.md#claudemd-refresh--memory-migration-0361
[50]: /notes/chores-05.md#sync-improvements--single-repo-support--quieter-dry-run-0363
[51]: /notes/chores-05.md#test-harness-refactor-0362
[52]: /notes/chores-05.md#open-questions--tbd
[53]: /notes/chores-05.md#first-dogfood-polish-for-push-0371
[55]: /notes/chores-05.md#temporary-bookmark-tracking-diagnostic-probe-0372
[56]: /notes/chores-05.md#fix-bm-track-bugs--rename--promote-to-permanent-0373
[57]: /notes/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[58]: /notes/chores-05.md#capture---message-file-design-for-push-0375
[54]: /notes/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[59]: /notes/chores-05.md#claudemd-polish-0376
[60]: /notes/chores-06.md#generalize---scope-to-all-commands-design
[61]: /notes/chores-06.md#push-hardening-state--stage-sanity-design
[62]: /notes/chores-06.md#bm-track-silent-when-clean-design
[63]: /notes/chores-06.md#non-tracking-remote-bookmark-detection-design
[64]: /notes/chores-06.md#notes-restructure-chores-06--trim-long-todo-entries-0377
[65]: /notes/chores-06.md#scope-design-refinements-0378
[66]: /notes/chores-06.md#bookmark-tracking-verification-0380
[67]: /notes/chores-06.md#push-hardening-state--stage-sanity-0390
[68]: /notes/chores-06.md#source-code-design-ref-convention-design
[69]: /notes/chores-06.md#vc-x1-validate-repo-command-design
[70]: /notes/chores-06.md#generalize---scope-across-commands-0400
[71]: /notes/chores-06.md#--scope-continuation-0410
[72]: /notes/chores-08.md#init--clone-redesign-0411
[73]: /notes/chores-08.md#cycle-structure--multi-step
[74]: /notes/chores-08.md#user-config-0411-3
[75]: /notes/chores-08.md#operations
