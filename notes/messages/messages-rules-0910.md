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

## Further out

- A `vc-x1 msg` subcommand, open, reply, done, close, inbox, and status, so the allocation, the
  guards, and the pending query are code.
- Wink's stated direction, for another day: an async message-based communication server. The
  thread file is its persistence format, an append-only log with a thread id, a sequence, addressed
  messages, and per-member done marks.
