# Chores-07.md

General chores notes — design captures (forward-looking) and
post-implementation chore entries. Same shape as chores-01..06.md;
07 starts here only because 06 grew large during the 0.40.0 /
0.41.0 cycles and a new top-level cycle is a natural file
boundary.

Subsection headers use the trailing-version format from CLAUDE.md
when they correspond to a release: `## Description (X.Y.Z)`.
Pre-implementation design captures may use a plain title; once
implemented, the title can become a release-versioned chore.

## --scope enum refactor (0.42.0)

Picks up the work the 0.41.0 cycle redirected away from. The
foundation captured in [2] (chores-06's `### 0.41.0-4: capture
--scope enum vocabulary`) lands here as code: the `Scope`
type becomes a sum (`Roles(Vec<Side>) | Single(PathBuf)`), the
flag surface unifies under `--scope` / `-s` with both keyword and
path forms, and every dual-repo-aware command migrates to the
new shape in dependency order.

**Cycle steps (initial sketch — sub-step boundaries can shift
once each command's call sites are seen).**

- **0.42.0-0** — this plan + version bump + new
  `notes/chores-07.md`. Notes only.
- **0.42.0-1** — `scope.rs` enum. Internal only — no
  CLI changes yet, no consumers updated. Existing
  `Scope(Vec<Side>)` → `enum Scope { Roles(Vec<Side>),
  Single(PathBuf) }`. Helpers (`has_code`, `has_bot`,
  `is_code_only`, `is_both`, `is_empty`) shift to operate
  on `Roles` only; `Single(_)` is a distinct mode. Tests
  follow.
- **0.42.0-2** — custom CLI value parser for `--scope` +
  retrofit `init`. Parser handles the keyword set
  (`code|bot|code,bot|bot,code`) and the prefixed-path
  form (`./...`, `/...`, `~/...`). Bare names that aren't
  keywords error with a "did you mean `./foo`?" hint.
  Init is the smallest consumer (it already takes
  `--scope=code|bot|code,bot`); migrating it first proves
  the parser end-to-end before sync's more complex
  resolution logic comes onto the new shape.
- **0.42.0-3** — retrofit `sync`. Drop `-R/--repo`. Add
  `-s` short form. Default-scope rules update to the
  three-state model from [2]: dual workspace →
  `Roles([Code, Bot])`, single-repo workspace →
  `Roles([Code])`, POR → `Single(cwd)`. `scope_to_repos`
  in `common.rs` updates to handle the `Single(_)`
  variant. Update sync's integration tests where they
  exercise the resolver chain.
- **0.42.0-4** — `push --scope`. State machine becomes
  scope-aware: each stage either runs or no-ops based on
  scope. `Single(<path>)` mode means single-repo push
  (no `commit-claude` / `bookmark-claude` /
  `finalize-claude`, no `ochid:` trailer). `Roles(...)`
  modes work as captured in chores-06 [1]. Persist scope
  in `PushState` so resumes use the same scope. Add
  integration tests for each scope shape.
- **0.42.0-5** — `finalize --scope`. Replace the existing
  `--repo` flag with `--scope` (`Roles` for the dual case,
  `Single(<path>)` for the single-repo case). `--repo`
  goes away; per the 0.41.0-3 capture this is the
  intentional break, not a deprecation.
- **0.42.0-6** — `clone --scope`. Parallel to init —
  bootstrap with the appropriate scope. Single-repo clone
  via the path form for `vc-template-x1`-shape remotes.
- **0.42.0-7** — Single(_) dogfood validation. Apply the
  full `sync → edit → push` flow against a fresh POR or
  single-repo fixture (likely the `vc-template-x1` repo
  itself, since it's the canonical single-repo target).
  Surface and fix anything the unit tests miss.
- **0.42.0 (final)** — cycle close-out. Drops the
  `-N` suffix; updates `notes/todo.md`'s In Progress and
  Done sections; chores-07 close-out subsection.

**Deferred to later cycles (per [2]).**

- `validate-desc` / `fix-desc` `--scope`. Read-side
  commands; `Single(_)` errors there (validate compares
  two repos by definition).
- `chid` / `desc` / `list` / `show` — CommonArgs sweep.
  All four pick up `--scope` via a shared change to
  `CommonArgs`; existing `-R/--repo` retires there too.
- `--message-file PATH` for push. Independent push
  feature; gates the CLAUDE.md refresh from CLAUDE2.

**References.** [2] points back at chores-06's
`### 0.41.0-4: capture --scope enum vocabulary` for
the full vocabulary, type-model, and per-command
applicability matrix. Read that subsection first before
diving into any of the `-N` steps below.

### 0.42.0-1: scope.rs enum

Internal-only refactor: tuple-struct `Scope(Vec<Side>)` →
`enum Scope { Roles(Vec<Side>), Single(PathBuf) }`. No CLI
surface changes, no consumer logic changes — every existing
caller still constructs and consumes `Roles(_)`; `Single(_)`
is staged for the parser (-2) and the sync resolver (-3).

- `src/scope.rs`: enum body + `PathBuf` import; helpers
  (`has_code` / `has_bot` / `is_*_only` / `is_both` /
  `is_empty`) reflect the `Roles` arm via `matches!`,
  returning `false` (resp. not-empty) on `Single(_)`. New
  test pins the `Single(_)` helper behavior; existing
  Roles tests retitled accordingly. `#[allow(dead_code)]`
  on the `Single` variant for the staging window.
- `src/common.rs`: `default_scope` returns
  `Scope::Roles(_)`; `scope_to_repos` matches on the enum
  and returns an explicit error for `Single(_)` carrying
  the `0.42.0-3` staging marker. New test locks that
  contract so the -3 wire-up is forced to update it.
- `src/sync.rs`: `Some(sides) => Scope::Roles(sides.clone())`
  in `resolve_args_to_repos`; integration-test
  constructors switch to `Scope::Roles(...)`.
- `src/init.rs`: `plan_init` constructs `Scope::Roles(...)`
  for both the default and explicit-flag paths. Helper
  call sites (`is_code_only`, `is_both`, `is_bot_only`)
  unchanged — the methods continue to work on `Roles`.

Build is warning-clean; 257 tests pass (255 baseline + 2
new in scope/common).

### 0.42.0-2: --scope parser + retrofit init

Lands the custom CLI value parser and migrates `init` onto
the new `Scope` shape. Parser is the shared boundary every
remaining `--scope` consumer (sync, push, finalize, clone)
will route through in -3+; init is the smallest first
consumer and exercises the keyword path end-to-end. The
path form is accepted by the parser but rejected at init
with a hint pointing at `--repo-local` / `--repo-remote`
(init creates a workspace, not a single-repo project).

- `src/scope.rs`: `pub fn parse_scope(s: &str) -> Result<Scope, String>`.
  Accepts the four keyword forms (`code`, `bot`, `code,bot`,
  `bot,code`) plus path forms prefixed by `./`, `../`, `/`,
  `~/`, or the bare `~`. Anything else errors with a hint
  pointing at the prefix-disambiguation rule. Order is
  preserved on the keyword forms. Drops the now-dead
  `is_empty` helper (parser rejects empty input upstream;
  the only consumer that called it was init's redundant
  validation). 13 new parser unit tests.
- `src/init.rs`: `--scope` field flips from
  `Option<Vec<Side>>` (clap `value_delimiter = ','`) to
  `Option<Scope>` with `value_parser = crate::scope::parse_scope`.
  `plan_init` matches on the parsed `Scope`: `Roles(_)`
  takes the existing bot-only-fatal path; `Single(_)` is
  rejected with the init-specific hint. Test scaffolding
  (`fixture_scoped`) and 5 fixture call sites migrated.
  `scope_parses_*` tests retitled around the new field
  type. New `scope_path_form_rejected_at_init` test pins
  the rejection contract.

Smoke-tested end-to-end via CLI:

- `vc-x1 init tf1 --scope code --dry-run` → succeeds
  (single-repo dry-run plan).
- `vc-x1 init tf1 --scope code,bot --dry-run` → succeeds.
- `vc-x1 init tf1 --scope bot --dry-run` → bot-only fatal
  (existing `plan_init` check).
- `vc-x1 init tf1 --scope ./foo --dry-run` → path-form
  fatal (new `plan_init` check).
- `vc-x1 init tf1 --scope '~/work' --dry-run` → path-form
  fatal.
- `vc-x1 init tf1 --scope foo --dry-run` → clap-level
  parser error with hint about `./foo` disambiguation.

# References

[1]: /notes/chores-06.md#--scope-continuation-0410
[2]: /notes/chores-06.md#0410-4-capture---scope-enum-vocabulary
