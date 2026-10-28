# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

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

What was done:

- A profile is a `[profile.<name>]` table of properties: `cargo`, the `Cargo.toml` profile, the
  encouraged form, `flags`, a list for a build by flags, `bin`, the binary, required, `suffix`,
  appended to the installed name, and any other, all filled as `{profile.<property>}`.
- `build`, `run`, and `install` take `--profile NAME`, else `[build] default-profile`, else
  `release`, an unknown name refused naming the profiles defined. One `[build] commands` and one
  optional `[install] commands`, with `[install] dir`, serve every profile, and a copy onto
  another profile's name in the same directory is refused.
- `command_list::expand` fills a list-valued placeholder with its items, a word each, none for an
  empty list, and `run` and `run_one` take lists.
- `--dev` and the `-dev` commands are refused naming `<command> --profile dev`.
- The nine old `[build]` and `[install]` keys left the schema and this workspace's config,
  `validate-config` reporting each with its replacement and the commands refusing a config still
  on the old form.
- This workspace defines `release` and `dev`, `dev` with suffix `-dev`, and a soak is one more
  profile, as `[profile.rc]` installing `vc-x1-rc` showed.
- The README, `vc-config.md` and its model, the help, `hooks.sh`'s guard test, and the v0.3.1
  proposal follow.

#### Acceptance check

`cargo test` passes with tests that a list-valued placeholder expands to its words, that
`--profile` chooses the profile's properties for `build`, `run`, and `install`, that an unknown
profile and `--dev` are refused naming the fix, and that `validate-config` reports the old keys.
Live, from the rung that lands `--profile` on: `vc-x1-dev install --profile dev` installs
`vc-x1-dev`, `vc-x1-dev run --profile dev -- -V` prints the dev build's banner, and `vc-x1-dev
install --profile rc --dir tmp/soak`, with a `[profile.rc]` building `release` under suffix `-rc`,
installs `tmp/soak/vc-x1-rc`, which `cmp` finds identical to `target/release/vc-x1`.

Result (2026-10-27, at the closing, `vc-x1-dev` 0.85.15): passed.

- Tests: the full validation passed at every rung, the profile rung's 891 tests and the retirement
  rung's 895 among them, covering list expansion, profile resolution, an unknown profile, `--dev`,
  the collision guard, the forwarder, and each retired key.
- Live: `vc-x1-dev install --profile dev` built `cargo build --profile dev` and installed
  `vc-x1-dev` through `install.commands`; `vc-x1-dev run --profile dev -- -V` printed `vc-x1
  0.85.15+a9fe7ae2d774.dirty`; `vc-x1-dev build --profile nope` was refused naming `dev, release`.
- Live, with a `[profile.rc]` added for the check and removed after: `vc-x1-dev install --profile
  rc --dir tmp/soak` built `cargo build --profile release` and installed `tmp/soak/vc-x1-rc`,
  which `cmp` found identical to `target/release/vc-x1`, its `-V` naming 0.85.15.

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
  and `validate-config` names the fix for an adopter on the old one. `install` also refuses an
  old install list beside no `install.commands`, so a custom list is never silently replaced.
- Three rungs' work was delegated to subagents, the profile code, the key retirement, and the
  README, each from a full brief, the agent reviewing each diff and running the full validation
  before its commit, AGENTS.md's Delegate practice applied to work too long for one context.
- `cargo` kept as the property's name and `[build] default-profile`, else `release`, as the
  default, the provisional choices standing at the opening's "lg".
- `--dev`, landed one cycle ago, retires to `--profile dev`, the treatment the positionals got.
- Multi-step: the placeholder expansion, the schema, the three commands, the retirement, and the
  docs, each reviewable on its own.
- Waiver, wink's "go, complete cycle except for landing on main using the new prototype v0.3.1
  agent-file version" (2026-10-27), after the opening's push.
  - Covers every rung from [1] through the closing: the work review, the description review, the
    per-push approval, and the hard stop after each push.
  - Does not cover Land: the closing is pushed without `--land`, its shape the default,
    trapezoid, until wink chooses at the Land.
  - The v0.3.1 form throughout: two-paragraph bodies ending "See details in zDOING.md.", the rung
    subsection the full account in `*` and `-` form, and the continuation notes in the closing.
  - Validation still runs before every push, and a deviation from the plan is still a stop.

#### Ladder

- [feat: build, install, and run take --profile opening][p0] (done)
- [feat: a list-valued placeholder expands to its words][p1] (done)
- [feat: the config defines named build profiles][p2] (done)
- [feat: --profile chooses the build and the install][p3] (done)
- [feat: the old build and install keys are refused][p4] (done)
- [docs: the README and the agent-files name --profile][p5] (done)
- [feat: build, install, and run take --profile closing][p6] (done)

##### feat: build, install, and run take --profile opening

Opens the cycle: the Todo moved here and widened into profiles, the version bumped.

##### feat: a list-valued placeholder expands to its words

`command_list`'s filling learns list values: a word that is a list-valued placeholder alone becomes
its words, none for an empty list.

* A list's words are split before its placeholders are filled, so a value is one word, and an
  empty value would be an empty argument, which cargo refuses.
  - `command_list::expand` takes list values beside the string ones: a word that is `{name}` alone,
    for a list, becomes the list's items, one word each, none for an empty one.
  - `words` stays, the strings-only form, delegating to `expand`, so its callers are unchanged until
    the profile rung passes lists.
* A list inside a longer word, `x{flags}`, has no word for its items to share.
  - It is refused, naming the list and the word, rather than joined with a guessed separator.
* The refusal for an unknown name offered the string placeholders alone.
  - It offers the lists too, so a typo is told every name the list may use.

##### feat: the config defines named build profiles

The schema's `[profile."<name>"]` tables, `[build] commands` and `default-profile`, and `[install]
dir` and `commands`, with this workspace's config and the model carrying `release` and `dev`.

* The schema has no key for a profile, a build for every profile, or an install for every profile.
  - `profile."<name>".cargo`, `flags`, `bin`, and `suffix`, documented each, `flags` a list and
    the rest strings, and `profile."<name>"."<property>"`, a catch-all, so any property is known:
    `validate-config` matches a dynamic key by its path, a `<...>` segment matching any one.
  - `build.commands` and `build.default-profile`, `release` by default, and `install.dir` and
    `install.commands`, the old keys left beside them until the rung that refuses them.
* This workspace has no profiles.
  - `[profile.release]` and `[profile.dev]` name their `Cargo.toml` profiles, `dev` with suffix
    `-dev`, and `[build] commands` and `[install] dir` and `commands` are added to the existing
    tables, the config's fences being one TOML document where a table is declared once. The old
    keys stay, since the installed 0.85.14 reads them for `install --dev` until this cycle lands.
  - The model is regenerated, and `schema::tests::profile_form` checks the new keys.

##### feat: --profile chooses the build and the install

The three commands take `--profile`, resolve the chosen profile, and fill `{profile.<property>}`,
`--dev` refused naming `--profile dev`.

* A build was chosen by `--dev` between two fixed key sets.
  - `build_cmd::resolve_profile` takes `--profile`, else `build.default-profile`, else `release`,
    and reads `[profile.<name>]`: string properties become `{profile.<property>}`, list ones, by
    their bracketed text, list placeholders, `bin` is resolved against the config's directory and
    required, and `suffix` is offered empty when absent. An unknown name is refused naming the
    profiles defined.
  - `build` runs `build.commands` with the profile's placeholders, checked before anything runs,
    and checks the profile's `bin` after it for every profile, the provenance best effort for all.
  - `run` runs the profile's build, then its `bin`, and its completion forward skips `--profile x`
    and `--profile=x`, resolving that profile's binary.
  - `install` copies to `--dir`, else `install.dir`, under the bin's file name with the suffix, or
    runs `install.commands` with the profile's placeholders and `{install.dir}`.
* A dev install was guarded only against the release's name.
  - Without an install list, a copy whose name equals another profile's in the same directory is
    refused, both ways, naming both suffix keys and `--dir`.
* `--dev` was the dev build's flag.
  - It is hidden and refused naming `<command> --profile dev`, and `build-dev`, `install-dev`, and
    `run-dev` name it too.
* The list runner filled strings alone.
  - `command_list::run` and `run_one` take lists and fill through `expand`, and
    `check_placeholders` takes list names, `words` kept for the tests alone.
* The code was written by a subagent from the agent's brief, and the agent reviewed its diff and
  ran the full validation, 891 tests passing.

##### feat: the old build and install keys are refused

The commands and `validate-config` name the profile form for each old key, and `init` writes it.

* The nine old keys, `[build] release`, `dev`, `release-bin`, `dev-bin`, and `[install]
  release`, `dev`, `release-dir`, `dev-dir`, `dev-suffix`, were in the schema and this workspace's
  config, read by nothing since the profile rung.
  - They leave the schema, `vc-config.md`'s links that named them pointing at the profile keys,
    and the model is regenerated.
  - `build_cmd::RETIRED_KEYS` maps each to its replacement, and `validate-config` reports a
    retired key once, with that sentence, before its unknown-key check.
* A config on the old form would fail with a missing key and no way forward.
  - `build`, `run`, and `install` refuse when `build.commands`, `install.dir`, or
    `install.commands` is absent and an old key is present, naming the keys held, the profile form,
    and `validate-config`. Without the old keys the errors are as before.
  - `install` refuses an old install list beside no `install.commands`, so a custom list, a signing
    step say, is never silently replaced by the built-in copy.
* This workspace held both forms.
  - Its `.vc-config.md` keeps the profile form alone, its prose rewritten and its references
    renumbered, the `--locked`, `.new`-then-move, and under-a-second facts kept.
* `init` writes no `[build]` or `[install]` keys, so it needed no change: its commented optional
  keys come from the schema, and `install.dev-suffix` left with it.
* Written by a subagent from the agent's brief, the diff reviewed and the full validation run by
  the agent.

##### docs: the README and the agent-files name --profile

The README, the help, and the agent-files' `install --dev`, told to the v0.3.1 proposal.

* The README described two fixed builds, `--dev`, and the nine old keys.
  - Its overview, Usage, flag table, build, install, and run sections, and its config tables
    describe profiles, `--profile`, `{profile.<property>}`, and the profile form, with a soak
    profile, `[profile.rc]` installing `vc-x1-rc`, and a custom `Cargo.toml` profile as examples.
  - `### install --dev` is `### install --profile`, its one link following, and the retired forms
    are named where the README says what was retired.
  - Written by a subagent from the agent's brief, its anchors checked at 0 failed, and the
    agent tagged the new `[profile.<name>]` table bullet with this cycle's version.
* `agent-data/hooks.sh`'s guard test named `install --dev`.
  - It names `install --profile dev`, still a deny, `hooks.sh test` passing its thirteen.
* **agent-files(proposal): v0.3.1** records the profile rename beside the flags cycles' notes.
* The help was written with the commands, in the profile rung.

##### feat: build, install, and run take --profile closing

Closing out the cycle.

- Close-out shape: trapezoid, the default, recorded under the waiver, which leaves the choice to
  wink at the Land, `commit --closing` making the merge, pushed without `--land`.
- `notes/agent-files-size.md` gained this cycle's column and row, 1855 lines, unchanged, the one
  agent-file touched being `hooks.sh`.
- The continuation notes are written in this closing, the first under v0.3.1's rule, and say what
  the next session checks after the Land.
- What closing taught: the acceptance check needed a profile this workspace does not keep, so it
  was added for the check and removed, the check's config not the committed one. A standing
  `[profile.rc]` would make the soak one command, a choice for wink rather than this closing.

# References

[p0]: #feat-build-install-and-run-take---profile-opening
[p1]: #feat-a-list-valued-placeholder-expands-to-its-words
[p2]: #feat-the-config-defines-named-build-profiles
[p3]: #feat---profile-chooses-the-build-and-the-install
[p4]: #feat-the-old-build-and-install-keys-are-refused
[p5]: #docs-the-readme-and-the-agent-files-name---profile
[p6]: #feat-build-install-and-run-take---profile-closing
