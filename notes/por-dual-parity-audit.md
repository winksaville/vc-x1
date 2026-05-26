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

# References

[1]: /notes/todo.md
[2]: /notes/por-dual-parity.md
