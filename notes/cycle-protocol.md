# Cycle protocol

This protocol uses [Prose form](../AGENTS.md#prose-form).

## Cycles

A cycle has three phases:

- **[Preparation](#preparation)** (`X.Y.Z-0`) — the cycle's
  first commit. Sets up the cycle:
  - Backfill the previous cycle's chores `Commits:` ref.
  - Bump `Cargo.toml` to `X.Y.Z-0`.
  - Pick up a `## Todo` item (typically the top-ranked,
    #1) into `## In Progress` (bold title + succinct problem
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
AGENTS.md [Chores conventions](../AGENTS.md#chores-conventions).

## Preparation

The cycle's first commit (`X.Y.Z-0`):

- **Backfill the previous cycle's chores section
  `Commits:` ref** — see
  [Chores commit references](../AGENTS.md#chores-commit-references).
- **Bump the version** in `Cargo.toml` to `X.Y.Z-0`.
- **Update `Cargo.lock`** to match — run `cargo build`
  (or `cargo check`) so the lockfile's `vc-x1` version
  tracks `Cargo.toml` in the same commit.
- **Move a `## Todo` item** (if the cycle has one) into
  `## In Progress` and the todo item should have:
  - A **bold title line** — that will be the chores
    section header, minus the `## ` prefix.
  - A **succinct problem statement**; add if one is needed
  - A **plan ladder**.
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
   the work repo, `-R .claude` for the bot repo (`-R` last
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

[Prose form](../AGENTS.md#prose-form) (intro + bullets),
wrap ≤72. Bullet content differs per repo:

- **Work-repo body**: file-by-file. One bullet per file
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

At close-out the cycle's *work* is done; its *published
shape* is the remaining choice, made at push time. Surface
the options and get user approval before pushing — once on
the target, changing shape is a remote rewrite (force-push,
needs approval), so choose deliberately.

- **Squash to one commit** — single entry on the target.
  Right for straightforward changes where the Work-N is
  focused on one or two files.
- **Merge non-ff** *(current default)* — `main` gains a
  merge commit (`X.Y.Z`); cycle commits stay reachable via
  two parents. `jj log -r ..@ -n <N>` shows the trapezoidal
  shape. See [Merge non-ff recipe](#merge-non-ff-recipe)
  for the full setup sequence.
- **Keep separate** — one commit per cycle entry on
  `main`. Use when the decomposition itself is
  informative. Each chores section keeps its own header /
  `Commits:` ref; no consolidation churn.

Set up squash/merge before invoking `vc-x1 push`; use
`jj git push` directly for non-standard shapes.

### Merge non-ff recipe

Setting up a [Merge non-ff](#shape-at-close-out-push)
close-out is a fixed sequence. `<closeout>` is the cycle's
close-out commit, `<prev>` the previous cycle's close-out
(the current `main` tip), `<work-tip>` the cycle's last
Work commit:

1. **Rebase the close-out into a merge** — `jj rebase -r
   <closeout> --onto <prev> --onto <work-tip>`.
   - `-r <closeout>` keeps the `<closeout>` commit in place.
   - `--onto <prev>` becomes its first parent (trunk).
   - `--onto <work-tip>` becomes the second parent.

   Together these make `<closeout>` a merge of
   `<prev>` + `<work-tip>`, forming a trapezoidal commit.
2. **Use `jj new <merge>`** to add an empty `@` above the
   merge. The rebase left `@` *on* the now-content-bearing
   merge, which git/IDE diff views show as uncommitted;
   `jj new` restores the clean empty `@` on top.
3. **Push** — `jj git push --bookmark main -R .`.

**Post-hoc caveat.** If the cycle was already pushed
[Keep separate](#shape-at-close-out-push) its commits are
immutable: the rebase needs `--ignore-immutable` and the
push force-updates `main`.
The standard sequence assumes the merge is set up *before*
the close-out push.

### vc-x1 push wrapper

`vc-x1 push <bookmark>` wraps per-push mechanics. See
`vc-x1 push --help` for current flags.

`vc-x1 push` injects the `ochid:` trailers itself (the
`commit-work` / `commit-bot` stages append them) — don't
hand-write them into the commit body or `--title`/`--body`.

**Current limitation**: only fully supports the
[Keep separate](#shape-at-close-out-push) shape; other
shapes need manual jj steps. Improvements tracked /
planned:

- Merge non-ff close-out without the manual pre-commits
  (`## Todo` entry "vc-x1 push: pause point between commit
  and publish stages") — commit stages run normally, pause
  for the merge rebase, resume via `--from bookmark-set`.
- N:1 code↔bot for code worked outside vc-x1 (`## Todo`
  entry "vc-x1 push: record uncovered code commits").
- Symmetric squash — demoted to `todo-backlog.md`
  ("vc-x1 push --squash"): after-publication squash is
  off the routine path now that Merge non-ff is the
  routine shape.

Landed: per-repo bookmark names (0.68.0) — `<bookmark>` is
work-repo-only; the bot repo is pinned to `main`
throughout push and sync.

### .claude cadence

**Cadence**: one push = one `.claude` commit, paired
with every code commit in that push.

The `.claude` working copy accumulates session data
across the cycle; its change ID stays stable across
snapshots, `jj describe`, and the squash-push fold, so
code-side `ochid:` trailers resolve.

`.claude` is a linear journal — all session work lives
on `main`, regardless of the work-side bookmark. **Do not
create or maintain `.claude` bookmarks that mirror
work-side branches** — risks the bot steering bot-repo
pushes to the wrong remote ref.

Ending a session: if the user runs `/exit` there will be
session information created, which we don't worry about.
The user can close the terminal instead and `@` will
remain empty — this behavior was verified in 0.64.0.

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

### After push or squash-push: stop and wait

After a **push** (crossing the remote boundary, by hand or
via the `vc-x1 push` wrapper — whose last stage publishes
the bot repo too) or a manual **squash-push** on the bot
repo, stop for the turn: no next step, edit, tool call, or
text output until the user directs otherwise. **Even when
the next step seems obvious — wait.**

- **Scope**: the stop follows the user's directive, not the
  push. A standing directive covering more work ("finish
  the remaining ladder commits on your own") makes an
  intermediate push just a step; the hard stop lands on the
  turn's *final* push.
- **Why**: the bot repo is a live journal — everything after
  the invocation (its own record, closing words) lands in
  `@` as a trailing tail. Between delegated pushes the tail
  rides into the next cycle's bot commit; the final push's
  tail has no next commit, and the bot's own squash-push is
  itself session data (`@` refills immediately), so only the
  user, after the turn, can capture it
  (`vc-x1 squash-push -R .claude`).
- **Silence**: put all closing words *before* the final
  push. There is no "harmless" closing line after it — a
  known slip.
- **Flush**: when the user wants `@` empty (no tail), they
  run `vc-x1 squash-push -R .claude` after the bot goes
  quiet — it flushes all bot session information into the
  published commit. Repeat if new writes land (see
  [Recovery](#recovery)).

### Recovery

- **If push exits before its last stage** — `push-work`
  succeeded but the bot-repo publish didn't run
  (`squash-push-bot` in `vc-x1 push --status` / `--from`
  stage names) — run the squash+push by hand:

  ```
  vc-x1 squash-push -R .claude
  ```

  It runs in-process, so a failure is a visible non-zero
  exit — no log file to chase.
- **Run squash-push again if `@` is non-empty** after a
  pass (also desirable after extra activity by the bot's
  agents).
  - Why: the bot keeps writing session data while the
    command runs — the invocation's own record plus any
    closing response land after the squash.
  - Safe to repeat: bot session data is append-only, so a
    re-run never conflicts or overwrites. (This could
    change; it is not under the user's control.)
  - No guarantees: events outside the bot's control can leave
    `@` non-empty — e.g. the bot's back end may decide to
    squash/consolidate session data, which can take minutes
    and land after the pass. The remedy is the same: just
    run squash-push again. This is why a single pass is never
    guaranteed to leave `@` empty.
- **Clear push's saved state** after any out-of-band
  recovery — `rm .vc-x1/push-state.toml` or `vc-x1 push
  <bookmark> --restart` — otherwise push resumes from a
  stale stage. A pre-0.69.0-4 state file may name a retired
  stage and also needs `--restart`.
- **Late work-repo tweak after `push-work` succeeded**
  (e.g. updating AGENTS.md or memory) requires `jj
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

Path semantics — paths start with `/`, the workspace root
(the work repo); `/.claude` is the bot sub-repo:

- `ochid: /<chid>` references a change in the **work repo**.
- `ochid: /.claude/<chid>` references a change in the
  **`.claude` repo**.

Trailers are blank-line-separated `key: value` lines at the
end of the commit body, using the chid's **12-character**
prefix:

```
ochid: /.claude/xvzvruqowktp   # points to a .claude change
ochid: /wtpmottvxqzl           # points to a work-repo change
```

How many, and which direction:

- **Code-side commits** each carry one
  `ochid: /.claude/<.claude-chid>` — the `.claude`
  change ID.
- **The `.claude` commit** has one `ochid: /<code-chid>`
  per code commit in that push. More than one occurs on
  Merge non-ff close-out (one ochid per Work commit in
  the cycle).

Use `vc-x1 chid -s code,bot -L` to capture the change
IDs (first line work repo, second `.claude`).

### Resolvability

A change ID travels with its commit: a **pushed** commit
resolves to the same chid in every clone — cloning the
`.claude` repo gave the published `main` tip the same chid
as an existing clone. The bot thinks jj carries the change
ID in the git commit object, so it survives
`jj git clone` / fetch.

The local-only case is the **working-copy `@`**: jj mints a
fresh random chid for `@` in each clone, so an unpushed `@`
is never a stable ochid target. This is why a `.claude`
ochid names `@-` (the last committed change), not `@`.

### .vc-config.toml

Each repo contains a `.vc-config.toml` that identifies its
location within the workspace, so tools resolve these paths
without repeating the workspace path in every trailer:

```toml
# In vc-x1 (workspace root):
[workspace]
path = "/"

# In .claude (sub-repo):
[workspace]
path = "/.claude"
```

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
