# Chores-18

Continuation of `chores-17.md` (closed after `0.80.3`, the fix-dev-artifacts cycle, at just
over 1200 lines). This file covers the cycles from `0.80.4-0` onward, opening with the
reshape-at-land cycle.

Reference numbering is file-local, per
[`agent-data/notes.md`](../../agent-data/notes.md#reference-numbering), and chores-18 starts
at `[1]`.

## Table of Contents

- [docs: reshape at land](#docs-reshape-at-land)

## docs: reshape at land

### Problem

The close-out builds the trapezoid, restores the plain name, and moves the published bookmark
sideways before the review that landing exists for, so a review finding costs sideways pushes of
published history. Three nearby gaps ride along: the at-rest contract does not name a
`jj git push` landing as a push, the version scheme's shape-versus-contents test is undecidable
in practice, and the close-out shape choice lacks the gitk views that show the net change at
full context.

### Solution

The trapezoid recipe's steps 2-5 moved under Land, so Close-out step 5 only chooses and records
the shape while Land is the five-step permanence sequence. Landing pushes came under the at-rest
contract, Advancing X.Y.Z became patch by default, and jj.md gained a section on reading a change
in gitk at full context, linked from README.md. Two edits joined on the way: custom.md emptied
into the agent-files, and Terminology gained Land, Trapezoid and Artifact while rationale.md's
Terminology lost the entries that defined rather than argued.

### Acceptance check

Run at the close-out. Four of the five report here, the fifth at Land:

- Pass: Close-out step 5 chooses and records only, and Land runs the restore and the reshape.
  The recipe's steps 2-5 read under Land in jj.md.
- Pass: jj.md's Land names hard rules 2 and 3 for its pushes, closing words before the final
  invocation.
- Pass: versioning.md advances patch by default, and the shape-versus-contents test is gone.
- Pass: jj.md explains gitk's three views at full context and README.md points at the section.
- At Land: this cycle's own landing runs the new order, linear bookmark until the go, then
  restore, reshape, fast-forward. The bookmark is linear as pushed and `main` sits at the hack
  commit, so the check is set up and reports when the sequence runs.

### Ladder

- [[8]] 0.80.4-0 [docs: reshape at land opening][1]
- [[9]] 0.80.4-1 [docs: advance patch by default][2]
- [[10]] 0.80.4-2 [docs: empty custom.md into the agent-files][3]
- [[11]] 0.80.4-3 [docs: land under the at-rest contract][4]
- [[12]] 0.80.4-4 [docs: move the reshape and restore under Land][5]
- [[13]] 0.80.4-5 [docs: read a change in gitk at full context][6]
- [[14]] 0.80.4 [docs: reshape at land closing][7]

### Deliberation

- born at the fix-dev-artifacts landing: the trapezoid built at the closing was amended after
  review, two sideways pushes where a linear bookmark needed one. The hack commit below this
  opening filed the discovery, and its Todo entry became this block
- the hack commit stays at the ladder's base and is not re-described (hard rule 4): its title
  is an honest record of what it is
- a single-step at-rest fix grew into this bundle: all four edits sit on one seam, Land as the
  permanence boundary
- a patch bump to 0.80.4, dogfooding the advance-patch-by-default rung in its own cycle
- the dev name is kept until Land, answering the Todo entry's open bullet by doing it
- the bookmark is `reshape-at-land`, wink's name from the hack, not the title's slug, a scoped
  exception as at fix-dev-artifacts
- no dogfood entry: it would be born resolved, and the chores record is the evidence
- the opening's subsection names the backfilled and swept work by version, wink's call: missed
  backfills keep happening, and the version is the greppable handle a checker uses, a scoped
  exception to prose.md's versions rule
- `main` was advanced to the at-rest-contract rung mid-cycle, against
  [Cycles run on a bookmark](../../AGENTS.md#cycles-run-on-a-bookmark), which left the trapezoid no
  trunk-side `<base>` and put the four pushed rungs inside `trunk()`, hence immutable. Rewound to
  the hack commit `qysmyxow` at wink's call, a ref move only: no commit is rewritten, so the rungs
  keep their SHAs and `ochid:` trailers, and the bookmark stays a descendant of `main` so Land's
  fast-forward is still a fast-forward. The hack is the trapezoid's `<base>` and so stays on the
  first-parent line, which the deliberation above already wanted

### Ladder details

#### docs: reshape at land opening

The cycle's setup, the bookkeeping an opening owes before its first rung.

- Backfill the seven fix-dev-artifacts rungs, the 0.80.3 cycle's.
- Sweep the 0.80.1 and 0.80.2 Done entries to done.md. The 0.80.3 entry stays for nearby
  context.
- Move the hack's Todo entry into this block.
- Start chores-18.md, since chores-17 passed 1000 lines.
- Bump the version-of-record.

#### docs: advance patch by default

versioning.md's shape-versus-contents test was undecidable in practice, so the section becomes
"patch by default".

- Patch is the default advance. Minor is deliberate and rare, called by the user at an opening
  and never inferred from a change's content or size, and major stays the project's call in
  custom.md.
- custom.md's version-bump paragraph is removed entirely, wink's call. The pinned rule is the
  whole story, and the paragraph restated it with a stale anchor and inline rationale.
- The Why moves to rationale.md as its first per-file section, per the intent that rule files
  carry the rule and rationale.md the argument. The sweep of the other agent-files' whys is
  filed as a Todo.
- Along the way wink stated the model, filed as the top Todo. The agent-files are the proposal
  the family dogfoods as its first users, and custom.md is the downstream users' override, kept
  at the payload default by the authors, so the Major bullet records a made promise as a local
  edit of the rule itself.
- The last "satellite" framing also leaves the agent-files, which are equals, each with its
  purpose.

#### docs: empty custom.md into the agent-files

Inserted at wink's direction after the payload-diff review. custom.md shrinks to its shell and
the dogfood log retires.

- Move custom.md's single-name convention into versioning.md's dev artifact name rule as the
  local diff, stated generically, and drop the Medium facts as derivable.
- Retire the dogfood log, draining its two in-flight entries first.
  - The semicolon exemption becomes a Todo, and the stable-name-install rule moves into the
    Land rung's principle.
  - notes/dogfood.md is deleted and the pinned references to it are swept.
  - The role survives as existing machinery: chafes become Todos, edits, and records.

#### docs: land under the at-rest contract

Born at the fix-dev-artifacts landing, where the recap followed the final push because the Land
bullets, the one text read at that moment, never mentioned the contract.

- AGENTS.md's At rest step 2 and jj.md's Land name a `jj git push` bookmark move as a push under
  hard rules 2 and 3, with closing words written before a landing sequence's final invocation,
  which runs into silence.
- At wink's review the section simplified to an actor timeline, two items for three.
  - The agent publishes: say it before the final publishing command, answer "Published", then
    nothing until the user speaks.
  - The user squash-pushes at will.
  - "Published" replaces "landed" as the token, Land now being a protocol step, and the push
    mechanics live in jj.md alone.

#### docs: move the reshape and restore under Land

Close-out step 5 chooses and records the shape, and Land executes it. The install is the last
act, run when nothing can enter the cycle anymore (drained from the dogfood log: a premature
install once let one version string mean two behaviors).

- Land, in jj.md, is the five-step permanence sequence: restore, reshape, fast-forward, install,
  delete. The single-name text now lives in versioning.md's Dev artifact name rule and follows
  the move, and Opening step 6's restore pointer follows it.
- The trapezoid recipe shrinks to the pure reshape, its merge published by the fast-forward
  itself. The topic bookmark is never re-pushed, so the old two-push window and its stale-SHA
  caution disappear.
  - notes.md's copy of that caution goes with them as the stale cross-reference this rung
    created, its "not on a permanent branch" half covering the case.
- Terminology rides the same seam. AGENTS.md gains Land, Trapezoid and Artifact, the vocabulary
  this rung made load-bearing.
  - rationale.md's Terminology loses the Rationale, Single-step or multi-step and Agent-files
    entries, wink's call. The first restates the mirroring rule the file's own preamble and
    How-to-read bullet already state, the second's definition is carried by AGENTS.md's Cycle
    entry, and the third argues three names without recording a cost, under the file's own bar.
  - AGENTS.md's two `[why](rationale.md#terminology)` links, both answered by deleted entries,
    go with them, leaving Retired names alone under the heading.
  - The name stays "trapezoid", not "trapezoidal-commit". The corpus says trapezoid throughout
    and the word already names the merge commit.
- rationale.md's At rest paragraph is rewritten here rather than in the already-pushed at-rest
  rung that authored it.
  - The three-clause chain collapses to one fixed-point argument, wink's: the agent's
    squash-push is an action that adds to the tail, so the user must do it. The duplicated
    "push cannot include its own record" clause goes, and the repeat-reason folds into "visibly
    or behind the scenes", one sentence carrying both who must act and when, the back end's
    late consolidation included.
  - The `ochid` sentence is corrected while it is open. A fold can only dangle the pointer
    aimed at the agent repo, which lives in the work-repo commit, not the agent-repo commit's
    own trailer, which aims the other way and is never at risk.

#### docs: read a change in gitk at full context

A jj.md section on gitk's three views at full context, for the reader making the shape decision.

- New version shows the result, with the additions lit, so it is the view for judging whether a
  change reads as one thing.
- Old version shows the file as it was, with the removals lit, which is what a diff states only
  as minus lines out of context.
- Diff shows the two interleaved, at full context the whole file with the edit marked in place.
- The section reads as two modes, wink's structure: one commit at a time, then a marked range.
  The range form is "Mark this commit" on the base and "Diff marked commit -> this" above it,
  after which the same three views show the net change, which for a cycle's opening parent and
  closing is what a squash would carry. Close-out shapes' Preview before choosing points here
  instead of restating the context trick.
- It is a `##` section rather than a subsection of Close-out shapes, since README.md links it
  from jj Tips for Git Users and the reading is not close-out-only.

#### docs: reshape at land closing

Closing out the cycle: the acceptance check above, the finalize, and the move of the In Progress
block into this section.

- The shape is trapezoid, wink's call at the `main` rewind. `<base>` is the hack commit
  `qysmyxow`, `<tip>` the gitk rung, and the merge goes out with Land's fast-forward, so the
  bookmark is never re-pushed.
- The title did not shift, so no anchor back-reference moved. The scope grew by two rungs, the
  custom.md emptying and the Terminology work folded into the Land rung, without changing what
  the cycle is about.
- The acceptance check's fifth item is the cycle dogfooding its own rule, so it reports at Land
  rather than here.

# References

[1]: #docs-reshape-at-land-opening
[2]: #docs-advance-patch-by-default
[3]: #docs-empty-custommd-into-the-agent-files
[4]: #docs-land-under-the-at-rest-contract
[5]: #docs-move-the-reshape-and-restore-under-land
[6]: #docs-read-a-change-in-gitk-at-full-context
[7]: #docs-reshape-at-land-closing
[8]: https://github.com/winksaville/vc-x1/commit/8e3cac729a8c "8e3cac729a8c01d6ef2d64bff3b285e5be0c2c7a"
[9]: https://github.com/winksaville/vc-x1/commit/fa4e8936fce6 "fa4e8936fce608270eb8fe2ad6425c8d5547299e"
[10]: https://github.com/winksaville/vc-x1/commit/d7ba0cc719b6 "d7ba0cc719b68cc39a022f9e91c700d730143ab8"
[11]: https://github.com/winksaville/vc-x1/commit/ba7a482ac2ec "ba7a482ac2ecf84ae1c2e17f570dba59ba10b5dd"
[12]: https://github.com/winksaville/vc-x1/commit/c8275ecde396 "c8275ecde396b73962295cae3737d81a0344476e"
[13]: https://github.com/winksaville/vc-x1/commit/94b2a161ee4b "94b2a161ee4b4a0a45e92e1ec8aad19c36c7c72d"
[14]: https://github.com/winksaville/vc-x1/commit/2dc8d969c3f3 "2dc8d969c3f351a0454d5c2b8f024b0db0af3965"
