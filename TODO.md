# Todo

This file contains near term tasks with a short description
and reference links to more details.

Intro paragraphs in `## Todo` and `## Bugs` should begin every
line with 1 leading space so they don't match the `^\d+\. `
pattern that locates numbered entries; 2 or 3 spaces also work.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder".
Each rung is prepended with its commit reference — a literal
`[[N]]` placeholder until the commit is pushed, then backfilled
to a real file-local `[[n]]` ref (same pattern as the chores
As-built rungs):
   - [[N]] 0.xx.y-0 blah (done)
   - [[N]] 0.xx.y-1 blah blah (current)
   - [[N]] 0.xx.y-2 blah blah blah
   - [[N]] 0.xx.y close-out and validation

_No cycle currently in progress._

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
   - The remaining stages are the individual
     "Refactor stage: …" entries below (listed in program
     order; the doc owns execution order, per-stage status,
     and design — DRY facade shipped at 0.73.0).
2. **Refactor stage: repo registry.** Drop the root-anchored
   `[workspace]` path grammar: values become ordinary paths
   (relative to the config file's dir, or absolute —
   allowed but docs discourage), resolved agreement replaces
   the identical-block invariant, ochid prefixes become
   registry labels (URLs later — the local-path half of
   "ochid: bot-repo location qualifier" below), and the
   section is renamed ("workspace" is overloaded — jj itself
   has `jj workspace`). Decided 2026-07-24; see
   [the stage](notes/refactor-20260716.md#stage-repo-registry).
   Right after facade owns topology, so the schema settles
   in one migration wave and de-gitify init builds on it.
3. **Refactor stage: de-gitify init.** Replace init's
   strip-jj → git-push → re-colocate flow with the verified
   jj-only sequence; see
   [the stage](notes/refactor-20260716.md#stage-de-gitify-init).
   After facade owns topology.
4. **Refactor stage: split push.rs.** Extract `push/state.rs`
   so the jj-lib migration reviews cleanly — built and parked
   on `support-trapezoid-commits` (0.72.0-1), replay or redo;
   see
   [the stage](notes/refactor-20260716.md#stage-split-pushrs).
5. **Refactor stage: stateless push.** Retire the push state
   file — derive resume from repo reality; see
   [the stage](notes/refactor-20260716.md#stage-stateless-push).
   After split push.rs.
6. **Refactor stage: jj-lib migration.** Facade internals and
   mutations move in-process; the index-lock retry becomes
   ours; see
   [the stage](notes/refactor-20260716.md#stage-jj-lib-migration).
   After split push.rs and stateless push.
7. **Refactor stage: push body-intro validation.** Validate
   the commit body opens with a non-dash intro line, with a
   clear error; see
   [the stage](notes/refactor-20260716.md#stage-push-body-intro-validation).
   After the jj-lib migration (program order).
8. **Refactor stage: trapezoid close-out.** `push --merge
   [<base>]` — the native trapezoid close-out; design settled
   in the stage notes; see
   [the stage](notes/refactor-20260716.md#stage-trapezoid-close-out).
   After stateless push (no state-file growth) and jj-lib.
9. **Restructure templates: single template repo + fixed bot
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
10. **ochid: bot-repo location qualifier.** An ochid is
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
11. **Version-number protocol is fragile — versions are
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
12. **sync follow-up: extract `move-bookmark` command.** The
    "put the bookmark / `@` where it belongs" step at the end
    of sync (reposition logic) is useful standalone — e.g. the
    t1B scenario where `main` is right but `@` isn't on it —
    and deserves an honestly-named command instead of a mode.
    - `vc-x1 move-bookmark` (name open): no fetch; move `@`
      (and optionally the bookmark) onto a target under the
      same safety rules as sync's reposition step.
    - Sync's final step becomes a call to the same logic.
    - Follow-up to the 0.67.0 single-mode sync cycle.
13. **sync follow-up: push preflight in-process; drop
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
14. **validate-numbering: rename the pair, check all
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
15. **pre-commit: single rule (no docs skip) + doc validators.**
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
16. **vc-x1 push: record uncovered code commits (N:1 code↔bot).**
    Today push assumes 1:1 symmetric WC commits with shared
    title/body. The interop / adoption scenario breaks that:
    the code side is worked single-repo style (commit +
    `jj git push` / `git push`, no `vc-x1 push` in the loop),
    so no bot pairings exist — one bot commit then records
    every code commit not yet covered by a prior `ochid:`,
    via a multi-line `ochid:` per the design in [[1]].
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
17. **Run validate-bot at every vc-x1 invocation
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
18. **README: audit flag tables and examples against the
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
19. **Shared-doc sync: As-built ladder rungs carry `[[N]]`
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
20. **Shared-doc sync: per-commit chores convention.**
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
21. **config: extract flag-backed key descriptions from Clap.**
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

_Migrated to [done.md](notes/done.md) on 2026-07-23 (0.69.1–0.71.0 batch)._

- refactor: facade owns topology — repo resolution is facade
  state: the symmetric `work`/`bot` `[workspace]` schema
  (identical block on both sides, side detection by location,
  `.bot` as new-init default only), every `.claude` literal
  resolved from config, dual-entry coherence preflight, the
  validate-desc/fix-desc por equalization, and the `config`
  command's positional target; third stage of the jj refactor
  program [[6]]

- refactor: hygiene riders — the work/bot terminology
  stragglers swept (~380 identifier sites, `Side::Work`,
  `ConfigRole`, narration labels, test remotes); the `-s`
  scope keyword renamed `code` → `work` outright (no alias —
  unreleased); the six single-field `options_flags` leaves
  adopt the `value` field shape with clap ids pinned, and
  clone.rs's drifted inline `dry_run` folds onto the leaf;
  second stage of the jj refactor program [[2]]

- docs: notes rework + config refresh — `.vc-config.toml`
  (both sides) adopts init's generated optional-keys block;
  jj-tips.md re-syncs with the template (reclassified as
  pedagogy, not history); template-restructure design
  promoted to Todo #10 with `.bot` / symmetric-schema
  decisions folded into the refactor program; new Idea:
  chores retire into a session index; bot repo seeded with
  LICENSE-* / README.md from vc-x1-bot-repo-template [[3]]

- docs: adopt new template repo names — live mentions of
  `vc-template-x1`(.claude) swept to `vc-x1-work-repo-template`
  / `vc-x1-bot-repo-template` (AGENTS.md byte-identical with
  the template again; README init examples now pass an
  explicit `CODE,BOT` pair); historical records keep the old
  name [[4]]

- refactor: DRY jj facade — one typed facade (`src/jj.rs`)
  for every read-only jj query spawn (log templates +
  bookmark listings), the tracking and ochid trailer parsers
  unified beside it, and the test fixture helpers deduped to
  one copy per crate; first stage of the jj refactor
  program, worked on `refactor-vc-x1` with main parked at
  the 0.71.0 tip [[5]]

# References

[1]: /notes/forks-multi-user.md
[2]: /notes/chores/chores-14.md#refactor-hygiene-riders
[3]: /notes/chores/chores-14.md#docs-notes-rework--config-refresh
[4]: /notes/chores/chores-14.md#docs-adopt-new-template-repo-names
[5]: /notes/chores/chores-14.md#refactor-dry-jj-facade
[6]: /notes/chores/chores-14.md#refactor-facade-owns-topology
