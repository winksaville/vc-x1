# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

(none)

## Todo

A markdown list of task to do in the near feature

 - Add integration tests for `finalize` subcommand using temp jj repos (tempfile crate)
 - Determine the organization of the commits in jj and determine how we can iterate over them.
   - I'd like to beable to see "all" commits organized appropriately, `gitk --all` sees many
     more commits than any jj command I've found

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


# References

A set of markdown references for tasks and details including vc changeID URLs.
See [ChangeID footer syntax](chores-01.md#changeid-footer-syntax).

[1]: /notes/chores-01.md#create-a-binary-that-lists-jj-info
[2]: /notes/chores-01.md#git-trailer-convention
[3]: /notes/chores-01.md#finalize-subcommand-for-session-repo-coherence
