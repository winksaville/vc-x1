# Contributing

How to build, test, and change vc-x1, for a person or an agent. What vc-x1 is, and how to install
and use it, is in [README.md](README.md). How a change is made, the cycle and its commits, is in
[AGENTS.md](AGENTS.md#cycle-protocol).

## Tasks

The project's tasks, in the order a change uses them. A change is made as a
[cycle](AGENTS.md#terminology): one commit, or a ladder of commits each called a rung, made on a
bookmark of its own and joined to `main` at the end, the Land. A task is named by its first column,
the name the agent-files use for it.

- Dev and release: the dev build is the quick debug one, for trying a change while it is made, and
  the release build is the optimized one installed as `vc-x1` at Land.
- The commands are the workspace's own `cargo xtask`, the `xtask/` package, which needs nothing
  installed: `cargo xtask -h` lists its tasks, and `cargo xtask completions` explains the shell
  alias, `cx` here, and its completion. Its settings are `Cargo.toml`'s
  `[workspace.metadata.xtask]`.

| Task | What it does | When | Command |
|---|---|---|---|
| bump | sets the version in `Cargo.toml` and refreshes `Cargo.lock`, `X.Y.Z-N` for a cycle's Nth commit and the bare `X.Y.Z` for its last ([Suffix scheme](agent-data/versioning.md#suffix-scheme)) | first, at each commit of a cycle | `cargo xtask bump next`, or `bump X.Y.Z` |
| validate | formats the code, runs the lints, and runs every test, the fixture tests included | before each review, and before every commit | `cargo xtask validate` |
| validate fast | runs the binaries' unit tests only, changing no file | while a review iterates | `cargo xtask validate --fast` |
| test | builds and runs the tests named | to run one test, or the tests matching a filter | `cargo test [<filter>]` |
| clippy | runs the lints, a warning failing it | to see the lints alone | `cargo clippy --all-targets -- -D warnings` |
| build | builds the dev binary, `target/debug/vc-x1`, with the commit it was built from | to build the change without running it | `cargo xtask build` |
| run | builds the dev binary, then runs it with the arguments, no `--` needed | to try the change | `cargo xtask run <args>` |
| install | builds the dev binary, then copies it into `~/.cargo/bin` as `vc-x1-dev`, beside the installed `vc-x1` | after each rung's commit, and to try the change from any directory | `cargo xtask install` |
| install release | builds the release binary and copies it into `~/.cargo/bin` as `vc-x1`, printing its version | at Land | `cargo xtask install --release` |

Each building task takes `--release` or `--profile NAME`, a `Cargo.toml` profile, dev by default,
and `--bin NAME` for another binary. vc-x1's own `validate`, `build`, `run`, and `install` are
deprecated, each warning and naming its task, and a later release refuses them.

## Agents

What an agent working here reads beyond the agent-files, read at acquaint after `custom.md`
([Acquaint](AGENTS.md#acquaint)). It is the project's layer over the agent-files, committed, so
every clone has it, and a workspace's `custom.md`, local and gitignored, overrides it.

- Agent-files: vc-x1's are the `v0.3.1-16` draft of `### agent-files(proposal): v0.3.1` in
  [TODO.md](TODO.md), adopted locally (2026-11-03, `-1` and `-2` 2026-11-04, `-8` to `-10`
  2026-11-11, `-11` to `-13` 2026-11-13, `-14` and `-15` 2026-11-16, `-16` 2026-11-18). The
  proposal stays open, to be made formal by the next cycle.
- Messaging: the projects of the family message each other through the `../vc-x1-messages` repo,
  beside the work-repo, a clone of `https://github.com/winksaville/vc-x1-messages.git`. A
  workspace without it clones it there. Its `README.md` is the protocol and it governs.
  - At an acquaint and at every cycle start the session does that README's Fetch action first,
    under its guards, and at acquaint reads what is pending for us, per its Read messages action.
  - A Fetch is asked for unless the workspace's `custom.md` gives a standing permission.
