# Bugs

Known defects we're aware of but haven't scheduled a fix for.
Each entry describes what goes wrong, when, and the cost of
the failure. Entries are numbered (`1.` `2.` …) the same way
as `## Todo` in `todo.md`; run
`vc-x1 fix-todo --no-dry-run notes/bugs.md` to renumber after
insert / delete / reorder.

## Bugs

1. **`vc-x1 push`: the session-repo push silently never happens
   when the bot runs push sandboxed.** Observed in
   [tprobe](https://github.com/winksaville/tprobe) 2026-07-14:
   both local repos consistent and the GitHub code repo current,
   but [tprobe.claude](https://github.com/winksaville/tprobe.claude)
   stuck 8 commits behind (`jj bookmark list --all-remotes -R
   .claude`: `main@origin (behind by 8 commits)`).
   - **Root cause (diagnosed 2026-07-14).** Push doesn't push
     `.claude` itself; its `finalize-claude` stage spawns a
     detached child that sleeps 10s first:
     - the child — `vc-x1 finalize --squash --push main
       --delay 10 --detach` — is `setsid`'d to outlive push
     - a sandboxed Bash command kills every process in its pid
       namespace at command exit; `setsid` escapes the process
       group, not the namespace
     - the child dies mid-sleep, before its first `jj` command
     - reproduced in this project's sandbox with a bare `setsid`
       child; the bot thinks the same kill explains tprobe,
       whose sandbox is always-on (`autoAllowBashIfSandboxed:
       true`, `allowUnsandboxedCommands: false`) with
       `vc-x1 push *` permission-allowed
   - **Evidence.**
     - tprobe app repo: 8 `push bookmark main` ops — the
       in-process `push-app` stage worked; network was fine
     - tprobe `.claude`: zero squash/push ops since clone — the
       child never ran even one command
     - tprobe `.vc-x1/` state dir empty — every push "completed
       all stages"
   - **Why it's silent.**
     - SIGKILL — the child never writes a failure marker
     - marker writes to `~/.cache/vc-x1` are outside the sandbox
       write allowlist anyway; `write_failure_marker` swallows
       errors by design
     - `verify_completion_sanity` checks local bookmarks only
     - related: `CliLogger::new` panics (`.expect`) when the
       `--log` path can't be opened — a sandbox denying
       `/tmp/vc-x1-finalize.log` kills the child at startup
   - **Side effect:** the trailing-data squash never runs; the
     tail rides into the next cycle's session commit (misplaced,
     not lost).
   - **Fix direction:** see the Todo "push: inline the
     session-repo push; finalize becomes the user's empty-@
     tidy-up".
   - **Recovery** for an affected project: `jj git push
     --bookmark main -R <proj>/.claude` from an unsandboxed
     shell (or any in-process command that finishes before its
     sandbox exits).

2. **`finalize` squash silently drops the source journal's
   ochids.** `finalize_exec` (`src/finalize.rs`) always runs
   `jj squash --ignore-immutable --use-destination-message
   --from <source> --into <target>`, so when the source
   commit's message carries `ochid:` trailers the squash
   discards them with the rest of the message — the
   destination's message wins unconditionally.
   - **When it bites:** the squash source (`@`) is a
     *described* journal instead of a bare trailing-data
     snapshot. Observed sequence (fc op log, 2026-06-08):
     the bot wrote the journal message via
     `jj describe wyrlsuusnyzz` while that change was still
     the uncommitted `@`; finalize then squashed
     `--from @ --into @-` where `@-` was the previous,
     already-pushed journal — destination message won,
     journal message + 6 ochids discarded. Finalize's
     assumption (`@` is disposable, `@-` holds the real
     message) inverts whenever the journal is described on
     `@` rather than committed first.
   - **Cost:** every code-side commit pointing at the
     squashed journal is left with a dangling
     `ochid: /.claude/<chid>` — the code↔session cross-link
     breaks. Recovery requires op-log surgery on the machine
     that still has the original object (this happened in the
     fc project: journal `wyrlsuusnyzz`, 6 dangling app
     ochids, recovered 2026-06-10 from a backup workspace's
     op log plus a `.claude main` force-push).
   - **Fix direction:** before squashing, detect `ochid:`
     trailers in the source message that the destination
     lacks; refuse (or merge the trailers into the
     destination message) instead of dropping them.

3. **`finalize::surface_previous_failures` is racy and
   bounded by "next vc-x1 run".** The current model has
   four gaps:
   - **Stale forever.** Markers sit on disk until the
     next `vc-x1 <anything>` runs. If the user abandons
     the workspace (e.g., CI / scheduled use), failures
     are never surfaced.
   - **Concurrent surface_previous_failures.** Two
     `vc-x1` runs racing: both `read_dir`, both print
     the marker, one deletes (the other's `remove_file`
     silently fails). The user sees the same failure
     printed twice.
   - **Mid-write torn read.** A `finalize --exec` child
     writing a marker while a sibling `vc-x1` is
     surfacing could read partial content. Atomic-rename
     on write would close this.
   - **No notify-at-failure path.** A detached
     `finalize --exec` failure only becomes visible when
     the user next runs *any* `vc-x1` command — fine for
     interactive use, invisible for CI / scheduled use
     where there may be no next run.

   The exec-child gate in `main.rs` (since 0.52.0-3)
   patches one related case — the detached child eating
   its own prior markers before the user can see them —
   but doesn't address any of the above. Holistic fix
   needs locking + atomic writes + maybe a
   notify-at-failure path for hands-off use.

4. **`init --repo local` bare remotes keep HEAD at
   `refs/heads/master`.** The only branch pushed is `main`,
   so a later `jj git clone` of that bare repo has no default
   branch to auto-track and `vc-x1 clone` fails its
   `verify_tracking` check ("bookmark 'main' has non-tracking
   remote 'main@origin'"). Found building `tests/cli_sync.rs`
   (worked around there with `git symbolic-ref HEAD
   refs/heads/main`).
   - **Fix direction:** init's local bare provisioning sets
     HEAD to `refs/heads/main` at creation.

5. **`clone` session-remote derivation mismatches init's
   local naming; relative TARGET breaks the session clone.**
   Two related defects, both found building
   `tests/cli_sync.rs`:
   - `derive_session_url` maps `<x>.git` →
     `<x>.claude.git`, but `init --repo local` names the
     session remote `remote-claude.git` — so a dual clone of
     a locally-init'd project's `remote-code.git` looks for
     `remote-code.claude.git` and fails.
   - A relative local-path TARGET fails on the session side
     regardless: `clone_dual` runs the session `jj git
     clone` with the just-cloned code repo as cwd, so the
     relative source no longer resolves. Workaround: pass an
     absolute TARGET.

# References
