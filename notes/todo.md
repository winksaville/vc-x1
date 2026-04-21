# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress


## Todo

A markdown list of task to do in the near feature

 - Add `push` subcommand — collapse commit+push+finalize ceremony [48]
 - Richer bookmark enumeration: per-bookmark remote presence + tracking status [52]
 - Per-line/per-thread runtime log points (future, maybe) [36]
 - Add Windows symlink support via `std::os::windows::fs::symlink_dir` [37]
 - Add "::" revision syntax for jj compatibility
 - Add -p, --parents, -c, --children so parent and child counts can be asymmetric
 - Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
 - Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]
 - Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [27]
 - Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [28],[29]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- CLAUDE.md refresh + memory migration (0.36.1) [49]
- Lift sync's inline test harness into shared `test_helpers` module (0.36.2) [51]
- Sync improvements: -R flag + quieter dry-run + sync-before-work discipline (0.36.3) [50]
- push subcommand scaffolding: flag surface, Stage enum, stub (0.37.0-0) [48]
- push state machine: state file, --status/--restart/--from, stage stubs (0.37.0-1) [48]
- push real stage bodies + jj-op snapshot rollback (0.37.0-2) [48]

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
