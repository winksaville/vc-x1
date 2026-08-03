# Dogfood log

Dated entries on where the pinned instructions chafed, failed, or got amended; the evidence
base for promoting local findings back to the template repository (vc-x1-template). Newest
first. Lived in custom.md's `## Dogfood log` until 2026-08-03, when the log moved here:
it is a record, and custom.md converges toward the template's skeleton.

- 2026-08-03: "treated as permanent (merge-only onto `main`, never rebased)" refined after
  wink deleted the fully merged `refactor-vc-x1@origin`
  - permanence had no value post-merge: every rung is reachable from `main`, the records cite
    commit ids, and the ochid trailers live in the commit bodies, not the ref
  - "never rebased" also overshot: ochid linkage rides chids, which survive rebase
    ([jj.md](../agent-data/jj.md#cross-repo-linking-ochid-trailers)), and the 0.78.0-0 retro
    re-describe + force-push was already an accepted coordinated rewrite
  - the rule as actually practiced: merge-only by default, no unilateral rewrites of published
    history, coordination as the escape hatch
  - replaced by the "Long-lived bookmark discipline" section in the pinned jj.md; template
    finding: bookmark guidance should distinguish in-flight history (protect it) from merged
    history (the bookmark is redundant)
  - promoted template-side 2026-08-03: landed in the proposed baseline
    `agents-protocol/AGENTS-vc-x1-f5-20260803-snapshot/` (jj.md section), alongside the
    pin-set dedup (cycle-protocol.md + versioning.md into agent-data/, jj-tips.md rehomed to
    the template, draft-reviews.md salvaged into the protocol); staged for iiac-perf's review
    via its mailbox
  - adopted here 2026-08-03 at 0.78.1 (dogfood-first, ahead of the review, the 0730
    precedent); the same commit resets custom.md to the bare skeleton as the
    generic-custom.md test (safe once this log moved here and the bookmark discipline was
    pinned: nothing uncommitted remained in custom.md's sections)

- 2026-08-02: the 0.78.0-9 push ran on the stable 0.71.0 binary, not the dev build
  - no instruction file named which binary to invoke, so the bot reached for `vc-x1`; caught
    by wink at the close-out's `--version` check, after the push
  - no damage: the commits, bookmarks, and ochid trailers verified correct on both repos, but
    the push that should have first dogfooded the in-process jj-lib mutations ran the old
    spawn-based code instead
  - fixed by the "Invoke `vc-x1-dev`, never `vc-x1`" convention in custom.md; template
    finding: a dev-artifact repo needs its custom.md to name the dogfood binary explicitly

- 2026-08-02: synced to `AGENTS-vc-x1-f5-20260802-snapshot/`, the tier-1 graduation this
  session authored template-side
  - the 0802 snapshot is the 0730 one plus the graduation of the conventions the two repos
    dogfooded: write-to-full-width, cycle bookend titles, the checklist's close-the-records
    step, the mailbox check at acquaint
  - the 0730 amendments landed en route, this repo having run on the pre-amendment set since
    0.78.0-1: rule 0, hard-rules-first, generic pin lines, chores as-built ladder, chores
    `## Table of Contents`
  - cut as a new snapshot directory, not an in-place amendment: the template repo carries no
    commits, so amending 0730 in place would have destroyed the adoption record (wink's call)
  - notes/cycle-protocol.md amended to match the ladder form (Chores sections, Commits
    backfill)
  - the one-home override narrowed, its backfill clause having become the universal rule
  - bookends retro-applied to `0.78.0-0` by a coordinated re-describe + force-push of the
    unmerged ladder
  - two 0730 prose.md findings, both fixed in the 0802 snapshot: the "Conventional-commit
    shape" chores bullet still described the retired `Commits:` line, and the "Banned:"
    opening contradicted the transcription exception (now "the prohibition is on authoring")
  - tier 2 staged for iiac-perf's read (mailbox message): one-home, cycle-protocol.md into
    the byte-identical set, every-commit-belongs-to-a-cycle, scope-based version advancement
- 2026-07-30: adopted at 0.78.0-1
  - semantics-preserving restructure of AGENTS.md + cycle-protocol satellites
  - proposal snapshot frozen in the template as `AGENTS-vc-x1-f5-20260730.md`
- 2026-07-30: conventions clarified by the user at the 0.78.0-1 review
  - amends prose.md, code.md, cycle-protocol.md
  - title <=72 (was 50), with optional `(scope)`
  - docs wrap <=100, commit title/body at <=72
  - commit-body bullets are sentence fragments
  - the version-first bullet is spelled `vX.Y.Z-xxxx`
- 2026-07-30: version protocol defined (user + bot)
  - amends versioning.md (new Grammar and storage section) and cycle-protocol.md
  - one prose spelling `X.Y.Z-<dot-nested suffix>`, exactly one dash ever
  - `v`: a display-only prefix, never stored
  - per-medium storage: SemVer verbatim, PEP 440 remaps the `-` to `+`
  - stored versions identify, never order
  - `+` reserved for the remap
  - driven by a sibling repo's Python linter incident and a packaging-26.2 parser test
