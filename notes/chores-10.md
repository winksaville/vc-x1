# Chores-10

Continuation of `chores-09.md` (which is closed at `0.48.4` —
the `0.42.0`–`0.48.x` cycles). This file covers the `0.49.0`
cycle onward, plus a living **refactor-tracking** section (the
per-subcommand status — for the Context+Params port and the
options_flags extraction — that used to sit in `ARCHITECTURE.md`).

See [`../ARCHITECTURE.md`](../ARCHITECTURE.md) for what those
two refactors *are* and why; this file is the *how* and the
*live status*. Reference numbering is file-local — see
[`README.md`](README.md#reference-numbering); chores-10 starts
at `[1]`.

## Refactor tracking — Context+Params / options_flags

Two parallel refactors of the subcommand layer (see
`ARCHITECTURE.md`). Status as of the entries below; update in
place as cycles land.

### Context+Params port — `pub fn x(args: &XxxArgs)` → `pub fn x(ctx: &Context, params: &XxxParams)`

| Subcommand | Status |
| --- | --- |
| `init` | done (0.44.0) — worked example; `From<&InitArgs>` |
| `finalize` | done (0.46.0) — `TryFrom<&FinalizeArgs>` (fallible); `--log` onto `Context` |
| `symlink` | done (0.48.0-1) — `From<&SymlinkArgs>` |
| `clone` | done (0.48.0-2) — `From<&CloneArgs>` |
| `sync` | done (0.48.0-3) — `From<&SyncArgs>` |
| `validate-desc` | done (0.48.0-4) — `From<&ValidateDescArgs>` |
| `fix-desc` | done (0.48.0-5) — `From<&FixDescArgs>` |
| `push` | done (0.48.0-6) — `From<&PushArgs>` (collapses the two bookmark spellings) |
| `chid` | done (0.49.0-3) — introduces shared `CommonParams`; `TryFrom` |
| `desc` | done (0.49.0-4) — `TryFrom` |
| `list` | done (0.49.0-5) — `TryFrom` |
| `show` | done (0.49.0-6) — `TryFrom`; also parses `--files` → `FileLimit` at the boundary |

Out of scope for the ports (deferred until a real consumer
surfaces): typed errors, returned-outcomes-vs-`println!`,
`ProgressSink`, `Context` fields beyond `UserConfig` + `--log`.

### options_flags extraction — per-subcommand inline `#[arg]` → `src/options_flags/` leaves/bundles

| Subcommand(s) | Status |
| --- | --- |
| `init` | fully composed — `account` / `repo` / `scope` / the `provision` bundle / `use_template` / `config` |
| `finalize` | `--squash` → `squash` leaf; `--delay` / `--detach` / `--exec` / `--repo` / `--push` still inline |
| `chid` / `desc` / `list` / `show` | flatten `options_flags::common_args::CommonArgs` (relocated there at 0.49.0-1.1 — an inline-fields bundle, no per-flag sub-leaves; see [Bundle](../src/options_flags/README.md#architecture)); `-R`/`--repo` → `-s`/`--scope` leaf in 0.49.0-2 |
| `sync` / `push` / `clone` / `validate-desc` / `fix-desc` / `symlink` | mostly inline; the `--scope` retrofits queued in `todo.md` are the usual entry point |

## chore: open 0.49.0 — finish Migration A (0.49.0-0)

Commits: [[1]]

Multi-step. Finish Migration A: port the last four subcommands
`pub fn x(args: &XxxArgs)` → `pub fn x(ctx: &Context, params:
&XxxParams)`, same shape as the `0.48.0` sweep. `chid` / `desc`
/ `list` / `show` all `#[command(flatten)]` `common::CommonArgs`,
so the cycle also adds a shared clap-free `CommonParams` they
reuse. Mechanical; no behavior change.

(Superseded at 0.49.0-1 — the cycle was expanded to the full
"CommonArgs sweep", B-first; see the next section's `### Cycle
re-scope`. This section keeps the original narrow framing as the
record of what the 0.49.0-0 commit's plan was.)

### Per-step shape

- `common::CommonParams` (flat, clap-free): the resolved
  `DotSpec` + `Header` + raw `repos: Vec<PathBuf>`, with
  `impl From<&CommonArgs>` doing the `resolve_spec` /
  `resolve_header` work at the binary edge. The `..` parsing and
  `-l`/`-L` resolution move out of the subcommand bodies.
- `XxxParams`: flat struct embedding `CommonParams`, plus the
  subcommand's own fields — `list` adds `width: usize`, `show`
  adds `files: FileLimit` (the `--files` string is parsed at the
  boundary). `impl From<&XxxArgs>` (total) for `chid` / `desc` /
  `list`; `impl TryFrom<&ShowArgs>` for `show` (fallible —
  `FileLimit::parse`), mirroring `finalize`.
- `pub fn x(args)` → `pub fn x(_ctx: &Context, params:
  &XxxParams)`; `ctx` unused (uniform-signature placeholder, as
  in `symlink` / `validate-desc` / `fix-desc`).
- `main.rs` dispatch arm builds `Context::load(cli.log)` + the
  params; the `suppress_banner` match keeps reading
  `a.common.no_label` off the args (clap edge, unchanged).
- Tests: existing `XxxArgs` parse tests untouched; add a small
  "construct `XxxParams` directly" test per the worked-example
  precedent.

### Ladder (original — superseded)

Smallest first; `show` (the `TryFrom` + `FileLimit` parse) last.

- 0.49.0-0 plan + version bump + this section + todo ladder
  (current)
- 0.49.0-1 chid + introduce `CommonParams` in `common.rs`
- 0.49.0-2 desc
- 0.49.0-3 list
- 0.49.0-4 show
- 0.49.0 close-out — drop suffix, todo→Done, ARCHITECTURE.md
  Migration A table all-done (12/12)

This cycle is Migration A only. Migration B for these four
(folding `CommonArgs` into the `options_flags/` leaf model,
dropping the repeatable `-R`/`--repo` for the `--scope` path
form) stays the separate "CommonArgs sweep" todo.

### Per-substep contract

Per `notes/substep-protocol.md`: `cargo fmt` / `clippy
--all-targets -- -D warnings` / `test` / `install --path .
--locked` + retest before each commit; bump `Cargo.toml` at
sub-step start; flip todo ladder markers; pair commits across
both repos with ochid trailers.

## refactor: CommonArgs → options_flags (0.49.0-1.1)

Commits: [[2]]

The options_flags-extraction relocation, and where the cycle is
re-scoped (see `### Cycle re-scope`). Relocate `common::CommonArgs`
— the shared arg set for `chid` / `desc` / `list` / `show` —
from `src/common.rs` into `src/options_flags/common_args.rs`.
Inline fields, no per-flag sub-leaves (an "inline-fields bundle"
— see [Bundle](../src/options_flags/README.md#architecture);
rationale in `### Why not decompose into leaves`). Pure
relocation — `vc-x1 chid -h` etc. byte-identical.

- `CommonArgs` → `options_flags/common_args.rs` (`impl OptionFlagBundle`).
- `src/common.rs`: `CommonArgs` + `use clap::Args` dropped (the
  `for_each_repo` / `collect_ids` / `resolve_*` / `format_*`
  helpers stay).
- `chid` / `desc` / `list` / `show`: flatten the relocated
  struct; each gains the `//!` module docstring it lacked;
  bodies otherwise unchanged.
- `main.rs`: `suppress_banner` reads `a.common.no_label` for all
  four (back to pre-cycle).

`0.49.0-1.2` (the ARCHITECTURE.md slim + chores-10 reorg, own
section below) follows; the two are kept separate at close-out —
no squash, no extra `0.49.0-1` commit.

An earlier `0.49.0-1.1` decomposed `CommonArgs` into per-flag
leaves; backed out on review, amended in place to the relocation
above — see `### Why not decompose into leaves`.

### Why not decompose into leaves

The first take decomposed `CommonArgs` into per-flag
`options_flags/` leaves (`revision` / `commit_limit` /
`repo_label`) flattened into a `common_bundle` — mirroring
`init` + `provision_bundle`. Backed out:

- **No reuse.** `init`'s leaves are reused (`--dry-run` →
  `clone`, etc.) — a per-flag leaf is the unit of sharing. The
  four here share the *whole set*; none of `revision` / `limit`
  / `label` / etc. is reused elsewhere, so per-flag leaves buy
  nothing the shared struct doesn't already give.
- **`value`/`id` friction.** Two single-field-`value` leaves
  ([README](../src/options_flags/README.md)) collide on the clap
  arg *id* (derived from the field name) when co-flattened —
  `clap_builder` panics. The fix (`#[arg(id = …)]` on each) is
  the "not obvious when `id` vs `value`" sharp edge —
  self-inflicted by decomposing leaves that didn't need to be.

Rule of thumb:

- Leaf — when *that flag / small group* is genuinely shared at
  that granularity (`--dry-run`, `--squash`).
- Inline-fields bundle — when a *whole arg set* is shared by N
  commands but its parts aren't reused.
- Extract a leaf later, when a second consumer is real:
  `CommonArgs` then `#[command(flatten)]`s the extracted type
  (one definition, no duplication).

### Cycle re-scope (0.49.0-1)

0.49.0-0 opened a "finish Migration A" cycle for these four
subcommands; at 0.49.0-1 review it was expanded to the full
**"CommonArgs sweep"** (`chores-06` / `chores-07`'s long plan)
and re-ordered B-first so the Context+Params ports land once.
(The 0.49.0-0 commit/section keep the original framing; this
records the change. "Migration A" / "Migration B" were the
working names — renamed here to the Context+Params port and the
options_flags extraction; the frozen 0.48.x / 0.49.0-0
commit-recording sections keep the old names.) The three parts:

- **options_flags extraction** (0.49.0-1, sub-steps -1.1
  relocation, -1.2 docs reorg):
  - relocate `CommonArgs` → `options_flags/common_args.rs`
    (inline-fields bundle, no per-flag leaves);
  - move all four subcommands onto it;
  - remove `src/common.rs`'s `CommonArgs`.
- **`--scope`** (0.49.0-2) — `-R`/`--repo` → `-s`/`--scope`
  (`code|bot|code,bot|<path>`):
  - wires up `scope.rs`'s `parse_scope` + `Scope::Single` (both
    `#[allow(dead_code)]`, built in 0.42.0 for this) and
    `common.rs`'s `default_scope` / `scope_to_repos`;
  - `-R` dropped, not aliased; arbitrary multi-repo
    (`-R . -R .claude`) not preserved — per the 0.42.0 capture
    (`chores-06` 0.41.0-4); `--scope=<path>` covers single-repo,
    default `default_scope()`;
  - `for_each_repo` takes an already-resolved `Vec<PathBuf>`
    (comma-expansion + `["."]` default move to the boundary);
  - `init`'s `--scope` (the separate `ScopeKind` `code,bot|por`)
    left alone — the "por/dual parity" todo.
- **Context+Params port** (0.49.0-3..-6) — `XxxParams` + `fn
  x(&Context, &XxxParams)` ports against the post-B/-scope
  `CommonArgs`:
  - introduces the shared clap-free `CommonParams` (resolved
    `DotSpec` + `Header` + repos);
  - all four take a fallible `TryFrom<&XxxArgs>` (scope→repo
    resolution can error);
  - `show` also parses `--files` → `FileLimit` at the boundary.

Revised ladder (supersedes the 0.49.0-0 one):

- 0.49.0-1 options_flags extraction — relocate `CommonArgs` to
  `options_flags/common_args.rs`; this re-scope. Landed as two
  kept-separate sub-steps (no `0.49.0-1` close-out commit):
  - 0.49.0-1.1 the relocation + all four importers
  - 0.49.0-1.2 docs: slim ARCHITECTURE.md; start chores-10
- 0.49.0-2 `-R`/`--repo` → `-s`/`--scope` (wire `parse_scope` /
  `default_scope` / `scope_to_repos`; slim `for_each_repo`) —
  may split into `-2.N`
- 0.49.0-3 chid Context+Params port + introduce `CommonParams`
- 0.49.0-4 desc Context+Params port
- 0.49.0-5 list Context+Params port
- 0.49.0-6 show Context+Params port (`TryFrom`, `FileLimit` parse)
- 0.49.0 close-out — drop suffix, todo→Done (Context+Params port
  12/12 + CommonArgs sweep), README + ARCHITECTURE.md

## docs: slim ARCHITECTURE.md; chores-10 (0.49.0-1.2)

Commits: [[3]]

Acting on review feedback that `ARCHITECTURE.md` had drifted
into "how" / transient territory — per-subcommand status,
version-by-version tables, sub-step ladders. Pull that out so
`ARCHITECTURE.md` is "what + some why, generic" and the transient
tracking lives here. Also renamed the two cross-cutting
refactors: "Migration A" → the Context+Params port, "Migration
B" → the options_flags extraction (the frozen 0.48.x / 0.49.0-0
commit-recording sections keep the old names; a note in `###
Cycle re-scope` records the change).

- `ARCHITECTURE.md` — slimmed to "what + some why, generic":
  - kept: Overview, the two-layer section + rationale + `Naming`,
    a generic module map (kinds of module — no per-subcommand
    table, no version annotations), the Subcommand-model recipe,
    See-also;
  - the two refactor sections (`## args → Context + Params` /
    `## per-subcommand flags → src/options_flags/`) → short
    "what + why" + a pointer here for live status (the
    per-subcommand status table, the version-by-version "done"
    list, the "State today" bullets are gone).
- `notes/chores-10.md` — new (this file): the `## Refactor
  tracking` tables (out of `ARCHITECTURE.md`) + the `0.49.0-*`
  sections moved out of `chores-09.md`.
- `notes/chores-09.md` — the `0.49.0-*` sections removed; it now
  ends at `0.48.4` (all done/closed).
- `notes/README.md` — notes that `chores-09` is closed and
  `chores-10` is the active file with the tracking section.
- `notes/todo.md` — "Design:" link repointed to `chores-10.md`;
  the `0.49.0-1` sub-step ladder shows `-1.1` (the relocation)
  and `-1.2` (this), kept separate — see the `0.49.0-1.1`
  section.
- `src/options_flags/README.md` — the "Migration B" mention →
  "the options_flags extraction".

## chore: tidy todo + process rule (0.49.0-2.1)

Commits: [[4]]

Opens `0.49.0-2` (the `-R`/`--repo` → `-s`/`--scope` rollout for
`chid` / `desc` / `list` / `show` — design already in `### Cycle
re-scope`'s `--scope` bullet) with a small bookkeeping/process
pass the user asked for first.

- Two `## Todo` items in `notes/todo.md` duplicated the
  In-Progress "CommonArgs sweep" task — "Subcommand layer / CLI
  decoupling — remaining subcommands" (its remaining work *is*
  the In-Progress task; its done work is already in `## Done`)
  and "CommonArgs sweep — add `--scope=…`" (= the `0.49.0-2`
  step). Both removed.
- New CLAUDE.md rule (in `### Versioning`, plus a catch-line in
  the pre-commit checklist): a `## Todo` item is deleted when it
  goes `## In Progress` — see `### Process: delete a Todo item
  when it goes In Progress`.
- Cycle-start chores: backfilled the `0.49.0-1.1` / `0.49.0-1.2`
  chores `Commits:` refs (`[[2]]` / `[[3]]`); bumped `Cargo.toml`
  to `0.49.0-2.1`; expanded the `notes/todo.md` In-Progress
  ladder with the `-2.1` / `-2.2` sub-steps.

### Process: delete a Todo item when it goes In Progress

The `0.49.0` "CommonArgs sweep" In-Progress task absorbed three
`## Todo` items (the Context+Params port for the four, the
options_flags extraction, the `--scope` rollout) without those
items being removed — so `## Todo` carried zombie entries
describing work already underway. The fix is a rule, not a
one-off cleanup: when a `## Todo` item is picked up — its `##
In Progress` ladder created, or an existing ladder *re-scoped*
to absorb it — the entry is deleted in the same commit. `## In
Progress` is the sole record until close-out, then it moves to
`## Done`. A `## Todo` entry that duplicates current In-Progress
work is a process bug; the pre-commit checklist gets a
catch-line for it.

## feat: chid+co -s/--scope flag (0.49.0-2.2)

Commits: [[5]]

The code half of the `0.49.0-2` rollout (`0.49.0-2.3` sweeps
the docs). Replaces today's `-R .,.claude` / `-R . -R .claude`
multi-repo forms with a roles flag (`-s code | bot | code,bot`),
keeps `-R` for single-path operation, and lets the two compose
as a workspace-root override. See `### Cycle re-scope`'s
`--scope` bullet for the wider plan; the two `###` subsections
below capture the design pivots that landed in this commit.

### Design: `-R` and `-s` compose rather than conflict

The chores plan had `-R` dropped, with `--scope=<path>` covering
single-repo via `Scope::Single`. On review the design walked
through three iterations:

- **Drop `-R`.** `--scope=./foo` covers single-path via
  `Scope::Single`; one flag, the `./` prefix disambiguates from
  the role keywords.
- **Mutually-exclusive `-R` + `-s`.** Keep `-R` for paths, add
  `-s` for roles, `conflicts_with` between them. Familiar `-R`
  preserved; redundant on the path side.
- **Composing `-R` + `-s`** (chosen). `-R` overrides the
  workspace root (replaces `find_workspace_root()`), `-s`
  selects sides (replaces `default_scope(...)`). Together:
  `-R <ws> -s <roles>` resolves the roles within `<ws>`. Each
  flag alone reads as today's behavior: `-R foo` → `[foo]`, no
  flag → `[.]`. The net-new expressivity is the combined form
  (e.g. `vc-x1 chid -R ../foo -s bot` → `[../foo/.claude]`).
  `-s` is keyword-only today (`parse_scope_roles` rejects paths
  with a hint at `-R`); `-s <path>` and the `-s <path>,roles`
  workspace-root override are queued as `## Todo` (a future
  `Scope::RolesAt { root, sides }` variant, probably). "Drop
  `-R` once `-s` is established" is a separate `## Todo` for
  after the migration period — kept for backwards-compat now.

### Design: `CommonArgs::resolve_repos(&self)` helper

`common::resolve_repos(repo, scope)` takes `Option<&Path>` +
`Option<&Scope>` — the standard "borrowed unsized" convention
(`&Path`, not `&PathBuf`). At each call site that produces an
asymmetric `c.repo.as_deref()` / `c.scope.as_ref()` pair — four
times across the subcommand bodies, with another four to come in
`0.49.0-3..-6`'s `TryFrom<&XxxArgs>` impls. A method on
`CommonArgs` localizes the conversion ceremony to one place so
callers read `c.resolve_repos()?`, and a new
`notes/rust-idioms.md` carries the `as_deref` vs `as_ref`
explainer the doc-comment links to. The free function stays as
the reusable primitive (a future `finalize --scope` /
`push --scope` calls it directly, not through `CommonArgs`).

## docs: chid+co -s/--scope flag (0.49.0-2.3)

Commits: [[6]]

The docs half of the `0.49.0-2` rollout. Code shipped in `-2.2`
but `README.md` still showed the pre-`-R`-single-path multi-repo
forms (`-R .,.claude` / `-R . -R .claude`) — anyone reading the
user-facing docs would be steered into invocations that no longer
parse.

- `README.md` `### Multi-repo queries` rewritten to lead with
  `-s code,bot`; every example updated to the new form.
- `-R PATH` retained as the single-path escape hatch (`-R .`,
  `-R .claude`, `-R ../other`) and as the workspace-root
  override that composes with `-s` (`-R ../other -s code,bot`).
- Trailing paragraph forward-links to the `-s <path>` and
  `-s <path>,roles` future-Todos so readers see the deferred
  expressivity.
- `ARCHITECTURE.md` gains `resolve_repos` in the `common.rs`
  helper list with a one-line note on what it composes.
- `sync` flag table at the bottom of `README.md` left alone —
  `sync` still has the old repeatable/comma-list `-R`; its
  migration is queued under "Drop `-R` from `CommonArgs`".

## docs: unify prose form in CLAUDE.md (0.49.0-2.4)

Commits: [[7]]

Process drift caught while writing `-2.3`'s chores section: the
intro+bullets shape that governs commits, chores, todo, and doc
comments was codified in three places in CLAUDE.md — once inside
`### Doc comments…`, once inside `## Commit Message Style`, once
inside `### Chores section content`. The three copies drifted
small-but-real and made it harder to remember which surfaces the
shape covered (Done entries weren't mentioned anywhere). Unified
into one top-level `## Prose form` section the others reference.
See [§ Design: Prose form unification](#design-prose-form-unification).

- New `## Prose form` section in CLAUDE.md is the single source
  of truth: the shape (intro + bullets, ≤72 wrap), the list of
  surfaces it covers, and the bullet-content rules per surface.
- `## Commit Message Style` body bullets slimmed to reference
  `## Prose form`; commit-specific bits (50-col title rule,
  file-by-file bullets, source-of-truth claim) retained.
- `### Chores section content` paragraph slimmed to reference
  `## Prose form`; the no-edit-list rule and the conceptual-bullet
  emphasis retained.
- `### Doc comments on every file…` Shape paragraph removed in
  favor of a one-line reference; the clap-derive
  `verbatim_doc_comment` note retained.
- Anchors on the three existing sections unchanged so `notes/`
  cross-refs don't churn.

### Design: Prose form unification

Three alternatives considered before picking the top-level
section:

- **Expand the existing Shape rule's list** to add commits and
  Done. Smallest change; leaves the rule buried in `### Doc
  comments…` where it's hard to find when writing a commit body
  or chores section.
- **Cross-link three definitions to each other.** Each site
  redefines the shape but pointers run between them. Reads
  consistent but still drifts: three copies of the same
  paragraph still need three edits to stay in sync.
- **Promote to a single top-level `## Prose form` section**
  (chosen). The shape is defined once; the consumer sites carry
  only the surface-specific add-ons (50-col title for commits,
  file-by-file bullets for commits, `Commits:` line + design
  `###`s for chores, etc.). Anchors stay stable on the existing
  sections so notes/ refs don't churn.

The chosen form also distinguishes bullet *content* per
surface — commit-body bullets are file-by-file (the source of
truth for the mechanical edit list — `git show` is the record);
chores / todo / done bullets are conceptual (design points,
structural notes — never a copy of the commit's edit list);
doc-comment bullets are whatever structure fits.

## refactor: chid → Context+Params (0.49.0-3)

Commits: [[8]]

First of the four `chid` / `desc` / `list` / `show` Context+Params
ports (the chid leg of the cycle's Context+Params half — see
`### Context+Params port` at the top). The Args/Params layering
arrives at the read-only commit-query subcommands and introduces
the shared `CommonParams` the next three reuse.

- `common::CommonParams` (flat, clap-free): resolved `DotSpec` +
  `Header` + `repos: Vec<PathBuf>`. `impl TryFrom<&CommonArgs>`
  does the `resolve_spec` / `resolve_header` / `resolve_repos`
  work at the binary edge — fallible because `resolve_repos`
  can fail (workspace lookup, path issues).
- `chid::ChidParams`: flat struct embedding `CommonParams`,
  nothing else (chid has no fields beyond `CommonArgs`).
  `impl TryFrom<&ChidArgs>` delegates to
  `CommonParams::try_from`.
- `pub fn chid(_ctx: &Context, params: &ChidParams)` — `ctx`
  unused (uniform-signature placeholder, as in
  `symlink` / `validate-desc` / `fix-desc`).
- `main.rs` dispatch builds `Context::load(cli.log)` +
  `ChidParams::try_from(&chid_args)` (mirrors the `Finalize`
  arm, the other `TryFrom` site).
- Tests: existing `ChidArgs` parse tests untouched; new
  `params_from_args_defaults` exercises the boundary
  resolution.

### Design: error type and import direction

Two small calls worth recording.

- **Error type `String`** on `TryFrom<&CommonArgs>` matches
  `finalize` (the existing fallible precedent), not
  `Box<dyn std::error::Error>`. The actual fallibility is
  `resolve_repos`'s `Box<dyn Error>` — coerced via
  `map_err(|e| e.to_string())` at the boundary. `String` keeps
  the param-construction error surface uniform across the four
  ports.
- **`common.rs` now imports
  `options_flags::common_args::CommonArgs`** to write the
  `TryFrom` impl — a new edge from `common` to `options_flags`.
  `options_flags/common_args.rs` already imports
  `crate::common::resolve_repos`, so the two modules now
  cross-reference within the crate. Not a layering inversion:
  `CommonParams` is the resolved/domain side, `CommonArgs` is
  the CLI-surface side, and the conversion is a one-way edge
  from CLI to domain. Defining the impl next to `CommonParams`
  (the target) is the natural place.

## refactor: desc → Context+Params (0.49.0-4)

Commits: [[9]]

Second of the four Context+Params ports; uses the shared
`CommonParams` introduced in `-3`. Mechanical follow-through —
no new design, same shape as chid (no fields beyond `CommonArgs`,
so `DescParams` is just a `CommonParams` wrapper).

- `desc::DescParams`: flat struct embedding `CommonParams`,
  nothing else. `impl TryFrom<&DescArgs>` delegates to
  `CommonParams::try_from`.
- `pub fn desc(_ctx: &Context, params: &DescParams)` — `ctx`
  unused (uniform-signature placeholder).
- `main.rs` dispatch builds `Context` + `DescParams` (matches
  the `-3` Chid arm verbatim, modulo the type names).
- Tests: existing `DescArgs` parse tests untouched; new
  `params_from_args_defaults` exercises the boundary resolution.

## refactor: list → Context+Params (0.49.0-5)

Commits: [[10]]

Third of the four Context+Params ports. First port with a
subcommand-specific field beyond `CommonArgs` — `list` carries
the ochid column `width: usize` — so `ListParams` adds that
field next to its embedded `CommonParams`.

- `list::ListParams`: `common: CommonParams` + `width: usize`.
  `impl TryFrom<&ListArgs>` delegates `common` to
  `CommonParams::try_from` and copies `width` straight over
  (clap-applied default already resolved on the args side).
- `pub fn list(_ctx: &Context, params: &ListParams)` — `ctx`
  unused (uniform-signature placeholder).
- `main.rs` dispatch builds `Context` + `ListParams` (matches
  the `-3` / `-4` arms; `list_args` carries the extra field
  the same way `chid_args` / `desc_args` didn't).
- Tests: existing `ListArgs` parse tests untouched; two new
  param-construction tests — `params_from_args_defaults`
  (defaults including `DEFAULT_OCHID_WIDTH`) and
  `params_from_args_with_width` (custom `-w 30`).

## refactor: show → Context+Params (0.49.0-6)

Commits: [[11]]

Fourth and last of the Context+Params ports, completing
Migration A's 12/12. `show`'s wrinkle is that `--files` ships
as a raw `String` on the args side and parses into `FileLimit`
at the binary edge — so `ShowParams::try_from` is fallible for
*two* reasons (`resolve_repos` + `FileLimit::parse`), not just
the `CommonParams` one. Both surface as `String` errors for
uniform handling in `main`.

- `show::ShowParams`: `common: CommonParams` + `files: FileLimit`.
  `impl TryFrom<&ShowArgs>` delegates `common` and calls
  `FileLimit::parse(&a.files)` at the boundary; the earlier
  in-`pub fn show` `FileLimit::parse` call moves out.
  `FileLimit::parse` widened from `fn` to `pub fn` so the
  boundary call can reach it.
- `pub fn show(_ctx: &Context, params: &ShowParams)` — `ctx`
  unused (uniform-signature placeholder); body reads
  `params.files` directly (no more parse step inside).
- `main.rs` dispatch builds `Context` + `ShowParams` (same
  shape as `-3` / `-4` / `-5`).
- Tests: existing `ShowArgs` parse tests untouched; three new
  param-construction tests — `params_from_args_defaults`
  (default `Cap(50)`), `params_from_args_files_variants`
  (`0` / `all` / `5`), and `params_from_args_files_invalid`
  (asserts the boundary parse error surfaces through
  `try_from`).

## chore: close CommonArgs sweep cycle (0.49.0)

Commits: [[12]]

The 0.49.0 cycle's close-out. The cycle started as "finish
Migration A" (0.49.0-0), was re-scoped at 0.49.0-1 into the
full **CommonArgs sweep** — options_flags extraction + `-s` /
`--scope` flag + Context+Params ports — and ran B-first so the
Context+Params ports landed once against the final
`CommonArgs` shape. The Refactor tracking table at the top now
reads all 12/12 done; new subcommands follow the established
shape.

- `Cargo.toml`: drop the suffix `0.49.0-6` → `0.49.0`.
- `notes/todo.md`: move the In-Progress entry to Done as a
  single line (`chid/desc/list/show CommonArgs sweep —
  options_flags + -s/--scope + Context+Params ports 12/12
  (0.49.0)`); new `[36]` ref points at the cycle's opening
  section (matching the `0.48.0` precedent). The full ladder
  detail is preserved in the `### As-built ladder` subsection
  below, not deleted.
- `ARCHITECTURE.md`: `## args → Context + Params` reworded —
  drop "ongoing port" framing, note the 12/12 milestone, keep
  the description of the pattern for new subcommands.
- `notes/chores-10.md`: backfill `-6`'s `Commits:` ref ([[11]]);
  add this close-out section and its `### As-built ladder`
  subsection (the in-progress ladder, moved here from
  `notes/todo.md` with `-2.4`'s leftover `(current)` marker
  corrected to `(done)`).
- `README.md` left alone — the `-s` / `--scope` Multi-repo
  queries rewrite landed in `-2.3`; nothing else in user-facing
  surface changed.
- Process note: `-2.4`'s `(current)` marker in `notes/todo.md`
  was never flipped to `(done)` (the substep-protocol step that
  should have happened before the `-2.4` push). The ladder
  moved here at close-out (with the marker corrected); the miss
  is recorded against the substep-protocol checklist for future
  cycles.

### As-built ladder

The full development ladder as it stood at close-out, moved
from `notes/todo.md > ## In Progress`. Each sub-step's design /
mechanics also lives in its own chores section above; this
view preserves the cycle-wide shape — what was planned at each
depth, what landed when, and the re-scope at `-1` that turned
"finish Migration A" into the full CommonArgs sweep.

**chid/desc/list/show — CommonArgs sweep** (options_flags
extraction + `--scope` + Context+Params port). Re-scoped at
0.49.0-1 from the original "finish Migration A" plan, B-first
so the Context+Params ports land once against the final
`CommonArgs` shape.

Design:
[chores-10.md](chores-10.md#chore-open-0490--finish-migration-a-0490-0)
+ the 0.49.0-1 re-scope subsection; prior `--scope` design in
[chores-06](chores-06.md#generalize---scope-across-commands-0400),
[chores-07](chores-07.md#--scope-enum-refactor-0420).

- 0.49.0-0 plan + version bump + chores section + ladder (done) [[1]]
- 0.49.0-1 options_flags extraction — relocate `CommonArgs`
  → `options_flags/common_args.rs`. Kept separate; no `-1`
  close-out commit.
  - 0.49.0-1.1 the relocation + all four importers (done) [[2]]
  - 0.49.0-1.2 docs: slim ARCHITECTURE.md; start chores-10 (done) [[3]]
- 0.49.0-2 `-R`/`--repo` → `-s`/`--scope` for `chid` /
  `desc` / `list` / `show`.
  - 0.49.0-2.1 open the step (done) [[4]]
    - tidy `## Todo` (drop the two now-in-progress items).
    - add CLAUDE.md rule — a picked-up `## Todo` item is
      deleted when it goes `## In Progress`.
    - backfill `-1.1` / `-1.2` chores `Commits:` refs.
  - 0.49.0-2.2 the rollout — code (done) [[5]]
    - kept `-R` as single path (`Option<PathBuf>`); added
      `-s`/`--scope` (`Option<Scope>` via `parse_scope_roles`,
      keyword-only — `-s <path>` is a future Todo).
    - the two compose (no `conflicts_with`): `-R` overrides
      workspace root, `-s` selects sides within it. New
      `common::resolve_repos(repo, scope)` does the match;
      `for_each_repo` takes a resolved `Vec<PathBuf>`.
    - defaults preserve today: no flag → `[.]`, `-R foo`
      alone → `[foo]`.
    - scope: four subcommand bodies + `--help`; tests;
      CLAUDE.md `chid -R .,.claude -L` → `chid -s code,bot -L`.
  - 0.49.0-2.3 docs (done) [[6]]
    - `README.md` `### Multi-repo queries` rewritten to lead
      with `-s code,bot`; every example updated. The prior
      `-R .,.claude` / `-R . -R .claude` forms no longer parse
      since `-R` is now single-path.
    - `-R PATH` retained for single-path use and as the
      workspace-root override that composes with `-s`
      (`-R ../other -s code,bot`).
    - `ARCHITECTURE.md`: `resolve_repos` added to the
      `common.rs` helper list with a one-line note on what it
      composes.
    - `notes/` swept; no stale `-R .,.claude` for these four.
  - 0.49.0-2.4 unify prose form in CLAUDE.md (done) [[7]]
    - new top-level `## Prose form` section as the single
      source of truth for the intro+bullets shape across
      commit bodies / chores / todo / done / doc comments.
    - slim `## Commit Message Style`, `### Chores section
      content`, and `### Doc comments…` to reference it.
    - surfaced as process drift while writing the `-2.3`
      chores section; deferred from -2.3 to keep that commit
      scoped to the `-s/--scope` docs sweep.
- 0.49.0-3 chid Context+Params port + introduce `CommonParams` (done) [[8]]
- 0.49.0-4 desc Context+Params port (done) [[9]]
- 0.49.0-5 list Context+Params port (done) [[10]]
- 0.49.0-6 show Context+Params port (`TryFrom`, `FileLimit` parse) (done) [[11]]
- 0.49.0 close-out — drop suffix, todo→Done (Context+Params
  port 12/12 + CommonArgs sweep), README + ARCHITECTURE.md (done) [[12]]

## chore: open Subcommand trait sweep (0.50.0-0)

Commits: [[13]]

Multi-step. The 12 subcommand match arms in `main.rs` repeat
the same `Context::load` + `try_from(&args)` + `run_command`
boilerplate — ~12 lines per arm. This cycle introduces a
`Subcommand` trait so each arm collapses to a single
`dispatch::<T>(args, cli.log)` line while keeping the
`Commands` enum as the dispatch source of truth (compile-time
exhaustiveness preserved). Plan-only opener; the trait + first
worked-example port land in `-1`.

### Trait shape (initial sketch)

- `Subcommand` trait on each subcommand's `Args` type:
  - `type Params`
  - `fn into_params(&self) -> Result<Self::Params, String>`
    — total `From` impls return `Ok(…)`; fallible `TryFrom`
    impls forward their error.
  - `fn run(ctx: &Context, params: &Self::Params)
    -> Result<(), Box<dyn Error>>`
  - `fn suppress_banner(&self) -> bool { false }` — `chid` /
    `desc` / `list` / `show` override.
  - `fn is_detached_exec(&self) -> bool { false }` —
    `finalize` overrides.
- Free helper `dispatch<S: Subcommand>(args: &S, log: …)
  -> ExitCode` wraps the per-arm body.
- `main.rs` match arms collapse to one line each; the
  banner / detached-exec peek logic moves behind the trait.

Shape is subject to refinement once `-1` exercises it on
`chid`.

### Ladder

- 0.50.0-0 plan + version bump + this section + todo ladder
  + linkme/inventory todos (current)
- 0.50.0-1 add `subcommand.rs` (trait) + port `chid`
  (worked example)
- 0.50.0-2..N port remaining 11 subcommands. Grouping
  decided per substep; candidate split:
  - `From` (total): `validate_desc`, `fix_desc`, `clone`,
    `init`, `symlink`, `sync`, `push`
  - `TryFrom` (fallible): `desc`, `list`, `show`, `finalize`
- 0.50.0-K `main.rs` dispatch rework — drop the per-arm
  match boilerplate; move banner / detached-exec peeks
  behind the trait.
- 0.50.0 close-out — drop suffix, todo→Done.

### Per-step evaluation

Effectiveness of the trait approach is evaluated after each
substep. Possible outcomes at any step boundary:

- Continue as planned.
- Significantly modify the trait shape (e.g. swap
  `into_params`/`run` for a single `dispatch` method on the
  trait; absorb the `Args` type as an associated type instead
  of `impl Subcommand for ArgsType`; etc.) — recorded in a
  new chores subsection at the next step.
- Abandon: revert the cycle to a non-trait baseline (close
  the cycle as a no-op with a chores `### Outcome` note
  capturing what didn't fit). The version-bump and todo
  entries land in Done as the record of "tried, didn't
  ship."

This is part of the cycle's contract, not an exceptional
exit path — the trait sweep is the lower-risk reading of
"reduce per-arm boilerplate" and `linkme` / `inventory` are
the higher-leverage / higher-cost alternatives queued as
follow-up todos.

### Why not linkme / inventory now

Both eliminate the `Commands` enum entirely (link-time
distributed slice or runtime registry). They'd cut
per-subcommand touchpoints from 3 (mod decl + enum variant +
match arm) to 1 (registration). Costs: compile-time
exhaustiveness check goes away (missing registration =
runtime gap); help-output ordering becomes link-order
unless sorted; either is macro-magic dependency. The trait
sweep gets every match arm to one line with none of those
costs; linkme + inventory are queued as separate Todo items
to revisit if the per-arm cost ever feels burdensome.

### Per-substep contract

Per `notes/substep-protocol.md`: `cargo fmt` / `clippy
--all-targets -- -D warnings` / `test` / `install --path .
--locked` + retest before each commit; bump `Cargo.toml` at
sub-step start; flip todo ladder markers; pair commits
across both repos with ochid trailers.

## refactor: SubcommandRunner trait + chid (0.50.0-1)

Commits: [[14]]

Worked example for the 0.50.0 cycle: introduce the
`SubcommandRunner` trait, validate its shape on `chid`, and
extract `main.rs`'s session-chrome into a global `sb_ide`
function consumed both by the trait's default `dispatch`
(for ported commands) and from `main` directly (for
unported arms). The Chid arm in `main` reaches its target
one-liner — `Commands::Chid(args) => args.dispatch(cli.log),`
— with chid's banner suppression riding on a new
`suppress_banner` field in `ChidParams` that the trait's
default `dispatch` reads via
`SubcommandRunner::suppress_banner(&params)`.

Three material design calls landed at this substep's
evaluation gate (full discussion below): the trait was
renamed from `Subcommand` to `SubcommandRunner` to sidestep
a `clap::Subcommand` collision; the session-chrome block
(banner + `surface_previous_failures`) was extracted into a
global `pub fn sb_ide` in `main.rs`; and the trait's
`suppress_banner` / `is_detached_exec` peeks were *kept* on
the trait — default `false`, signature `(_params:
&Self::Params)` — with the chrome data living on each
command's `Params` (e.g. `ChidParams::suppress_banner`) so
ported arms become true one-liners.

- New `SubcommandRunner` trait (`src/subcommand.rs`):
  associated `Params` type; required `to_params(&self) ->
  Result<Params, String>` (absorbs both `From` and `TryFrom`
  shapes uniformly, matching the `String` error type set by
  `finalize` / `chid`); required `run(ctx, params)` as an
  associated function (no `&self` — `params` carries
  everything the body needs); default-`false` peek
  associated functions `suppress_banner(params)` and
  `is_detached_exec(params)`; default `dispatch(&self, log)`
  that loads `Context`, builds `Params` via `to_params`,
  calls `crate::sb_ide(Self::suppress_banner(&p),
  Self::is_detached_exec(&p))`, then runs via `run` and
  maps the result to `ExitCode`. Method name `to_params`
  (not `into_params` per the -0 sketch — `&self`-borrowing
  reads more truthfully).
- `chid::ChidParams` gains a `suppress_banner: bool` field
  (clap-free; `ChidParams::try_from` copies it from
  `a.common.no_label` at the binary edge). `chid::ChidArgs
  impl SubcommandRunner` overrides
  `fn suppress_banner(params) -> bool { params.suppress_banner }`;
  otherwise standard `to_params` / `run` delegations.
  `is_detached_exec` keeps its trait default (chid is never
  the detached child).
- `src/main.rs`: pre-match `if !is_detached_exec { … banner
  + surface_previous_failures }` block deleted; replaced by
  a global `pub fn sb_ide(suppress_banner: bool,
  is_detached_exec: bool)` defined near `BANNER` and
  `bm_track`. Chid match arm collapses to
  `Commands::Chid(args) => args.dispatch(cli.log),`. The
  11 unported arms call `sb_ide(suppress_banner,
  is_detached_exec)` directly, reading the bools from a
  shrunken pre-match peek match (chid removed; each future
  port removes its variant too). When the last command is
  ported the pre-match peek and the unported-side `sb_ide`
  callers both disappear; only the trait's call site
  remains.
- Tests unchanged — `ChidParams::try_from` happy-path is
  the same code path the existing tests exercise; the
  trait adds no new logic; banner / -L behavior preserved
  end-to-end (see `### Session chrome: order shift`).

### Naming: `SubcommandRunner`

`clap::Subcommand` is the name of both a clap trait and a
derive macro (used on `enum Commands`). Naming our trait
`Subcommand` collides. Three real options:

- **Rename our trait** ← landed: `SubcommandRunner`. The
  `*Runner` suffix is idiomatic Rust (`TestRunner`,
  `BenchmarkRunner`, …); reads cleanly at `impl
  SubcommandRunner for ChidArgs`; no namespace gymnastics
  anywhere.
- **`SubcommandTrait`** — rejected. Rust style guides
  specifically discourage `*Trait` suffixes (parallel to
  `*Interface` in older Java style).
- **`SubcommandExec`** — fine, just less idiomatic than
  `*Runner`.
- **Keep `Subcommand`, qualify clap at use site** — was the
  first thing tried (`#[derive(clap::Subcommand, ...)]`,
  drop `Subcommand` from `use clap::{...}`). Worked, but
  the one ugly use site read worse than a clean rename.

### Trait scope: peek methods read from Params

The trait carries:

- `type Params` (associated type for the clap-free domain
  struct each subcommand operates on).
- `to_params(&self) -> Result<Params, String>` (required).
- `run(ctx, params) -> Result<(), Box<dyn Error>>` (required).
- `suppress_banner(params: &Self::Params) -> bool { false }`
  (peek, default false).
- `is_detached_exec(params: &Self::Params) -> bool { false }`
  (peek, default false).
- `dispatch(&self, log) -> ExitCode` (default impl below).

Earlier iterations considered dropping the peek methods —
the reasoning was that they hid a field read without
collapsing the per-arm match in `main.rs`. That argument
no longer applies once the chrome lives in the trait's
default `dispatch`: the peeks are read by `dispatch` (via
`Self::suppress_banner(&params)` /
`Self::is_detached_exec(&params)`) and passed into
`crate::sb_ide(…)`. They're load-bearing, not indirection.

The peeks take `params: &Self::Params` (not `&self`) so
the chrome data lives on each command's `Params` struct
(e.g. `ChidParams::suppress_banner`). Commands that don't
need either flag (the 10 non-chid, non-finalize ones)
just don't override; the default `false` applies.

This makes ported arms in `main` true one-liners
(`Commands::Chid(args) => args.dispatch(cli.log),`) —
all the chrome behavior is driven by the command's own
`Params` content, queried through trait methods, with no
visible coupling to `main`.

### Session chrome: order shift

Today `main` has an inline `if !is_detached_exec { …
log::info!("{BANNER}"); finalize::surface_previous_failures();
}` block *before* the outer match. Each subcommand runs
under that shared chrome.

The extraction lifts this block into a global
`pub fn sb_ide(suppress_banner: bool, is_detached_exec: bool)`
in `main.rs` (placed near `BANNER` and `bm_track`). Two
call sites:

- The trait's default `dispatch` calls `crate::sb_ide(...)`
  for ported commands (after `Context::load` and
  `to_params` succeed).
- Each unported arm in `main` calls
  `sb_ide(suppress_banner, is_detached_exec)` directly,
  reading the bools from a shrunken pre-match peek match
  (which only enumerates the unported `-L`-supporting
  variants — chid is gone, future ports remove their
  variants too).

**Two visible output-order shifts vs. today.** Both
stem from the chrome no longer running once-pre-match:

- `bm-track enter` now prints *before* the banner. The
  banner used to be the first line; `bm-track` now is.
  Same logical content, swapped order.
- For ported commands, the banner is emitted *after*
  `Context::load` and `to_params`. If `to_params` fails
  (e.g. workspace lookup error) the banner is suppressed
  along with the rest of the chrome — the user sees the
  error message only. Today the banner would have
  printed before the error. Affects error paths only;
  happy-path output is identical.

The user has accepted both shifts; the rationale is that
the banner content (and to a lesser extent `bm-track`)
is informational chrome, not load-bearing for any
script-parseable mode (which already uses `-L`).

### Why no closure

A closure in `fn main()` was the prior iteration's shape;
this one promotes `sb_ide` to a global function so both
the trait's default `dispatch` (via `crate::sb_ide`) and
`main`'s unported arms can call it. A closure local to
`fn main()` would not be reachable from the trait module.

### Why peek methods take `&Self::Params` (not `&self`)

The chrome data is naturally a Params concept (the
clap-free domain struct each subcommand operates on),
not an Args concept. Reading the peek off `Params` means
the trait's default `dispatch` can be entirely
clap-free; only `to_params` straddles the Args→Params
boundary.

A minor consequence: chrome runs *after* `to_params`
rather than before, so a `to_params` failure suppresses
the banner. See `### Session chrome: order shift`.

### Evaluation hook

Per the cycle's per-step evaluation gate (see the `-0`
section's `### Per-step evaluation`), this `-1` was the
first opportunity to modify the trait shape significantly
or abandon the cycle. Three material design calls landed
(see `### Naming: SubcommandRunner`, `### Trait scope:
peek methods read from Params`, and `### Session chrome:
order shift` above). The shape can still change at `-2`'s
boundary based on how the worked example reads.

## chore: dicom-rs gotchas → todo (0.50.0-1.1)

Commits: [[15]]

Inter-substep tidy: record two surfacings from the
manual por→dual conversion of `../dicom-rs`
(2026-05-14) — the user not only converted a por
workspace by hand but also named the bot side `.bot`
(rather than `.claude`), exercising two
not-yet-covered seams.

**Gotcha 1 — por → dual is arduous.** The bundled
`notes/todo.md` "por/dual parity + bidirectional
conversion" entry was split: `por → dual` is promoted
to its own higher-priority item; the remaining entry
covers parity + the `dual → por` (detach) direction.
The manual exercise was incoherent enough that the user
plans to redo it, which makes a routine subcommand
worth surfacing separately from the broader parity
goal. Both items continue to point at
`notes/por-dual-parity.md`; the design stub already
covers both directions and needs no edit.

**Gotcha 2 — `.claude` hardcoded in diagnostics.**
`vc-x1 finalize --repo .bot` honored `--repo`
throughout the actual operations (squash, bookmark set,
push to origin all ran against `.bot/`), but the
`bm-track` enter/exit lines still emitted
`.claude(main)=no-jj` — the diagnostic label has
`.claude` baked into the string. Cosmetic today (output
text only), but load-bearing once bot-repo name
becomes configurable.

- `notes/todo.md`: new "**`por → dual` conversion.**"
  entry inserted above the (now trimmed) parity item;
  trimmed item retitled "**por/dual parity + `dual →
  por` conversion.**" — wording adjusted to reflect
  the remaining scope; both reference `[[4]]`. New
  "**Audit hardcoded `.claude` in diagnostics /
  logging.**" entry inserted just below them.

## chore: --exec doc + matches! → match (0.50.0-1.2)

Commits: [[16]]

Inter-substep tidy: a small readability rewrite in
`main.rs` and a doc expansion on `FinalizeArgs::exec` so
the `--detach` / `--exec` handshake is spelled out at
the field that implements it.

- `src/main.rs`: `let is_detached_exec = matches!(cli.command,
  Commands::Finalize(ref f) if f.exec)` → explicit
  two-arm `match`. Semantically identical (both expand to
  the same thing); the trailing-`if`-guard inside
  `matches!` was found weird enough to be worth replacing
  at this site.
- `src/finalize.rs`: `FinalizeArgs::exec` doc expanded
  from the one-liner "Internal: run exec path (used by
  detach)" to a paragraph anchored on the sibling `pub
  detach` field — when `--detach` is passed the parent
  `fork`s and re-execs `vc-x1 finalize --exec …`, the
  child sees `exec: true` and routes through
  `finalize_exec()` for the background work — plus a
  second paragraph spelling out why `hide = true` is set.

## refactor: hoist Context::load (0.50.0-2)

Commits: [[17]]

`Context::load` was invoked in every match arm — once
in `dispatch` for the ported `Chid` arm, and inline at
the head of each of the 10 unported arms. The body is
identical, the result is identical, only one arm runs
per invocation. Hoist the load to a single site at the
top of the match in `main`, change
`SubcommandRunner::dispatch` to take `&Context` instead
of building one itself, and let the unported arms read
the same `&ctx`. Net `-63` lines in `main.rs`.

The trait shape change is small but load-bearing for
the rest of the cycle — every future port arrives at the
settled `dispatch(&ctx)` shape rather than the
`dispatch(cli.log)` shape. Splitting the hoist out as
its own substep keeps the per-port commits (starting
with `desc` at `-3`) reviewable in isolation: each one
becomes "add `suppress_banner` + impl `SubcommandRunner`
+ collapse the arm" with no infrastructure churn mixed
in.

- `src/main.rs`: `Context::load` lifted out of each
  match arm into a single block above the `match
  cli.command`; 10 inline blocks deleted.
- `src/subcommand.rs`: `dispatch(&self, log:
  Option<PathBuf>)` → `dispatch(&self, ctx: &Context)`;
  the inline `Context::load` inside dispatch deleted.
  Module + item doc-comments updated for the new
  signature (`args.dispatch(&ctx)`, "~11 lines"
  encapsulated, "Context is loaded once in main and
  passed in by reference"). Unused `use
  std::path::PathBuf` deleted.

## refactor: desc → SubcommandRunner (0.50.0-3)

Commits: [[18]]

First port off the `chid` worked example: `desc` lands
on the trait-based `dispatch` shape settled in `-2`.
The arm in `main.rs` collapses to
`Commands::Desc(args) => args.dispatch(&ctx),`, and
`desc` joins `chid` in the trait-owned banner-suppression
path (so `desc -L` no longer needs a special-case match
arm at the top of `main`).

Mechanical, no design calls. The shape mirrors `chid`
(0.50.0-1) exactly: add a `suppress_banner: bool` to
`DescParams`, set it in `TryFrom<&DescArgs>` from
`a.common.no_label`, and implement `SubcommandRunner for
DescArgs` with `to_params` / `run` / `suppress_banner`
overrides. The `desc` module doc-comment is unchanged —
`chid`'s isn't either, so the trait impl is left visible
inside the file rather than re-advertised at the top.

- `Cargo.toml`: `0.50.0-2` → `0.50.0-3`.
- `src/desc.rs`: `SubcommandRunner for DescArgs` impl
  added; `suppress_banner: bool` field added to
  `DescParams`; `TryFrom<&DescArgs>` updated to copy
  `a.common.no_label` into it.
- `src/main.rs`: `Commands::Desc(...)` arm collapsed
  to `args.dispatch(&ctx)`; `Commands::Desc` line
  dropped from the `suppress_banner` match; doc
  comment over that match updated to reflect that
  `desc` is now trait-owned alongside `chid`.
- `notes/chores-10.md`: backfilled `Commits: [[17]]`
  on the 0.50.0-2 section.
- `notes/todo.md`: ladder updated — flip 0.50.0-2 →
  `(done)` (absorbing the missed flip from the prior
  commit); add `0.50.0-3 port desc (current)`;
  placeholder updated to "remaining 10 subcommands".

## refactor: list → SubcommandRunner (0.50.0-4)

Commits: [[19]]

Second mechanical port off the trait shape settled in
`-2`: `list` follows `desc` (`-3`) onto
`SubcommandRunner::dispatch`. The arm in `main.rs`
collapses to `Commands::List(args) => args.dispatch(&ctx),`,
and `list -L` joins `chid -L` / `desc -L` in the
trait-owned banner-suppression path. Only `show` remains
in the top-level `suppress_banner` match.

Same shape as the `desc` port. `ListParams` already
carried a non-`CommonParams` field (`width`) — the
new `suppress_banner: bool` simply joins it; nothing
about the trait shape changes to accommodate the
"`Params` has extra fields" case.

- `Cargo.toml`: `0.50.0-3` → `0.50.0-4`.
- `src/list.rs`: `SubcommandRunner for ListArgs` impl
  added; `suppress_banner: bool` field on `ListParams`.
- `src/main.rs`: `List` arm collapsed to
  `args.dispatch(&ctx)`; `Commands::List` line dropped
  from the `suppress_banner` match; doc comment updated
  to reflect that `list` is now trait-owned alongside
  `chid` / `desc`.
- `notes/chores-10.md`: backfilled `Commits: [[18]]`
  on the 0.50.0-3 section.
- `notes/todo.md`: ladder updated — 0.50.0-3 →
  `(done)`; add `0.50.0-4 port list (current)`;
  placeholder trimmed to "remaining 9".

## refactor: port 8 → SubcommandRunner (0.50.0-5)

Commits: [[20]]

Bulk port: the 8 subcommands that don't need the
`is_detached_exec` peek — `show` (TryFrom +
suppress_banner), and the 7 total-`From` commands
(`validate_desc`, `fix_desc`, `clone`, `init`,
`symlink`, `sync`, `push`). Each arm in `main.rs`
collapses to `args.dispatch(&ctx)`. After this commit,
`finalize` is the only remaining unported arm; it
lands separately in `-6` because its
`is_detached_exec=true` plumbing is the only
non-mechanical bit left.

The top-level `suppress_banner` match in `main.rs` is
gone (show was its last consumer). The
`is_detached_exec` match remains — `finalize`'s arm
still reads it; the `bm_track` enter/exit gates also
still consult it. Both will fold into the trait when
finalize ports in `-6`.

Shape per command:

- **`show`** (TryFrom): same as `desc` / `list` —
  added `suppress_banner: bool` to `ShowParams`, set
  from `a.common.no_label` in `TryFrom`, impl
  `SubcommandRunner` with `to_params` / `run` /
  `suppress_banner` overrides.
- **7 `From` commands**: no `suppress_banner` flag
  needed (none of them have `-L`). `to_params`
  returns `Ok(XxxParams::from(self))`; `run`
  forwards to the existing op function. Trait
  defaults cover `suppress_banner` / `is_detached_exec`
  (both `false`).

- `Cargo.toml`: `0.50.0-4` → `0.50.0-5`.
- `src/show.rs`: `SubcommandRunner for ShowArgs` impl
  added; `suppress_banner` field on `ShowParams`.
- `src/validate_desc.rs` / `src/fix_desc.rs` /
  `src/clone.rs` / `src/init.rs` / `src/symlink.rs` /
  `src/sync.rs` / `src/push.rs`: `SubcommandRunner`
  impl added per file.
- `src/main.rs`: 8 arms collapsed to
  `args.dispatch(&ctx)`; `let suppress_banner = …`
  match deleted (no remaining consumer); finalize's
  `sb_ide(suppress_banner, …)` → `sb_ide(false, …)`.
- `notes/chores-10.md`: backfilled `Commits: [[19]]`
  on the 0.50.0-4 section.
- `notes/todo.md`: ladder updated — 0.50.0-4 →
  `(done)`; add `0.50.0-5 port 8 subcommands (current)`
  and `0.50.0-6 port finalize` entry.

## refactor: finalize → SubcommandRunner (0.50.0-6)

Commits: [[21]]

Last per-command port: `finalize` joins the other 11
arms on `SubcommandRunner::dispatch`. The
`is_detached_exec` slot on the trait (default `false`,
unused since `-1`) finally gets its implementor —
`FinalizeArgs::is_detached_exec(params)` returns
`params.exec`, which the trait's default `dispatch`
threads into `crate::sb_ide` so the detached
`finalize --exec` child stays silent.

`FinalizeParams` already carried `exec: bool` from
the existing `TryFrom` impl — no new field needed.
The trait override reads it directly.

What lands in this commit:

- The Finalize arm in `main.rs` collapses to one line.
  Every match arm is now `args.dispatch(&ctx)`.
- The `run_command` helper in `main.rs` becomes dead
  (no remaining caller) and is deleted. Clippy
  `-D warnings` would block the cycle otherwise; the
  deletion is the natural consequence of the last
  port.

What does **not** land — deferred to `-K`:

- The top-level `let is_detached_exec = match
  cli.command { Commands::Finalize(ref f) => f.exec,
  _ => false };` block in `main.rs` stays. It gates
  the `bm_track` enter/exit pair which lives outside
  `dispatch`. Moving those calls inside the trait
  (so `main` no longer needs the peek) is `-K`'s job.

After this commit `main.rs`'s exec_code match is
twelve `args.dispatch(&ctx)` lines and nothing else.

- `Cargo.toml`: `0.50.0-5` → `0.50.0-6`.
- `src/finalize.rs`: `SubcommandRunner for FinalizeArgs`
  impl added; `is_detached_exec(params)` override
  returns `params.exec`.
- `src/main.rs`: `Finalize` arm collapsed to
  `args.dispatch(&ctx)`; `fn run_command` deleted
  (now unused).
- `notes/chores-10.md`: backfilled `Commits: [[20]]`
  on the 0.50.0-5 section.
- `notes/todo.md`: 0.50.0-5 → `(done)`; 0.50.0-6
  marked `(current)`.

## refactor: move bm_track into dispatch (0.50.0-7)

Commits: [[22]]

Cycle cleanup (formerly the `-K` placeholder).
With every subcommand on `SubcommandRunner::dispatch`,
the top-level `is_detached_exec` match and the
`bm_track` enter/exit gates that consumed it no longer
need to live in `main`. Move them inside `dispatch`,
where they read the trait's `is_detached_exec(&params)`
directly. `main.rs`'s `fn main` collapses to:

```
let cli = …;
let _log = …;
let ctx = Context::load(…);
match cli.command { Commands::X(args) => args.dispatch(&ctx), … }
```

— roughly the shape the cycle was aiming for. The
twelve-arm match itself stays (compile-time
exhaustiveness on `Commands`); each arm is a single
`args.dispatch(&ctx)` line, and `main` does no other
work.

`bm_track` becomes `pub` so `dispatch` can call it
across the module boundary. The function name lookup
(`std::env::args().nth(1)`) moves into `dispatch`
unchanged — clap has validated the positional by the
time `dispatch` runs.

- `Cargo.toml`: `0.50.0-6` → `0.50.0-7`.
- `src/main.rs`: `fn bm_track` → `pub fn bm_track`;
  top-level `let is_detached_exec = match …` block,
  `let command_name = …` line, and both `if
  !is_detached_exec { bm_track(…) }` blocks deleted;
  the `let exit_code = match … ; … ; exit_code`
  pattern collapses to a single `match` expression.
- `src/subcommand.rs`: `dispatch` now resolves
  `is_detached_exec` once, threads it into both
  `sb_ide` and the `bm_track` enter/exit gates;
  trait-method doc on `is_detached_exec` expanded to
  mention the second consumer; dispatch doc expanded
  to cover the new bracket.
- `notes/chores-10.md`: backfilled `Commits: [[21]]`
  on the 0.50.0-6 section.
- `notes/todo.md`: 0.50.0-6 → `(done)`; rename
  `0.50.0-K …` line to `0.50.0-7 main.rs dispatch
  rework (current)`.

## chore: close Subcommand trait sweep (0.50.0)

Cycle close-out. All 12 subcommands now run through
`SubcommandRunner::dispatch`; `main.rs`'s `fn main`
is `Context::load` + a 12-arm dispatch match and
nothing else. The per-arm boilerplate the cycle set
out to remove (`Context::load` + `try_from` +
`run_command`, ~12 lines × 12 arms) is gone; each arm
is a single `args.dispatch(&ctx)` line. The
`Commands` enum stays the dispatch source of truth
(compile-time exhaustiveness preserved), so the
`linkme` / `inventory` follow-up items in `## Todo`
remain queued as higher-leverage alternatives if the
per-arm cost ever feels burdensome again.

### As-built ladder

- 0.50.0-0 plan + version bump + chores section +
  todo ladder + linkme/inventory todos
- 0.50.0-1 add `subcommand.rs` (`SubcommandRunner`
  trait) + port `chid` (worked example, three
  material design calls landed: trait rename
  `Subcommand` → `SubcommandRunner` to dodge
  `clap::Subcommand` collision; session-chrome
  extracted as `pub fn sb_ide`; peek methods read
  from `Params`)
- 0.50.0-1.1 dicom-rs gotchas → todo (inter-substep
  tidy)
- 0.50.0-1.2 --exec doc + matches! → match
  (inter-substep tidy)
- 0.50.0-2 hoist `Context::load` to a single site in
  `main`; change `dispatch` to take `&Context`
- 0.50.0-3 port `desc`
- 0.50.0-4 port `list`
- 0.50.0-5 bulk port: `show` + 7 `From` commands
  (`validate_desc`, `fix_desc`, `clone`, `init`,
  `symlink`, `sync`, `push`); top-level
  `suppress_banner` match deleted
- 0.50.0-6 port `finalize` (`is_detached_exec`
  override wires `params.exec` through the trait);
  `fn run_command` deleted from `main`
- 0.50.0-7 move `bm_track` enter/exit into
  `dispatch`; delete the top-level
  `is_detached_exec` match in `main` (was the `-K`
  placeholder)
- 0.50.0 close-out

### Outcome

The trait approach landed cleanly — none of the
per-substep evaluation gates fired the "modify shape
significantly" or "abandon" outcomes the open
section reserved. The shape settled at `-1` (with the
three design calls noted above) held for the
remaining 10 ports without revision. `linkme` /
`inventory` would have gone further (eliminate the
`Commands` enum entirely), but at compile-time
exhaustiveness and macro-magic costs; they remain
queued in `## Todo` as separate items.

- `Cargo.toml`: `0.50.0-7` → `0.50.0` (suffix
  dropped — cycle close marker).
- `notes/chores-10.md`: backfilled `Commits: [[22]]`
  on the 0.50.0-7 section; new close-out section
  with the as-built ladder + outcome notes.
- `notes/todo.md`: deleted the In Progress ladder
  block; added `Subcommand trait sweep — 12
  subcommands ported via SubcommandRunner trait,
  main.rs collapsed to Context::load + thin dispatch
  (0.50.0)` to `## Done` with `[[37]]` ref.

# References

[1]: https://github.com/winksaville/vc-x1/commit/10788bd158c4 "10788bd158c4574fe5a10fab41ea32e4becc86d3"
[2]: https://github.com/winksaville/vc-x1/commit/cc19273e2ca3 "cc19273e2ca30f1beedd55198a11bdf045b281ee"
[3]: https://github.com/winksaville/vc-x1/commit/f6438bc7394e "f6438bc7394e76a3d83de08467c6fafec7a819b7"
[4]: https://github.com/winksaville/vc-x1/commit/7e1ea28cc7f6 "7e1ea28cc7f62c2f0920d25ae7c21dba69629e02"
[5]: https://github.com/winksaville/vc-x1/commit/af7d87a031ea "af7d87a031eaa6b4773fa01ed16a6eea734c5262"
[6]: https://github.com/winksaville/vc-x1/commit/14a86674add0 "14a86674add076ec2fcb0784c9d6c955223f769c"
[7]: https://github.com/winksaville/vc-x1/commit/c1784a0548df "c1784a0548dfb93dbbdbd93aeb69802b0561f258"
[8]: https://github.com/winksaville/vc-x1/commit/d0d886a09956 "d0d886a0995679d82cdb67c10b24c7c17f1915e0"
[9]: https://github.com/winksaville/vc-x1/commit/6d453b551f78 "6d453b551f781c8c793da72cba0d4a70c44277ce"
[10]: https://github.com/winksaville/vc-x1/commit/00f49f10b7a3 "00f49f10b7a3b55192f9feb6313e5968efa16bb0"
[11]: https://github.com/winksaville/vc-x1/commit/d772a204be15 "d772a204be150ee8da8d2cbc33496410940aecb5"
[12]: https://github.com/winksaville/vc-x1/commit/4b73862668ab "4b73862668abe34675f06f97e53555f92c4dc08d"
[13]: https://github.com/winksaville/vc-x1/commit/040aa2880421 "040aa28804211e529baa4ebf0a27f3ebfcef6e95"
[14]: https://github.com/winksaville/vc-x1/commit/9a447b843b81 "9a447b843b81eeca565db33cb12ece3095bff903"
[15]: https://github.com/winksaville/vc-x1/commit/9a7d33ba6556 "9a7d33ba6556cdb5a575c96236554cf19d57b23b"
[16]: https://github.com/winksaville/vc-x1/commit/8066eabc0752 "8066eabc0752a08880ff3bbc14e5a4674f7a7e4f"
[17]: https://github.com/winksaville/vc-x1/commit/25d515c7aa5d "25d515c7aa5df80a4ae39db2d19b84b4e6100a55"
[18]: https://github.com/winksaville/vc-x1/commit/c4a9b73648a9 "c4a9b73648a9ba3d8e01139c6a32e0fccab444df"
[19]: https://github.com/winksaville/vc-x1/commit/288b9627e380 "288b9627e380105ebe6703f28ec0683660e4c95f"
[20]: https://github.com/winksaville/vc-x1/commit/5899dc21e7ec "5899dc21e7ec3fc6ad0ac79dbaf78fddfffc5075"
[21]: https://github.com/winksaville/vc-x1/commit/56fac6ee4913 "56fac6ee4913051b112f064fd53ee37981898029"
[22]: https://github.com/winksaville/vc-x1/commit/00c87177b85e "00c87177b85e1d3104b47279baf6aa214a362682"
