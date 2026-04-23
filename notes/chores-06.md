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
