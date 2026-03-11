# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

A markdown list of Tasks currently in progress


## Todo

A markdown list of task to do in the near feature

 - Determine the organization of the commits in jj and how we can iterate over them.
   - I'd like to beable to see "all" commits organized appropriately, `gitk --all` sees many more commits than any jj command I've found

## Done

See older [done.md](done.md).

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed.

- Add --version and -V flags using std lib (clap not a dependency)
- Use git trailers for inter/intra repo info: ochid trailer, changeID path syntax, .vc-config.toml[[2]]
- Document git trailer convention (ochid:) and .vc-config.toml for workspace identity
- Document why jj log shows fewer commits than gitk (refs/jj/keep, obslog, ::@ revset)
- Create a binary that lists jj info[[1]]


# References

A set of markdown references for tasks and details including vc changeID URLs.
See [ChangeID footer syntax](chores-01.md#changeid-footer-syntax).

[1]: /notes/chores-01.md#create-a-binary-that-lists-jj-info
[2]: /notes/chores-01.md#git-trailer-convention
