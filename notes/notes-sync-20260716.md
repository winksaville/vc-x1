# Notes sync 2026-07-16 — change manifest (vc-x1 draft)

Goal: iiac-perf, vc-x1, and vc-template-x1 carry an identical
shared protocol set — `AGENTS.md` (root), `notes/cycle-protocol.md`,
`notes/versioning.md`, `notes/jj-tips.md` — while project-state
and project-local notes differ. vc-template-x1 is frozen for the
round; vc-x1 is the working draft. This file lists what changed
here and why, for the template-side review. Where a change is a
correction, "stale" means the template text described vc-x1
behavior that 0.69.0 already replaced.

## Shared-set status

- `notes/versioning.md` — copied from template verbatim;
  byte-identical.
- `notes/jj-tips.md` — copied from template verbatim;
  byte-identical. Replaces vc-x1's `jj-revsets.md` (deleted;
  jj-tips is a superset — all revset content is under
  `## Revsets`). Live inbound refs updated; historical chores
  mentions left as-is.
- `notes/cycle-protocol.md` — copied from template, then six
  corrections applied here (below). Intentionally diverges;
  this text is the candidate for all three projects.
- `AGENTS.md` — template copy plus two corrections (below).
  Intentionally diverges; candidate for all three.
- `notes/README.md` — NOT synced; shared-vs-local is an open
  decision (vc-x1's is project-flavored).

## cycle-protocol.md corrections

1. **Section "After push or finalize" → "After push or
   squash-push"** (full rewrite).
   - Why: the `vc-x1 finalize --squash --push --delay 10
     --detach` flow is retired — `finalize` was renamed
     `squash-push` at 0.69.0-2 (no alias), and the detached
     delayed flow was the root cause of the 0.68.1 silent
     session-push loss; 0.69.0 made the bot-repo publish an
     inline in-process push stage.
   - The new text is vc-x1's 0.69.0 version
     (Scope/Why/Silence/Flush bullets); the template's
     empty-turn-token detail was merged into the Silence
     bullet rather than lost.
2. **Recovery section** (rewrite).
   - Why: same retirement — recovery is now
     `vc-x1 squash-push -R .claude` (in-process; a failure is
     a visible non-zero exit, no log file), and the stage
     names are `push-work` / `squash-push-bot` (0.69.0-4).
     `push-app` / `finalize-claude` are stale twice over —
     old names, and "app" contradicts the work-repo
     terminology standard.
   - Dropped vc-x1's "pre-0.69.0-4 state file" transition
     note: project history, not shared protocol.
3. **"vc-x1 push wrapper": planned-improvements list
   replaced** with a pointer ("tracked in the project's
   `notes/todo.md` — this protocol describes only the stable
   mechanism") plus one stable-mechanism line (`<bookmark>`
   is work-repo-only; bot repo pinned to `main`).
   - Why: the list was project state and stale — "per-repo
     bookmark names (planned)" landed at 0.68.0; "`## Todo`
     entry P1" is a positional todo reference, which the
     conventions forbid (name by title). Roadmap belongs in
     each project's todo.md, not a verbatim-shared file.
4. **Policy section**: "hard stop after push/finalize" →
   "push/squash-push".
5. **".claude cadence"**: "the finalize commit" → "the
   squash-push fold".
6. **Recovery, late-tweak bullet**: "after `push-work`
   succeeded" → "after the work-repo push succeeded".
   - Why: readability — that bullet means "the work repo is
     already published"; the formal stage token adds no
     precision there. The first Recovery bullet keeps its
     backticked `push-work` / `squash-push-bot` because it
     references the actual `--from`/`--status` tokens.

## AGENTS.md corrections

1. **Preflight paragraph** ("`vc-x1 push` behaviors")
   rewritten: preflight checks repo state only — bookmark
   tracking, the bot-published invariant
   (`main == main@origin`), `sync --check` — no build/tests.
   - Why: stale — the hardcoded cargo fmt/clippy/test
     preflight was removed at 0.69.0-3 (vc-x1 assumes nothing
     about repo contents beyond `.jj` + `.vc-config.toml`).
     Verified against `src/push.rs` `stage_preflight`. The
     medium's validation belongs to the per-commit flow, run
     before invoking push.
2. **"Hard stop after push/finalize" → "push/squash-push"**,
   and its link updated to the renamed protocol anchor
   `#after-push-or-squash-push-stop-and-wait`.

## Open items (deliberately not in this round)

- `Cargo.toml` `[lints.clippy]` (`unwrap_used` /
  `expect_used` = warn): a code change needing its own
  annotate-sweep cycle — the per-commit flow runs
  `clippy -D warnings`, so enabling the lints fails until
  every existing site carries its `#[allow]`.
- `notes/README.md`: decide shared skeleton vs per-project.
- `notes/todo.md` → `./TODO.md` move: captured as vc-x1
  Todo #1; a three-project change (touches the shared
  AGENTS.md "File reads" section), sequenced after this
  round lands.
- Proposal: each shared file carries a short "shared
  verbatim across sibling projects" declaration (as
  versioning.md already does) and a future `validate-repo`
  check diffs the shared set against a sibling.
