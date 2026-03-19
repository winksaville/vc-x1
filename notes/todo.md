# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

 - Add "::" revision syntax for jj compatibility
 - Add -p, --parents, -c, --children so parent and child counts can be asymmetric
 - Add integration tests for `finalize` subcommand using temp jj repos (tempfile crate)
 - Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]

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


# References

[14]: /notes/chores-01.md#bold-primary-revision-in-output-0150
[15]: /notes/chores-01.md#indent-desc-body-lines-0160
[16]: /notes/chores-01.md#finalize-detach-and-manual-recovery-0170
[17]: /notes/chores-02.md#jj-commit-organization-and-traversal-mechanisms-0170
[18]: /notes/chores-02.md#show-subcommand-0180
