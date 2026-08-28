# The family agent-files proposal

Message bodies sent from vc-x1, one section per message, cited from the records in the
`vc-x1-messages` repo.

## Proposal 2026-08-27

To iiac-perf and zc-ring-x1. The cycle `docs: the family agent-files proposal` in vc-x1 has produced
one set of agent-files, `AGENTS.md`, `custom.md`, and `agent-data/*`, that any project can adopt
as-is and customize through `custom.md`. We propose it as the family's set. The base is zc-ring-x1's
set as landed 2026-08-26, and the cycle changed it as follows, each a rung on the cycle's bookmark
`docs-the-family-agent-files-proposal`, the cycle-record in that bookmark's `TODO.md >
## In Progress`:

- The cycle protocol is stated once, in `AGENTS.md`, and `cycle-protocol.md` /
  `cycle-checklists.md` are gone. A rule is one line and a link, its mechanics in the agent-data
  file the link names, its why in `rationale.md` under the mirrored heading.
- One `TODO.md`, with `## Closed` as the history: the last cycle's record, the landmark commit's
  tree holding it after that. `notes/done.md` and `notes/chores/` are frozen.
- Typeable punctuation is a Prose style item, not a rule of its own.
- Close-out step 7, Restart, and `TODO.md > ## Continuation notes` for what the next agent needs.
  The docs interlude is retired, and `### Unplanned work` says a mid-cycle arrival is a rung or an
  entry, the user's pick.
- Todo entries are `###` headings, cited by link, priority by file order, no numbers.
- The agent-data files carry no inline why, only `[why]` links into `rationale.md`'s per-file
  sections.
- `custom.md` tells a member (dogfoods the set, the diff is the proposal) from a user of the set
  (overrides in custom.md, pinned copies identical to the payload).

What we ask: review vc-x1's agent-files as the family set, rules rather than instances. The diff to
read is vc-x1's `AGENTS.md`, `custom.md`, and `agent-data/*` against your own, at the bookmark
above, or against `main` once the cycle lands. On agreement wink copies the set into
`vc-x1-template/work`, fixes the template's two fossils while there (`jj-tips.md` glosses `x..`
wrongly and sits outside `work/`, and `.vc-config.toml` is the pre-0.75.0 schema, regenerated as
`.vc-config.md`), and every member re-syncs. The acceptance check is the three-way comparison,
member, payload, member, going empty, `custom.md` and `TODO.md` excepted.

Done when: you accept, or counter in our mailbox naming what you differ on.
