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

**feat: bot-session transcript viewer**

Display one Claude Code session transcript
(`.claude/<uuid>.jsonl`) as a readable conversation — first step
toward seeing all bot sessions and linking prompts to the
changes they produced. Two-layer parse: serde_json as text →
`Value` only (no derive); hand-written extraction into our own
structs that evolve as we learn the format. v1 is file-path in,
conversation view out; index view, session discovery, and
cross-file references (sidechain `agent-*.jsonl`, compaction
chains) come in later cycles. Full design in the approved plan
(bot session of 2026-07-17). v1 uses no version-control code;
when later cycles link prompts to commits (chids, ochid
trailers), they go through jj-lib in-process per the typed
jj facade Todo #1 — no new `run("jj", …)` sites.

- 0.70.0-0 chore: open show-session cycle (done)
- 0.70.0-1 feat: transcript parse + typed layer for show-session (done)
- 0.70.0-2 feat: bot-session command + conversation renderer (done)
  (command renamed from show-session mid-step; the pushed
  -0/-1 titles keep the old stem)
- 0.70.0 close-out and validation

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
2. **vc-x1 push: pause point between commit and publish
   stages.** The merge non-ff close-out is a three-step
   sequence:
   - commit the close-out pair locally (normal 1:1 commit
     stages, ochids injected both directions)
   - rebase the code-side close-out into the merge (chids
     survive rebase, so every ochid stays valid)
   - publish

   Push has no supported stop after `commit-bot`, so today
   the recipe pre-commits both sides manually and resumes via
   `--from bookmark-set --yes` — skipping exactly the stages
   that inject `ochid:` trailers.
   - Add a stop after the commit stages (`--to commit-bot`
     or `--no-publish`; name open); the existing `--from
     bookmark-set` is already the resume half.
   - Retires the close-out workaround; the merge commit
     carries its code→bot ochid because it was injected
     normally, before the rebase.
   - Together with the Todo "push/sync: bookmark is
     code-repo-only; pin the bot repo to main", completes
     the trapezoidal-commit workflow (1:1 bot↔code
     throughout; the merge is a code-side-only shape
     operation).
   - Interim recipe only: the refactor program's
     [trapezoid close-out stage](notes/refactor-20260716.md#stage-trapezoid-close-out)
     is the end state; this pause point remains the manual
     path until it lands.

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
   - Also covers a cycle held local and published all at
     once (the ochid-trailers section's "one ochid per Work
     commit" on merge close-out) — work commits never
     individually paired.
   - Out of scope: the trapezoid close-out. That flow stays
     1:1 (the close-out pair commits normally, then the
     merge rebase; chids survive rebase, so ochids stay
     valid); its enabler is the Todo "vc-x1 push: pause
     point between commit and publish stages".
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
12. **bot-session --raw mode.** Verbatim source lines
    (`cat`-like, no rendering) but honoring `--lines` —
    whose unit then becomes *source JSONL lines*, matching
    the malformed-line warnings and what jq/editors see, not
    rendered lines. Needs its own small design pass (unit
    switch must be explicit in help; summary suppressed or
    to stderr in raw mode). Could ride with the index-view
    cycle.

## Ideas

 Items not yet solid enough for `## Todo` (or surfaced
 during close-out / end-of-day before they're fully
 formed). Triaged at the next Preparation: promote to
 `## Todo` / `notes/todo-backlog.md`, fold into a
 picked-up cycle, or drop.

1. **Codify ochid invariant + bot-repo rules + squash
   gating + cross-repo migration in
   `notes/cycle-protocol.md`.** Was planned for
   `0.59.0-2` but deferred when 0.59.0 closed out via
   squash + Option F (manual bot-side rewrite). The
   rules were exercised manually for the close-out;
   this Idea formally codifies them.
   - Codify the **ochid invariant** in `## ochid
     trailers`: every public ochid must resolve in the
     public graph.
   - Codify bot-repo rules: never squashed; descriptions
     editable (`jj describe` preserves chid).
   - Codify squash gating: until `vc-x1 push --squash`
     exists, manual symmetric squash (Option F: app
     squash + bot-side trailer rewrite + force-push) is
     the standard recipe; merge non-ff is the default
     shape for multi-commit cycles (the concrete recipe is
     its own Todo — "merge-non-ff recipe").
   - Sketch cross-repo migration: ochids change at
     every merge until the change reaches the canonical
     repo's `main`.
   - Apply Ideas-aware Preparation/Close-out updates:
     Preparation triages `## Ideas` (promote / fold /
     drop) before declaring the plan; Close-out
     captures unresolved follow-ups into `## Ideas`.

2. **`vc` as a code+conversation provenance tool (grander
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
3. **Restructure the design-cli parity docs (target
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

- feat: pin bot repo to main (0.68.0) — `--bookmark` is code-repo-only in push and sync; the session repo's side of every step (tracking preflight, classify/act, `bookmark-set` — renamed from `bookmark-both` — `finalize --push`, completion sanity) is pinned to `main`; plus two mid-cycle sync fixes: `reposition_session` no-ops when `@-` is the `main` tip, and the clean case prints one `nothing to sync` summary line [[23]]
- docs: diagnose silent session-push loss (0.68.1) — Bugs #1 root-caused: push's detached finalize child is killed at sandbox teardown before its delayed squash/push runs, so bot-run pushes never push `.claude`; diagnosis recorded in bugs.md, fix design queued as Todo #1 (inline session push + preflight backstop + finalize as the user's empty-@ tidy-up); 0.68.0 chores `Commits:` backfilled [[25]]
- feat: inline session push + squash-push (0.69.0) — push's session publish is in-process (`squash-push-bot` stage; a failure is a visible push failure — the silent session-push loss fixed); `finalize` renamed to the zero-ceremony `squash-push` (detach / delay / failure markers retired, no alias); new `vc-x1 validate-bot` + an erroring push-preflight backstop enforce the at-rest `main == main@origin` invariant (no auto-fix); push preflight drops the hardcoded cargo steps (vc-x1 assumes nothing about repo contents beyond `.jj` + `.vc-config.toml`); work/bot terminology + stage renames across code and docs, README rewritten (Terminology section, live-validated walkthroughs); crate renamed back to `vc-x1` [[20]]
- docs: shared protocol sync + jj refactor plan — adopted the vc-template-x1 shared notes set (AGENTS.md, cycle-protocol.md, versioning.md, jj-tips.md) with vc-x1's 0.69.0 corrections ratified template-side (manifest: [notes-sync-20260716.md](notes/notes-sync-20260716.md)); jj facade → jj-lib refactor program planned in [refactor-20260716.md](notes/refactor-20260716.md), absorbing eight Todos [[1]]
- docs: move todo.md to root TODO.md — todo list moved from notes/ to the conventional root-file family; live references swept (AGENTS.md, cycle-protocol.md, README, ARCHITECTURE, notes/*); no-arg validate-todo / fix-todo default follows the move; historical files keep `notes/todo.md`; the shared doc set diverges until vc-template-x1 and iiac-perf apply the same change [[2]]

# References

[1]: /notes/chores/chores-13.md#docs-shared-protocol-sync--jj-refactor-plan
[2]: /notes/chores/chores-13.md#docs-move-todomd-to-root-todomd
[10]: /notes/forks-multi-user.md
[20]: /notes/chores/chores-13.md#feat-inline-session-push--squash-push-0690
[23]: /notes/chores/chores-13.md#feat-pin-bot-repo-to-main-0680
[25]: /notes/chores/chores-13.md#docs-diagnose-silent-session-push-loss-0681
