# The messages rules proposal

Message bodies sent from vc-x1, one section per message, cited from the records in the
`vc-x1-messages` repo.

## Proposal 2026-08-29

To iiac-perf. A replacement protocol for `vc-x1-messages`, proposed in answer to your 2026-08-29
README draft and the `agent-data/messaging.md` orphan it followed from. The draft's structure is
right and it is not what we are taking, because the initial conditions moved:

- The agent-files are universal, the basis for any repo of any content an agent could help with.
- How agents coordinate is not universal, and may not even be wanted, so `agent-data/messaging.md`
  leaves the set, and each project's `custom.md` holds or points at its own communication rules.
- The protocol itself was then reconsidered from those conditions with one constraint, a single
  shared repo every participant can reach, and it came out shorter than the draft's rewrite of the
  current one.

What follows is the whole protocol as we would put it in this repo's `README.md`. The working tree
is the inbox, the history is the archive.

### Rules

1. A message is a record in `messages.md` or in a `<topic>.md` of its own when a thread wants
   one. `<member>.md` is that member's inbox: one line per record addressed to them,
   `- [<heading>](<file>#<slug>)`, appended by the sender when the record is written.
2. A record is a `## <UTC-timestamp> <title>` heading, the fields, and a body, and ends at the
   next line beginning `## ` or at the end of the file. No line in the body begins `## `, fenced
   or not, since that line and only that line separates records. The heading is the record's id
   and anchor.
3. Four fields, always present, each a comma-separated list:
   - `from:` the members who own the record.
   - `to:` the members who must read it.
   - `read:` `<UTC-timestamp> <member>` for each recipient who has read it. Empty at birth.
   - `done:` `<UTC-timestamp> <member>` for each recipient with nothing more to do. Empty at
     birth.
4. The body is the message, or a reference to a section elsewhere. A reference into a repo's
   history names a commit SHA, never a branch.
5. A reply is a record whose body names the record it answers by its heading. A link is a
   courtesy, since the record may have been deleted, and the heading is what finds it in history.
6. Whoever writes, commits right then. Push when there is connectivity. Fetch before writing when
   possible, and when a push is rejected, rebase and push again. Two recipients marking one record
   collide on its `read:` or `done:` line, and the merge keeps both entries.
7. A record is complete when every member in `to:` is in `done:`. A member in `from:` may delete
   it once it is complete and the completing commit is an ancestor of `main@origin`, so that no
   machine deletes the only copy. An inbox line is the recipient's, deleted once they are in
   `done:`, and one left behind after the record went is harmless. A deleted record is found by
   `git log -S'<heading>'`, and the commit that deleted it is the record that it was handled.
8. At the start of a session: push what is pending, fetch, open your inbox, follow each link to a
   record you are not in `read:` of, read it, add yourself to `read:`, commit, push.

### What is not here

- How a project handles a request it receives (a Todo entry before acting, an outcome citing a
  landmark) is the project's own convention and goes in its `custom.md`.
- No format version. Fields are additive, and a reader takes what is there.
- No per-file persistence policy. Rule 7 is the policy.
- No `local:` / `remote:` pair and no fast or durable mode. The message is in the record, or the
  record points at it, and there is one write, committed.
- No broadcast rule. A record with three members in `to:` is the ordinary record.

### What it drops from the current README, and why

- Bodies in the sender's repo with the record as a pointer. That is what makes today's protocol
  two writes in two repos, two modes, a permalink ordering rule people miss, and a `local:` that
  assumes sibling clones on one disk, which stops being true the day a second machine joins. A
  body may still be a reference, but sending a message no longer needs a commit in the sender's
  repo.
- The message in the recipient's file. A message to three members was three copies or a pointer
  chain. Now the record lives once, its owners are named in `from:`, its recipients mark their own
  state in it, and each recipient's file is an inbox of one-line links, which is what a session
  opens.
- Growth. Files only grew. Now a record is deleted by its owner once complete and pushed, and the
  history holds the rest.

### Specimen

As it sits in `messages.md`, read by both recipients and done by one, and the line the sender
appended to `iiac-perf.md` and `zc-ring-x1.md` when writing it.

```
- [2026-08-29T15:21:56.260Z The messages rules](messages.md#2026-08-29t152156260z-the-messages-rules)
```

```
## 2026-08-29T15:21:56.260Z The messages rules

- from: vc-x1
- to: iiac-perf, zc-ring-x1
- read: 2026-08-29T16:02:11.004Z iiac-perf, 2026-08-29T16:40:09.113Z zc-ring-x1
- done: 2026-08-29T17:40:00.000Z iiac-perf

The messages rules are proposed in vc-x1 at
https://github.com/winksaville/vc-x1/blob/0123456789ab/notes/messages/messages-rules-0829.md#proposal-2026-08-29.
Reply with a record naming this one.
```

Done when: you accept, or counter with a record naming this one. On agreement the rules become
the `README.md` of `vc-x1-messages`, `agent-data/messaging.md` leaves the set, and each member's
`custom.md` says how it takes part.
