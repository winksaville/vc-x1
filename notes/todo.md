# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

 - Determine the organization of the commits in jj and determine how we can iterate over them.
   - I'd like to beable to see "all" commits organized appropriately, `gitk --all` sees many
     more commits than any jj command I've found
 - Add integration tests for `finalize` subcommand using temp jj repos (tempfile crate)
 - Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]

## Done

See older [done.md](done.md).

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed.

- Add --version and -V flags using std lib (clap not a dependency)
- Use git trailers for inter/intra repo info: ochid trailer, changeID path syntax, .vc-config.toml[[2]]
- Document git trailer convention (ochid:) and .vc-config.toml for workspace identity
- Document why jj log shows fewer commits than gitk (refs/jj/keep, obslog, ::@ revset)
- Create a binary that lists jj info[[1]]
- Convert CLI to subcommand structure with `list` command
- Add finalize subcommand arg parsing (0.6.0-dev1) [3]
- Add finalize daemonize with debug logging (0.6.0-dev2) [3]
- Implement finalize exec with squash/push logic (0.6.0-dev3) [3]
- Add --ignore-immutable and unique log paths (0.6.0-dev4) [3]
- Finalize subcommand complete (0.6.0) [3]
- Plan refactor and desc subcommand (0.7.0-dev0) [4],[5]
- Extract common.rs and refactor list (0.7.0-dev1) [4],[5]
- Refactor finalize into src/finalize.rs (0.7.0-dev2) [4],[5]
- Implement desc subcommand (0.7.0-dev3) [4]
- Refactor and desc subcommand complete (0.7.0) [4]
- Migrate CLI parsing to clap derive (0.8.0) [6]
- Move subcommand args into per-module structs (0.9.0) [7]
- Add --revision/-r, --repo/-R, --limit/-l to list (0.10.0-dev1) [8]
- Add --revision/-r, --repo/-R, --limit/-l to desc (0.10.0-dev2) [8]
- Revision and repo options complete (0.10.0) [8]
- Show changeID and commitID in desc output (0.11.0) [9]
- Add chid subcommand (0.12.0) [10]
- Add --limit to chid subcommand (0.13.0) [11]
- Add positional `..` revision notation (0.14.0) [12]
- Add required `--bookmark` to finalize (0.14.0) [13]


# References

A set of markdown references for tasks and details including vc changeID URLs.
See [ChangeID footer syntax](chores-01.md#changeid-footer-syntax).

[1]: /notes/chores-01.md#create-a-binary-that-lists-jj-info
[2]: /notes/chores-01.md#git-trailer-convention
[3]: /notes/chores-01.md#finalize-subcommand-for-session-repo-coherence
[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[6]: /notes/chores-01.md#migrate-cli-parsing-to-clap-080
[7]: /notes/chores-01.md#move-subcommand-args-into-modules-090
[8]: /notes/chores-01.md#add-revision-and-repo-options-to-list-and-desc-0100
[9]: /notes/chores-01.md#show-changeid-and-commitid-in-desc-output-0110
[10]: /notes/chores-01.md#add-chid-subcommand-0120
[11]: /notes/chores-01.md#add---limit-to-chid-subcommand-0130
[12]: /notes/chores-01.md#add-positional--revision-notation-0140
[13]: /notes/chores-01.md#add-required---bookmark-to-finalize-0140
