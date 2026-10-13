# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### agent-files(proposal): v0.2.8

(wink + agent, 2026-10-13) The agent-files name a fetch at an acquaint only, as a `jj git fetch`
and a look, and it is as important that a repo is up to date at the start of every cycle and
when a cycle lands. The Todo entry picked is "agent-files(proposal): v0.2.8", parked since
2026-10-09 as one unfinished commit on this cycle's bookmark. Wink asked for the sync rules
clean, "as simple as possible but no simpler".

#### Deliberation

- The rule is one section, [Sync](AGENTS.md#sync), the agent's shape for "as simple as possible":
  the command is written once and the three steps that run it point there.
  - Acquaint looks with `--dry-run`, and Opening and Land sync, each as its first step, wink's
    call, 2026-10-11, on a review of the parked commit.
  - The cycle's bookmark is looked at with `--bookmark` and never synced. The earlier look,
    `jj bookmark list --all-remotes`, read remote refs the fetch had just moved, and a look-only
    sync moves none, so the list would be stale.
  - Land's sync is a clause of the Land paragraph in `jj.md` and not a numbered step, so the
    trapezoid recipe's "Land step 2" keeps its number.
- A sync that fails or holds a repo back stops the step, and `--force` and `--rebase` are the
  user's to ask for, the agent's addition. Sync holds a repo back since **feat: sync is safe by
  default**, and the parked commit was written before it.
- Started over and not rebased, wink leaving the choice to whichever was simpler.
  - The parked commit's wording was of the sync that rebased a `main` that diverged, and its
    Todo entry had already written it off.
  - Its reasoning is kept in `rationale.md`: why Opening syncs, cycle starts and no oftener,
    "whatever its form", and the bookmark left to a look.
  - The bookmark `agent-filesproposal-v028` was moved to `main` and force-pushed at the opening,
    wink's go, so the parked commit, `2b14ec7e`, change `psuuzqlv`, is findable by that id in a
    clone that fetched it and nowhere on the remote.
- The messages repo's fetch is a line in `custom.md`. Wink's standing permission is the parked
  commit's, dated 2026-10-09, and it takes effect with this cycle.
- An adoption copies `custom.md` and ends on a diff, wink's call: `agent-files copy -c`, then
  `agent-files diff -c` as its last act, in [Agent-files
  version](agent-data/versioning.md#agent-files-version).
  - The family's three copies of `custom.md` are the one messaging line, so the file is shared
    in practice, and a copy without `-c` would leave the fetch line behind.
  - An adopter that disagrees with a line drops the `-c`, its prerogative.
  - The other members carry v0.2.5, so their next adoption takes v0.2.6, v0.2.7, and this one in
    one copy, and the versions are not combined here.
- The line saying the session lock is off stays in `custom.md`, wink's call, so every member has
  the option of turning it on. The agent had proposed removing it.
  - The instruction moves into the set, [session-lock.md](agent-data/session-lock.md), wink's
    call, so the family decides on it. A `notes/` directory is every project's and its contents
    are the project's own, so the line's link to `notes/session-lock.md` would be dead in every
    other member's copy.
  - The new file holds the instruction, how to turn it on and off, and what is known of it, and
    names no project. What the tests found stays in `notes/session-lock.md`.
- The rule of the Todo entry "A commit body is an intro and a pointer to the record" is left for
  a proposal of its own, the agent's proposal at the opening, so this one is the sync rules
  alone. The entry said it was part of v0.2.8, and now says what happened.
- The entry's bullet "It stays parked", waiting on whether `zDOING.md` and `zTODO.md` are the
  right files, is lifted by wink's go.
- How a Land recovers from a `main` that moved is left to the first time it happens. We think a
  trapezoid takes the moved `main` as its merge's first parent with no rung rewritten, and a
  single-step commit needs a rebase, which rewrites a published bookmark.
- A cycle first opened this session as "docs: delete stale agent-files proposal v0.2.8", on a
  bookmark of its own, was dropped before any commit when wink chose to complete the proposal.
  Its bookmark was deleted locally and on `origin`.
- The commit body is its intro paragraph and a pointer to this record, wink's call at the
  description review, where [Commit-body form](agent-data/prose.md#commit-body-form) asks for the
  facets and their solutions under it. The exception covers this commit's body and nothing else.
- A single-step cycle, with a bare version, a bare agent-files version, and no `-dev` name, the
  cycle being documents alone.
- The `## Waiting` conditions are unmet, so nothing promotes: `vc-x1 closed` is not a command,
  and the project builds against jj-lib 0.45.

#### Solution

The agent-files gain a section, [Sync](AGENTS.md#sync): both repos are brought up to date by
`vc-x1 sync --scope=both` at the three moments a cycle depends on `main`.

- An acquaint's repo check looks, with `--dry-run`, in place of its `jj git fetch`.
- An opening syncs as its first step, before the bookmark.
- Land syncs as its first step, and a `main` that moved stops it.
- The why of each is in [rationale.md](agent-data/rationale.md#sync).
- The family's `custom.md` adds the messages repo's fetch at an acquaint and at a cycle start.
- An adoption is `vc-x1 agent-files copy -c` and ends on `vc-x1 agent-files diff -c`.
- The session lock's instruction is in the set, off, for the family to decide on.

#### Acceptance check

- [Sync](AGENTS.md#sync) names the command once, and [Acquaint](AGENTS.md#acquaint),
  [Opening](AGENTS.md#opening), and [Close-out](AGENTS.md#close-out)'s Land step each point at
  it. Passed.
- `rg 'jj git fetch' AGENTS.md custom.md agent-data/` finds only the why of the look in
  `rationale.md`, which names the earlier form. Passed.
- `vc-x1 sync --scope=both` run in this cycle reports both repos up to date. Passed.
- `vc-x1 validate` passes. Passed.

#### Ladder

- agent-files(proposal): v0.2.8 (done)

# References
