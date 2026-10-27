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

_None._

## In Progress

### feat: build, install, and run take --profile

Picked from `## Todo` as **install --dev --release soaks a release candidate** (wink + agent,
2026-10-27), deferred from **feat: build, install, and run take --dev**, and widened at its
opening's review into named build profiles, `--profile` replacing `--dev` (wink + agent,
2026-10-27). The first cycle run in the v0.3.1 form, wink's call (2026-10-27): it begins with
`commit --opening`, its commit bodies are two paragraphs, the problem and "Solution:", its ladder
labels count from `[0]`, and its continuation notes go in the closing.

#### Problem

A build is chosen by two fixed names, `release` and `dev`, each with its own keys in `[build]` and
`[install]`, and `--dev` chooses the dev build and the `-dev` name at once. So no other build can be
made, a custom `Cargo.toml` profile included, and a release build cannot be installed under another
name for a period of use beside the landed one, a soak: the `[install] dev` list names
`{build.dev-bin}`, so the dev list with the release build would copy the debug binary.

#### Solution

- A profile is a `[profile.<name>]` table of properties, any name and any properties. vc-x1 reads
  two itself, `bin`, the binary the build leaves, and `suffix`, appended to the installed name, ""
  when absent. The rest are the lists' to use.
- `build`, `run`, and `install` take `--profile NAME`, and `{profile.<property>}` in `[build]
  commands` and `[install] commands` is the chosen profile's property, so one build list and one
  install list serve every profile. The default profile is `[build] default-profile`, else
  `release`, and a name with no table is refused naming the tables there are.
- The encouraged form is a `Cargo.toml` profile, named by the `cargo` property: `cargo build
  --profile {profile.cargo} --locked`, cargo's `dev` and `release` and any custom one alike. A
  list-valued property such as `flags = ["--release"]` expands to its words where it is named, none
  for an empty list, so a build by flags works too.
- `release` and `dev` are examples the generated config and the model carry, `dev` with suffix
  `-dev`, and a soak is one more profile, the release build with its own suffix.
- `--dev` is retired, refused naming `--profile dev`, and the old keys, `[build] release`, `dev`,
  `release-bin`, `dev-bin`, and `[install] release`, `dev`, `release-dir`, `dev-dir`, `dev-suffix`,
  are reported by `validate-config` and refused by the commands, each naming the profile form. This
  workspace's config moves to it.
- The README, the help, `vc-config.md` and its model, `init`'s generated config, and the
  agent-files' `install --dev` say `--profile`, the last told to **agent-files(proposal): v0.3.1**.

#### Acceptance check

`cargo test` passes with tests that a list-valued placeholder expands to its words, that
`--profile` chooses the profile's properties for `build`, `run`, and `install`, that an unknown
profile and `--dev` are refused naming the fix, and that `validate-config` reports the old keys.
Live, from the rung that lands `--profile` on: `vc-x1-dev install --profile dev` installs
`vc-x1-dev`, `vc-x1-dev run --profile dev -- -V` prints the dev build's banner, and `vc-x1-dev
install --profile rc --dir tmp/soak`, with a `[profile.rc]` building `release` under suffix `-rc`,
installs `tmp/soak/vc-x1-rc`, which `cmp` finds identical to `target/release/vc-x1`.

#### Deliberation

- Named profiles over a `--release` beside `--dev`, wink's call at the opening's review
  (2026-10-27): a profile names the build and the installed name together, so a soak is a profile
  and not two flags composed, and any build can be made, a custom `Cargo.toml` profile included.
- Named properties over positions, the agent's suggestion taken (2026-10-27): wink's sketch,
  `release = ["--release", "--locked", "target/release/vc-x1"]` read as `{profile.release[2]}`,
  put the meaning in the index, which differed between profiles.
- One `[build] commands` and one `[install] commands` filled from the chosen profile, rather than a
  list per profile, since `{profile.<property>}` is the chosen profile's.
- `Cargo.toml` profiles encouraged, wink's call (2026-10-27), read as a `cargo` property naming the
  profile, "simpler" since cargo then owns the flags and the directory: wink wrote
  `{profile.full}` with `[profile].full = "xyz"`, and the name `cargo` is the agent's, provisional.
  The flags form works too.
- `release` and `dev` are examples, not built in, wink's call (2026-10-27): any profile can be
  built. The default when `--profile` is left out is provisional, `[build] default-profile`, else
  `release`.
- A list-valued property expands to words, since lists are split before they are filled and an
  empty value would otherwise be an empty argument cargo refuses.
- The old keys are refused, not kept working beside the new: two shapes would be two code paths,
  and `validate-config` names the fix for an adopter on the old one.
- `--dev`, landed one cycle ago, retires to `--profile dev`, the treatment the positionals got.
- Multi-step: the placeholder expansion, the schema, the three commands, the retirement, and the
  docs, each reviewable on its own.

#### Ladder

- [feat: build, install, and run take --profile opening][p0] (done)
- [feat: a list-valued placeholder expands to its words][p1]
- [feat: the config defines named build profiles][p2]
- [feat: --profile chooses the build and the install][p3]
- [feat: the old build and install keys are refused][p4]
- [docs: the README and the agent-files name --profile][p5]
- [feat: build, install, and run take --profile closing][p6]

##### feat: build, install, and run take --profile opening

Opens the cycle: the Todo moved here and widened into profiles, the version bumped.

##### feat: a list-valued placeholder expands to its words

`command_list`'s filling learns list values: a word that is a list-valued placeholder alone becomes
its words, none for an empty list.

##### feat: the config defines named build profiles

The schema's `[profile."<name>"]` tables, `[build] commands` and `default-profile`, and `[install]
dir` and `commands`, with this workspace's config and the model carrying `release` and `dev`.

##### feat: --profile chooses the build and the install

The three commands take `--profile`, resolve the chosen profile, and fill `{profile.<property>}`,
`--dev` refused naming `--profile dev`.

##### feat: the old build and install keys are refused

The commands and `validate-config` name the profile form for each old key, and `init` writes it.

##### docs: the README and the agent-files name --profile

The README, the help, and the agent-files' `install --dev`, told to the v0.3.1 proposal.

##### feat: build, install, and run take --profile closing

Closing out the cycle.

# References

[p0]: #feat-build-install-and-run-take---profile-opening
[p1]: #feat-a-list-valued-placeholder-expands-to-its-words
[p2]: #feat-the-config-defines-named-build-profiles
[p3]: #feat---profile-chooses-the-build-and-the-install
[p4]: #feat-the-old-build-and-install-keys-are-refused
[p5]: #docs-the-readme-and-the-agent-files-name---profile
[p6]: #feat-build-install-and-run-take---profile-closing
