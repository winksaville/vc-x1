# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

 - Add `clone command` [33]
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
