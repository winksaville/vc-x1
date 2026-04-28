# Draft reviews

A lightweight workflow convention to make in-progress work
durable and reviewable without committing prematurely. Adopted
2026-04-28 after an accidental ESC interrupted a queued file
write and surfaced the question.

## What "draft saved for review" means

After authoring or editing a unit of work, the bot writes all
relevant files to disk and pauses with the following prompt:

> Draft saved for review: approve (y/n), edit, or suggest
> changes.

"Draft saved" = files are on disk in their proposed state.
Nothing has been jj-committed, jj-squashed, or pushed.

The user can then:

- **Approve (y)** — bot proceeds (typically: pre-commit
  checklist, then propose commit message for the second
  approval gate).
- **Approve (n) or silence** — bot waits.
- **Edit** — user opens the files in their editor and makes
  changes directly. Bot resumes only when told.
- **Suggest changes** — user describes desired changes; bot
  re-edits the draft and pauses again with the same prompt.

## Why

- **ESC-resilience.** Each `Write` lands atomically on disk.
  An interruption between writes loses only the in-flight
  one, not previously-saved files.
- **Full-fidelity review.** Markdown renders properly, code
  type-checks, tests can run — anything you'd do on the real
  files works on the draft.
- **Trivial revert.** `jj restore` undoes any draft change
  that wasn't liked. No commit/abandon dance needed during
  review.
- **No premature jj state.** Nothing to squash, no commit_id
  churn, no force-push concerns. The review is purely a
  filesystem-level activity until the user approves.

## Mechanics

- **No preflight during review iteration.** `cargo fmt`,
  `cargo clippy`, `cargo test`, etc. are *not* run between
  draft saves. fmt would mutate files in ways that interact
  badly with mid-review edits, and the noise distracts from
  the review itself. Preflight runs once, at final approval,
  before the actual jj commit.
- **Multiple draft saves are normal.** Each round of
  suggested changes ends in a fresh "Draft saved for review"
  prompt. The user's "approve (y)" is the explicit transition
  out of draft mode.
- **Cadence: one draft review per file (or small cluster).**
  Batching many files into a single review defeats the
  workflow's intent — the whole defense-in-depth value comes
  from frequent checkpoints where the user can intervene
  before lost-work surface area grows. A `Cargo.toml` +
  `Cargo.lock` version-bump pair counts as one cluster; a
  notes file + its `todo.md` reference update can also count
  as one. Independent files each get their own prompt.
- **Summary still required.** The pause-for-review message
  should include a short summary of what's in the draft
  (file list with one-line gists, plus any noteworthy
  decisions) so the user can decide quickly whether to
  inspect closely or approve.
- **Scope-change escape hatch.** If a review surfaces that
  the work needs substantial rework (not just edits), the
  right move is to discard the draft (`jj restore`) and
  re-author rather than iteratively patching. Squash is for
  small adjustments.

## Relationship to the existing two-gate flow

CLAUDE.md's `Commit-Push-Finalize Flow` already specifies two
explicit approval gates: review (after work) and message
(before push). Draft reviews refine the *first* gate — they
make explicit that the "review" stage is iterative and lives
on disk, not in jj. The second gate (commit message) is
unaffected and still happens at `vc-x1 push` time.

Codification in CLAUDE.md is deferred until the convention
has been used for a cycle or two and any wrinkles are
ironed out.
