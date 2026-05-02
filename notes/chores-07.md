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

## Substep protocol formalization (0.42.0-4.5)

Notes-only side cycle. Branched off `main` at 0.42.0-4
(init+clone redesign capture); landed between 0.42.0-4
and the planned 0.42.0-5 (finalize --scope). Formalizes
the substep protocol first exercised in 0.41.1-6.5,
validates the close-out squash recipe, and pulls the
supporting jj revset vocabulary into a sibling cheatsheet
so the protocol can refer to it without duplicating.

Version path: numbered `0.42.0-4.5` rather than `0.42.1-0`
to avoid pre-empting the patch line while 0.42.0 is
mid-cycle. The `.5` suffix positions this as side work
between -4 and -5 — semver-legal (pre-release identifiers
are dot-separated numeric components and compare
numerically) and keeps Cargo.toml's version monotonically
increasing.

The user opened the cycle by drafting `notes/jj-revsets.md`
from their own learning notes (using `notes/substep-test.sh`
to build a scratch ladder under `/tmp/substep-test`). The
bot reviewed, proposed the substep protocol formalization,
and the work landed in five substeps using the protocol it
was documenting — dogfood validation in flight.

### Edits

- `notes/jj-revsets.md`: new. Revset primitives cheatsheet
  (chid/cid stability, `@`/`@-`/`@+`, `..`/`::` ranges,
  prefix matching). User-authored intro and worked
  examples; bot-added `### Interpretation` blocks
  cataloguing the operator semantics from the empirical
  data, two review passes for typos and grammar.
- `notes/substep-protocol.md`: new. Formal protocol of
  record. Five sections — purpose, per-substep contract
  (`cargo test --bins` non-negotiable), navigation
  (cross-link to jj-revsets.md), close-out, recovery —
  plus a worked end-to-end example. Drops the prior
  "draft text / working hypothesis" hedges. Close-out
  recipe validated against scratch ladders at N=3 and N=5
  substeps:
  ```
  jj squash --from "<base>..@-" --into @ -u -R .
  ```
- `notes/substep-test.sh`: new. Four-revision ladder
  scaffold under `/tmp/substep-test`. Used to validate
  the squash recipe; remains as a reusable scratch tool
  for future substep experiments.
- `CLAUDE.md`: `#### Substeps within a multi-step X.Y.Z-N`
  subsection added under `### Versioning`. Points at
  `notes/substep-protocol.md` (full procedure) and
  `notes/jj-revsets.md` (revset primitives the procedure
  relies on). Conservative integration scope; broader
  CLAUDE.md reorg candidates (consolidating the
  `## Committing` / `## Commit Message Style` /
  `## ochid Trailers` triplet, surfacing recovery as its
  own section, top-of-file TOC) surfaced in the substep-4
  commit body for next-cycle discussion.
- `notes/todo.md`: `## Done` entry; reference [73] points
  here.
- `notes/chores-07.md`: this subsection.

### Substep ladder (squashed at close-out)

- substep 0: `notes/jj-revsets.md` review fixes (typos,
  two `### Interpretation` blocks for relative- and
  absolute-revset operators).
- substep 1: validate close-out squash recipe against
  N=3 and N=5 ladders in `/tmp/substep-test`. No file
  changes; finding folded into the next substep.
- substep 2: `notes/substep-protocol.md` formal rewrite.
- substep 3: `CLAUDE.md` pointer subsection.
- substep 4: second-pass review fixes after the user's
  in-place tweaks added the concrete `lu::` example and
  surfaced typos in the absolute-revsets bullet list.
- close-out: this commit. Single squashed commit on
  `main` via the validated recipe.

### Decisions made during design

- **`-R .` explicitness.** The protocol shows `-R .` on
  every `jj` invocation. For the bot in a dual-repo
  workspace this is correct (always be explicit about
  which repo); for human users in a single-repo project
  the flag is optional. Closing note in the protocol
  spells this out.
- **Linear vs parallel substep topology.** The ladder is
  linear (`jj new` chains commits). User noted at
  close-out that a parallel-then-merge topology
  (`jj new -r <chid>` to branch from a specific point)
  might give cleaner per-concern diffs. Captured as a
  future experiment; the close-out recipe would change
  too — `<base>..@-` assumes linear ancestry.
- **`jj op restore` as recovery vs close-out.** The draft
  protocol had floated `jj op restore` as a close-out
  alternative. Validation showed it discards substep
  work — it's a recovery tool, not close-out. Final
  protocol keeps it under Recovery only.
- **`-u` on the close-out squash.** `--use-destination-message`
  preserves `@`'s description through the squash (which
  `vc-x1 push`'s `commit-app` then overwrites with the
  close-out title/body anyway). Without `-u`, jj opens
  `$EDITOR` to combine source and destination
  descriptions — unwanted at close-out, since none of
  the substep titles are meant to ship.

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

### 0.42.0-3: sync --scope retrofit

Migrates `sync` onto the new `Scope` shape and lands the
`Single(_)` resolver wiring. `-R/--repo` retires (single-
repo callers move to `--scope=<path>`); `-s` becomes the
short form of `--scope`. Default-scope rules pick up the
three-state model captured in chores-06's [1] (-4
vocabulary): dual workspace → `Roles([Code, Bot])`,
single-repo workspace → `Roles([Code])`, POR →
`Single(cwd)`.

- `src/common.rs`: `default_scope` signature gains a `cwd`
  parameter; the POR branch (no workspace_root) now
  returns `Scope::Single(cwd.to_path_buf())` instead of
  `Roles([Code])`. `scope_to_repos`'s `Single(p)` arm
  resolves to `vec![p.clone()]` (no workspace lookup;
  shell-style expansion happens at the parser/consumer
  boundary). The `0.42.0-3` staging-error test is
  replaced with `scope_to_repos_single_returns_path` —
  the happy-path contract — and `default_scope_por`
  becomes `default_scope_por_returns_single_cwd`.
- `src/sync.rs`: `--repo`/`-R` flag and the `repos:
  Vec<PathBuf>` field deleted. `--scope` retypes from
  `Option<Vec<Side>>` (with `value_delimiter = ','`) to
  `Option<Scope>` (with `value_parser = parse_scope`)
  and gains `short = 's'`. SyncArgs doc rewritten around
  the new resolution rules. `resolve_args_to_repos`
  drops the `args.repos` branch and now feeds `cwd` into
  `default_scope`. `split_repos` and its 4 unit tests
  go away with `-R`; `parse_scope_repo_conflict`,
  `parse_single_repo_flag`, `parse_repeated_repo_flag`
  too. `parse_scope_*` tests assert against `Some(Scope)`;
  new `parse_scope_path_form` and `parse_scope_short_form`
  tests pin the new entry points. `apply_args` integration
  helper drops the `repos` field. `Side` import is gated
  `#[cfg(test)]` (production sync no longer needs it
  directly — only the tests construct `Side::*`).

Smoke-tested end-to-end:

- `vc-x1 sync --check` in this dual workspace → `2 repos,
  all bookmarks up-to-date`.
- `vc-x1 sync --check -s code` → `1 repo, all bookmarks
  up-to-date`.
- `vc-x1 sync --check -s bot` → ditto for `.claude`.
- `vc-x1 sync --check -s ./.claude` → also `1 repo` —
  Single-mode resolves directly to that path, bypassing
  the workspace's other-repo lookup.
- `cd /tmp && vc-x1 sync --check` → POR detected,
  attempts to sync `/tmp` (errors because `/tmp` isn't a
  jj repo — confirms `default_scope` returned
  `Single(cwd)`).

## init + clone redesign (0.41.1)

Empirical validation 2026-04-27 against `vc-x1 0.42.0-3` (see
`notes/vc-x1-init.md`) surfaced cosmetic anomalies and a
substantive design gap. Init's flag surface (`--repo-local`,
`--repo-remote`, `--owner`, `--dir`, `[NAME]`) carries 6+
mutually-exclusive preflight checks just to prevent impossible
combinations from being typed. Clone already implements a
unified-positional alternative (URL / `owner/name` shorthand);
this cycle extends that pattern to init, adds POR support to
both, and unifies their CLI surfaces.

Branched off 0.41.0 (commit `6747a27`). On close-out, the
in-flight 0.42.0 work rebases on top.

### Command structure

```
vc-x1 init  <TARGET> [NAME] [--scope code,bot|por] [--private] [--dry-run]
vc-x1 clone <TARGET> [NAME] [--scope code,bot|por]             [--dry-run]
```

Identical surfaces modulo `--private` (init only).

`<TARGET>` accepts:

- **URL** — `git@host:owner/name(.git)?`, `https://...(.git)?`
  (detected by `://` or SSH `<host>:<path>` shape with `@`).
- **`owner/name` shorthand** — single `/`, no path prefix;
  resolved to `git@github.com:owner/name.git`.
- **Path-prefixed** — `./X`, `../X`, `/X`, `~/X`, `~`. Path
  IS the target; last component is the workspace name. Bare
  `.` and bare `NAME` are errors (require explicit prefix).

`[NAME]`:

- For URL / `owner/name` forms: overrides the derived
  destination dir name. `vc-x1 clone owner/foo my-name`
  clones into `./my-name/` instead of `./foo/`.
- For path-prefix form: error if combined (path already
  specifies the full target).
- Path-form for clone: error (clone needs a remote URL;
  see "Operations" for clone's target rules).

`--scope`:

- `code,bot` (default): both repos, dual-repo layout.
- `por`: single repo, no `.vc-config.toml` written.
- `code` / `bot` standalone: error. Reason — these are
  config-lookup keywords; init has no config to look up
  against, and clone's simplification omits them too. The
  manual decomposition (clone code as POR, clone bot as
  POR, place at `.claude/`, run `vc-x1 symlink`) covers
  the rare cases where a user wants the dual layout
  composed by hand.

`--private`: only meaningful for init when creating a new
GitHub repo. Sets visibility at `gh repo create`. Silently
warn-and-ignore if the remote already exists. Errors on
clone (clone never creates a remote).

`--dry-run`: validate the operation as far as possible
without side effects. Print what would be done.

### Operations

**Init** is *smart* — it detects the target dir's state and
chooses the right operation.

| Target state | `--scope=code,bot` | `--scope=por` |
|---|---|---|
| empty / nonexistent | create dual-repo from scratch | create POR |
| POR (jj+git, no config) | upgrade to dual | error (already POR) |
| POR (git only, no `.jj/`) | bootstrap `jj git init`, then upgrade | error (already POR-shape) |
| code POR + `.claude/` peer POR (configs missing) | write configs only | error |
| dual-repo (configs present) | error (already there) | error (downgrade) |
| single-repo (illegal shape) | error | error |
| anything else (random files, etc.) | error with diagnostic | error with diagnostic |

**Clone** is *dumb* — it just clones URLs into target dirs.

| `--scope=por` | clone the URL into `<target>` |
| `--scope=code,bot` | clone code URL into `<target>`, derive bot URL (`<URL>.claude`), clone into `<target>/.claude/`, then create symlink via `vc-x1 symlink` |

For both clone modes: target must NOT exist (`vc-x1 clone`
errors if anything is there). Use `vc-x1 sync` to update an
existing checkout.

`.vc-config.toml` files for the dual-repo case are *tracked
content in the remotes*, not files clone writes. They ride
along with the clone. If a remote doesn't have them
committed, `vc-x1 init <path> --scope=code,bot` from inside
the cloned dir handles the config-only upgrade case (see
init's state table above).

`--scope=code,bot` is implemented as in-process composition
of the `--scope=por` primitive (called twice: once for code
URL, once for derived bot URL) plus a `symlink::create`
call. No subprocess overhead.

If init or clone fails partway through, manual cleanup is
required (no rollback).

### Example Layout local repos

cwd = ~/prgs

`vc-init test-repos/tf1 --scope=code,bot`
`vc-init test-repos/tf1`

TODO: add actual output of ls or tree cmds (fill at close-out)

`vc-init ../test-repos/tf1 --scope=code,bot`
`vc-init ../test-repos/tf1`

TODO: add actual output of ls or tree cmds (fill at close-out)

### Preflight

- Verify `jj` is installed (`jj --version`); friendly error
  with install link if missing.
- For init URL / `owner/name` forms: probe with `git
  ls-remote <url>`. Exists → clone. Doesn't → create via
  `gh repo create` (errors if host isn't GitHub or `gh`
  missing).
- For clone URL / `owner/name` forms: probe with `git
  ls-remote <url>`. Exists → clone. Doesn't → error
  (clone is dumb, no auto-create).

### Edits

- `Cargo.toml`: bump to `0.41.1`.
- New `src/repo_url.rs` (or fold into `src/common.rs`): lift
  `derive_name` and `resolve_url` from `clone.rs`. Add
  `parse_target(s) -> Target` enum `{Url(String),
  OwnerName(String, String), Path(PathBuf)}`. Single source
  of truth for positional parsing across init and clone.
- `src/clone.rs`:
  - `CloneArgs`: `<TARGET>` + optional `[NAME]` positionals;
    add `--scope code,bot|por` (default `code,bot`); drop
    `--dir`. Old `--repo` positional rename: `repo` →
    `target`.
  - Refactor body into `clone_one(url, target_dir)` (the
    primitive, also used by `--scope=por`) and
    `clone_dual(code_url, target_dir)` (orchestrator —
    derives bot URL, calls `clone_one` twice, calls
    `symlink::create`).
  - Pre-clone check: error if `target_dir` exists.
- `src/init.rs`:
  - `InitArgs`: drop `owner`, `repo_local`, `repo_remote`,
    `dir`; replace bare-name `name` positional with
    `<TARGET>` + optional `[NAME]`.
  - Remove the 6 mutually-exclusive flag checks.
  - New `detect_state(path) -> InitTargetState`
    `{Empty, PorJjGit, PorGitOnly, PorWithPeerPor,
    SingleRepoWorkspace, DualRepoWorkspace, Other(reason)}`.
  - Refactor body into `init_one(target_dir, opts)` (the
    `--scope=por` primitive, also used as a step inside
    `--scope=code,bot`) and `init_dual(target_dir, opts)`
    (orchestrator — calls `init_one` twice, writes both
    `.vc-config.toml` files, ochid-links, `symlink::create`).
  - POR upgrade paths:
    - PorJjGit + `--scope=code,bot` → skip code-side init,
      run bot-side `init_one` into `.claude/`, write configs,
      ochid-link existing code commit + new bot commit,
      symlink.
    - PorGitOnly + `--scope=code,bot` → `jj git init` first,
      then PorJjGit path.
    - PorWithPeerPor + `--scope=code,bot` → write only the
      two `.vc-config.toml` files + symlink (both repos
      already exist).
- `src/test_helpers.rs`: `Fixture::new_opts` reshapes to use
  the new `target` positional shape (and `[NAME]` if needed).
- `notes/todo.md`: add this cycle to In Progress.
- `notes/chores-07.md`: per-step post-impl subsections.
- `notes/vc-x1-init.md`: close out cosmetic anomalies (fold
  into final close-out).

### Cycle structure — multi-step

- `-0` — plan + version bump + this notes section.
- `-1` — lift `derive_name` / `resolve_url` / `parse_target`
  to shared module; clone migrates internally (no behavior
  change yet).
- `-2` — clone reshape: `<TARGET>` + `[NAME]` positionals,
  add `--scope code,bot|por`, refactor into `clone_one` /
  `clone_dual`. Add target-exists pre-check.
- `-3` — init reshape: drop old flags, add `<TARGET>` +
  `[NAME]`, add `--scope=por`, refactor into `init_one` /
  `init_dual`. Existing create-from-empty operations work
  via the new shape.
- `-4` — init POR detection + upgrade paths
  (PorJjGit, PorGitOnly auto-bootstrap, PorWithPeerPor
  config-only).
- `-5` — `test_helpers::Fixture` migration; audit downstream
  callers across the test suite.
- final — cycle close-out: fill in Example Layout outputs,
  address `notes/vc-x1-init.md` cosmetic anomalies, drop
  the `-N` suffix.

### Decisions made during design

- **Version + cycle line.** This work + the sync `--check`
  fix land on the 0.41.x line. Init+clone = 0.41.1. Sync
  fix = a separate cycle (likely 0.41.2). Then rebase the
  in-flight 0.42.0 work on top of both.
- **Path-prefix vocabulary.** `./NAME` and the standard
  prefixes (`../`, `/`, `~/`, `~`) only. Bare `.` and bare
  `NAME` are errors — explicit prefix required.
- **`--private` on existing remote.** Warn and ignore;
  visibility was set at create time.
- **Cosmetic anomalies** from `notes/vc-x1-init.md` —
  addressed at close-out, not deferred.
- **`--scope=code` and `--scope=bot` for clone.** Dropped
  from the menu. Manual decomposition (two `--scope=por`
  clones + `vc-x1 symlink`) covers the use case.
- **Composition over duplication.** `--scope=code,bot` is
  implemented as in-process composition of the `--scope=por`
  primitive — single source of truth for the actual
  clone/init operation, thin wrapper for the dual case.

# References

[1]: /notes/chores-06.md#--scope-continuation-0410
[2]: /notes/chores-06.md#0410-4-capture---scope-enum-vocabulary
