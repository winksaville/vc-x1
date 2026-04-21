# Chores-05

## CLAUDE.md refresh + memory migration (0.36.1)

Adopt `iiac-perf/CLAUDE.md` as the canonical baseline and apply it
identically to `/CLAUDE.md` (this repo) and `vc-template-x1/CLAUDE.md`
(template for new projects). CLAUDE.md is now explicitly generic —
no project-specific content — so one source of truth serves all
vc-managed workspaces.

### Scope

**Port from iiac-perf/CLAUDE.md:**

- `## Memory` — direct bots away from the per-project memory
  directory (`~/.claude/projects/<path>/memory/`) in favor of
  CLAUDE.md and committed `notes/`. Discoverable, reviewable,
  committed.
- `## Speculation marker` — prefix mechanism / causal / predictive
  claims in durable text with "The bot thinks ..." so measured vs
  inferred stays visible.
- `## Commit-Push-Finalize Flow` — the stricter per-step discipline
  (run the full flow after every step, not only session-end;
  "after finalize: stop and wait" hard stop).
- Improved `## Pre-commit Requirements > Review before proposing
  the commit block` — summarize and stop before pre-emptively
  laying out commit commands.
- Commit-body guidance — app-repo body mirrors the chores-N.md
  edits as a scan-able index, not a duplicate.

**New in this refresh (not in iiac-perf):**

- `## Versioning` section **in CLAUDE.md** (moved from
  `notes/README.md`), including the new `-N` pre-release suffix
  convention: `0.37.0-0`, `0.37.0-1`, ..., `0.37.0` as the done
  marker. Replaces `-devN`. `notes/README.md` keeps a brief
  pointer to CLAUDE.md.
- `## Code Conventions` additions (lifted from current
  `memory/`):
  - **Doc comments on every file** (`//!` module docstring at
    top of each `.rs`) **and every fn/method** (`///`). Matches
    existing vc-x1 style. Overrides CLAUDE.md's generic "no
    comments" default explicitly.
  - **Ask on ambiguity** — when input is ambiguous, ask for
    clarification rather than spinning.
  - **Stuck detection** — 5+ min of thinking on a simple task
    means stuck; stop, summarize what's blocking, ask.

### Memory dir cleanup

- Replace `memory/MEMORY.md` with an empty stub (one-line
  comment pointing at CLAUDE.md).
- Delete individual feedback files — their content either
  already in CLAUDE.md or folded into the new Code Conventions
  section per above.

### Rationale

CLAUDE.md is committed, diffed, reviewed, and visible to every
collaborator — human or bot, any machine. The memory directory
is hidden under `~/.claude`, machine-local, and invisible to
review. Easy-for-everyone-to-find beats convenient-for-the-bot.
Same argument scales across all vc-managed projects, which is
why CLAUDE.md goes generic and ships in `vc-template-x1/`.

### Canonical vs user-facing content

CLAUDE.md is the **single canonical home** for any rule the bot
should follow (versioning, commit style, commit-push-finalize flow,
code conventions, pre-commit checklist). Each rule lives in exactly
one place. READMEs — both project-top-level and `notes/` — contain
only user-facing content (what the tool does, how to install, how
to clone) plus **pointers** into the relevant CLAUDE.md sections
for the bot-facing topics. No rule text is duplicated across files.

Rationale: users will scan README first, so the pointers need to be
there; but maintaining the same rule in two files drifts and rots.
One source of truth, cheap links from everywhere else.

### Files touched

vc-x1 repo:

- `/CLAUDE.md` — full refresh (new canonical baseline)
- `/README.md` — update `## Contributing` to point at CLAUDE.md
  sections (was pointing at now-removed `notes/README.md` sections)
- `/notes/README.md` — strip Versioning + old Code Conventions
  sections; add pointer to `CLAUDE.md`

vc-template-x1 repo (sibling, separate repo, parallel commit):

- `CLAUDE.md` — byte-identical to vc-x1's new CLAUDE.md
- `README.md` — add `## Contributing` section (didn't exist) with
  the same pointer structure as vc-x1's
- `notes/README.md` — strip `## Versioning during development`
  (outdated `-devN` form); add pointer to `CLAUDE.md`

Memory dir (`~/.claude/projects/-home-wink-data-prgs-rust-vc-x1/memory/`):

- `MEMORY.md` — empty stub (one paragraph pointing at `CLAUDE.md`)
- `feedback_*.md` (15 files), `user_profile.md` — delete

## Test harness refactor (0.36.2)

Lift `sync`'s inline test harness into a shared module so `push`'s
tests (and any future subcommand's) can reuse it without
copy-paste.

### Scope

- Extract from `sync.rs:521–560` into a shared location:
  - `unique_base(tag)` — tempdir path builder with ns-timestamp
    + per-process atomic counter for parallel-test collision
    avoidance
  - `Fixture` — owned dual-repo fixture struct with RAII cleanup
    via `Drop`
- Target module: `src/test_helpers.rs` as a new file, or a
  `#[cfg(test)] pub mod helpers` inside `test_fixture.rs`. Decide
  during implementation; `test_helpers.rs` probably cleaner since
  `test_fixture.rs` already has its own purpose (the
  `test-fixture` subcommand handler).
- Migrate `sync`'s integration tests to the shared module. No
  behavior change; tests pass identically.
- No new functionality; pure refactor.

### Rationale

`push`'s tests want the same dual-repo fixture + unique tempdir
pattern. Copy-paste would work but accretes drift; a shared
harness keeps tests consistent and lowers the bar for future
subcommand integration tests (which the todo list already calls
out as a general need).

## Sync improvements — single-repo support + quieter dry-run (0.36.3)

Two related tweaks so `vc-x1 sync` is fit to be (a) run against
single-repo projects like `vc-template-x1` and (b) codified as a
pre-work discipline in `CLAUDE.md`'s Commit-Push-Finalize Flow
without generating noise in the common clean-state case.

### Scope

- **Add repeatable `-R` / `--repo` flag to the `sync` CLI.**
  Currently `sync.rs:29` hardcodes `const REPOS: [&str; 2] =
  [".", ".claude"]`. The underlying `sync_repos()` already takes
  `&[PathBuf]`, so the change is purely CLI: accept repeated /
  comma-separated `-R`, default to `.,.claude` when none given.
  Matches the `-R` form on `chid`, `desc`, `list`, `show`.

- **Collapse the all-up-to-date output to a one-line summary** so
  "run sync before work" is genuinely cheap to sprinkle
  everywhere. `sync.rs`'s `run_plan` now fetches and classifies
  silently (capturing `jj git fetch`'s stderr rather than letting
  it stream via `common::run`'s `info!` hook), then emits based on
  outcome:
  - clean everywhere: one line, `sync: N repos, all up-to-date`;
  - action needed: per-repo fetch + state + `dry-run` hint.
  Initial scoping only suppressed the trailing hint — expanded
  after live testing showed 6 lines of chatter was still too much
  to sprinkle silently through the workflow.

- **Add `-q` / `--quiet` for scripts.** When set, temporarily
  clamps `log::max_level` to `Warn` for the duration of the sync
  call (restored on return), so all `info!` output from sync plus
  any subprocess stderr routed through `common::run` goes dark.
  `Warn` / `Error` still surface so script callers don't lose
  diagnostics. Exit code is the only signal in the common case.

- **Codify "sync before work" in `CLAUDE.md`'s Commit-Push-Finalize
  Flow** as a pre-step. Only useful once the quieter-dry-run polish
  lands, so it ships in this same chore.

### Rationale for keeping dry-run the default

While we're in here: the decision *not* to flip sync's default to
`--no-dry-run` is deliberate, capturing the conversation that led
to this chore:

- **Consistency.** `vc-x1 fix-desc` also defaults to dry-run. A
  divergent default for `sync` would rot the convention.
- **Preview before mutation is load-bearing in the `Diverged`
  case.** Even with `op_restore` rollback, *not* triggering a
  conflicted rebase at all is better than triggering one and
  rolling back. The preview lets the user inspect the remote
  commit first.
- **Composition layering stays clean.** Once `push`'s preflight
  stage (0.37.0) calls `sync`, it invokes it with `--no-dry-run`
  internally because `push` is the commit-to-mutation moment.
  Interactive `sync` stays exploratory; `push` is the doer. If
  interactive `sync` mutated by default, users would learn two
  different "sync"s (the interactive one and the preflight one).

User guidance phrasing (lands in CLAUDE.md): "Run `vc-x1 sync` to
see state; re-run with `--no-dry-run` when you're ready to apply.
`push` does this for you automatically."

### Forward pointer: return-value / exit-code refinement

The current exit code is binary (`0 = success`, non-zero = error).
For `--quiet` scripted use, callers may eventually want a richer
signal — e.g. `0 = clean`, `1 = action taken`, `2 = action needed
but in dry-run`, `3 = error`. Not needed for 0.36.3's use cases;
flagged here so the moment a script wants to distinguish "synced
cleanly" from "already clean" we know the conversation started.

### Version

Single-step `0.36.3` — five touches that compose into one
deliverable (CLI `-R` flag, `--quiet` flag, output collapse to
one-line summary, hint gating, docs).

## Add push subcommand (0.37.0)

Collapse the dual-repo commit+push+finalize ceremony into a single
resumable `push` subcommand. Today's workflow is correct but has
many manual steps (commit app, commit .claude, advance both
bookmarks, push app, finalize .claude) with easy-to-miss gates in
between. `push` owns the choreography so the human doesn't have to.

### User-visible flow: two approval gates

Today has N approval gates (one per step). Collapse to two:

1. **Before commit-message authoring** — "is the work done right?"
   User reviews the diff / tests. All pre-flight checks (fmt,
   clippy, test, install, retest) have already run by this point.
2. **After commit-message authoring** — "is this the right
   message?" User reviews the shared title+body. On approval,
   `push` performs commits → bookmark advances → push app →
   finalize .claude as one sequence.

The mechanical between-step gates (approve each commit, approve
each bookmark move, approve push, approve finalize) go away.

### Unified commit message

Both repos get the **same title and body**; only the ochid trailer
differs per repo. The session-body / app-body split is dropped —
the session *is* the code work here, so one description reads fine
in both histories.

### Empty-.claude handling

If `.claude` has no changes (empty working copy), skip the .claude
commit and point the app commit's ochid at `.claude`'s current
`@-` (latest session commit). Rationale:

- Every app commit still has a resolvable ochid counterpart —
  existing validators (`validate-desc`, `fix-desc`) stay simple.
- Semantic is truthful: "this code change references the session
  state at commit X."
- Avoids special cases like `ochid: none` that every downstream
  tool would have to handle.

### Resumable state machine

`push` is a state machine with persistent progress. Bare `push`
auto-resumes from the last incomplete stage; explicit flags
override.

**Stages:**

1. `preflight` — fmt / clippy / test / install / retest
2. `review` — show diff, wait for approval #1
3. `message` — compose/edit commit message, wait for approval #2
4. `commit-app`
5. `commit-claude` (skipped if `.claude` has no changes — decision
   recorded in state so resume doesn't re-check)
6. `bookmark-both` — advance both bookmarks to `@-`
7. `push-app`
8. `finalize-claude`

**State file**: location defined in `.vc-config.toml` under a new
`[push]` section, defaults:

```toml
[push]
state-dir = ".vc-x1"
state-file = "push-state.json"
```

Stored in the app repo. Contains stage reached, commit message,
ochid decisions, bookmark name, `jj op` snapshot IDs for both
repos, and timestamps. Deleted on successful completion.

**Missing / invalid state handling** (two distinct scenarios):

- *First run, no state file*: normal — proceed from stage 1
  silently.
- *Resume with corrupt state, or repo op-ids no longer match the
  recorded snapshot* (someone committed / rewound in between):
  graceful failure with message suggesting `--restart`, optionally
  combined with `--step` so the user controls re-execution stage
  by stage.

**Atomic rollback via `jj op`**: reuse `sync`'s snapshot pattern
(`sync.rs:82–99`). Before stage 4 (`commit-app`), snapshot both
repos' op ids. Any failure in stages 4–6 restores both repos to
their snapshot op, so failed runs leave no half-state behind.
After stage 7 (`push-app`) succeeds, the boundary is crossed —
the app commit is now remote and immutable; rollback is no longer
an option, so from that point on failure recovery is forward-only
(retry `push-app`, retry `finalize-claude`, or fall into the
`--ignore-immutable` squash path).

**`.gitignore` coherence**: the state file must not be committed.
`init` / `clone` add `state-dir` to `.gitignore`; `push` verifies
on every run that the currently configured `state-dir` /
`state-file` are matched by a `.gitignore` entry, and warns (not
fails) if not. The warning nudges the user to update `.gitignore`
after changing the config — otherwise config and gitignore can
drift silently.

### Tests

Add unit and integration tests on top of the shared test harness
landed in `0.36.2`. No copy-paste of fixture infrastructure —
everything goes through the shared module.

### Flags

- `--bookmark <name>` — required; same semantics as today's
  `finalize --bookmark`
- `--restart` — clear state file, start from stage 1
- `--from <stage>` — explicit jump (advanced / debug use)
- `--step` — pause between every stage (recovers today's
  one-gate-per-step feel for users who want it)
- `--status` — print where the state file thinks we are, then exit
- `--recheck` — re-run `preflight` even on resume (default: skip
  preflight if the last run succeeded)
- `--no-finalize` — stop before `finalize-claude` so the user can
  run it manually (debug / safety)
- `--dry-run` — print the exact commands for every stage, no side
  effects
- `--title <str>` / `--body <str>` — compose message inline,
  skipping `$EDITOR`

### Failure modes and recovery

- **Preflight fails**: state stays at `preflight`. User fixes,
  re-runs `push`. No data to roll back.
- **Approval declined**: state resets cleanly — user re-runs.
- **Commit fails** (jj error): state stays at `commit-app` or
  `commit-claude`. Error message explains the jj failure; user
  fixes, re-runs.
- **Push fails** (non-fast-forward, network): state at `push-app`.
  Resume retries push. User may need to fetch/rebase first.
- **Finalize partially fails**: `finalize --detach` returns before
  it completes. `push` either waits (blocking mode) or checks the
  status marker `finalize` leaves in `.claude` to decide if resume
  should re-run it.
- **Post-push late edit** (CLAUDE.md tweak, memory update): the
  app commit is already remote and immutable. Recovery matches
  today's pattern — `jj squash --ignore-immutable`, re-push.
  `push` could detect "state file completed but working copy
  dirty" and offer the squash path, but first cut keeps this
  manual.

### Open questions / TBD

- **Bookmark resolution** *(deferred — 0.37.0-1 keeps `--bookmark`
  or positional required)*: long-term, `push` should auto-detect
  the target from `@-`'s bookmarks so the common case reads as
  `vc-x1 push` with no argument. Blocking prerequisite: a
  richer bookmark enumeration primitive that reports, per
  bookmark, whether a remote counterpart exists (`@origin`) and
  whether the local bookmark tracks it. That information is needed
  to pick a sensible default and to refuse auto-detect when the
  situation is ambiguous. Today's `format_bookmarks_at` helper
  (0.36.0) returns plain names only — extending it with remote /
  tracking flags is the 0.38.x-scale follow-up that enables this.

- **Post-push immutability**: once the app commit is pushed, we
  can't retry `commit-app` via resume; need `--ignore-immutable`
  squash path instead. State machine must record the post-push
  boundary explicitly.
- **Editor invocation**: `$EDITOR` interactive, or compose inline
  via `--title` / `--body`, or both? Probably both, with editor as
  default.
- **Non-tty**: in CI / scripted contexts, require `--yes` to skip
  approvals; fail otherwise.
- **Relationship to `finalize`**: `push` calls it; no
  flag-duplication layering — `push` owns the user-facing flags
  and invokes `finalize` with the right internals.
- **Dev-step coverage**: the current per-step push+finalize
  discipline (CLAUDE.md Commit-Push-Finalize Flow) is satisfied
  by the same subcommand with `--bookmark dev-X.Y.Z` or similar.

### Version

Multi-step with the new `-N` pre-release convention (numeric
suffix, no `dev` prefix). Pre-release identifiers compare
numerically and all sort below the suffix-free release, so the
final no-suffix bump is an unambiguous "done" marker.

Pre-work (shipped first, own commits):

- `0.36.1` — CLAUDE.md refresh + memory migration (this file, its
  own chore section above)
- `0.36.2` — test harness refactor (its own chore section above)
- `0.36.3` — sync improvements: `-R` flag + quieter dry-run +
  codify sync-before-work discipline (its own chore section
  above)

Push subcommand ladder (expanded from the original 4 to 6 after
adding an integration-test step ahead of the first dogfood):

- `0.37.0-0` — scaffolding: flag surface, `Stage` enum, stub
  `push()`
- `0.37.0-1` — state file + stage-dispatch loop with stage stubs;
  `--status`, `--restart`, `--from`
- `0.37.0-2` — real stage bodies (commits, bookmarks, push,
  finalize) + `jj op` snapshot rollback
- `0.37.0-3` — integration tests + workspace-root refactor
  (thread `root: &Path` through every stage so fixtures can
  target tempdirs); first `vc-x1 push` dogfood ships this
  commit
- `0.37.0-4` — interactivity: `-y/--yes`, review prompt,
  `$EDITOR`, message persistence across resumes, CLI-wide
  version banner
- `0.37.0-5` — polish: `--dry-run`, `--step`, non-tty detection,
  `.gitignore` coherence warning
- `0.37.0` — docs + workflow migration: update `CLAUDE.md`
  Commit-Push-Finalize Flow to point at `push`, update
  `notes/README.md` pointers, retire the by-hand steps. No
  suffix — this is the "done" marker.

### Per-step record

Running log of what each `0.37.0-N` commit shipped. Commit bodies
point at these per-step anchors for detail rather than the whole
design block above. "Pending" steps below get filled in as they
land.

#### 0.37.0-0 — scaffolding

- `src/push.rs` (new) — `PushArgs` flag surface, `Stage` enum
  (8 kebab-case variants), stub `push()` returning "not yet
  implemented"; 6 parse-test units
- `src/main.rs` — `mod push;`, `Push(PushArgs)` variant + dispatch,
  `propagate_version = true` so `-V` works on every subcommand
  (mid-review fix folded in)

#### 0.37.0-1 — state file + stage dispatch

- `src/push.rs` — `Stage::as_str` / `from_str` / `next` / `first`;
  `resolve_state_layout` reads `[push]` section of
  `.vc-config.toml` with defaults (`.vc-x1/push-state.toml`);
  `PushState` flat-TOML save/load with format-version guard;
  `--status`, `--restart`, `--from` control paths; dispatch loop
  saves state after each stage (bodies still stubs); bookmark
  accepted as positional or `--bookmark`; 9 new unit tests
- `.gitignore` — `/.vc-x1`
- Forward-pointer captured in Open Questions: richer bookmark
  enumeration needed before `vc-x1 push` can auto-detect from
  `@-`

#### 0.37.0-2 — real stage bodies + rollback

- `src/push.rs` — all 8 stage bodies wired (preflight shells to
  `cargo fmt/clippy/test`; review non-interactive; message
  collects chids + detects `.claude` pending state; commit-app /
  commit-claude / bookmark-both / push-app real; finalize-claude
  shells to `vc-x1 finalize --detach`); `jj op` snapshot at
  commit-app entry + `rollback_on_failure` restores both repos
  for failures in stages 4-6; `PushState` adds `app_chid`,
  `claude_chid`, `claude_had_changes`, `op_app`, `op_claude`
  (all `Option<_>` — older state files still load); 4 new unit
  tests
- `src/sync.rs` — `current_op_id` / `op_restore` promoted to
  `pub(crate)` for reuse

#### 0.37.0-3 — integration tests + workspace-root refactor

- `src/push.rs` — `pub(crate) fn push_in(workspace_root, args)`
  splits CLI entry (cwd) from test entry (fixture tempdir);
  every stage body + `rollback_on_failure` takes `&Path root`;
  `claude_path(root)` helper; `rollback_on_failure` promoted to
  `pub(crate)` so tests can exercise rollback directly; new
  `#[cfg(test)] mod integration_tests` with 4 end-to-end tests
  (happy-clean, happy-dirty, rollback, resume)
- First `vc-x1 push` dogfood ships this commit

#### 0.37.0-4 — interactivity + version banner

- `src/push.rs` — `-y/--yes` on `PushArgs`; `title`/`body`
  persisted in `PushState` via `escape_multiline` /
  `unescape_multiline`; `stage_review` prints `jj diff --stat`
  and prompts `[y/N]` unless `--yes`; `stage_message` resolves
  message by precedence (flags → persisted state → `$EDITOR`
  template); `compose_message_via_editor` writes template under
  `state_dir`, launches `$EDITOR` (`VISUAL` → `vi` fallback),
  parses saved content (strips `#` comments, splits title/body
  on first blank line, aborts on empty); `resolve_message`
  helper consolidates title/body lookup; `run_stage` threads
  `&StateLayout`; 2 new unit tests (multiline-escape round-trip,
  `parse_message` cases)
- `src/main.rs` — `BANNER` + `TOP_ABOUT` consts built at compile
  time from `CARGO_PKG_NAME` / `CARGO_PKG_VERSION`;
  `cli_with_banner` walks clap tree with `mut_subcommand` to set
  `before_help` on every subcommand (top-level's `about` carries
  the combined `name + version + tagline` on one line);
  `main()` switched from `Cli::parse()` to `get_matches() +
  Cli::from_arg_matches` so the customized tree is what clap
  parses; `BANNER` emitted as `info!` at the start of every
  run except the detached `finalize --exec` re-entry, and
  suppressed when the active subcommand has `--no-label` set
  (so `chid -L` / `desc -L` / `list -L` / `show -L` stay
  script-parseable)

#### 0.37.0-5 — polish *(pending)*

Planned: `--dry-run` (print commands, no side effects),
`--step` (pause between every stage), non-tty detection (fail
fast when interactive without `--yes`), `.gitignore` coherence
runtime warning (check the configured state path is ignored,
warn if not).

#### 0.37.0 — docs + workflow migration *(pending, done marker)*

Planned: retire CLAUDE.md's by-hand Commit-Push-Finalize Flow
in favor of `vc-x1 push`; add a `push` section to `README.md`;
annotate this chore block with a "shipped" trailer.
