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

## --scope sum-type refactor (0.42.0)

Picks up the work the 0.41.0 cycle redirected away from. The
foundation captured in [2] (chores-06's `### 0.41.0-4: capture
--scope sum-type vocabulary`) lands here as code: the `Scope`
type becomes a sum (`Roles(Vec<Side>) | Single(PathBuf)`), the
flag surface unifies under `--scope` / `-s` with both keyword and
path forms, and every dual-repo-aware command migrates to the
new shape in dependency order.

**Cycle steps (initial sketch — sub-step boundaries can shift
once each command's call sites are seen).**

- **0.42.0-0** — this plan + version bump + new
  `notes/chores-07.md`. Notes only.
- **0.42.0-1** — `scope.rs` sum type. Internal only — no
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
`### 0.41.0-4: capture --scope sum-type vocabulary` for
the full vocabulary, type-model, and per-command
applicability matrix. Read that subsection first before
diving into any of the `-N` steps below.

# References

[1]: /notes/chores-06.md#--scope-continuation-0410
[2]: /notes/chores-06.md#0410-4-capture---scope-sum-type-vocabulary
