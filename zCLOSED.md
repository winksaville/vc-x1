# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### chore: custom.md is untracked and gitignored

(wink + agent, 2026-10-17) This is a public repo, meant to be cloned by anyone, and they will not
be members of the vc-x1 family. A committed `custom.md` hands every clone one machine's layer: the
family's sibling paths and the session's standing permissions.

#### Deliberation

- The family is local, and no repo assumes one, wink's call. `custom.md` stops being committed,
  and the family's shared layer lives in `../vc-x1-family`.
- This cycle untracks `custom.md` and nothing else, and what follows from it is in `TODO.md`, the
  Todo entry that held the untracking and the agent-files' wording split in two (2026-10-18).
- `custom.md` stays on disk here and leaves the tree from this commit on. Earlier commits keep it,
  holding nothing private. A clone that pulls this commit has its `custom.md` deleted, and we
  think this machine holds the only one.
- A copy of the tree without `custom.md` passed `cargo test` once made a jj repo, and failed three
  `cli_smoke` tests as a plain git repo, a finding older than this cycle, filed as [The CLI smoke
  tests need no jj repo](TODO.md#the-cli-smoke-tests-need-no-jj-repo).

#### Solution

`custom.md` is gitignored and untracked, and the work that follows from it is in `TODO.md > ##
Todo`.

#### Acceptance check

- `jj file list custom.md` prints nothing, `.gitignore` holds `/custom.md`, and `custom.md` is
  still on disk.
- `vc-x1 validate-anchors TODO.md zCLOSED.md` passes.
- A `jj git clone` of the commit's tree holds no `custom.md`, and `cargo install --path . --locked
  --root <scratch>` there installs a `vc-x1` that runs, `~/.cargo/bin` untouched.
- After the push and before Land, a `vc-x1 clone` of the remote, on the bookmark by `jj new
  <bookmark>@origin`, holds no `custom.md` and installs the same way. Its result is reported, not
  recorded, since the commit carrying this record is the one it checks.
- Result: pass. `jj file list custom.md` prints nothing, `.gitignore` holds `/custom.md`, and the
  file is on disk. The anchors check clean. The `jj git clone`, of this repo at a local bookmark
  on the working copy, held no `custom.md` and installed `vc-x1 0.85.2 (agent-files v0.2.9)`.
  - `vc-x1 clone` of the same local path failed, the source's git `HEAD` being detached, so the
    clone tracks no `main`. Filed as [vc-x1 clone of a local jj workspace tracks no
    main](TODO.md#vc-x1-clone-of-a-local-jj-workspace-tracks-no-main), and the remote clone
    after the push stands in for it.

#### Ladder

- chore: custom.md is untracked and gitignored (done)

# References
