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

## Versioning during development

This is using jujustiu, jj + git and we'll see how it goes. Below is my
git workflow, jj will be different but we'll have to discover that as
we go.

Every plan must start with a version bump. Choose the approach based on scope:

- **Single-step** (recommended for mechanical/focused changes): bump directly to
  `X.Y.Z`, implement in one commit. Simpler history.
- **Multi-step** (for exploratory/large changes): bump to `X.Y.Z-devN`, implement
  across multiple commits, final commit removes `-devN`.

The plan should recommend one approach and get user approval before starting.

For multi-step:
1. Bump version to `X.Y.Z-devN` with a plan and commit as a chore marker
2. Implement in one or more `-devN` commits (bump N as needed)
3. Final commit removes `-devN`, updates todo/chores — this is the "done" marker

The final release commit (without `-devN`) signals completion rather than amending
prior commits. This keeps the git history readable and makes it easy to see which
commits were exploratory vs final.

## Code Conventions

### `// OK: …` comments on `unwrap*` calls

Non-test code that calls `.unwrap()`, `.unwrap_or(…)`, `.unwrap_or_default()`,
or `.unwrap_or_else(…)` must have a trailing `// OK: …` comment that justifies
why the call is acceptable.

- `// OK: <specific reason>` — document the real precondition, invariant, or
  domain reason. Preferred whenever the reason isn't self-evident.
- `// OK: obvious` — the default is self-evident from context (e.g.
  `desc.lines().next().unwrap_or("")` — empty desc → empty title).

Bare `// OK` is not used (reads like a truncated comment). Abbreviations
(e.g. `SE`) are not used because they require a decoder ring for readers
seeing the code out of context.

For provably-unreachable `.unwrap()` calls, also prefix with
`#[allow(clippy::unwrap_used)]` so the site stays silent if we enable the
project-wide `clippy::unwrap_used` lint later.

```rust
// Specific reason
let max = stderr_level.unwrap_or(LevelFilter::Info); // OK: default verbosity when -v/-vv absent

// Self-evident
let first_line = desc.lines().next().unwrap_or(""); // OK: obvious

// Proven precondition
match matches.len() {
    1 => {
        #[allow(clippy::unwrap_used)]
        // OK: `1 =>` arm guarantees matches.len() == 1
        Ok(TitleMatch::One(matches.into_iter().next().unwrap()))
    }
    // ...
}
```

Tests (`#[cfg(test)]`) are exempt — panicking on setup failure is the correct
test behavior.

## Todo format

Todo.md contains two main sections "Todo" and "Done" each item is a
short explanations of a tasks and links to more details using 1 or more
references.

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
