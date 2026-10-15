# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

- Start with the Todo [Every repo ignores Claude Code's
  settings.local.json](TODO.md#every-repo-ignores-claude-codes-settingslocaljson), the top entry,
  as its own cycle. This commit is on the bookmark `next-time`, off `main`, and carries only the
  entry and this note: land it, or carry the two edits into that cycle's opening, the user's call.
- The work-repo's `.claude/` is read-only (`dr-xr-xr-x`), set 2026-10-14 to test which setting
  wins, and holds a `settings.local.json` that turns the sandbox off for this project. The local
  file beat the global `enabled: true`. With `.claude/` read-only, `/sandbox` fails with EACCES,
  and a sandboxed command cannot remove its placeholder files, so they leak. To restore, outside
  the sandbox: `chmod u+w .claude`, then `rm .claude/settings.local.json`, and the global
  settings govern again. Ask the user before either.

## In Progress

_No cycle currently in progress._

# References
