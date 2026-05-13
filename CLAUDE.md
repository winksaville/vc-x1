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

## Committing

Use `-R` (`--repository`) at the end to target the correct repo. Use
relative paths to reduce noise. Putting `-R` last keeps the verb/action
visible at the start of the command.

### App repo
```
jj commit -m \
"title" \
-m "body

ochid: /.claude/<changeID>" \
-R .
```

### Bot session repo
```
jj commit -m \
"title" \
-m "body

ochid: /<changeID>" \
-R .claude
```

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

## Commit Message Style

Use [Conventional Commits](https://www.conventionalcommits.org/) with
a version suffix:

```
<type>: <short description> (<version>)
```

- **Title**: ≤50 chars (the "50" of the git 50/72 rule), a short
  summary of *what* changed. The version suffix counts toward the
  50 — the descriptive part gets whatever's left after `<type>: `
  and ` (<version>)`, so favor terse phrasings (`port X to Context`
  over `X → Context + XParams`) when the names run long. Common
  types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`.
- **Body line width**: wrap every body line at ≤72 cols (the "72"
  of the 50/72 rule); bullet continuations indent two spaces.
- **App-repo body**: short intro paragraph (1–3 sentences), then a
  terse bullet list — one bullet per file changed, the file plus a
  one-line gist (e.g. `README.md: new Overview intro`). This list
  **is the source of truth** for the cycle's mechanical change
  record; the chores section carries the narrative + design, not a
  copy of it. Keep bullets terse — a scannable index, not prose;
  promote any "why" to a chores `###` subsection.
- **Session-repo body**: terse intro + a few session-activity
  bullets. Doesn't need to mirror chores since it describes
  in-session work, not code changes.
- Examples:
  - `feat: add fix-ochid subcommand (0.22.0)`
  - `fix: fix-ochid prefix bug (0.22.1)`
  - `refactor: deduplicate common CLI flags (0.21.1)`

## Pre-commit Requirements

### User approval

Never execute commit, squash, push, or finalize commands without the
user's explicit approval. Present changes for review first; only run
them after the user confirms. This applies to late changes too —
pause for review before squashing into an existing commit.

### Review before proposing the commit block

After finishing a unit of work, **summarize what changed and stop
there**. Do not pre-emptively lay out the Checkpoint-1 commit
commands. Wait for the user to signal review is complete before
proposing the commit block. Changes during review are the norm,
not the exception; proposing commit text too early creates noise
and signals that I consider the work done when it usually isn't.

This applies per-step in a multi-step flow too — each step gets a
review pause before its commit block appears.

Signals that review is complete include explicit approval ("let's
commit", "looks good, commit it") **and any directive to start the
next step** ("do step 4", "next", "go N+1"). In that case the
previous step must be committed first — always commit the current
step before starting the next; don't ask.

### Per-file review checkpoints

The "review before commit block" rule applies to in-progress work
too, at finer grain. After each file edit, STOP, summarize what
changed, and wait for explicit go-ahead before touching the next
file or running follow-on commands (cargo test/install, dogfood
invocations, `vc-x1 …`, etc.).

**Exceptions** — both ride along with the change they accompany,
no separate review pause:

- **Refactors where code is moving across files** — e.g.
  lifting helpers into a new module and updating the original
  file's imports in the same change. Reviewing each half in
  isolation isn't useful.
- **`Cargo.toml` version bumps** that go with a specific code
  change.

**Why:** the user reviews diffs as work lands. Chaining multiple
file edits without pausing forces them to untangle cumulative
state instead of inspecting each step in isolation.

**How to apply:**

- "continue", "go", "do it", "lg" approves *the next unit
  only*, not all remaining units of a multi-file plan. A green
  light to proceed with a *direction* is not a green light to
  skip per-step checkpoints.
- After landing a unit: brief summary (2–5 lines) and stop. No
  cargo build / test / install / dogfood runs unless the user
  explicitly asks.
- In addition to (not replacement for) the
  Review-before-commit-block rule above.
- **Sub-step exception:** within a multi-step cycle's
  sub-step (or sub-sub-step) ladder, the per-step checkpoints
  defer to the commit-first review model — see
  [Sub-step Workflow > Commit-first review model](#commit-first-review-model).
  Sub-step commits land as the bot finishes them; user reviews
  the committed revision in their editor. The cycle-level
  push at close-out keeps the two-gate ceremony.

### Notes references

Reference *citations* are double-bracketed so the brackets render
— `[[N]]`, or `[[2]],[[3]]` for several (comma-separated, not
`[2,3]` or `[[2]][[3]]`). The `[N]:` definitions in a file's
`# References` section and inline `[text](url)` / `[text](#anchor)`
links stay single-bracketed. See
[Todo format](notes/README.md#todo-format) for details.

### Markdown anchor links

GitHub anchor algorithm: lowercase, strip non-alphanumeric
characters in place, map remaining spaces to hyphens 1-for-1. Do
**not** collapse adjacent whitespace — so `a + b` → `a--b` (spaces
on both sides of `+`), but `a: b` → `a-b` (only trailing space on
`:`). General markdown reference:
[markdownguide.org](https://www.markdownguide.org). GitHub
publishes no official spec for auto-generated anchors; the
de-facto reference implementation is
[github-slugger](https://github.com/Flet/github-slugger).

### Versioning

Every plan must start with a version bump. Choose the approach based
on scope:

- **Single-step** (recommended for mechanical/focused changes): bump
  directly to `X.Y.Z`, implement in one commit. Simpler history.
- **Multi-step** (for exploratory/large changes): bump to `X.Y.Z-0`,
  implement across multiple commits incrementing the numeric
  suffix. The final commit drops the suffix.

The plan should recommend one approach and get user approval before
starting.

A plan that picks up a `## Todo` item — or *re-scopes* an
existing `## In Progress` ladder to absorb one — **deletes that
`## Todo` entry in the same commit** (the version-bump / plan
commit). `## In Progress` is the sole record of that work until
close-out, when it moves to `## Done`; a cycle that merges
several entangled `## Todo` items deletes all of them. A `## Todo`
entry that duplicates current `## In Progress` work is a process
bug.

For multi-step:

1. Bump version to `X.Y.Z-0` with the plan and commit as a chore
   marker.
2. Implement in one or more `X.Y.Z-N` commits (increment N as
   needed).
3. Final commit bumps to `X.Y.Z` (no suffix), updates
   `notes/todo.md` and `notes/chores-*.md` — this is the "done"
   marker.

Multi-step cycles surface the ladder at the top of
`notes/todo.md > ## In Progress` as a bullet list with `(done)` /
`(current)` markers — see the file's intro paragraph for the
format. When starting a new step, the *first* edit is to mark
that step `(current)` in `notes/todo.md` — before any code/doc
work — so the In Progress view reflects what's actually
happening. The flip back to `(done)` is part of the pre-commit
checklist (item 6).

**Why numeric suffixes (`-0`, `-1`, …) rather than `-devN`:**
semver pre-release identifiers may consist of a single numeric
component, and they compare numerically per spec. So
`X.Y.Z-1 < X.Y.Z-2 < … < X.Y.Z` correctly orders the dev ladder
below the done marker. Cargo accepts this form. The `-dev` prefix
adds no information the git log doesn't already convey and
doubles typing per commit.

The final release commit (no suffix) signals completion rather than
amending prior commits. This keeps history readable and makes it easy
to see which commits were exploratory vs final.

For decomposition finer than `X.Y.Z-N` — sub-steps (`X.Y.Z-N.M`)
or sub-sub-steps (`X.Y.Z-N.M-K`) — see
[Sub-step Workflow](#sub-step-workflow). The cycle close-out
choice (squash to one commit vs land each sub-step separately)
is made at close-out, not cycle start.

See also [`notes/substep-protocol.md`](notes/substep-protocol.md)
for the full protocol — when to use, per-substep contract
(`cargo test --bins` is non-negotiable), navigation, the validated
close-out squash recipe, and recovery. Revset primitives the
protocol relies on (`@`, `@-`, `..`, `::`, prefix matching) live in
[`notes/jj-revsets.md`](notes/jj-revsets.md).

### Headings and entries that record a commit

A `chores-NN.md` `##` section header that records a specific
commit, the matching `todo.md > ## Done` entry, and any `[N]`
reference to that section all use **exactly that commit's
title** — `<type>: <desc> (<version>)`, the same string the
commit gets (see [Commit Message Style](#commit-message-style)).
E.g. the chores header `## refactor: port push to Context
(0.48.0-6)` and the Done line `- refactor: port push to Context
(0.48.0-6) [[3]]`. For a multi-step cycle the `## Done` entry
uses the close-out commit's title.

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
header — see below), a short intro paragraph (what changed and
*why*, a notch above file-list granularity), and any `###`
design subsections. It does **not** carry a per-file edit list —
that lives in the commit message body, which is the source of
truth for "what changed mechanically" (immutable, `git
show`-able, naturally scoped to the commit). The chores section
is the source of truth for the design thinking; the two
cross-link, neither restates the other.

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

### Pre-commit checklist

At cycle *start* (before the version bump): (a) backfill the
previous chores section's `Commits:` ref with the just-pushed
commit's URL + full SHA — see
[Chores commit references](#chores-commit-references); (b) if
this cycle picks up a `## Todo` item (or re-scopes `## In
Progress` to absorb one), delete that `## Todo` entry — see
[Versioning](#versioning).

Before proposing a commit, run all of the following and fix any issues:

1. `cargo fmt`
2. `cargo clippy`
3. `cargo test`
4. `cargo install --path . --locked` (if applicable) — `--locked`
   is required: without it, `cargo install` ignores `Cargo.lock`
   and re-resolves from scratch, which can pick incompatible
   versions even when `cargo build` / `cargo test` succeed.
5. Retest after install
6. Update `notes/todo.md` — for multi-step cycles, flip the
   just-completed step's marker from `(current)` to `(done)`
   **before** running `vc-x1 push`. The commit being pushed
   should reflect the new state. (The next step's `(current)`
   marker is set later, at the *start* of that step — see the
   Versioning multi-step section.)
7. Update `notes/todo.md` — at cycle close-out (final commit),
   move the entry from `## In Progress` to `## Done`.
8. Update `notes/chores-*.md` — add a section (header =
   provisional commit title; intro paragraph + any `###` design
   subsections; **no** per-file edit list — that's the commit
   body).
9. Sync the chores section header to the **final** commit title
   and update every markdown anchor back-reference to it.
10. Update `notes/README.md` — if functionality changed (new
    flags, new subcommands, changed behavior).

## Sub-step Workflow

Larger work decomposes into named depths. The version-suffix
scheme nests:

- **Single step** — one change, one commit + push + finalize.
- **Multi-step** — planned series within a target bump:
  `0.5.0 → 0.6.0-0`, `0.6.0-0 → 0.6.0-1`, …, `0.6.0-N → 0.6.0`.
- **Sub-step** — finer granularity within a step:
  `0.6.0-3.1 → 0.6.0-3.2 → … → 0.6.0-3` (close-out).
- **Sub-sub-step** — finer still:
  `0.6.0-3.4-0 → 0.6.0-3.4-1 → … → 0.6.0-3.4` (close-out).

The conventions below apply at **whichever leaf depth the
current cycle planned to** — sub-step, sub-sub-step, etc.
"Sub-step" reads as shorthand for that leaf level throughout
this section.

### Version suffix in titles and Cargo.toml

Sub-step commits carry the leaf-level version in commit titles
**and** `Cargo.toml`. So `vc-x1 -V` shows the active sub-step
at build time. Bump Cargo.toml at the start of each sub-step
(extends the existing `X.Y.Z-N` step-start bump rule down a
level).

Cargo accepts arbitrary numeric segments in semver pre-release
identifiers; lexical comparison gives the expected ordering
(`0.6.0-3.4-1 < 0.6.0-3.4-2 < 0.6.0-3.4`).

### todo.md status flips

Markers in `notes/todo.md > ## In Progress` flip on a defined
cadence:

- **Start of (M):** mark (M) `(current)` as the first edit.
- **End of (M):** flip (M) `(current)` → `(done)` **before**
  the cargo cycle and commit. The commit captures the
  completed state.
- **Start of (M+1):** mark (M+1) `(current)` as that
  sub-step's first edit.

Each sub-step's commit carries the "this sub-step is done"
record; the next sub-step's commit carries "next sub-step
starts."

### Pre-commit cargo cycle (per sub-step)

Run before every sub-step commit (not just at cycle close-out):

1. `cargo fmt`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo install --path . --locked`
5. (re-test if anything substantive)

Keeps every intermediate commit buildable so bisection works
across the cycle's stack.

### Commit-first review model

Per sub-step:

1. Make the sub-step changes.
2. Run the cargo cycle.
3. **Commit immediately** (both repos with ochid trailers, no
   separate approval gate at sub-step granularity).
4. Summarize the commit briefly in chat.
5. User reviews the committed revision in their editor (full
   file context, not chat-pasted diffs).
6. User iterates if needed; bot squashes follow-ups into the
   sub-step commit via `jj squash --into @-` (and
   `jj describe @-` — plus the chores section header and its
   anchor back-refs — if the title needs to change).
7. User signals "go to (M+1)" to advance.

This **replaces** the per-sub-step review pause that the
top-level `### Review before proposing the commit block` and
`### Per-file review checkpoints` rules would otherwise
impose. Local jj commits are mutable until close-out, so
committing freely is safe.

The two-gate ceremony (review + message approval) is preserved
for the **cycle-level push** at close-out — that crosses the
local→remote boundary and warrants explicit approval.

### Ochid trailers on sub-step commits

Sub-step commits include ochid trailers paired across the two
repos (same shape as top-level cycle commits):

- App body: `ochid: /.claude/<.claude-chid>`
- `.claude` body: `ochid: /<app-chid>`

Use `vc-x1 chid -s code,bot -L` to capture both pre-commit
change IDs (first line app, second `.claude`).

### `.claude` cadence

`.claude` commits **per sub-step alongside the app repo**.
Each sub-step's `.claude` commit captures the session state at
that moment.

Alternative considered (`.claude` accumulates across the cycle
and commits once at close-out) was rejected — keeping the
per-sub-step pairing preserves flexibility (any sub-step can
be promoted to its own push without restructuring).

**`.claude` is a linear journal — all session work lives on
`main`.** The repo has no need for parallel feature-branch
bookmarks to mirror app-side branches. When the app sits on
e.g. `init-clone-refactor`, `.claude` still commits to `main`.
Cross-references between sides are carried by the `ochid:`
trailer + commit timestamps; that's enough to associate
session activity with whichever app-side branch the cycle was
on.

**Do not create or maintain `.claude` bookmarks that mirror
app-side branches.** This was tried once during 0.41.1-6.7 on
the impression that an app-side fork needed a `.claude`
partner; the partner bookmark went unused for the cycle and
the work landed on `.claude main` regardless. Keeping such
bookmarks risks the bot misreading them as "this branch needs
to advance" and steering session pushes to the wrong remote
ref.

### Cycle close-out — squash or keep separate?

Two valid shapes for landing the sub-step stack on `main`:

- **Squash to one cycle commit** — single entry on `main`;
  sub-step granularity preserved only in the commit body's
  edit list. Right when the work is one logical change with
  intermediate validation points. **Cost:** the per-sub-step
  chores sections collapse into one, whose header becomes the
  single close-out commit title, and every anchor back-ref to a
  now-gone per-sub-step section gets re-pointed — real work,
  done as part of the squash.
- **Keep separate (N + 1 commits)** when the decomposition is
  itself informative (different conceptual stages, design
  progression worth showing in `git log`). Used on
  `0.41.1-6.7` (8 sub-sub-step commits + 1 close-out commit).
  Each section keeps its own header / `Commits:` ref — no
  consolidation churn.

Pick at close-out, not at cycle start. No firm default — weigh
the squash's chores-consolidation cost against a cleaner
`git log`; that cost biases toward keeping separate unless the
decomposition genuinely isn't informative.

### Reviewing committed sub-steps

The commit-first model assumes the reviewer can read the diff
of an already-committed revision. Don't `jj edit -r @-` back
into a past commit to view it — that marks the commit mutable,
shifts the WC pointer, forces a `jj new -r <head>` recovery.
Use one of:

**Terminal (always works):**

```
jj diff -r @-                  # diff of previous commit
jj diff --from <X> --to <Y>    # diff between two arbitrary revs
jj show -r <X>                 # description + diff for one rev
jj log -r @-..@                # what's between two points
```

Pipe through `delta` / `diff-so-fancy | less -R` for color and
side-by-side.

**External diff tool** — configure jj to launch your editor:

```
# ~/.config/jj/config.toml
[ui]
diff-editor = ["zed", "--diff", "$left", "$right"]
```

Then `jj diff -r @-` opens the configured tool. Works for
arbitrary `--from`/`--to` ranges.

**VS Code** — Source Control panel → Commit Graph →
right-click commit A → "Copy Commit ID" → right-click commit
B → "Compare with…" → paste / pick A. Two-commit diff opens
with the changed-files list. GitLens extension adds richer
"Open Comparison" actions; `Git: Compare with…` in the
Command Palette is a fallback.

## Code Conventions

### Doc comments on every file, function, and method

Every `.rs` file must begin with a `//!` module docstring. Every
function and method must have a `///` doc comment. Keep them brief —
one sentence of purpose is often enough; the discipline is that the
comment exists, not that it be long.

This is a deliberate override of the generic "write no comments"
default that applies to inline `//` comments. Doc comments on the
module / item surface are expected; inline explanatory comments
inside function bodies remain discouraged unless they capture a
non-obvious WHY.

**Shape:** short intro (1–3 sentences, shorter is better), then
a `-` bullet list for any structured details. Avoid long prose
paragraphs — they read as a wall of text and hide the structure
that bullets make scannable. Same shape applies to:

- Module / function / struct / field doc comments in `.rs` files.
- Chore descriptions in `notes/chores-NN.md`.
- Todo entries in `notes/todo.md` (when an item needs more than
  one line of detail; pure one-liners are still fine).

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

## ochid Trailers

Every commit body must include an `ochid:` trailer pointing to the
counterpart commit in the other repo. The value is a workspace-root-relative
path followed by the changeID:

- App repo commits point to `.claude`: `ochid: /.claude/<changeID>`
- Bot session commits point to app repo: `ochid: /<changeID>`

Use `vc-x1 chid -s code,bot -L` to get both changeIDs (first line
is app repo, second is `.claude`).

## Commit-Push-Finalize Flow

**Use `vc-x1 push <bookmark>` — it wraps the full flow in one
command with two interactive approval gates.** Push runs
preflight (sync + fmt + clippy + test), prompts for review,
composes the commit message (via `--title`/`--body` or `$EDITOR`),
prompts for message approval, then commits both repos → advances
bookmarks → pushes app → finalizes `.claude`. Failures inside the
local mutation window roll both repos back via `jj op restore`;
after `push-app` succeeds the remote boundary is crossed and
recovery is forward-only.

**Run this flow after every step** — not only at session end.
Single-step and multi-step changes are of equal importance: a
single-step change is one `push` invocation; a multi-step change
is one `push` per `X.Y.Z-N` commit plus one for the final release
commit. Each step gets its own commit, its own push, and its own
finalize — so dev markers land on the remote and in `.claude` as
they happen rather than being batched until the end.

### Run `vc-x1 push`

`<bookmark>` below is the **app-side working branch** — the
bookmark sitting at the tip of the chain `vc-x1 push` should
advance and push. For most cycles this is `main`; for
feature-branch work it's the feature-branch name (e.g.
`init-clone-refactor`). Pass the literal name, not the
placeholder. **`.claude` always pushes its `main`** regardless
of the app-side bookmark passed (see
[`.claude` cadence](#claude-cadence) — `.claude` is a linear
journal).

```
vc-x1 push <bookmark>                                  # interactive (review + $EDITOR)
vc-x1 push <bookmark> --title "..." --body "..."       # flags skip $EDITOR
vc-x1 push <bookmark> --yes --title "..." --body "..." # full non-interactive
vc-x1 push <bookmark> --dry-run                        # preview (no side effects)
vc-x1 push <bookmark> --from commit-app                # resume from specific stage
vc-x1 push --status                                    # show saved state
vc-x1 push <bookmark> --restart                        # clear saved state; start fresh
```

**Bookmark mismatch is currently silent.** `vc-x1 push`
doesn't verify that `<bookmark>` matches the working-copy
chain — passing a bookmark whose tip isn't an ancestor of `@`
will (silently) push that bookmark anyway, leaving your work
unpushed. Always confirm `jj log -r <bookmark>..@` shows the
commits you mean to send. A safety check is on the 0.42.0
todo (`vc-x1 push: --scope` flag item).

The two approval gates are surfaced by push itself:

1. **Review** — push prints `jj diff --stat` for both repos and
   prompts `[y/N]`. Approve = "the work is done right".
2. **Message** — push either uses `--title`/`--body` (non-editor
   path) or opens `$EDITOR` on a template. Approve = "the message
   reads right".

Both titles and bodies are the **same** across the two commits;
only the `ochid:` trailer differs per repo. Push collects the
pre-commit chids internally so you don't hand-manage them.

For the full flag list and stage machine, see `vc-x1 push --help`
and `notes/chores-05.md > Add push subcommand (0.37.0)`.

### Bot communication during the flow

When applying the flow on the user's behalf, use plain prose at
each gate — no insider jargon ("Gate N signal", "Checkpoint N",
etc.):

1. **After completing the work** — summarize what changed
   (file-by-file or feature-by-feature, terse) and end with
   something like:

   > Work complete. Please review. On approval I'll prep the
   > commit title and body for a second approval before pushing.

2. **After review approval** — present the proposed `$TITLE` and
   `$BODY` explicitly, then ask permission to run `vc-x1 push`.
   Do **not** spell out the full
   `vc-x1 push <bookmark> --yes --title "$TITLE" --body "$BODY"`
   invocation by default — it's mechanical and obvious from the
   title/body. Show it only on explicit request (verbose mode /
   debugging).

3. **After execution approval** — run the push command. `push`
   handles commit + bookmark + push + finalize internally;
   nothing should be output after the push command (finalize is
   detached and absolute-last; see "After finalize: stop and
   wait" below).

### Pre-step: `vc-x1 sync` (still useful)

Push's preflight runs `vc-x1 sync --check` as its first step. Check
mode verifies divergence without applying — if anything is `behind`
or `diverged`, preflight errors and you resolve explicitly with
`vc-x1 sync --no-check` before re-running push. This keeps the
two approval gates (review, message) authoritative; push never
performs an unsupervised rebase.

Running sync manually before you *start* editing is still cheap
(one line when clean) and surfaces remote changes earlier:

```
vc-x1 sync --check                    # dual-repo workspace, verify
vc-x1 sync --no-check                 # apply (rebase / fast-forward)
vc-x1 sync --check -R .               # single-repo project
vc-x1 sync --check --quiet            # silent; exit code signals result
```

Bots and scripts must pass `--check` or `--no-check` explicitly —
defaults can shift, explicit flags lock in the contract. Interactive
use can take the default (which is `--check`).

Output shape:

- **Clean**: one line — `sync: N repos, all bookmarks up-to-date`.
  Note: scope is bookmark-vs-remote tracking, not working-copy
  cleanliness — `@` may have uncommitted changes; sync doesn't
  speak to that. Proceed.
- **Action needed** (`behind` / `diverged`) under `--check`:
  per-repo fetch + state lines, then a fatal
  `sync: N repos need action — resolve with vc-x1 sync --no-check
  and re-run`. Inspect, run `vc-x1 sync --no-check`, then proceed.
- **`--quiet`**: no output; exit code is the only signal.

### After finalize: stop and wait

After `vc-x1 finalize` is launched — **whether mid-session per-step
or at session end** — you **MUST NEVER** proceed to a next step, edit
files, run tools, or emit any text (prose, recaps, acknowledgements),
until the user explicitly directs you to continue. Treat finalize as
a hard stop for the whole turn. Any final words (e.g. "next is ...")
must be said in the approval prompt *before* executing finalize; the
finalize `Bash` call is the last thing in the turn and nothing
follows it.

This holds even when the next step seems obvious (e.g. "next is
N+1" or "now I should bump the version and commit the release").
Wait. The user controls cadence — every push+finalize is a checkpoint
they may want to inspect, think about, hand off, or take a break at.
Auto-proceeding bypasses that checkpoint and produces unwanted writes
between finalize and the next explicit instruction.

Exceptions to this rule may emerge later but are not authorized at
this stage. Until told otherwise, treat as absolute.

### Late changes after push

If the app repo needs a tweak after `push-app` succeeded (e.g.
updating CLAUDE.md or memory), the commit is immutable. Use
`--ignore-immutable` to squash the changes into `@-` — the
bookmark moves with the rewritten commit — then re-push:

```
jj squash --ignore-immutable -R .
jj git push --bookmark <bookmark> -R .
```

If you squash somewhere other than `@-`, add
`jj bookmark set <bookmark> -r <target> -R .` between those
two commands so the bookmark lands on the rewritten commit.

`.claude` is also mutable via this pattern when needed, though
push's `finalize-claude` stage normally handles trailing session
writes so you rarely hit this case there.

### Manual finalize fallback

If push exited before `finalize-claude` (e.g. `--no-finalize`
was set, or a failure between `push-app` and `finalize-claude`),
run finalize by hand:

```
vc-x1 finalize --repo .claude --squash --push <bookmark> --delay 10 --detach --log /tmp/vc-x1-finalize.log
```

**Nothing should happen after finalize** — no memory writes, no
tool calls, no additional output. If any work is done after
finalize, run finalize again so the trailing writes are captured.
Do **not** echo or restate the finalize output — the Bash tool
already displays it.

**Clear push's saved state after any out-of-band recovery** —
manual `vc-x1 finalize`, manual `jj squash --ignore-immutable`
+ force-push, etc., all leave `.vc-x1/push-state.toml` pointing
at a now-stale halt point. Either `rm .vc-x1/push-state.toml`
or run `vc-x1 push <bookmark> --restart` (which clears and
restarts in one go) before the next `vc-x1 push`. Otherwise
push resumes from a bogus stage and can falsely declare
success.
