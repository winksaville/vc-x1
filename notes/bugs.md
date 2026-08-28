# Bugs

Known defects we're aware of but haven't scheduled a fix for.
Each entry describes what goes wrong, when, and the cost of
the failure. Entries are numbered (`1.` `2.` ...) the same way
as `## Todo` in `TODO.md`. Run
`vc-x1 fix-todo --no-dry-run notes/bugs.md` to renumber after
insert / delete / reorder.

## Bugs

1. **`push` `bookmark-set` races the git index lock.**
   `jj bookmark set` on the colocated work repo failed twice
   (the 0.69.0-3 and 0.69.0-4 pushes, same stage) with
   "Failed to reset Git HEAD state … could not acquire lock
   for `.git/index` … after 1 attempt(s)". The lockfile was
   already gone on inspection seconds later. Seen again at
   the 0.75.0-2 push (which also triggered #3) and the
   0.77.0-0 push (2026-07-28), four occurrences, always at
   `bookmark-set`, always transient.
   - **Cost:** push aborts mid-flow (rollback restored both
     repos cleanly both times), and recovery is a `--restart`
     rerun, which succeeded both times.
   - The bot thinks a git-aware watcher (shell prompt,
     editor) briefly re-reads the repo after the commit
     stages touch `.git`, holding the index lock exactly
     when `bookmark-set` resets git HEAD, and jj gives up
     after a single attempt.
   - **Fix direction** (two options, possibly combined):
     - retry with short backoff around the bookmark-set
       stage (or all jj invocations that reset git HEAD
       state)
     - use jj-lib in-process instead of spawning `jj`
       commands (suggested 2026-07-15). The lock
       contention is external, so this alone doesn't
       remove the race, but the retry loop becomes ours
       (catch the lock error, back off, retry) with real
       error types instead of stderr parsing. This is the
       refactor program's
       [jj-lib migration stage](refactor-20260716.md#stage-jj-lib-migration)
   - **Fixed at 0.78.0-8**, both options combined as
     predicted: the session's colocated git writes (HEAD and
     index reset, ref export) run inside `retry_git_lock`,
     which classifies by walking the error's source chain for
     the typed `gix::lock::acquire::Error` (never a message
     substring) and retries with a doubling backoff, about
     375 ms in total, before giving up. Only git writes that
     precede the operation commit are wrapped, so a retry
     never doubles an op-store write. Pinned by
     `mutation_survives_transient_index_lock` (a planted
     `.git/index.lock` released by a thread mid-backoff).

2. **stdout output panics on a closed pipe (EPIPE).**
   `vc-x1 bot-session <file> | head` panics once `head`
   closes the pipe: the logger's `println!` aborts with
   "failed printing to stdout: Broken pipe". Repo-wide
   behavior of the `info!` -> `println!` path, but
   bot-session (0.70.0-2) is the first subcommand whose
   output routinely feeds a pager/filter. Found during
   0.70.0-2 verification.
   - **Cost:** ugly panic + backtrace hint instead of the
     Unix-conventional silent exit, and output before the
     pipe closed is intact.
   - **Fix direction:** handle EPIPE in the logger (write
     via `writeln!` to a locked stdout and exit 0 on
     `BrokenPipe`), or reset SIGPIPE to default on unix at
     startup.

3. **`push` resume-after-rollback replays from the wrong
   stage.** Observed at the 0.75.0-2 push (2026-07-23): the
   `push` `bookmark-set` git-index-lock race fired, the error path
   `op restore`d both repos, undoing `commit-work` /
   `commit-bot`, but the state file still said
   `stage = bookmark-set`. The rerun resumed there,
   *skipping the commit stages*: bookmark-set pinned the
   bookmarks to `@-` (the **previous** cycle commit),
   push-work no-op re-pushed it, and squash-push-bot
   squashed the accumulated session data into the
   already-published previous bot commit and republished
   it. The completion sanity check caught the chid
   mismatch and warned, but after the damage.
   - **Cost:** no data loss (work `@` kept the uncommitted
     changes, and the bot chid is rebase-stable so the ochid
     pairing survived), but session data landed under the
     previous commit's title, and the published bot commit
     was rewritten in place.
   - The state file and the op-restore rollback disagree
     about where the run stopped: rollback rewinds the
     *repos* to pre-commit, but not the *state* to the
     `message` stage. Any `Err` between `commit-work` and
     the stage save has the same shape.
   - **Fix direction:** on rollback, rewind (or delete) the
     state file in the same breath, or retire the state
     file entirely and derive resume from repo reality,
     which is the refactor program's
     [stateless push stage](refactor-20260716.md#stage-stateless-push).
     This incident is its strongest evidence yet.
   - **Second occurrence, 0.77.0-0 push (2026-07-28):** the
     same #1 lock race, the same shape, caught before any
     damage this time. The rollback was clean (both repos
     back to pre-commit, `@` holding the uncommitted changes,
     nothing published, the failure being two stages before
     `push-work`), and the state file still read
     `stage=bookmark-set`. A plain rerun would have set both
     bookmarks to the *previous* cycle's commit and squashed
     this session's data into the already-published bot
     commit. `--restart` is the safe rerun until the fix
     lands. Noted while opening the very cycle that fixes it.

4. **`push` `commit-work` commits an empty `@`, minting a
   duplicate stamped commit.** Observed at the 0.76.0-1 push
   (2026-07-27): the work commit had been made by hand before
   invoking `vc-x1 push`, so `@` was empty. `commit-bot`
   skips a clean repo, but `commit-work` committed the empty
   `@` anyway with the supplied `--title`/`--body`. The
   result: an empty duplicate of the real commit on top of
   it, the ochid trailer stamped on the duplicate (push
   stamps only the topmost commit), the bookmark pushed at
   the duplicate, and the bot commit's `ochid:` pointing at
   the duplicate instead of the real commit.
   - **Cost:** no data loss, but published history needed a
     dual-repo repair: describe + abandon + sideways
     force-push on both sides (`--ignore-immutable`).
   - **Fixed at 0.77.0-2:** `commit-work` now skips an empty
     `@` exactly as `commit-bot` does, and `stage_message`
     resolves the work chid from `@-` when `@` is empty, so
     the bot's trailer names the real commit. Skipping rather
     than erroring, because a legitimately empty `@` is the
     publish-only case: the trapezoid recipe's last step has
     the commits already made and only the bookmark and remote
     left to advance. Push does not rewrite a description it
     didn't author, so a hand-made commit keeps its message
     and simply carries no work-side trailer, and
     `validate-desc` / `fix-desc` add one. Pinned by
     `push_empty_work_at_skips_commit_work`.

5. **`validate-desc` / `fix-desc` error when run against the
   bot repo.** `vc-x1 validate-desc --repo .claude` (or run
   with cwd inside `.claude`) fails with "workspace
   incoherent: ... `repos.work` resolves to ..., not to the
   workspace root itself". Found 2026-08-01 at the 0.78.0-6
   review.
   - **Cost:** the bot-side halves of validate-desc and
     fix-desc are unusable without the workaround, and no
     data is touched (the coherence check stops before any
     action).
   - The prelude resolves the counterpart repo with
     `common::bot_repo_path(&params.repo)`, which answers
     "the bot side of the workspace rooted at this path" and
     so assumes the target is the work side. Against the bot
     dir it asks for the bot-of-bot: `.claude`'s config says
     `repos.bot = "."`, the coherence check then runs with
     root = bot = `.claude`, and its self-identification
     step correctly refuses. The check is right, and the
     caller hands it the wrong root.
   - Introduced at 0.75.0-1 (`refactor: topology por
     equalization`, 2026-07-23): the replaced
     `other_repo_from_config` read the target repo's own
     config, side-aware by construction, while `bot_repo_path`
     lost the "other side relative to me" semantics, and the
     coherence check (added the same day) makes it loud.
   - **Workaround:** `--other-repo` bypasses the resolution:
     `vc-x1 validate-desc --repo .claude --other-repo .`
     from the workspace root.
   - **Fix direction:** make the prelude side-aware again in
     both commands: `find_workspace_root_from(params.repo)`
     for the real root, then `is_bot_dir(params.repo)` picks
     the counterpart (bot side -> work root, work side ->
     `bot_repo_path(root)`, POR -> no-op as today).
   - The fix must be pinned by tests on a dual fixture, both
     entry angles for both commands: target the bot dir via
     the repo flag, and default-repo `.` resolved from inside
     the bot dir, and each asserts the counterpart resolves to
     the work root and the command succeeds.

6. **`init` prints steps out of order.** Reported by iiac-perf + wink (mailbox, 2026-07-31),
   observed on a real 0.71.0 run: output order was `Step 6 (skipped)`, `Step 8 (skipped)`,
   `Step 7: Setting code bookmark`, while `--dry-run` lists 1-11 in order. They think the
   skip-notices are emitted eagerly.
   - **Cost:** cosmetic, though a transcript is ambiguous about what ran when.

7. **`push --body` rejects a body whose first character is `-`.** Hit twice at iiac-perf
   (dogfood log, 2026-08-01): once at vc-x1's own clap (worked around with `--body=`), then
   again inside push's `jj commit -m <body>` (same clap leading-hyphen rejection in jj), which
   rolled both repos back cleanly.
   - **Cost:** a file-by-file body opening with its first bullet cannot be pushed.
   - **Workaround:** open the body with its intro line, never a bare bullet, which prose form
     wants anyway.
   - **Fix direction:** pass bodies to jj as `-m=<body>` or via stdin/file. The `-6` jj-lib
     mutations migration may have retired the jj half already, and the clap half is ours
     either way. Verify both before closing.

8. **`push` adopts a stale state file from an earlier invocation and resumes at its final
   stage.** Reported by iiac-perf + wink (mailbox, 2026-08-06), observed on `vc-x1` 0.71.0:
   three consecutive runs of a fresh `vc-x1 push <bookmark> --title ... --body ...` each began
   directly at `squash-push-bot`, skipping every earlier stage. The work repo was never
   committed (surfaced only as a completion warn), and each run squashed the new session's
   transcript into the previous cycle's already-published bot commit, and run 3 force-pushed it
   sideways (roughly 3.4 MB under the old cycle's title and ochid). Run 4, state cleared,
   landed the pair correctly.
   - **Cost:** a silently skipped work-repo commit, and a published bot commit rewritten
     sideways with the wrong session's content. Permanent residue in iiac-perf's bot repo,
     with the decoder below.
   - Mechanism, confirmed in the 0.71.0 source: the state file (`.vc-x1/push-state.toml`) is
     cleared only when a `vc-x1 push` run completes all stages, and a run that dies mid-flow
     leaves it behind, by design, for resume. The next invocation adopts any existing state
     unconditionally: the positional bookmark and `--title`/`--body` are ignored (state
     carries its own copies), and the run resumes at `state.stage`. `verify_state_sanity`
     passes because stale-but-self-consistent state describes exactly the world the completed
     previous cycle left behind, so nothing distinguishes "resume" from "new cycle".
   - We think the state survived the previous cycle because that cycle's final transfer was
     completed out-of-band: their sandboxed shell drops multi-MB pushes mid-transfer, so a
     rerun from wink's shell or a manual jj push finished it. No `vc-x1 push` run ever reached
     the state-clearing epilogue, and the out-of-band completion made the world match the
     stale state exactly, defeating the sanity check. Wink confirms this fits the cycle's
     events.
   - Refined at 0.78.4 (`test: Claude Code can complete a cycle`): the size correlation was
     coincidence. A sandboxed session cannot use ssh at all, having no readable auth key and
     no route out on port 22, so a large transfer dying where a small one survived was ssh
     failing at two different points, not a size ceiling. Closed 2026-08-07: iiac-perf
     confirms (mailbox) their remotes were also cloned over ssh, and wink repointed both at
     https before their next push, so ssh explains their incident too.
   - Same family as #3 (state file and reality disagreeing), different trigger: #3 is
     rollback rewinding the repos but not the state within one failed run, while this is state
     legitimately outliving a run and a later, unrelated invocation adopting it.
   - **Fixed by design at 0.77.0-3** (`refactor: drop push state and preflight`): the state
     file, resume, and preflight are deleted, every stage checks its own precondition and
     no-ops when its work is already done, the per-run `Run` struct lives one process, and
     `verify_completion` compares the remote against the chids this run actually committed,
     closing the vacuous "landed on remote" pass. The report's first two suggestions (key
     state to the invocation, and refuse to squash into a remote-existing commit this run did not
     create) are satisfied structurally: there is no state to key, and `squash-push-bot`
     targets the bot commit this run just made. The side note about the review gate
     auto-passing on non-tty stdin is also 0.71.0-only: current `stage_review` errors on a
     non-tty unless `--yes` is passed.
   - Still open, tracked here: `verify_completion` remains warning-only (their third
     suggestion, defensible now that it checks this run's own chids, but worth a look).
     iiac-perf's 2026-08-07 view concurs: with no state to be stale, the check is on this
     run's own work, and warning-only is defensible. The incident also triggered eliminating
     vc-x1's one remaining cross-invocation state file, `sync-state.toml`, and removing
     `revert` (0.78.3).
   - **Ochid-chain decoder (iiac-perf residue):** their bot commit e89957e6
     (`docs: experiment in the local agent-files`) carries most of the session that reasoned
     out `docs: steps are titles, versions are stamps`, while the correctly paired bot commit
     bb97240e holds only the session's tail. Anyone walking that ochid chain should look one
     bot commit earlier.
   - **Remedy for iiac-perf:** the fix ships in 0.77.0 and later. The triage decision
     (switch to `vc-x1-dev`) was superseded 2026-08-07: they upgraded to `vc-x1` 0.78.4
     instead, on wink's design argument that resuming after an error is precisely what is
     not wanted, so stateless push with per-stage idempotence is the right shape.

9. **`config --validate` reports "I gave up" as a finding, and one abort path contradicts
   the function's own contract.** Found by reading, not by a run (wink + bot, 2026-08-12),
   while triaging iiac-perf's binary/schema-skew note. `validate`'s doc comment states the
   intent plainly: a side's coherence failure "is reported as a finding, not a hard error, so
   the work-side report still lands". Two paths break or blur it.
   - **Cost:** `config: 1 problem(s) found` means either "a key is misspelled" or "I cannot
     tell which of your two config files is real", and nothing in the output or the exit
     status separates them. A reader re-reads the warning and recovers, and a script cannot.
   - **The abort:** `validate_file` loads via `load_file(path)?`, so malformed TOML or an
     unclosed fence propagates out of `validate` and kills the run, which is exactly the hard
     error the contract disclaims. The summary line never prints. With the default `work,bot`
     target the work-side warnings have already printed, so only the tally is lost, and with a
     `bot,work` target the run dies before the work side is examined at all.
   - **The blur:** a both-carriers side (`vc_config_path` erroring) is counted as
     `findings += 1`, one problem alongside the misspelled keys, though it means the
     validation that follows it is meaningless.
   - The legacy-`[workspace]` path is the same species handled correctly: it returns early
     with a single finding *and* a comment explaining that the remaining checks would be
     redundant. It reached the right behavior without the vocabulary to name it.
   - **Fix direction:** sort every outcome into "checked, found something" versus "could not
     check", report the two differently, and never let the second kill a side that could
     still be reported. That classification is also the whole input to the tiered exit code
     at `## Todo`'s **Tiered exit status for `config --validate`** (#5), which becomes a
     rendering of it rather than new work.

10. **`init` rejects a pre-created GitHub repo that every other host requires.** Preflight
    on a `github.com` URL errors with "GitHub repo '<slug>' already exists" (wink,
    2026-08-21, `vc-x1 init https://github.com/winksaville/t4-vc-x1` against an empty repo
    created moments before), while a non-GitHub URL takes the `ExternalPreExisting` path
    and can only work if both remotes were pre-created. So the rule a user learns is
    "pre-create, unless GitHub, where you must not".
    - **Cost:** a user who pre-creates on GitHub, the habit every other host trains, is
      stopped at preflight with no hint that deleting the empty repo is the way through.
    - **Fix direction:** test emptiness rather than existence. Missing: provision as today.
      Existing and empty (`gh repo view <slug> --json isEmpty`): skip the create step and
      push into it, the same as the non-GitHub path. Existing with commits: keep the error,
      saying "has commits" so it reads as protection. Check both slugs, since `<name>` and
      `<name>.claude` can differ. Rides the "Drop the global config and the account notion"
      Todo entry, which reshapes init's remote surface.

11. **`init` cannot publish to gitlab.com: both repos are created locally, then nothing is
    pushed.** With a gitlab.com URL (wink, 2026-08-21, `t1-vc-x1`) init built and committed
    both sides, then the push failed: GitLab does not create a project on push, so the
    remote was "not found" and the local tree was left behind for the user to delete.
    - **Cost:** no GitLab support in practice, and the leftover directory makes a rerun stop
      at preflight with "already exists".
    - **Fix direction:** provision the two projects before pushing, through the GitLab API or
      its CLI (`glab repo create`), as a gitlab.com arm beside the `gh` one. Rides the "Drop
      the global config and the account notion" Todo entry with #10.

12. **A slashed TARGET with no path prefix was read as the retired `owner/name` shorthand.**
    `vc-x1 init tmp/vc-x1-dev.0.80.6-2` (wink, 2026-08-28) created the work directory at the
    repo root rather than under `tmp/`, then asked GitHub to create the repo in an
    organization named `tmp`, which failed with "winksaville does not have the correct
    permissions to execute `CreateRepository`". `parse_target` classified any single-slash
    string as the shorthand, and only `./X` was read as a path.
    - **Cost:** a plausible relative path silently became a request against someone else's
      namespace, and the error named permissions rather than the misreading, so the cause was
      invisible from the message.
    - The two readings are undecidable on syntax: nothing needs to exist for a path target,
      since init creates missing parents, so `tmp/foo` is a well-formed path and a well-formed
      `owner/name` at once.
    - **Fixed** by "fix: init takes a URL or a path": the shorthand is retired and a slashed
      target with no path prefix is refused, naming both readings. Once nobody reaches for the
      shorthand, a slashed target can simply mean the path.

13. **A `.git` directory name became a GitHub repo name, and GitHub renamed it out from under
    the remote.** `vc-x1 init ./tmp/xx1.git` (wink, 2026-08-28) asked for `winksaville/xx1.git`
    and `winksaville/xx1.git.claude`. GitHub drops a trailing `.git` at creation, so the work
    repo it made was `xx1`, while init wrote the remote `git@github.com:winksaville/xx1.git.git`.
    The first push then failed with "Could not read from remote repository ... make sure you
    have the correct access rights and the repository exists", the second half being the true
    one.
    - **Cost:** init leaves a local workspace, two GitHub repos, and a remote that points at
      neither, and the message reads as an auth failure. It cost this cycle a wrong diagnosis
      before the repo listing settled it.
    - **Cause:** `plan_from_url` normalized the name through `derive_name`, which strips
      `.git`, and `plan_from_path` took the directory's `file_name` verbatim. Two branches,
      two answers to one question.
    - **Fixed** by "fix: init takes a URL or a path": both branches derive the name the same
      way, and `github_slug_from_url` refuses a name GitHub would rename rather than asking
      for it.

14. **`init` writes ssh remotes.** A path or bare-NAME target resolves its remote through the
    user-config chain, which yields `git@github.com:<owner>` shaped prefixes, so a workspace
    init creates pushes over ssh while `gh` works over https with a token.
    - **Cost:** a push failing at the network leg looks like a missing repo, and the working
      practices say to check for an ssh remote first precisely because it is hard to tell
      apart.
    - **Fix direction:** unsettled, and larger than it looks, since the remote scheme is the
      user config's to state. Rides "Drop the global config and the account notion", which
      deletes that chain, with #10 and #11.

# References
