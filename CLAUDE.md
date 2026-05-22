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

## Prose form

Long-lived prose on this project follows one shape: a short intro
that explains the *why* or the high-level *what*, then a `-` bullet
list for the details. Wrap lines at ≤72 cols (bullet continuations
indent two spaces). Avoid wall-of-prose paragraphs — they hide the
structure that bullets make scannable.

Surfaces that use this shape:

- Module / function / struct / field doc comments in `.rs` files —
  see [Doc comments](#doc-comments-on-every-file-function-and-method).
- Commit message bodies (both app-repo and session-repo). The
  ≤50-col title is the commit-specific add-on; see
  [Per-commit flow](#per-commit-flow).
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

## Pre-commit Requirements

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

### Headings and entries that record a commit

A `chores-NN.md` `##` section header that records a specific
commit, the matching `todo.md > ## Done` entry, and any `[N]`
reference to that section all use **exactly that commit's
title** — `<type>: <desc> (<version>)`, the same string the
commit gets (see [Per-commit flow](#per-commit-flow)).
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

Every change runs as a **cycle**: Preparation → Work-N →
Close-out. One protocol; push and squash are actions you take
when wanted, not modes chosen up front. The mechanics below
run in the order they happen.

### Numbering

The version suffix on each commit carries its phase, with one
separator — `.` — at every depth (semver mandates `-` once at
the start of the prerelease, then `.` for each level after):

- `X.Y.Z-0` — Preparation
- `X.Y.Z-1`, `X.Y.Z-2`, … — Work commits
- `X.Y.Z` — Close-out (bare version, no suffix)

A **trailing `0` identifier marks a Preparation**, and the
rule nests to any depth: when a Work commit needs subdividing
into its own sub-cycle, append another level — `X.Y.Z-3.0`
(Preparation of the `-3` sub-cycle), `X.Y.Z-3.1` /
`X.Y.Z-3.2` (its Work), `X.Y.Z-3` (its Close-out),
`X.Y.Z-3.1.0` (Preparation of the `-3.1` sub-sub-cycle), and
so on. A sub-cycle needing no Preparation just omits the `.0`
(`-3.1`, `-3.2`, `-3`); one that grows a Preparation later
adds `-3.0` with no renumbering of siblings.

Each commit carries its phase version in its title **and**
`Cargo.toml` — bump `Cargo.toml` at the start of each phase,
so `vc-x1 -V` shows the active phase at build time.

**Ordering caveat.** The one ordering guarantee semver gives
is that the bare `X.Y.Z` close-out outranks every `X.Y.Z-…`
dev version. Within the ladder it does *not* hold for nested
close-outs — `X.Y.Z-3` sorts *before* its children
`X.Y.Z-3.0` / `X.Y.Z-3.1`, because a prefix always precedes
its extensions. Cosmetic — nothing depends on within-ladder
order — but don't claim monotonicity.

**Why numeric suffixes** (`-0`, `-1`, …) rather than `-devN`:
semver pre-release identifiers compare per spec and Cargo
accepts them. `-dev` adds no information the git log doesn't
carry and doubles typing per commit. The final close-out
commit (bare `X.Y.Z`) signals completion rather than amending
prior commits — history stays readable, exploratory vs final
commits obvious.

### Preparation

The cycle's first commit (`X.Y.Z-0`):

- **Backfill the previous cycle's chores `Commits:` ref.**
  Grab the just-pushed commit's URL + full SHA, fill in the
  newest chores section that was left `Commits:`-less. See
  [Chores commit references](#chores-commit-references).
- **Bump the version** in `Cargo.toml` to `X.Y.Z-0`.
- **Pick up a `## Todo` item** (if the cycle has one). Move
  that entry's text out of `## Todo` and into `## In
  Progress` — its overview and to-do list, followed by the
  plan ladder — so `## In Progress` is self-describing, not
  a bare ladder. A cycle that merges several entangled
  `## Todo` items moves all of them. A `## Todo` entry that
  duplicates current `## In Progress` work is a process bug.
- **Open the chores section.** Header = provisional commit
  title; intro that **mirrors** the `## In Progress` content
  so the cycle stays readable once `## In Progress` is
  cleared at close-out; any `###` design subsections. No
  per-file edit list — that lives in the commit body.

### Per-commit flow

Every commit (Preparation, each Work commit, Close-out) goes
through:

1. **Mark this commit `(current)`** as the first edit in
   `notes/todo.md > ## In Progress`, before any code/doc
   work.
2. **Do the work.**
3. **Flip this commit `(current)` → `(done)`** in `## In
   Progress` — before the cargo cycle and the commit, so
   the commit captures the completed state.
4. **Cargo cycle** (skip-able for purely-docs commits; full
   cycle mandatory at close-out):
   1. `cargo fmt`
   2. `cargo clippy --all-targets -- -D warnings`
   3. `cargo test`
   4. `cargo install --path . --locked`
   5. (re-test if anything substantive changed)
5. **Gate 1 — work review.** Stop *before* writing any
   description; tell the user "ready to commit." The user
   reviews the working-copy changes in their editor. Most
   change requests come here — iterate until the work is
   right.
6. **Write the commit description.** [Conventional
   Commits](https://www.conventionalcommits.org/) with a
   version suffix:

   ```
   <type>: <short description> (<version>)
   ```

   - **Title**: ≤50 chars. Common types: `feat`, `fix`,
     `refactor`, `test`, `docs`, `chore`. The version
     suffix counts toward the 50 — favor terse phrasings.
   - **Body**: [Prose form](#prose-form) (intro + bullets),
     wrap ≤72. App-repo bullets are **file-by-file** — one
     bullet per file changed, file plus a one-line gist.
     When a file has several distinct changes, list them
     as `--` sub-bullets under the file bullet (commit
     messages aren't rendered as markdown, so the
     conventional nested `- ` would be ambiguous):

     ```
     - path/to/file
       -- first distinct change
       -- second distinct change, wrapping to next line
          with continuation indented 5
     ```

     The file-by-file list is the source of truth for the
     cycle's mechanical change record; chores carries the
     narrative + design, not a copy of it. Promote any
     "why" beyond one sentence to a chores `###`
     subsection.
   - **`ochid:` trailer** as the last line of the body —
     see [ochid trailers](#ochid-trailers).
   - **Session-repo (`.claude`) body**: same shape; bullets
     describe in-session activity rather than code changes.
7. **Gate 2 — message review.** Show the title + body and
   stop. The user reviews the description. Iterate.
8. **Commit.** `jj commit -m "title" -m "body" -R .` for
   the app repo, `-R .claude` for the bot repo (`-R` last
   keeps the verb visible):

   ```
   jj commit -m \
   "<type>: <short description> (<version>)" \
   -m "<intro paragraph>

   - file1: gist
   - file2: gist

   ochid: /.claude/<chid>" \
   -R .
   ```

**Two overrides apply:**

- **Deviation or question** — any time the work deviates
  from the agreed plan, or a question arises, stop and
  surface it; don't push through.
- **ESC-ESC** — the user can interrupt at any point to pull
  a review or question forward.

### Reviewing changes

The two-gate model reviews **uncommitted working-copy
changes** in your editor, on the way to commit. Zed shows
the diff naturally; for terminal use:

```
jj diff                          # working-copy diff (uncommitted)
jj diff -r @-                    # diff of the previous commit
jj diff --from <X> --to <Y>      # any two revisions
jj show -r <X>                   # description + diff for one rev
```

Pipe through `delta` or `diff-so-fancy | less -R` for color
and side-by-side. To launch your editor as the diff tool:

```
# ~/.config/jj/config.toml
[ui]
diff-editor = ["zed", "--diff", "$left", "$right"]
```

Then `jj diff --from X --to Y` opens it. Don't `jj edit
-r @-` to view a past commit — that marks it mutable and
shifts the working-copy pointer; use `jj diff -r @-` or
`jj show -r @-`.

[`notes/substep-protocol.md`](notes/substep-protocol.md) is
a longer-form companion (close-out squash recipe, recovery)
that predates this rewrite and still uses the old "sub-step"
vocabulary — reconciling it is a follow-up. Revset
primitives (`@`, `@-`, `..`, `::`, prefix matching) are in
[`notes/jj-revsets.md`](notes/jj-revsets.md).

### Close-out

The cycle's last commit (bare `X.Y.Z`):

- **Move the picked-up item** from `## In Progress` to a
  one-line entry (with a chores `[N]` ref) in `## Done`.
- **Finalize the chores section.** Sync the section header
  to the **final** commit title and update every markdown
  anchor back-reference to it; add an `### As-built ladder`
  listing the cycle's commits; `### Outcome` notes if
  useful.
- **Update `notes/README.md`** if functionality changed
  (new flags, new subcommands, changed behavior).

Whether to **squash** the cycle into one commit before the
publishing push, or push as-is, is decided at push time —
see [Pushing](#pushing).

### Pushing

**Push is discretionary** — done when you want to back up
work or publish interim progress, and **always at
close-out** (the cycle's result must be published). Interim
pushes are a judgment call; nothing forces push-per-commit.

**Squash, merge, or keep at the close-out push.** Before
the publishing push, decide what shape the cycle lands as:

- **Squash to one commit** — single entry on the target;
  per-commit granularity preserved only in the commit
  body's edit list. Right when the work is one logical
  change with intermediate validation points. Cost: the
  per-commit chores sections collapse into one (its header
  becomes the close-out commit title), and every anchor
  back-ref to a now-gone section gets re-pointed — real
  work, done as part of the squash.
- **Merge (non-FF)** — `main` gains one merge commit
  (`X.Y.Z`) with parents `<prev>` and the cycle tip; the
  sub-step commits stay reachable as the merge's second
  parent. `git log` shows everything; `git log
  --first-parent` shows main as one jump per cycle. Best
  when the sub-steps are worth preserving but main should
  read linearly. The paired `.claude` commit's body
  carries a multi-line ochid listing every app commit in
  the push — the multi-ochid case
  [`notes/forks-multi-user.md`](notes/forks-multi-user.md)
  designs. Set up with `jj rebase -r <tip> -d <prev> -d
  <sub-tip>` before invoking `jj git push`.
- **Keep separate** — one commit per cycle entry on main —
  when the decomposition is itself informative and you
  want `git log` to show every step. Each chores section
  keeps its own header / `Commits:` ref; no consolidation
  churn.

If squashing or merging, do it before invoking `vc-x1
push` (or push manually with `jj git push` for
non-standard shapes).

The flow is wrapped by `vc-x1 push <bookmark>`, which runs:

1. Preflight (`vc-x1 sync --check` + `cargo fmt` / `clippy`
   / `test`).
2. **Review-gate prompt** — shows `jj diff --stat` for both
   repos and asks `[y/N]`.
3. **Message-gate prompt** — uses `--title` / `--body` or
   opens `$EDITOR`.
4. Commits both repos with ochid trailers, advances the
   bookmark, `jj git push --bookmark <bookmark>` for the
   app, finalizes `.claude` (its single per-push commit).

`<bookmark>` is the **app-side working branch** (often
`main`, sometimes a feature branch). `.claude` always pushes
its `main` regardless of the app-side bookmark. See
`vc-x1 push --help` for the full flag list, resumability
(`--from`, `--restart`, `--status`), and recovery; CLAUDE.md
keeps only the rules that govern bot behavior. `vc-x1 sync
--check` may also be run manually before editing to surface
remote changes early.

**`.claude` cadence — once per push.** The `.claude`
working-copy node accumulates session data across the
cycle's commits and is described + committed when a push
happens — one push, one `.claude` commit, paired with every
code commit in that push. Its change ID is stable from the
first code commit that references it through to that
`.claude` commit (working-copy snapshots, `jj describe`, and
the finalize `jj commit` all preserve the change ID), so the
`ochid:` trailers resolve across the push.

`.claude` is a linear journal — all session work lives on
`main`. The repo has no need for parallel feature-branch
bookmarks to mirror app-side branches; when the app sits on
a feature branch, `.claude` still commits to `main`.
Cross-references between sides are carried by the `ochid:`
trailer + commit timestamps; that is enough. **Do not
create or maintain `.claude` bookmarks that mirror app-side
branches** — tried during 0.41.1-6.7, went unused, and
risks the bot steering session pushes to the wrong remote
ref.

**Bot communication at the gates.** Use plain prose — no
insider jargon ("Gate N signal", "Checkpoint N", etc.):

- **At work-review** — summarize what changed and stop.
  "Work complete. Please review."
- **At message-review** — present `$TITLE` and `$BODY`
  explicitly, then ask permission to commit (or push).
  Don't spell out the full
  `vc-x1 push <bookmark> --yes --title "$TITLE" --body
  "$BODY"` invocation by default; it's mechanical and
  obvious from title/body. Show it only on explicit
  request.

**After finalize: stop and wait.** After `vc-x1 finalize`
launches — whether mid-session per-push or at session-end —
you **MUST NEVER** proceed to a next step, edit files, run
tools, or emit any text (prose, recaps, acknowledgements),
until the user explicitly directs you to continue. Treat
finalize as a hard stop for the whole turn. Any final words
must be said in the approval prompt *before* executing
finalize; the finalize `Bash` call is the last thing in the
turn and nothing follows it.

This holds even when the next step seems obvious. Wait. The
user controls cadence — every push+finalize is a checkpoint
they may want to inspect, think about, hand off, or take a
break at. Auto-proceeding bypasses that checkpoint.

If push exits before `finalize-claude` (e.g. failure between
`push-app` and `finalize-claude`), run finalize by hand:

```
vc-x1 finalize --repo .claude --squash --push <bookmark> --delay 10 --detach --log /tmp/vc-x1-finalize.log
```

Clear push's saved state after any out-of-band recovery —
`rm .vc-x1/push-state.toml` or `vc-x1 push <bookmark>
--restart` — otherwise push resumes from a stale stage.

A late code-repo tweak after `push-app` succeeded (e.g.
updating CLAUDE.md or memory) requires `jj squash
--ignore-immutable` and a re-push; that is a remote rewrite
and needs explicit approval like any push.

### ochid trailers

Every commit body carries `ochid:` trailer(s) linking it to
its counterpart(s) in the other repo. The path is
workspace-root-relative — `/.claude/<chid>` from the app
side, `/<chid>` from the `.claude` side.

Because `.claude` commits once per push:

- **Code-side commits** each carry one
  `ochid: /.claude/<.claude-chid>` — the `.claude`
  working-copy change ID, stable until `.claude` is
  committed at push time.
- **The `.claude` commit** carries one `ochid: /<code-chid>`
  per code commit in that push: a single trailer when one
  code commit is pushed (the squashed case), a list when
  several are. The multi-line `ochid:` list and its
  fork / multi-user generalization are designed in
  [`notes/forks-multi-user.md`](notes/forks-multi-user.md).

Use `vc-x1 chid -s code,bot -L` to capture the change IDs
(first line app, second `.claude`).

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

