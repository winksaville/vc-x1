# Chores-06.md

General chores notes — design captures (forward-looking) and
post-implementation chore entries. Same shape as chores-01..05.md;
06 starts here only because 05 has gotten long.

Subsection headers use the trailing-version format from CLAUDE.md
when they correspond to a release: `## Description (X.Y.Z)`.
Pre-implementation design captures may use a plain title; once
implemented, the title can become a release-versioned chore.

## Notes restructure: chores-06 + trim long todo entries (0.37.7)

Notes-only release. Three doc-hygiene moves bundled:

- Start `notes/chores-06.md` (this file). chores-05.md grew to
  ~1000 lines; new chore entries land here going forward. No
  reservation for design-only — chores-06 is general-purpose.
- Trim long todo entries to 2-3 line summaries; move detail to
  the relevant chores-05 design subsection or a new chores-06
  subsection. One item per top-level list entry, no `##`
  subsections, loose topical ordering only — and order
  highest-priority-first.
- Adopt lazy numbering for Todo items: every entry uses `1. `;
  the markdown renderer auto-numbers. Lets the user reference
  items by displayed number without manual renumbering on
  reorder/insert. Done section stays on `-` bullets — items
  aren't referenced by number once completed.

Design subsections added below for items that previously lived
inline in `notes/todo.md`:

- "Generalize --scope to all commands (design)"
- "Push hardening: state + stage sanity (design)"
- "bm-track silent-when-clean (design)"
- "Non-tracking-remote bookmark detection (design)"

Also: removed the redundant "Allow `vc-x1 push` to work on code
or bot repo together or independantly..." entry from todo —
fully subsumed by the `--scope` + `--squash` work.

- `notes/chores-06.md` — new file with intro + this chore entry +
  four design subsections.
- `notes/todo.md` — Todo section rewritten in priority order;
  long entries trimmed to short summaries; redundant entry
  removed; switched to `1. ` lazy numbering; Done entry +
  new `[60]`–`[64]` references added.
- `notes/README.md` — `## Todo format` updated with the
  lazy-numbering convention note.

## Generalize --scope to all commands (design)

Surfaced 2026-04-23 (from the user, top of todo): vc-x1 commands
should support both single and dual repo modes. The technique
sketched for `vc-x1 push` (`--scope=app|claude|both`) is the
right shape, but the scope concept should be a project-wide
convention rather than push-only.

**Naming:** `--scope=app|other|both` — "other" rather than
"claude" because the second repo isn't always `.claude`; could
be a different bot or a different sibling project structure.

**Why scope-first** (before adding more commands or push
features): each new command that operates across both repos
would otherwise need its own ad-hoc handling. Locking in the
convention now means new commands inherit it for free, and
existing commands (sync, finalize, push) get retrofitted under
the same vocabulary.

**Rough scope of the change:**

- Shared CLI flag definition + parsing helper (likely in
  `common.rs`).
- `sync`: already accepts `-R` for single-repo. Migrate to
  `--scope`; alias or deprecate `-R` (TBD below).
- `push`: `--scope` per the existing design, with
  warn-on-WC-mismatch.
- `finalize`: already takes `--repo`; rationalize naming.
- New commands (`status` etc.) start with `--scope` from day 1.

**Open:** do we deprecate `-R` immediately, alias it, or keep
both indefinitely? Probably: alias for one minor cycle,
deprecate, remove. Decide before implementation.

## Push hardening: state + stage sanity (design)

Two related correctness fixes for `vc-x1 push` resume behavior.
Both surfaced 2026-04-22 from a 0.37.1 dogfood incident and are
described together because they tackle the same failure mode
(silent false success after stale state).

**The incident:** 0.37.1 push errored at finalize-claude;
out-of-band recovery (manual finalize + force-push of a squash)
moved the world forward; 0.37.2 push then resumed at the parked
finalize-claude, no-op'd, and falsely declared "completed all
stages" while working copies still held uncommitted changes.

**State-sanity preflight on resume.** Before any stage runs,
verify saved state matches reality:

- `state.app_chid` still exists (not abandoned/rewritten).
- `main` bookmark at `state.app_chid`'s commit.
- `state.claude_chid` consistent with `.claude` working copy.

On mismatch, refuse with a loud "state is stale — run
`vc-x1 push --restart`" error.

**Stage-prereq verification + honest completion.** Each stage
declares what it expects (working-copy dirty for commit-app;
bookmark at specific commit for bookmark-both; etc.); the
dispatcher checks before running. "Completed all stages" should
only print when stages genuinely ran or were
verified-already-done, not when they were skipped without
verification.

**Implementation order:** state-sanity first (broader guard),
then per-stage prereqs (more invasive). Both should land before
the `--squash` work to avoid building on a known-fragile resume
path.

## bm-track silent-when-clean (design)

Today bm-track prints two lines per command (entry + exit)
unconditionally. In steady state both lines say
`tracked,tracked` — pure noise. Defer-flagged in 0.37.4 until
signal-confidence was established through more dogfood; several
0.37.x cycles have now used the probe with no false positives
or missed regressions.

**Proposed behavior:**

- Always probe on entry and exit (no change).
- Print on entry only when state isn't fully tracked.
- Print on exit when state isn't fully tracked OR when the exit
  state differs from entry.
- When printing on exit, include the entry state in the output
  so the transition is explicit:
  `bm-track vc-x1 <cmd>: enter=<state> → exit=<state>`.

The entry-state-in-exit-output requirement is important —
"entry was ok (not printed), exit failed" is inference;
"entry=tracked, exit=NOT_TRACKED" in one line is evidence.

**Implementation note:** record entry state in a local variable
at the enter call site and pass into the exit call.

## Non-tracking-remote bookmark detection (design)

Diagnosed 2026-04-22 dogfood: jj's tracking state is
**per-workspace** (local `.jj` store), not shared via git refs.
Sync across machines transfers refs but not the tracking flag —
so a fresh workspace fetched-into never auto-tracks. The failure
surfaces only when `jj git push` is attempted, which is too late
(push-app already succeeded in our case).

**Policy (decided):** error loudly, with the exact
`jj bookmark track <b> --remote=<r> -R <repo>` remediation
command. No self-heal — keeps the fix explicit and visible.

**Scope (every command that creates or mutates repo state):**

- `vc-x1 init`: already correct (Step 10 calls `jj bookmark
  track`). Add the check as a post-condition sanity assertion
  anyway.
- `vc-x1 clone`: does `git clone` then `jj git init --colocate`.
  Whether that combination auto-establishes tracking in jj's
  workspace store is unclear; probably needs an explicit
  `jj bookmark track` for each cloned bookmark after the init.
  Verify and fix.
- `vc-x1 test-fixture`: Step 7's `jj git push --bookmark main`
  establishes tracking as a side effect of the first push.
  Works correctly — confirmed via `jj bookmark list --tracked`
  on a fresh fixture. No change needed; the post-condition
  sanity check would naturally cover it.
- `vc-x1 sync` preflight: detect + error.
- `vc-x1 push` preflight: detect + error (before any mutation).
- `vc-x1 finalize`: detect + error before the squash, so a
  failed push doesn't leave a half-finalized state.

**Shared helper:** `common::verify_tracking(repo, bookmark,
remote) -> Result<(), Err>` or similar. Probably use
`jj bookmark list --tracked -T <template>` under the hood
rather than parsing human-readable output.

## Scope design refinements (0.37.8)

Notes-only release. Captures six refinements to the scope and
squash designs ([57], [60]) from a 2026-04-23 dogfood discussion
(the manual squash workflow used to fold late changes into the
0.37.7 commit, plus follow-up Q&A on cwd independence and
plain-old-repo handling).

**Scope semantic invariant.** With `--scope=app|other|both`
(per [60]):

- `--scope=both` (or omitted; `both` is the default) → dual-repo
  operation.
- `--scope=app` → single-repo operation on the app repo (direct
  reference).
- `--scope=other` → single-repo operation on the "other" repo
  (indirect reference; resolves via `.vc-config.toml`'s
  `[workspace].other-repo` field, which already exists).

Anything that isn't `both` is single-repo. The dispatcher branches
once on this invariant; new commands inheriting `--scope` get the
semantic for free.

**`--other` is project config, not a per-command flag.** Came up
during the same discussion: should there be a `--other=PATH` flag
to override the project's "other" repo per command? Answer: no.
That belongs in `.vc-config.toml` (already there as
`[workspace].other-repo`). Keeping `--other` out of the
per-command surface keeps the flag set small and the per-command
vocabulary uniform across all commands — "other" always resolves
the same way.

**Cwd independence + counterpart-rename suggestion.** Scope names
are workspace-anchored, not cwd-relative. Each repo has its own
`.vc-config.toml` self-describing its role:

- App's config: `[workspace] path = "/" other-repo = ".claude"`.
- `.claude`'s config: `[workspace] path = "/.claude"
  other-repo = ".."`.

`path = "/"` identifies the app repo (only the app has it). The
dispatcher reads the local config, determines "am I app or other?"
via `path`, then resolves `--scope=app` and `--scope=other` to the
right physical paths regardless of cwd. From `cd .claude`:

- `--scope=app` → `..` (the app repo, found via `.claude`'s
  `other-repo`).
- `--scope=other` → `.` (`.claude` itself, since `path != "/"`
  → I AM the workspace's "other").

Naming gotcha: the config field `other-repo` means "this repo's
counterpart" (cwd-flavored direction; from `.claude` it's `..` =
the app), while CLI `--scope=other` means "the workspace's 'other'
role" (workspace-anchored; always = `.claude`). They coincide
from app's cwd (both physically = `.claude`) but diverge from
`.claude`'s cwd (config `other-repo` = `..`, CLI `--scope=other`
= `.`). Consider renaming the config field to `counterpart` or
`peer` to disambiguate from the CLI scope name.

**Plain-old-repo handling: three workspace states.** vc-x1 should
be practical without a full workspace too. Three distinguishable
states, all described by `.vc-config.toml` presence/contents:

- **POR** — no `.vc-config.toml`. Pure git/jj repo, vc-x1
  doesn't know it exists. Implicit `--scope=app`;
  `--scope=other/both` errors with "not in a vc-x1 workspace
  (no `.vc-config.toml`) — drop --scope or use --scope=app".
- **Single-repo workspace** — `.vc-config.toml` with `path`
  only (no `other-repo`). vc-x1-aware, no companion. Example:
  `vc-template-x1`. Implicit `--scope=app`; `--scope=other/both`
  errors with "no other-repo configured. Add `other-repo = …`
  to `.vc-config.toml` to enable dual-repo operations".
- **Dual-repo workspace** — `.vc-config.toml` with `path` +
  `other-repo`. Full setup. Example: `vc-x1` itself.
  `--scope=app/other/both` all valid.

Edge cases: a `.vc-config.toml` with `path != "/"` and no
`other-repo` (e.g. `path = "/.claude"`) — the repo identifies as
the "other" side but its companion is missing. Error: "config
identifies this as `/.claude` but no `other-repo` to point at the
app side; companion is missing." Dual-repo with `other-repo`
pointing somewhere that doesn't exist → loud error (workspace
state corrupted), not POR fallback.

**`vc-x1 push --squash` composition.** With `--scope` in place,
the squash design (per [57]) composes naturally:

- `vc-x1 push --squash` (no scope) → squash both repos and
  force-push both. End-to-end version of the manual two-step
  recipe in CLAUDE.md "Late changes after push".
- `vc-x1 push --squash --scope=app` → squash app only.
- `vc-x1 push --squash --scope=other` → squash other only.

**Dogfood validation (2026-04-23).** The manual recipe was applied
successfully to both repos in sequence to fold late changes into
the 0.37.7 commit. Observations worth carrying into the `--squash`
implementation:

- jj's `jj squash --ignore-immutable` preserves the changeID of
  the squashed-into commit. Both repos' ochid trailers stayed
  valid through the squash without any fixup. The future
  `--squash` implementation can rely on this — no ochid rewrite
  needed.
- The trimmed "Late changes after push" recipe (0.37.6, two
  lines for the `@-` case) worked identically on both sides;
  no `jj bookmark set` needed since the bookmark moves with the
  rewritten commit.
- Push state was already cleared at the end of the prior
  successful push (state reported "completed all stages (state
  cleared)"). The post-recovery `--restart` / `rm` step
  documented in CLAUDE.md "Manual finalize fallback" wasn't
  needed in this case — but the doc remains correct for cases
  where prior push didn't clean up.

- `notes/chores-06.md` — this subsection.
- `notes/todo.md` — Done entry + new `[65]` reference + new
  Todo entry for the template restructure (vc-template-x1 +
  `.claude/` subdir, captured separately for later
  implementation).

## Bookmark tracking verification (0.38.0)

Cross-command tracking verification per the [63] design — every
repo-modifying command checks its target bookmark for non-tracking
remote refs and errors with the exact remediation command if any
are found. Multi-step rollout:

- **0.38.0-0** — shared helper + tests (foundational refactor; no
  behavior change beyond finalize).
- **0.38.0-1** — wire into setup commands (`init`, `clone`,
  `test-fixture`).
- **0.38.0-2** — wire into preflight commands (`sync`, `push`).
- **0.38.0** — release commit (notes + any doc tweaks).

### 0.38.0-0: shared helper + tests

`finalize` already had `find_non_tracking_remote` + a preflight
check at finalize.rs:163,215-232 (added in an earlier 0.37.x
release). Step 0 promotes that logic into `common.rs` so the
remaining commands can share it without copy-paste.

Signature deviates slightly from the [63] sketch
(`verify_tracking(repo, bookmark, remote)`): kept the existing
"detect any non-tracking remote for this bookmark" semantic
rather than checking a specific `(bookmark, remote)` pair —
matches what the call sites actually want, and avoids a second
helper layer.

```rust
pub fn find_non_tracking_remote(list_output: &str, bookmark: &str) -> Option<String>;
pub fn verify_tracking(repo: &Path, bookmark: &str) -> Result<(), Box<dyn std::error::Error>>;
```

`verify_tracking` runs `jj bookmark list -a <bookmark> -R <repo>`,
parses the output via `find_non_tracking_remote`, and returns
`Err` with the standard message:

```
bookmark '{b}' has non-tracking remote '{b}@{r}' —
run `jj bookmark track {b} --remote={r} -R {repo}` to fix
```

- `src/common.rs` — added `find_non_tracking_remote` + `verify_tracking`
  + 4 parser tests (moved from `finalize.rs`).
- `src/finalize.rs` — removed the local `find_non_tracking_remote`
  fn + 4 tests; preflight now calls `crate::common::verify_tracking`.
  Behavior unchanged.
- `notes/chores-06.md` — new `## Bookmark tracking verification
  (0.38.0)` parent + this `### 0.38.0-0` sub-section + stubs for
  `### 0.38.0-1` / `### 0.38.0-2`.
- `notes/todo.md` — Done entry for this step + In Progress entries
  for the remaining 0.38.0 steps + new `[66]` reference.

### 0.38.0-1: wire into setup commands

Wired `common::verify_tracking` into the three setup commands as
post-condition assertions. One real fix landed (clone), two
sanity assertions (init, test-fixture).

**Empirical answer to the [63] open question:** `jj git init
--colocate` after `git clone` does **NOT** auto-establish
bookmark tracking. Probed live (cwd `/tmp`):

- jj's own hint after colocate: "The following remote bookmarks
  aren't associated with the existing local bookmarks:
  main@origin. Run `jj bookmark track main --remote=origin` to
  keep local bookmarks updated on future pulls."
- `jj bookmark list -a main` shows `main@origin` at column 0
  (the non-tracking format that `find_non_tracking_remote`
  detects).
- `jj bookmark list --tracked` returns empty.

So clone.rs needed an explicit `jj bookmark track` step after
each colocate — added in this commit. Without it, every fresh
`vc-x1 clone` would have left both repos in a non-tracking state
that the new preflight checks (0.38.0-2) would reject — exactly
the silent-bug-now-loud-error story the [63] design targets.

**Wiring details:**

- `clone` Step 3/4: after `jj git init --colocate`, run
  `jj bookmark track main --remote=origin`, then
  `verify_tracking(&dir, "main")?`. Real fix.
- `init` Step 10: already had the explicit track call; added
  `verify_tracking(&dir, "main")?` after each side as a sanity
  assertion. No-op on happy path; catches regressions.
- `test-fixture` Step 7: each `jj git push --bookmark main`
  establishes tracking as a side effect (confirmed correct in
  [63] design). Added `verify_tracking(&dir, "main")?` after
  each push as a post-condition assertion. No-op on happy path.

All 204 tests pass — the test suite exercises test_fixture
heavily (every integration test uses it via `Fixture`), so the
new assertions there are well-validated. init and clone don't
have full integration tests (would need real git remotes); the
new assertions there are short-circuit safety nets.

- `src/clone.rs` — Step 3/4 add `jj bookmark track` + verify.
- `src/init.rs` — Step 10 adds verify after existing track calls.
- `src/test_fixture.rs` — Step 7 adds verify after each push.
- `notes/chores-06.md` — promote `### 0.38.0-1` from TBD to
  filled (this).
- `notes/todo.md` — Done entry + `## In Progress` entry for
  `0.38.0-1` removed.

### 0.38.0-2: wire into preflight commands

Wired `common::verify_tracking` into the two preflight commands.
With `finalize` already covered (0.38.0-0), the full set of
repo-modifying commands is now under the same check.

**`sync` (sync.rs `sync_repos`):** added a verify loop at the top
— for each repo in the operation, verify tracking on
`args.bookmark` (defaults to `main`). Runs before snapshot
collection, before fetch, before any state mutation. If any repo
errors, sync returns `Err` and the standard cross-repo revert
path doesn't fire (nothing to revert — we hadn't snapshotted
yet).

**`push` (push.rs `stage_preflight`):** added the verify call as
the first non-dry-run step, before `vc-x1 sync --check`. Verifies
both repos (`root` and `claude_path(root)`) on `state.bookmark`.
Plumbed `state` through to `stage_preflight` (was previously
`(root, args)`; now `(root, state, args)` matching other stage
fns). Belt-and-suspenders with sync's own check — `vc-x1 sync
--check` would catch it too, but push's explicit check makes the
contract local to push and fails fast before delegating.

**`finalize`:** unchanged — already calls `verify_tracking`
(promoted to common.rs in 0.38.0-0).

Behavior for healthy repos: zero observable change (silent pass).
For non-tracking remote refs: errors before any mutation, with
the standard `jj bookmark track <b> --remote=<r> -R <repo>`
remediation. All 204 tests pass — fixtures use the
`test_fixture` flow which establishes tracking via push, so the
new sync/push checks see the happy path.

- `src/sync.rs` — verify loop at top of `sync_repos`.
- `src/push.rs` — `stage_preflight` signature gains `state`,
  dispatcher arm updated, two `verify_tracking` calls added
  before `vc-x1 sync --check`.
- `notes/chores-06.md` — promote `### 0.38.0-2` from TBD to
  filled (this) + new `# References` section at the bottom so
  inline `[N]` refs render as clickable links and the URLs are
  copy-pasteable from raw markdown.
- `notes/todo.md` — Done entry + `## In Progress` shrinks to
  one item (release).

### 0.38.0: release

Release commit closing out the multi-step cycle. Recap of what
landed across -0/-1/-2:

- **0.38.0-0** (refactor): promoted finalize's
  `find_non_tracking_remote` + tracking-preflight logic into
  `common::find_non_tracking_remote` + `common::verify_tracking`.
  4 parser tests moved with it. Behavior unchanged.
- **0.38.0-1** (fix + assertions): wired `verify_tracking` into
  setup commands. Real fix in `clone` (added explicit
  `jj bookmark track` after `jj git init --colocate` —
  empirically confirmed colocate doesn't auto-track). Sanity
  assertions in `init` and `test-fixture`.
- **0.38.0-2** (feat): wired `verify_tracking` into preflight
  commands (`sync`, `push`). Both detect non-tracking remote refs
  before any mutation, with the standard
  `jj bookmark track <b> --remote=<r> -R <repo>` remediation.

Final coverage: every repo-modifying command (`init`, `clone`,
`test-fixture`, `sync`, `push`, `finalize`) verifies bookmark
tracking either as a setup post-condition or as a preflight gate.

**Dogfood validation (2026-04-23/24).** Three call-site / cwd
combinations exercised on both this repo and a fresh `tf-1/work`
test fixture:

- workspace root cwd, app side untracked → error with `-R .`
- workspace root cwd, `.claude` side untracked → error with
  `-R .claude`
- `.claude` cwd, `.claude` side untracked → error with `-R .`
  (cwd-relative; paste-and-run from current cwd)

All three produce error messages whose embedded
`jj bookmark track …` command is directly copy-paste-able from
the cwd the user is in. The cwd-relative path resolution falls
out of how each command calls `verify_tracking(repo, …)` with the
repo path it's already using — no special cwd handling needed.
Validates both the tracking design ([63]) and the cwd-independence
claim ([65]).

- `notes/chores-06.md` — this `### 0.38.0` close-out subsection.
- `notes/todo.md` — `## In Progress` cleared; Done bullet for
  `0.38.0` (the cycle marker).

## Push hardening: state + stage sanity (0.39.0)

Implementation cycle for [61] (Push hardening: state + stage
sanity design). Multi-step rollout addressing the silent
false-success failure mode that surfaced in the 0.37.1 dogfood
incident: push parked at finalize-claude after a failure;
out-of-band recovery moved the world forward; subsequent push
resumed at the parked stage, no-op'd, and falsely declared
"completed all stages" while WC still held uncommitted changes.

- **0.39.0-0** — state-sanity preflight on resume (broader
  guard, less invasive).
- **0.39.0-1** — stage-prereq verification + honest completion
  (per-stage "what I expect" guards).
- **0.39.0** — release commit + dogfood evidence.

### 0.39.0-0: state-sanity preflight on resume

Added `verify_state_sanity(root, state)` in push.rs. Runs at the
top of `run_from`, before the initial state save and before any
stage executes. Three checks per [61] design:

1. `state.app_chid` resolves in the app repo (errors if
   abandoned or rewritten).
2. After `bookmark-both` has run (`state.stage ∈ {PushApp,
   FinalizeClaude}`), the bookmark points at a commit whose chid
   matches `state.app_chid` (catches "world moved forward via
   manual recovery" — the 0.37.1 incident).
3. `state.claude_chid` resolves in `.claude` (when set).

A fresh state (no chids yet) is a no-op — nothing to verify.
The check on (2) is gated on stage because between `message`
and `bookmark-both`, `app_chid` is set but the bookmark hasn't
been moved yet — naively comparing would always fail.

On mismatch: error with the exact `vc-x1 push <bookmark>
--restart` remediation. Example shape:

> `state-sanity: app_chid 'xxx' has been abandoned or no longer
> resolves in the app repo. State is stale — run \`vc-x1 push
> main --restart\` to clear.`

All 204 tests pass — fixtures use happy paths (fresh state +
clean fixture) so the new check is a no-op there. Failure paths
get manual dogfood at the 0.39.0 release commit.

Test-strategy note: explicit unit tests for the comparison
logic weren't added because most of the behavior is jj-shelling
(hard to mock without infrastructure). Existing test suite
covers the happy path; failure paths get manual dogfood.

- `src/push.rs` — new `verify_state_sanity` fn; called from
  `run_from` before initial state save.
- `notes/chores-06.md` — new `## Push hardening: state + stage
  sanity (0.39.0)` parent + this `### 0.39.0-0` filled
  sub-section + `### 0.39.0-1` and `### 0.39.0` TBD stubs.
  References block converted from full GitHub URLs to anchor
  links (`#name` for self-refs, `/notes/chores-05.md#name` for
  cross-file [57]) — works in local markdown viewers and on
  GitHub render alike, cleaner in raw markdown view. Plus new
  `## Source-code design ref convention (design)` subsection
  [68] capturing the pattern that triggered this thread (the
  opaque `[N]` ref problem in source code).
- `notes/todo.md` — Done bullet for this step + `## In Progress`
  populated with remaining 0.39.0 steps + new Todo entry for
  the source-code design ref sweep + new `[67]` and `[68]`
  references.

### 0.39.0-1: honest completion via post-completion verification

Added `verify_completion_sanity(root, state)` in push.rs — the
post-loop counterpart to `verify_state_sanity` from -0. Runs
after all stages complete; verifies the world matches the saved
state before declaring "completed all stages".

Three checks:

1. App bookmark's chid matches `state.app_chid`.
2. App WC has no uncommitted changes (commit-app should have
   captured anything that was there). **This is the direct
   counter to the 0.37.1 false-success symptom** — push declared
   "completed all stages" while WC still held uncommitted
   changes from a stale-state resume.
3. `.claude` bookmark's chid matches `state.claude_chid` (when
   set). No `.claude` WC-clean check — `.claude` may legitimately
   have new session writes from the push run itself, which
   finalize-claude (detached) handles.

On failure: warning-only (not Err) because push has already
crossed the remote boundary by completion; rollback isn't sound.
State is cleared either way, but the success message changes:

- Pass → `push: completed all stages (verified, state cleared)`.
- Fail → `push: completed stages but post-completion verification
  failed: <reason>; state cleared anyway (work landed on remote);
  investigate the discrepancy before next push`.

**Scope note — per-stage prereqs deferred to 0.39.0-2.** The
[61] design called for *both* per-stage prereq checks *and*
honest completion. Per-stage prereqs would require changing
every stage's signature (`Result<()>` → `Result<StageOutcome>`)
and threading outcomes through the dispatcher — substantially
more invasive than the post-completion check, and with
state-sanity preflight (-0) already covering the resume case,
the marginal value is lower. Captured as the new `### 0.39.0-2`
TBD subsection below.

**Tests:** 4 new integration tests in `push.rs` exercise
`verify_completion_sanity` directly with manually-constructed
`PushState` against `Fixture` repos. Coverage:

- `completion_sanity_pass` — happy path after a real push.
- `completion_sanity_fail_app_chid_mismatch` — bogus
  `state.app_chid` triggers check 1.
- `completion_sanity_fail_dirty_wc` — uncommitted WC changes
  trigger check 2.
- `completion_sanity_fail_claude_chid_mismatch` — bogus
  `state.claude_chid` triggers check 3.

Total tests: 204 → 208.

**Bug found during test-dev:** initial check 2 used `jj diff
--stat` output-emptiness, but jj always prints `"0 files changed,
0 insertions(+), 0 deletions(-)"` even when clean — false
positive on every push. Switched to `jj_log_empty` template-based
check (re-uses an existing helper in push.rs). The
`completion_sanity_pass` test caught this immediately, which is
exactly why the integration tests were worth writing.

- `src/push.rs` — new `verify_completion_sanity` fn (with
  `jj_log_empty`-based WC-clean check); called from `run_from`
  after the stage loop. Success message qualified with
  "(verified)"; failure surfaces as a warning. 4 new
  integration tests in the `integration_tests` module.
- `notes/chores-06.md` — promote `### 0.39.0-1` from TBD to
  filled (this) + new `### 0.39.0-2` TBD stub for the deferred
  per-stage prereq work + new `## vc-x1 validate-repo command
  (design)` subsection [69] capturing the diagnostic-command
  idea that surfaced during this commit's review.
- `notes/todo.md` — Done bullet for this step + `## In Progress`
  updated (added `0.39.0-2` entry, kept release) + new Todo
  entry for `validate-repo` + new `[69]` reference.

### 0.39.0-2: per-stage prereq verification — SKIPPED

Designed but not implemented. Decided at the 0.39.0 release
commit: the per-stage prereq checks would have changed every
`stage_xxx` signature (`Result<()>` → `Result<StageOutcome>`)
and threaded outcomes through the dispatcher — a substantial
refactor for low marginal value over -0 + -1. The 0.37.1
false-success incident is fully covered by the existing pair
(state-sanity catches stale resume; post-completion catches
end-state mismatch). Per-stage prereqs would be audit polish
("which stages ran vs were already-done") rather than safety
— and jj's own `"Nothing changed"` output for no-op stages
already gives audit signal.

Original design preserved here for reference if requirements
shift later.

### 0.39.0: release

Close-out commit for the Push hardening cycle.

**Cycle recap:**

- **0.39.0-0** (feat): state-sanity preflight on resume.
  Catches stale-state-after-out-of-band-recovery (the 0.37.1
  symptom-source) before any stage runs.
- **0.39.0-1** (feat): post-completion sanity check + 4
  integration tests. Verifies the world matches saved state at
  the end of a successful run — direct counter to the 0.37.1
  false-success symptom (push declared completion while WC
  still held uncommitted changes).
- **0.39.0-2** (skipped): per-stage prereq verification. See
  the subsection above for the rationale.

**Final coverage matrix** for push:

| Concern | Mechanism | Added in |
| --- | --- | --- |
| Bookmark tracking | `verify_tracking` (preflight) | 0.38.0 |
| Stale-state on resume | `verify_state_sanity` (pre-stage) | 0.39.0-0 |
| End-state mismatch | `verify_completion_sanity` (post-stage) | 0.39.0-1 |

The 0.37.1 false-success incident class is now closed at both
ends — caught at resume time (state-sanity) AND at completion
time (post-completion).

**Dogfood (2026-04-24):** the post-completion check has fired
on every successful push since 0.39.0-1 landed — the
`(verified, state cleared)` suffix in the success message is
live evidence. Failure paths are covered by the 4 integration
tests added in -1 rather than manual probing (constructing
stale state would require interrupt + tamper, not worth it
given the test coverage).

- `notes/chores-06.md` — `### 0.39.0-2` updated from TBD to
  SKIPPED with rationale. New `### 0.39.0` close-out
  subsection (this).
- `notes/todo.md` — `## In Progress` cleared. Items #1
  (state-sanity) and #2 (stage-prereq) removed from `## Todo`
  (now covered by the cycle). Done bullet for `0.39.0` added.

Release commit closing out the cycle: recap of -0/-1 + dogfood
evidence (induce stale state → state-sanity fires; induce
stage-prereq violation → stage-prereq fires).

## Source-code design ref convention (design)

When source code references a design captured in `notes/`, the
ref should be useful on its own — three things matter: the
descriptive **section name** (what the design is), the **URL**
to it (where to find it), and **clickable form** (so the reader
navigates without a decoder ring). The opaque `[N]` syntax used
in markdown notes doesn't translate to source — `(per chores-06
[61] design)` is unhelpful without the references table
memorized.

**Pattern (decided):**

```rust
/// One-line gist per the "Section Name" design:
///   https://github.com/winksaville/vc-x1/blob/main/notes/<file>.md#anchor
```

Full URL (`blob/main/...`) is right for source code because:

- Source has no markdown rendering — anchor-only `#name` doesn't
  navigate from a `.rs` file.
- It's clickable in IDEs that recognize URLs.
- Tracking `main` (not a commit hash) means the link auto-shows
  the *current* design, which is what implementation code should
  be aligning to. Pinning to a commit hash is right for
  historical refs ("the design at the time of this incident")
  but not for ongoing implementation refs.

**Stable section names matter.** Once a design subsection lands,
don't rename its `## …` header — links break silently (the URL
still resolves to the file but the anchor goes nowhere). If a
topic evolves substantially, add a new subsection rather than
rewriting the old one's name.

**Sweep targets** (when this design lands as code):

- `src/sync.rs:142` — already correct (landed 0.38.0-2).
- `src/push.rs:4` — name + path, no URL. Upgrade.
- `src/push.rs:121` — path only, no name. Upgrade.
- `src/push.rs:645` — opaque `[61] design`. Upgrade.
- `src/push.rs:1219` — opaque `[61] design`. Upgrade.

**CLAUDE.md codification** (same commit as the source sweep):

1. Source-code design refs use section name + `blob/main/...` URL.
2. Markdown-internal refs use anchor-only (`#anchor`) for
   self-file or `/notes/<file>.md#anchor` for cross-file.
3. Don't rename design subsection headers post-landing
   (link stability).

## `vc-x1 validate-repo` command (design)

Surfaced 2026-04-24 during the 0.39.0-1 review. A new top-level
subcommand that consolidates all the `verify_*` checks we've
built across the codebase (and adds a few more) into one
diagnostic command. Use cases: pre-flight before starting work;
CI hook; "feels off, what's wrong?" debugging.

**Proposed shape:** `vc-x1 validate-repo [--scope=app|other|both]`

**Checks** (composed from existing + new):

- Bookmark tracking — `common::verify_tracking` (0.38.0).
- Push state freshness — if `.vc-x1/push-state.toml` exists,
  run state-sanity logic (currently push-private; promote to
  `common`).
- Ochid trailer integrity — both repos' commit-body ochid
  trailers reference real changeIDs in the counterpart repo.
  No equivalent today; new code in this command.
- No jj conflicts — `jj log -r conflicts() --no-graph` returns
  empty per repo.
- Workspace config sanity — `.vc-config.toml` present,
  parseable, `path` field matches workspace structure.
- Bookmark/remote tracking matrix — richer view of [52].
- Working-copy state — clean / dirty summary per repo (sibling
  to [54]).

**Output shape:**

- One line per check: `✓ <check>` (pass) or `✗ <check>: <reason>`
  (fail).
- Summary line: `validate-repo: N/M checks passed`.
- Exit code: number of failed checks (0 = clean).

**Implementation hooks:**

- `verify_state_sanity` and `verify_completion_sanity` in
  push.rs should be promoted to `common.rs` so `validate-repo`
  can call them. Natural refactoring during implementation.
- `verify_tracking` already in `common.rs` (0.38.0).

**Relationship to other todos:**

- `vc-x1 status` (existing todo): *operational* — "what is the
  current state" (`jj st` across repos).
- `vc-x1 validate-repo` (this design): *correctness* — "does
  the current state make sense / pass health checks".
- Distinct commands, distinct concerns. Both worth having.

## Generalize --scope across commands (0.40.0)

Foundation for item #1 on todo ([60]): commands should support
single-repo, dual-repo, and plain-old-repo (POR) workspaces
without hard-coding the `.claude` dual-repo assumption.

**Scope default resolution** (no `--scope` given):

- No `.vc-config.toml` → `--scope=app`.
- `.vc-config.toml` present but `other-repo` missing or empty
  → `--scope=app`.
- `.vc-config.toml` with a non-empty `other-repo` → `--scope=both`.

**Ambiguity rule.** Any ambiguous combination of scope-related
flags on a single invocation (e.g. `-R` with `--scope`) is a
fatal error — callers must pick one way to express repo
selection per call.

**Cycle steps.**

- **0.40.0-0** — this plan. Notes + version bump only.
- **0.40.0-1** — `vc-x1 init --scope=app|other|both`.
  - `app` → single-repo workspace (`.vc-config.toml` with
    `path = "/"` and no `other-repo`; no `.claude` subdir; one
    GitHub repo; no `ochid:` trailer on the initial commit).
  - `both` (default) → current dual-repo behavior.
  - `other` → error; meaningless at init time.
  - Every other subcommand errors loudly if `--scope` is passed.
- Subsequent `-N` steps wire `--scope` into sync, push,
  finalize, and the read-only commands. Specifics decided when
  each lands.

**Helper placement is deliberately open** — `common.rs`,
`init.rs`, or a dedicated `scope.rs` / `scope_helpers.rs` are
all fine for step `-1`; we can refactor later once more call
sites exist. The plan is subject to change as we learn; `-0`
records the current best guess, not a contract.

# References

[57]: /notes/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[60]: #generalize---scope-to-all-commands-design
[61]: #push-hardening-state--stage-sanity-design
[63]: #non-tracking-remote-bookmark-detection-design
[64]: #notes-restructure-chores-06--trim-long-todo-entries-0377
[65]: #scope-design-refinements-0378
[66]: #bookmark-tracking-verification-0380
[67]: #push-hardening-state--stage-sanity-0390
[68]: #source-code-design-ref-convention-design
[69]: #vc-x1-validate-repo-command-design
