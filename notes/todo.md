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

**feat: inline session push + squash-push (0.69.0)**

Push's `finalize-claude` stage detaches a delayed child to
squash+push `.claude`; a sandboxed bot run kills the child at
command exit — silently, every cycle (see Bugs #1 [[24]]). The
empty-@ goal behind the detach/delay dance is self-referential
for the bot (finalizing is itself session data), so the bot
forgoes it; only the user, acting after the turn, can capture
the full tail.

Design notes:
- naming (decided 2026-07-15): `finalize` → `squash-push`
  - mechanism-named and repo-generic: a squash `@ → @-` +
    push
  - needed frequently on the bot repo (the session tail)
  - occasionally useful on the app repo: amend-and-push —
    a deliberate published-history rewrite + forced update
- squash-push of a just-pushed session commit rewrites it,
  so its push is a forced update
  - functions fine whether the bot or the user runs it
  - but on the bot repo only the user gets an empty `@`:
    the bot's own invocation writes new session data, so
    its `@` is non-empty again immediately
  - on any repo without a live writer it works as expected
    for anyone
- CLI shape
  - `-R`/`--repo`, default `.` (house convention: `-R`
    repo, `-r` revision)
  - positional BOOKMARK defaulting to `main` (mirrors push)
  - repo stays a flag — no command takes a positional repo
- no-op feedback
  - `@` empty and bookmark already at remote → "repo is
    already sync'd with remote", exit 0
  - `@` empty but remote behind → skip the squash, still
    push
- the bot may also run `squash-push` as a separate later
  command to capture the `vc-x1 push` record (leaves only
  its own)

Plan:
- 0.69.0-0 prep: backfill Commits:, bump version, pick up
  Todo #1, open chores section (done)
- 0.69.0-1 push: session squash+push inline (done)
  - replace the finalize-claude shell-out with an
    in-process squash of the trailing session writes +
    `jj git push --bookmark main -R .claude`
  - a session-push failure is a visible push failure
  - crate temporarily named `vc-x1-dev` so per-commit
    installs don't clobber the stable `vc-x1` another bot
    instance uses
  - tests
- 0.69.0-2 squash-push: rename `finalize` → `squash-push`
  - zero-ceremony default: `-R .`, BOOKMARK positional
    defaulting to `main`, no-op feedback
  - retire `--detach` / `--delay`
  - module + types follow: `finalize.rs` →
    `squash_push.rs`, `FinalizeArgs` / `FinalizeParams` →
    `SquashPushArgs` / `SquashPushParams`
  - `finalize_inline` dissolves: with the detach branch
    gone, preflight + exec is the command's only path, so
    the shim merges into the renamed entry point
  - decide the detach failure-marker machinery
    (`write_failure_marker` / `surface_previous_failures`):
    retire with `--detach`, but consider keeping the
    surfacing one cycle to drain markers from older
    installed versions
  - rename push stage `finalize-claude` →
    `squash-push-claude`; `--no-finalize` →
    `--no-squash-push`
  - decide deprecated `finalize` alias
  - tests
- 0.69.0-3 push preflight backstop: error (or auto-push)
  when `.claude main` is ahead of `main@origin`; tests
- 0.69.0-4 docs: cycle-protocol rewrite ("After push or
  finalize: stop and wait" rule, Recovery section), README
- 0.69.0 close-out and validation

Continuity (resume 2026-07-15):
- next: pick up -2 (mark it `(current)`) — the squash-push
  rename; every decision needed is in Design notes above
- installs this cycle: `cargo install --path . --locked`
  lands `vc-x1-dev`; PATH `vc-x1` stays 0.69.0-0 for the
  other bot instance; rename the crate back to `vc-x1` when
  that window closes (decide at close-out)
- cycle pushes go straight to `main` (keep-separate shape;
  -0 and -1 already published)
- -1 was pushed with `vc-x1-dev push` — first dogfood of
  the inline session push; push preflight + tests need an
  unsandboxed run (`~/.config/jj` writes)

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

1. **vc-x1 push: pause point between commit and publish
   stages.** The merge non-ff close-out is a three-step
   sequence:
   - commit the close-out pair locally (normal 1:1 commit
     stages, ochids injected both directions)
   - rebase the code-side close-out into the merge (chids
     survive rebase, so every ochid stays valid)
   - publish

   Push has no supported stop after `commit-claude`, so today
   the recipe pre-commits both sides manually and resumes via
   `--from bookmark-set --yes` — skipping exactly the stages
   that inject `ochid:` trailers.
   - Add a stop after the commit stages (`--to commit-claude`
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

2. **Version-number protocol is fragile — versions are
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
3. **sync follow-up: extract `move-bookmark` command.** The
   "put the bookmark / `@` where it belongs" step at the end
   of sync (reposition logic) is useful standalone — e.g. the
   t1B scenario where `main` is right but `@` isn't on it —
   and deserves an honestly-named command instead of a mode.
   - `vc-x1 move-bookmark` (name open): no fetch; move `@`
     (and optionally the bookmark) onto a target under the
     same safety rules as sync's reposition step.
   - Sync's final step becomes a call to the same logic.
   - Follow-up to the 0.67.0 single-mode sync cycle.
4. **sync follow-up: push preflight in-process; drop
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
5. **validate-numbering: rename the pair, check all
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
6. **pre-commit: single rule (no docs skip) + doc validators.**
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
7. **vc-x1 push: validate body opens with an intro paragraph.**
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
9. **single-field `options_flags` leaves → `value` field.**
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
10. **`por → dual` conversion.** Attach a `.claude`
    companion repo + `.vc-config.toml` to an existing por
    workspace; emit cross-links going forward. Manual
    setup on an external por workspace (2026-05-14)
    proved arduous; this should be a routine subcommand.
    Design stub in [[1]] § 2.
11. **`validate-desc` / `fix-desc` por equalization.**
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

_Migrated to [done.md](done.md) on 2026-07-14 (0.51.0–0.65.2 batch)._

- feat: reposition @ onto synced bookmark (0.66.0) — after a successful `--no-check` sync, `@` is repositioned onto the just-synced bookmark: code repo `jj new <b>` when clean (or `--rebase`/prompt-gated rebase when dirty; left in place when diverged/ahead), `.claude` always `jj new main` (or errors when `@-` is off main), all as a final pass *outside* the `op_restore` revert region; replaces `ensure_at_on_main`; new `--rebase` flag; README `### sync` docs + examples [[20]]
- feat: single-mode sync + revert command (0.67.0) — plain `vc-x1 sync` is one atomic operation (fetch, converge bookmark, reposition `@`; `--no-check` gone, `--check` a hidden deprecated alias for push preflight); failures stop for inspection with each repo's pre-sync op id persisted to `.vc-x1/sync-state.toml`; new `vc-x1 revert` restores from the snapshots; TDD via the two-clone `tests/cli_sync.rs` regression test of the t1A/t1B scenario [[21]]
- docs: todo cleanup + trapezoid entries (0.67.1) — push-related todos reshaped around the trapezoidal (merge non-ff) workflow: new #1 bookmark-invariant fix and #2 push pause point; "record uncovered code commits (N:1)" re-scoped to code worked outside vc-x1; `push --squash` demoted to todo-backlog.md; cycle-protocol.md push-wrapper list synced [[22]]
- feat: pin bot repo to main (0.68.0) — `--bookmark` is code-repo-only in push and sync; the session repo's side of every step (tracking preflight, classify/act, `bookmark-set` — renamed from `bookmark-both` — `finalize --push`, completion sanity) is pinned to `main`; plus two mid-cycle sync fixes: `reposition_session` no-ops when `@-` is the `main` tip, and the clean case prints one `nothing to sync` summary line [[23]]
- docs: diagnose silent session-push loss (0.68.1) — Bugs #1 root-caused: push's detached finalize child is killed at sandbox teardown before its delayed squash/push runs, so bot-run pushes never push `.claude`; diagnosis recorded in bugs.md, fix design queued as Todo #1 (inline session push + preflight backstop + finalize as the user's empty-@ tidy-up); 0.68.0 chores `Commits:` backfilled [[25]]

# References

[0]: AGENTS.md#prose-form
[1]: /notes/design-cli/por-dual-parity.md
[10]: /notes/forks-multi-user.md
[13]: /notes/chores/chores-12.md#docs-pordual-parity-design-0610
[20]: /notes/chores/chores-13.md#feat-reposition--onto-synced-bookmark-0660
[21]: /notes/chores/chores-13.md#feat-single-mode-sync--revert-command-0670
[22]: /notes/chores/chores-13.md#docs-todo-cleanup--trapezoid-entries-0671
[23]: /notes/chores/chores-13.md#feat-pin-bot-repo-to-main-0680
[24]: /notes/bugs.md
[25]: /notes/chores/chores-13.md#docs-diagnose-silent-session-push-loss-0681
