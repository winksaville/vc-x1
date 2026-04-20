# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress


## Todo

A markdown list of task to do in the near feature

 - Per-line/per-thread runtime log points (future, maybe) [36]
 - Add Windows symlink support via `std::os::windows::fs::symlink_dir` [37]
 - Show bookmarks in `list` output
 - Add "::" revision syntax for jj compatibility
 - Add -p, --parents, -c, --children so parent and child counts can be asymmetric
 - Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
 - Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]
 - Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [27]
 - Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [28],[29]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- Remove deprecated fix-ochid subcommand (0.25.0) [25]
- Add shell completion via clap_complete env (0.26.0) [26]
- Fix validate-desc/fix-desc other-repo resolution with -R flag (0.26.2) [30]
- Add `fn claude-symlink` and `symlink` subcommand (0.27.0) [31]
- Add `init` subcommand for dual-repo project creation (0.28.0) [32]
- Add `clone` command + fix init submodule/ochid bug (0.29.0) [33]
- Universal --verbose, common::run() refactor, chid bold removal (0.30.0) [34]
- Adopt `log` crate with per-module runtime filtering (0.31.0) [35]
- Remove submodule from init/clone (0.31.1) [38]
- Audit `unwrap`/`unwrap_or` usage, add `// OK: …` convention (0.32.0) [39]
- Make `finalize` failures visible — pre-flight, subprocess logging, tty reconnect, status marker (0.33.0) [40]
- Fix deprecated `jj bookmark track <bookmark>@<remote>` syntax for jj 0.40.0 (0.33.1) [41]
- Silence untracked-remote hint in `init` step 9 (0.33.2) [42]
- Compatible dep refresh via `cargo update` (0.33.3) [43]
- Add `--use-template` to `init` and `test-fixture` (0.34.0) [44]
- Bump `jj-lib` to 0.40 + tighten `clap` floor to 4.6 (0.34.1) [45]
- Add `sync` subcommand — fetch + classify + rebase both repos (0.35.0) [46]

# References

[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[25]: /notes/chores-02.md#0250--remove-deprecated-fix-ochid
[26]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[27]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores-02.md#testing-results
[29]: /notes/chores-02.md#shell-completion-discovery
[30]: /notes/chores-02.md#0262--fix-validate-descfix-desc-other-repo-resolution-with--r
[31]: /notes/chores-03.md#add-fn-claude-symlink-0270
[32]: /notes/chores-03.md#add-init-command-0280
[33]: /notes/chores-03.md#add-clone-command-0290
[34]: /notes/chores-03.md#universal---verbose-and-commonrun-refactor-0300
[35]: /notes/chores-03.md#adopt-log-crate-with-per-module-filtering-0310
[36]: /notes/chores-03.md#per-lineper-thread-runtime-log-points-future
[37]: /notes/chores-03.md#windows-symlink-support
[38]: /notes/chores-03.md#remove-submodule-from-initclone-0311
[39]: /notes/chores-04.md#audit-unwrapunwrap_or-usage-0320
[40]: /notes/chores-04.md#make-finalize-failures-visible-0330
[41]: /notes/chores-04.md#fix-deprecated-jj-bookmark-track-syntax-0331
[42]: /notes/chores-04.md#silence-untracked-remote-hint-in-init-step-9-0332
[43]: /notes/chores-04.md#compatible-dep-refresh-0333
[44]: /notes/chores-04.md#add---use-template-to-init--test-fixture-0340
[45]: /notes/chores-04.md#bump-jj-lib-to-040--tighten-clap-floor-0341
[46]: /notes/chores-04.md#add-sync-subcommand-0350
