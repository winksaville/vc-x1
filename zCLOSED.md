# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### agent-files(proposal): v0.3.0

(wink + agent, 2026-10-17, opened 2026-10-18) The cycle **chore: custom.md is untracked and
gitignored** took `custom.md` out of the tree, so anyone can clone this public repo without
inheriting one machine's layer. The agent-files still read as if every project has a `custom.md`
and belongs to a family: the rule **Read custom.md first**, [Acquaint](AGENTS.md#acquaint)'s first
step, [custom.md](AGENTS.md#custommd), and the family's terms across `agent-data/`.

#### Deliberation

- Every mention of the family or of `custom.md` leaves the agent-files but one: when a `custom.md`
  exists it is read first, and it overrides anything in the agent-files.
- A project's own change to the agent-files is a cycle like any other, so `custom.md` stops being
  the place for one, and nothing marks a change as project-only, wink's call.
- The template's shape is TBD, its work in progress. We think its `main` will carry no `custom.md`
  and a branch the family's, and the one rule holds either way.
- The rule says it once, at the top of `AGENTS.md` and in its section: the agent must read a
  `custom.md` first, it overrides the agent-files, and it is the suggested home of most or all
  local changes. A line in each `agent-data/` file's boilerplate saying the same was tried and
  dropped, once being enough, wink's call.
- By the same reasoning the banner itself goes, wink's call at the description review: `AGENTS.md`
  is read first and covers every agent-file, and `commit-model.md` and `cycle-model.md` never had
  one.
- The session lock becomes an experiment the set does not use: it works, and the complication is
  not worth it, wink's call. A workspace that wants it has its `custom.md` point at
  `session-lock.md` or carry the instruction, so that file names `custom.md` too.
- Single-step, reviewed as a local ladder: `AGENTS.md`, then `agent-data/` but the rationale, then
  `rationale.md`.
- A read of the whole set after the push, wink's ask, found three sentences still describing the
  old world, fixed in the pushed commit before Land: "Diff is the proposal" overclaimed once a
  project's own changes share the diff, `versioning.md` said it is copied verbatim while its `+`
  line says edit your copy, and nothing in the set linked `session-lock.md`. The rule's "most or
  all local changes" became "what is local to the workspace", and "A session's rules are its own
  agent-files" names `custom.md` beside them.
- Wink's review of those fixes: the session lock was a real intention, so `custom.md` names it as
  an experiment it may carry, not an example. And nothing in the set is owed acceptance: a
  proposal may ask for anything, whoever is asked may decline with its why or the change that
  would win it, and an adopter left in the minority keeps its own way or leaves. Wink rewrote
  **Changing the agent-files** to say so in four bullets, experiment, proposal, convergence,
  acceptance, and the own-commit, own-cycle, correction, and adopted-ahead rules retired with it,
  the proposal bullet covering them, wink's call.
- The agent's review of that rewrite: the four bullets are peers negotiating, and three places
  still spoke the hub they replaced. "Official" left the section's intro, and the terms gained a
  proposer and a "We think" that the template repository is still to come, v0.3.0 being a version
  on the path, wink's reading. The one correctness break was `versioning.md`: acceptance "identical
  or not" meant two adopters could carry different texts under one number and an adoption with
  accepted differences failed its own diff, so a version now names the text as proposed, an
  adopter's accepted differences are its diff, and the adoption check prints them. Then the
  maintainer went too, wink's call: there is no maintainer, only adopters and proposers.

#### Solution

- `AGENTS.md` states the rule once, in `## custom.md` and its index line: when a `custom.md` exists
  the agent must read it first, it overrides anything in the agent-files, and it holds what is
  local to the workspace, the session lock an experiment it may carry. Acquaint's first step links
  it.
- The terminology drops `custom.md` from the agent-files and retires "Project layer", and
  "Changing the agent-files" is four bullets: a change is tried in the local copy, proposed as one
  set of diffs, to the rules or beyond them, reviewed by each adopter as a negotiation whose goal is
  not a requirement, and accepted when adopter and proposer agree, identical or not.
- The seven `agent-data/` files that opened on a "Universal file, shared with the template
  repository" banner, its last sentence "Project-local content goes in custom.md", drop it whole.
  A project's own commit types and the `+` deviation are declared in its copy of `prose.md` and
  `versioning.md`, the adoption bullet drops `-c` and the family's `custom.md`, and the agent-files
  version names the text as proposed, an adopter's accepted differences being its diff.
- `session-lock.md` presents the lock as an experiment the set does not use, which a workspace's
  `custom.md` may point at or carry.
- The family's terms give way to the set's own, adopters and proposers, and `rationale.md`
  follows each rule, its `## custom.md` rewritten for the local file.
- `notes/agent-files-size.md` gains the v0.3.0 column. Its `custom.md` row, 15, is this machine's
  local file, which `vc-x1 agent-files size` still counts, so the agent-files' own total is 1826.

#### Acceptance check

- `grep -n -i 'family\|custom\.md' AGENTS.md agent-data/*.md` prints only the one rule, its index
  line, Acquaint's link, the session-rules sentence naming it, `session-lock.md`'s switch, that
  rule's rationale with the links to it, and a sha-link into history, and no line links to
  `custom.md`.
- `vc-x1 validate-anchors` passes on `AGENTS.md`, `agent-data/*.md`, `TODO.md`, and `zCLOSED.md`.
- Result: pass. The grep prints 18 lines, 7 in `AGENTS.md`, 3 in `session-lock.md`, and 8 in
  `rationale.md`, each of the allowed kinds, no line links to `custom.md`, and the anchors check
  reports 0 failed across 13 files.

#### Ladder

- agent-files(proposal): v0.3.0 (done)

# References
