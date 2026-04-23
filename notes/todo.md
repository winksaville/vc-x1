# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress


## Todo

A markdown list of task to do in the near feature

 - sync: surface working-copy state in the up-to-date summary (per-repo
   pending-files count or compact stat). Wording-only fix shipped in
   0.37.1; this is the design+impl of the actual signal. [54]
 - Add `status` (alias `st`) subcommand: `jj st` across both repos in
   one shot. Natural home for the working-copy signal called out in [54].
 - Mirror `--check` / `--no-check` onto `vc-x1 push` (forwards through to
   the preflight `vc-x1 sync` invocation). 0.37.1 hard-codes `--check`;
   `--no-check` would be the user-opt-in to "auto-rebase under the gates,
   I trust the sync state". Default stays `--check`.
 - `vc-x1 push`: `--message-file PATH` flag. Reads a git-style commit
   message file — first line is title, blank line, rest is body.
   Equivalent to today's `--title "$(cat t)" --body "$(cat b)"` but
   one flag, one file, no shell escaping gymnastics. Motivation
   (2026-04-22): when scripting a push with a long multi-line body,
   the `--title "$T" --body "$B"` pattern makes the actual `vc-x1
   push …` invocation visually disappear under the variable
   assignments in terminal output — hard to scan. A file-based
   message keeps the invocation short. No short form (`-m`, `-F`,
   etc.) — infrequent enough that the long name is fine. Default `both` matches
   current behavior. `--scope=app` or `--scope=claude` runs push against
   one repo only — for docs-only fixes on the app side, or a `.claude`
   finalize that doesn't need to bundle app changes. Independent of
   the squash work below; simpler and self-contained. **Warn on
   scope/WC mismatch**: if `--scope=app` is chosen and `.claude` has
   pending changes (or vice versa), emit a non-fatal warning
   naming the pending files — catches the "I meant --scope=both"
   case before the user commits.
 - `vc-x1 push`: `--squash` flag. Instead of creating a new commit,
   squashes WC into the current repo's `@-` via `--ignore-immutable`
   and force-pushes. Message stage pre-fills `$EDITOR` with the
   existing description so typical use is edit-in-place; `--title`
   / `--body` still override. Motivation: the manual squash +
   describe + force-push sequence (CLAUDE.md "Late changes after
   push") recurred three times during the 0.37.x dogfood — it's
   not a rare edge case, it's a workflow, and each manual run is
   a chance to miss a step or leave stale state. Once `--squash`
   exists, the "Late changes after push" section in CLAUDE.md
   retires; everything goes through the same two approval gates.

   **Safety requirements** (implement as part of the squash work):
   - Needs a `--force-with-lease`-equivalent. Without it, squash
     is a silent history eraser if someone else pushed between
     our last fetch and our force-push.
   - The review gate should surface "will rewrite commit X → Y
     and force-push to remote" so the user's approval is about
     the rewrite, not just the diff.
   - Stage-prereq checks (separate todo entry above): squash needs
     to verify `@-` is actually the commit we're squashing, not
     something stale from a prior aborted run. Ties into the
     state-sanity preflight todo.

   **Recommended ordering** (squash builds on earlier todos):
   1. State-sanity preflight on resume (earlier todo).
   2. Stage-prereq verification + honest completion (earlier todo).
   3. `--scope=app|claude|both` (simpler, independent).
   4. `--squash` (the big one; benefits from 1–3 being solid).
 - "Oh shit" revert — post-success undo via `.vc-x1-ops/`. Just an
   idea at this stage. Sketch: every repo-mutating command drops an
   anchor (timestamp, command+args, per-repo pre-op-id, per-repo
   pre-push remote ref snapshot) into a workspace-root `.vc-x1-ops/`
   directory. `vc-x1 undo` restores both repos to the anchored op
   via `jj op restore` and force-pushes remotes back to the anchored
   ref. Piggybacks on jj's native op-log retention (~30d default) for
   local restore; snapshots remote refs separately since jj's op log
   is local-only. Safety: TTL or generation cap on anchors, same
   `--force-with-lease`-equivalent as `--squash`, explicit confirm
   (destructive). Scope-aware (`vc-x1 undo --scope=app`). Generalizes
   beyond push — sync / finalize / init / fix-desc could all drop
   anchors. `.vc-x1-ops/` needs the same init + fixture auto-write +
   `.gitignore` discipline as `.vc-x1/`. Motivation: the common
   operator-error failure mode (wrong cwd, wrong scope, wrong
   thing) currently has no built-in recovery — just the error-prone
   manual `jj op restore` + `jj git push --allow-backwards` dance.
 - bm-track "silent when clean" refinement. Always probe on entry
   and exit, but only *print* when the state isn't fully tracked or
   the exit state differs from entry. **When we do print on exit,
   include the entry state in the output** so the transition is
   explicit rather than inferred — "entry was ok (not printed), exit
   failed" is inference; "entry=tracked, exit=NOT_TRACKED" in one
   printed line is evidence. Record entry state in a local variable
   at the enter call site and pass into the exit call. Output shape:
   `bm-track vc-x1 <cmd>: enter=<state> → exit=<state>` — one
   line only when there's something to say. Preserves detection
   value, removes the steady-state two-lines-per-command noise, and
   keeps provenance unambiguous. Deferred in 0.37.4 until
   signal-confidence is established through more varied dogfood —
   changing output behavior before then would muddy the "is the
   probe itself reliable?" question.
 - `vc-x1 push`: state-sanity preflight on resume. Before any stage runs,
   verify saved state matches reality: `state.app_chid` still exists
   (not abandoned/rewritten)?  `main` bookmark at `state.app_chid`'s
   commit? `state.claude_chid` consistent with `.claude` working copy?
   On mismatch, refuse with a loud "state is stale — run `vc-x1 push
   --restart`" error. Surfaced 2026-04-22: 0.37.1 push errored at
   finalize-claude; out-of-band recovery (manual finalize + force-push
   of a squash) moved the world forward; 0.37.2 push then resumed at
   the parked finalize-claude, no-op'd, and falsely declared "completed
   all stages" while working copies still held uncommitted changes.
 - `vc-x1 push`: stage-prereq verification + honest completion. Each
   stage declares what it expects (working-copy dirty for commit-app;
   bookmark at specific commit for bookmark-both; etc.); dispatcher
   checks before running. "Completed all stages" should only print
   when stages genuinely ran or were verified-already-done, not when
   they were skipped without verification. Same dogfood surfaced this.
 - `vc-x1 test-fixture` should refuse `--path` values that resolve inside
   the current workspace root (error or warn). Dogfood surfaced this:
   `--path ./tf-1` inside the repo let jj snapshot the fixture's bare-git
   remotes into the commit — a 56-file noise blob that got force-push'd
   off the remote later. Tool-level prevention, not `.gitignore`.
 - Non-tracking-remote bookmark detection across every repo-modifying
   command. Diagnosed 2026-04-22 dogfood: jj's tracking state is
   **per-workspace** (local `.jj` store), not shared via git refs.
   Sync across machines transfers refs but not the tracking flag —
   so a fresh workspace fetched-into never auto-tracks. The failure
   surfaces only when `jj git push` is attempted, which is too late
   (push-app already succeeded in our case).

   **Policy (decided):** error loudly, with the exact
   `jj bookmark track <b> --remote=<r> -R <repo>` remediation
   command. No self-heal — keeps the fix explicit and visible.

   **Scope (every command that creates or mutates repo state):**
   - `vc-x1 init`: already correct (Step 10 calls `jj bookmark track`).
     Add the check as a post-condition sanity assertion anyway.
   - `vc-x1 clone`: does `git clone` then `jj git init --colocate`.
     Whether that combination auto-establishes tracking in jj's
     workspace store is unclear; probably needs an explicit
     `jj bookmark track` for each cloned bookmark after the init.
     Verify and fix.
   - `vc-x1 test-fixture`: Step 7's `jj git push --bookmark main`
     establishes tracking as a side effect of the first push. Works
     correctly — confirmed via `jj bookmark list --tracked` on a
     fresh fixture. No change needed; the post-condition sanity
     check would naturally cover it.
   - `vc-x1 sync` preflight: detect + error.
   - `vc-x1 push` preflight: detect + error (before any mutation).
   - `vc-x1 finalize`: detect + error before the squash, so a
     failed push doesn't leave a half-finalized state.

   Shared helper: `common::verify_tracking(repo, bookmark, remote)
   -> Result<(), Err>` or similar. Probably use
   `jj bookmark list --tracked -T <template>` under the hood
   rather than parsing human-readable output.
 - Allow `vc-x1 push` to work on code or bot repo together or independantly and
   specifically we should be able to "squash" in th code repo just as we do in
   the bot repo.
 - Richer bookmark enumeration: per-bookmark remote presence + tracking status [52]
 - Per-line/per-thread runtime log points (future, maybe) [36]
 - Add Windows symlink support via `std::os::windows::fs::symlink_dir` [37]
 - Add "::" revision syntax for jj compatibility
 - Add -p, --parents, -c, --children so parent and child counts can be asymmetric
 - Add integration tests in tests/ for subcommands using temp jj repos (tempfile crate)
 - Fix .claude repo history: dev0 through dev2 sessions squashed into wrong commit [4],[5]
 - Add `vc-x1 setup` subcommand: completions install, .claude repo init, symlink setup [27]
 - Add dynamic revision completion via `ArgValueCompleter` (jj doesn't complete revsets either) [28],[29]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

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

# References

[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[27]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[28]: /notes/chores-02.md#testing-results
[29]: /notes/chores-02.md#shell-completion-discovery
[36]: /notes/chores-03.md#per-lineper-thread-runtime-log-points-future
[37]: /notes/chores-03.md#windows-symlink-support
[48]: /notes/chores-05.md#add-push-subcommand-0370
[49]: /notes/chores-05.md#claudemd-refresh--memory-migration-0361
[50]: /notes/chores-05.md#sync-improvements--single-repo-support--quieter-dry-run-0363
[51]: /notes/chores-05.md#test-harness-refactor-0362
[52]: /notes/chores-05.md#open-questions--tbd
[53]: /notes/chores-05.md#first-dogfood-polish-for-push-0371
[55]: /notes/chores-05.md#temporary-bookmark-tracking-diagnostic-probe-0372
[56]: /notes/chores-05.md#fix-bm-track-bugs--rename--promote-to-permanent-0373
[57]: /notes/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[58]: /notes/chores-05.md#capture---message-file-design-for-push-0375
[54]: /notes/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[59]: /notes/chores-05.md#claudemd-polish-0376
