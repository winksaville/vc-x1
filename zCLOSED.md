# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### feat: every repo ignores Claude Code's settings.local.json

(wink + agent, 2026-10-14, picked 2026-10-15) Claude Code writes `.claude/settings.local.json` in
the project directory, its local project settings, one user's for this project: `/sandbox`
writes it there on its first toggle, and we think a permission prompt's "don't ask again" adds to
it. Its content belongs to one user and one history, the sandbox's on or off among it, so it is
never committed. iiac-perf's
agent-repo is `.claude`, and there the file is tracked and pushed. The shared `settings.json`
beside it stays visible. One line in each repo covers every layout, nested, external, or single:

- Work-repo, and a single-repo workspace: `/.claude/settings.local.json`. `init` writes neither
  today (`render_work_gitignore` and `GITIGNORE_APP_ONLY` in `src/init.rs`), and vc-x1's own
  `.gitignore` drops its whole-directory `/.claude` for it, so anything else Claude Code writes
  there shows in `jj st` and is reviewed.
- Agent-repo: `/settings.local.json`, which matters only when the agent dir is `.claude`, and
  harms nothing otherwise. `init` writes it into `GITIGNORE_SESSION`, and vc-x1's
  `.agent-session/.gitignore` takes it.
- Decide at the opening: whether `.cc-writes/`, the empty mode-700 directory the sandbox's first
  command makes beside it, gets the same pair (`/.claude/.cc-writes/`, `/.cc-writes/`).
- Record the findings in [session-lock.md](notes/session-lock.md#the-sandboxs-placeholder-files):
  `.claude/` is made on demand, not at launch, `.cc-writes/` by the sandbox's first Bash
  command and `settings.local.json` by `/sandbox`, both tested 2026-10-14 in fresh empty
  directories.
- Later, not this cycle: a warning, at acquaint or in `validate`, when the file exists.
- Not this cycle: a repo that already tracks the file also needs `jj file untrack`. For iiac-perf
  that is a message to its agent.

#### Deliberation

- The rules bend by wink's waiver of 2026-10-15: "permission granted to complete the change and
  commits and ready it for landing on main".
  - It covers the work review, the description review, and the approval of each push, the
    bookmark's included, through the close-out.
  - Land is not inside it, and waits for wink's go.
- The cycle carries `next-time`'s commit, which held the Todo entry and the continuation note,
  wink's call: the bookmark is renamed to this cycle's, and the commit stays under it.
  - The commit was published, and its pair on the agent-repo's `main` carries a trailer naming
    it. Carrying it keeps the pair, where redoing its two edits here would leave that trailer
    naming an abandoned commit.
  - The cost accepted: the bookmark holds two commits, and the fast-forward at Land takes both.
  - Folding it into the cycle's commit, for one rung, was tried and undone, wink's call: the
    trailer naming it would have named nothing.
- The file was first called per-machine here, from the Todo's wording. It is Claude Code's local
  project settings, one user's for this project, and the text says so.
- Single-step: config lines, the `init` renderers with their tests, and the notes, one solution.
- `.cc-writes/` is ignored nowhere: jj and git record files, never directories, and it holds none.
  Should Claude Code ever write a file into it, the file shows in `jj st` and is reviewed, which
  is why the work-repo's ignore narrows to one file.
- The agent-repo's tracked `settings.local.json` is deleted, wink's call. Claude Code reads
  project settings from the project's `.claude/` only, and the agent dir left that name on
  2026-09-28. The file's last change was 2026-08-28, its rules name `.claude` as the agent-repo,
  and no code or agent-file reads it.
- `.agent-session/.claude/`, an empty `.cc-writes/` of 2026-10-09 and nothing else, was deleted
  at acquaint, wink's go. Untracked, so no commit carries it.
- The second continuation note was stale at acquaint: `.claude/` is writable again and its
  `settings.local.json` turns the sandbox on, so its restore steps are moot. Reset with the first.

- An adopted directory's `.gitignore` gets the settings line too, beside the agent directory's,
  each added only when missing: an adopted workspace is the likeliest to hold the file already.
- The placeholders the sandbox mounts under `.claude/` while a command runs are no longer hidden
  from a snapshot taken outside it, the cost the cycle `chore: the sandbox's names leave
  .gitignore` accepted for the root names, accepted here for the same reason. Inside the sandbox
  they are devices jj never records, and the work-repo's `jj st` listed none.
- The warning when the file exists, deferred by the entry, is filed in the backlog as "A warning
  when a repo tracks Claude Code's settings.local.json". iiac-perf's untrack stays a message for
  wink to send or have sent.
- The agent-repo's `settings.local.json` was gone from disk before the cycle's `rm` ran, by a
  hand other than the agent's. The deletion recorded is the intended one.

#### Solution

The project's own repos and every workspace `init` makes ignore `settings.local.json` by one line
each, and the work-repo stops ignoring the rest of `.claude/`.

- The work-repo's `/.claude` narrows to `/.claude/settings.local.json`, and the file, already
  snapshotted into the working copy, is untracked.
- The agent-repo's `.gitignore` takes `/settings.local.json`, and its tracked copy, unread since
  the agent dir was renamed, is deleted.
- `init` writes `/.claude/settings.local.json` into a dual workspace's work-repo and a single-repo
  workspace's `.gitignore`, `/settings.local.json` into the agent-repo's, and adds the settings
  line to an adopted directory's `.gitignore` that lacks it. The tests cover each.
- The README's `init` passages and layout, the `init` notes, and
  [session-lock.md](notes/session-lock.md#the-sandboxs-placeholder-files) record the lines and
  what `.claude/` holds.

#### Acceptance check

- The work-repo's `jj st` does not list `.claude/settings.local.json`, the file on disk, and the
  commit's tree has no path under `.claude/`. Passed.
- The agent-repo's commit has no `settings.local.json`, and its `.gitignore` names it. Passed.
- A fresh `init` in `tmp/`, dual and single-repo, writes the lines, checked by the `init` tests
  and by reading the files. Passed: in the dual workspace a new `.claude/settings.local.json` and
  `.agent-session/settings.local.json` were ignored and a `.claude/other.json` was listed. The
  dual `init` stopped at its last step, the `~/.claude/projects` symlink, which the sandbox
  refuses, after its files were written and pushed.
- `vc-x1 validate` passes. Passed.

#### Ladder

- feat: every repo ignores Claude Code's settings.local.json (done)

Close-out shape: trapezoid, wink's call at Land. The cycle is single-step, but the bookmark holds
the carried `next-time` commit beneath it, so the merge keeps the two as one landing on `main`.

# References
