# custom.md - the project layer

The project's own layer over the other agent-files (see
[AGENTS.md](AGENTS.md#custommd)). Loaded last. On conflict, this file wins.

## Project conventions and overrides

Project-local conventions and overrides of the agent-files. An override names the section it
supersedes.

- Lock: at acquaint, before anything else, a session runs `flock -xn .agent-running true` at the
  workspace root, alone. If it fails, the session responds with the one sentence "Another session
  holds `.agent-running`, stopping." and runs no further command, not even to find the holder. If
  it succeeds, the session starts `flock -xn .agent-running sleep infinity` as a background
  command, leaves it running, and says "Lock taken." If that command ever exits, the same
  sentence and the same stop. A session that already holds one does neither. This keeps a second
  session from working in the workspace, so two cannot push or amend at once
  ([The lock a running agent holds](TODO.md#the-lock-a-running-agent-holds)).
- Messaging: the `../vc-x1-messages` repo. Its `README.md` is the protocol and it governs, and
  a session reads what is pending for us there at acquaint, per its Read messages action.
