# Done

As TODO.md `## Done` sections fills move them to here.

- Add --version and -V flags using std lib (clap not a dependency)
- Use git trailers for inter/intra repo info: ochid trailer, changeID path syntax, .vc-config.toml[[2]]
- Document git trailer convention (ochid:) and .vc-config.toml for workspace identity
- Document why jj log shows fewer commits than gitk (refs/jj/keep, obslog, ::@ revset)
- Create a binary that lists jj info[[1]]
- Convert CLI to subcommand structure with `list` command
- Add finalize subcommand arg parsing (0.6.0-dev1) [3]
- Add finalize daemonize with debug logging (0.6.0-dev2) [3]
- Implement finalize exec with squash/push logic (0.6.0-dev3) [3]
- Add --ignore-immutable and unique log paths (0.6.0-dev4) [3]
- Finalize subcommand complete (0.6.0) [3]
- Plan refactor and desc subcommand (0.7.0-dev0) [4],[5]
- Extract common.rs and refactor list (0.7.0-dev1) [4],[5]
- Refactor finalize into src/finalize.rs (0.7.0-dev2) [4],[5]
- Implement desc subcommand (0.7.0-dev3) [4]
- Refactor and desc subcommand complete (0.7.0) [4]
- Migrate CLI parsing to clap derive (0.8.0) [6]
- Move subcommand args into per-module structs (0.9.0) [7]
- Add --revision/-r, --repo/-R, --limit/-l to list (0.10.0-dev1) [8]
- Add --revision/-r, --repo/-R, --limit/-l to desc (0.10.0-dev2) [8]
- Revision and repo options complete (0.10.0) [8]
- Show changeID and commitID in desc output (0.11.0) [9]
- Add chid subcommand (0.12.0) [10]
- Add --limit to chid subcommand (0.13.0) [11]
- Add positional `..` revision notation (0.14.0) [12]
- Add required `--bookmark` to finalize (0.14.0) [13]
- Bold primary revision in chid, list, desc output (0.15.0) [14]
- Indent desc body lines with --indent/-i, default 3 spaces (0.16.0) [15]
- Finalize: replace --foreground with --detach, document manual recovery (0.17.0) [16]
- jj commit organization and traversal mechanisms (0.17.0) [17]
- Add show subcommand with header, bookmarks, and diff summary (0.18.0) [18]
- Flesh out show header to match gitk, add .. notation and file limiting (0.18.1) [18]
- Unify `..` notation and CLI across all subcommands (0.19.0) [18]
- Reorganize notes: move older done items to done.md (0.19.1)
- Multi-repo `-R` support with `-l`/`--label` and `-L`/`--no-label` for chid, desc, list, show (0.20.0) [18]
- Disperse CLI parsing tests from main.rs into per-subcommand files (0.20.1) [19]
- Show ochid in list output, clean up CLI help defaults (0.21.0) [19]
- Deduplicate common CLI flags with `#[command(flatten)]` (0.21.1) [19]
- Add fix-ochid subcommand with validation and --fallback (0.22.0) [19]
- Fix fix-ochid prefix bug: read workspace.path from .vc-config.toml (0.22.1) [19]
- Fix fix-ochid short ID extension, add notes to pre-commit checklist (0.22.2) [19]
- Add --add-missing to fix-ochid for inferring ochid from title+timestamp (0.23.0) [19]
- Add --max-fixes to fix-ochid to limit commits actually changed (0.24.0) [19]
- Add validate-desc subcommand, extract desc_helpers (0.25.0-dev1) [21]
- Add fix-desc subcommand using shared helpers (0.25.0-dev2) [22]
- Add lost/none special ochid status, improved error messages (0.25.0-dev2) [22],[23]
- Read other-repo from .vc-config.toml, make positional arg a --other-repo flag (0.25.0-dev3) [24]
- Run fix-desc on both repos to fix ochid trailers with --fallback for lost IDs (0.25.0) [20]
- Remove deprecated fix-ochid subcommand (0.25.0) [25]
- Add shell completion via clap_complete env (0.26.0) [26]
- Fix validate-desc/fix-desc other-repo resolution with -R flag (0.26.2) [30]
- Add `fn claude-symlink` and `symlink` subcommand (0.27.0) [31]
- Add `init` subcommand for dual-repo project creation (0.28.0) [32]
- Add `clone` command + fix init submodule/ochid bug (0.29.0) [33]
- Universal --verbose, common::run() refactor, chid bold removal (0.30.0) [34]
- Adopt `log` crate with per-module runtime filtering (0.31.0) [35]
- Remove submodule from init/clone (0.31.1) [38]
- Audit `unwrap`/`unwrap_or` usage, add `// OK: …` convention (0.32.0) [39]
- Make `finalize` failures visible — pre-flight, subprocess logging, tty reconnect, status marker (0.33.0) [40]
- Fix deprecated `jj bookmark track <bookmark>@<remote>` syntax for jj 0.40.0 (0.33.1) [41]
- Silence untracked-remote hint in `init` step 9 (0.33.2) [42]
- Compatible dep refresh via `cargo update` (0.33.3) [43]
- Add `--use-template` to `init` and `test-fixture` (0.34.0) [44]
- Bump `jj-lib` to 0.40 + tighten `clap` floor to 4.6 (0.34.1) [45]
- Add `sync` subcommand — fetch + classify + rebase both repos (0.35.0) [46]
- Show bookmarks in `list`, `show`, `desc` output (0.36.0) [47]
- CLAUDE.md refresh + memory migration (0.36.1) [49]
- Lift sync's inline test harness into shared `test_helpers` module (0.36.2) [51]
- Sync improvements: -R flag + quieter dry-run + sync-before-work discipline (0.36.3) [50]
- push subcommand scaffolding: flag surface, Stage enum, stub (0.37.0-0) [48]
- push state machine: state file, --status/--restart/--from, stage stubs (0.37.0-1) [48]
- push real stage bodies + jj-op snapshot rollback (0.37.0-2) [48]
- push integration tests + workspace-root refactor (0.37.0-3) [48]
- push interactivity: review prompt, $EDITOR, message persistence (0.37.0-4) [48]
- push polish: --dry-run, --step, non-tty detection, gitignore warning (0.37.0-5) [48]
- push docs + workflow migration — CLAUDE.md rewrite + README section (0.37.0) [48]
- First-dogfood polish for push: editor template, gitignore-fatal, sync --check, log prefix, quieter subprocess (0.37.1) [53]
- Temporary bookmark-tracking diagnostic probe on command entry/exit (0.37.2) [55]
- Fix bm-track bugs + rename + promote to permanent (0.37.3) [56]
- Capture squash-mode + scope design for push (0.37.4) [57]
- Capture --message-file design for push (0.37.5) [58]
- CLAUDE.md polish: markdown-anchor rule, shell-path brevity, state-file clearing, late-changes recipe trimmed (0.37.6) [59]
- Notes restructure: chores-06 + trim long todo entries (0.37.7) [64]
- Scope design refinements (0.37.8) [65]
- Bookmark tracking verification: shared helper + tests (0.38.0-0) [66]
- Bookmark tracking verification: wire into setup commands (0.38.0-1) [66]
- Bookmark tracking verification: wire into preflight commands (0.38.0-2) [66]
- Bookmark tracking verification: cycle close-out + dogfood validation (0.38.0) [66]
- Push hardening: state-sanity preflight on resume (0.39.0-0) [67]
- Push hardening: honest completion via post-completion verification (0.39.0-1) [67]
- Push hardening: cycle close-out, 0.39.0-2 skipped (0.39.0) [67]
- Scope generalization: init --repo-local + --repo-remote (0.40.0-1) [70]
- Scope generalization: init --scope=code|bot|code,bot (0.40.0-2) [70]
- Scope generalization: integration tests migrate onto init --repo-local (0.40.0-3) [70]
- Scope generalization: cycle close-out, init --scope foundation shipped (0.40.0) [70]
- Pre-commit checklist requires `--locked` for `cargo install` (0.41.0-1) [71]
- Scope continuation: sync --scope (0.41.0-2) [71]
- Scope continuation: capture --scope-everywhere direction (0.41.0-3) [71]
- Scope continuation: capture --scope enum vocabulary (0.41.0-4) [71]
- Scope continuation: cycle close-out — push/finalize work deferred to 0.42.0 (0.41.0) [71]
- Init+clone redesign: chores-08 + forks-multi-user + draft-reviews + vc-x1-init forward (0.41.1-0) [72]
- Init+clone redesign: shared repo_url module + clone/init migrate (0.41.1-1) [73]
- Init+clone redesign: clone reshape — TARGET + NAME + --scope=code,bot|por (0.41.1-2) [73]
- Init+clone redesign: user config — ~/.config/vc-x1/config.toml (0.41.1-3) [74]
- Init+clone redesign: user config rewrite — account/category schema (0.41.1-4) [74]
- Init+clone redesign: init reshape — TARGET + NAME + --account + --repo (0.41.1-5) [73]
- Init+clone redesign: init refactor — substep ladder -6.0 through -6.8 (0.41.1-6) [75]
- Init+clone redesign: cycle close-out — symmetric schema + Fixture migration + dual|por split deferred (0.41.1) [72]
- Substep protocol formalization + jj revsets cheatsheet (0.42.0-4.5) [77]
- init-clone-refactor recovery + post-mortem playbook (0.42.0-4.6) [78]
- Init-clone-refactor rebase landing — main rebased + .claude re-paired (0.42.0-4.7) [79]
- 0.42.0 cycle close-out at -4.7 — partial --scope sweep, continuation deferred [81]
- Test-module extraction across init/push/sync/common (0.43.0) [84]
- InitParams + Context — init subcommand-layer decoupling worked example (0.44.0) [85]
- ARCHITECTURE.md + subcommand-layer terminology reconciliation (0.45.0) [86]
- finalize subcommand-layer migration — Context.log + TryFrom (0.46.0) [87]
- finalize Migration B — squash options_flags leaf (0.47.0) [88]
- Migration A sweep — subcommand-layer ports: symlink/clone/sync/validate-desc/fix-desc/push (0.48.0) [89]
- docs: por-dual capture + icr cleanup (0.48.1) [90]
- docs: chores edit list → commit message (0.48.2) [91]
- docs: chores-09 → new shape (0.48.3) [92]
- docs: renumber todo.md + chores-09 refs (0.48.4) [93]
- chid/desc/list/show CommonArgs sweep — options_flags + `-s`/`--scope` + Context+Params ports 12/12 (0.49.0) [94]
- Subcommand trait sweep — 12 subcommands ported via `SubcommandRunner` trait, `main.rs` collapsed to `Context::load` + thin dispatch (0.50.0) [95]
- chores subdir reshape — `notes/chores-*.md` → `notes/chores/`; 0.44.0–0.50.0 Done batch migrated to done.md (0.51.0) [[96]]
- `sb_ide` elimination — banner off by default (`-V` toggles), `bm_track` → `debug!`, `sb_ide` + `SubcommandRunner::{is_detached_exec, suppress_banner}` removed (0.52.0) [[97]]
- todo renumber + `notes/fix-todo.py` interim script; cycle re-scoped at close-out, scope CLI cleanup deferred to 0.54.0 (0.53.0) [[98]]
- scope CLI cleanup — `--scope` roles-only, `--por` boolean replaces `ScopeKind`, `Scope` relocated to `options_flags/`, sync gains `-R` (0.54.0) [[99]]
- validate-todo / fix-todo subcommands — check + renumber `## Todo` / `## Bugs` entry numbering, replacing `notes/fix-todo.py` (0.55.0) [[100]]
- refine cycle protocol — one protocol (Preparation/Work-N/Close-out), `.`-separator nested numbering with trailing-`0`=Preparation, push & squash discretionary, `.claude` once per push, two-gate review (work then message, both before commit), CLAUDE.md cycle/commit/push docs consolidated into one linear `## Cycle Protocol` (~39% smaller) (0.56.0) [[101]]
- add `--merge` todo entry — Todo #1 records future `vc-x1 push --merge` flag (close-out shape, sibling to planned `--squash`); dogfoods the Preparation/Work-N/Close-out protocol on a deliberately small docs cycle (0.57.0) [[102]]
- notes/todo restructure — split `## Bugs` → `bugs.md` and the long-tail `## Todo` → `todo-backlog.md`; `## Priorities` with tier sub-headings (`### P1`/`### P2`/`### P3`); CLAUDE.md `## File reads` rule + protocol codification (chores title-only during cycle, In Progress moves into chores at close-out, problem+plan shape) (0.58.0) [[103]]
- extract cycle protocol — `notes/cycle-protocol.md` becomes the canonical self-contained home for the cycle workflow (504 lines, extensively tightened from the CLAUDE.md extract); CLAUDE.md keeps a 10-line pointer; `notes/substep-protocol.md` folded in as `## Sub-cycle ladders`; `## Ideas` section added to `notes/todo.md`; first squash close-out via manual Option F (app squash + bot-side `af60f979` trailer rewrite + force-push) (0.59.0) [[104]]
- consolidate notes conventions — three notes-file sections (`Todo format`, `Reference numbering`, `Retiring Done entries`) move from notes/README.md into new CLAUDE.md `## Notes file conventions` umbrella alongside existing `## Chores conventions`; `[[N]]` citation duplicate dropped; cargo cycle (`fmt` / `clippy` / `test` / `install`) surfaced at CLAUDE.md `## Cycle Protocol` and notes/README.md (had been buried in cycle-protocol.md since 0.59.0); README.md `## Contributing` rewritten against current anchor homes (0.60.0) [[105]]
- por/dual parity design — eight-commit audit + design cycle producing `notes/design-cli/por-dual-parity-audit.md` as the canonical CLI-design doc (audit + commonality + feature axes + 5-layer resolution chain + subcommand × parameter matrix + per-axis Decisions blocks); new sibling `notes/design-cli/copying.md` stub for the broader file-copy mechanism that subsumes `--config` / `--gitignore` / `--use-template`; `notes/design-cli/` subdir created and three design notes regrouped under it; 14 implementation gaps seeded for 0.62.0+ cycles; one Todo promoted (`validate-desc` / `fix-desc` equalization, cheapest prototype for the topology-from-config rule) (0.61.0) [[106]]
- apply max review #1 — applied six concerns, four nits, and the process observation from the `max-review-1` working list to the por/dual parity design + copying stub; reframed Todo #1 (push validate body intro), seeded pre-commit-single-rule + `validate-numbering` Todos; working list fully drained, then retired (deleted — git history holds it) (0.62.0) [[107]]
- docs: adopt AGENTS.md — rename `CLAUDE.md` → `AGENTS.md` (Zed and the agent-tooling ecosystem default to it); one-line `@AGENTS.md` import shim at `CLAUDE.md` keeps Claude Code auto-loading; live `CLAUDE.md` references repointed to `AGENTS.md` so links resolve in editors and on GitHub; history prose (`chores-01..12` / `done.md`) left as written, with only the 3 navigational anchor links in the `chores-10/11/12` headers repointed (0.63.0) [[108]]
- docs: tighten after-finalize rule — rename to "After push or finalize: stop and wait" (both triggers named) and spell out that `vc-x1 push` bundles the push + `vc-x1 finalize` on `.claude` as tail stages, so all closing words go *before* invoking the wrapper and nothing is emitted after it returns (0.63.1) [[109]]
- docs: codify merge-non-ff recipe — promote the merge-non-ff close-out recipe to a `### Merge non-ff recipe` subsection in cycle-protocol.md (rebase → `jj new` lift → push + post-hoc caveat); reword `### Shape at close-out push` (work-done framing, Merge non-ff tagged default); standardize jj rebase `-d` → `--onto`/`-o` in AGENTS.md and drop the post-amend `jj new` note (the recipe now owns the empty-`@` why); also clarified the Preparation step (Cargo.lock, In-Progress move wording) (0.64.0) [[110]]
- docs: record finalize ochid-loss bug (0.65.1) — bugs.md gains the fc finalize ochid-drop incident as Bugs #1 with the fix queued as Todo #1; fc AGENTS.md additions ported (jj-not-git, one-command-per-invocation, push-injects-trailers, ochid resolvability + `.vc-config.toml`); stale chores-10 "active file" prose genericized in notes/README.md + ARCHITECTURE.md [[111]]
- fix: refuse ochid-dropping squash (0.65.2) — `finalize` refuses a squash that would drop source-only `ochid:` trailers (`extract_ochids` / `ochids_at_risk` / `check_squash_keeps_ochids` + tests), guarding in preflight and again in `finalize_exec` after `--delay`; failure-marker surfacing moved after the command's output with a historical banner and the `error=` value flattened; README manual-test section + `support/gen-exmpl-1-3.sh` regenerator [[112]]
- feat: reposition @ onto synced bookmark (0.66.0) — after a successful `--no-check` sync, `@` is repositioned onto the just-synced bookmark: code repo `jj new <b>` when clean (or `--rebase`/prompt-gated rebase when dirty; left in place when diverged/ahead), `.claude` always `jj new main` (or errors when `@-` is off main), all as a final pass *outside* the `op_restore` revert region; replaces `ensure_at_on_main`; new `--rebase` flag; README `### sync` docs + examples [[113]]
- feat: single-mode sync + revert command (0.67.0) — plain `vc-x1 sync` is one atomic operation (fetch, converge bookmark, reposition `@`; `--no-check` gone, `--check` a hidden deprecated alias for push preflight); failures stop for inspection with each repo's pre-sync op id persisted to `.vc-x1/sync-state.toml`; new `vc-x1 revert` restores from the snapshots; TDD via the two-clone `tests/cli_sync.rs` regression test of the t1A/t1B scenario [[114]]
- docs: todo cleanup + trapezoid entries (0.67.1) — push-related todos reshaped around the trapezoidal (merge non-ff) workflow: new #1 bookmark-invariant fix and #2 push pause point; "record uncovered code commits (N:1)" re-scoped to code worked outside vc-x1; `push --squash` demoted to todo-backlog.md; cycle-protocol.md push-wrapper list synced [[115]]

- feat: bot-session transcript viewer — display a session transcript as a conversation: two-layer tolerant parse (serde_json text → Value; hand extraction into our structs, raw retained), eight-item composable output (--<item> / --no-<item> / --all / --none) with git-style config defaults (CLI > .vc-config.toml > user config > built-in), --lines slicing, UTC headers; --raw and index view deferred (Todo #12), EPIPE logger panic recorded (Bugs #4) [[119]]
- feat: bot-session --result-lines knob — the [result]-body
  cap becomes a flag: `--result-lines N` (default 10, 0 =
  unlimited), Output-range help group; was hardwired to 10
  even under --all [[120]]
- feat: bot-session --fields + --raw explorer — bot-session
  doubles as a schema explorer: --fields (dotted-path
  inventory per entry type: count, kinds, samples),
  --unknown (inventory minus the extractor's KNOWN_PATHS —
  the unmodeled surface; 132 paths on first real run),
  --raw (pretty-printed source lines); --per-line (a fields
  section per source line, composes with --unknown); --lines
  unified to source-JSONL-line units in every view,
  conversation included; drift-over-time baseline deferred to the
  discovery/index cycle [[121]]
- feat: bot-session --col-width knob — the field views'
  (`--fields`/`--unknown`/`--per-line`) first-column pad
  becomes `--col-width N`, default widened 44 → 68 (aligns
  the type column for ~99% of observed key paths; only the
  long-tail `snapshot.trackedFileBackups.<abs path>.*` keys
  overflow); config-hierarchy resolution deferred to Todo #12 [[122]]
- docs: move todo.md to root TODO.md — todo list moved from notes/ to the conventional root-file family; live references swept (AGENTS.md, cycle-protocol.md, README, ARCHITECTURE, notes/*); no-arg validate-todo / fix-todo default follows the move; historical files keep `notes/todo.md`; the shared doc set diverges until vc-template-x1 and iiac-perf apply the same change [[123]]
- docs: shared protocol sync + jj refactor plan — adopted the vc-template-x1 shared notes set (AGENTS.md, cycle-protocol.md, versioning.md, jj-tips.md) with vc-x1's 0.69.0 corrections ratified template-side (manifest: [notes-sync-20260716.md](notes/notes-sync-20260716.md)); jj facade → jj-lib refactor program planned in [refactor-20260716.md](notes/refactor-20260716.md), absorbing eight Todos [[124]]
- bot-session: --fields / --unknown output clarification — the
  inventory views are now documented rather than opaque:
  [transcript-format.md](notes/transcript-format.md) defines
  entry / entry type and the `.`/`[]` field notation with a
  bot-session example (0.71.0-8), and the ambiguous "dotted"
  wording was retired (0.71.0-7). In-view column labeling
  (headers / a legend) left as an optional nicety
- feat: config discoverability + scalar hierarchy — a
  code-declared config schema registry (`config_schema.rs`) as
  the single source of truth for every settable config key; the
  new `config` command (print, `--home`, `--validate`), init's
  commented `.vc-config.toml` defaults, and bot-session's
  `--result-lines`/`--col-width` config layer all derive from it,
  so they can't drift. Also a `notes/transcript-format.md` SSOT +
  sample for the bot-session format, and a sweep retiring the
  ambiguous "dotted" wording [[125]]

- refactor: DRY jj facade — one typed facade (`src/jj.rs`)
  for every read-only jj query spawn (log templates +
  bookmark listings), the tracking and ochid trailer parsers
  unified beside it, and the test fixture helpers deduped to
  one copy per crate; first stage of the jj refactor
  program, worked on `refactor-vc-x1` with main parked at
  the 0.71.0 tip [[126]]

- docs: adopt new template repo names — live mentions of
  `vc-template-x1`(.claude) swept to `vc-x1-work-repo-template`
  / `vc-x1-bot-repo-template` (AGENTS.md byte-identical with
  the template again; README init examples now pass an
  explicit `CODE,BOT` pair); historical records keep the old
  name [[127]]

- docs: notes rework + config refresh — `.vc-config.toml`
  (both sides) adopts init's generated optional-keys block;
  jj-tips.md re-syncs with the template (reclassified as
  pedagogy, not history); template-restructure design
  promoted to a Todo with `.bot` / symmetric-schema
  decisions folded into the refactor program; new Idea:
  chores retire into a session index; bot repo seeded with
  LICENSE-* / README.md from vc-x1-bot-repo-template [[128]]

- refactor: hygiene riders — the work/bot terminology
  stragglers swept (~380 identifier sites, `Side::Work`,
  `ConfigRole`, narration labels, test remotes); the `-s`
  scope keyword renamed `code` → `work` outright (no alias —
  unreleased); the six single-field `options_flags` leaves
  adopt the `value` field shape with clap ids pinned, and
  clone.rs's drifted inline `dry_run` folds onto the leaf;
  second stage of the jj refactor program [[129]]

- refactor: facade owns topology — repo resolution is facade
  state: the symmetric `work`/`bot` `[workspace]` schema
  (identical block on both sides, side detection by location,
  `.bot` as new-init default only), every `.claude` literal
  resolved from config, dual-entry coherence preflight, the
  validate-desc/fix-desc por equalization, and the `config`
  command's positional target; third stage of the jj refactor
  program [[130]]

- docs: refactor program ladder + conventions: the refactor
  program's remaining stages consolidated into four cycles
  and laid out as a program ladder under a new heading-based
  `## In Progress` shape; the parked 0.72.0 branch declared
  quarry (version gap accepted); the version-first-bullet
  body convention added to cycle-protocol.md; chores-14
  0.75.0 rung refs backfilled [[131]]

- refactor: repo registry: `.vc-config.toml`'s `[workspace]`
  block becomes a `[repos]` registry: ordinary file-relative
  (or absolute) paths, side detection by self-resolution,
  resolved agreement + self-identification replacing the
  identical-block invariant, and ochid prefixes as canonical
  side labels decoupled from the bot dir's spelling; legacy
  reads consolidated in `src/legacy_vc_config.rs` for later
  retirement. De-gitify init rode as the last rung, so init's
  publish path is jj-only and bugs.md #1/#2 are fixed; fourth
  stage of the jj refactor program [[132]]

- docs: trapezoid close-out recipe: the four steps that
  publish a trapezoid close-out consolidated into one
  definitive procedure in cycle-protocol.md (base rule,
  the two-parent verification, the sideways-move backfill
  embargo, recovery), with the refactor stage keeping only
  implementation deltas and README waiting for the flag;
  vocabulary collapsed to "trapezoid"; chores-15 opened
  [[133]]

- refactor: stateless push: push keeps no state and cannot
  resume: the state file, `--restart` / `--from` / `--status`,
  the stale-state verifier, the `[push]` config keys and the
  `.gitignore` coherence check are gone, and so is preflight,
  whose `sync --check` self-spawn forced the tests'
  stage-skipping flag. What replaces them is a property:
  every stage no-ops when its work is already done, so a
  failed run is re-run rather than resumed. bugs.md #3 is
  unrepresentable and #4 is fixed; `push.rs` 1480 -> 816
  lines, ~940 net removed; fifth stage of the jj refactor
  program [[134]]

- docs: jj-lib design notes + trapezoid recipe: the op-store
  coexistence risk answered from jj-lib 0.41 source against an
  installed jj 0.40.0 (unenforceable, low blast radius, so a
  decision rather than a step 0.78.0 assumes), and the
  trapezoid recipe corrected where the 0.77.0 close-out found
  it wrong: step 4 is `jj git push`, not `vc-x1 push` [[135]]

- docs: typeable punctuation: `—`, `–`, `…` and `→` no longer
  authored in durable text, and the 553 sites in the five prose
  files converted. None can be typed at a terminal, so none can
  be grepped for, and an em dash next to option syntax reads as
  another flag. The rule sorts a character by the role it plays
  (naming it, doing a job, transcribed from outside) and warns
  that converting a heading moves its anchor. Scope covers
  commit titles and `src/`; that sweep is deferred to `## Todo`
  [[136]]

- docs: re-describe rule + defer punctuation sweep: `jj
  describe` on a published or already-stamped commit is a
  history rewrite that silently drops the `ochid:` trailer, so
  it is coordinate-first; the sub-cycle ladder is the named
  exception, being local until its single Close-out push. Two
  planned `0.77.x` rungs retired, the source sweep to `## Todo`
  and interlude shape to the backlog [[137]]

- build: bump jj-lib to 0.43: the local `jj` moved to 0.43.0,
  leaving the pin two releases behind. `use_glob_by_default`
  is gone from `RevsetParseContext` and `commit_change_ids()`
  returns a stream rather than an iterator, so `futures` joins
  the direct dependencies. The bump also moved the default
  revset string-pattern kind from substring to glob, which the
  compiler could not report and which no revset of ours uses
  [[138]]

- refactor: jj-lib migration [[139]]: facade internals and every
  mutation move in-process on jj-lib, ending jj and git spawning;
  the version gate makes the op-store coupling enforceable and the
  index-lock retry (bugs.md #1) is the headline prize.

- style: typeable punctuation + line-width source sweep [[140]]: src/ +
  tests/ are ASCII-clean and <=100 cols (JSONL fixture literals and
  comment URLs exempt as literal rows); 863 counted sites plus four
  uncounted species the enumeration missed, the load-bearing ellipsis
  in truncate_chars, and the config/show output separators; README
  config samples regenerated from the installed binary.

- refactor: drop sync state and remove revert [[141]]: sync keeps its
  pre-sync op snapshots in memory only, retiring `sync-state.toml`,
  vc-x1's last cross-invocation state file; `revert` is removed, its
  role taken by sync's failure report printing the manual
  `jj op log` / `jj op restore` recovery, until an op-log-derived
  design earns a reintroduction; init stops writing `/.vc-x1` to new
  workspaces' `.gitignore`. Triggered by bugs.md #8, the push
  stale-state incident at iiac-perf. Riders: the single-name
  convention (the package name is the binary's name, `vc-x1` on main
  and per-line dev names on branches, guarded by build.rs on every
  cargo verb), the argv0 runtime banner, and the first `vc-x1`
  promotion under it.

- test: Claude Code can complete a cycle [[142]]: a controlled
  experiment settling why `vc-x1 push` failed from sandboxed
  sessions, the cycle itself being the demonstration. Both repos were
  cloned over ssh, and the sandbox denies both the key material and
  a port-22 route, so the `git` child that jj-lib spawns had
  neither; wink repointed both remotes at https and the push went
  through. The competing hypotheses (bot-repo writability, the
  sandbox-masked config paths inside `.claude`, the interactive
  editor) were each killed by test rather than by argument.

- docs: adopt the merged agent-file set [[143]]: iiac-perf's
  `agent-files-model` proposal merged onto this repo's file layout
  with the review's corrections (cycles on their own bookmark, the
  six-item cycle record with one home, problem-then-solution bodies
  at <=50-col titles, steps named not numbered, versions living only
  in the version-of-record); two rules written during the review and
  applied set-wide, a semicolon joins equals and a pinned file names
  no project; `custom.md` shrinks to a stub reaching the new
  `custom-family.md` and `CLAUDE.md` collapses to `@AGENTS.md`.

- docs: fix three semicolons [[144]]: the three prose semicolons in
  AGENTS.md reworded to comma and period joins; the proposed
  prose-wide ban examined and dropped, argument in the chores
  section.

- docs: consolidate line widths [[145]]: the width numbers live only
  in prose.md's Line widths subsection, every former restatement a
  pointer; commit bodies wrap at <=75.

- docs: freshen vc-config and config subcmd [[146]]: the markdown
  config carrier landed (`toml` fences are the config, prose the doc,
  `.vc-config.md` on both sides); the iiac-perf convergence trio
  trialed, accepted, and landed with the template baseline and the
  jj-lib 0.44 bump; closed early (wink, 2026-08-17), the five config
  rungs deferred to the "Finish the vc-config surface" Todo.

- docs: drop the orphaned depth-note paragraph [[147]]: iiac-perf's
  2026-08-18 proposal accepted, the paragraph deleted family-wide,
  the pinned set diffing empty across the three repos modulo
  custom*.

- docs: retire the refactor program block [[148]]: the jj facade
  refactor program's block moved from `## In Progress` to an as-built
  ladder in refactor-20260716.md, bounded at 0.78.4; the freshen and
  depth-note cycles' seventeen rungs backfilled with versions and
  SHAs.

- 0.79.0 **refactor: retire the remaining jj spawns** [[152]]
  - the jj-CLI spawn path is gone: push and facade reads, sync repositioning, op
    recovery, squash, and init/clone provisioning all run in-process through jj-lib,
    the network legs staying jj-lib's own git children
  - clippy.toml bans `Command::new` outside a documented, closed allowlist (the version
    gate's `jj -V`, push's `$EDITOR`, init's gh provisioning, test helpers), deny-level,
    demonstrated once failing a scratch site, the enforced sites enumerable by a
    recorded rg one-liner
  - the acceptance check ran at close, all four items pass, the test-spawn audit finding
    every spawn integration-type

- 0.79.1 **docs: pin two rules and close the convergence record** [[149]]
  - iiac-perf's 2026-08-15 convergence record closed: all three proposals accepted without
    modification, `outcome-*` pointing at the [trial rung][150] that landed them, and a reply
    record sent
  - the `(done)` flip moved from before validation to work-review acceptance in
    cycle-checklists.md and cycle-protocol.md, and the semicolon rule's history clause moved
    from ask-on-alter to convert-on-touch in prose.md with code.md stating it for source-file
    comments
  - both entries retired from `custom-family.md`'s experimental section and the dogfood log,
    the diff against the payload now carrying them to the family

# References

[1]: /notes/chores/chores-01.md#create-a-binary-that-lists-jj-info
[2]: /notes/chores/chores-01.md#git-trailer-convention
[3]: /notes/chores/chores-01.md#finalize-subcommand-for-session-repo-coherence
[4]: /notes/chores/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[6]: /notes/chores/chores-01.md#migrate-cli-parsing-to-clap-080
[7]: /notes/chores/chores-01.md#move-subcommand-args-into-modules-090
[8]: /notes/chores/chores-01.md#add-revision-and-repo-options-to-list-and-desc-0100
[9]: /notes/chores/chores-01.md#show-changeid-and-commitid-in-desc-output-0110
[10]: /notes/chores/chores-01.md#add-chid-subcommand-0120
[11]: /notes/chores/chores-01.md#add---limit-to-chid-subcommand-0130
[12]: /notes/chores/chores-01.md#add-positional--revision-notation-0140
[13]: /notes/chores/chores-01.md#add-required---bookmark-to-finalize-0140
[14]: /notes/chores/chores-01.md#bold-primary-revision-in-output-0150
[15]: /notes/chores/chores-01.md#indent-desc-body-lines-0160
[16]: /notes/chores/chores-01.md#finalize-detach-and-manual-recovery-0170
[17]: /notes/chores/chores-02.md#jj-commit-organization-and-traversal-mechanisms-0170
[18]: /notes/chores/chores-02.md#0180--initial-show-subcommand
[19]: /notes/chores/chores-02.md#0200--multi-repo-support
[20]: /notes/chores/chores-02.md#0250--refactor-into-validate-desc--fix-desc
[21]: /notes/chores/chores-02.md#0250-dev1--add-validate-desc-extract-desc_helpers
[22]: /notes/chores/chores-02.md#0250-dev2--add-fix-desc-subcommand
[23]: /notes/chores/chores-02.md#special-ochid-values-lost-and-none
[24]: /notes/chores/chores-02.md#0250-dev3--read-other-repo-from-config
[25]: /notes/chores/chores-02.md#0250--remove-deprecated-fix-ochid
[26]: /notes/chores/chores-02.md#0260--shell-completion-via-clap_complete-env
[30]: /notes/chores/chores-02.md#0262--fix-validate-descfix-desc-other-repo-resolution-with--r
[31]: /notes/chores/chores-03.md#add-fn-claude-symlink-0270
[32]: /notes/chores/chores-03.md#add-init-command-0280
[33]: /notes/chores/chores-03.md#add-clone-command-0290
[34]: /notes/chores/chores-03.md#universal---verbose-and-commonrun-refactor-0300
[35]: /notes/chores/chores-03.md#adopt-log-crate-with-per-module-filtering-0310
[38]: /notes/chores/chores-03.md#remove-submodule-from-initclone-0311
[39]: /notes/chores/chores-04.md#audit-unwrapunwrap_or-usage-0320
[40]: /notes/chores/chores-04.md#make-finalize-failures-visible-0330
[41]: /notes/chores/chores-04.md#fix-deprecated-jj-bookmark-track-syntax-0331
[42]: /notes/chores/chores-04.md#silence-untracked-remote-hint-in-init-step-9-0332
[43]: /notes/chores/chores-04.md#compatible-dep-refresh-0333
[44]: /notes/chores/chores-04.md#add---use-template-to-init--test-fixture-0340
[45]: /notes/chores/chores-04.md#bump-jj-lib-to-040--tighten-clap-floor-0341
[46]: /notes/chores/chores-04.md#add-sync-subcommand-0350
[47]: /notes/chores/chores-04.md#show-bookmarks-in-list-show-desc-output-0360
[48]: /notes/chores/chores-05.md#add-push-subcommand-0370
[49]: /notes/chores/chores-05.md#claudemd-refresh--memory-migration-0361
[50]: /notes/chores/chores-05.md#sync-improvements--single-repo-support--quieter-dry-run-0363
[51]: /notes/chores/chores-05.md#test-harness-refactor-0362
[53]: /notes/chores/chores-05.md#first-dogfood-polish-for-push-0371
[55]: /notes/chores/chores-05.md#temporary-bookmark-tracking-diagnostic-probe-0372
[56]: /notes/chores/chores-05.md#fix-bm-track-bugs--rename--promote-to-permanent-0373
[57]: /notes/chores/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[58]: /notes/chores/chores-05.md#capture---message-file-design-for-push-0375
[59]: /notes/chores/chores-05.md#claudemd-polish-0376
[64]: /notes/chores/chores-06.md#notes-restructure-chores-06--trim-long-todo-entries-0377
[65]: /notes/chores/chores-06.md#scope-design-refinements-0378
[66]: /notes/chores/chores-06.md#bookmark-tracking-verification-0380
[67]: /notes/chores/chores-06.md#push-hardening-state--stage-sanity-0390
[70]: /notes/chores/chores-06.md#generalize---scope-across-commands-0400
[71]: /notes/chores/chores-06.md#--scope-continuation-0410
[72]: /notes/chores/chores-08.md#init--clone-redesign-0411
[73]: /notes/chores/chores-08.md#cycle-structure--multi-step
[74]: /notes/chores/chores-08.md#user-config-0411-3
[75]: /notes/chores/chores-08.md#operations
[77]: /notes/chores/chores-07.md#substep-protocol-formalization-0420-45
[78]: /notes/chores/chores-07.md#init-clone-refactor-recovery-0420-46
[79]: /notes/chores/chores-09.md#chore-init-clone-refactor-rebase-landing-0420-47
[81]: /notes/chores/chores-09.md#chore-close-0420-cycle-at--47-0420
[84]: /notes/chores/chores-09.md#chore-open-0430-cycle-0430-0
[85]: /ARCHITECTURE.md
[86]: /notes/chores/chores-09.md#docs-architecture-md--subcommand-layer-naming-0450
[87]: /notes/chores/chores-09.md#refactor-finalize--context--finalizeparams-0460
[88]: /notes/chores/chores-09.md#refactor-finalize---squash--options_flags-leaf-0470
[89]: /notes/chores/chores-09.md#chore-open-0480-cycle--migration-a-sweep-0480-0
[90]: /notes/chores/chores-09.md#docs-por-dual-capture--icr-cleanup-0481
[91]: /notes/chores/chores-09.md#docs-chores-edit-list--commit-message-0482
[92]: /notes/chores/chores-09.md#docs-chores-09--new-shape-0483
[93]: /notes/chores/chores-09.md#docs-renumber-todomd--chores-09-refs-0484
[94]: /notes/chores/chores-10.md#chore-open-0490--finish-migration-a-0490-0
[95]: /notes/chores/chores-10.md#chore-close-subcommand-trait-sweep-0500
[96]: /notes/chores/chores-11.md#chore-move-chores-under-noteschores-0510
[97]: /notes/chores/chores-11.md#chore-close-sb_ide-elimination-0520
[98]: /notes/chores/chores-11.md#chore-todo-renumber--fix-todopy-0530
[99]: /notes/chores/chores-11.md#refactor-scope-cli-cleanup-0540
[100]: /notes/chores/chores-11.md#feat-validate-todo--fix-todo-0550
[101]: /notes/chores/chores-11.md#docs-refine-cycle-protocol-0560
[102]: /notes/chores/chores-11.md#docs-add---merge-todo-entry-0570
[103]: /notes/chores/chores-12.md#refactor-notestodo-restructure-0580
[104]: /notes/chores/chores-12.md#docs-extract-cycle-protocol-0590
[105]: /notes/chores/chores-12.md#docs-consolidate-notes-conventions-0600
[106]: /notes/chores/chores-12.md#docs-pordual-parity-design-0610
[107]: /notes/chores/chores-12.md#docs-apply-max-review-1-0620
[108]: /notes/chores/chores-13.md#docs-adopt-agentsmd-0630
[109]: /notes/chores/chores-13.md#docs-tighten-after-finalize-rule-0631
[110]: /notes/chores/chores-13.md#docs-codify-merge-non-ff-recipe-0640
[111]: /notes/chores/chores-13.md#docs-record-finalize-ochid-loss-bug-0651
[112]: /notes/chores/chores-13.md#fix-refuse-ochid-dropping-squash-0652
[113]: /notes/chores/chores-13.md#feat-reposition--onto-synced-bookmark-0660
[114]: /notes/chores/chores-13.md#feat-single-mode-sync--revert-command-0670
[115]: /notes/chores/chores-13.md#docs-todo-cleanup--trapezoid-entries-0671
[116]: /notes/chores/chores-13.md#feat-pin-bot-repo-to-main-0680
[117]: /notes/chores/chores-13.md#docs-diagnose-silent-session-push-loss-0681
[118]: /notes/chores/chores-13.md#feat-inline-session-push--squash-push-0690
[119]: /notes/chores/chores-13.md#feat-bot-session-transcript-viewer
[120]: /notes/chores/chores-13.md#feat-bot-session---result-lines-knob
[121]: /notes/chores/chores-13.md#feat-bot-session---fields----raw-explorer
[122]: /notes/chores/chores-13.md#feat-bot-session---col-width-knob
[123]: /notes/chores/chores-13.md#docs-move-todomd-to-root-todomd
[124]: /notes/chores/chores-13.md#docs-shared-protocol-sync--jj-refactor-plan
[125]: /notes/chores/chores-13.md#feat-config-discoverability--scalar-hierarchy
[126]: /notes/chores/chores-14.md#refactor-dry-jj-facade
[127]: /notes/chores/chores-14.md#docs-adopt-new-template-repo-names
[128]: /notes/chores/chores-14.md#docs-notes-rework--config-refresh
[129]: /notes/chores/chores-14.md#refactor-hygiene-riders
[130]: /notes/chores/chores-14.md#refactor-facade-owns-topology
[131]: /notes/chores/chores-14.md#docs-refactor-program-ladder--conventions
[132]: /notes/chores/chores-14.md#refactor-repo-registry
[133]: /notes/chores/chores-15.md#docs-trapezoid-close-out-recipe
[134]: /notes/chores/chores-15.md#refactor-stateless-push
[135]: /notes/chores/chores-15.md#docs-jj-lib-design-notes--trapezoid-recipe
[136]: /notes/chores/chores-15.md#docs-typeable-punctuation
[137]: /notes/chores/chores-15.md#docs-re-describe-rule--defer-punctuation-sweep
[138]: /notes/chores/chores-15.md#build-bump-jj-lib-to-043
[139]: /notes/chores/chores-15.md#refactor-jj-lib-migration
[140]: /notes/chores/chores-16.md#style-typeable-punctuation--line-width-source-sweep
[141]: /notes/chores/chores-16.md#refactor-drop-sync-state-and-remove-revert
[142]: /notes/chores/chores-16.md#test-claude-code-can-complete-a-cycle
[143]: /notes/chores/chores-16.md#docs-adopt-the-merged-agent-file-set
[144]: /notes/chores/chores-16.md#docs-fix-three-semicolons
[145]: /notes/chores/chores-16.md#docs-consolidate-line-widths
[146]: /notes/chores/chores-16.md#docs-freshen-vc-config-and-config-subcmd
[147]: /notes/chores/chores-16.md#docs-drop-the-orphaned-depth-note-paragraph
[148]: /notes/chores/chores-16.md#docs-retire-the-refactor-program-block
[149]: /notes/chores/chores-17.md#docs-pin-two-rules-and-close-the-convergence-record
[150]: /notes/chores/chores-16.md#docs-trial-the-iiac-perf-convergence-proposals
[152]: /notes/chores/chores-17.md#refactor-retire-the-remaining-jj-spawns
