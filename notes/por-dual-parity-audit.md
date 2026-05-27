# por/dual parity audit (0.61.0-1)

Read-only snapshot audit of every place the `dual` workspace
topology (code repo + `.claude/` companion, cross-linked by
`ochid:` trailers, configured by `.vc-config.toml`) gets a
privileged code path over `por` (Plain Old Repo — single code
repo, no `.claude/`, no `.vc-config.toml`). Snapshot taken at
commit `463e6bd9` (`docs: prep por/dual parity audit
(0.61.0-0)`). Parent design item is T5 in `todo.md` [[1]];
design stub is `por-dual-parity.md` [[2]].

- Scope: every subcommand, the `--por` flag's reach, the
  ochid-touching helpers, test fixtures.
- Out of scope: proposing equalization shape (each finding
  carries a one-sentence sketch only — concrete fixes are
  follow-up cycles).
- Method: grep + read-through. No code changes.

## 1. `options_flags/por.rs` — the `--por` gate

- Files touched:
  - `src/options_flags/por.rs` (whole file, 1-20).
  - Consumers: `src/init.rs:13,61-65,592-596,1199-1203`,
    `src/clone.rs:26,48-51,70,81,143-163`,
    `src/init/params.rs:39,60`.
  - Test references only:
    `src/test_helpers.rs:24,105,196`,
    `src/init/tests.rs` (many).
- Divergence: `PorFlag` is consumed by **exactly two**
  subcommands — `init` and `clone`. No other subcommand
  surfaces `--por` (`grep "PorFlag" src/` shows zero other
  hits). Doc on the flag itself is creation-time framed: "at
  creation time" — there is no equivalent runtime flag for
  the post-creation commands (`push`, `sync`, `finalize`,
  `validate-desc`, `fix-desc`, `show`, etc.) to declare
  "treat this workspace as por".
- Severity / category: *Defaulting* + *Feature gap*. Absent
  flag = dual; `--por` is a creation-only override; runtime
  commands infer topology indirectly (presence /
  `workspace.other-repo` value of `.vc-config.toml`).
- Equalization sketch: The bot thinks the path is to either
  promote `PorFlag` into a shared option flag plumbed
  through every command, or commit to topology-from-config
  uniformly (no `--por` runtime flag, all commands read
  `default_scope` / `.vc-config.toml` and branch from
  there). The latter is closer to today's `sync`-style
  resolution.

## 2. `init` — dual privileged at the orchestration layer

- Files touched:
  - `src/init.rs` — `plan_init` (583-641), `create_por`
    (1217-1265), `create_dual` (1280-1380),
    `write_por_vc_config` / `write_por_gitignore` (355-363),
    `write_code_config` / `write_session_config` (379-391),
    dispatch (1199-1203).
  - `src/init/params.rs:39,60` — `por: bool` field.
- Divergence: Symmetric on the surface (`create_por` /
  `create_dual` are sibling orchestrators), but the
  defaulting is asymmetric:
  - `plan_init` synthesizes `Scope(vec![Side::Code])` when
    `params.por` else `Scope(vec![Side::Code, Side::Bot])`
    (init.rs:592-596).
  - `--config` is rejected unless `--por` is set
    (init.rs:608-613, "`--config` is only valid with
    `--por` (dual configs are per-side and unconditional)")
    — dual gets implicit per-side configs; por gets
    user-controllable config and an unconditional
    `.gitignore`.
  - Dispatch defaults to `create_dual` (init.rs:1199-1203:
    `if params.por { create_por } else { create_dual }`).
  - The dry-run printout treats dual as the baseline and
    emits "(skipped — …)" lines for por
    (init.rs:1184-1196).
- Severity / category: *Defaulting* (medium — dual is the
  default branch; por is the opt-in branch with extra
  config-knob asymmetry).
- Equalization sketch: The bot thinks the two sibling
  orchestrators could share a common config-writing layer
  with topology-driven side selection — but the bigger ask
  is reframing the dispatch so neither shape is "the
  default" at the type level (e.g. a `Topology { Por,
  Dual }` enum threaded through, with the default decided by
  user config, per the deferred user-config-topology-default
  proposal).

## 3. `clone` — same shape as init

- Files touched:
  - `src/clone.rs` — `clone_repo` (113-167), `clone_one`
    (180-196), `clone_dual` (202-226), `CloneParams`
    (67-85).
- Divergence: Mirror of init at the dispatch layer
  (clone.rs:156-163: `if params.por { clone_one } else {
  clone_dual }`). `clone_one` clones a single repo and
  verifies tracking; `clone_dual` clones code + bot
  (deriving the bot URL via `derive_session_url`) and
  installs the symlink. The bot side is mandatory in
  `clone_dual` ("no graceful skip — both sides required by
  the default dual shape").
- Severity / category: *Defaulting* (medium). Dual is the
  default; `clone_one` is reachable only via `--por`.
- Equalization sketch: Same as init — topology choice
  could be lifted off the flag and onto the
  user/workspace-config layer; the two orchestrators are
  already cleanly factored, so the wiring change is small.

## 4. `push` — dual-only, no por support at all

- Files touched:
  - `src/push.rs` (whole file).
  - `src/push/integration_tests.rs` (whole file).
- Divergence: `push` is dual-baked from the ground up. No
  `--por` flag is exposed; `PushArgs` (push.rs:127-183) has
  no topology selector. The state machine declares stages
  `CommitClaude`, `BookmarkBoth`, `PushApp`,
  `FinalizeClaude` (push.rs:46-63, 100-113); the helper
  `claude_path(workspace_root)` (push.rs:666-668)
  hard-codes `<root>/.claude` as the session repo; commit
  bodies are stamped with `ochid: /.claude/<chid>` and
  `ochid: /<chid>` (push.rs:1106, 1169). Sanity checks
  (`verify_state_sanity`, `verify_completion_sanity`)
  validate both repos' bookmarks against
  `state.app_chid` / `state.claude_chid`
  (push.rs:1314-1501). Finalize is shelled out against
  `--repo .claude` (push.rs:1265-1290). The 1:1
  symmetric-WC-commits assumption noted in `todo.md > #1`
  lives here.
- Severity / category: *Feature gap* (high). Running
  `vc-x1 push` against a por workspace would try to
  resolve `.claude/...` paths that don't exist.
- Equalization sketch: The bot thinks a por-mode push
  collapses to "preflight → review → message →
  commit-app → bookmark-app → push-app" (drop the
  `*_claude` stages) — a topology check at entry, plus
  conditional stage skipping, would cover most of it; the
  ochid trailer omission is the only nontrivial wrinkle.

## 5. `sync` — partially por-aware

- Files touched:
  - `src/sync.rs:17,160-169` (`resolve_params_to_repos`).
  - `src/common.rs:569-653` (`find_workspace_root`,
    `default_scope`, `scope_to_repos`).
  - `src/sync/integration_tests.rs` (whole file).
- Divergence: Sync is the one place por is a first-class
  case at runtime. `default_scope` (common.rs:604-616)
  returns `Scope([Code])` when there is no workspace root
  (POR) or no `workspace.other-repo` key; returns
  `Scope([Code, Bot])` only when the dual marker is
  present. `scope_to_repos` (common.rs:626-653) resolves
  `Side::Code` to cwd's `.` when `workspace_root` is None,
  so a por run still operates on cwd. The `-R` flag (per
  done #5) lets a por or single-repo path be supplied
  directly (sync.rs:160-169). The privileging that
  remains: the `--scope=bot` path errors out for por
  (common.rs:638-647), as it should, but the *default*
  reads the workspace marker — a por workspace silently
  falls through to "current directory only".
- Severity / category: *Defaulting* (low). Behavior is
  correct for por; the asymmetry is just that dual gets a
  richer default while por gets a fallback default.
- Equalization sketch: Likely no change needed; the
  `default_scope` shape already encodes the topology
  fallback cleanly.

## 6. `finalize` — topology-neutral, dual-shaped use site

- Files touched:
  - `src/finalize.rs` (whole file; spot-checked 1-100,
    580-650).
- Divergence: `finalize` itself is a single-repo operator
  — it takes `--repo <path>` and operates on that one repo
  (squash + bookmark + push, optionally detached). No
  topology branching, no `.vc-config.toml` reads, no
  `.claude/` assumption inside the body. The *callers* are
  dual-shaped, however:
  - `push::stage_finalize_claude` (push.rs:1251-1290)
    invokes it with `--repo .claude`.
  - The user/bot workflow's "one push = one `.claude`
    commit" cadence is dual-only; por has nothing to
    finalize since there is no `.claude/`.
- Severity / category: *Coverage gap* (low). Code is
  topology-neutral; the surrounding workflow is dual-shaped
  by convention.
- Equalization sketch: No changes inside `finalize`. The
  bot thinks the por equivalent is "finalize is unused";
  documenting that explicitly (and/or making
  `push --por` skip stage-finalize-claude) would close the
  workflow gap.

## 7. ochid helpers — dual-assumed unconditionally

- Files touched:
  - `src/desc_helpers.rs:12-42` (`VC_CONFIG_FILE`,
    `other_repo_from_config`, `ochid_prefix_from_config`).
  - `src/validate_desc.rs:121-138` (load other repo, read
    ochid prefix, then validate every commit's trailer).
  - `src/fix_desc.rs:140-160,295-318` (same pattern; also
    `--fallback /.claude/lost`).
  - `src/chid.rs`, `src/desc.rs`, `src/list.rs`,
    `src/show.rs` — neutral; they don't read ochids, only
    accept `-R .claude` as a repo path. The flexibility is
    in shared `CommonArgs` (anchored at `common.rs:674-684,
    `resolve_repos`).
- Divergence: `validate_desc` and `fix_desc` *require* a
  `.vc-config.toml` with `workspace.other-repo`; running
  them in a por workspace immediately errors with
  "missing workspace.other-repo in .vc-config.toml"
  (desc_helpers.rs:18-24) or fails to open
  `<repo>/.vc-config.toml`. There is no opt-out, no
  topology check, no skip-when-por path. The ochid concept
  itself is dual-only (it points across repos), but the
  command-level error message offers no por-aware
  guidance.
- Severity / category: *Feature gap* (medium) +
  *Defaulting* (the commands silently assume dual).
- Equalization sketch: The bot thinks the natural shape
  is: detect por topology at command entry, and either
  no-op with a clear message ("por has no other repo;
  validate-desc has nothing to check") or refuse with a
  topology-aware error pointing at `--por`-equivalent
  workflows. The ochid concept stays dual-only.

## 8. Tests and fixtures

- Files touched:
  - `src/test_helpers.rs:68-150` (`Fixture` — dual),
    `153-220` (`FixturePor`).
  - `src/init/tests.rs` — 9 uses of `Fixture::new`, 5
    uses of `FixturePor::new` /
    `FixturePor::new_with_config` (clear por coverage
    here).
  - `src/push/integration_tests.rs` — 8 uses, **all
    `Fixture::new`**.
  - `src/sync/integration_tests.rs` — 9 uses, **all
    `Fixture::new`**.
  - `src/clone.rs` tests (228-303) — parse-only, no
    end-to-end por fixture exercising `clone_one`.
- Divergence: `FixturePor` exists and is exercised inside
  `init` tests, but no other integration-test surface
  drives a por workspace end-to-end. `push` and `sync`
  integration coverage is dual-only.
- Severity / category: *Coverage gap* (medium). `push`
  has no por tests because it has no por code path
  (compounding); `sync` *does* have a por code path
  (cwd-only resolution) but no fixture asserts it.
- Equalization sketch: The bot thinks `FixturePor` is the
  right starting point — extend it (e.g. add a
  `with_pending`-style variant) and add at least one
  `sync` integration test exercising `default_scope` →
  `Scope([Code])` against a `FixturePor`. `push` por
  coverage waits on push itself growing a por path.

## Summary

| Area | Category | Severity | Equalization size |
| --- | --- | --- | --- |
| 1. `PorFlag` reach | Defaulting + Feature gap | High | L |
| 2. `init` orchestration | Defaulting | Medium | M |
| 3. `clone` orchestration | Defaulting | Medium | M |
| 4. `push` — no por path | Feature gap | High | L |
| 5. `sync` — partial | Defaulting | Low | S |
| 6. `finalize` body | Coverage gap | Low | S |
| 7. ochid helpers / `validate-desc` / `fix-desc` | Feature gap + Defaulting | Medium | M |
| 8. Test fixtures | Coverage gap | Medium | S–M |

Headline: `push` is the largest gap (dual is structurally
baked in — there is no por code path at all), followed by
the runtime ochid helpers (`validate-desc` / `fix-desc`
error out instead of recognizing por). `sync` already
handles por cleanly via `default_scope` / `scope_to_repos`
and is the closest model for "topology from config, not
from a flag." `init` / `clone` are symmetric at the
orchestrator layer but asymmetric at the defaulting layer
(dual is the implicit default, `--por` is the opt-out;
`--config` is por-only). Test coverage mirrors the code
gap: `FixturePor` exists but only `init` exercises it.

## Commonality

The audit above inventoried *divergences*. This section
inverts the view — for each subcommand, what's already
shared between por and dual, what's dual-only, what's
por-only. Equalization is cheapest where the shared bucket
is already large and the dual-only bucket is a thin veneer.

### Per-subcommand buckets

#### `chid` / `desc` / `list` / `show` — fully shared

- **Shared (all)** — these four route through the
  `CommonArgs` + `common::for_each_repo(&c.repos, ...)`
  shape (`src/chid.rs:53`, `src/desc.rs:68`,
  `src/show.rs:118`, `src/list.rs`). Repos resolve via
  `default_scope` + `scope_to_repos` (`src/common.rs:594`,
  `:618`), which already handle por gracefully
  (`Scope([Code])` when `other-repo` is missing/empty).
- **Dual-only** — none in the runtime body. The dual case
  just adds a second entry to the repo list.
- **Por-only** — none.

The bot thinks this family is the *template* for the rest
of the codebase. Scope-driven iteration is the working
model of topology-neutral dispatch.

#### `sync` — fully shared

- **Shared (all)** — same scope-driven shape as the
  `CommonArgs` family; per the 0.54.0 cleanup
  (`notes/todo.md > ## Done`) sync got `-R` and routes
  through `default_scope` / `scope_to_repos`.
- **Dual-only** — none.
- **Por-only** — none.

#### `validate-desc` / `fix-desc` — dual-only outliers

- **Shared** — title-matching, ochid extraction, and the
  per-commit walk (`src/desc_helpers.rs` exports
  `extract_bare_id`, `find_matching_commit`,
  `validate_ochid`, etc. and these are topology-neutral).
- **Dual-only** — the entry shape. Both call
  `other_repo_from_config(&config)?` directly
  (`validate_desc.rs:133`, `fix_desc.rs:152`), which errors
  on por instead of resolving to a no-op or single-repo
  validation.
- **Por-only** — none.

The bot thinks equalization here is local: convert the
dual-required prelude into a `default_scope`-style
resolution that no-ops `Side::Bot` when absent. The shared
body doesn't need changes.

#### `init` / `clone` — shared body, dual default

- **Shared** — repo-creation primitives (`init_one` /
  `clone_one` per `chores-07.md > init + clone redesign
  (0.41.1)`), GitHub remote provisioning, push retries,
  template seeding mechanism, `--account` / `--repo`
  resolution via `config.rs`.
- **Dual-only** — the orchestration outer loop runs twice
  (code + bot) when `--por` is absent; `.vc-config.toml`
  write; `.claude/` directory creation; `--use-template
  <code,bot>` accepts a bot value.
- **Por-only** — `--config <none|PATH>` (overrides the
  canned `.vc-config.toml` write, only meaningful when
  there'd be one).

The asymmetry is at the *defaulting* layer (dual is the
implicit default; `--por` is the opt-out) more than the
*code* layer. The body is already roughly symmetric.

#### `push` — dual-only

- **Shared** — bookmark tracking, the push state machine's
  generic stages (`prepare`, `commit-app`, `push-app`),
  the retry/resume scaffolding around `.vc-x1/push-state.toml`.
- **Dual-only** — `claude_path()` resolution; the
  `CommitClaude` / `FinalizeClaude` stages; ochid trailer
  composition (`ochid: /.claude/<chid>` on app, multi-line
  `ochid: /<code-chid>` on bot); the `--from
  bookmark-both` flag; the 1:1 symmetric WC-commits
  assumption flagged in T1.
- **Por-only** — none.

The largest gap. The bot thinks no por code path exists
*at all* today; a por workspace running `vc-x1 push` would
resolve nonexistent `.claude/` paths during the
`CommitClaude` stage.

#### `finalize` — body shared, use site dual

- **Shared (all)** — the body is a single-repo operator
  (squash + push + cleanup). Topology-neutral.
- **Dual-only** — none in the body. The dual shape is
  *external*: `push` schedules a detached `finalize`
  against `.claude` only after `push-app` succeeds.
- **Por-only** — none.

Once `push` gains a por path, `finalize` requires no
changes.

#### Tests and fixtures

- **Shared** — `Fixture` / `CliFixture` provide a dual
  workspace; `FixturePor` provides a por workspace; both
  honor `$VC_X1_TEST_TMPDIR` / `$VC_X1_TEST_KEEP` and the
  RAII drop pattern.
- **Dual-only** — every integration test for `push` (8
  uses) and `sync` (9 uses); the `chid` / `desc` / `show` /
  `list` test paths (these route through `CommonArgs`,
  which works on por, but the fixtures don't cover that
  case).
- **Por-only** — five uses in `init/tests.rs` exercising
  the `--por` initialization path.

The bot thinks the test-coverage gap mirrors the runtime
gap exactly: `push` and the desc outliers have zero por
coverage because the runtime code doesn't support por
there; the `CommonArgs` family has zero por coverage
despite supporting it.

### Equalization candidates, ranked

Ordered from closest-to-shared (cheap wins) to
furthest (architectural work):

1. **`validate-desc` / `fix-desc`** — local refactor:
   replace the `other_repo_from_config` prelude with a
   scope-aware resolution that no-ops when `Side::Bot`
   is absent. Body unchanged. The bot thinks this is the
   smallest concrete equalization and a good prototype.
2. **`CommonArgs`-family por test coverage** — pure test
   work; surface bugs in scope handling without
   architectural risk. Likely lands a `FixturePor` use
   in `chid`/`desc`/`show`/`list` test modules.
3. **`init` / `clone` defaulting** — make `--por` /
   `--dual` peer flags rather than dual-default + `--por`
   opt-out; thread the chosen topology through the body
   without changing the inner primitives. User-config
   `[default].topology` (the original user proposal)
   becomes a small follow-on once peers exist.
4. **`push`** — the structural one. `claude_path()`,
   stage names, ochid composition all carry the
   dual-shape baked in. The bot thinks this is best done
   *after* `--por` becomes a runtime-known fact across
   all subcommands (i.e. after the desc outliers and
   defaulting are handled), so push has a stable contract
   to dispatch against.
5. **Topology-from-config rule** — codify (CLAUDE.md or a
   small ARCHITECTURE note) that every runtime subcommand
   resolves topology via `default_scope`, never from a
   `--por` flag. The flag stays creation-time only. The
   bot thinks this rule is the right *outcome* of the
   equalization above, not a prerequisite — codify it
   once the prototype validates the shape.

### Summary

Three structural classes today:

- **Topology-neutral via scope** —
  `chid` / `desc` / `list` / `show` / `sync` / `finalize`-body.
  This is the working pattern.
- **Topology-required (dual)** —
  `push` / `validate-desc` / `fix-desc`. These bypass
  scope and assume `.claude/` exists.
- **Topology-creating** — `init` / `clone`. The `--por`
  flag's only legitimate home; chooses the workspace
  shape that downstream commands then read from
  `.vc-config.toml`.

Equalization is "make Topology-required match
Topology-neutral via scope" — not a new pattern, just
extending an existing one. The smallest concrete win is
`validate-desc` / `fix-desc`; the largest is `push`. The
two creation-time commands stay roughly as-is, with the
defaulting layer (peer flags + optional user-config
default) as the only change.

## Feature axes

The audit and commonality passes treat `--por` and `--dual`
as the primitives. They aren't — they're bundles. The
concrete shape `vc-x1 init --por <name>` produces today is
"single repo, with a (degenerate) `.vc-config.toml`, pushed
to a freshly-created GitHub repo." Each of those is an
independent choice, but the user can only ask for them as a
bundle.

This section names the independent axes, says what each
controls, lists today's flags and defaults, and identifies
gaps. **No implementation — defining only.** Equalization
across these axes lands in 0.62.0+ cycles.

### Axes

#### A1. Topology

- **States** — `single` (one code repo) | `dual` (code repo
  + `.claude/` companion cross-linked by `ochid:`).
- **Today** — dual is the implicit default; `--por`
  is the boolean opt-out. No `--dual` peer.
- **Surface** — `--por` (boolean, `init` and `clone` only).
  Downstream commands infer from `.vc-config.toml >
  [workspace] other-repo` via `default_scope`.
- **Gap** — `--dual` doesn't exist as a peer flag, so the
  defaulting is asymmetric: you can be explicit about por
  but not about dual.

#### A2. `.vc-config.toml` write — collapsed

A2 was originally written as an independent axis. After
the A1 decisions, **A2 collapses** — presence of the
workspace `.vc-config.toml` is perfectly correlated with
A1's topology choice:

- A1=por → no `.vc-config.toml` (por never reads or
  writes the workspace file).
- A1=dual → `.vc-config.toml` is mandatory (it's what
  detects dual at runtime).

A2 is **not an independent axis**.

The capability `--config <path>` was meant to serve
(custom workspace metadata, arbitrary file copy) is
deferred to a broader **copying** design — see
[`notes/copying.md`](copying.md) [[3]]. That design uses
`--init-from-code` / `--init-from-bot` flags to copy
arbitrary files (including `.vc-config.toml` and
`.gitignore`), suppresses canned writes when engaged,
and defers the "is this dual workspace functional?"
check to the first downstream subcommand.

Today's `--config <none|PATH>` flag (`init.rs:610`,
"only valid with `--por`") becomes vestigial under the
new design. The 0.62.0+ rollout drops it.

#### A3. Remote provisioning

- **States** — `github-create` (default; creates
  `winksaville/<name>` via gh CLI) | `github-skip` (URL
  is non-GitHub or pre-existing) | `local-bare` (`git
  init --bare` under `--repo local=<dir>`) | `none`
  (no remote configured at all — just the working repo).
- **Today** — `--repo <cat>[=<val>]` resolves to one of
  the first three via the user-config `[account.<a>.
  repo.category]` lookup; `none` doesn't exist.
- **Surface** — `--repo` for the cat/val, `--private` for
  GitHub visibility, `--push-retries` /
  `--push-retry-delay` for the post-create push.
- **Gap** — no flag for "no remote." A user wanting a
  local workspace (or to provision the remote separately)
  has no clean way to express it.

#### A4. Private vs public

- **States** — `public` (default) | `private`.
- **Today** — `--private` flag.
- **Surface** — `--private`.
- **Scope** — only meaningful when A3 = `github-create`.
- **Gap** — none structurally; the flag is independent
  and orthogonal.

#### A5. Template seeding

- **States** — `none` (default) | `code-only <path>` |
  `code-and-bot <path,path>`.
- **Today** — `--use-template <CODE[,BOT]>`. Second value
  is dual-only.
- **Surface** — `--use-template`.
- **Gap** — none structurally; per-side templating is
  naturally A1-aware. Bot-side template under A1=single
  is meaningless and rejected today.

#### A6. Working-tree scaffolding (jj init, `.gitignore`)

- **States** — `on` (default) | `off`.
- **Today** — always `on`. No flag.
- **Gap** — probably not worth a knob. jj is the workspace
  scaffolding; opting out doesn't yield a meaningful
  workspace. `.gitignore` is unconditional and a fixed
  content list. The bot thinks A6 stays a non-axis
  unless a concrete use case surfaces.

### Defaults summary

| Axis | Default | Today's flag | Independent today? |
| --- | --- | --- | --- |
| A1 Topology | dual | `--por` (opt-out only) | No (no `--dual` peer) |
| A2 `.vc-config.toml` | written | `--config <none\|PATH>` | No (rejected when dual) |
| A3 Remote | github-create | `--repo <cat>[=<val>]` | Mostly (`none` missing) |
| A4 Private | public | `--private` | Yes |
| A5 Template | none | `--use-template <C[,B]>` | Yes (A1-aware) |
| A6 Scaffolding | on | (none) | Not an axis today |

### Mapping `--por` / `--dual` onto axis combinations

Today `--por` and (implicit) `--dual` are *bundle
shorthands*. After axes are independent, they remain as
shorthands for the common combinations:

- `--dual` (or no flag — back-compat) → `(A1=dual,
  A2=written, A3=github-create, A4=public, A5=none)`.
- `--por` → `(A1=single, A2=written, A3=github-create,
  A4=public, A5=none)`. Note A2=written is what today's
  `--por` actually does, not `not-written` (the
  `.vc-config.toml` is still written, just degenerate).

A user wanting a fully-plain single repo today can't
spell it; tomorrow it would be `--por --config none --no-remote` (or
shorthand `--bare` if that combination is common enough to
deserve one).

### Connections to user-config

Once axes are independent, each maps to a `[default].*`
key in `~/.config/vc-x1/config.toml`, so the user-config
proposal that opened this cycle lands cleanly:

- `[default].topology = "single" | "dual"`
- `[default].write-vc-config = true | false | "<path>"`
- `[default].remote = "github" | "local" | "none"`
- `[default].private = true | false`

`--account`-scoped overrides ride on the existing
`[account.<a>]` substrate. The bot thinks A3 is the most
useful default to make user-configurable (different
accounts → different remote providers), with A1 second.

### Resolution chain

The per-axis sections above each say "CLI → user-config →
error." That's a sketch; the full resolution chain has
four layers plus an explicit-required floor. The same chain
applies to **every axis** and every field a config carries
(account, repo category/value, private, topology, …) —
nothing axis-specific.

#### Layers (highest precedence first)

1. **CLI flag** — `--por`, `--account`, `--private`,
   `--config <path>`, `--global-config <path>`, etc. The
   per-invocation surface. CLI is god — it wins over
   every other layer when present.
2. **Environment variable** — `VC_X1_<KEY>`. The
   per-session surface, shaped by `export` or one-shot
   prefix. Loses to CLI.
3. **Local config** — `./.vc-config.toml` (the workspace
   metadata file), or whatever `--config <path>` resolved
   to. Per-project durable intent. Loses to env-var and
   CLI; **may carry any field a CLI flag exposes** — no
   carve-outs (a local config can pin
   `[global] config-path` just as it pins `account`).
4. **Global config** — `~/.config/vc-x1/config.toml`
   (XDG-aware), or whatever `--global-config <path>` /
   `VC_X1_GLOBAL_CONFIG` / a local-config
   `[global] config-path` resolved to. Per-user durable
   defaults. Lowest-precedence config; loses to all
   above.
5. **Error** — "no `<key>` specified; set it via CLI,
   `VC_X1_<KEY>`, `./.vc-config.toml`, or
   `~/.config/vc-x1/config.toml`."

Each layer is **optional**. Per-axis behavior under
the chain: the resolved value of a key is the value from
the highest layer that defines it; absence at every layer
falls to the error floor (or the axis's safe default if
one is defined — A5 template defaults to `none` because
"no template" is the only meaningful absence; no other
axis has a safe default today).

#### Escape hatches

- **`--no-local-config`** — skip layer 3 entirely for this
  invocation. Today functionally equivalent to runtime
  `--por` (because local config carries only topology
  metadata); add as a peer flag once local config gains
  non-topology fields.
- **`--no-global-config`** — skip layer 4 entirely.
  Necessary for tests / CI / "ignore my defaults this
  once."

Both flags are layer-1 (CLI) — they can't themselves be
pinned by config, since their purpose is to ignore config.

#### Redirection and cycles

Both local and global config can redirect to a different
config file via `[global] config-path = "…"` (in either
file) and `[local] config-path = "…"` (in global).
**Circular redirections are user error** — the loader
keeps a visited-set and errors with the cycle path; no
attempt is made to break the cycle automatically.

The bot thinks redirection in local pointing at a
custom global is the most useful case (a project that
ships a team-wide default file in the repo), and global
pointing at another global is mostly an unintended
consequence of letting the field exist universally.
Allowing it costs nothing.

#### Env-var naming

`VC_X1_<KEY>`, flat namespace, key matches the resolved
field name:

| Axis | Env-var | Resolves to |
| --- | --- | --- |
| A1 | `VC_X1_TOPOLOGY` | `single` \| `dual` |
| A3 account | `VC_X1_ACCOUNT` | account name |
| A3 repo | `VC_X1_REPO` | `<cat>` or `<cat>=<val>` |
| A4 | `VC_X1_PRIVATE` | `true` \| `false` |
| A5 | `VC_X1_USE_TEMPLATE` | path or `<path,path>` |
| — | `VC_X1_CONFIG` | local config path or `none` |
| — | `VC_X1_GLOBAL_CONFIG` | global config path |
| — | `VC_X1_NO_LOCAL_CONFIG` | `true` \| `false` |
| — | `VC_X1_NO_GLOBAL_CONFIG` | `true` \| `false` |

Precedent in the codebase: `config.rs:113` already honors
`XDG_CONFIG_HOME`; `test_tmp_root.rs` uses
`VC_X1_TEST_TMPDIR` / `VC_X1_TEST_KEEP`. The pattern is
established; `VC_X1_*` just extends it.

#### Por's view of the chain

Following the decisions in this cycle:

- **Workspace `.vc-config.toml`** (layer 3) — dual-only.
  Por never reads, never creates. Today's degenerate
  `[workspace] path = "/"` write under `--por` is residue
  from when `--por` was a half-implemented opt-out and
  should be dropped.
- **User-config `~/.config/vc-x1/config.toml`**
  (layer 4) — topology-neutral. Both por and dual consult
  it.
- **Runtime `--por`** — overrides workspace topology
  detection across every subcommand (not just
  `init`/`clone`). In a dual workspace, runtime `--por`
  short-circuits `default_scope` → `Scope([Code])` and
  ignores `.claude/`.

### Gap list (input for close-out)

The concrete gaps for follow-up cycles to seed Todos
against:

1. **A1 has no `--dual` peer** — asymmetric defaulting.
   Add `--dual` (alias of "no `--por`") for explicit
   parity; allow `--por`/`--dual` exactly one of.
2. **A2 errors on dual** — change the error to
   "(A1=dual, A2=not-written) is impossible because dual
   needs `.vc-config.toml` for runtime topology" rather
   than the current flag-restriction message; allow
   `--config <path>` (override) under `--dual`.
3. **A3 missing `none`** — add a way to spell "no remote."
   Probably `--repo none` (orthogonal to the existing
   `remote`/`local` categories) or a separate `--no-remote`
   flag. The bot thinks `--repo none` keeps the
   one-knob-for-A3 shape.
4. **A6 is non-negotiable today** — confirm at close-out
   whether to leave it that way; if so, drop A6 from the
   axis list.
5. **User-config keys** — once A1–A5 are independent,
   wire `[default].topology` / `.write-vc-config` /
   `.remote` / `.private` into the `init` resolution
   chain. Follow the three-step `resolve_repo` shape
   (CLI → `[default]` → error).

These five become candidate `## Todo` entries at
close-out, ranked by the equalization-candidate ordering
from `## Commonality` (axis fixes that overlap with
`validate-desc` / `fix-desc` equalization land cheapest).

# References

[1]: /notes/todo.md
[2]: /notes/por-dual-parity.md
[3]: /notes/copying.md
