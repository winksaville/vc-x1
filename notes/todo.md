# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

A bulleted list of the in-progress task's development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

## Todo

A markdown list of tasks to do in the near future, ordered
highest-priority first. Keep entries brief — 1-3 lines.
Detailed motivation, safety requirements, and ordering belong
in `notes/chores-NN.md` design subsections; link via `[N]` ref.

Items use lazy numbering — every entry begins with `1. `; the
markdown renderer auto-numbers them, so reorder/insert without
renumbering. Reference by displayed number ("let's work on #3").
1. **Ops layer / CLI decoupling — sweep remaining
   subcommands.** Init ported in 0.44.0 as the worked
   example (`Context` + `InitParams`, `From<&InitArgs>`
   at the binary edge, op body reads `params.account`
   not `args.account.account`). Remaining: sync, chid,
   push, clone, desc, fix-desc, validate-desc, list,
   show, finalize, symlink — each its own cycle, same
   shape (`<Mod>Params` + `From<&XArgs>`). [80]
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
   hardens. [82],[83]
1. **Rebase note — CLAUDE.md `### Per-file review checkpoints`.**
   Both `main` (0.42.0 work) and `init-clone-refactor`
   (0.41.1) authored this subsection independently —
   same intent, different wording. When 0.42.0 rebases on
   top of 0.41.1 at close-out, resolve CAREFULLY: don't
   take either side wholesale, reconcile to preserve the
   best of both. Likely conflict surface is the bullet
   list under "How to apply".
1. **Symmetric `.vc-config.toml` schema.** Add `code = "/"`
   and `bot = "/.claude"` (workspace-root-relative paths) so
   both repos read from the same shape. Side detection walks
   up from cwd via existing `find_workspace_root`, then maps
   cwd-relative-to-root onto the configured `code` / `bot`
   paths. Cwd-basename match is a fast-path shortcut at
   role-root level only — subdirs need the walk-up. Was the
   deferred half of `0.41.1-6`'s title. Migration story for
   existing workspaces TBD at design time. [75]
1. **`test_helpers::Fixture` migration + downstream callers.**
   Plus rename `Fixture` → `TestFixtureDual` and `FixturePor`
   → `TestFixturePor` so call sites carry the test-only
   signal that `#[cfg(test)] mod test_helpers` doesn't
   communicate. Was `0.41.1-7`. [73]
1. **`--scope` sweep continuation: finalize + Single(_)
   dogfood.** Carry-over from 0.42.0 cycle (closed at -4.7;
   -5/-7 deferred). Two pieces: `vc-x1 finalize --scope`
   replacing `--repo` (was 0.42.0-5), and `Single(_)`
   end-to-end dogfood validation (was 0.42.0-7). The third
   originally-paired item, `vc-x1 clone --scope` (was
   0.42.0-6), is tracked in its own entry below. Design
   lives in chores-07. [76]
1. **init dual|por arg split.** Via `#[command(flatten)]` of
   `ProvisionOptionFlagBundle` (built in -6.7) +
   `provision_side(role, …)` shared helper. CLI surface
   decision (subcommands `init dual|por` vs preserved
   `--scope` flag with manual two-pass parse) deferred to
   design time. Was `0.41.1-6.9`; may or may not happen.
1. Decide merge direction for `init-clone-refactor`
   (recovered 2026-05-02). Rebase `0.41.1` onto
   `0.42.0` → `0.43.0`-or-`0.42.1`, vs force-rewrite
   main, vs other topology. [78]
1. Cross-file chores-NN ordering sanity pass at
   `init-clone-refactor` merge time. Bring chores-08.md
   (0.41.1 per-step subsections) onto main alongside
   chores-06/07; normalize section ordering across all
   three. Tied to merge-direction decision above. [78]
1. Add a vc-x1 validate-repo?
1. vc-x1 push: `--scope=code|bot|code,bot|<path>` flag.
   Was 0.42.0-4 (deferred when cycle pivoted to icr
   rebase work; cycle closed at -4.7). State machine
   becomes scope-aware (single-side path skips
   `commit-claude`/bookmark-claude/`finalize-claude`;
   `Single(_)` is single-repo mode).
   [57],[60],[71],[72]
1. vc-x1 clone: `--scope=code|bot|code,bot|<path>` flag.
   Parallel to `init --scope`; single-repo clone target
   via the path form. Was 0.42.0-6 (deferred at -4.7
   close-out). [60],[71],[72]
1. vc-x1 validate-desc / fix-desc:
   `--scope=code|bot|code,bot` flag. Same role vocabulary
   as elsewhere — `code` validates code's commits against
   bot, `bot` reverses, `code,bot` does both (new
   default). `Single(_)` errors here (validate compares
   two repos by definition). [60],[71],[72]
1. CommonArgs sweep — add `--scope=code|bot|code,bot|<path>`
   to `chid`/`desc`/`list`/`show` in one cycle (single
   shared `CommonArgs` change picks all four up). Drops
   the existing `-R`/`--repo` repeatable flag in favor of
   the new path form. [60],[71],[76]
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

- Substep protocol formalization + jj revsets cheatsheet (0.42.0-4.5) [77]
- init-clone-refactor recovery + post-mortem playbook (0.42.0-4.6) [78]
- Init-clone-refactor rebase landing — main rebased + .claude re-paired (0.42.0-4.7) [79]
- 0.42.0 cycle close-out at -4.7 — partial --scope sweep, continuation deferred [81]
- Test-module extraction across init/push/sync/common (0.43.0) [84]
- InitParams + Context — init op decoupling worked example (0.44.0) [80]

# References

[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[27]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores-02.md#testing-results
[29]: /notes/chores-02.md#shell-completion-discovery
[36]: /notes/chores-03.md#per-lineper-thread-runtime-log-points-future
[37]: /notes/chores-03.md#windows-symlink-support
[52]: /notes/chores-05.md#open-questions--tbd
[54]: /notes/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[57]: /notes/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[58]: /notes/chores-05.md#capture---message-file-design-for-push-0375
[60]: /notes/chores-06.md#generalize---scope-to-all-commands-design
[62]: /notes/chores-06.md#bm-track-silent-when-clean-design
[68]: /notes/chores-06.md#source-code-design-ref-convention-design
[69]: /notes/chores-06.md#vc-x1-validate-repo-command-design
[71]: /notes/chores-06.md#--scope-continuation-0410
[72]: /notes/chores-08.md#init--clone-redesign-0411
[73]: /notes/chores-08.md#cycle-structure--multi-step
[74]: /notes/chores-08.md#user-config-0411-3
[75]: /notes/chores-08.md#operations
[76]: /notes/chores-07.md#--scope-enum-refactor-0420
[77]: /notes/chores-07.md#substep-protocol-formalization-0420-45
[78]: /notes/chores-07.md#init-clone-refactor-recovery-0420-46
[79]: /notes/chores-09.md#init-clone-refactor-rebase-landing-0420-47
[80]: /notes/chores-09.md#ops-layer-architecture-forward-looking
[81]: /notes/chores-09.md#0420-close-out
[82]: /notes/forks-multi-user.md
[83]: /notes/bot-data-formats.md
[84]: /notes/chores-09.md#test-module-extraction-0430
