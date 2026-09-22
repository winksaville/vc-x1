# vc-x1 init validation notes

Empirical observations from testing `vc-x1 init --scope` at
0.42.0-3, focused on anomalies that need follow-up. Structural
correctness confirmed for each tested case unless noted.

## Test setup

- Binary: `vc-x1 0.42.0-3`
- Test root: `../tmp-vc-x1/` (sibling of project dir)
- Session: 2026-04-27

## Test 1: `--scope=code`

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=code`

Single-repo workspace produced correctly: no `.claude/`,
config has `path = "/"` only, no `other-repo`.

Anomalies:

- `bm-track exit` line prints `app(main)=tracked,
  .claude(main)=tracked` even though no `.claude/` exists in
  single-repo mode. The bot thinks the bm-track wrapper format
  is hard-coded for the dual-repo case.
- Step 4 has no console announcement (output jumps 3 → 5).
  Cosmetic only.
- "Done!" footer labels the bare remote as `Code repo:`. The
  shown path is `remote.git` (bare), not the working repo at
  `repoA`. Misleading label — should distinguish working repo
  vs bare remote.

## Test 2: `--scope=code,bot`

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=code,bot`

Dual-repo workspace produced correctly: both `.vc-config.toml`
files have expected shape (`path` + `other-repo`), both initial
commits carry cross-referencing `ochid:` trailers, symlink at
`~/.claude/projects/...repoA` resolves to a real directory.

Anomalies:

- Same labeling issue as Test 1: "Done!" footer labels both
  bares as `Code repo:` / `Session repo:`. Paths shown are
  `remote-code.git` / `remote-claude.git` (bares), not the
  working repos.
- Symlink target embeds the literal `..` from the
  `--repo-local` invocation:
  `/home/wink/data/prgs/rust/vc-x1/../tmp-vc-x1/repoA/.claude`.
  Functional (filesystem resolves `..`) but not canonicalized.
  The bot thinks `init` passes `--repo-local` verbatim into
  the symlink target rather than calling `canonicalize` first.

## Test 3: `--scope=bot,code`

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=bot,code`

Verified structurally identical to Test 2: same `.vc-config.toml`
contents in both repos, same ochid cross-reference pattern (each
initial commit points at the other's chid), same symlink shape.
Order-of-sides in the keyword form is commutative for init's
purposes.

No new anomalies.

## Test 4: `--scope=bot` from a clean dir

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=bot`

Expected fatal — confirmed:

- Exit 1.
- Error: `--scope=bot is meaningless at init time — use
  --scope=code for single-repo or --scope=code,bot (default)
  for dual-repo`.
- Filesystem unchanged (no `repoA/` created).
- `bm-track exit` line still prints
  `app(main)=tracked, .claude(main)=tracked` even though the
  init aborted before touching any repo. Same cosmetic anomaly
  as Test 1 (the bm-track wrapper format appears scope-blind).

## Test 5: `--scope=code` after a failed bot attempt

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=code`

Same outcome as Test 1 (single-repo workspace produced
correctly). Confirms the failed Test 4 attempt left no
residual state that interferes with the subsequent
single-repo init.

## Test 6: `--scope=bot` on top of existing code-only workspace

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=bot`
(after Test 5 left a code-only workspace in place)

Expected: ideally an incremental upgrade — add `.claude/`,
rewrite parent's `.vc-config.toml` to include `other-repo`,
ochid-link the existing initial commit to a new bot-side
initial commit, create the symlink. Actual:

- Same plan-time error as Test 4 — the
  `--scope=bot is meaningless` check fires before any
  inspection of the existing workspace.
- Filesystem unchanged.
- The error message suggests `--scope=code,bot` as the
  alternative, but that path is also blocked (see Test 7).

## Test 7: `--scope=code,bot` on top of existing code-only workspace

`vc-x1 init repoA --repo-local=../tmp-vc-x1 --scope=code,bot`
(after Test 5 left a code-only workspace in place)

Expected fatal — confirmed, but at a different check than
Test 6:

- Exit 1.
- Error fires at preflight (after `Preflight checks...`):
  `'/home/wink/data/prgs/rust/vc-x1/../tmp-vc-x1/repoA'
  already exists`.
- Filesystem unchanged.

## Findings: missing incremental-upgrade path

Answered at 0.84.12 by `init --adopt`, whose design is [Adopting an existing
tree](#adopting-an-existing-tree). The `--scope` sketch below is kept as the record of what was
wanted.

`init` today supports two and only two terminal modes from a
clean dir:

| Mode | Invocation | Outcome |
|---|---|---|
| Single-repo | `--scope=code` | code repo + `.vc-config.toml` (no `other-repo`) |
| Dual-repo | `--scope=code,bot` (or default) | both repos + cross-ref configs + ochid trailers + symlink |

There is **no path** to grow a single-repo workspace into a
dual-repo workspace via `init`. Both candidate invocations
fail:

- `init --scope=bot` on existing code-only → fatal at
  plan-time (`--scope=bot is meaningless at init time`),
  before any inspection of the existing workspace.
- `init --scope=code,bot` on existing code-only → fatal at
  preflight (`'<path>/repoA' already exists`).

The bot's design sketch for the upgrade path:

- Detect the upgrade case in plan-time: `--scope=bot` AND the
  target dir contains a `.vc-config.toml` with `path = "/"`
  AND no `other-repo`.
- Switch to upgrade mode: skip code-side init steps; create
  `.claude/`, write `.claude/.vc-config.toml`, rewrite
  parent's `.vc-config.toml` to add `other-repo = ".claude"`,
  bare-init the bot remote, write a bot-side initial commit
  with an `ochid:` trailer pointing at the existing code
  initial commit, optionally rewrite the existing code commit
  to add the reverse `ochid:` trailer (or accept asymmetry
  for the upgrade case), create the symlink.
- `init --scope=code,bot` on existing code-only could either
  also enter upgrade mode (reusing the existing code repo) or
  stay fatal — the user-facing semantics matter (does
  `code,bot` mean "ensure both" or "create both fresh"?). The
  bot's lean: keep `code,bot` strict-create (today's
  semantics) and route the upgrade through the explicit
  `--scope=bot`-on-existing form.

This is a feature gap, not a bug — to be filed as a todo
candidate after the 0.41.1 sync work is settled.

## Test 8: `--scope=<path>` (path-form)

`vc-x1 init repoA --scope=../tmp-vc-x1-path`
(no `--repo-local`, no `--repo-remote`)

Expected fatal — confirmed:

- Exit 1.
- Error: `--scope=../tmp-vc-x1-path: path-form scope is
  meaningless at init time — use --repo-local <PATH> for a
  local fixture or --repo-remote <URL> for an existing
  remote`.
- Filesystem unchanged (no `repoA/` or `tmp-vc-x1-path/`
  created).
- Same `bm-track` cosmetic anomaly fires.

Status: per the chores-07 design (`### --scope enum refactor
(0.42.0)`'s applicability matrix), init was deliberately
specified as `Single(_)` accepted = no, on the rationale
"init creates workspaces, not POR repos." Re-opened
2026-04-27 — see "Findings: path-form support" below.

## Findings: path-form support is wanted

The user wants `init --scope=<path>` to work. The current
flag set splits two concerns that the path form could fold
together:

| Today | Path-form shorthand |
|---|---|
| `--scope=code` (mode) + `--repo-local=<dir>` (location) | `--scope=<dir>` |

Two reasonable semantics for `init NAME --scope=<path>`:

- **(a) Local-fixture single-repo shorthand.** Equivalent
  to `init NAME --repo-local=<path> --scope=code`. Creates
  a single-repo workspace at `<path>/NAME` with a local bare
  remote, no GitHub. Mirrors the `Single(_)` semantic from
  sync — "ignore the dual-repo defaults, operate on this one
  spot." This is the bot's recommended reading.
- **(b) Bare-init at exactly `<path>`.** The path IS the
  workspace location, no `NAME` segment appended. Conflicts
  with the existing `NAME` positional arg (would have to
  become optional / mutually exclusive). More invasive.

Reading (a) is the cleanest: it preserves the existing
`NAME` positional, drops to single-repo by default
(consistent with `Single(_)` everywhere else), and the
generated workspace looks identical to what
`--scope=code --repo-local=<path>` produces today.

The user's stated priority (2026-04-27): implement
path-form support **before** the 0.41.1 sync fix, since the
sync fix's validation harness benefits from the shorthand.

## Adopting an existing tree

`init --adopt` (0.84.12, the **feat: init adopts an existing tree** cycle) grows an existing
directory into a dual workspace. Without it, an existing target is refused, named by what it
holds. `src/init/adopt.rs` reads the target first:

| State | What adopt does |
|---|---|
| Absent | refuses: `--adopt` on a missing target is likelier a typo than a request |
| Plain directory, no repo | both repos created around it, its content the work repo's first commit |
| A repo with no workspace config | one commit on top, its history kept |
| A single-repo workspace | as above, its config edited in place |
| A dual-repo workspace | refuses: nothing to grow, the agent repo of one included |

The decisions behind it:

- A repo is a `.git` or a `.jj` in the target itself. Adopt takes a jj repo colocated with git and
  points a git-only one at `jj git init --colocate`. A working copy with uncommitted work is
  refused, since adopt's commit would take it in.
- An adopted repo's commit, "Adopt as a dual-repo workspace", carries `.vc-config.md`, the
  `.gitignore` line, and the `ochid:` trailer, and goes on top of its history. With an `origin`,
  the work side pushes nothing, that commit being the user's to land, and the agent repo's remote
  is derived beside the origin, the provisioner read off its URL. With no origin, the remotes come
  from `--repo` as a fresh init's do.
- The `--repo` chain needs the user config, which a repo with an origin does not, so under
  `--adopt` the chain's error is held and raised only when the target has no origin.
- A single-repo workspace's config is edited by adding lines only, `agent =` after the last key of
  `[repos]` and `agent-repo =` under `[remote]`, with prose, comments, and other keys untouched.
  The file is read back and restored when it does not declare both keys.
- An adopted directory's `.gitignore` is kept and given the agent directory's line.
- The first commit tracks every file whatever its size, since jj leaves a new file over
  `snapshot.max-new-file-size` untracked, and names the ones over the limit.
- `--use-template` is refused with `--adopt`, since a template would overwrite files of the same
  name.
