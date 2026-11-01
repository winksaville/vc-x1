# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

**Rule:** a cycle changing `vc-x1` runs its own latest build, wink's (2026-10-30): `vc-x1-dev`,
installed after the rung before, for every command, `validate` and `install` as well as `commit`. The
landed `vc-x1` is used only for a sequence `vc-x1-dev` cannot run, and saying so. Until **install has a role and a
profile** runs, `vc-x1-dev install` needs `--profile dev`, since bare it builds and replaces the landed
`vc-x1`.

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

Written 2026-10-30:

- **feat: improve trailers**, renamed from **feat: ochid trailers are URLs**, is closed and not landed,
  and `main` is still `8be6ff7c` in both repos. Its bookmark is `feat-improve-trailers`, and `origin`
  still has `feat-ochid-trailers-are-urls` at the old closing, `cd16d37d`. The plan, each step wink's go:
  - Force-push: `vc-x1-dev push -s w --bookmark feat-improve-trailers`, and `delete-bookmark -s w
    --bookmark feat-ochid-trailers-are-urls --skip local`.
  - Do not commit, squash, or `squash-push` the agent repo before the Land. Every work commit names
    its `@`, `qrxovvvqxnrq`, and the Land commits it.
  - Land: `vc-x1-dev land -s b --onto main --dry-run`, `land -s b --onto main`, `push -s b --bookmark
    main`, `delete-bookmark -s w --bookmark feat-improve-trailers --skip local`, and `vc-x1 install`.
    After the push, `validate-desc -s b` for this cycle, where every `ocid:` has its SHA.
- Next after that: **install has a role and a profile** and **desc takes a title, body, and trailers**.
- `tmp/mistake-aside-todo.patch` and `tmp/mistake-aside-zDOING.md` hold **chore: carry setting a
  mistake aside into v0.3.1**, set aside when **WIP: cleanup** was dropped. They are gitignored and
  on this machine alone. The patch's context has moved, so its edits may need
  reapplying by hand: **Setting a mistake aside** added to **agent-files(proposal): v0.3.1**,
  **abandon and revert set a mistake aside** deleted from `## Todo`, and a reread note on
  **squash-push shows what it takes, and push follows jj**.
- `tmp/wflow.sh`, `tmp/wflow2.sh`, and `tmp/wflow3.sh` are the trials behind the matrix in **commit works at every scope, and --local keeps it on the
  machine**.

## In Progress

_No cycle currently in progress._
