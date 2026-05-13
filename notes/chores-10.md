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
| `chid` | pending (0.49.0-3) — introduces the shared `CommonParams`; `TryFrom` |
| `desc` | pending (0.49.0-4) — `TryFrom` |
| `list` | pending (0.49.0-5) — `TryFrom` |
| `show` | pending (0.49.0-6) — `TryFrom`; also parses `--files` → `FileLimit` at the boundary |

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

# References

[1]: https://github.com/winksaville/vc-x1/commit/10788bd158c4 "10788bd158c4574fe5a10fab41ea32e4becc86d3"
[2]: https://github.com/winksaville/vc-x1/commit/cc19273e2ca3 "cc19273e2ca30f1beedd55198a11bdf045b281ee"
[3]: https://github.com/winksaville/vc-x1/commit/f6438bc7394e "f6438bc7394e76a3d83de08467c6fafec7a819b7"
[4]: https://github.com/winksaville/vc-x1/commit/7e1ea28cc7f6 "7e1ea28cc7f62c2f0920d25ae7c21dba69629e02"
[5]: https://github.com/winksaville/vc-x1/commit/af7d87a031ea "af7d87a031eaa6b4773fa01ed16a6eea734c5262"
[6]: https://github.com/winksaville/vc-x1/commit/14a86674add0 "14a86674add076ec2fcb0784c9d6c955223f769c"
