# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### docs: a session takes the workspace lock at acquaint

(wink + agent, 2026-10-11) Two agent sessions started in one workspace share one working copy,
and a push takes the working copy whole, so one session's edits land in the other's commit even
when the two touch different files. Nothing told a second session that a first was running.

#### Deliberation

- The lock is exclusive, one session per workspace, wink's call.
  - It was first taken shared, with rename and delete to ask for it exclusive. A second session
    then started as freely as the first, which is what the lock was wanted to stop.
  - A user who wants two agents at once makes a second clone.
- A foreground probe comes before the holder, `flock -xn .agent-running true`.
  - The holder is a background command, and its failure is reported seconds after it starts.
    The first trial session fetched both repos in that time.
  - The second trial session, told to confirm its holder was running, searched for it with
    `pgrep`, which sees only its own sandbox and printed the sandbox's command line.
  - Two sessions starting in the same second can both pass the probe. The loser's holder then
    exits, and the line gives that exit the same stop.
- A refusal is one fixed sentence, and no command follows it.
  - "Reports that" drew a summary, a suggested `lslocks`, and an offer to go on without the lock,
    which Claude Code turned into a one-key suggested prompt.
  - "Does nothing further" was read as nothing further of the acquaint: the next session still
    ran `ps` to find the holder. "Runs no further command, not even to find the holder" closed it.
- It is a rule of this project's `custom.md` and no `agent-files` proposal, wink's call.
  - A rule meant for the set belongs in [Acquaint](AGENTS.md#acquaint), and this one is not
    ready for other adopters: `flock` is a Linux tool, and two cases are untested, below.
  - The number v0.2.8 is the parked proposal's. We think the lock is proposed by the cycle that
    has `vc-x1` take a lock of its own.
- The Todo entry [The lock a running agent holds](TODO.md#the-lock-a-running-agent-holds) was not
  moved here. Most of it is still to do, `.vc-x1-running` first, so the block opens with the
  problem and the entry stays first in `## Todo`, where wink moved it.
- Two cases are left to that entry, untested: whether the holder dies with its session, which
  ends the session that would report it, and what a reacquaint does in the session that holds
  the lock.
- A reacquaint does not always read `custom.md` again. Two trial sessions skipped the read, so an
  edit to the line reached only a fresh session. It is a line in the Todo entry.
- The title is 53 characters, over the cap in [Line widths](agent-data/prose.md#line-widths),
  wink's call when the agent's count came after the bookmark was published. The bend covers
  this cycle's title alone.
- The cycle is two stages and one work commit, the shape the last cycle tried, wink's call.
  - It bends [Cycle shape](AGENTS.md#cycle-shape), [Cycle-record](AGENTS.md#cycle-record),
    [Opening](AGENTS.md#opening), and [Close-out](AGENTS.md#close-out) as that cycle's record
    says, in the landmark `docs: update single-step cycle add zCLOSED.md`, the record kept in
    `zDOING.md` included.
  - The fold of the closing into the work commit is by hand again, `push --amend` not being
    written, and is a coordinated re-describe with one trailer written by hand.
- The bookmark's create and publish was approved by wink at the opening.
- The size table's new column is labelled `v0.2.7-lock`, the set's version with a trailer, the
  set itself being unchanged.
- The `## Waiting` conditions are unmet, `vc-x1 closed` not landed and jj-lib offering no remote
  query, so nothing promotes.

#### Solution

`custom.md` carries the lock as its first line, and the plan for the rest is first in `## Todo`.

- A session probes `.agent-running` at acquaint before anything else and holds it exclusive
  with a background command when it is free.
- A session that finds it held replies "Another session holds `.agent-running`, stopping." and
  runs no further command.
- The lock file is in `.gitignore`.
- The Todo entry describes the exclusive lock and the probe, and plans `.vc-x1-running`, a lock
  `vc-x1` takes for every run.

#### Acceptance check

- `vc-x1 validate` passes. Passed, on the pushed work.
- A fresh session started in this workspace while another holds the lock replies "Another
  session holds `.agent-running`, stopping." and its transcript shows the probe as its only
  shell command. Passed, before the push and not run again after it, wink's call: the session
  met the wording that was pushed, `custom.md` not having changed since.

#### Ladder

- docs: a session takes the workspace lock at acquaint (done)

#### Closing

Close-out shape: one commit, the closing folded into the work commit.

- A rule an agent follows is tested by the transcript of a session that meets it cold.
  - The line took four wordings, and each fault showed only in a transcript: a fetch beside the
    lock, a `pgrep`, a `ps` after a reply that read as correct.
  - A session already open tests nothing, having the old wording in mind.
- A count that binds is taken before the name is published.
  - The title was offered at 53 characters and its slug pushed as the bookmark before the count
    was made.
- A validation's verdict was lost once to its own output.
  - The run printed more than the session's tool shows, the last line with it, and it was run
    again into a file under `tmp/`, where the exit status says pass or fail.

# References
