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

- 2026-10-22, the closing **feat: vc-x1 install stages, deploys, and runs builds closing** is
  pushed at 0.85.7, the merge of `main` and the run rung made by `commit -s b --closing`, its
  record moved to `zCLOSED.md`. Land waits for wink's go.
- Land, by `jj.md`'s sequence under the bend: sync `-s b`, a moved `main` stopping it; no name to
  restore and nothing to reshape, the closing being the trapezoid; the fast-forward, `vc-x1
  move-bookmark -s w --bookmark main --target <closing>` or `jj bookmark set main`, and its push;
  the install from `main`, `cargo run --locked -- build`, `run -- -V`, and `install`, the
  acceptance check's last clause, its report to show `main` and `main@origin`, clean, and a bare
  0.85.7; then the bookmark's deletion, locally and remotely.
- Binaries: `~/.cargo/bin/vc-x1` is wink's test install of 0.85.7-6 (2026-10-22, 15:03), state
  `unknown`, and Land's install replaces it with 0.85.7. There is no `~/.cargo/bin/vc-x1-dev`.
  `cargo run --locked -- <command>` rebuilds `target/debug/vc-x1` when it is missing.

## In Progress

_No cycle currently in progress._
