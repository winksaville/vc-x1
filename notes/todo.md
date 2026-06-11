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

**fix finalize ochid-dropping squash.** `finalize_exec`
(`src/finalize.rs`) squashes with `--use-destination-message`
unconditionally, so a described journal on `@` loses its message
and `ochid:` trailers — this broke the code↔session cross-links
in the fc project ([bugs.md](bugs.md) Bugs #1). Decision: refuse
the squash when the source message carries `ochid:` trailers the
destination's message lacks — the destination has its own ochids,
so any automatic message merge guesses wrong in some direction.

   - 0.65.2-0 Preparation: backfill 0.65.1 Commits ref, bump
     version, pick up this entry, open chores section (done)
   - 0.65.2-1 ochid-trailer guard in finalize: extract + compare
     trailers, refuse in preflight and finalize_exec; unit tests
     + README manual-test section + post-run marker surfacing
     + support/gen-exmpl-1-3.sh transcript regenerator (done)
   - 0.65.2 close-out and validation

## Todo

 Entries are in **strict priority rank** — #1 highest,
 descending. Reprioritize by moving an entry, then
 `vc-x1 fix-todo --no-dry-run notes/todo.md` to renumber.
 The numbers are positional rank, not stable IDs — to refer
 to a Todo, name it by its **title** (a greppable mention;
 a numbered list item has no anchor to link to), not its
 number. Long-tail entries
 live in [todo-backlog.md](todo-backlog.md). Use the
 [Prose Form in AGENTS.md](/AGENTS.md#prose-form); deeper
 detail goes in `notes/chores/chores-NN.md` design
 subsections (link via `[N]` ref).

1. **Version-number protocol is fragile — versions are
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
2. **validate-numbering: rename the pair, check all
   sequence-managed notes files generically.** `validate-todo`
   / `fix-todo` only operate on the single file passed, so a
   renumber slip in `bugs.md`, `todo-backlog.md`, or
   `todo.md`'s `## Ideas` section passes unnoticed — too weak
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
     files (`todo.md`, `todo-backlog.md`, `bugs.md`) so the
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
   - Open: revisit fixed-vs-glob at implementation if the
     fixed list proves annoying to maintain.
3. **pre-commit: single rule (no docs skip) + doc validators.**
   The pre-commit (cargo cycle: fmt/clippy/test/install) only
   checks code, so it's "skip-able for purely-docs commits" —
   but that exception is exactly where checks slip (skipped on
   0.62.0-7/-8 until caught). And `vc-x1 push`'s `preflight`
   stage re-runs the same cycle, which invites treating push as
   the gate rather than a redundant safety-net.
   - Adopt one rule, no exception: the pre-commit runs before
     Work review on every commit; push's `preflight` is a
     safety-net, not the primary gate. (docs: AGENTS.md Cycle
     Protocol summary + cycle-protocol.md per-commit-flow.)
   - Enrich the pre-commit so it's meaningful on docs commits:
     add the doc validators — `validate-numbering` (its own
     Todo, a prereq) plus `validate-repo` when it exists — to
     both the documented flow and push's `preflight` stage
     (`push.rs`), with a test. (code)
   - This dissolves the docs exception: with doc validators in
     the pre-commit there's always something to validate, so
     the carve-out stops making sense.
   - Its own near-term cycle (chosen over a 0.61.1 insert to
     avoid rewriting published 0.62.0-x history); no version
     pre-assigned — see the Todo "Version-number protocol is
     fragile" on fragile version targets.
4. **vc-x1 push: validate body opens with an intro paragraph.**
   A body whose first line is a bullet (`- file: …`) is a
   Prose-Form violation — bodies must open with an intro
   paragraph, then bullets. Today such a body trips jj's arg
   parser (`jj commit -m "<body>"` reads the leading `-` as a
   stray flag) and push fails with an opaque error. Hit on
   0.62.0-5.
   - Feature, not a parser bug (reframed): push should
     *validate* the body opens with a non-dash intro line and
     flag its absence with a clear, specific error pointing at
     the offending first line — rather than letting jj emit a
     confusing one, or quietly accepting a bullet-first body.
   - Enforcing the intro is the intended behavior, matching
     the Prose-Form convention; we are not "fixing" the parser
     to accept bullet-first bodies.
   - Workaround until the explicit check lands: prepend a
     non-dash intro sentence to the body.
5. **vc-x1 push: support new cycle protocol shape (N:1 code↔bot).**
   Today push assumes 1:1 symmetric WC commits with shared
   title/body. The new cycle protocol has a different shape on
   each side:
   - code side: fully committed before push (N commits via
     merge), nothing to commit at push time
   - `.claude` side: one commit with its own message (distinct
     from any code commit) and a multi-line `ochid:` listing
     all N code commits

   Teach push to:
   - detect this shape
   - skip `commit-app` when code WC is empty
   - compose a `.claude`-specific message
   - emit multi-line `ochid:` per the design in [[10]]

   Today's workaround: pre-commit `.claude` manually, then
   `vc-x1 push <bm> --from bookmark-both --yes`.
6. **vc-x1 push --squash: symmetric squash on both repos.**
   Automate Option F (manually exercised in 0.59.0
   close-out): app-side squash + bot-side description
   rewrite + force-push, atomically. Without this,
   squash is a manual recipe that future cycles must
   follow each time.
   - App side: squash cycle commits into one new commit;
     capture the squashed chid.
   - Bot side: rewrite the prior push commit's
     description — replace its per-commit `ochid:`
     trailers with one pointing at the squashed chid;
     add a rewrite-note acknowledging the change
     (preserves historical truth for future readers).
   - Force-push bot `main` (rewrites the published
     commit; chid preserved via `jj describe`).
   - Push app `main`. The new bot commit paired with
     this push receives `ochid: /<squashed-chid>` as
     normal — produces a 2:1 bot→code pattern that's
     already in the protocol's design space.
   - Gates `Squash to one commit` as a routine
     close-out shape vs. the current manual recipe.
7. **single-field `options_flags` leaves → `value` field.**
   `0.47.0` introduced the convention (single-field leaf names
   its field `value`, declares the flag via `#[arg(long = "…")]`,
   so consumers read `args.<leaf>.value` not `args.<leaf>.<leaf>`)
   on the new `squash` leaf. Sweep the pre-existing single-field
   leaves to match: `repo`, `dry_run`, `private`, `account`,
   `config`, `use_template` + their consumers
   (`init.rs`, tests).

   Note: can a single field be defined as an type or enum instead
   of a struct and maybe eliminate the `args.<leaf>.<leaf>` name
   issue.
8. **`por → dual` conversion.** Attach a `.claude`
   companion repo + `.vc-config.toml` to an existing por
   workspace; emit cross-links going forward. Manual
   setup on an external por workspace (2026-05-14)
   proved arduous; this should be a routine subcommand.
   Design stub in [[1]] § 2.
9. **`validate-desc` / `fix-desc` por equalization.**
   Replace the `other_repo_from_config` prelude in both
   subcommands (`validate_desc.rs:133`, `fix_desc.rs:152`)
   with a scope-aware resolution that no-ops `Side::Bot`
   when absent. Body unchanged. The 0.61.0 audit/design
   work [[13]] identified this as the cheapest concrete
   equalization and the right prototype for the
   topology-from-config rule (subcommand reads topology
   from `default_scope`, not from a flag). Validates the
   broader design before larger pieces (`push`,
   `--init-from*`) commit to it. The remaining 13
   implementation gaps live in [[13]]'s `## Gap list` for
   future Preparation passes to pick up.

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

_See [bugs.md](bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

_Migrated to [done.md](done.md) on 2026-05-15 (0.44.0–0.50.0 batch)._

- chores subdir reshape — `notes/chores-*.md` → `notes/chores/`; 0.44.0–0.50.0 Done batch migrated to done.md (0.51.0) [[2]]
- `sb_ide` elimination — banner off by default (`-V` toggles), `bm_track` → `debug!`, `sb_ide` + `SubcommandRunner::{is_detached_exec, suppress_banner}` removed (0.52.0) [[3]]
- todo renumber + `notes/fix-todo.py` interim script; cycle re-scoped at close-out, scope CLI cleanup deferred to 0.54.0 (0.53.0) [[4]]
- scope CLI cleanup — `--scope` roles-only, `--por` boolean replaces `ScopeKind`, `Scope` relocated to `options_flags/`, sync gains `-R` (0.54.0) [[5]]
- validate-todo / fix-todo subcommands — check + renumber `## Todo` / `## Bugs` entry numbering, replacing `notes/fix-todo.py` (0.55.0) [[6]]
- refine cycle protocol — one protocol (Preparation/Work-N/Close-out), `.`-separator nested numbering with trailing-`0`=Preparation, push & squash discretionary, `.claude` once per push, two-gate review (work then message, both before commit), CLAUDE.md cycle/commit/push docs consolidated into one linear `## Cycle Protocol` (~39% smaller) (0.56.0) [[7]]
- add `--merge` todo entry — Todo #1 records future `vc-x1 push --merge` flag (close-out shape, sibling to planned `--squash`); dogfoods the Preparation/Work-N/Close-out protocol on a deliberately small docs cycle (0.57.0) [[8]]
- notes/todo restructure — split `## Bugs` → `bugs.md` and the long-tail `## Todo` → `todo-backlog.md`; `## Priorities` with tier sub-headings (`### P1`/`### P2`/`### P3`); CLAUDE.md `## File reads` rule + protocol codification (chores title-only during cycle, In Progress moves into chores at close-out, problem+plan shape) (0.58.0) [[9]]
- extract cycle protocol — `notes/cycle-protocol.md` becomes the canonical self-contained home for the cycle workflow (504 lines, extensively tightened from the CLAUDE.md extract); CLAUDE.md keeps a 10-line pointer; `notes/substep-protocol.md` folded in as `## Sub-cycle ladders`; `## Ideas` section added to `notes/todo.md`; first squash close-out via manual Option F (app squash + bot-side `af60f979` trailer rewrite + force-push) (0.59.0) [[11]]
- consolidate notes conventions — three notes-file sections (`Todo format`, `Reference numbering`, `Retiring Done entries`) move from notes/README.md into new CLAUDE.md `## Notes file conventions` umbrella alongside existing `## Chores conventions`; `[[N]]` citation duplicate dropped; cargo cycle (`fmt` / `clippy` / `test` / `install`) surfaced at CLAUDE.md `## Cycle Protocol` and notes/README.md (had been buried in cycle-protocol.md since 0.59.0); README.md `## Contributing` rewritten against current anchor homes (0.60.0) [[12]]
- por/dual parity design — eight-commit audit + design cycle producing `notes/design-cli/por-dual-parity-audit.md` as the canonical CLI-design doc (audit + commonality + feature axes + 5-layer resolution chain + subcommand × parameter matrix + per-axis Decisions blocks); new sibling `notes/design-cli/copying.md` stub for the broader file-copy mechanism that subsumes `--config` / `--gitignore` / `--use-template`; `notes/design-cli/` subdir created and three design notes regrouped under it; 14 implementation gaps seeded for 0.62.0+ cycles; one Todo promoted (`validate-desc` / `fix-desc` equalization, cheapest prototype for the topology-from-config rule) (0.61.0) [[13]]
- apply max review #1 — applied six concerns, four nits, and the process observation from the `max-review-1` working list to the por/dual parity design + copying stub; reframed Todo #1 (push validate body intro), seeded pre-commit-single-rule + `validate-numbering` Todos; working list fully drained, then retired (deleted — git history holds it) (0.62.0) [[14]]
- docs: adopt AGENTS.md — rename `CLAUDE.md` → `AGENTS.md` (Zed and the agent-tooling ecosystem default to it); one-line `@AGENTS.md` import shim at `CLAUDE.md` keeps Claude Code auto-loading; live `CLAUDE.md` references repointed to `AGENTS.md` so links resolve in editors and on GitHub; history prose (`chores-01..12` / `done.md`) left as written, with only the 3 navigational anchor links in the `chores-10/11/12` headers repointed (0.63.0) [[15]]
- docs: tighten after-finalize rule — rename to "After push or finalize: stop and wait" (both triggers named) and spell out that `vc-x1 push` bundles the push + `vc-x1 finalize` on `.claude` as tail stages, so all closing words go *before* invoking the wrapper and nothing is emitted after it returns (0.63.1) [[16]]
- docs: codify merge-non-ff recipe — promote the merge-non-ff close-out recipe to a `### Merge non-ff recipe` subsection in cycle-protocol.md (rebase → `jj new` lift → push + post-hoc caveat); reword `### Shape at close-out push` (work-done framing, Merge non-ff tagged default); standardize jj rebase `-d` → `--onto`/`-o` in AGENTS.md and drop the post-amend `jj new` note (the recipe now owns the empty-`@` why); also clarified the Preparation step (Cargo.lock, In-Progress move wording) (0.64.0) [[17]]
- docs: record finalize ochid-loss bug (0.65.1) — bugs.md gains the fc finalize ochid-drop incident as Bugs #1 with the fix queued as Todo #1; fc AGENTS.md additions ported (jj-not-git, one-command-per-invocation, push-injects-trailers, ochid resolvability + `.vc-config.toml`); stale chores-10 "active file" prose genericized in notes/README.md + ARCHITECTURE.md [[18]]

# References

[0]: /AGENTS.md#prose-form
[1]: /notes/design-cli/por-dual-parity.md
[2]: /notes/chores/chores-11.md#chore-move-chores-under-noteschores-0510
[3]: /notes/chores/chores-11.md#chore-close-sb_ide-elimination-0520
[4]: /notes/chores/chores-11.md#chore-todo-renumber--fix-todopy-0530
[5]: /notes/chores/chores-11.md#refactor-scope-cli-cleanup-0540
[6]: /notes/chores/chores-11.md#feat-validate-todo--fix-todo-0550
[7]: /notes/chores/chores-11.md#docs-refine-cycle-protocol-0560
[8]: /notes/chores/chores-11.md#docs-add---merge-todo-entry-0570
[9]: /notes/chores/chores-12.md#refactor-notestodo-restructure-0580
[10]: /notes/forks-multi-user.md
[11]: /notes/chores/chores-12.md#docs-extract-cycle-protocol-0590
[12]: /notes/chores/chores-12.md#docs-consolidate-notes-conventions-0600
[13]: /notes/chores/chores-12.md#docs-pordual-parity-design-0610
[14]: /notes/chores/chores-12.md#docs-apply-max-review-1-0620
[15]: /notes/chores/chores-13.md#docs-adopt-agentsmd-0630
[16]: /notes/chores/chores-13.md#docs-tighten-after-finalize-rule-0631
[17]: /notes/chores/chores-13.md#docs-codify-merge-non-ff-recipe-0640
[18]: /notes/chores/chores-13.md#docs-record-finalize-ochid-loss-bug-0651
