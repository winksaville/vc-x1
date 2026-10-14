# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### chore: the sandbox's names leave .gitignore

(wink + agent, 2026-10-12, picked 2026-10-14) Claude Code's sandbox puts empty files at a
workspace's root while an agent's command runs, `.bashrc`, `.gitconfig`, `.mcp.json`,
`.claude/settings.json`, and some twenty more. A snapshot taken from outside that sandbox at that
moment sees them as new files, and a push commits them: 23 went into a commit in the two-session
tests ([session-lock.md](notes/session-lock.md)). The Todo entry picked is "init writes the agent
sandbox's names into .gitignore":

- This repo's `.gitignore` names them, added by hand, and the one `init` writes has four lines
  and none of them.
- `init` writes the names, and a workspace made before it does gets them by hand or by a command
  that brings a workspace up to date.
- The list is Claude Code's and can change with a release, so where it is kept, and how it is
  checked against what the sandbox does, is settled in the cycle.
- We think a user's own `jj st` beside a running agent command sees the files too. Not tested.

#### Deliberation

- The names are ignored nowhere, wink's lean and the cycle's finding: an ordinary command leaves
  nothing jj records.
  - Inside the sandbox a protected name that does not exist is `/dev/null` mounted for the life of
    the command, a character device, and jj snapshots no device. A `jj st` run there with the
    names unignored listed only the cycle's edits.
  - Tested with wink watching Zed: a sandboxed command held for 20 seconds showed the eleven root
    names for exactly that long, and the same command unsandboxed showed none.
  - Seen in the sandbox's own arguments: `bwrap` mounts `/dev/null` over each missing protected
    path by name, before it starts the shell.
  - What the ignore hid in this repo were real files: 24 left on disk at 2026-10-11T21:09:22Z by
    the session lock's holder, a background command ended with its session, so the cleanup an
    ordinary command gets did not run. wink deleted them, 2026-10-14.
  - The cost accepted: a snapshot or commit made outside the sandbox while an agent command runs
    sees the files. wink's editor listed them for some seconds, twice, during the cycle's edits.
  - The cost avoided, wink's reason: an ignored name hides a file a user makes there. It is never
    committed, and nothing says so until a clone comes up without it. The flicker is surprising
    and visible, the loss is silent.
  - Weighed and not taken: `init` writing the list, and the list in each user's global git ignore.
    The list is Claude Code's, changes with a release, and names files a project may track.
- The direction was reversed twice before it settled. A probe with the leftovers still on disk
  showed all eleven root names in an in-sandbox `jj st`, which argued for the ignore. The
  leftovers' birth times then showed they were not the sandbox's per-command files at all.
- `init` is unchanged: it never wrote the names, so the entry's code half is dropped.
- Cargo's comments go too, wink's call: noise, with `/target` already the file's first line.
- So does `/.agent-running`, wink's call: the lock is off, and turning it on puts the line back.
- The set's copy of the lock instruction learns of the leak in a cycle of its own, being an
  agent-file change: filed as a Todo.
- zc-ring-x1 carries the same list and an older leftover, `.bashrc` of 2026-08-27, and is told
  after Land.
- Single-step, a config edit and its notes.

#### Solution

The sandbox's names are ignored nowhere, and the record says why.

- The project's `.gitignore` loses the eleven names, cargo's comments, and the lock's
  `/.agent-running`, keeping the six entries for the build, the scratch, and the repos.
- [session-lock.md](notes/session-lock.md#the-sandboxs-placeholder-files) records what the
  sandbox's placeholder files are, why one session never meets them, what a killed holder leaves
  on disk, and how to find and delete it. Its decision that blamed the `.gitignore` is corrected,
  and the section on turning the lock off names the holder as the kind of command that leaks.
- The set's copy of the lock instruction is a Todo entry, "The set's session-lock file says a
  killed holder leaves files behind".

#### Acceptance check

- An in-sandbox `jj st`, the names unignored, lists only the cycle's edits. Passed.
- wink's own `jj st`, outside the sandbox, lists only the cycle's edits. Passed, run once the
  leftovers were deleted.
- `vc-x1 validate` passes.

#### Ladder

- chore: the sandbox's names leave .gitignore (done)

# References
