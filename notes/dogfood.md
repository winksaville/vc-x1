# Dogfood log

Dated entries on where the pinned instructions chafed, failed, or got amended; the evidence
base for promoting local findings back to the template repository (vc-x1-template). Newest
first. Lived in custom.md's `## Dogfood log` until 2026-08-03, when the log moved here:
it is a record, and custom.md converges toward the template's skeleton.

- 2026-08-18: the `(done)` flip ran ahead of the review it claims (wink, at the jj-spawns
  port rung)
  - the per-commit checklist's step 3 flips `(current)` to `(done)` before validation and the
    work review, so the record says "done" while the user may still reject or reshape the
    work. Wink's tweak, adopted this session: the rung stays `(current)` through validation
    and the work-review stop, and flips to `(done)` when the user accepts the work and asks
    for the description, the moment "done" becomes true
  - a candidate for cycle-checklists.md's per-commit steps 3/6 (and the protocol's per-commit
    flow) at a convention cycle: the flip moves from "before validation" to "on work-review
    acceptance, before the description"

- 2026-08-18: a premature stable-name install let one version string mean two behaviors
  - the 0.78.8 close-out's validation ran `cargo install` (producing plain `vc-x1 0.78.8`)
    while the cycle could still change, and a test rung was then inserted, so the installed
    binary and the eventual landed 0.78.8 would have differed under one version string. Wink
    caught it from `vc-x1 -V`; resolved by folding the test into the closing commit, so
    exactly one commit carries bare 0.78.8 and the final validation's install is the real
    promotion
  - the rule this teaches, a candidate for versioning.md's Dev artifact name and
    custom.md's validation notes: the stable-name install is the cycle's *last* act, run
    when nothing can enter the cycle anymore (final close-out validation or post-landing).
    Every earlier install in the flow produces the dev name only, which is what versioning.md's
    "never by the per-commit flow's install" already meant and the close-out flow contradicted

- 2026-08-07: the merged agent-file set adopted; two rules born in its review
  - the set is iiac-perf's `agent-files-model` proposal merged onto this repo's file layout
    with the review's corrections (two behavioral regressions fixed, the unsynced baseline
    content preserved). The verdicts and the acceptance-check results live in the cycle's
    chores section
  - two rules were written during the review and applied set-wide: a semicolon joins equals
    (prose.md `Semicolons`, sharpened from wink's "item; detail" objection), and a pinned
    file names no project (prose.md), which retroactively cleaned six member-history
    references out of the pinned set
  - watch: `CLAUDE.md` is now one line, so `custom.md` stops being auto-loaded and hard
    rule 0 is load-bearing. The first sessions after landing are the sample for whether a
    session reads it unprompted
  - friction found: notes.md says the as-built ladder is the first content under a chores
    header, while the six-item shape lists the ladder fourth of six. This cycle put the
    ladder first and the other five items after it. If that keeps being the answer, the
    pinned text should say it

- 2026-08-07: the 2026-08-03 sandbox publish failure was ssh itself, not transfer size
  - that entry guessed the sandbox's proxying killed long SSH transfers, since the large bot
    pack died mid-transfer where the small work pack succeeded; the size correlation was
    coincidence, and ssh does not work from a sandboxed session at all
  - the sandbox denies reading `~/.ssh` except the commit-signing key and `known_hosts`, so no
    auth key can be offered, and we think a host allowlist cannot admit port 22 either, since
    ssh carries no SNI or Host header for a filter to match on
  - `vc-x1 push` inherits all of it because jj-lib performs transfers by spawning the real
    `git` binary (`GitSettings::to_subprocess_options`), and that child runs under the same
    sandbox as the session
  - fixed by repointing both remotes at https (wink, 2026-08-07); git's
    `store --file ~/.gitcreds` helper then serves both repos from a file the sandbox permits
    reading, and no vc-x1-side credential handling is involved at any point
  - finding for AGENTS.md: the dual-repo section places the bot repo at a symlink from
    `~/.claude/projects/<path-to-project-root>/.claude`, which has one path component too many
    and the direction reversed; `<project>/.claude` is the real directory and the `projects`
    entry is the symlink pointing into it
  - finding for the template: an instruction set expecting a sandboxed agent should say to
    clone over https, since ssh is unusable from inside one and the failure surfaces late, at
    the close-out push, where it is most expensive

- 2026-08-03: the punctuation ban's enumeration invited a subset audit, again
  - prose.md bans four named characters; the 0.78.2 source sweep found four more untypeable
    species doing the same jobs: `⇒` (13 sites), box-drawing `─` in CLI output (56), `≥`,
    and a U+2212 minus
  - the 0.77.2 close-out had already recorded the subset-audit failure mode (an em-dash-only
    count), and the Todo entry's per-character count repeated it at a wider subset
  - template finding for prose.md: state the rule as "ASCII-only in durable text and source,
    with the four common offenders named as examples", so a sweep's acceptance check is
    `grep -P '[^\x00-\x7F]'` rather than four greps; staged for the iiac-perf review

- 2026-08-03: the generic-custom.md test's first bites, at the 0.78.2 opening
  - validation commands unnamed: this session used the 0.78.0-era cargo commands from
    conversation memory; a fresh session would have to rediscover them (the checklist's
    "commands are in custom.md" now points at placeholders)
  - the one-home override is gone, so this cycle runs the pinned per-commit chores build-up
    (chores-16 opened at Preparation, built rung by rung)
  - the close-out shape default (trapezoid) is gone; shape falls back to
    chosen-at-push-time per the close-out checklist
  - the dogfood-binary rule (invoke `vc-x1-dev`, never `vc-x1`) and the mailbox parameters
    (member name, template path) are unnamed; session memory covered both this time

- 2026-08-03: a sandboxed session could not publish the bot repo; the user's shell could
  - three bot-repo publish attempts at the 0.78.1 push died mid-transfer ("the remote end
    hung up unexpectedly") while the small work-repo pack pushed fine; wink's direct shell
    succeeded first try
  - we think the session sandbox's network proxying kills long SSH transfers; the bot pack
    carried this long session's multi-megabyte transcript
  - recovery was the documented one: `squash-push` reruns safely, and the chid stayed
    stable across the re-squashes, so the ochid linkage was never at risk
  - finding: a large bot-repo publish may need the user's own shell; the work-repo push
    succeeding while the bot-repo push fails is the signature

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
