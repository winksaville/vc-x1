# vc-config: settable configuration keys

This file is two things at once, and that is the point: the **schema prototype** vc-x1 is built
from, and the **long-form documentation** every generated `reference:` url lands on. One `##`
section per settable key holds that key's prose and, in a `toml` fence, that key's metadata, so
the documentation and the schema cannot describe different keys.

It is not a config file. Your workspace's config is `.vc-config.md` (note the leading dot).
Editing this file changes what the binary knows, takes effect on the next build, and is reviewed
like any source change.

## How this file is read

The format is the config markdown carrier's own: the `toml` fences, concatenated in document
order, form the TOML that gets parsed. build.rs shares the filter (`src/md_fence.rs`) with the
instance-config loader, so the prototype and a `.vc-config.md` are read by one rule, and prose
outside the fences never reaches a parser.

Shape: one TOML table per settable key, named by the key's path, dynamic segments quoted
(`[repo.category."<cat>"]`). Key order here is rendering order everywhere downstream. The
entries a key's table may carry:

- `homes`: which config file(s) may contain the key. vc-x1 writes it only into those files, and
  `vc-x1 config --validate` reports it as unknown anywhere else. The values:
  - `"user"`: the user-wide config, `~/.config/vc-x1/config.toml` (or
    `$XDG_CONFIG_HOME/vc-x1/config.toml`). One per user, still TOML rather than markdown
  - `"workspace-code"`, `"workspace-bot"`: the work side's and the bot side's instance config
- `kind`: value shape, one of `"str"`, `"usize"`, `"item-list"`
- `doc`: one-line description (rendered into generated configs, so keep it tight)
- `used-by`: what reads the value once it is set, which is a different question from `homes`:
  the command or flag it feeds, or the structural role it plays
- `default`: the built-in value; omit when there is none
- `example`: representative value for keys with no default
- `required`: active (not commented) in generated configs; the value is role-specific, filled by
  `init`
- `reference`: optional override url for the key's docs; when absent (the norm) build.rs derives
  `<reference-base>/blob/HEAD/vc-config.md#<anchor of the path>`, which is the key's own section
  below

## The prototype's own metadata

`[vc-config]` is this file's metadata rather than a settable key. `reference-base` is the bare
repo url every derived reference is built from: build.rs joins it with `/blob/HEAD/vc-config.md#`
and the key's heading anchor, so a reference follows the repo's default branch and no branch name
is baked in. A fork points it at its own repo and edits its own copy of this file.

```toml
[vc-config]
reference-base = "https://github.com/winksaville/vc-x1"
```

## How the keys resolve

The `bot-session.*` keys resolve git-style, most specific wins: CLI flag, then the workspace
instance config, then the user config (`~/.config/vc-x1/config.toml`), then the built-in default.
The other keys live in a single home each: `default.*`, `repo.*`, and `account.*` in the user
config, `repos.*` in the workspace configs.

## default.account

The account profile `init` and other account-aware commands use when `--account` is not given on
the command line: the name of an `[account.<name>]` section in the user config.

```toml
[default.account]
homes = ["user"]
kind = "str"
doc = "Account profile (an [account.<name>] section) to use when --account is absent"
used-by = "--account (init and account-aware commands)"
example = "work"
```

## default.debug

Reserved: the value `--debug` would assume when used without an argument. No command consumes it
yet; setting it is harmless and does nothing.

```toml
[default.debug]
homes = ["user"]
kind = "str"
doc = "Default --debug value when used without an argument (reserved; not yet consumed)"
used-by = "--debug (reserved; not yet consumed)"
example = "true"
```

## repo.default

The repo category `--repo` assumes when given bare (no `<cat>` argument): the name of a
[repo.category.\<cat\>](#repocategorycat) entry, one of the built-ins (`remote`, `local`) or
your own.

```toml
[repo.default]
homes = ["user"]
kind = "str"
doc = "Default repo category when --repo is bare: a [repo.category.<cat>] name: a built-in (remote, local) or your own"
used-by = "--repo (default category when --repo is bare)"
example = "acmehousing"
```

## repo.category.\<cat\>

The literal value behind a repo category name: a remote url prefix (`init` appends
`/<NAME>.git`) or a local parent directory. `init` resolves `--repo <cat>` through this table
when its target is a path or a bare name.

```toml
[repo.category."<cat>"]
homes = ["user"]
kind = "str"
doc = "Literal value for repo category <cat>: a remote URL prefix (init appends /<NAME>.git) or a local parent dir"
used-by = "--repo <cat> (init remote/local resolution)"
example = "git@github.com:acmehousing"
```

## account.\<name\>.repo.default

Per-account variant of [repo.default](#repodefault): the default category used while the
`<name>` account profile is active.

```toml
[account."<name>".repo.default]
homes = ["user"]
kind = "str"
doc = "Per-account default repo category: a [repo.category.<cat>] name: a built-in (remote, local) or your own"
used-by = "--account <name> with --repo"
example = "acmehousing"
```

## account.\<name\>.repo.category.\<cat\>

Per-account variant of [repo.category.\<cat\>](#repocategorycat): a category defined inside an
account profile, shadowing the top-level table while that account is active.

```toml
[account."<name>".repo.category."<cat>"]
homes = ["user"]
kind = "str"
doc = "Per-account literal value for repo category <cat> (remote URL prefix or local parent dir)"
used-by = "--account <name> with --repo <cat>"
example = "git@github.com:acmehousing"
```

## bot-session.items

The default item set the `bot-session` conversation view renders, a comma-separated list (e.g.
`headers,user,assistant,tool,summary`). The per-item `--<item>` / `--no-<item>` flags override
individual items, and `--all` / `--none` replace the base set entirely.

```toml
[bot-session.items]
homes = ["user", "workspace-code", "workspace-bot"]
kind = "item-list"
doc = "Default bot-session item set (comma-separated)"
used-by = "bot-session --<item> / --no-<item> / --all / --none"
default = "headers,user,assistant,tool,summary"
```

## bot-session.result-lines

Maximum lines shown per tool result in the conversation view when results are rendered; `0`
means unlimited. The `--result-lines` flag overrides it.

```toml
[bot-session.result-lines]
homes = ["user", "workspace-code", "workspace-bot"]
kind = "usize"
doc = "Default --result-lines: max lines shown per tool result (0 = unlimited)"
used-by = "bot-session --result-lines"
default = 10
```

## bot-session.col-width

First-column width in `bot-session`'s field-inventory views: `--fields`, `--unknown`, and
`--per-line`. The conversation view never consults it. The built-in default aligns the type
column for ~99% of observed key paths: every structural key except a long tail of
`snapshot.trackedFileBackups.<absolute path>.*` keys, whose embedded absolute paths can be
arbitrarily long and so are left to overflow. The `--col-width` flag overrides it.

```toml
[bot-session.col-width]
homes = ["user", "workspace-code", "workspace-bot"]
kind = "usize"
doc = "Default --col-width: first-column width in the --fields / --unknown / --per-line views"
used-by = "bot-session --col-width"
default = 68
```

## repos.work

The work repo's path, relative to the config file's directory: `"."` in the work repo, `".."`
in the bot repo. Structural: written by `init`, read by workspace-root discovery, and the entry
that resolves to the config's own directory names that file's side. Every instance config must
carry it.

```toml
[repos.work]
homes = ["workspace-code", "workspace-bot"]
kind = "str"
doc = "The work repo's path, relative to this config file's directory (\".\" in the work repo, \"..\" in the bot repo); the entry resolving to the config's own directory names the side"
used-by = "find_workspace_root, side detection (structural; written by init)"
required = true
example = "."
```

## repos.bot

The bot repo's path, relative to the config file's directory (e.g. `".claude"` in the work
repo, `"."` in the bot repo). Its presence is what signals dual-repo mode; a single-repo
workspace simply omits it.

```toml
[repos.bot]
homes = ["workspace-code", "workspace-bot"]
kind = "str"
doc = "The bot repo's path, relative to this config file's directory (e.g. \".claude\" in the work repo, \".\" in the bot repo); presence signals dual-repo mode"
used-by = "default_scope, scope resolution, ochid prefixes (structural)"
example = ".claude"
```
