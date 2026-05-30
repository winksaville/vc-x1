# Chores-12

Continuation of `chores-11.md` (which is closed at `0.57.0` —
the `--merge` todo entry cycle). This file covers the `0.58.0`
cycle onward. Reference numbering is file-local — see
[`CLAUDE.md`](../../AGENTS.md#reference-numbering); chores-12
starts at `[1]`.

## refactor: notes/todo restructure (0.58.0)

Commits: [[1]]

`notes/todo.md` is too large for routine reads and its
`## Todo` intro duplicates `notes/README.md`.

### As-built ladder

- 0.58.0-0 Preparation
  - backfill `0.57.0` `Commits:` ref in chores-11
  - bump Cargo.toml to `0.58.0-0`
  - open chores-12 with bare section header
  - add minimal `## File reads` to CLAUDE.md
  - populate `## In Progress`
- 0.58.0-1 add `## Priorities` + trim `## Todo` intro
- 0.58.0-2 move `## Bugs` → `notes/bugs.md`; pointer in todo.md
- 0.58.0-3 extract `notes/todo-backlog.md` (prioritized entries
  stay in todo.md; references re-packed per file)
- 0.58.0-4 expand CLAUDE.md `## File reads` to bugs.md / chores
- 0.58.0-5 codify in CLAUDE.md: chores title-only during cycle,
  In Progress moves into chores at close-out, problem+plan
  shape for In Progress / chores intros / Todo entries
- 0.58.0 close-out: move In Progress block into chores-12;
  `## Done` entry; update `notes/README.md`

### Outcome

Cycle grew from 3 planned commits (intro trim + Priorities +
bugs split) to 6 as the work surfaced two related concerns:
the bot's routine read cost (handled by extracting
`todo-backlog.md` and the CLAUDE.md `## File reads` rule),
and the chores/In-Progress duplication during cycles (handled
by codifying the new "chores title-only during cycle, move at
close-out" mechanic and the problem+plan shape).

`notes/todo.md` is now sized for cheap routine reads — the
head limit=60 covers intro + `## In Progress` + `## Priorities`,
the live surfaces. The split infrastructure (`notes/bugs.md`,
`notes/todo-backlog.md`, re-packed per-file refs) makes
future similar splits cheap.

## docs: extract cycle protocol (0.59.0)

Commits: [[2]]

`notes/cycle-protocol.md` becomes the canonical,
self-contained home for the cycle workflow. CLAUDE.md
keeps a 10-line pointer. Originally scoped as "verbatim
relocate"; opened up into a top-to-bottom iterative
tightening of the extracted protocol.

Late-stage design (ochid invariant + bot-repo rules +
squash gating + cross-repo migration + Ideas-aware
Preparation/Close-out) was planned for `-2` but
deferred at close-out. The rules were exercised
manually via Option F (app squash + bot-side trailer
rewrite + force-push), establishing the recipe that
`vc-x1 push --squash` (Todo #2, P1) will automate.

### As-built ladder

- 0.59.0-0 Preparation — backfill 0.58.0 chores
  `Commits:` ref; bump Cargo.toml; capture 0.58.0
  follow-up Todo (`vc-x1 push` N:1 P1); refresh
  Priorities.
- 0.59.0-0.1 capture design notes for `-1` as `###`
  subsections (see existing subsections below — they're
  the early-stage design that got deferred when 0.59.0
  pivoted to squash close-out).
- 0.59.0-1 Extract + tighten cycle protocol —
  `notes/cycle-protocol.md` (504 lines after iterative
  tightening); `notes/substep-protocol.md` recipes
  folded as `## Sub-cycle ladders`; CLAUDE.md `## Cycle
  Protocol` (~390 lines) → 10-line pointer;
  `init-clone-refactor-conflict.md` cross-ref updated;
  2 entries added to `notes/todo-backlog.md` + 42
  renumbered.
- 0.59.0 Close-out via squash + Option F —
  `## Ideas` section added to `notes/todo.md`
  (chores-convention-relocation entry #1; deferred
  codification entry #2); symmetric-squash captured as
  Todo #2 (P1); close-out bookkeeping (In Progress →
  chores; `## Done` entry); app squash; bot-side
  `jj describe` of `af60f979` (replace two `ochid:`
  trailers with one to squashed chid + rewrite note);
  force-push `.claude/main`; push app `main`.

### Outcome

cycle-protocol.md landed at significantly higher quality
than originally scoped — the "verbatim relocate" plan
opened the door to a top-to-bottom tightening pass that
improved structure throughout (phase-first overview;
`###` subsections; Chores sections + Commit description
extracted as their own `##` sections; Iterative work
pattern captured; chid/ochid defined locally).

First squash close-out via the manual Option F recipe;
first cycle to rewrite a published bot-side commit's
description. The codification follow-up that would make
the recipe binding (and surface the ochid invariant in
cycle-protocol.md) is captured as `## Ideas` entry #2.

### Squash carve-out (for CLAUDE.md `### Pushing`)

Default = merge for any multi-commit cycle. Squash = explicit
opt-in for cycles whose sub-steps are intermediate validation
points rather than distinct landings. Every cycle is ≥3
commits (Preparation + ≥1 Work + Close-out), so the
"merge of one parent" edge case is moot.

### Close-out follow-up capture (for CLAUDE.md `### Close-out`)

At close-out, write any follow-up Todos surfaced during the
cycle to `notes/todo.md > ## Todo` (if prioritized) or
`notes/todo-backlog.md` (if long-tail). List them in the
close-out commit body so the additions are reviewable.

### Close-out surfaces follow-ups (meta-note, for `## Cycle Protocol`)

Close-out often surfaces follow-up items that became visible
only by doing the cycle's work. Capture them at close-out
(per the rule above) so they survive `/exit` between cycles
— sessions don't carry conversational context across `/exit`.

## docs: consolidate notes conventions (0.60.0)

Commits: [[3]]

Bot is the primary writer of notes files, so the
file-format conventions belong in CLAUDE.md as the single
source of truth. Three notes-file sections (`Todo format`,
`Reference numbering`, `Retiring Done entries`) move from
notes/README.md into a new `## Notes file conventions`
umbrella in CLAUDE.md, alongside the existing
`## Chores conventions` umbrella; the `[[N]]` citation
rule duplicated between CLAUDE.md `## Notes references`
and notes/README.md `## Todo format` collapses into one
place.

### Cargo cycle discoverability

Mid-cycle the bot lost track of where the per-commit
cargo cycle (`fmt` / `clippy` / `test` / `install`)
lived. It had been buried inside `notes/cycle-protocol.md
> ## Per-commit flow > step 4` since the 0.59.0
extraction, with no direct surface in CLAUDE.md or
notes/README.md — and README.md `## Contributing` still
listed seven `CLAUDE.md#…` anchors that had been removed
in 0.59.0 (only `#code-conventions` still resolved).
Cycle scope expanded to fix all three: CLAUDE.md
`## Cycle Protocol` now names the cargo cycle inline;
notes/README.md `## Workflow and conventions` breaks
cycle-protocol.md into explicit pointers (Cycles /
Per-commit flow / Commit description / Pushing);
README.md `## Contributing` is rewritten against current
anchor homes.

### As-built ladder

- 0.60.0-0 Preparation — backfill 0.59.0 chores
  `Commits:` ref; bump Cargo.toml; pick up the "notes
  conventions → CLAUDE.md" Idea into `## In Progress`;
  open chores section.
- 0.60.0-1 Consolidate notes-file conventions in
  CLAUDE.md — three sections moved into the new
  umbrella; `[[N]]` duplicate dropped; chores files'
  reference-numbering pointer redirected to CLAUDE.md;
  `cycle-protocol.md` broken anchor fixed; cargo cycle
  surfaced at CLAUDE.md + notes/README.md; README.md
  `## Contributing` rewritten.
- 0.60.0 Close-out — chores narrative; `## Done` entry +
  `[12]:` ref; `## In Progress` reset.

## docs: por/dual parity design (0.61.0)

Commits: [[4]]

Design cycle for T5 (`por/dual parity + dual → por`).
Scope grew over eight commits from an audit to a full
CLI design — the audit doc became the canonical record
for both the parity divergences (where dual gets
privileged code paths over por) and the negotiated
designed surface (feature axes, decisions per axis,
resolution chain with baked-in default, subcommand ×
parameter matrix, copying mechanism for file seeding).
Output is input for 0.62.0+ equalization cycles. No
code changes land in 0.61.0 itself.

The cycle started as "audit + ranked Todos for future
fixes" and grew when the audit's findings made clear
that defaulting topology (the user's original framing)
wouldn't be safe before equalizing the underlying paths,
and that `--por` itself was a tangled bundle of
independent feature axes (topology, `.vc-config.toml`
write, remote, privacy, copying, scaffolding) rather
than a single toggle. The In Progress block (moved
below) records the problem statement and the as-built
ladder.

By close-out the deferred proposal had in fact landed:
the negotiated design exposes `[default].topology =
"single" | "dual"` in the user-config connections
(`por-dual-parity-audit.md` → "Connections to user-
config") — the very knob the user opened the cycle with.
So the bot's pushback was about *sequencing* (don't ship
the default before the underlying paths are equalized),
not about the idea; the cycle delivered the user's
original ask, just through a more thorough audit-first
process. Recorded here so the per-axis Decisions blocks
aren't read as if the design emerged from the bot's
analysis alone.

### Problem statement (preserved from In Progress)

Audit where `dual` (code + `.claude/` companion, cross-
linked by `ochid:`) gets privileged code paths vs `por`
(plain single repo). Goal: produce a parity-gap document
that drives future parity-equalization cycles. T5 in
`## Todo` is the parent design item; this cycle is the
audit step that precedes any code-level equalization.

The user proposed starting from `~/.config/vc-x1/config.
toml` to make the topology choice user-configurable; the
bot counter-proposed that a defaults knob shouldn't ship
before the underlying paths are equalized, and the user
agreed — audit first, equalize next, then expose
defaults.

### Scope arc

The cycle's scope expanded mid-flight three times — each
time a new pass revealed something the prior pass didn't:

- **Audit walk (`-1`)** found that `push` is fully dual-
  baked, `validate-desc` / `fix-desc` are dual-required
  outliers, and `sync` already implements the topology-
  neutral pattern via `default_scope` / `scope_to_repos`.
- **Commonality (`-2`)** inverted the view and showed
  that the codebase *already has* the right pattern;
  half the subcommands route through it. The "por is
  bolted on" framing inverted: por is bolted on
  everywhere the scope pattern wasn't applied.
- **Feature axes (`-3`)** decomposed `--por` from a
  topology toggle into six axes: Topology,
  `.vc-config.toml` write, Remote, Privacy, Copying,
  Scaffolding. Plus a 4-layer resolution chain (CLI >
  ENV > Local > Global) — later extended to 5 layers
  with a baked-in default at the bottom.
- **`.vc-config.toml` write collapse + copying stub
  (`-4`)** showed that the `.vc-config.toml` write axis
  is degenerate
  post-Topology, and the broader file-copying mechanism
  (`--init-from*`) subsumes both `--config` and
  `--gitignore` flags. `notes/design-cli/copying.md`
  captures the design.
- **`design-cli/` subdir (`-5`)** grouped the
  accumulating CLI design notes (copying.md,
  por-dual-parity.md, and the cycle's audit doc) under
  one home.
- **Decisions + matrix (`-6`)** captured per-axis
  negotiation outcomes as `**Decisions (0.61.0):**`
  blocks inline, plus the subcommand × parameter matrix
  the user requested.
- **Label rename + ledger + finalize fix (`-7`)** dropped
  `A1`–`A6` prefixes for descriptive axis names, added
  a clean acronym-to-axis ledger to the matrix, and
  acknowledged at the top of the doc that this is a
  pre-implementation snapshot — the final shape will
  differ.

### Mid-cycle reversals worth noting

Two negotiated reversals shaped the final design:

- **Topology floor: strict-explicit-required → default-
  dual.** Initial decision was "no implicit default;
  error if not specified." User reframed to "sensible
  defaults where natural, errors only for user-specific
  keys" — landing default-dual as the natural primary
  use case. Cascaded to Privacy (default-public),
  Remote (default-github-create), Copying (default-
  none).
- **Baked-in default config as bottom-of-chain.** Closes
  the "what does the error message say at the bottom"
  question for natural-default axes — there's always a
  fallback from the binary's `default-config.toml`. New
  `vc-x1 config dump` subcommand exposes it for users
  to save and modify.

### Status note (in the audit doc top)

The audit doc carries an explicit "this design is pre-
implementation; the final shape will differ" note. Future
readers (including the bot in 0.62.0 cycles) should treat
the design as a starting position, update it when code
reveals divergence, and not bend code to fit stale
sketches.

### As-built ladder

- 0.61.0-0 Preparation — backfill 0.60.0 chores
  `Commits:` ref; bump Cargo.toml; pick up T5 into
  `## In Progress`; open chores section; lay out the
  audit scope.
- 0.61.0-1 Audit walk — produce
  `notes/design-cli/por-dual-parity-audit.md` with 8
  sections inventorying dual-privileged code paths
  (per-area: files touched, divergence, severity,
  equalization sketch).
- 0.61.0-2 Commonality analysis — append `## Commonality`
  inverting the view (per-subcommand shared / dual-only
  / por-only buckets, ranked equalization candidates).
- 0.61.0-3 Feature-axes decomposition — append
  `## Feature axes` (six axes with current/target
  states); 4-layer resolution chain (CLI > ENV >
  Local > Global > Error); escape hatches; env-var
  table; por's view of the chain.
- 0.61.0-4 `.vc-config.toml` write collapse + copying
  stub — `.vc-config.toml` write axis collapsed under
  Topology decisions;
  `notes/design-cli/copying.md` design stub captures
  the broader file-copying mechanism that subsumes
  `--config`, `--gitignore`, and `--use-template`.
- 0.61.0-5 `design-cli/` subdir — group three CLI design
  notes (copying, por-dual-parity, the audit doc) into
  `notes/design-cli/`; update cross-references; leave
  historical chores untouched.
- 0.61.0-6 Captures + matrix — `**Decisions (0.61.0):**`
  blocks under each axis section; new
  `### Subcommand × parameter matrix` (14 subcommand
  rows × 7 parameter families with ledger + legend +
  footnotes).
- 0.61.0-7 Label rename + ledger + status — drop
  `A1`–`A6` prefixes for descriptive axis names; refactor
  matrix parameter-families intro into clean ledger
  table; correct `finalize` matrix row (T runtime + SC
  for standalone use); add `## Status` note framing
  this as pre-implementation.
- 0.61.0 Close-out — Resolution-chain rewrite for the
  baked-in-default 5-layer model + two-class principle;
  Gap-list refresh (14 implementation-cycle gaps for
  0.62.0+ to seed Todos from); In Progress → chores
  migration; `## Done` entry; `## In Progress` reset.

### Outcome — what's seeded for 0.62.0+

The refreshed Gap list in the audit doc carries 14
implementation gaps. The close-out commit promotes one
into `## Todo` (T7: `validate-desc` / `fix-desc`
equalization — cheapest concrete equalization, validates
the topology-from-config rule via prototype). The rest
live in the audit doc's Gap list for future Preparation
passes to pick up.

## docs: apply max review #1 (0.62.0)

Commits: [[5]]

Apply the accepted items from the **max review #1**
working list to the por/dual parity design (the
[audit doc](../design-cli/por-dual-parity-audit.md)) and
the [copying stub](../design-cli/copying.md). The working list
(`max-review-1.md`) was seeded at `0.62.0-1` and drained
one item per `-N` commit; fully applied by `-10`, it was
**retired (deleted)** at close-out — git history holds the
original review. Concern #1 and #2 (Topology) had been
applied once in an abandoned-during-rebase `0.61.0`
snapshot (`604fb9e8`); this cycle re-applied them cleanly
with the rest.

The review's Strengths (gone with the working list, so
recorded here): the three-pass methodology (audit →
commonality → axes), the genuine axis decomposition of
`--por`, the earned `.vc-config.toml`-write collapse, the
two-class "defaults where natural / errors where
ambiguous" principle, T7-only Todo promotion, and the
"implementation will diverge" status framing.

- **Concern #1 — runtime `--por` semantics** (0.62.0-2).
  Three doc-text spots (Topology Decisions "Runtime
  override" bullet, "Por's view of the chain" bullet,
  matrix Topology-column sub-bullet) read runtime
  `--por` as code-privileged (`→ Scope([Code])`, "just
  my code"). Rewrote all three to the target-relative
  rule: `--por` declares the target repo (`.` or
  `-R/--repo <path>`) single and suppresses sibling
  discovery symmetrically across code and bot — it
  doesn't privilege a side.
- **Concern #2 (Topology) — `--mode=<single|dual>` surface**
  (0.62.0-3). Concern #2 flagged the contradictory "exactly-one-of
  `--por` / `--dual` with default-dual" phrasing. Rather
  than reword, replaced the two boolean flags with one
  value-bearing flag `--mode=<single|dual>` (default
  `dual`, `single` ≡ por, optional `s` / `d` aliases) —
  a single flag can't conflict with itself, so the
  contradiction dissolves. Propagated across the design
  sections (Topology Decisions, Por's view, matrix +
  quick-reference, Mapping section,
  [Gap #8](../design-cli/por-dual-parity-audit.md#gap-list-refreshed-for-0620));
  the runtime
  override from Concern #1 becomes `--mode=single`. Audit
  findings (`## 1`–`## 8`) keep `--por` — they describe
  today's code.
- **Concern #2 (Privacy) — `--visibility` + clone-row fix**
  (0.62.0-4). Resolves
  Concern #2's Privacy half the same way `--mode` did for
  Topology: Privacy axis adopts
  `--visibility=<public|private>` (default `public`,
  `--private` / `--public` kept as shortcuts),
  **`init`-only**. Empirically confirmed (`init -h`,
  `clone -h`, `clone.rs`) that `clone` never provisions a
  remote — so the matrix's clone row is corrected from
  `A/R ✓ Priv ✓` to `—` for both, with a footnote. Closes
  Concern #2.
- **Concern #3 — single `--init-from` (copying surface)**
  (0.62.0-5). Concern #3 flagged the six `--init-from*`
  flags as surface doubling. The fix went further — to one
  flag for every topology.
  - Dropped the `-recursive` variants: a directory operand
    always recurses. The review's `cp`-parity rationale
    was wrong (`cp` errors on a directory without `-r`);
    the honest reason is that a non-recursive directory
    copy is meaningless here (an empty directory), so
    nothing needs an opt-in to guard.
  - Dropped the per-side `-code` / `-bot` split too — that
    scope is the Topology axis `--mode=<single|dual>`
    already owns. A dual seed comes from one
    workspace-shaped source tree. Two different-origin
    sources route through the planned `por -> dual`
    conversion. Leaves `--mode` untouched.
  - Added `@<file>` manifest semantics: literal source
    paths, one per line, relative to the manifest's
    location, no nested `@` — keeping shell-side expansion
    the only globbing path.
  - Edits land in the `copying.md` stub. The audit doc's
    Copying Decisions summary tracks the surface change.
- **Concern #4 — `--repo none` × dual** (0.62.0-6).
  Concern #4 flagged that the Remote axis
  added `--repo none` without saying what it means under
  dual. Resolved as first-class: `init --mode=dual --repo
  none` creates both repos local with no remotes, matching
  the deferred-validation stance (`init` permissive,
  downstream enforces).
  - `sync` no-ops its remote steps. `push` / `finalize`
    error early and clearly when a side has no remote,
    instead of failing deep in the workflow.
  - Remotes can be added later (`por -> dual` or a future
    add-remote). One Decisions bullet added to the audit
    doc's Remote axis.
- **Concern #5 — list-typed-axis "wins" rule** (0.62.0-7).
  Flagged that "CLI replaces config" is non-obvious for
  *list-valued* axes (users from `PATH` / `LD_LIBRARY_PATH`
  expect a merge). Kept the replace semantics but added an
  explicit List-typed-axis rule sub-bullet to the Copying
  Decisions so it isn't re-litigated.
- **Concern #6 — gap-list ordering prerequisite**
  (0.62.0-8). The audit doc's
  [Gap list](../design-cli/por-dual-parity-audit.md#gap-list-refreshed-for-0620)
  is ranked by blast radius, but
  [Gap #7](../design-cli/por-dual-parity-audit.md#gap-list-refreshed-for-0620)
  (`default_scope` broken-dual detection) is a prerequisite
  for
  [Gap #9](../design-cli/por-dual-parity-audit.md#gap-list-refreshed-for-0620)
  (the copying mechanism, whose validation defers to the
  first downstream subcommand). Added a note that structural
  prerequisites override size ordering. Folded in unrelated
  todo captures (version-number-protocol Idea, pre-commit
  Todo).
- **Nits N1–N4** (0.62.0-9). Four small audit-doc edits:
  - N1 — `finalize` matrix row marked `✓*` with a footnote
    that its T/SC support is *latent* (body supports it, no
    caller surfaces it).
  - N2 — a `## Reading guide` at the top (jump-by-task
    anchors) for the ~1200-line doc.
  - N3 — reframed "Por's view of the chain" into a "When is
    topology consulted?" debugging cheat-sheet.
  - N4 — footnote that `validate-todo` / `fix-todo` are
    topology-blind by design (operate on `notes/`-family
    files, outside the workspace shape).
  - Also reframed Todo #1 (push should validate the body's
    intro paragraph) and seeded a `validate-numbering` Todo
    (generalize the single-file checker to all
    sequence-managed notes files).
- **Process observation — original user framing**
  (0.62.0-10). The deferred user-config topology default
  that *opened* the cycle did land in the design by
  close-out; added a loop-closure note to the `0.61.0`
  narrative (above) recording that the bot's pushback was
  about *sequencing*, not the idea.

### As-built ladder

- `0.62.0-0` Preparation — backfill `0.61.0` `Commits:`,
  bump version, pick up the review into In Progress.
- `0.62.0-1` Seed the `max-review-1.md` working list.
- `0.62.0-2` Concern #1 — runtime `--por` rewrite.
- `0.62.0-3` Concern #2 (Topology) → `--mode`.
- `0.62.0-4` Concern #2 (Privacy) → `--visibility` +
  clone-row fix.
- `0.62.0-5` Concern #3 — copying → single `--init-from`.
- `0.62.0-6` Concern #4 — `--repo none` × dual.
- `0.62.0-7` Concern #5 — list-typed-axis "wins" rule.
- `0.62.0-8` Concern #6 — gap-list prereq + fold captures.
- `0.62.0-9` Nits N1–N4 + reframe Todo #1 +
  validate-numbering Todo.
- `0.62.0-10` Process observation → chores narrative.
- `0.62.0` Close-out — flesh out this section, retire
  `max-review-1.md`, capture two Ideas.

### Outcome

- The max review #1 working list is fully applied and
  **retired**; git history is the record. All six concerns,
  four nits, and the process observation landed across
  `-2` … `-10`.
- Mid-cycle process fixes captured as Todos: push should
  validate the body's intro paragraph (reframed Todo #1); a
  single pre-commit rule with doc validators; a
  `validate-numbering` generalization of `validate-todo` /
  `fix-todo`.
- Two follow-up Ideas seeded: a code+conversation
  **provenance viewer** (the grander `vc` ambition), and a
  **design-cli doc restructure** (split the audit/design
  doc, refocus the parity stub to conversion) targeted at
  `0.63.0`.
- Out-of-band: `0.62.0-7` / `-8` ochid trailers were
  repaired — a duplicate on the app side and a bogus
  self-ref on the bot side, both from a hand-added `ochid:`
  line in the push body — via `jj describe
  --ignore-immutable` + force-push of both `main`s. Fix
  forward: `vc-x1 push` auto-stamps ochids, so push bodies
  omit them.

# References

[1]: https://github.com/winksaville/vc-x1/commit/a199d062ff6e "a199d062ff6e88b5e2d87d57551d1c60e75b073b"
[2]: https://github.com/winksaville/vc-x1/commit/e67e44b8e1c5 "e67e44b8e1c55b8e7c33087b8f2184df87181885"
[3]: https://github.com/winksaville/vc-x1/commit/41ef8842d885 "41ef8842d885a7713416a7321e2cd7ae67802b68"
[4]: https://github.com/winksaville/vc-x1/commit/258b24101900 "258b24101900d5784095775386e4962350ed3098"
[5]: https://github.com/winksaville/vc-x1/commit/8943992c6bb5 "8943992c6bb5f3afa3e59c09541aa47d37275067"
