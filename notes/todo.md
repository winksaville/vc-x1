# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

 - Add "::" revision syntax for jj compatibility
 - Add -p, --parents, -c, --children so parent and child counts can be asymmetric
 - Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
 - Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]
 - Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [27]
 - Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [28],[29]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- Bold primary revision in chid, list, desc output (0.15.0) [14]
- Indent desc body lines with --indent/-i, default 3 spaces (0.16.0) [15]
- Finalize: replace --foreground with --detach, document manual recovery (0.17.0) [16]
- jj commit organization and traversal mechanisms (0.17.0) [17]
- Add show subcommand with header, bookmarks, and diff summary (0.18.0) [18]
- Flesh out show header to match gitk, add .. notation and file limiting (0.18.1) [18]
- Unify `..` notation and CLI across all subcommands (0.19.0) [18]
- Reorganize notes: move older done items to done.md (0.19.1)
- Multi-repo `-R` support with `-l`/`--label` and `-L`/`--no-label` for chid, desc, list, show (0.20.0) [18]
- Disperse CLI parsing tests from main.rs into per-subcommand files (0.20.1) [19]
- Show ochid in list output, clean up CLI help defaults (0.21.0) [19]
- Deduplicate common CLI flags with `#[command(flatten)]` (0.21.1) [19]
- Add fix-ochid subcommand with validation and --fallback (0.22.0) [19]
- Fix fix-ochid prefix bug: read workspace.path from .vc-config.toml (0.22.1) [19]
- Fix fix-ochid short ID extension, add notes to pre-commit checklist (0.22.2) [19]
- Add --add-missing to fix-ochid for inferring ochid from title+timestamp (0.23.0) [19]
- Add --max-fixes to fix-ochid to limit commits actually changed (0.24.0) [19]
- Add validate-desc subcommand, extract desc_helpers (0.25.0-dev1) [21]
- Add fix-desc subcommand using shared helpers (0.25.0-dev2) [22]
- Add lost/none special ochid status, improved error messages (0.25.0-dev2) [22],[23]
- Read other-repo from .vc-config.toml, make positional arg a --other-repo flag (0.25.0-dev3) [24]
- Run fix-desc on both repos to fix ochid trailers with --fallback for lost IDs (0.25.0) [20]
- Remove deprecated fix-ochid subcommand (0.25.0) [25]
- Add shell completion via clap_complete env (0.26.0) [26]
- Fix validate-desc/fix-desc other-repo resolution with -R flag (0.26.1) [30]



# References

[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[14]: /notes/chores-01.md#bold-primary-revision-in-output-0150
[15]: /notes/chores-01.md#indent-desc-body-lines-0160
[16]: /notes/chores-01.md#finalize-detach-and-manual-recovery-0170
[17]: /notes/chores-02.md#jj-commit-organization-and-traversal-mechanisms-0170
[18]: /notes/chores-02.md#0180--initial-show-subcommand
[19]: /notes/chores-02.md#0200--multi-repo-support
[20]: /notes/chores-02.md#0250--refactor-into-validate-desc--fix-desc
[21]: /notes/chores-02.md#0250-dev1--add-validate-desc-extract-desc_helpers
[22]: /notes/chores-02.md#0250-dev2--add-fix-desc-subcommand
[23]: /notes/chores-02.md#special-ochid-values-lost-and-none
[24]: /notes/chores-02.md#0250-dev3--read-other-repo-from-config
[25]: /notes/chores-02.md#0250--remove-deprecated-fix-ochid
[26]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[27]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores-02.md#testing-results
[29]: /notes/chores-02.md#shell-completion-discovery
[30]: /notes/chores-02.md#0261--fix-validate-descfix-desc-other-repo-resolution-with--r
