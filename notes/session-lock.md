# The session lock, and what two sessions in one workspace do

The session lock is the exclusive `flock` on `.agent-running` that an agent session holds for as
long as it runs, taken because an instruction in `custom.md` tells the agent to take it. This file
holds what was found out about it on 2026-10-12: what Claude Code's hooks could do in the agent's
place, and what happens when two sessions work in one workspace with no lock at all. The lock's
own tests, of 2026-10-11, are in the landmark `docs: the session lock is settled as flock`.

The short of it: two sessions lose no data and leave a wrong record, the instruction is kept, and
no hook is built.

## What the lock is today

- The agent takes it by hand: `flock -xn .agent-running true` as a probe, then
  `flock -xn .agent-running sleep infinity` as a background command. `vc-x1` has no code that
  touches the lock.
- Nothing releases it. The kernel drops a `flock` when the process holding it exits, and Claude
  Code ends a session's background commands when the session ends.
- It closes no race and needs none closed: a `flock` has one winner whoever asks. A session that
  loses between its probe and its holder learns of it seconds later, when the holder exits, and
  the instruction has it stop then.
- Its weakness is that an agent has to read the instruction and follow it.

## What a hook could do

A hook is a command Claude Code itself runs at a moment in a session, whether or not the agent
reads anything. The documents were read 2026-10-12 (the hooks reference and guide at
`code.claude.com/docs/en/hooks`), and what they do not say was found with a probe: a script
hooked to a session's start, to each prompt, and to its end, logging what it was given and
leaving a watcher process behind. Wink ran it in four sessions under Claude Code 2.1.296: one
exited, one killed with `kill -9`, one whose terminal was closed, one resumed.

- From the documents:
  - A session-start hook fires on a fresh start, a resume, a `/clear`, a compaction, and a fork.
  - It cannot refuse a session. It can add to the agent's context or show a message.
  - A prompt hook can stop a prompt before the agent sees it, and a hook on tool calls can deny
    every call.
  - A session-end hook has a second and a half and cannot refuse.
  - A hook runs in a process session of its own, with no controlling terminal.
- From the probe:
  - A process a hook leaves behind outlives the hook, and Claude Code does not end it.
  - A hook is given `CLAUDE_PID`, the Claude Code process, which is also the hook's parent. The
    documents do not name that variable.
  - The watcher saw the Claude Code process gone within a second or two of each ending.
  - The session-end hook fired on the `/exit` and on the closed terminal, and not on the kill.
  - A `/clear` ends the session and starts another, under a new session id, in the same
    process. A resume keeps the session id in a new process. So the one thing that lasts as long
    as a lock should is the process id.
- The design that follows: a script at a session's start takes the `flock`, writes the process
  id into `.agent-running`, and leaves a holder that exits when that process is gone, and the
  same script at each prompt stops a prompt from a process that is not the owner.
- It was not built, wink's call. It rests on a variable the documents do not name and on hook
  behaviour they do not state, and what it guards against turned out to lose no data.

## What two sessions do with no lock

Three tests, run by wink in a scratch workspace made by `vc-x1 init xxx --repo local=<parent>`,
with two Claude Code sessions, A and B, open in it at once and no agent-files to tell either of
the other. Each result was read from the two repos, their remotes, the transcripts, and jj's
operation log.

- Pushes minutes apart. A wrote `a.txt`, B wrote `b.txt`, A pushed, then B pushed.
  - A's work commit held both files, B's under A's title.
  - B's push found nothing left to commit in the work-repo, said so, and made an agent-repo
    commit under B's title.
  - Two agent-repo commits then named the one work commit, which named A's alone.
- Pushes in the same second, each session's command sleeping to the top of the minute.
  - B's push committed both sessions' edits, pushed the work-repo, and died with
    `error: Concurrent checkout`, its agent-repo stage not run.
  - A's push reported success. Its work commit held none of A's edit, and it made no agent-repo
    commit, taking B's as its partner, so two work commits named one agent-repo commit.
  - B's commit also held 23 empty files, `.bashrc`, `.gitconfig`, `.claude/settings.json`, and
    the rest: the names the agent sandbox puts on disk while a command of A's ran. A's commit
    deleted them.
  - jj recorded "reconcile divergent operations" twice, with no conflict and no divergent
    change. Both remotes ended with everything and the tree was right.
- One session resumed in two terminals, `claude --resume <id>` in each.
  - Both wrote to the one transcript file, eight prompts and replies, two of the prompts in the
    same second, and every line of it parses.
  - The conversation forked without a word: each terminal's messages follow its own, so the
    file holds two branches, interleaved line by line.
- In all three every transcript line parses and nothing either session wrote was lost.
  - A session writes a transcript file named by its session id, so two sessions write two
    files, and a file two processes share is appended to a whole line at a time.
- What goes wrong is the record: a title that does not say what its commit holds, a trailer that
  names the wrong session or one of two, a push that dies with an error that explains nothing,
  and a transcript that reads as two conversations shuffled.

## What was decided

- The lock stays as the instruction in `custom.md`. It costs a line and it prevents all of the
  above.
- No hook, and no code in `vc-x1` for the session lock.
- The failed push and the wrong partner are two `vc-x1` runs overlapping, which is what the lock
  `vc-x1` takes for a run is for ([vc-x1 takes a lock of its own for every
  run](../TODO.md#vc-x1-takes-a-lock-of-its-own-for-every-run)).
- The empty files are a fault of the `.gitignore` that `init` writes ([init writes the agent
  sandbox's names into
  .gitignore](../TODO.md#init-writes-the-agent-sandboxs-names-into-gitignore)).
- Not tested: which branch a resume of a forked transcript takes, what `vc-x1 lookup` makes of
  one, and whether a user's own `jj st` beside a running agent command picks up the sandbox's
  files as the second session's snapshot did. We think it does.
