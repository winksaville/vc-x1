# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

Multi-step `0.38.0` cycle (Bookmark tracking verification, [66]):

1. 0.38.0 release: notes/doc cleanup + done marker. [66]

## Todo

A markdown list of tasks to do in the near future, ordered
highest-priority first. Keep entries brief — 1-3 lines.
Detailed motivation, safety requirements, and ordering belong
in `notes/chores-NN.md` design subsections; link via `[N]` ref.

Items use lazy numbering — every entry begins with `1. `; the
markdown renderer auto-numbers them, so reorder/insert without
renumbering. Reference by displayed number ("let's work on #3").

1. Non-tracking-remote bookmark detection across every
   repo-modifying command. Caused real finalize failure
   2026-04-22 (push-app succeeded, finalize couldn't push). [63]
1. vc-x1 push: state-sanity preflight on resume. Refuse with a
   loud "state is stale" error when saved state doesn't match
   reality. Surfaced by 0.37.1 dogfood. [61]
1. vc-x1 push: stage-prereq verification + honest completion.
   "Completed all stages" must reflect what actually ran. [61]
1. vc-x1 commands: support single and dual repo. Generalize via
   `--scope=app|other|both` across all commands; foundational
   for new commands and for retrofitting sync / push / finalize
   under one vocabulary. [60]
1. vc-x1 push: `--scope=app|claude|both` flag. Applies the
   generalized convention; warn on scope/WC mismatch. [57],[60]
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
1. Add `status` (alias `st`) subcommand: `jj st` across both
   repos in one shot. Uses `--scope` from day one. Natural
   home for the working-copy signal called out in [54].
1. sync: surface working-copy state in the up-to-date summary
   (per-repo pending-files count or compact stat). Wording-only
   fix shipped in 0.37.1; this is the design+impl. [54]
1. bm-track silent-when-clean refinement. Print on entry/exit
   only when state isn't fully tracked or when exit state
   differs from entry. [62]
1. "Oh shit" revert — post-success undo via `.vc-x1-ops/`
   anchor dir. Idea-stage; every repo-mutating command drops a
   pre-op snapshot, `vc-x1 undo` restores both repos. [57]
1. vc-x1 test-fixture should refuse `--path` values that resolve
   inside the current workspace root (error or warn). Dogfood
   surfaced this: `--path ./tf-1` inside the repo let jj
   snapshot the fixture's bare-git remotes — 56-file noise blob.
1. Restructure templates: replace separate `vc-template-x1` +
   `vc-template-x1.claude` repos with a single `vc-template-x1`
   that has `.claude/` as a subdir (covers `LICENSE-*` etc. for
   both sides in one place). Updates to `vc-x1 init` / `clone`
   needed for the new layout.
1. Richer bookmark enumeration: per-bookmark remote presence + tracking status [52]
1. Per-line/per-thread runtime log points (future, maybe) [36]
1. Add Windows symlink support via `std::os::windows::fs::symlink_dir` [37]
1. Add "::" revision syntax for jj compatibility
1. Add -p, --parents, -c, --children so parent and child counts can be asymmetric
1. Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
1. Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]
1. Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [27]
1. Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [28],[29]

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
