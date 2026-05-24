# Todo Backlog

 Non-prioritized `## Todo` entries — the long tail. When an
 entry becomes a priority, move it (and any refs it cites)
 to `notes/todo.md > ## Todo` and add a tier line under the
 right `### Pn` in `notes/todo.md > ## Priorities`.

 Same formatting rules as `notes/todo.md > ## Todo` — see
 `notes/README.md > ## Todo format`. Run
 `vc-x1 fix-todo --no-dry-run notes/todo-backlog.md` to
 renumber.

## Todo

1. **vc-x1 push: `--merge` flag (close-out shape).** Teach
   push to set up the non-FF merge ("trapezoid") shape
   itself instead of requiring the user to pre-rebase
   before invoking push. Sibling to `--squash` (#23);
   both are close-out shape choices made at push time.
   Dogfooded manually in `0.56.0`. [[1]]
   - Preparation (`-0`): open chores section; settle flag
     surface (`--merge` mutually exclusive with `--squash`
     / `--keep`?) and preflight checks (cycle tip is
     descendant of `<prev>`, not already a merge, sub-tip
     reachable from current bookmark, etc.).
   - Work-1 (`-1`): parse the flag through `PushArgs` →
     `PushParams`; reject mutually-exclusive combinations.
   - Work-2 (`-2`): implement the rebase setup inside
     push's state machine — locate `<tip>` and
     `<sub-tip>`, run the equivalent of `jj rebase -r
     <tip> -d <prev> -d <sub-tip>`; ensure the `.claude`
     commit emits a multi-line `ochid:` covering every
     sub-step in the merge.
   - Work-3 (`-3`): tests + dogfood on a real cycle.
   - Close-out: finalize chores; `## Done` entry; CLAUDE.md
     `### Pushing` text updated to describe the flag.
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
4. **Audit hardcoded `.claude` in diagnostics / logging.**
   `vc-x1 finalize --repo .bot` on `../dicom-rs`
   (2026-05-14) honored `--repo` throughout actual
   operations, but `bm-track` still emitted
   `.claude(main)=no-jj` — diagnostic strings have
   `.claude` baked in. Cosmetic today; load-bearing once
   bot-repo name becomes configurable (see Symmetric
   `.vc-config.toml` schema entry below).
5. **forks-multi-user + bot-data-formats follow-through.**
   Design captured across two notes; concrete work to
   land when a cycle picks it up. Major pieces:
   multi-line `ochid:` parser/emitter; bot-side
   immutability enforcement; URL-shaped ochid (per-user
   / cross-repo); vendor-subdir layout
   (`.bot/<vendor>/<version>/<id>.<ext>`) +
   flat-to-vendor migration; `.claude/` → `.bot/` rename
   (gated behind symmetric `.vc-config.toml` schema).
   Each piece is its own future TODO when the design
   hardens. [[2]],[[3]]
6. **Symmetric `.vc-config.toml` schema.** Add `code = "/"`
   and `bot = "/.claude"` (workspace-root-relative paths) so
   both repos read from the same shape. Side detection walks
   up from cwd via existing `find_workspace_root`, then maps
   cwd-relative-to-root onto the configured `code` / `bot`
   paths. Cwd-basename match is a fast-path shortcut at
   role-root level only — subdirs need the walk-up. Was the
   deferred half of `0.41.1-6`'s title. Migration story for
   existing workspaces TBD at design time. [[4]]
7. **`test_helpers::Fixture` migration + downstream callers.**
   Plus rename `Fixture` → `TestFixtureDual` and `FixturePor`
   → `TestFixturePor` so call sites carry the test-only
   signal that `#[cfg(test)] mod test_helpers` doesn't
   communicate. Was `0.41.1-7`. [[5]]
8. **`vc-x1 finalize --scope` flag.** Replace `--repo`
   with the role vocabulary used elsewhere
   (`code|bot|code,bot`). Carry-over from the 0.42.0
   `--scope` sweep (was 0.42.0-5; deferred at -4.7
   close-out). The paired `Single(_)` dogfood item
   (0.42.0-7) is moot after `0.53.0` — `Single(_)`
   deleted. Design lives in chores-07. [[6]]
9. Cross-file `chores-NN.md` ordering sanity pass.
   `chores-08.md` (the 0.41.1 cycle) landed on `main` via
   the `0.42.0-4.7` rebase; check that section ordering
   across `chores-06`/`-07`/`-08`/`-09` is chronologically
   coherent and normalize if not. Low priority.
10. Add a vc-x1 validate-repo?
11. vc-x1 push: rework the two bookmark parameters.
    `PushArgs` has `bookmark_pos` (positional `BOOKMARK`) +
    `bookmark` (`--bookmark` flag) for one logical value,
    forcing an `or_else` in `From<&PushArgs>`. Collapse to a
    single positional with `--bookmark` as a true clap alias,
    or drop one spelling. [[7]]
12. vc-x1 push: `--recheck` — implement or remove. Parsed by
    `PushArgs`, never read; mirrored into `PushParams` with
    `#[allow(dead_code)]`. Either wire the
    skip-preflight-on-resume behavior or drop the flag. [[8]]
13. vc-x1 push: `--scope=code|bot|code,bot` flag.
    Was 0.42.0-4 (deferred when cycle pivoted to icr
    rebase work; cycle closed at -4.7). State machine
    becomes scope-aware — single-side scope skips
    `commit-claude`/bookmark-claude/`finalize-claude`.
    [[9]],[[10]],[[11]],[[12]]
14. vc-x1 clone: `--scope=code|bot|code,bot` flag.
    Parallel to `init --scope` for role selection;
    topology (`--por` vs dual) is the separate `--por`
    boolean. Was 0.42.0-6 (deferred at -4.7
    close-out). [[10]],[[11]],[[12]]
15. vc-x1 validate-desc / fix-desc:
    `--scope=code|bot|code,bot` flag. Same role vocabulary
    as elsewhere — `code` validates code's commits against
    bot, `bot` reverses, `code,bot` does both (new
    default). [[10]],[[11]],[[12]]
16. Unify `.vc-config.toml` accessors onto Pattern B
    (typed struct + `load_from(path)`, like new
    `config::UserConfig` and `push::resolve_state_layout`).
    Replaces the map-typed helpers in `desc_helpers.rs` /
    `fix_desc.rs` / `validate_desc.rs` with a typed
    `WorkspaceConfig` struct. ~50 LOC, mechanical.
    Candidate for 0.41.2. [[13]]
17. Layered config precedence (user → workspace → CLI)
    once `WorkspaceConfig` is typed. Workspace can
    override `[github].owner` etc. for a specific project;
    init can't use the layer (chicken-and-egg) but
    post-init commands can. Depends on the
    `WorkspaceConfig` typed-struct refactor above.
    Candidate for 0.41.2. [[13]]
18. Help layout: force over-under everywhere. Apply
    `next_line_help(true)` at the root (or via the existing
    `cli_with_banner` walker) so every subcommand's `-h` /
    `--help` uses the same layout. Today clap auto-picks
    per-command based on the widest flag spec, so
    `sync -h` is two-column but `init -h` is over-under —
    visual inconsistency.
19. Replace "Step N" log prefixes with single-word
    `label: body` convention (`bookmark`, `provision`,
    `colocate`, `cross-ref`, `symlink`, …); indent labels
    under per-side `code:` / `bot:` headers in dual.
    Originally planned as 0.41.1-6.7; deferred.
20. Consider renaming the `.vc-config.toml` `[workspace]`
    section. Rust readers expect `[workspace]` to mean a
    Cargo workspace, which a vc-x1 dual-repo isn't.
    Candidates: `[repo-list]`, `[project]`, `[dual-repo]`.
    Breaking change — needs migration story (read both
    names during a transition cycle, or one-shot rewrite
    in `vc-x1 sync`/`init` on first contact). Drives the
    broader "stop saying workspace in user-facing surfaces"
    sweep.
21. Add `status` (alias `st`) subcommand: `jj st` across both
    repos in one shot. Uses `--scope` from day one. This is
    natural home for the working-copy signal called out and
    it needs to include remotes, like remotes/origin/main. [[14]].
22. `vc-x1 init --dry-run` should bypass the
    `--repo-remote` path-existence preflight (currently fires
    before the dry-run early-return; observed dogfooding
    2026-04-24).
23. vc-x1 push: `--squash` flag. Squashes WC into `@-` via
    `--ignore-immutable` and force-pushes; needs
    `--force-with-lease`-equivalent + state-sanity preflight in
    place first. [[9]]
24. vc-x1 push: `--message-file PATH` flag. Git-style commit
    message file (first line = title, blank, rest = body).
    Alternative to `--title` + `--body`. [[15]]
25. Mirror `--check` / `--no-check` onto `vc-x1 push` (forwards
    through to the preflight `vc-x1 sync` invocation).
    0.37.1 hard-codes `--check`; default stays `--check`.
26. Add `validate-repo` subcommand: diagnostic that runs all
    `verify_*` checks (tracking, push state freshness, ochid
    integrity, conflicts, config sanity, working-copy state)
    plus chores↔commit consistency — every `[N]:` anchor
    resolves, and each chores `##` section's recorded title
    matches its `Commits:` commit's title — and reports
    per-check pass/fail. Exit code = number of failed checks.
    Implementation: promote `verify_state_sanity` /
    `verify_completion_sanity` from push.rs to `common.rs`. [[16]]
27. sync: surface working-copy state in the up-to-date summary
    (per-repo pending-files count or compact stat). Wording-only
    fix shipped in 0.37.1; this is the design+impl. [[14]]
28. bm-track silent-when-clean refinement. Print on entry/exit
    only when state isn't fully tracked or when exit state
    differs from entry. [[17]]
29. "Oh shit" revert — post-success undo via `.vc-x1-ops/`
    anchor dir. Idea-stage; every repo-mutating command drops a
    pre-op snapshot, `vc-x1 undo` restores both repos. [[9]]
30. Restructure templates: replace separate `vc-template-x1` +
    `vc-template-x1.claude` repos with a single `vc-template-x1`
    that has `.claude/` as a subdir (covers `LICENSE-*` etc. for
    both sides in one place). Updates to `vc-x1 init` / `clone`
    needed for the new layout.
31. Source-code design ref sweep + CLAUDE.md codification:
    adopt section-name + `blob/main/...` URL pattern for source
    code refs to designs; codify in CLAUDE.md alongside the
    existing markdown ref conventions. Sweep targets:
    src/push.rs lines 4, 121, 645, 1219. [[18]]
32. Richer bookmark enumeration: per-bookmark remote presence + tracking status [[19]]
33. Per-line/per-thread runtime log points (future, maybe) [[20]]
34. Add Windows symlink support via `std::os::windows::fs::symlink_dir` [[21]]
35. Add "::" revision syntax for jj compatibility
36. Add -p, --parents, -c, --children so parent and child counts can be asymmetric
37. Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
38. Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [[22]],[[23]]
39. Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [[24]]
40. Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [[25]],[[26]]
41. Test-tempdir override resolution chain. Both
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
42. **`validate-todo` / `fix-todo`: flag malformed lines.**
    A column-0 line inside `## Todo` / `## Bugs` that is
    neither an entry (`N. `) nor a heading is malformed;
    it's currently tolerated silently. Report it so stray
    lines / typos surface.

# References

[1]: /notes/chores/chores-11.md#docs-refine-cycle-protocol-0560
[2]: /notes/forks-multi-user.md
[3]: /notes/bot-data-formats.md
[4]: /notes/chores/chores-08.md#operations
[5]: /notes/chores/chores-08.md#cycle-structure--multi-step
[6]: /notes/chores/chores-07.md#--scope-enum-refactor-0420
[7]: /notes/chores/chores-09.md#push-dual-bookmark-parameters
[8]: /notes/chores/chores-09.md#push-unimplemented-recheck-flag
[9]: /notes/chores/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[10]: /notes/chores/chores-06.md#generalize---scope-to-all-commands-design
[11]: /notes/chores/chores-06.md#--scope-continuation-0410
[12]: /notes/chores/chores-08.md#init--clone-redesign-0411
[13]: /notes/chores/chores-08.md#user-config-0411-3
[14]: /notes/chores/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[15]: /notes/chores/chores-05.md#capture---message-file-design-for-push-0375
[16]: /notes/chores/chores-06.md#vc-x1-validate-repo-command-design
[17]: /notes/chores/chores-06.md#bm-track-silent-when-clean-design
[18]: /notes/chores/chores-06.md#source-code-design-ref-convention-design
[19]: /notes/chores/chores-05.md#open-questions--tbd
[20]: /notes/chores/chores-03.md#per-lineper-thread-runtime-log-points-future
[21]: /notes/chores/chores-03.md#windows-symlink-support
[22]: /notes/chores/chores-01.md#refactor-and-add-desc-subcommand
[23]: /notes/chores/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[24]: /notes/chores/chores-02.md#0260--shell-completion-via-clap_complete-env
[25]: /notes/chores/chores-02.md#testing-results
[26]: /notes/chores/chores-02.md#shell-completion-discovery
