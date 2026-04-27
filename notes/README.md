# Notes

This directory contains various notes and documentation related to the project.
Each file is organized by topic for easy reference.

By default there are chores-*.md and todo.md. Chores are general notes
about tasks and todo.md contains short term tasks and their status.

In the future we I expect we may want to create a "notes"
database to better manage the information, TBD.

Examples chore file:
```
# Chores-01.md
 
General maintenance tasks and considerations for the project see other files for
more specific topics. A chore in a chores file provides quick information on the
how and why of a particular chore.

## Create a binary that lists jj info 

This binary should list the changeID, commitID, and description title
and using `jj-lib`
```

## Workflow and conventions

Bot-facing workflow, versioning, and code conventions live in
[`../CLAUDE.md`](../CLAUDE.md). Start there for:

- **Versioning during development** — single-step vs multi-step,
  `-N` pre-release suffixes, done-marker discipline.
- **Code Conventions** — doc comments on every file / fn / method,
  `// OK: …` justifications on `unwrap*` calls, ask-on-ambiguity,
  stuck detection.
- **Commit-Push-Finalize Flow** — two-checkpoint per-step
  discipline with hard stop after finalize.

## Todo format

Todo.md contains two main sections "Todo" and "Done" each item is a
short explanations of a tasks and links to more details using 1 or more
references.

Todo items use lazy numbering — every entry begins with `1. ` and the
markdown renderer auto-numbers them. Reorder or insert items without
renumbering, and reference an entry by its displayed number ("let's
work on #3"). The Done section keeps `-` bullets — items aren't
referenced by number once completed.

Multiple references must be separated: `[2],[3]` not `[2,3]` or `[2][3]`.
In markdown, `[2,3]` is a single ref key (won't resolve) and `[2][3]`
is parsed as display text `2` with ref key `3` (so `[2]` won't resolve).

Examples:

# Todo
- Add new feature X [details](features.md#feature-x)
- Fix bug Y [1]

# Done
- Fixed issue Z [2],[3]

[1]: bugs.md#bug-y
[2]: issues.md#issue-z
[3]: fixes.md#fix-z

## Reference numbering

Every note file (`todo.md`, `chores-NN.md`, `done.md`) keeps a
file-local `# References` section at the bottom. Reference numbers
are scoped to that file — `[1]` in `chores-07.md` and `[1]` in
`chores-01.md` are independent slots that may point at completely
different URLs. New chores files start their numbering at `[1]`.

## Retiring Done entries

`todo.md`'s `## Done` section is a rolling buffer of recently shipped
work, not a permanent log. Move entries into `done.md` at two natural
beats:

- **Closing a ladder** — when the final `X.Y.Z` (no suffix) commit
  ships, decide which prior entries are no longer needed for nearby
  context and migrate them.
- **Opening a new ladder** — at `X.Y.Z-0`, do the same sweep before
  bumping the version.

Migration mechanics:

- Move the bullet itself from `todo.md > ## Done` to
  `done.md` (preserving the original ref number).
- Copy any references the moved entries cite into
  `done.md`'s `# References` section (those refs are file-local,
  so coexisting with `todo.md`'s namespace is fine).
- Prune any references in `todo.md > # References` no longer cited
  by anything in `## In Progress` / `## Todo` / `## Done`. This
  frees the numbers for future reuse.
