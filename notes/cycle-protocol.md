# Cycle protocol

## Cycles

A cycle has three phases:

- **[Preparation](#preparation)** (`X.Y.Z-0`) — the cycle's
  first commit. Sets up the cycle:
  - Backfill the previous cycle's chores `Commits:` ref.
  - Bump `Cargo.toml` to `X.Y.Z-0`.
  - Pick up a `## Todo` item (typically via `## Priorities`)
    into `## In Progress` (bold title + succinct problem
    statement + plan ladder).
  - Open the [chores section](#chores-sections).
- **[Work-N](#work-n)** (`X.Y.Z-1`, `X.Y.Z-2`, …) — the
  commits that implement the change. As many as the change
  needs; each runs through the
  [per-commit flow](#per-commit-flow).
- **[Close-out](#close-out)** (bare `X.Y.Z`) — the cycle's
  last commit. Bookkeeping only:
  - Move the picked-up item to a one-line entry in
    `## Done`.
  - Move the `## In Progress` block into the
    [chores section](#chores-sections).
  - Optionally update `notes/README.md` if functionality
    changed.

A cycle's commits are published to the project remote
either incrementally or as one batch at close-out; the
result must always be published at close-out. See
[Pushing](#pushing).

**Sub-cycles.** When a Work commit's scope grows enough to
warrant its own ladder, it subdivides — `X.Y.Z-3.0`
Preparation, `X.Y.Z-3.1` / `X.Y.Z-3.2` Work, `X.Y.Z-3`
Close-out. The same three-phase shape applies recursively
at every depth. See [Numbering](#numbering) for the
suffix rule and [Sub-cycle ladders](#sub-cycle-ladders)
for the local-ladder mechanics.

## Chores sections

A **chores section** is a `##` section in
`notes/chores/chores-NN.md` recording one cycle:

- One section per cycle is the default.
- Per-commit sections (like 0.55.0's four) are a
  deliberate alternative.

The phrase **"Open" the chores section** means append a
`##` header to the current `notes/chores/chores-NN.md`
with the cycle's anticipated close-out title (e.g.
`## refactor: foo bar (X.Y.Z)`). The body is empty;
content arrives at close-out (see [Close-out](#close-out)).

Fuller chores conventions (content rules, header sync,
design subsection pattern, `Commits:` formatting) live in
CLAUDE.md [Chores conventions](../CLAUDE.md#chores-conventions).

## Preparation

The cycle's first commit (`X.Y.Z-0`):

- **Backfill the previous cycle's chores section
  `Commits:` ref** — see
  [Chores commit references](../CLAUDE.md#chores-commit-references).
- **Bump the version** in `Cargo.toml` to `X.Y.Z-0`.
- **Pick up a `## Todo` item** (if the cycle has one).
  Move into `## In Progress` as:
  - A **bold title line** — exact match of the chores
    section header, minus the `## ` prefix.
  - A **succinct problem statement**.
  - A **plan ladder**.

  See [Prose form](../CLAUDE.md#prose-form). A Todo that
  duplicates current `## In Progress` work is a process
  bug.
- **Open the [chores section](#chores-sections)** —
  append a `##` header with the cycle's anticipated
  close-out title.

## Work-N

The cycle's work commits (`X.Y.Z-1`, `X.Y.Z-2`, …)
implement the change. As many as needed:

- Each commit runs through the
  **[per-commit flow](#per-commit-flow)**.
- **Interim pushes** are optional (backup, progress
  visibility).
- Close-out is the only mandatory push (see
  [Pushing](#pushing)).
- **Subdivide into a sub-cycle** if a Work commit's
  scope grows enough (see
  [Sub-cycle ladders](#sub-cycle-ladders)).

## Close-out

The cycle's last commit (bare `X.Y.Z`) does bookkeeping
only — the commit body describes that bookkeeping, not
what happens post-squash:

- **Move the picked-up item** from `## In Progress` to a
  one-line entry (with a chores `[N]` ref) in `## Done`.
- **Move the `## In Progress` block into the
  [chores section](#chores-sections)** (the title-only
  placeholder opened during Preparation):
  - Cut the problem statement + plan ladder out of
    `## In Progress` (drop the bold title line — the
    chores `##` header already carries that string).
  - Paste under the chores `##` header.
  - Sync the chores header to the **final** commit title
    if the cycle's scope shifted; update every anchor
    back-reference.
  - Add an `### As-built ladder` listing the cycle's
    commits; optional `### Outcome` notes.
  - Replace `## In Progress` with
    `_No cycle currently in progress._`.
- **Update `notes/README.md`** if functionality changed
  (new flags, new subcommands, changed behavior).

Whether to **squash** the cycle into one commit before the
publishing push, or push as-is, is decided at push time —
see [Pushing](#pushing).

## Numbering

The version suffix on each commit encodes its phase —
the **final identifier `0` marks a Preparation**:

- `X.Y.Z-0` — Preparation
- `X.Y.Z-1`, `X.Y.Z-2`, … — Work commits
- `X.Y.Z` — Close-out (bare version, no suffix)

Disambiguation:

- `-10` — Work commit #10 (final identifier `10`), not a
  Preparation.
- `-1.0` — Preparation of the `-1` sub-cycle (final
  identifier `0`).

**Nesting.** Sub-cycles append another level, recursively:

- `X.Y.Z-3.0` — Preparation of the `-3` sub-cycle
- `X.Y.Z-3.1`, `X.Y.Z-3.2` — its Work
- `X.Y.Z-3` — its Close-out
- `X.Y.Z-3.1.0` — Preparation of the `-3.1` sub-sub-cycle

Sub-cycles needing no Preparation omit `.0` (`-3.1`,
`-3.2`, `-3`); one that grows a Preparation later adds
`-3.0` without renumbering siblings.

Bump `Cargo.toml` at the start of each phase so `vc-x1 -V`
reports the active phase at build time.

## Per-commit flow

Every commit (Preparation, each Work commit, Close-out) goes
through:

1. **Mark this commit `(current)`** as the first edit in
   `notes/todo.md > ## In Progress`.
2. **Do the work** (see [Iterative work](#iterative-work)
   for the loop-and-squash technique).
3. **Flip this commit `(current)` → `(done)`** in `## In
   Progress` — before the cargo cycle and the commit.
4. **Cargo cycle** (skip-able for purely-docs commits; full
   cycle mandatory at close-out):
   1. `cargo fmt`
   2. `cargo clippy --all-targets -- -D warnings`
   3. `cargo test`
   4. `cargo install --path . --locked`
   5. (re-test if anything substantive changed)
5. **Work review.** Stop *before* writing any description;
   tell the user "ready to commit." The user reviews the
   changes and we iterate until complete.
6. **Write the commit description** — see
   [Commit description](#commit-description).
7. **Commit Description review.** Show the title + body
   and stop. The user reviews the description. Iterate.
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

## Commit description

[Conventional Commits](https://www.conventionalcommits.org/)
with a version suffix:

```
<type>: <short description> (<version>)
```

### Title

- ≤50 chars total (version suffix counts).
- Common types: `feat`, `fix`, `refactor`, `test`,
  `docs`, `chore`.
- Favor terse phrasings.

### Body

[Prose form](../CLAUDE.md#prose-form) (intro + bullets),
wrap ≤72. Bullet content differs per repo:

- **App-repo body**: file-by-file. One bullet per file
  changed (file plus a one-line gist). Sub-bullets for
  files with multiple distinct changes:

  ```
  - path/to/file1
    - first distinct change
    - second distinct change, wrapping to next line
       with continuation indented 5
  ```

  The file-by-file list is the source of truth for the
  cycle's mechanical change record. Chores carries the
  narrative + design, not a copy of it. Promote any
  "why" beyond one sentence to a chores `###` subsection.

- **Session-repo (`.claude`) body**: bullets describe
  in-session activity rather than code changes.

### Trailer

`ochid:` as the last line of the body — see
[ochid trailers](#ochid-trailers).

## Reviewing changes

Work review looks at the **uncommitted working-copy diff**,
on the way to commit. The user opens diffs in their
editor (Zed, VSCode); jj commands are for terminal:

- `jj diff` — working-copy diff (uncommitted)
- `jj diff -r @-` — diff of the previous commit
- `jj diff --from <X> --to <Y>` — any two revisions
- `jj show -r <X>` — description + diff for one rev

Don't `jj edit -r @-` to view a past commit — that marks
it mutable and shifts `@`; use `jj diff -r @-` or
`jj show -r @-`.

See [Sub-cycle ladders](#sub-cycle-ladders) for the
close-out squash recipe and recovery; revset primitives
are in [`jj-revsets.md`](jj-revsets.md).

## Pushing

### Policy

Push is **discretionary** during the cycle (backup,
progress visibility) and **mandatory at close-out** —
the cycle's result must be published.

### Shape at close-out push

Decide the shape and get user approval before pushing —
the choice is hard to undo once on the target. Surface
the options, wait for the user.

- **Squash to one commit** — single entry on the target.
  Right for straightforward changes where the Work-N is
  focused on one or two files.
- **Merge non-ff** — `main` gains a merge commit
  (`X.Y.Z`); cycle commits stay reachable via two
  parents. `jj log -r ..@ -n <N>` shows the trapezoidal
  shape. Set up: `jj rebase -r <tip> --onto <prev>
  --onto <sub-tip>` where `<prev>` is the previous
  cycle's closed-out commit (current main tip). Two
  `--onto` → `<tip>` becomes a merge of `<prev>` +
  `<sub-tip>`.
- **Keep separate** — one commit per cycle entry on
  `main`. Use when the decomposition itself is
  informative. Each chores section keeps its own header /
  `Commits:` ref; no consolidation churn.

Set up squash/merge before invoking `vc-x1 push`; use
`jj git push` directly for non-standard shapes.

### vc-x1 push wrapper

`vc-x1 push <bookmark>` wraps per-push mechanics. See
`vc-x1 push --help` for current flags.

**Current limitation**: only fully supports the
[Keep separate](#shape-at-close-out-push) shape; other
shapes need manual jj steps. Improvements tracked /
planned:

- N:1 code↔bot for Merge non-ff (`## Todo` entry P1).
- Symmetric squash (planned, to be captured in `-2`).
- Per-repo bookmark names (planned).

### .claude cadence

**Cadence**: one push = one `.claude` commit, paired
with every code commit in that push.

The `.claude` working copy accumulates session data
across the cycle; its change ID stays stable across
snapshots, `jj describe`, and the finalize commit, so
code-side `ochid:` trailers resolve.

`.claude` is a linear journal — all session work lives
on `main`, regardless of the app-side bookmark. **Do not
create or maintain `.claude` bookmarks that mirror
app-side branches** — risks the bot steering session
pushes to the wrong remote ref.

### Bot communication at the reviews

Use plain prose — no insider jargon ("Gate N signal",
"Checkpoint N", etc.):

- **At Work review** — summarize what changed and stop.
  "Work complete. Please review."
- **At Commit Description review** — present `$TITLE`
  and `$BODY` explicitly; ask permission to commit/push.
  Don't spell out the full `vc-x1 push ... --title ...
  --body ...` invocation by default.
- **At Post close-out review** — surface the shape
  options (squash / merge / keep) and the push target;
  wait for the user's choice before any `jj squash` /
  `jj rebase` / `jj git push` invocation.

### After finalize: stop and wait

After `vc-x1 finalize` launches (mid-session per-push or
at session-end), you **MUST NEVER** proceed (next step,
edit, tool call, text output) until the user explicitly
directs you to continue. **Even when the next step seems
obvious — wait.** Treat finalize as a hard stop for the
whole turn; any final words go in the approval prompt
*before* executing finalize. Auto-proceeding bypasses
the push+finalize checkpoint the user controls.

### Recovery

- **If push exits before `finalize-claude`** (e.g.
  failure between `push-app` and `finalize-claude`), run
  finalize by hand:

  ```
  vc-x1 finalize --repo .claude --squash --push <bookmark> --delay 10 --detach --log /tmp/vc-x1-finalize.log
  ```

- **Clear push's saved state** after any out-of-band
  recovery — `rm .vc-x1/push-state.toml` or `vc-x1 push
  <bookmark> --restart` — otherwise push resumes from a
  stale stage.
- **Late code-repo tweak after `push-app` succeeded**
  (e.g. updating CLAUDE.md or memory) requires `jj
  squash --ignore-immutable` and a re-push; that is a
  remote rewrite and needs explicit approval like any
  push.

## ochid trailers

A **chid** is jj's change ID — a permanent identifier
that survives rebases and describes (see
[`jj-revsets.md`](jj-revsets.md)). An **ochid** trailer
on a commit body links it to its counterpart in the
other repo via that counterpart's chid, written as a
workspace-root-relative path. See
[`forks-multi-user.md`](forks-multi-user.md) for the
multi-line trailer design (multi-user / forking).

- **Code-side commits** each carry one
  `ochid: /.claude/<.claude-chid>` — the `.claude`
  change ID.
- **The `.claude` commit** has one `ochid: /<code-chid>`
  per code commit in that push. More than one occurs on
  Merge non-ff close-out (one ochid per Work commit in
  the cycle).

Use `vc-x1 chid -s code,bot -L` to capture the change
IDs (first line app, second `.claude`).

## Iterative work

When work for a single commit (the **target**) benefits
from incremental review, loop:

1. `jj new -R .` — fresh empty `@` on top of the target.
2. Make the next round of changes.
3. User reviews the round (see
   [Reviewing changes](#reviewing-changes)).
4. `jj squash -R .` folds into the target and creates a
   new empty `@`.
5. If not done, go to step 2.

Same jj mechanics as a
[sub-cycle ladder](#sub-cycle-ladders), but at
single-commit scope — the cycle's version suffix
doesn't change.

## Sub-cycle ladders

When a Work commit subdivides into a sub-cycle (see
[Numbering](#numbering) for suffix nesting), its Work
commits typically live as a local jj `@` chain and
**collapse into the sub-cycle's Close-out** before the
parent cycle continues. Ladder commits are scratch —
for review and bisection only.

### Per-Work-commit contract within a ladder

For each Work commit in the ladder:

1. `jj new -R .` — create a fresh empty `@`.
2. Do the commit's work.
3. Run `cargo test --bins`. **Non-negotiable** — build
   and clippy alone miss regressions until a later
   commit runs the full suite, raising bisection cost.
4. `jj describe -m "..." -m "..." -R .` — working title
   only (no version suffix); the sub-cycle Close-out
   collects everything into one final commit with the
   `(X.Y.Z-N)` marker.

### Navigating the ladder

Common moves:

- `jj log -r '<base>::' -R .` — see the whole ladder
  from its base.
- `jj edit -r <prefix> -R .` — jump `@` to any ladder
  commit by chid prefix; useful for bisection.
- `jj edit @-- -R .` — quick-jump back two commits.
- `jj diff -r <chid> -R .` — review one commit in
  isolation.

Modifications to any ladder commit rewrite it in place;
descendants auto-rebase.

### Close-out: squash the ladder

When all ladder Work commits are done and tests pass:

```
jj squash --from "<base>..@-" --into @ -u -R .
```

`<base>` is the parent of the first ladder commit; `-u`
keeps `@`'s description and discards the sources'.
After squash, history is linear: `<base> → @`;
intermediate commits are auto-abandoned.

Then `vc-x1 push <bookmark>` as for any other commit.

For N = 1 the squash is a no-op (`<base>..@-` is empty
when `@-` is `<base>`); push the single commit directly.

### Recovery

If a ladder commit goes wrong, back out without losing
prior commits:

- **Discard the current commit.** `jj abandon @ -R .`
  drops it; you get a fresh empty `@` on the same
  parent.
- **Edit an earlier commit.** `jj edit -r <chid> -R .`,
  make corrections, then `jj edit -r <last-ladder-chid>`
  to return. Descendants auto-rebase.
- **Discard the entire ladder.** `jj op log -R .` shows
  the op history; `jj op restore <op-id> -R .` reverts
  to that point. Full undo — removes *all* ladder work
  after the chosen op. Use only to start over.

# References

- [`jj-revsets.md`](jj-revsets.md) — revset primitives
  (chid/cid, `@`/`@-`/`@+`, `..`/`::` ranges, prefix matching).
- [`forks-multi-user.md`](forks-multi-user.md) — multi-ochid
  trailer design (fork / multi-user generalization).
- [`substep-test.sh`](substep-test.sh) — script that
  scaffolds a 4-revision ladder under `/tmp/substep-test`
  for squash-recipe experiments.
- 0.41.1-6.5 cycle — first multi-commit ladder usage. The
  per-commit `cargo test --bins` gate originated there
  after a regression introduced in an early commit wasn't
  caught until a later one ran the full suite.
