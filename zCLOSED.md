# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### agent-files(proposal): v0.2.9

(wink, 2026-10-15) The continuation note that opened this cycle, moved here whole:

> The next cycle is an `agent-files` proposal (wink, 2026-10-15): the Validate step of [The
> per-rung flow](AGENTS.md#the-per-rung-flow) gains the rule that a failure is read and its
> cause named before a second run, and a failure that does not repeat is a finding, reported
> and given a Todo entry. It was the Todo "A validation failure is investigated before it is
> run again", retired by the cycle `chore: the validation-failure Todo retires to the per-rung
> flow change`, whose record in [zCLOSED.md](zCLOSED.md) holds the case the rule was drawn from.
> Open it at once, ahead of the Todo's first entry, and move this note into its record.

The case is in that cycle's record, the `zCLOSED.md` of its landmark on `main`, `e331001`, which
this record replaced.

#### Deliberation

- Single-step: two rules and their whys in one commit, as the proposals v0.2.6 to v0.2.8 were.
- The version-of-record goes to 0.85.0, a minor bump in place of the default patch, wink's call
  at the work review.
- The validation rule is one sentence in the Validate step, and no Rules index line. The index's
  "Read the step before the action" already sends a session to the step, and the rule binds only
  there.
- The validation rule's why names the defect class and the case's date, not the test or the
  project, per [Agent-files name no project](agent-data/prose.md#agent-files-name-no-project).
  The case is reachable from this commit through its record.
- v0.2.9 leaves out iiac-perf's proposed check under Before any push, that the review shows the
  agent-repo carrying the session's transcript (`m-10-0`). It goes with the snapshot bug it came
  with, the cycle after this one (wink, 2026-10-15).
- A per-rung sync joins the cycle, raised by wink at the work review: step 0 of [The per-rung
  flow](AGENTS.md#the-per-rung-flow), required at the opening and the closing, optional between,
  the agent's or the user's call.
  - Step 0, wink's call over the push: a rung's start has an empty working copy, so it is the
    cheapest moment to take in a moved `main`, and the sync never moves the bookmark. The first
    draft synced before the push, as a bullet under [Before any
    push](AGENTS.md#before-any-push), for the agent-repo's `main`, which every push lands on
    directly. It came out, the agent-repo case being the cost accepted.
  - Numbered 0 so no step renumbers, since steps are cited by number.
  - Its moments go into [Sync](AGENTS.md#sync) too, which holds the command so no step restates
    it, and the opening's existing first step is its step 0.
  - "Optional" names whose call it is, the agent's or the user's, wink adding the user at the
    work review, since an option no one owns is skipped.
  - A step 0 sync that moved a `main` stops the rung, as Land's stops the Land.
  - It replaces the Sync rationale's "Cycle starts and no oftener", which weighed only "when a
    repo has not been synced in a while", a moment an agent cannot check.
- rationale.md's per-rung flow entry said step 4 bumps the version, where it is step 2, a stale
  reference fixed in passing.

#### Solution

The Validate step of [The per-rung flow](AGENTS.md#the-per-rung-flow) gained the rule that a
failure is read and its cause named before validation runs again, and that a failure that does
not repeat is a finding. The flow gained a step 0 sync, required at the opening and the closing
and optional between, [Sync](AGENTS.md#sync) lists its moments, the Rules index's Sync line names
the closing, and rationale.md carries the why of each.

#### Acceptance check

- AGENTS.md's Validate step says a failure is read and its cause named before validation runs
  again, and that a failure that does not repeat is a finding, reported and given a Todo entry.
  Passed.
- rationale.md's `### The per-rung flow` carries the rule's why. Passed.
- AGENTS.md's per-rung flow has a step 0 sync, required at the opening and the closing and
  optional between, Sync lists those moments, the Rules index's Sync line names the closing, and
  rationale.md's `## Sync` carries the why. Passed.
- `ls agent-data` shows `agent-files-v0.2.9` and no other version file. Passed.
- `vc-x1 validate` passes. Passed.

#### Ladder

- agent-files(proposal): v0.2.9 (done)

Close-out shape: none to choose, single-step. Land fast-forwards `main` over the one commit.

# References
