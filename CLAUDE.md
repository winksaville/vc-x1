# CLAUDE.md - Bot Instructions

## Project Structure

This project uses **two separate jj-git repos**:

1. **App repo** (`/` — project root): Contains the application source code.
2. **Bot session repo** (`/.claude/`): Contains Claude Code session data.

Both repos are managed with `jj` (Jujutsu), which coexists with git.

## Repo Paths (relative from project root)

- App repo: `.` (project root)
- Bot session repo: `.claude`
  (symlink from `~/.claude/projects/<path-to-project-root>/.claude`)

## Working Directory

Prefer staying in the project root. Use `-R` flags or absolute paths
to target other directories rather than `cd`. If `cd` seems necessary,
discuss with the user first — losing track of cwd causes subtle
command failures downstream.

When invoking shell commands, prefer the shortest unambiguous path —
usually relative to cwd (`ls notes/`, not
`ls /home/wink/data/prgs/rust/vc-x1/notes/`). Long absolute paths
clutter the transcript without adding information. Out-of-workspace
paths (`/tmp/...`, `~/.config/...`) stay absolute. Read/Edit/Write
tool args also stay absolute — that's a tool-boundary constraint,
not a stylistic choice.

## File reads — read the slice you need

Long notes files are appended to over time. Read only the
slice your task needs; grep or read further on demand.

- **`notes/todo.md`** (the routine acquaint read) — first
  ~60 lines covers intro + `## In Progress` + `## Priorities`.
  `Read` with `offset=0, limit=60`. Read further only when
  picking up a `## Todo` entry, chasing a `[N]` ref, or
  auditing the whole list.
- **`notes/todo-backlog.md`** — the long-tail backlog
  (non-prioritized `## Todo` entries). Read only when
  picking up a backlog item; grep to locate it first.
- **`notes/bugs.md`** — the bug list. Small; read whole
  when triaging a bug or chasing the `## Bugs` pointer in
  todo.md.
- **`notes/done.md`** + **`notes/chores/chores-NN.md`** —
  historical / append-mostly. Scan headings first
  (`grep '^## ' notes/chores/chores-NN.md`), then read only
  the section you need.

**Why:** before the 0.58.0 split, `notes/todo.md` ran ~370
lines and grew every cycle. The split moved the backlog and
bugs into sibling files so the routine read stays small; the
same "slice you need" rule applies to historical files.

## Memory

Do not use the bot's per-project memory directory
(`~/.claude/projects/<path>/memory/`). In a dual-repo setup with
CLAUDE.md it provides no capability CLAUDE.md doesn't already cover,
and it loses on discoverability:

- **CLAUDE.md** — at the repo root, a well-known location for bot
  instructions, committed, reviewable, visible to every collaborator
  (human or bot).
- **Memory directory** — hidden under the user's home, tied to one
  machine, invisible to anyone but the bot, never diffed or reviewed.

Easy for everyone to find beats convenient for the bot alone. Put
durable context in CLAUDE.md (or committed `notes/`) instead.

## Speculation marker

Durable text the bot writes — CLAUDE.md, `notes/`, commit bodies,
chores sections — should stick to observations and direct descriptions
of the code or data. If a mechanism, hypothesis, or causal claim
enters the text, prefix it with "The bot thinks ..." so a reader can
tell the measured from the inferred.

**Why:** unmarked speculation reads like evidence, and a future reader
(or the bot on a later session) can pick it up as a known fact when
it's not. Measured / inferred is a distinction worth keeping visible
in the written record.

**How to apply:** observations and factual descriptions need no
marker. Prefix with "The bot thinks ..." (or a close variant like
"The bot's guess is ...") when the claim is a mechanism ("X wins
because Y caches better"), a cause ("the drift was due to thermal
state"), a prediction ("this should scale linearly"), or any
reasoning not directly supported by the data on hand.

## jj Basics

- `jj st -R .` / `jj st -R .claude` — show working copy status
- `jj log -R .` / `jj log -R .claude` — show commit log
- `jj commit -m "title" -m "body" -R <repo>` — finalize working copy into a commit
- `jj describe -m "title" -m "body" -R <repo>` — set description without committing
- `jj git push --bookmark <name> -R <repo>` — push a bookmark (no
  `--allow-new` flag; jj pushes new bookmarks without special flags)
- In jj, the working copy (@) is always a mutable commit being edited.
  `jj commit` finalizes it and creates a new empty working copy on top.
- The `.claude` repo always has uncommitted changes during an active
  session because session data updates continuously.
- **Post-amend `jj new`** — `jj edit <rev>`,
  `jj rebase -r <rev> -d ...`, and `jj squash --into <rev>`
  all leave `@` on the rewritten commit rather than on an
  empty change above. The most common case is the close-out
  push for a [Merge (non-FF)](notes/cycle-protocol.md#pushing) shape (`jj rebase
  -r <tip> -d <prev> -d <sub-tip>` leaves `@` on the new
  merge). In a colocated jj+git repo, git then detaches
  HEAD at the commit's first parent, so `git status`,
  gitk, and IDE diff views read the commit's content as
  uncommitted relative to HEAD. Follow up with `jj new` to
  create an empty `@` above; HEAD shifts to the commit and
  the working tree reads clean.

## Prose form

Long-lived prose on this project follows one basic shape: a short intro
that explains the *why* or the high-level *what*, then a `-` bullet
list for the details. Wrap lines at ≤72 cols (bullet continuations
indent two spaces). Avoid wall-of-prose paragraphs — they hide the
structure that bullets make scannable.

Surfaces that use this shape:

- Module / function / struct / field doc comments in `.rs` files —
  see [Doc comments](#doc-comments-on-every-file-function-and-method).
- Commit message bodies (both app-repo and session-repo). The
  ≤50-col title is the commit-specific add-on; see
  [Per-commit flow](notes/cycle-protocol.md#per-commit-flow).
- Chore descriptions in `notes/chores/chores-NN.md` — see
  [Chores section content](#chores-section-content--no-edit-list-git-is-the-record).
- Todo and Done entries in `notes/todo.md` when an entry needs more
  than one line of detail. Pure one-liners are still fine.

Bullet *content* differs by surface:

- **Commit bodies** — bullets are file-by-file: one bullet per file
  changed, file plus a one-line gist (e.g.
  `README.md: new Overview intro`). Source of truth for the
  mechanical edit list.
- **Chores / todo / done** — bullets are conceptual (design points,
  structural notes, the "what landed and why" at a notch above
  file-list granularity). Never a copy of the commit's edit list —
  see [Chores section content](#chores-section-content--no-edit-list-git-is-the-record).
- **Doc comments** — bullets are whatever structure fits (fields,
  cases, invariants).

**Problem + plan shape.** `## In Progress` cycle blocks,
chores section intros, and `## Todo` entries use a sharper
form of the same shape:

- **Problem statement** (the why) — one or two sentences;
  don't pad with intent, don't restate the plan.
- **Plan bullets** (the what/when) — formality differs by
  surface:
  - In Progress / chores: numbered ladder (`-N` suffixes,
    `(current)` / `(done)` markers) — a committed sequence.
  - Todo entries: rough informal bullets, no numbering;
    formalized only when the entry is picked up into a
    cycle.

**Semicolons inside bullets.** A bullet that joins
multiple clauses with semicolons (`A; B; C`) is a list
hiding inside running prose — break the clauses into
sub-bullets so the structure shows. Semicolons in running
prose (intro paragraphs, sentence-joins) are fine. Not
absolute: very short clauses or tight pairs can stay
joined inside a bullet when breaking would be more noise
than signal.

## Notes references

Reference *citations* are double-bracketed so the brackets render
— `[[N]]`, or `[[2]],[[3]]` for several (comma-separated, not
`[2,3]` or `[[2]][[3]]`). The `[N]:` definitions in a file's
`# References` section and inline `[text](url)` / `[text](#anchor)`
links stay single-bracketed. See
[Todo format](notes/README.md#todo-format) for details.

## Markdown anchor links

GitHub anchor algorithm: lowercase, strip non-alphanumeric
characters in place, map remaining spaces to hyphens 1-for-1. Do
**not** collapse adjacent whitespace — so `a + b` → `a--b` (spaces
on both sides of `+`), but `a: b` → `a-b` (only trailing space on
`:`). General markdown reference:
[markdownguide.org](https://www.markdownguide.org). GitHub
publishes no official spec for auto-generated anchors; the
de-facto reference implementation is
[github-slugger](https://github.com/Flet/github-slugger).

## Chores conventions

### Headings and entries that record a commit

A `chores-NN.md` `##` section header that records a specific
commit, the matching `todo.md > ## Done` entry, and any `[N]`
reference to that section all use **exactly that commit's
title** — `<type>: <desc> (<version>)`, the same string the
commit gets (see [Per-commit flow](notes/cycle-protocol.md#per-commit-flow)).
E.g. the chores header `## refactor: port push to Context
(0.48.0-6)` and the Done line `- refactor: port push to Context
(0.48.0-6) [[3]]`. The `## Done` entry uses the close-out
commit's title.

This does **not** apply to organizational headings (`## Todo`,
`## In Progress`, `# References`) or to design `###` subsections
inside a chores section — those are named for whatever fits.
Among the commit-recording ones, exact match is the strong
default (nothing absolute): a near-miss just makes it harder to
line a record up with its commit.

A commit-recording header is provisional while the work is in
progress; the *last* edit before `vc-x1 push` syncs it — and the
`## Done` entry / `[N]` anchor for that commit — to the final
commit title. See [Markdown anchor links](#markdown-anchor-links)
for the slug algorithm; the pre-commit checklist catches a
dangling `#anchor`, and a future `vc-x1 validate-repo` should too
(and should verify the recorded title matches the commit).

Existing sections and `## Done` entries (most of chores-01..09;
the pre-`0.48.1` `## Done` lines) predate this and keep their
free-form text; the convention applies going forward — `0.48.2`
converts the `0.48.1` section + its Done entry as the worked
example.

### Chores section content — no edit list; git is the record

A chores section is: a `Commits:` line (first line under the
header — see below), then [Prose form](#prose-form) (intro +
bullets) for what landed and why, and any `###` design
subsections. Bullets here are **conceptual** — design points,
structural notes — never a per-file edit list. That lives in the
commit message body, which is the source of truth for "what
changed mechanically" (immutable, `git show`-able, naturally
scoped to the commit). The chores section is the source of truth
for the design thinking; the two cross-link, neither restates the
other.

When the intro starts wanting to explain a mechanism,
hypothesis, or wrinkle, don't inflate it — promote that to its
own `###` subsection inside the same `chores-NN.md`. If the
wrinkle is a live design concern (something that *should*
change, not just be recorded), also add a `notes/todo.md` item
with a `[N]` ref pointing at that subsection (todo→chores is the
normal ref direction).

**Why:** a chores edit list and the commit body were specified
to be the same content in two places — and detail written twice
drifts. Git owns the mechanical record; chores owns the
narrative; `Commits:` links them.

### Chores commit references

The first line under a chores section header is a `Commits:`
line citing the git commit(s) that section records:

```
## refactor: port push to Context (0.48.0-6)

Commits: [[3]]

<intro paragraph...>
```

`Commits:` uses the file-local `[N]` reference machinery (see
[notes/README.md](notes/README.md#reference-numbering)),
**double-bracketed** so the brackets render — `Commits: [[3]]`,
or `Commits: [[3]],[[5]]` for several. (`[[3]]` shows as a
literal `[`, the `[3]` link, then a literal `]`; the inner
`[3]` resolves against its `[3]:` definition — CommonMark /
GitHub / VS Code all do this.) The `# References` definition
puts the **commit URL** as the destination, with the **full
40-hex SHA** in the title slot:

```
[3]: https://github.com/winksaville/vc-x1/commit/<12-hex> "<40-hex>"
```

- The 12-hex short SHA in the URL keeps it short; GitHub /
  GitLab resolve a unique prefix to the canonical commit page
  (GitLab's path has a `/-/` before `commit/`).
- The full SHA in the title is host-agnostic and unambiguous —
  it survives a repo host change, `git show <40-hex>` works in
  any clone, and external tooling scraping the notes (a
  database, say) gets the canonical identifier.

**Timing.** The commit doesn't exist when its chores section is
written, so the `Commits:` line is **backfilled when the next
change to that repo is started** — the cycle-start step grabs
the just-pushed commit's URL + SHA and fills it in. The single
newest section is briefly `Commits:`-less; that's fine — the
commit itself is the record, and `git log --grep "(X.Y.Z)"`
finds it.

## Cycle Protocol

Every change runs as a **cycle**: Preparation (`X.Y.Z-0`) →
Work commits (`X.Y.Z-1`, `X.Y.Z-2`, …) → Close-out (bare
`X.Y.Z`). The full protocol — numbering, per-commit flow,
reviewing changes, close-out, pushing, ochid trailers,
sub-cycles — lives in
[`notes/cycle-protocol.md`](notes/cycle-protocol.md). Read
it before any commit work.


## Code Conventions

### Doc comments on every file, function, and method

Every `.rs` file must begin with a `//!` module docstring. Every
function and method must have a `///` doc comment. Keep them brief —
one sentence of purpose is often enough; the discipline is that the
comment exists, not that it be long. Doc comments follow the
[Prose form](#prose-form) shape (intro + bullets).

This is a deliberate override of the generic "write no comments"
default that applies to inline `//` comments. Doc comments on the
module / item surface are expected; inline explanatory comments
inside function bodies remain discouraged unless they capture a
non-obvious WHY.

**Clap-derive args:** doc comments on `#[arg(...)]` fields drive
`--help` output. Clap reflows by default and collapses bullets
into running prose. Add `#[arg(verbatim_doc_comment, ...)]` on
any field whose doc comment uses bullets so each `- …` lands on
its own line in the rendered help.

### `// OK: …` comments on `unwrap*` calls (Rust)

Non-test code that calls `.unwrap()`, `.unwrap_or(…)`,
`.unwrap_or_default()`, or `.unwrap_or_else(…)` must have a trailing
`// OK: …` comment that justifies why the call is acceptable.

- `// OK: <specific reason>` — document the real precondition,
  invariant, or domain reason. Preferred whenever the reason isn't
  self-evident.
- `// OK: obvious` — the default is self-evident from context (e.g.
  `desc.lines().next().unwrap_or("")` — empty desc → empty title).

Bare `// OK` is not used (reads like a truncated comment).
Abbreviations (e.g. `SE`) are not used because they require a decoder
ring for readers seeing the code out of context.

For provably-unreachable `.unwrap()` calls, also prefix with
`#[allow(clippy::unwrap_used)]` so the site stays silent if we enable
the project-wide `clippy::unwrap_used` lint later.

```rust
let max = stderr_level.unwrap_or(LevelFilter::Info); // OK: default verbosity when -v/-vv absent
let first_line = desc.lines().next().unwrap_or("");  // OK: obvious

match matches.len() {
    1 => {
        #[allow(clippy::unwrap_used)]
        // OK: `1 =>` arm guarantees matches.len() == 1
        Ok(TitleMatch::One(matches.into_iter().next().unwrap()))
    }
    // ...
}
```

Tests (`#[cfg(test)]`) are exempt — panicking on setup failure is the
correct test behavior.

### Ask for clarification on ambiguous input

When user input is ambiguous or missing necessary detail, stop and
ask a specific question. Do not proceed on a guess and hope the
result lands right — a clarifying question costs a few seconds;
redoing misaligned work costs much more.

### Recognize when stuck

If a simple task has eaten 5+ minutes of thinking or back-and-forth
without progress, stop. Summarize what's blocking — unclear
requirements, unfamiliar API surface, conflicting signals — and ask.
Continued flailing produces worse outcomes than a direct "I'm stuck
on X."
