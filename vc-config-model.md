# vc-x1 config file

A model config file: every table and key a workspace config may carry, each with its default or a
typical value. Generated from the schema, so it cannot fall behind the keys the binary knows. Copy
the tables you need into your own `.vc-config.md` and drop the rest.

The values inside a `toml` fence are the workspace's own, and the prose around them is
documentation. Only a fence tagged exactly `toml` is read, so any other fence is prose like the
text beside it. Each bullet links to its key's entry in the schema documentation.

The `[agent-session]` table
- items: Default agent-session item set (comma-separated) [[1]]
- result-lines: Default --result-lines: max lines shown per tool result (0 = unlimited) [[2]]
- col-width: Default --col-width: first-column width in the --fields / --unknown / --per-line views
  [[3]]
```toml
[agent-session]
items = "headers,user,assistant,tool,summary"
result-lines = 10
col-width = 68
```

The `[repos]` table
- work: The work repo's path, relative to this config file's directory ("." in the work repo, ".."
  in the agent repo). The entry resolving to the config's own directory names the side [[4]]
- agent: The agent repo's path, relative to this config file's directory (e.g. ".agent-session" in
  the work repo, "." in the agent repo). Presence signals dual-repo mode [[5]]
```toml
[repos]
work = "."
agent = ".agent-session"
```

The `[remote]` table
- agent-repo: The agent-repo's name on its remote, the last URL segment only, since the owner and
  host come from the work-repo's remote. Absent means a name derived from repos.agent: a dot-led
  last component appended to the work repo's name, any other used whole [[6]]
```toml
[remote]
agent-repo = "myproject.agent-session"
```

The `[family]` table
- member: This repo's member name in its agent-file family (also its record file in the messages
  repo) [[7]]
- template: Path to the family's template payload, the directory holding the pinned agent-files
  [[8]]
- messages: Path to the family's messages repo, relative to this config file's directory [[9]]
```toml
[family]
member = "vc-x1"
template = "../vc-x1-template/work"
messages = "../vc-x1-messages"
```

The `[agent-files.diff]` table
- dir: Default DIR for agent-files diff: a directory holding a copy of the set [[10]]
- custom: Default for agent-files diff --custom: compare custom.md like the rest of the set [[11]]
```toml
[agent-files.diff]
dir = "../iiac-perf"
custom = false
```

The `[agent-files.copy]` table
- dir: Default DIR for agent-files copy: a directory holding a copy of the set [[12]]
- custom: Default for agent-files copy --custom: copy custom.md with the rest of the set [[13]]
```toml
[agent-files.copy]
dir = "../vc-x1-template/work"
custom = false
```

The `[squash-push]` table
- yes: Default for squash-push: act without asking once the precheck found work [[14]]
```toml
[squash-push]
yes = true
```

The `[validate]` table
- full: Full validation, in order, one invocation per element, run by `vc-x1 validate` [[15]]
- fast: Fast validation, in order, one invocation per element, run by `vc-x1 validate --fast` [[16]]
```toml
[validate]
full = [
  "cargo fmt",
  "cargo clippy --all-targets -- -D warnings",
  "cargo test -- --include-ignored",
]
fast = ["cargo test --bins"]
```

The `[build]` table
- release: The release build, in order, one invocation per element, run by `vc-x1 build` [[17]]
- release-bin: The release build's binary, run by `vc-x1 run` and copied by `vc-x1 install` [[18]]
- dev: The dev build, in order, one invocation per element, run by `vc-x1 build-dev` [[19]]
- dev-bin: The dev build's binary, run by `vc-x1 run-dev` and copied by `vc-x1 install-dev` [[20]]
```toml
[build]
release = ["cargo build --release --locked"]
release-bin = "target/release/vc-x1"
dev = ["cargo build --locked"]
dev-bin = "target/debug/vc-x1"
```

The `[install]` table
- release: Optional, the release install run by `vc-x1 install` in place of its own copy [[21]]
- release-dir: Where `vc-x1 install` copies the release build, `~` expanded [[22]]
- dev: Optional, the dev install run by `vc-x1 install-dev` in place of its own copy [[23]]
- dev-dir: Where `vc-x1 install-dev` copies the dev build, `~` expanded [[24]]
- dev-suffix: Appended to the dev build's file name by `vc-x1 install-dev` [[25]]
```toml
[install]
release = [
  "cp {build.release-bin} {install.release-dir}/vc-x1.new",
  "mv {install.release-dir}/vc-x1.new {install.release-dir}/vc-x1",
]
release-dir = "~/.cargo/bin"
dev = [
  "cp {build.dev-bin} {install.dev-dir}/vc-x1{install.dev-suffix}.new",
  "mv {install.dev-dir}/vc-x1{install.dev-suffix}.new {install.dev-dir}/vc-x1{install.dev-suffix}",
]
dev-dir = "~/.cargo/bin"
dev-suffix = "-dev"
```

The `[closing]` table
- body: The closing commit's default body, `{title}` the cycle title [[26]]
```toml
[closing]
body = "Closes the cycle {title}. See details in zCLOSED.md."
```

The `[test]` table
- fixtures: Directory of the acceptance-test fixtures, one dual workspace per name, read by `cargo
  test` [[27]]
```toml
[test]
fixtures = ".fixtures"
```

# References

[1]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessionitems
[2]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessionresult-lines
[3]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessioncol-width
[4]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#reposwork
[5]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#reposagent
[6]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#remoteagent-repo
[7]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#familymember
[8]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#familytemplate
[9]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#familymessages
[10]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-filesdiffdir
[11]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-filesdiffcustom
[12]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-filescopydir
[13]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-filescopycustom
[14]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#squash-pushyes
[15]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#validatefull
[16]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#validatefast
[17]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#buildrelease
[18]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#buildrelease-bin
[19]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#builddev
[20]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#builddev-bin
[21]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#installrelease
[22]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#installrelease-dir
[23]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#installdev
[24]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#installdev-dir
[25]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#installdev-suffix
[26]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#closingbody
[27]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#testfixtures
