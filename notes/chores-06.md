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

### 0.38.0-1: wire into setup commands (TBD)

Wire `common::verify_tracking` into `init`, `clone`, and
`test-fixture` as post-condition assertions:

- `init`: Step 10 already calls `jj bookmark track`; add the
  verify call as a sanity assertion to catch regressions.
- `clone`: open question (per [63]) — does `jj git init
  --colocate` after `git clone` auto-establish tracking? Verify
  empirically; if not, add an explicit `jj bookmark track` step
  after the colocate, then assert.
- `test-fixture`: Step 7's first `jj git push --bookmark main`
  establishes tracking as a side effect; verify call as a
  post-condition is a no-op in the happy path but catches
  regressions if push semantics shift.

### 0.38.0-2: wire into preflight commands (TBD)

Wire `common::verify_tracking` into `sync` and `push` preflight:

- `sync`: detect + error in preflight before any fetch/rebase
  attempt.
- `push`: detect + error in preflight before any mutation
  (commit, bookmark move, push). The existing `vc-x1 sync
  --check` step that push runs first will already cover sync's
  side; this adds the explicit per-bookmark check that
  push-app needs before crossing the remote boundary.

`finalize` already calls `verify_tracking` (now via the shared
helper in 0.38.0-0); no change needed in -2.
