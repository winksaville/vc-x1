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

#### 0.37.0-5 — polish

- `src/push.rs` — `--dry-run`: skip every side-effect subprocess
  (preflight / commit-app / commit-claude / bookmark-both /
  push-app / finalize-claude), emit `[dry-run] would run: ...`
  lines instead; review still shows the diff (that's the *point*
  of dry-run); state file not persisted in dry-run so a later
  real run starts clean. `--step`: after each completed stage
  (with a next stage pending), prompt `[y/N]` to continue;
  `--yes` short-circuits; non-tty without `--yes` errors fast.
  Non-tty detection via `std::io::IsTerminal` — review stage
  errors when stdin isn't a tty and `--yes` isn't set; message
  stage does the same before launching `$EDITOR`.
  `check_gitignore_coherence` — reads `.gitignore` at the
  workspace root and warns (non-fatal) when the configured
  state-dir name isn't present. Helps catch the "user changed
  `[push].state-dir` in `.vc-config.toml` and forgot to update
  `.gitignore`" case.

#### 0.37.0 — docs + workflow migration (done marker)

- `src/push.rs` — fold `vc-x1 sync --no-dry-run` into `preflight`
  as the first step (divergence resolved before cargo burns
  cycles); matches the 0.36.3 design's "push preflight calls
  sync" note.
- `CLAUDE.md` (both repos, byte-identical) — rewrite
  Commit-Push-Finalize Flow around `vc-x1 push`: intro says
  "use push"; drops the two-checkpoint manual ceremony
  (Checkpoint 1 / Checkpoint 2 / Finalize the .claude repo);
  keeps "After finalize: stop and wait" and "Late changes
  after push"; adds a "Manual finalize fallback" subsection
  for `--no-finalize` and post-failure cases; pre-step sync
  section reframed as "still useful" (push runs sync internally
  but running it earlier surfaces divergence sooner).
- `README.md` — new `### push` subsection under Usage: stage
  table, flag table, state-file config, link to this chore's
  per-step record.
- `notes/chores-05.md` — per-step record filled in for this
  step; block now complete through `0.37.0`.

**Status: shipped.** The push subcommand design is implemented,
tested, and documented. The manual Commit-Push-Finalize Flow is
retired in CLAUDE.md — `vc-x1 push <bookmark>` is the primary
entry point going forward.

## First-dogfood polish for push (0.37.1)

First dogfood of the interactive `vc-x1 push` flow exposed seven
papercuts. They're all small, all surfaced in the same session, and
all share the same "post-ship polish" framing — so they ship as one
single-step `0.37.1` rather than a `0.37.0-6` continuation. The
0.37.0 arc was closed by the done marker; this keeps the dev ladder
of the feature distinct from sandpaper.

### 1. `$EDITOR` template wording + parser relaxation

The header was structured "title on first line / blank line / body
after" — easy to read as "title now, body in a second prompt later".
A user typed only the title, `:wq`'d, and was surprised when push
proceeded with an empty body instead of re-prompting. The template
also required a literal blank line between title and body — if a
user typed title-then-body with no blank between, `parse_message`'s
`splitn(2, "\n\n")` made the *whole thing* the title. Silent
footgun.

- `src/push.rs` — template header rewritten:

  > Leave as-is to abort. Enter the Title on the first line,
  > optionally followed by the Body on subsequent lines. The
  > blank line between Title and Body is inserted automatically,
  > and the ochid trailer is appended per-repo automatically —
  > don't add either here.
  > Lines starting with `#` are ignored.

  States the contract up front (one editor session, title on line
  1, body optional after) and surfaces the two auto-insertions
  (separator blank line, ochid trailer) so the user knows not to
  type either.

- `src/push.rs` — `parse_message` rewritten: first non-comment
  line = title, remainder (trimmed) = body. Blank line between is
  allowed but not required. The "blank line between title and body
  in the final commit message" responsibility lives in the
  commit-stage `format!` calls (`"{title}\n\n{body}\n\nochid: ..."`),
  which were already there.

- `src/push.rs` — `parse_message_cases` grows two new cases:
  title+body with no blank line (new behavior), and a multi-line
  body with internal blank lines (regression guard).

### 2. Gitignore coherence: fatal in push, auto-write in init + fixture

The gitignore-coherence check was a `warn!` — easy to miss in the
preflight wall of output. A committed push-state file is a real
foot-gun (jj would track session-only state into history). Promote
to fatal, and have `init` and `test-fixture` ship `/.vc-x1` in their
generated `.gitignore` so the fresh-repo path is clean.

- `src/push.rs` — `check_gitignore_coherence` returns
  `Result<(), Box<dyn Error>>` and `push_in` propagates with `?`.
  Error message names the file and the line to add. No bypass
  flag; the fix is one line.

- `src/init.rs` — `GITIGNORE_CODE` adds `/.vc-x1`; matching test
  asserts the new line.

- `src/test_fixture.rs` — `GITIGNORE_CODE` adds `/.vc-x1`; matching
  test asserts the new line. Push integration tests use this
  fixture, so the now-fatal check passes there transparently.

- `README.md` — fixture-layout snippet adds `/.vc-x1`.

### 3. `vc-x1 sync`: `--check` / `--no-check` rename + fatal check mode

Push's preflight ran `vc-x1 sync --no-dry-run`, which auto-rebased
the local repo against the remote *before* the user reached the
review/message gates. A surprise rebase mid-push is exactly the
kind of unsupervised mutation the gates are meant to prevent.

Rename `--no-dry-run` to `--no-check` and `--dry-run` to `--check`
(the implicit default; passable explicitly). Check mode is now
**fatal** when any repo is `behind`/`diverged` — the user must
resolve with `vc-x1 sync --no-check` and re-run. Push's preflight
calls `vc-x1 sync --check` so divergence surfaces early but never
rewrites local state without explicit consent.

The bot uses explicit forms in scripts and automation (per
"defaults can shift, explicit flags lock in the contract"); the
interactive user can rely on the default.

- `src/sync.rs` — `SyncArgs.no_dry_run: bool` → two fields
  `check: bool` and `no_check: bool`, mutually `conflicts_with`.
  Field `args.no_dry_run` → `args.no_check` everywhere internally
  (semantic preserved: true = act). New phase-4 logic: in check
  mode (`!args.no_check`) with `any_action_needed`, return Err
  with explicit "resolve with `vc-x1 sync --no-check` and re-run"
  message and a per-repo state recap above (already printed by
  phase 2).

- `src/sync.rs` — `parse_defaults` / `parse_overrides` tests
  updated; new `parse_check_flag` and `parse_check_no_check_conflict`
  tests exercise the new flag surface.

- `src/sync.rs` — `apply_args()` test helper updated to
  `{ check: false, no_check: true, ... }`.

- `src/push.rs` — `stage_preflight` calls `vc-x1 sync --check`
  (was `--no-dry-run`). Doc comment rewritten to explain why
  check mode (no auto-rebase before the gates).

- `CLAUDE.md` — "Pre-step: `vc-x1 sync` (still useful)" section
  rewritten around `--check`/`--no-check`. Output-shape bullets
  updated to describe the fatal check-mode error.

- `README.md` — sync usage examples + flag table + output-shape
  section updated. Push-stage table's `preflight` row updated to
  `vc-x1 sync --check`.

### 4. Stage-prefix log format: `push:STAGE:` → `push STAGE:`

Stage-tagged log lines were `push:preflight:`, `push:review:`,
… `push:finalize-claude:`. With nine prefixes plus subprocess
output, the colon-pair felt dense; a single space between "push"
and the stage parses faster.

- `src/push.rs` — nine stage prefixes renamed (preflight,
  review, message, commit-app, commit-claude, bookmark-both,
  push-app, finalize-claude, step). Bare `push: ...` lines
  (top-level, no stage) are unchanged.

  Side note: `push push-app:` now reads with the word "push"
  twice in a row. The stage's name is `push-app` because its
  job is to push the app repo; renaming it would touch the
  state-file format and `--from` values, which is wider surgery
  than warranted here.

### 5. Subprocess output hidden by default (visible with `-v`)

Subprocess stderr (jj's `Working copy now at ...`, `Rebased N
commits`, etc.) was emitted at `info!` in `common::run`, so it
showed by default. Combined with the dense `push:STAGE:` prefix
and the multi-stage flow, the output felt noisy. Drop subprocess
stderr to `debug!` — hidden by default, surfaced with `-v`. The
debug formatter already indents per-line two spaces, so when `-v`
is set, subprocess lines sit visually under the caller's `info!`
header. This change subsumes the originally-planned "indent
stderr two spaces in info!" fix.

- `src/common.rs` — `run` doc comment rewritten. Subprocess
  stderr success path: `info!("{stderr}")` → `debug!("{stderr}")`.
  Subprocess stdout: `debug!("  {stdout}")` → `debug!("{stdout}")`
  (drop the redundant inline indent — the formatter already
  indents debug lines).

  Scope note: `common::run` is shared by every subcommand, so this
  silences subprocess output globally (init, sync, finalize, clone,
  push). Trade-off accepted: high-volume jj chatter is rarely
  load-bearing for the user, and `-v` brings it back. If a specific
  subcommand needs a particular subprocess line surfaced, the right
  fix is an explicit `info!` in that subcommand at the moment the
  thing happens — not relying on subprocess passthrough.

### 6. Sync up-to-date wording: clarify scope (bookmarks, not work tree)

`sync` reported `sync: N repos, all up-to-date` regardless of whether
the working copy had uncommitted changes. Reads as broader than its
actual scope (bookmark-vs-remote tracking only). User dogfood: ran
`vc-x1 sync` after `echo hello > hello.txt`, got "all up-to-date"
while `jj st` showed the new file — confusing.

Wording-only fix here. The richer "tell me about working-copy state
too" feature is a follow-up — see open-question subsection below.

- `src/sync.rs` — phase-2 summary `sync: {n} {noun}, all up-to-date`
  → `sync: {n} {noun}, all bookmarks up-to-date`. The per-repo
  `State::UpToDate` line (`{repo}: up-to-date`) is unchanged — its
  scope is already clear from context.
- `src/main.rs` — sync `long_about` output-shape example updated.
- `CLAUDE.md` — output-shape clean bullet adds a scope note: "scope
  is bookmark-vs-remote tracking, not working-copy cleanliness".
- `README.md` — same scope note in the output-shape section, plus
  a `jj st` pointer for the working-copy case.

### 7. Versioning + bookkeeping

- `Cargo.toml` / `Cargo.lock` — version `0.37.0` → `0.37.1`.
- `notes/todo.md` — Done entry added (covers all six fixes); new
  Todo entry added for the deferred working-copy-signal work.

## Open: sync up-to-date should mention working-copy state

Deferred from 0.37.1's wording fix (subsection 6 above). Wording
clarified the scope (`all bookmarks up-to-date`) but the user's
underlying need was a single command that says "you have N pending
files in repo X" without having to run `jj st` per repo.

Design-open. Sketch of the question space:

- **Format.** Per-repo line under the summary
  (`./: 1 pending file (hello.txt)`) vs. compact aggregate
  (`sync: 2 repos clean, 1 with pending changes`)?
- **Detail level.** Just count? Count + names (cap at N)? Stat-style?
- **Always-on vs --status.** Show every run, or gate behind a
  `--working-copy` / `--all` flag so the cheap clean-case stays
  one line?
- **Scope under push preflight.** Push always commits working-copy
  changes in the next stage, so a "you have pending changes" note
  during preflight is noise. Maybe sync grows the signal but
  `vc-x1 sync --check` (the form push uses) suppresses it?

Probable shape: per-repo "N pending file(s)" line under the summary
when non-zero, no flag needed, suppressed by `--quiet`. But the
push-preflight noise question needs a deliberate answer first.

## Temporary bookmark-tracking diagnostic probe (0.37.2)

0.37.1's first-dogfood push hit a `bookmark 'main' has non-tracking
remote 'main@origin'` error in the `.claude` repo during the
`finalize-claude` stage. Recovery was one-time: `jj bookmark track
main --remote=origin -R .claude`. But the open question is *when*
and *why* the tracking flag got lost — the fwlaptop op log shows
`.claude` pushing fine from there, so the 3900x workspace ended
up in a different state at some historical point.

The detect-and-error work (`notes/todo.md` top entry) is the
long-term policy fix. This chore is a **temporary diagnostic**
to localize the culprit: probe `main`-bookmark tracking status on
entry and exit of every vc-x1 command. If entry and exit differ
during a single command, that command is the culprit; if entry
differs from the previous command's exit, something *between*
invocations broke it (external tooling, manual jj commands, etc.).

Output shape (as originally shipped in 0.37.2; renamed to
`bm-track` in 0.37.3):

```
track-probe enter vc-x1 push: app(main)=tracked, .claude(main)=tracked
track-probe exit  vc-x1 push: app(main)=tracked, .claude(main)=NOT_TRACKED
```

- `src/main.rs` — `use std::path::Path;` added. Two new functions:
  `track_probe(phase, command_name)` prints the status line for
  both repos; `track_probe_one(repo, bookmark, remote)` queries
  jj via `jj bookmark list --tracked <bookmark> -R <repo>`. The
  tracked-list entry includes a line starting with `@<remote>:`
  when the bookmark is tracking that remote; absence means
  not-tracking.
- `src/main.rs` — dispatch block wraps the `match cli.command`
  in a `let exit_code = match ...;` binding, with `track_probe`
  calls immediately before and after. Skipped when `is_detached_exec`
  (the `finalize --exec` re-entry) since probe noise in that log
  isn't useful.
- Graceful degradation — if the repo doesn't exist, missing-`.jj/`
  detection emits `no-jj`; subprocess failure emits `err(<first line>)`.
  Never blocks the wrapped command.
- `Cargo.toml` / `Cargo.lock` — `0.37.1` → `0.37.2`.

**How to rip it out** once the culprit is identified (as framed
at the time of 0.37.2; superseded in 0.37.3 when the probe was
promoted to permanent):

```bash
grep -rn 'track-probe\|track_probe' src/
```

Three places to touch:
1. `src/main.rs` — delete the two functions (`track_probe`,
   `track_probe_one`) and the `use std::path::Path` import if
   nothing else needs it.
2. `src/main.rs` — revert the dispatch to plain
   `match cli.command { ... }` (drop the `let exit_code = ...`
   binding, the two `track_probe(...)` calls, and the
   `command_name` binding).
3. `Cargo.toml` / `Cargo.lock` — bump to the next appropriate
   version.

Net removal: ~80 lines.

## Fix bm-track bugs + rename + promote to permanent (0.37.3)

Three concerns bundled into this patch — all tied to the
bookmark-tracking probe added in 0.37.2:

1. **Two bugs in the probe itself** (the original motivation for
   0.37.3). First real use in 0.37.2 caught them — not vc-x1's
   push flow, as the output had misleadingly suggested.
2. **Rename** from `track-probe` / `track_probe*` to `bm-track` /
   `bm_track*` to avoid collision with the actor-x1 sibling
   project's "TProbe" (time probe) naming.
3. **Promote from temporary to permanent.** User data point:
   "I've had it happen more than once." Drops the 0.37.2
   ripout-checklist framing in favor of a stays-in-place sanity
   check.

### Bug A — cwd-relative paths mislabel and miss

The 0.37.2 probe hardcoded `Path::new(".")` and
`Path::new(".claude")` as the repo paths. Run from inside
`.claude`, those resolve to `.claude` and `.claude/.claude`
respectively — probe reported `app(main)=tracked,
.claude(main)=no-jj`, which is doubly wrong: the "app" column
is actually showing `.claude`'s status, and the ".claude" column
is showing a nonexistent path. Fix: walk up from cwd looking for
`.vc-config.toml` with `path = "/"`. That file marks the app-repo
side of the dual-repo pair, so its containing dir is the
workspace root. Probe `<root>` and `<root>/.claude` unconditionally
— gives consistent labeling from any cwd inside the workspace.
Outside any workspace, probe prints `no-workspace` and skips.

### Bug B — `@origin:` parse misses the divergent-decorated form

The 0.37.2 probe checked `line.trim_start().starts_with("@origin:")`.
But when local `main` is ahead of `@origin`, jj decorates the line
as `@origin (ahead by 1 commits): <old_commit>` — same tracking
relationship, different prefix, the check fails. **This is what
produced the scary-looking `NOT_TRACKED` at 0.37.2's push-exit
probe** — the bookmark WAS tracking, it was just briefly
local-ahead between `bookmark-both` and the detached finalize's
push. False positive. Fix: accept both `@origin:` (synced) and
`@origin ` (divergent-decorated, trailing space) as tracking
markers. The tracking relationship is what matters; sync-state is
orthogonal.

### Rename

Log prefix `track-probe` in 0.37.2's output collides conceptually
with actor-x1's "TProbe" (time probe). Renamed here to `bm-track`
(short for "bookmark tracking") — shorter, domain-specific, no
collision. Grep-replaced across source and the current chores
entry; the 0.37.2 subsection above was left using the original
`track-probe` name for historical accuracy at that shipping point.

### Promote to permanent

0.37.2 framed the probe as temporary with a ripout checklist.
User confirmed "happens more than once" — i.e., the tracking-loss
class of failures is rare but real, and a permanent always-on
sanity check beats waiting to rip the probe out after one catch.
Probe logic unchanged; only its doc comment and the 0.37.2
ripout-checklist framing are retired (the checklist is left in
the 0.37.2 subsection as historical record, annotated as
superseded here).

### Edits

- `src/main.rs` — `bm_track` rewritten to call
  `find_workspace_root()` first, then probe `<root>` +
  `<root>/.claude`. Graceful fallback to `no-workspace` when cwd
  is outside any vc-x1 workspace.
- `src/main.rs` — new `find_workspace_root()` helper. Walks up
  from cwd reading each `.vc-config.toml` it finds; matches if
  the file has `path = "/"` (with or without surrounding
  whitespace). Returns `None` at filesystem root if no match
  found.
- `src/main.rs` — `bm_track_one` tracking detection loosened.
  Now accepts either `@{remote}:` or `@{remote} ` (trailing
  space for the decorated form). Doc comment expanded to call
  out both cases.
- `src/main.rs` — functions and log prefix renamed:
  `track_probe` → `bm_track`, `track_probe_one` →
  `bm_track_one`, log prefix `track-probe` → `bm-track`.
- `src/main.rs` — `bm_track` doc comment drops "TEMPORARY" and
  the "Rip out once the culprit is identified" line. Adds context
  on why it stays in place (multiple past occurrences reported)
  and notes the deferred "silent when clean" refinement that
  would preserve detection value while removing steady-state
  noise (see `notes/todo.md`).
- `Cargo.toml` / `Cargo.lock` — `0.37.2` → `0.37.3`.

Implication for the chain of diagnoses: the `NOT_TRACKED` seen
at 0.37.2's push-exit probe was a false positive from Bug B.
There is no evidence yet of an actual tracking-loss bug inside
vc-x1's push flow. The *original* tracking loss that triggered
the 0.37.1 finalize-claude failure was real (`jj git push`
genuinely refused), but that loss happened before any probe
existed — origin lies somewhere in `.claude`'s historical
bootstrap, not in the recent commit+push sequence. The probe
(now permanent) continues to provide value going forward: it
would catch any new tracking-loss the moment it happens.

## Capture squash-mode + scope design for push (0.37.4)

After force-pushing 0.37.3 through the manual sequence (jj
squash --ignore-immutable → jj describe → jj git push on both
repos), the user asked: could `vc-x1 push` do this end-to-end
with the right flags? Answer — yes, and it's worth the effort.
This chore is doc-only: captures the design framing into
`notes/todo.md` so we don't lose it.

Two flag additions sketched:

1. `--scope=app|claude|both` (default `both`). Lets push run
   against one repo singly. Self-contained, independent of
   squash.
2. `--squash`. Squashes WC into `@-` via `--ignore-immutable`
   and force-pushes. Message stage pre-fills `$EDITOR` with
   the existing description. Motivation: the manual force-push
   sequence recurred three times during the 0.37.x dogfood
   (tf-1 removal, then 0.37.4-rolled-into-0.37.3 for the
   bm-track work, then would-have-been-again if we'd rolled
   THIS chore into 0.37.3). Not a rare edge case — a workflow.

Safety requirements for squash (captured in the todo entry):
`--force-with-lease`-equivalent, review gate surfaces the
rewrite (not just the diff), stage-prereq checks verify `@-`
is the commit we think it is.

Recommended ordering (earlier todos listed above this one):
state-sanity preflight → stage-prereq verification → `--scope`
(simpler, independent) → `--squash` (benefits from the first
three being solid).

- `Cargo.toml` / `Cargo.lock` — `0.37.3` → `0.37.4`.
- `notes/todo.md` — two new Todo entries under the push flag
  section: `--scope=app|claude|both` and `--squash`. The squash
  entry embeds the safety requirements + recommended ordering
  inline so future work has the framing without needing to
  re-derive it from chat transcripts.

Also captured in the same todo.md pass (follow-up conversation
after the initial framing):

- `--scope` entry grew a "warn on scope/WC mismatch" note —
  non-fatal warning when `--scope=app` leaves `.claude` pending
  changes on the table (or vice versa). Catches "I meant
  `--scope=both`" before commit.
- New Todo: "oh shit" revert via a `.vc-x1-ops/` anchor dir.
  Idea-stage only. Every repo-mutating command drops an anchor
  (command, per-repo pre-op-id, per-repo pre-push remote ref
  snapshot). `vc-x1 undo` restores via `jj op restore` +
  force-push to anchored remote ref. Piggybacks on jj's op-log
  retention for local, snapshots remotes separately. Generalizes
  beyond push (sync / finalize / init / fix-desc all eligible).
  Needs the same init + fixture + `.gitignore` discipline as
  `.vc-x1/`. Full framing lives in the todo entry.
