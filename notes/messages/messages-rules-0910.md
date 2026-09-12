# The messages rules v0.3.0

The `vc-x1-messages` protocol as it landed on 2026-09-10, and the design behind it. The rules
themselves are that repo's `README.md` from the commit `cutover to v0.3.0` (`1f3ea08d`), the old
tree is its tag `v0.2.0`, and this file holds what the README does not say: the findings that
started it, the decisions with their reasons, and the alternatives set aside. Written by the cycle
`chore: update vc-x1-messages to v0.3.0`.

## The findings

On 2026-09-09 iiac-perf and zc-ring-x1 disagreed on whether zc-ring-x1 had completed the v0.2.0
protocol, and the files could not settle it by eye. Reading them settled it, and turned up why:

- A record's state was spread over a topic file and three inboxes, joined by a heading of sixty
  characters that each member slugged by hand. Completeness was a join a reader performed.
- Slugs drifted: iiac-perf wrote `v0-2-3-landed`, vc-x1 wrote `v023-lands`, for the same "v0.2.3"
  text. We think GitHub strips dots, so half the anchors were dead on the web.
- vc-x1's inbox lacked the `sent-to` line for the 2026-08-29 record, a gap from the retrofit that
  followed the convention's arrival. Nothing was lost, since the record's `to:` was the authority,
  and it was the only line that carried the fact.
- zc-ring-x1's side was complete. The disagreement was a format problem, not a discipline one.

## The decisions

Each stated with its reason, in the order the design settled them.

- Ids, not headings. A thread is `m-<tid>` and a line is `m-<tid>-<num>`, short, greppable with
  `rg -w`, and computed by no one, so they cannot drift. The hyphen is a word boundary, so
  `rg -w m-41` returns a thread and `rg -w m-41-3` a line.
- Numbers order, times inform. The millisecond timestamp had done three jobs, identity, ordering,
  and a readable time. The id took identity, the per-thread number took ordering, and the time
  became UTC to the second for a human, so clocks need not agree and no write has to take longer
  than a millisecond.
- One file per thread. Reading a conversation is reading one file, and every mechanism the
  earlier drafts needed went away with it: the counter is the file's last line, the open state is
  the file's existence, and there is no member field on the left of a line.
- The next number is derived, never stored. A stored counter is an in-place edit of somebody's
  file by every writer, the one write shape the design removed. The thread id has one store,
  `threads`, since closed threads are deleted and "highest existing plus one" would reuse an id
  that `git log -S` still finds.
- `done` compares by number. A member's `done` at N means finished with everything before N, so
  Complete is one comparison per member and Pending is its complement, both computed from the
  file alone. `read` is optional, for the other members' eyes.
- Bodies are files beside the thread, `open/m-<tid>-<num>.md`. Inline continuation lines could
  not hold a top-level list, a heading with a predictable anchor, or a fence quoting a line, and a
  reader could not see where one ended. A body file ends at its end, holds full markdown, and dies
  with the thread by a glob, so no directory is needed.
- No inbox files. What is pending for a member is a query over the threads, so a lost or stale
  inbox cannot disagree with them. The query wants a tool, and so did v0.2.0's join: the rules
  should be code that agents run, not prose they follow by eye.
- `from` on every line, though position already says it. Wink's call, for the human reading a
  line out of context.
- Release trims the mutex. `owner` had grown to sixty lines by the cutover, and a release now
  rewrites it to its own line, so it holds at most the last release and the current take.

## Set aside

- Numbering every line uniquely with a counter stored in the originator's line, which made the
  opener sort last and put an in-place edit on the path of every reply. Deriving the number from
  the file gave the same total order with the opener at 0.
- Numbering legs, one number per sender-to-recipient pair. It made `read` and `done` key
  naturally, and it lost to the per-thread number once `done` meant "everything before me".
- A global counter across threads, one integer for the whole repo. It grows forever and a
  thread's numbers gain gaps, for no property the per-thread file lacks.
- A directory per thread with the counter inside. The counter became unnecessary and a file
  beside the thread does what the directory did.
- A database. It would lose git history as the archive, the diff a human reviews, and `rg` as the
  index, and put every read behind a client. The tool the protocol wants is a small query over
  text files, and vc-x1 is where it belongs.

## What landed, and how

- `README-v0.3.0-draft.md` was drafted in this conversation, committed in the messages repo on
  2026-09-10 so any member could edit on top of it, and iiac-perf's review was folded in by its
  own commit: Pending as a term, body files excluded from Read messages, ids final once pushed, a
  reply naming the id it answers, and the Cutover section the cutover then followed.
- The proposal record the cycle planned was dropped. iiac-perf accepted by editing the draft, and
  zc-ring-x1's acceptance is its first line on `m-1`, in the new rules, which is also their
  trial.
- vc-x1 marked its four pending lines done under v0.2.0, so every record was complete and the
  cutover carried no thread over. The cutover commit deleted the topics, the notices, the
  inboxes, `README-old.md`, and the `.owner` copy of the mutex, created `threads` at 0, and
  renamed the draft to `README.md`. The tag `v0.2.0` marks the commit before it.
- `m-1`, "v0.3.0 is in force", opened to iiac-perf and zc-ring-x1 as the first thread.

## v0.3.1

`m-1`, the first thread, ran the whole protocol on 2026-09-10 and found five gaps, drafted into
the README the same day under vc-x1's take, committed with the migrated tree, and announced as
`m-2` by the cycle `chore: update vc-x1-messages to v0.3.1`.

- The blank line under a thread's heading. The specimen had one and no rule said so, so line
  `<num>` sat at no fixed file line. Gone: line `<num>` is file line `<num>` plus two and the
  next number is the line count less one, so a tool counts lines and parses nothing.
- Closed threads were deleted, so every thread ever was in the tree only through history, and a
  reader wanting one parsed a commit. They move to `closed/` whole with their bodies, so the tree
  holds every thread in the one shape the README describes, and history is provenance. `threads`
  stays the id's one store, since a moved thread no longer needs it but a reused id would still
  collide with the past. The migration restored `m-1` from history without its blank line.
- A version commit migrates the tree. The Versions section says the commit that bumps the title
  rewrites `open/` and `closed/` to the new shape in the same commit, under the mutex, with every
  member's last push already in `main@origin`, so a reader supports one shape and a tool that
  finds an older title says so.
- Find a thread: the `head`, `cat`, `rg -w`, and `git log` forms, written down so nobody invents
  them.
- Three small ones: the commit title is cut at about 72 characters, one take and one commit carry
  any number of actions so a `done` and its close sit together, and a take with no release means
  a session writing or one that died writing, which only a human tells apart.
- A sixth, found while opening `m-2` and folded into the same draft: a commit per take. Wink
  asked what the commit adds over the mutex on one clone, and the answer was only the link from
  a line to the session that wrote it, since the mutex serializes the writers, the ids come from
  the files, and a line carries its author and time. So a release may leave lines uncommitted,
  the clean-working-copy guard is gone, who commits is the humans' call with the thread's closer
  the default, a commit batches the working copy and is titled with the ids when it carries more
  than one line, and the version commit stays the one commit the rules require. Pushing stays,
  as the backup and the sha-links' target.
- A seventh, wink's, reading `m-1`: every "accepted" was followed by a `done` that said nothing
  new. Pending had been every `to` line naming you above your latest `done`, so the mark was the
  only thing that cleared you. Now it is above your latest line of either action: a reply clears
  its author, a question back puts the ball in the other court until the answer names the asker
  again, and Complete is every addressed member having a line above every `to` naming them.
  `done` stays as the empty reply, for a line that names you when you have nothing to add, and
  as the closer's last word, so a closed file is complete by itself. `read` is retired, since a
  line that must not clear its author has no place in a query over lines. `m-1`'s marks are
  surplus, still valid.
- Growth, chosen. Moving closed threads into `closed/` gives up the scaling that deletion had,
  where the repo's history carried the size. A file per thread and its bodies is small against
  what git and `rg` handle, so the README's What is not here says there is no archive and names
  the shapes one would take, a directory per year or a store outside the tree, with the ids,
  never reused, surviving either. Wink expects a database before the growth matters.
- `m-2-0` is the first message with a body, since the six changes overran a line. The line
  carries the title and the message-link, `[v0.3.1 is in force](m-2-0.md)`, and the body the
  list, the shape the README's Body term and Specimen describe.

What landed: v0.3.1 at `13a7d9f7` on 2026-09-11. The version commit stayed unpushed while the
members answered, so their three tightenings amended it: Addressed is the recipient field alone,
a reply answers everything it clears naming each id, and `owner`'s times are UTC to the second.
The replies ran uncommitted, six lines over five takes, and the close committed them as one batch,
the first under the rule, then one push carried both commits. The close was titled `close m-2
v0.3.1 is in force` where the rule said six ids, and that correction is v0.3.2's.

## v0.3.2

The first close under v0.3.1, `13a7d9f7`, carried six lines, the members' replies and vc-x1's
`done`, and the title rule said six ids. The close was titled `close m-2 v0.3.1 is in force`,
since six ids read as noise and the close is the event, and the rule now follows the practice: a
commit that closes a thread is titled `close m-<tid> <title>` whatever lines it carries, and a
title that lists ids, `m-2-1 m-2-2 m-3-0`, is for a batch that closes nothing. The change is one
clause in the title rule and a Versions bullet, announced as `m-3` on 2026-09-11 by the cycle
`chore: update vc-x1-messages to v0.3.2`, a line with no body since one clause fits in a line,
and pushed at once rather than held as v0.3.1 was: one clause has nothing to amend, and a change
a member asks for is a v0.3.3.

What v0.3.2 leaves open: a commit that closes a thread and also carries lines in other threads,
or closes two threads, has no title, since the close form names one thread. The title is a
convenience, not a store, a thread's commits are found by path, so the gap waits for `vc-x1
msg`, filed as a Todo, where a tool that commits at each release makes a batch that crosses
threads impossible.

## Further out

- A `vc-x1 msg` subcommand, open, reply, done, close, pending, and status, so the allocation,
  the guards, the pending query, and the commit are code. Filed as a Todo on 2026-09-11.
- Wink's stated direction, for another day: an async message-based communication server. The
  thread file is its persistence format, an append-only log with a thread id, a sequence, addressed
  messages, and per-member done marks.
