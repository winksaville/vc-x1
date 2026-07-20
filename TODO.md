# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

**feat: push merge close-out (trapezoid)** — teach
`vc-x1 push` the Merge non-ff close-out shape natively.
The trapezoid is a first-party shape the user chooses at
push time (this project's usual choice for a multi-commit
cycle; `--merge` is opt-in, never automatic), but push
only supports Keep separate: today a trapezoid close-out
pre-commits both sides manually, rebases the code
close-out into the merge, and resumes via
`--from bookmark-set --yes` — skipping exactly the stages
that inject `ochid:` trailers.
   - Pulled forward from the refactor program's
     [trapezoid close-out stage](notes/refactor-20260716.md#stage-trapezoid-close-out),
     spawn-based for now — the in-process merge transaction
     folds into the jj-lib migration stage.
   - Absorbs the retired Todo "vc-x1 push: pause point
     between commit and publish stages" — the pause was the
     interim manual path for the same close-out; native
     merge support supersedes it unimplemented.
   - Folds in the retired Ideas "Codify ochid invariant …"
     codification remnants at the docs step (ochid
     invariant, bot-repo rules, Ideas-aware
     Preparation/Close-out); its cross-repo migration
     sketch moved to todo-backlog.md.
   - Design points for the merge stage:
     - merge sits after commit-bot, before bookmark-set —
       chids survive the rebase, so both ochid directions
       are injected normally and stay valid
     - `--merge [<base>]` — base is the merge's first
       parent (the parent of the task's first commit):
       - branch style (task on a feature bookmark, target
         bookmark sitting at the base): base defaults to
         the target bookmark's position
       - main style (task grew on the target, bookmark
         sitting at the work tip): base can't be inferred
         — the explicit `<base>` revision is required
     - mechanics: `jj rebase -r <closeout> --onto <base>
       --onto <work tip>` then `jj new <merge>` to re-seat
       an empty `@`
     - preconditions: base is a proper ancestor of the
       work tip; close-out has exactly one parent
       pre-rebase; target bookmark is at the base or at
       the work tip — anything else is divergence (e.g.
       another repo instance moved the target): reconcile
       first, preflight's sync check catches the moved
       remote
     - both styles publish forward — the old bookmark
       position is one of the merge's parents, so the push
       never forces; post-hoc conversion of an
       already-published close-out stays the manual
       `--ignore-immutable` recipe (cycle-protocol's
       post-hoc caveat), not this flag
     - ochid list: one `ochid:` per work commit the push
       newly publishes (held-local cycle: whole ladder +
       merge); whether to skip commits already covered by
       an interim 1:1 push decided at the stamping step
       (cheap bot-journal trailer scan exists in
       `squash_push::extract_ochids`)
   - Ladder:
     - 0.72.0-0 chore: open merge close-out cycle — version
       bump, todo pickup + triage, chores section (done)
     - 0.72.0-1 refactor: extract push/state.rs — the
       refactor program's split-push.rs stage pulled
       forward (Stage, PushState, state layout) so the
       merge stage lands in a file that reviews cleanly
     - 0.72.0-2 push: `--merge [<base>]` flag + merge-work
       stage — two-parent rebase, `jj new` re-seat, state
       persistence/resume/--status/--dry-run/rollback;
       integration tests of both styles (branch-based and
       on-target ladders)
     - 0.72.0-3 push: N-ochid stamping — commit-bot emits
       one `ochid:` per newly published work commit; tests
     - 0.72.0-4 validators: validate-desc + push sanity
       verifiers learn the merge shape (two parents, N
       ochids); one string-level multi-ochid parser
     - 0.72.0-5 docs:
       - cycle-protocol.md: Merge non-ff recipe → wrapper
         flag; wrapper-limitation note; ochid invariant +
         bot-repo rules codified
       - the two trapezoid variants — committer-owned
         (close-out is the merge; this flag) vs PR-style
         (the integrator authors the merge; needs no
         vc-x1 machinery) — and why merge-commit
         integration, never squash/rebase, preserves
         chids and thus ochids; point at
         forks-multi-user.md
       - notes/README.md; refactor doc stage status
     - 0.72.0 close-out and validation — dogfood: this
       cycle lands via `push --merge`

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

1. **Refactor: typed jj facade → jj-lib in-process; end
   subprocess spawning.** Version-control operations are
   ~30 hand-rolled `run("jj", …)` spawns plus every
   mutation, with per-module private wrappers and raw-git
   vestiges in init — stderr parsing instead of typed
   errors, and jj's single-attempt index-lock acquisition
   (the push `bookmark-set` lock race in [bugs.md](notes/bugs.md))
   can't be retried where it fails. A multi-ladder program;
   the staged plan, design detail, and the eight absorbed
   former Todos live in
   [refactor-20260716.md](notes/refactor-20260716.md).
   - Stages in execution order: DRY facade → hygiene
     riders → facade owns topology → de-gitify init →
     split push.rs → jj-lib migration → push body-intro
     validation → trapezoid close-out → por → dual
     conversion.
   - split push.rs + trapezoid close-out pulled forward
     into the in-progress "feat: push merge close-out
     (trapezoid)" cycle, spawn-based; their in-process
     form folds into the jj-lib migration stage.
2. **ochid: bot-repo location qualifier.** An ochid is
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
3. **Version-number protocol is fragile — versions are
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
4. **sync follow-up: extract `move-bookmark` command.** The
   "put the bookmark / `@` where it belongs" step at the end
   of sync (reposition logic) is useful standalone — e.g. the
   t1B scenario where `main` is right but `@` isn't on it —
   and deserves an honestly-named command instead of a mode.
   - `vc-x1 move-bookmark` (name open): no fetch; move `@`
     (and optionally the bookmark) onto a target under the
     same safety rules as sync's reposition step.
   - Sync's final step becomes a call to the same logic.
   - Follow-up to the 0.67.0 single-mode sync cycle.
5. **sync follow-up: push preflight in-process; drop
   `--check`; revisit push auto-rollback.** Push's preflight
   shells out to `vc-x1 sync --check` — a verify-only pass
   that is both racy (remote can move before the user's
   later apply) and not actually read-only (jj's fetch
   auto-ffs tracked bookmarks). Follow-up to the 0.67.0
   single-mode sync cycle.
   - Preflight becomes a real in-process sync (stop-on-error
     halts the push before anything is committed); the
     `sync --check` shell-out and PATH dependency go away.
   - Remove the deprecated hidden `--check` alias once
     nothing invokes it.
   - Apply the stop-on-error + `vc-x1 revert` philosophy to
     push's commit-stage rollback (today it auto-runs
     `jj op restore`, hiding the evidence).
6. **validate-numbering: rename the pair, check all
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
7. **pre-commit: single rule (no docs skip) + doc validators.**
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
8. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
   Today push assumes 1:1 symmetric WC commits with shared
   title/body. The interop / adoption scenario breaks that:
   the code side is worked single-repo style (commit +
   `jj git push` / `git push`, no `vc-x1 push` in the loop),
   so no bot pairings exist — one bot commit then records
   every code commit not yet covered by a prior `ochid:`,
   via a multi-line `ochid:` per the design in [[10]].
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
9. **Run validate-bot at every vc-x1 invocation
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
10. **README: audit flag tables and examples against the
    current CLI.** 0.69.0-4 fixed the init section (it
    documented retired `--owner` / `--dir` / `--repo-local`
    flags) and the 0.69.0 surfaces, but the README's other
    tables (clone, symlink, sync, revert, list/desc/chid/
    show) have never had a systematic `-h` comparison and
    drift silently.
    - Sweep each section against `vc-x1 <cmd> -h`.
    - Consider regenerating transcripts via support
      scripts (the gen-exmpl pattern) so examples stay
      reproducible.
11. **Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs.** Adopted in chores-13 (0.69.2 ladder,
    backfilled during 0.70.0-0): each rung is prepended
    with its commit reference so the rung↔commit
    correlation is direct; `Commits:` stays as the
    section-level list. The convention's home —
    cycle-protocol.md Close-out ("Add an `### As-built
    ladder`…") — is in the byte-identical shared doc set
    (vc-x1, vc-template-x1, iiac-perf), so the doc edit
    needs a coordinated three-project sync, not a
    mid-cycle local change.
12. **Shared-doc sync: per-commit chores convention.**
    0.71.0 changed how chores are recorded — each work commit
    appends its As-built rung + narrative as it lands, rather
    than the narrative waiting for close-out. That wording edit
    was made locally in vc-x1's `cycle-protocol.md` / `AGENTS.md`
    (the byte-identical shared doc set), so vc-template-x1 and
    iiac-perf now diverge until the same edit is applied there —
    a coordinated three-project sync (same family as the Todo
    "Shared-doc sync: As-built ladder rungs carry `[[N]]`
    commit refs").
13. **config: extract flag-backed key descriptions from Clap.**
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

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

_Migrated to [done.md](notes/done.md) on 2026-07-14 (0.51.0–0.65.2 batch)._

- feat: config discoverability + scalar hierarchy — a
  code-declared config schema registry (`config_schema.rs`) as
  the single source of truth for every settable config key; the
  new `config` command (print, `--home`, `--validate`), init's
  commented `.vc-config.toml` defaults, and bot-session's
  `--result-lines`/`--col-width` config layer all derive from it,
  so they can't drift. Also a `notes/transcript-format.md` SSOT +
  sample for the bot-session format, and a sweep retiring the
  ambiguous "dotted" wording [[7]]
- bot-session: --fields / --unknown output clarification — the
  inventory views are now documented rather than opaque:
  [transcript-format.md](notes/transcript-format.md) defines
  entry / entry type and the `.`/`[]` field notation with a
  bot-session example (0.71.0-8), and the ambiguous "dotted"
  wording was retired (0.71.0-7). In-view column labeling
  (headers / a legend) left as an optional nicety
- docs: shared protocol sync + jj refactor plan — adopted the vc-template-x1 shared notes set (AGENTS.md, cycle-protocol.md, versioning.md, jj-tips.md) with vc-x1's 0.69.0 corrections ratified template-side (manifest: [notes-sync-20260716.md](notes/notes-sync-20260716.md)); jj facade → jj-lib refactor program planned in [refactor-20260716.md](notes/refactor-20260716.md), absorbing eight Todos [[1]]
- docs: move todo.md to root TODO.md — todo list moved from notes/ to the conventional root-file family; live references swept (AGENTS.md, cycle-protocol.md, README, ARCHITECTURE, notes/*); no-arg validate-todo / fix-todo default follows the move; historical files keep `notes/todo.md`; the shared doc set diverges until vc-template-x1 and iiac-perf apply the same change [[2]]
- feat: bot-session --col-width knob — the field views'
  (`--fields`/`--unknown`/`--per-line`) first-column pad
  becomes `--col-width N`, default widened 44 → 68 (aligns
  the type column for ~99% of observed key paths; only the
  long-tail `snapshot.trackedFileBackups.<abs path>.*` keys
  overflow); config-hierarchy resolution deferred to Todo #12 [[6]]
- feat: bot-session --fields + --raw explorer — bot-session
  doubles as a schema explorer: --fields (dotted-path
  inventory per entry type: count, kinds, samples),
  --unknown (inventory minus the extractor's KNOWN_PATHS —
  the unmodeled surface; 132 paths on first real run),
  --raw (pretty-printed source lines); --per-line (a fields
  section per source line, composes with --unknown); --lines
  unified to source-JSONL-line units in every view,
  conversation included; drift-over-time baseline deferred to the
  discovery/index cycle [[5]]
- feat: bot-session --result-lines knob — the [result]-body
  cap becomes a flag: `--result-lines N` (default 10, 0 =
  unlimited), Output-range help group; was hardwired to 10
  even under --all [[4]]
- feat: bot-session transcript viewer — display a session transcript as a conversation: two-layer tolerant parse (serde_json text → Value; hand extraction into our structs, raw retained), eight-item composable output (--<item> / --no-<item> / --all / --none) with git-style config defaults (CLI > .vc-config.toml > user config > built-in), --lines slicing, UTC headers; --raw and index view deferred (Todo #12), EPIPE logger panic recorded (Bugs #4) [[3]]

# References

[1]: /notes/chores/chores-13.md#docs-shared-protocol-sync--jj-refactor-plan
[2]: /notes/chores/chores-13.md#docs-move-todomd-to-root-todomd
[3]: /notes/chores/chores-13.md#feat-bot-session-transcript-viewer
[4]: /notes/chores/chores-13.md#feat-bot-session---result-lines-knob
[5]: /notes/chores/chores-13.md#feat-bot-session---fields----raw-explorer
[6]: /notes/chores/chores-13.md#feat-bot-session---col-width-knob
[7]: /notes/chores/chores-13.md#feat-config-discoverability--scalar-hierarchy
[10]: /notes/forks-multi-user.md
