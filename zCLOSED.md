# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### feat: a hook enforces the shell rules

Not picked from `## Todo`: opened at the acquaint of 2026-10-27, ahead of its top entry, wink's
call. Single-step, turned from a ladder at the opening's review, nothing yet pushed.

#### Problem

`AGENTS.md > Working practices` forbids masking a command's exit status and bundling unrelated
commands into one invocation, and the agent broke both through the session of 2026-10-26, a
trailing `; echo rc=${PIPESTATUS[0]}` on `sync`, `validate`, `install-dev`, and the pushes, and
again at the acquaint of 2026-10-27, three invocations joined by `;`. Nothing fails when it slips,
so the habit holds until the user catches it, and neither rule is in the Rules index, where the
rules an agent reads as hard are listed. `bash-allow.sh`, the user's `PreToolUse` hook, allows any
command starting `cargo`, `jj`, or `vc-x1`, so a masked `cargo test` runs with no prompt.

#### Solution

What was done:

- `agent-data/bash-guard.sh`, a `PreToolUse` hook on Bash, two text matches on the command and no
  parsing, each a deny whose reason is the rule's form:
  - an `echo` of `$?` or `PIPESTATUS`
  - a `cargo` or `vc-x1` command, by any path and `vc-x1-dev` included, piped into `tail`, `head`,
    `grep`, or `rg`, with no `pipefail` in the command
  - Matching hooks run in parallel and the strictest decision wins, so it overrides
    `bash-allow.sh`'s `allow`.
- The set carries the user's two other hooks, `agent-data/bash-allow.sh` and
  `agent-data/inhibit-sleep.sh`, copied verbatim from `~/.claude/hooks/`, and
  `agent-data/hooks.sh`:
  - `check`: each hook in `~/.claude/hooks/` identical to the set's and registered in
    `~/.claude/settings.json`, else what is missing, different, or unregistered, exit 1
  - `install`: copies what differs and registers what is missing, the settings file backed up once
    first, a registration joining the group with its matcher when there is one
  - `test`: `bash-guard.sh` over thirteen commands it must deny or allow
  - `HOOKS_DIR` and `SETTINGS` override the destinations, for a trial install
- In `AGENTS.md`:
  - Acquaint gains step 2, Hooks, `hooks.sh check`, its findings reported for the user to install,
    and the steps after it renumber.
  - Exit status is one form: the command under test bare and last, `set -o pipefail` first when
    piped, the `|| { rc=$?; ... }` form kept for reporting.
  - A Hooks bullet names the three hooks, the install, and what the guard denies.
  - The Rules index lists One command per invocation and Exit status.
- `rationale.md` gives why under Acquaint and Working practices, and **agent-files(proposal):
  v0.3.1** gains **The exit-status rule is one form, and hooks guard it**.
- Carried from the Land of **feat: vc-x1 land** (wink, 2026-10-27):
  - **Continuation notes leave the work-repo dirty after Land** gains the design from that Land.
  - The v0.3.1 proposal gains **Continuation notes go in the closing**, provisional.
  - The v0.3.1 proposal's Land names the candidate, `install --dir`, in place of installing the
    closing with `vc-x1 install`.

#### Acceptance check

`hooks.sh test` passes. `hooks.sh check` fails before the install, naming `bash-guard.sh` missing
and unregistered, and passes after. With the hooks installed, in a session: `false; echo rc=$?` is
denied, `cargo test 2>&1 | tail -3` is denied, `jq -n 1 | grep 1` runs, and `cargo --version || {
rc=$?; echo failed=$rc; exit $rc; }` runs.

Result (2026-10-27, 3900x), before the install: `hooks.sh test` passed its thirteen cases, and
`hooks.sh check` named `bash-guard.sh` missing and its registration absent, the other two hooks and
their six registrations already in place. A trial install into `tmp/hooks-try` with a copy of the
settings registered seven entries with one backup, the `Bash` ones joining the existing group, and
a second run changed nothing. wink ran the install, which put `bash-guard.sh` in and registered it,
and `hooks.sh check` passed. Live, in this session, with no restart: `false; echo rc=$?` and `cargo
test 2>&1 | tail -3` were denied, each with its reason and the rule's form, and `jq -n 1 | grep 1`
and `cargo --version || { rc=$?; echo failed=$rc; exit $rc; }` ran. Passed.

#### Deliberation

- The hooks in the set, in `agent-data/`, wink's call (2026-10-27): the slip is the agent's in
  every project, so the hook is installed in `~/.claude/hooks/` for all of them, not a `vc-x1
  guard-bash`, and the set carries its source so it travels with the rule, to fwlaptop on the next
  sync and to an adopter by `agent-files copy`.
  - Flat in `agent-data/`, since the set is the plain files directly under it and a subdirectory
    would not travel.
  - The other two hooks with it and `hooks.sh` to check and install them, wink's call: one place
    for a machine's hooks, and acquaint finds a machine that lacks them.
- Text matches, not a parser, wink's question at the opening's review (2026-10-27): the agent's
  commands carry heredocs, `jq` and `python -c` programs with `;` inside quotes, and `for` and `if`
  blocks, so a check of joins by `;` or `&&` needs a shell parser, and without one it would stop
  legitimate scripts all day. The two denies need none, and a false hit costs a rewrite.
  - One command per invocation stays a rule alone, indexed and caught at review.
  - An if/else form, wink's alternative, works, `$?` in `else` being the condition's, but would be
    a third form for what the rule already has one for, and fails the same way, by not being used.
    One form, bare, is what the hook checks.
  - `cmd | tail; exit ${PIPESTATUS[0]}`, which the old rule allowed, is denied: one form.
  - A command's text is matched, so a search for the pattern itself, `rg 'echo.*\$\?'`, is denied
    too, and is written another way.
- Documented at https://code.claude.com/docs/en/hooks: the events and when each fires, the
  `matcher`, the stdin JSON with `tool_input.command`, and under PreToolUse decision control the
  precedence `deny` > `defer` > `ask` > `allow` and that a hook's `allow` does not override a deny
  or ask rule.
- The install is the user's in practice too: the agent's run of `hooks.sh install` was denied by
  Claude Code's auto-mode classifier as self-modification, a write to the settings every session
  reads.
- Late finding on **feat: vc-x1 land**, landmark `8a2cf954`: its acceptance check passed. After the
  Land, `tmp/candidate/vc-x1 install` installed commit `8a2cf954d98e`, clean, and `cmp
  tmp/candidate/vc-x1 ~/.cargo/bin/vc-x1` found no difference, so the binary tested is the one
  installed (fwlaptop, 2026-10-27, in the agent-repo's session `9b45dc9f`). 3900x found `main` at
  the closing in both repos and on origin and the three bookmarks gone.
- `notes/README.md` names no hooks, so it is unchanged.

#### Ladder

- feat: a hook enforces the shell rules (done)
