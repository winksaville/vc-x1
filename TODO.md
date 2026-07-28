# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

When a `## Todo` item is picked up, its text **moves** here
(never copied — one home per text). The picked-up task is a
`###` heading; a multi-cycle program adds one level — the
program is the `###` and its current stage a `####` (headings
give the current work durable anchors, which numbered Todo
entries can't). The problem overview is followed by the
"plan" — a bulleted list of the development "ladder". Each
rung is prepended with its commit reference — a literal
`[[N]]` placeholder until the commit is pushed, then
backfilled to a real file-local `[[n]]` ref (same pattern as
the chores As-built rungs):
   - [[N]] 0.xx.y-0 blah (done)
   - [[N]] 0.xx.y-1 blah blah (current)
   - [[N]] 0.xx.y-2 blah blah blah
   - [[N]] 0.xx.y close-out and validation

### Refactor: typed jj facade → jj-lib in-process; end subprocess spawning

Version-control operations were ~30 hand-rolled `run("jj", …)`
spawns plus every mutation, with per-module private wrappers
and raw-git vestiges in init — stderr parsing instead of typed
errors, and jj's single-attempt index-lock acquisition (the
push `bookmark-set` lock race in [bugs.md](notes/bugs.md))
can't be retried where it fails. A multi-ladder program; the
staged plan, design detail, and the eight absorbed former
Todos live in
[refactor-20260716.md](notes/refactor-20260716.md).

Program ladder — one rung per cycle (adjacent stages
consolidated 2026-07-24; rung titles are the anticipated
close-out commit titles; unshipped versions are provisional —
jj-lib may split into two cycles). Shipped-rung refs point at
close-out commits on `refactor-vc-x1`, treated as permanent:
the branch is long-lived and lands on main merge-only, never
rebased.

- [[1]] 0.73.0 refactor: DRY jj facade (done)
- [[2]] 0.74.0 refactor: hygiene riders (done)
- [[3]] 0.75.0 refactor: facade owns topology (done)
- [[9]] 0.76.0 refactor: repo registry (done) — first
  trapezoid published by the four-step recipe
- [[N]] 0.77.0 refactor: stateless push (current, below)
- [[N]] 0.78.0 refactor: jj-lib migration (Todo #1)
- [[N]] 0.79.0 refactor: trapezoid-push + body-intro
  validation (Todo #2)

#### refactor: stateless push

Picked up 2026-07-28. `push.rs` (~1.5k lines) holds the
`Stage` machine, TOML state persistence, eight stage bodies,
two sanity verifiers, and the interactive gates in one file.
The state file is where the defects come from: bugs.md #3 —
the rollback rewinds the *repos* but not the *state*, so the
rerun skipped the commit stages and republished a previous bot
commit — and both sanity verifiers exist largely to defend
against that staleness. Retiring it, deriving the resume point
from repo reality as standalone `squash-push` already does,
deletes the class rather than patching it. Fifth cycle of the
refactor program; see
[split push.rs](notes/refactor-20260716.md#stage-split-pushrs)
and
[stateless push](notes/refactor-20260716.md#stage-stateless-push).

Working ladder (greppable stem `push`) — detail per rung
below:

- [[6]] 0.77.0-0 chore: open stateless push cycle (done)
  [detail](#0770-0-chore-open-stateless-push-cycle)
- [[7]] 0.77.0-1 refactor: extract push/state.rs (done)
  [detail](#0770-1-refactor-extract-pushstaters)
- [[8]] 0.77.0-2 fix: push skips an empty work commit (done)
  [detail](#0770-2-fix-push-skips-an-empty-work-commit)
- [[N]] 0.77.0-3 refactor: drop push state and preflight
  (current)
  [detail](#0770-3-refactor-drop-push-state-and-preflight)
- [[N]] 0.77.0 refactor: stateless push — close-out,
  published by the four-step recipe with its new step 4

This block is the cycle's single working record — narrative
included — and moves to `notes/chores/chores-15.md` wholesale
at close-out. See Todo "One home for a cycle's narrative"; the
0.77.0 cycle is the trial run, adopted at -2 (the rungs
already written were migrated back here, so chores-15 holds
no section for this cycle until close-out).

##### Decisions at cycle open

- **Trapezoid support was folded in and unfolded the same
  day.** The fold rested on rung -4 deleting `--from`, which
  the manual trapezoid recipe's step 4 uses — but that rung's
  own docs rider makes step 4 a bare `vc-x1 push <bookmark>`,
  since after the reshape the repos are exactly what
  reality-derived resume recognizes. Nothing was stranded, so
  the fold bought nothing and cost a seven-rung cycle.
  Trapezoid support returns to 0.79.0.
- **Program order stays as planned** (stateless push → jj-lib
  → trapezoid). A swap was considered — jj-lib first, since
  the index-lock race keeps firing — and rejected: the
  "we'd write the stage bodies twice" argument doesn't hold
  up. Resume detection is *reads*, which already go through
  the `src/jj.rs` facade; jj-lib rewrites facade internals
  and the hand-rolled mutations, not call sites that already
  went through it. The overlap is thin. (Worth confirming
  against the stage bodies when rung -3 opens.)
- **Doing this first shortens the exposure to bugs.md #1**
  even though it can't fix it. The race can only be retried
  once jj-lib owns the lock acquisition, but rung -4 fixes
  bugs.md #3 — so a rollback stops leaving a poisoned state
  file and a plain rerun becomes safe rather than dangerous.
  The -0 push hit exactly this: the race fired, both repos
  rolled back cleanly, and the state file still said
  `stage=bookmark-set`.
- Ordering within the cycle: the empty-`@` fix (bugs.md #4)
  lands early, at -2, so every later rung's dogfood push is
  protected by it.

##### 0.77.0-0 chore: open stateless push cycle

- version 0.77.0-0; the stage picked into `## In Progress`
  as the program ladder's current `####` rung with a
  five-rung ladder
- rider: 0.76.1 `Commits:` backfill
- rider: `## Done` retirement sweep into done.md
- the push itself hit bugs.md #1 twice before landing —
  recorded there as the fourth and fifth occurrences, and as
  #3's second occurrence (clean rollback, poisoned state
  file, `--restart` the safe rerun)

##### 0.77.0-1 refactor: extract push/state.rs

- `Stage`, `StateLayout` / `resolve_state_layout`,
  `PushState`, `STATE_FORMAT_VERSION`, the state-dir /
  state-file defaults, and the escape helpers move to
  `src/push/state.rs`; push.rs 1480 → 1101 lines with no
  behavior change
- the parked 0.72.0-1 extraction was **reference, not
  base**: `support-trapezoid-commits` turns out to be
  published (`@origin`), so rebasing it would rewrite a
  pushed commit. Its boundary was reused (the same item set,
  plus `STATE_FORMAT_VERSION` which it left behind at
  version 1) and the extraction redone against the current
  file. Deleting that bookmark — a remote branch — is still
  outstanding
- `escape_multiline` / `unescape_multiline` stay private, so
  their round-trip test moves into `state.rs`'s own
  `#[cfg(test)] mod tests` rather than widening visibility
  for a test's benefit; the remaining state tests reach
  `STATE_FORMAT_VERSION` through push.rs's `#[cfg(test)]`
  re-export beside `DEFAULT_STATE_DIR` / `_FILE` (which
  `config_schema`'s tests already used)
- the module doc records what the stateless-push rungs
  delete, so the next reader knows the file is scaffolding
  with a scheduled end

##### 0.77.0-2 fix: push skips an empty work commit

- bugs.md #4 fixed: `commit-work` skips an empty `@` the way
  `commit-bot` always has, and `stage_message` resolves the
  work chid from `@-` when `@` is empty — the unconditional
  `@` was what made the bot's trailer name the duplicate
  instead of the real commit
- **skip, not error.** The bug report offered erroring
  loudly as the alternative, but an empty work `@` is
  legitimate in the publish-only case: commits already made,
  only the bookmark and the remote left to advance — the
  trapezoid recipe's final step, and the shape rung -3
  teaches push to recognize on its own. Erroring would break
  the flow rung -4 depends on
- push does not rewrite a description it didn't author, so a
  hand-made commit keeps its message and simply carries no
  work-side ochid; `validate-desc` / `fix-desc` are the tools
  for adding one. The skip warns rather than informs, because
  a supplied `--title`/`--body` silently going unused is
  worth noticing
- `push_empty_work_at_skips_commit_work` reproduces the bug's
  exact scenario and was confirmed to fail without the fix
  (the empty duplicate's title lands on the bookmark) before
  being kept
- rider: the one-home trial starts here — chores-15's
  in-flight section (intro, decisions, As-built rungs) moved
  back into this block and its two now-orphaned commit refs
  were pruned there, since TODO.md already carries them

##### 0.77.0-3 refactor: drop push state and preflight

Rungs -3 and -4 collapsed into one after the 2026-07-28
design conversation went further than the ladder assumed. A
first attempt at -3 derived the resume point from the repos
instead of the state file; that work is superseded here and
was never committed.

The premise: **resume is not recoverable in general.** A
recorded position describes a world we may no longer
understand, and we cannot know why a run failed. If something
goes wrong, push stops and the user and bot put the repos
right — that is not vc-x1's job and cannot be. What vc-x1
owes them is that *rerunning is always safe*, which is a
property of each stage, not of a record.

- **State file gone**, with `PushState`, `StateLayout` /
  `resolve_state_layout`, `STATE_FORMAT_VERSION`, the escape
  helpers, `src/push/state.rs`, the `[push]` `state-dir` /
  `state-file` config keys, and the `.gitignore` coherence
  check that existed only to keep the file out of commits.
  `--restart` / `--from` / `--status` go with it — all three
  are resume machinery.
- **Per-stage guards instead of a start point.** Each stage
  checks its own precondition and no-ops when its work is
  already done, so a rerun after any failure is just a run.
  `commit-work` already skips an empty `@` (-2);
  `bookmark-set` is a set; `push-work` needs no guard of its
  own — jj's push reports nothing to do when the bookmark is
  already published — and instead picks up preflight's
  tracking check, which is its real precondition;
  `squash-push-bot` skips an empty `@`; `message` doesn't
  demand a title when neither side will commit. This
  subsumes the derived-start-point idea: nothing needs to
  decide *where* to begin.
- **Preflight dropped entirely.** Its three checks were
  vc-x1's own preconditions, not project checks (the Rust
  cargo cycle was always outside vc-x1). Bookmark tracking
  moves to `push-work`, which is what needs it; `sync
  --check` goes — it was the expensive one, it re-invoked
  `current_exe()` as a subprocess, and that self-spawn is
  why the integration tests needed `--from message` to skip
  the stage at all. Dropping it takes the workaround with
  it.
- **The bot-published invariant goes too**, after being
  argued down twice: jj's op log means local content isn't
  lost, and a bot repo that hasn't been squash-pushed yet is
  not a broken state — it's an unfinished errand with no
  deadline, and it self-heals on the next push. What we
  accept is a window where a published work trailer names a
  bot commit no fresh clone can resolve; that only matters
  to a third party, and that world is
  [forks-multi-user](notes/forks-multi-user.md)'s to solve.
- **The in-process `jj op` rollback stays.** Persisted state
  is a guess; a snapshot taken moments earlier in the same
  process is not. It is what makes a failure before the
  remote boundary cost nothing — both of the -0 push's
  index-lock failures rolled back clean.
- `squash-push` stays repo-agnostic. It knows there is a
  repo and a bookmark, not whether it is the bot side, and
  it should not consult `.vc-config.toml` to find out. Its
  exists-and-tracked check stays, for a reason unrelated to
  topology: `jj bookmark set` creates rather than errors and
  `jj git push` publishes a new bookmark without ceremony
  (confirmed at 0.76.0-5), so a typo'd name would create a
  branch on origin instead of failing.

## Todo

 Entries are in **strict priority rank** — #1 highest,
 descending. Reprioritize by moving an entry, then
 `vc-x1 fix-todo --no-dry-run TODO.md` to renumber.
 The numbers are positional rank, not stable IDs — to refer
 to a Todo, name it by its **title** (a greppable mention;
 a numbered list item has no anchor to link to), not its
 number. Long-tail entries
 live in [todo-backlog.md](notes/todo-backlog.md). Use the
 [Prose Form in AGENTS.md](/AGENTS.md#prose-form); deeper
 detail goes in `notes/chores/chores-NN.md` design
 subsections (link via `[N]` ref).

1. **refactor: jj-lib migration.** Facade internals and
   mutations move in-process; the index-lock retry becomes
   ours; see
   [the stage](notes/refactor-20260716.md#stage-jj-lib-migration).
   After split push.rs + stateless push. May split into two
   cycles (reads lift, then mutations lift) once the
   op-store-coexistence risk is spiked.
2. **refactor: trapezoid-push + body-intro validation.**
   `vc-x1 trapezoid-push` — a **subcommand**, not a flag on
   `push` (decided 2026-07-28) — publishes a close-out as a
   non-fast-forward merge; body-intro validation rides as
   the first rung. See
   [trapezoid close-out](notes/refactor-20260716.md#stage-trapezoid-close-out)
   and
   [push body-intro validation](notes/refactor-20260716.md#stage-push-body-intro-validation).
   After jj-lib, so the reshape is built in-process.
   - `push` keeps a stateable invariant: it never produces
     a merge. A mode flag that rewires the stage sequence
     would cost that.
   - Shared implementation, not a second copy: the common
     pipeline (preflight, both gates, message, commit-work,
     commit-bot, bookmark-set, push-work, bot squash) moves
     into its own module that both subcommands call, with
     the reshape as the one inserted step. The
     stateless-push cycle shrinks that pipeline first,
     which is what makes the extraction cheap.
   - A backend `trait` (jj today, git or another VCS later)
     is the natural next abstraction if a second backend
     ever appears — worth converting these concepts to
     traits then, not now: we are committed to jj, and a
     one-implementation trait buys nothing but indirection.
3. **One home for a cycle's narrative: TODO during, chores
   at close-out.** Today the ladder and its detail are
   maintained in both `TODO.md > ## In Progress` and the
   chores `### As-built ladder` while a cycle runs, so every
   rung is written twice and every `Commits:` backfill lands
   in two files. Instead keep it all in `TODO.md` — a
   succinct working ladder with the detail in sections
   beneath it, linked locally — and at close-out move the
   whole block into `notes/chores/chores-NN.md` as the
   durable record.
   - Preserves what the per-commit convention was protecting:
     the narrative is still written while the work is fresh,
     just in one place.
   - Removes the dual backfill — commit refs are filled once,
     in the working ladder, and travel with it.
   - Watch first, decide after: the remaining rungs of the
     0.77.0 cycle are the sample. If close-out migration
     turns out to lose detail that per-commit capture kept,
     that is the argument against.
   - Touches AGENTS.md [Chores conventions] and
     cycle-protocol.md's Chores sections + Close-out, both
     shared with the template family, so it fans out.
4. **Remove `revert` — and `.vc-x1/` with it.** `revert`
   promises "undo the sync"; it restores the pre-sync `jj op`
   recorded in `.vc-x1/sync-state.toml`, which means "rewind
   the repo to that moment". The two coincide only while
   nothing has happened since — one commit later, revert
   would silently rewind that too, and nothing readable at
   revert time distinguishes the cases. We are not in control
   enough to do this reliably; jj's own `jj op log` /
   `jj op undo` is both safer and more informative, since it
   shows what is being undone before committing to it.
   - Confirm what revert actually restores (both repos?
     bookmarks only? full op state?) before deleting —
     `src/revert.rs`, `src/sync/state.rs`.
   - Delete the subcommand, sync's `sync-state.toml` write,
     and the docs/help text that describe them
     (`README.md`, `src/main.rs` help strings).
   - `.vc-x1/` then empties — push's `push-state.toml` is
     retired by the stateless-push cycle — so the directory,
     `init`'s `/.vc-x1` `.gitignore` line, and any leftover
     `[push]` state config keys go too.
   - Existing workspaces: **never edit their `.gitignore`
     automatically.** Inspect it, and when the `/.vc-x1` line
     is found, report that it is no longer needed and leave
     the removal to the user — a report, not a rewrite. It is
     the user's file, and a stale ignore line is harmless.
     *When* the check runs — which surface, and how often —
     is TBD; `config --validate` and the proposed
     `validate-repo` are the candidates, and push's
     `check_gitignore_coherence` is not (it retires with the
     state file).
   - Cheap now, expensive later: few workspaces depend on it
     today.
5. **`squash-push --title` / `--body`.** `squash-push` amends
   content only — it folds the working copy into the last
   commit and force-updates the remote, but the commit keeps
   its existing message. Fixing a published commit's *message*
   is therefore two steps (`jj describe -r @-`, then
   `squash-push`). Accepting `--title` / `--body` makes it
   one.
   - No new risk: squash-push already rewrites a published
     commit and force-updates the remote. This only changes
     which part of the commit it edits.
   - **ochid handling — tell, don't force.** A user-supplied
     body drops the `ochid:` trailer unless it repeats it,
     which silently breaks the cross-repo link. vc-x1 should
     *not* inject the trailer (unlike `push`, which authors
     the message and stamps it; here the user authors it and
     the tool shouldn't rewrite their text). It should error
     when the new message loses a trailer the commit had,
     naming what would be lost — with an explicit override
     flag for the case where dropping it is intended.
   - The content-side guard is the precedent: squash-push
     already refuses a squash that would drop source-only
     trailers (the 0.65.1 ochid-loss incident). Same check,
     new input.
6. **Restructure templates: single template repo + fixed bot
   seed manifest.** Replace the separate
   `vc-x1-work-repo-template` + `vc-x1-bot-repo-template`
   repos with the one work-repo template, whose live
   `.claude/` doubles as the bot-side seed source; retire
   `vc-x1-bot-repo-template`. `vc-x1 init` / `clone` updates
   for the new layout. First up after the refactor program.
   - `--use-template` rule: explicit `CODE,BOT` copies all
     non-hidden files from BOT (unchanged — the escape
     hatch for rich bot seeds); `CODE` alone seeds the bot
     side from a fixed manifest — `LICENSE-*`, `README.md`
     — taken from `<CODE>/.claude/`. The `<CODE>.claude`
     sibling default is dropped.
   - The manifest is the safety property: a live `.claude`
     has non-hidden session artifacts at top level, and
     the known subset is what lets it double as the seed
     source without leaking session history into new
     projects.
   - Manifest members missing in the source are skipped —
     a code template with no `.claude/` content yields a
     bare-but-valid bot repo (the bot template is
     optional; init already generates the true minimum
     itself).
   - `memory/MEMORY.md` moves from copied to generated:
     it is intentionally empty (seeded only because Claude
     tends to create it otherwise), so init emits it like
     `.vc-config.toml` instead of copying — no "is it
     still empty?" invariant left in the template.
7. **ochid: bot-repo location qualifier.** An ochid is
   workspace-relative (`/.claude/<chid>`) — nothing in a
   published commit says *where* the companion bot repo
   lives (vc-x1's is `github.com/winksaville/vc-x1.claude`,
   discoverable only by convention). Anyone cloning just the
   work repo can't resolve bot-side ochids. Design already
   sketched in forks-multi-user.md
   [Per-user bot repos via URL-shaped ochid](notes/forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid):
   URL-shaped trailers, plus the complementary
   `.vc-config.toml` repo-index form; resolver dispatch is
   one rule (URL → fetch, else workspace-relative), existing
   path-form trailers stay the backward-compatible case.
   - Cheap first rung: declare the companion's URL once in
     the committed `.vc-config.toml` (no trailer-format
     change; any work-repo clone then knows where the bot
     repo lives). Rides naturally with the refactor
     program's facade-owns-topology stage
     (bot-repo-location config).
   - Link rot + mirroring mitigations are in the same doc
     section.
8. **Version-number protocol is fragile — versions are
   baked into titles/bodies/todo/done/chores before the
   change lands.** The cycle protocol embeds an `X.Y.Z-N`
   version in commit titles and bodies, `## Todo` /
   `## Done` entries, and chores headers — all written
   while the work is in progress, i.e. before it lands.
   But version numbers are subject to change: in a public,
   merge-based flow (e.g. Linux), the version a change
   ships under is only fixed when it merges into `main`,
   so the landing version can't be anticipated while the
   work is underway. Pervasive version-in-text is
   therefore fragile for any non-linear / multi-contributor
   workflow. Promoted from Ideas at 0.65.2-0; slated for
   the cycle after 0.65.2.
   - Live in-repo example (2026-07-24): 0.72.0 was
     pre-assigned to the trapezoid close-out cycle, which
     paused on `support-trapezoid-commits` after `-1`; the
     refactor program then ran 0.73.0+ directly off the
     0.71.0 main tip, leaving 0.72.0 a permanent gap —
     renumbering either branch would rewrite cross-linked
     history. Disposition recorded in the
     [split push.rs stage](notes/refactor-20260716.md#stage-split-pushrs).
   - Related numbering thought (2026-07-24): program-shaped
     work could claim one minor and number its cycles
     `X.Y.1..n` (the jj refactor's seven cycles would have
     been 0.73.1..0.73.7) — program membership encoded in
     the version. Trade-off: a per-prep "is this a program?"
     call vs today's decision-free minor-per-cycle.
   - Open question: what identifies a cycle's commits if
     not a pre-assigned version?
     - Needs to be unique within some agreed upon domain.
       A contributors email address would do it, but also
       a UUID (short-version) for a contribution. I could
       imagine a UUID generated from the initial email/issue
       that and then "version number" schema appended to that.
   - Surfaces to update once the identifier is chosen:
     cycle-protocol.md (title shape, Numbering), AGENTS.md
     (commit-recording headers), and the `vc-x1` validators
     that parse `(X.Y.Z)` strings.
9. **sync follow-up: extract `move-bookmark` command.** The
   "put the bookmark / `@` where it belongs" step at the end
   of sync (reposition logic) is useful standalone — e.g. the
   t1B scenario where `main` is right but `@` isn't on it —
   and deserves an honestly-named command instead of a mode.
   - `vc-x1 move-bookmark` (name open): no fetch; move `@`
     (and optionally the bookmark) onto a target under the
     same safety rules as sync's reposition step.
   - Sync's final step becomes a call to the same logic.
   - Follow-up to the 0.67.0 single-mode sync cycle.
10. **sync follow-up: retire the hidden `--check` alias;
    revisit push's auto-rollback.** The first half of this
    entry — push shelling out to `vc-x1 sync --check`, which
    was racy and not actually read-only — is done: 0.77.0-3
    deleted preflight outright, taking the shell-out and its
    PATH dependency with it. What survives:
    - Remove sync's deprecated hidden `--check` alias. Nothing
      invokes it now except `tests/cli_sync.rs`'s alias test,
      so this became actionable the moment preflight went.
    - Push's commit-stage rollback auto-runs `jj op restore`,
      which hides the evidence of what failed. This cycle
      deliberately kept it — an in-process snapshot taken
      moments earlier is knowledge, not a guess, and both
      index-lock failures during 0.77.0 cost nothing because
      of it. Revisit only with a concrete case where the
      hidden evidence mattered.
11. **validate-numbering: rename the pair, check all
    sequence-managed notes files generically.** `validate-todo`
    / `fix-todo` only operate on the single file passed, so a
    renumber slip in `bugs.md`, `todo-backlog.md`, or
    `TODO.md`'s `## Ideas` section passes unnoticed — too weak
    for a pre-commit gate. Prereq for the pre-commit doc
    validators (Todo "pre-commit: single rule ...").
    - Rename the pair: `validate-todo` → `validate-numbering`,
      `fix-todo` → `fix-numbering` — they validate numbered-
      sequence integrity, not todos specifically.
    - Generic detection: for every `#…#` section, validate the
      column-0 `^\d+\.␠` entries form a contiguous 1..N run.
      Drops the Todo/Bugs special-casing; auto-covers
      `## Ideas` and any new numbered section. Keep the
      column-0 anchor so indented sub-lists aren't counted.
    - Default scope: a fixed list of sequence-managed notes
      files (`TODO.md`, `todo-backlog.md`, `bugs.md`) so the
      no-arg pre-commit run covers them all. Fixed rather than
      a `notes/**.md` walk because prose docs
      (`cycle-protocol.md`, design notes) carry ordinary
      numbered lists that aren't managed sequences — a walk
      would false-positive (markdown renders `1. 1. 1.` as
      1-2-3, a legitimate prose pattern).
    - Override args follow the `--init-from` convention:
      positional files/dirs (a dir → its `*.md`) plus an
      `@<file>` manifest, additive — for ad-hoc validation of
      a specific file.
    - Add wrapper-level tests while restructuring: the analyze
      cores are covered (`todo_helpers` 15 tests,
      `desc_helpers` 22) but the `validate-todo` / `fix-todo` /
      `validate-desc` / `fix-desc` wrappers have none — file
      I/O, output formatting, exit codes, and the no-arg
      default path (changed to `TODO.md` at 0.69.2-2) are
      unexercised.
    - Open: revisit fixed-vs-glob at implementation if the
      fixed list proves annoying to maintain.
12. **pre-commit: single rule (no docs skip) + doc validators.**
    The pre-commit (cargo cycle: fmt/clippy/test/install) only
    checks code, so it's "skip-able for purely-docs commits" —
    but that exception is exactly where checks slip (skipped on
    0.62.0-7/-8 until caught). (Since 0.69.0-3 push's
    `preflight` no longer re-runs the cargo cycle — vc-x1
    assumes nothing about repo contents — so the pre-commit is
    the *only* gate, strengthening the no-skip case.)
    - Adopt one rule, no exception: the pre-commit runs before
      Work review on every commit. (docs: AGENTS.md Cycle
      Protocol summary + cycle-protocol.md per-commit-flow.)
    - Enrich the pre-commit so it's meaningful on docs commits:
      add the doc validators — `validate-numbering` (its own
      Todo, a prereq) plus `validate-repo` when it exists.
      Whether push's `preflight` may run them needs a decision
      against the content-agnostic principle (they read
      `notes/` — repo content; the repo-declared-checks idea
      was rejected 2026-07-15 in favor of "run checks
      yourself").
    - This dissolves the docs exception: with doc validators in
      the pre-commit there's always something to validate, so
      the carve-out stops making sense.
    - Its own near-term cycle (chosen over a 0.61.1 insert to
      avoid rewriting published 0.62.0-x history); no version
      pre-assigned — see the Todo "Version-number protocol is
      fragile" on fragile version targets.
13. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
    Today push assumes 1:1 symmetric WC commits with shared
    title/body. The interop / adoption scenario breaks that:
    the code side is worked single-repo style (commit +
    `jj git push` / `git push`, no `vc-x1 push` in the loop),
    so no bot pairings exist — one bot commit then records
    every code commit not yet covered by a prior `ochid:`,
    via a multi-line `ochid:` per the design in [[4]].
    - Out of scope: the trapezoid close-out — handled
      natively by the in-progress "feat: push merge
      close-out (trapezoid)" cycle, whose N-ochid stamping
      also covers a cycle held local and published all at
      once. This Todo is only the no-bot-pairings interop
      case; the stamping step's multi-line `ochid:` emit is
      shared groundwork.
    - Teach push to:
      - detect the shape (code WC empty, uncovered commits at
        the bookmark)
      - skip `commit-app`
      - compose a `.claude`-specific message
      - emit one `ochid:` line per uncovered commit
    - Open: computing "uncovered" — likely a revset from the
      code bookmark back to the newest commit referenced by
      the bot journal's ochids.
14. **Run validate-bot at every vc-x1 invocation
    (config-gated).** The check is one jj spawn
    (`jj bookmark list main --all-remotes`), cheap enough
    to run at every execution — noted 2026-07-15 as a
    "could, not should". Design points:
    - locate the bot repo (`<cwd>/.claude` or config;
      shares the lookup with the refactor program's
      [facade-owns-topology stage](notes/refactor-20260716.md#stage-facade-owns-topology))
      and silently skip when absent
    - severity knob in `.vc-config.toml`
      (`warn|error|off`): unrelated commands (fix-todo)
      warn at most; push / squash-push / validate-bot
      already have their own handling from 0.69.0-3
15. **CLI reference lives in `--help`; README owns concepts.**
    Each command is described in three places — clap's
    `long_about`, a README section with a flag table, and
    sometimes AGENTS.md — and only the flag *descriptions*
    self-sync, because those come from the field doc
    comments. Every hand-written block drifts silently:
    0.69.0-4 found the init section documenting retired
    `--owner` / `--dir` / `--repo-local`, and 0.77.0-3 found
    push's `long_about` still advertising a state machine
    that had just been deleted. The fix is removing the
    duplication, not auditing it on a schedule.
    - `--help` becomes the reference: what a command does,
      its stages, its flags, its invariants. It ships with
      the binary, so it always matches the binary being run.
    - README keeps workflows and concepts — the dual-repo
      model, the cycle, testing recipes, worked examples —
      and points at `--help` instead of restating flag
      tables. Delete the tables; that is the drift source.
    - Clap reflows prose and collapses bullets unless a
      field carries `verbatim_doc_comment`, so help owns the
      reference, not the explanations. `long_about` does
      preserve explicit newlines (0.77.0-3's push stage
      list renders as an aligned two-column list).
    - Optional enforcement, cheapest first: assert README
      has no flag-table rows; snapshot-test `--help` output
      so unintended changes surface in review; or generate
      the reference from clap and assert the committed file
      matches. The third rhymes with "config: extract
      flag-backed key descriptions from Clap" — same
      single-sourcing shape.
    - Sweep each section against `vc-x1 <cmd> -h`.
    - Consider regenerating transcripts via support
      scripts (the gen-exmpl pattern) so examples stay
      reproducible.
16. **Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs.** Adopted in chores-13 (0.69.2 ladder,
    backfilled during 0.70.0-0): each rung is prepended
    with its commit reference so the rung↔commit
    correlation is direct; `Commits:` stays as the
    section-level list. The convention's home —
    cycle-protocol.md Close-out ("Add an `### As-built
    ladder`…") — is in the byte-identical shared doc set
    (family: vc-x1, vc-x1-work-repo-template, iiac-perf, zc-msg-x1,
    tprobe), so the doc edit needs a coordinated family-wide sync, not a
    mid-cycle local change. Not included in the 2026-07-20
    vc-x1-work-repo-template sync (straight copy); still pending for the
    whole family, vc-x1 included.
17. **Shared-doc sync: per-commit chores convention.**
    0.71.0 changed how chores are recorded — each work commit
    appends its As-built rung + narrative as it lands, rather
    than the narrative waiting for close-out. That wording edit
    was made locally in vc-x1's `cycle-protocol.md` / `AGENTS.md` (the
    byte-identical shared doc set). vc-x1-work-repo-template synced
    2026-07-20 (AGENTS.md + cycle-protocol.md byte-identical again, plus
    the TODO.md move); iiac-perf, zc-msg-x1, and tprobe still diverge —
    the plan is to fan out from vc-x1-work-repo-template (same family as
    the Todo "Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs").
18. **config: extract flag-backed key descriptions from Clap.**
    `config`'s key descriptions live in `config_schema.rs`
    (`doc`/`used_by`). For the handful of keys that map 1:1 to a
    CLI flag (`bot-session.col-width` ↔ `--col-width`,
    `--result-lines`), the description could instead be pulled
    from the Clap arg's help via `Cli::command()` introspection,
    so `vc-x1 config` and `--help` share one source and can't
    disagree.
    - Only ~2 keys map cleanly (most are config-only, flag-sets,
      or value-providers), so it's a partial source — the schema
      stays authoritative for the rest.
    - Defaults still come from the schema/consts (the args
      dropped `default_value_t`, so Clap no longer holds them).
    - Output format is unchanged, only the text source — no
      rework of the 0.71.0-9 rendering.

## Ideas

 Items not yet solid enough for `## Todo` (or surfaced
 during close-out / end-of-day before they're fully
 formed). Triaged at the next Preparation: promote to
 `## Todo` / `notes/todo-backlog.md`, fold into a
 picked-up cycle, or drop.

1. **`vc` as a code+conversation provenance tool (grander
   ambition).** Today `vc-x1` manages a dual repo (code +
   `.claude`) cross-linked by `ochid:`. The larger aim is
   to *surface* that link — view history with the
   conversation and the code side by side: provenance, the
   *why* of a change, not just the *what*. The dual-repo +
   `ochid` design is already the substrate; the cross-links
   make code↔conversation navigable, so the viewer is UI
   over an already-solved data link.
   - Build direction: keep resolution/assembly in `vc` — an
     editor-agnostic Rust engine/lib extending the
     `show` / `chid` / `desc` family ("given a commit,
     resolve its ochid and assemble the paired diff +
     conversation slice"); the editor add-on is a thin
     presentation layer over it.
   - Front-end leans a Zed add-on (Rust, preferred), maybe
     VSCode / other. Verify Zed's extension API can host a
     rich side-by-side panel before committing — an
     editor-agnostic core hedges the bet.
   - `vc-x2`? A rewrite is unwarranted: the audit's
     Commonality pass found the architecture sound (por is
     bolted on where an existing good pattern wasn't
     applied) — equalize incrementally. "vc-x2" only makes
     sense if the viewer changes the *core* architecture
     (an index / daemon / data model). Separate
     engine-rewrite (no) from product-reposition (open).
   - Possible artifact: a top-level
     `notes/design-cli/vision.md` framing the direction,
     with the parity and conversion docs as sub-designs.
2. **Restructure the design-cli parity docs (target
   0.63.0).** `por-dual-parity-audit.md` (~1200 lines)
   fuses a *frozen* audit (the `## 1`–`## 8` snapshot
   evidence) with a *living* design (axes, decisions,
   matrix, gap list); the "audit" name undersells it and
   the halves have different lifecycles. And
   `por-dual-parity.md` (the stub) overlaps on parity but
   uniquely holds the `por ↔ dual` conversion design.
   - Split the audit doc into a frozen audit snapshot + a
     living design doc (names TBD; could reclaim
     `por-dual-parity.md` for the design).
   - Refocus the stub to conversion-only and rename (e.g.
     `por-dual-conversion.md`); drop its redundant parity
     half.
   - Repoint refs (`todo.md` `[1]` + the `por → dual` Todo,
     `copying.md`, the audit's internal anchors + Reading
     guide) and validate; `chores-10/11/12` mentions are
     historical and stay.
   - Promote the Gap-list items to anchored
     `#### Gap N — <title>` sub-headings so cross-cycle
     citations can deep-link a specific gap (markdown
     anchors headings, not list items). Trade-off: stable
     anchors, but the ordinal lives in the heading text
     (manual renumber on reorder) — fine for a consumed
     backlog. The 3 `Gap #N` links in the `0.62.0`
     close-out chores narrative resolve only to the section
     until this lands.
   - Deferred from the 0.62.0 close-out: close-out is
     bookkeeping-only, and the split is substantive,
     anchor-heavy work warranting its own cycle.
3. **Chores retire into a session index (post-viewer).**
   Once the provenance viewer ("`vc` as a code+conversation
   provenance tool" above) can present a commit's session
   and code side by side, the hand-written chores narrative
   is a distillation of a conversation the bot repo already
   records verbatim — the DRY argument that removed edit
   lists from chores (git owns the mechanics) then applies
   to the narrative too (the session owns it). Chores
   collapses to an index into the session.
   - The `ochid:` trailer links a work commit to a session
     *commit*; the index adds within-session granularity —
     which conversation span produced the commit, where the
     design argument happened. We think it can be generated
     (the transcript records when pushes happen), making it
     drift-proof where hand-written chores never were.
   - What survives: the curated design layer (the
     refactor-20260716.md pattern). Sessions are an
     immutable journal — good as record, poor to cite
     into — so live design references keep pointing at
     curated docs, not per-cycle narrative sections.
   - The template side already points this way: chores
     files are not seeded; a new project's history is its
     own commits + bot session from day one.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

_Migrated to [done.md](notes/done.md) on 2026-07-24 (the DRY jj facade
cycle and its two docs interludes: template repo names, notes rework)._

- docs: trapezoid close-out recipe — the four steps that
  publish a trapezoid close-out consolidated into one
  definitive procedure in cycle-protocol.md (base rule,
  the two-parent verification, the sideways-move backfill
  embargo, recovery), with the refactor stage keeping only
  implementation deltas and README waiting for the flag;
  vocabulary collapsed to "trapezoid"; chores-15 opened
  [[10]]

- refactor: repo registry — `.vc-config.toml`'s `[workspace]`
  block becomes a `[repos]` registry: ordinary file-relative
  (or absolute) paths, side detection by self-resolution,
  resolved agreement + self-identification replacing the
  identical-block invariant, and ochid prefixes as canonical
  side labels decoupled from the bot dir's spelling; legacy
  reads consolidated in `src/legacy_vc_config.rs` for later
  retirement. De-gitify init rode as the last rung — init's
  publish path is jj-only and bugs.md #1/#2 are fixed; fourth
  stage of the jj refactor program [[8]]

- docs: refactor program ladder + conventions — the refactor
  program's remaining stages consolidated into four cycles
  and laid out as a program ladder under a new heading-based
  `## In Progress` shape; the parked 0.72.0 branch declared
  quarry (version gap accepted); the version-first-bullet
  body convention added to cycle-protocol.md; chores-14
  0.75.0 rung refs backfilled [[5]]

_Migrated to [done.md](notes/done.md) on 2026-07-28 (the
hygiene-riders and facade-owns-topology cycles)._

# References

[1]: https://github.com/winksaville/vc-x1/commit/b5e40e7458b8 "b5e40e7458b8506574b2ae01f52f7ccae9023418"
[2]: https://github.com/winksaville/vc-x1/commit/946dc964b75c "946dc964b75ca29e2cc4b6c59f03aec2c364feee"
[3]: https://github.com/winksaville/vc-x1/commit/dc14a421d850 "dc14a421d8509e58fa05741fd1a868329540731e"
[4]: /notes/forks-multi-user.md
[5]: /notes/chores/chores-14.md#docs-refactor-program-ladder--conventions
[6]: https://github.com/winksaville/vc-x1/commit/4898d93e4172 "4898d93e41720070cddb995bfd4e53ffc38ccb88"
[7]: https://github.com/winksaville/vc-x1/commit/ab3a07d4903b "ab3a07d4903bbe6ae7cec5490f5edd622161c72e"
[8]: https://github.com/winksaville/vc-x1/commit/846b5eee5b98 "846b5eee5b988b0cd8887559a0fba3397155ee19"
[8]: /notes/chores/chores-14.md#refactor-repo-registry
[9]: https://github.com/winksaville/vc-x1/commit/71611891f67a "71611891f67a34f5e11a344ffe4e439ace93750f"
[10]: /notes/chores/chores-15.md#docs-trapezoid-close-out-recipe
