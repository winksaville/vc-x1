# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: update single-step cycle add zCLOSED.md

(wink + agent, 2026-10-11) A session of planning left `TODO.md` holding a day of decisions as
uncommitted edits, with no cycle to carry them: what `vc-x1 delete` and `vc-x1 rename` are to do,
the order `init` is fixed in, and two faults the day turned up, commits no command reports and a
checker that reports a working link as broken. Carrying them became a trial of how a single-step
cycle runs and of where the last record is kept, and the trial is what the cycle is named for.

#### Deliberation

- The cycle is single-step, wink's call: the edits were made in conversation, entry by entry.
- The cycle is a trial of a new shape for a single-step cycle, wink's call at the work review of
  the one-commit form, before any `vc-x1 push`: two stages, the work and the closing, each
  validated, reviewed, and pushed, so a single-step cycle and a multi-step one run alike.
  - The first stage carries the opening's duties and the work, under the dev name at the `-1`
    suffix, the record in `## In Progress`.
  - The closing runs the acceptance check against the pushed work, sets the bare version, and
    moves the block to this file.
  - The work-repo carries one commit and the agent-repo two, wink's call at the closing's
    review. The closing is folded into the pushed work commit, which keeps its change id, and
    the second agent-repo commit's trailer names that same commit.
    - The work commit carries two `ochid:` trailers, one for each agent-repo commit.
    - Two work commits squashed at Land would have left one agent-repo commit naming a change id
      that no longer exists, the reason a ladder is never squashed.
    - It was done by hand: a `jj squash`, a `jj describe` that kept the first trailer and added
      the second, written by hand, and then `vc-x1 push`, which skips an empty work side and
      stamps the agent-repo commit with the existing work commit's id.
    - It is a re-describe of a published commit, coordinated with wink, and it bends [No
      hand-written trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers) for the one
      trailer.
    - `vc-x1 validate-desc` and `fix-desc` read the last trailer alone. Both, and a `push
      --amend` that does the fold, are Todo entries written by this closing.
  - The cycle was renamed at the closing, wink's call: it opened as "docs: Todo plans init,
    delete, and rename", which names the occasion and not what was done.
    - The work commit is re-titled with the fold. The first agent-repo commit keeps the old
      title, that repo being a journal of what was said at the time, and the bookmark keeps the
      old name until Land deletes it.
  - The last cycle's record stays readable until this one's closing replaces it, wink's call at
    the second work review, where the opening would have deleted it. There is then always a
    finished record, and the opening's diff is its own cycle alone. Wink means it for a
    multi-step cycle too.
    - It was first tried as two blocks in `zDOING.md`, the old one left under `## Closed`. They
      shared the file's reference definitions, so the new block's had to number on from the
      old one's, and they repeated the item headings, Deliberation, Solution, Acceptance check,
      and Ladder, which `vc-x1 validate-anchors` reported as four duplicate anchors.
    - A list of references inside each block was weighed and left: a definition is the file's
      wherever it is written, so two blocks cannot both number from 1. A rung's title as its
      label would have answered that and not the headings.
    - A file of its own, wink's idea, answers both: `zCLOSED.md` holds the last record and its
      references, each file numbers from 1, and the closing replaces that file's block whole.
      Its cost is a third file beside `zDOING.md` and `TODO.md`.
    - The duplicate reports brought the Todo entry [validate-anchors warns on a duplicate anchor
      only when a link names
      it](TODO.md#validate-anchors-warns-on-a-duplicate-anchor-only-when-a-link-names-it), which
      stays as a fix to the checker with its case gone.
  - Land restores the plain name, as a multi-step cycle's does. Restoring it in the closing was
    weighed and left: the closing's validation would install the stable binary before `main`
    carries it.
  - The one commit lands as it is, with no merge commit, and `main` is pushed once.
  - The bookmark's create and publish needs no approval, wink's call. The two pushes and the
    Land keep theirs.
  - It bends [Cycle shape](AGENTS.md#cycle-shape), [Cycle-record](AGENTS.md#cycle-record),
    [Opening](AGENTS.md#opening), [Close-out](AGENTS.md#close-out), and [Before any
    push](AGENTS.md#before-any-push), for this cycle alone and as far as the bullets above say.
    Whether it becomes the rule is an `agent-files` proposal's to settle.
  - The ladder is one unlinked rung with the bare title, as a single-step cycle's is today, there
    being one work commit, and no title carries " closing". What the closing taught is under a
    `#### Closing` heading of its own, a rung subsection headed by the bare title colliding with
    the title heading's anchor.
- The record was kept in `zDOING.md`, continuing the trial the last cycle began, wink's call at
  the opening.
  - It bends [Cycle-record](AGENTS.md#cycle-record) as that cycle's did: where the block is
    written and its order, and nothing else.
  - Whether that file and `zTODO.md` are the right ones is not settled, which is why
    `agent-files(proposal): v0.2.8` stays parked.
- The commit's body is one paragraph, the problem, what the commit does, and a pointer to this
  record, wink's call at the description review, where the first draft ran to 34 lines.
  - It bends [Commit-body form](agent-data/prose.md#commit-body-form), for this cycle's body
    alone. The rule is the Todo entry [A commit body is an intro and a pointer to the
    record](TODO.md#a-commit-body-is-an-intro-and-a-pointer-to-the-record), part of the parked
    proposal.
- No `## Todo` entry was picked up, so the block opens with the problem, where an entry's text
  would be.
- The order of `## Todo` is wink's: the `validate-anchors` entries first, then `init`, delete,
  and rename, with the parked proposal after them.
- The checker's false failure is left standing and is the first entry's to fix. Taking the
  backticks out of the heading it trips on would pass the check today and lose the heading's
  mark of a command.
- The lock a running agent holds is planned as an experiment in this project's `custom.md`, with
  no `agent-files` proposal, wink's call. Nothing of it is in this cycle.
- The restore written for `sync --undo` was a draft under `tmp/` and is gone, wink having cleared
  `tmp/`. Its entry now says it is rewritten from the description.
- The `## Waiting` conditions are unmet, `vc-x1 closed` not landed and jj-lib offering no remote
  query, so nothing promotes.

#### Solution

`TODO.md` took the day's planning in the first stage, and this closing was folded into its
commit.

- Three `validate-anchors` entries head `## Todo`: a new one for the slug of a heading holding a
  code span, the cross-file check, moved up, and a new one for duplicate anchors no link names.
- The entry for `init`, delete, and rename follows, in the order they are to be done, with what
  stops the two commands and the lock a running agent holds.
- A new entry has `vc-x1 status` report the commits no bookmark reaches.
- The parked proposal's entry says why it stays parked, and the `sync --undo` entry that its
  draft is gone.
- The continuation notes are reset in both files, each fact acted on or filed.
- The last cycle's record has a file of its own, `zCLOSED.md`, and this block replaced it there.

#### Acceptance check

- `vc-x1 validate` passes. Passed, on the pushed work.
- `vc-x1 validate-anchors TODO.md` reports one failure, the false one on the link to
  `#support-por-workspaces-in-push`, and no other. Passed.

#### Ladder

- docs: update single-step cycle add zCLOSED.md (done)

#### Closing

Close-out shape: one commit, the closing folded into the work commit.

- The shape of the cycle changed five times at its two work reviews, each time by looking at
  what would land.
  - One commit became two, the last record was kept, the last record moved to a file of its
    own, the two commits became one, and the cycle took a new name. None was in the plan the
    opening was approved on.
  - What is to become of the trial is the Todo entry [A single-step cycle is two stages and one
    commit, and the last record has a file of its
    own](TODO.md#a-single-step-cycle-is-two-stages-and-one-commit-and-the-last-record-has-a-file-of-its-own).
- A cycle that only describes what is wanted changes nothing, wink's remark at the closing.
  - The trial changed how this cycle ran, and the agent-files still describe the old shape, so
    the next session runs the old shape unless it is told.
  - We think an override in `custom.md` is the next step, short of a proposal to the set.
- A review that iterates on markdown pays for a whole validation at each turn.
  - A dozen runs of clippy, the tests, and an install were made for edits to three markdown
    files, the rule being that validation passes after the last edit.
  - We think a change that touches no source wants a check of its own, the anchors and the prose
    rules, and this is not yet an entry.
- One validation failed on a test that was not this cycle's, and passed when run again.
  - It is bug 21 in [bugs.md](notes/bugs.md), written by this closing.

# References
