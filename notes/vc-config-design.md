# vc-config: settable configuration keys

Long-form documentation for every settable vc-x1 config key, one section per key, headed by the
key's path. The `reference:` line in a generated config and in `vc-x1 config` output links to the
key's section here: build.rs derives the link from `[vc-config] reference-base` in
`vc-config.toml` (the schema prototype) plus the heading's anchor, and the test suite fails if a
key has no matching heading. Sections are in schema order.

The `bot-session.*` keys resolve git-style, most specific wins: CLI flag, then the workspace
`.vc-config.toml`, then the user config (`~/.config/vc-x1/config.toml`), then the built-in
default. The other keys live in a single home each: `default.*`, `repo.*`, and `account.*` in
the user config, `repos.*` in the workspace configs.

## default.account

The account profile `init` and other account-aware commands use when `--account` is not given on
the command line: the name of an `[account.<name>]` section in the user config.

## default.debug

Reserved: the value `--debug` would assume when used without an argument. No command consumes it
yet; setting it is harmless and does nothing.

## repo.default

The repo category `--repo` assumes when given bare (no `<cat>` argument): the name of a
[repo.category.\<cat\>](#repocategorycat) entry, one of the built-ins (`remote`, `local`) or
your own.

## repo.category.\<cat\>

The literal value behind a repo category name: a remote url prefix (`init` appends
`/<NAME>.git`) or a local parent directory. `init` resolves `--repo <cat>` through this table
when its target is a path or a bare name.

## account.\<name\>.repo.default

Per-account variant of [repo.default](#repodefault): the default category used while the
`<name>` account profile is active.

## account.\<name\>.repo.category.\<cat\>

Per-account variant of [repo.category.\<cat\>](#repocategorycat): a category defined inside an
account profile, shadowing the top-level table while that account is active.

## bot-session.items

The default item set the `bot-session` conversation view renders, a comma-separated list (e.g.
`headers,user,assistant,tool,summary`). The per-item `--<item>` / `--no-<item>` flags override
individual items, and `--all` / `--none` replace the base set entirely.

## bot-session.result-lines

Maximum lines shown per tool result in the conversation view when results are rendered; `0`
means unlimited. The `--result-lines` flag overrides it.

## bot-session.col-width

First-column width in `bot-session`'s field-inventory views: `--fields`, `--unknown`, and
`--per-line`. The conversation view never consults it. The built-in default aligns the type
column for ~99% of observed key paths: every structural key except a long tail of
`snapshot.trackedFileBackups.<absolute path>.*` keys, whose embedded absolute paths can be
arbitrarily long and so are left to overflow. The `--col-width` flag overrides it.

## repos.work

The work repo's path, relative to the config file's directory: `"."` in the work repo, `".."`
in the bot repo. Structural: written by `init`, read by workspace-root discovery, and the entry
that resolves to the config's own directory names that file's side. Every `.vc-config.toml`
must carry it.

## repos.bot

The bot repo's path, relative to the config file's directory (e.g. `".claude"` in the work
repo, `"."` in the bot repo). Its presence is what signals dual-repo mode; a single-repo
workspace simply omits it.
