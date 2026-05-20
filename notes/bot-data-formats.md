# Bot data formats and multi-vendor support

Captures design discussion about how vc-x1 manages bot data
across multiple vendors, multiple formats, and multiple
versions. Companion to `forks-multi-user.md` (which covers
forking, multi-user collaboration, and bot-repo size); this
file focuses on the *content* of the bot repo rather than
its multi-user structure.

This file is forward-looking: most of what it describes is
how the bot thinks the design should evolve, not what's
implemented today. Treat as a design reference, not a
status doc.

## Core principle: vc-x1 treats bot data as opaque

vc-x1 manages git/jj plumbing, commit conventions, and
trailer parsing. It does **not** parse bot data content,
know what a "Claude session" is, or care which vendor
authored which file.

Layered responsibility:

- **vc-x1's job:** track files, manage commits, manage
  trailers, run sync. Bot data is opaque bytes identified
  by file path within the bot repo.
- **Viewer / explorer's job:** render bot data per its
  understood format. Dispatches on file path or content
  metadata.
- **Bot data's job:** carry vendor / format / version
  identity discoverably (via path, filename, or header
  line).

This division survives any future evolution. Anthropic
changes their jsonl format → viewers update; vc-x1
unchanged. New vendor (Cursor, Aider, custom in-house) →
viewers add a renderer; vc-x1 unchanged. Multiple bots in
one repo → trivially supported; vc-x1 doesn't notice.

The bot thinks this is the load-bearing principle that
should govern every interaction between vc-x1 and bot data,
including future subcommand-layer / CLI decoupling work
(see [`ARCHITECTURE.md`](../ARCHITECTURE.md)). The
`Context` handle there exposes paths and commits; it
does not expose parsed sessions.

## Why dual-repo (merits-based defense)

The dual-repo split (code repo + bot repo) originated
tactically — Claude Code stores sessions at
`~/.claude/projects/<path>/` by default, so symlinking
out to a sibling jj-git repo was the natural path of
least resistance. The split wasn't architectural at design
time. But it has earned its keep on its own merits, even
if those merits weren't visible up front:

- **Size separation.** The bot repo dominates total
  workspace size (78:1 measured on 2026-05-07; see
  `forks-multi-user.md > Bot-repo size and scaling
  thresholds`). A code-only clone (CI, bisect, drive-by
  contributor) can skip the bot data entirely.
  Single-repo cannot offer that.
- **Mutability separation.** The bot-side immutability
  rule (never rewrite) applies cleanly to one side
  without contaminating code-side rebase semantics.
  Single-repo would create awkward "rebase this code
  commit but don't disturb its bot trailer" cases.
- **Sync cadence.** Bot pushes per session (frequent);
  code pushes per PR (less often). Independent histories
  let each move at its own pace without contention.
- **Access pattern.** Bot data is cold (archaeology);
  code is hot (every build). Independent storage matches
  independent access.
- **Access control.** Possible to gate bot data
  separately (e.g. private design conversations on a
  maintainer-only repo, code public). Single-repo
  bundles permissions.

The bot thinks dual is the right default; single is a
valid mode for projects that don't need the separation.
The `--por` flag on `init` / `clone` selects that mode
(see `src/options_flags/por.rs`).

## Vendor-subdir directory layout

Convention for organizing bot data within the bot repo:

```
.bot/
  claude/
    v1/<uuid>.jsonl
    v2/<uuid>.jsonl
  cursor/
    <session_id>.json
  aider/
    <chat_id>.md
  custom-tool/
    <id>.txt
```

Hierarchy: `.bot/<vendor>/<version>/<id>.<ext>`.

- **`<vendor>`:** lowercase short-name. `claude`,
  `cursor`, `aider`, `codex`, `custom-tool`. Conflicts
  resolved by project convention.
- **`<version>`:** vendor-specific version label. `v1`,
  `v2`, `2024-12`, etc. Vendors with no breaking-change
  history yet (Cursor today) can omit the version layer
  initially and add later when v2 ships.
- **`<id>`:** opaque identifier from the vendor (UUID,
  session ID, chat ID, whatever the vendor uses).
- **`<ext>`:** vendor's native extension (`.jsonl`,
  `.json`, `.md`, `.txt`, …).

Viewers register against `<vendor>/<version>` keys,
dispatch to the right parser, render. New version →
register `<vendor>/<v_new>`. New vendor →
register `<vendor_new>/*`.

## Multi-bot in one repo

Falls out automatically from the layout. The bot repo is
a forest of linear chains (per
`forks-multi-user.md > Bot repo as a forest of linear
chains`); some chains are Claude sessions, some Cursor,
some custom. Each chain's commits modify files in that
chain's vendor subdir only. ochid trailers point to
commits regardless of which vendor authored them — the
trailer format doesn't change.

A single project can mix bots freely:

- Developer A uses Claude Code for design work →
  `.bot/claude/v2/...`
- Developer A uses Cursor for IDE edits →
  `.bot/cursor/...`
- Developer B uses an in-house Codex fork →
  `.bot/codex-internal/...`

All in the same bot repo, all referenced by ochid trailers
from code-side commits, all rendered by their respective
viewer plugins. vc-x1 doesn't need to know any of this.

## Format versioning within a vendor

Vendors will change formats over time. Future major
changes likely include multi-modal payloads, tool-call
schema revisions, and conversation-tree representation
shifts.

The directory-layout convention handles this without
parser-level coordination: a new format ships at
`.bot/claude/v3/`, old sessions stay at `.bot/claude/v2/`
unmodified (the immutability rule). Viewers register a v3
parser when they support v3; sessions written before that
date remain readable via v2.

The bot thinks **forward compatibility is not a goal** —
old viewers reading new formats can fail loudly. The point
of the version layer is *backward* compatibility (new
viewers reading old formats), enforced by keeping old
sessions in their original subdir untouched.

## Migration from `.claude/<uuid>.jsonl`

Today's layout is flat: `.claude/<uuid>.jsonl`. Migrating
to vendor-subdir layout means:

```
.claude/<uuid>.jsonl  →  .bot/claude/v1/<uuid>.jsonl
```

Two viable approaches:

1. **One-shot rewrite at migration time.** A
   `vc-x1 migrate-bot-layout` subcommand walks the bot
   repo, moves every flat file into its
   `<vendor>/<version>/` subdir, commits the move. The
   existing chids stay stable because the migration is a
   forward-only commit (a new commit that moves files),
   not a history rewrite. All past ochid trailers
   continue to resolve.
2. **Transitional read-both window.** vc-x1 + viewers
   accept both layouts for a transition period. New
   sessions go to the new layout; old sessions stay
   where they are. Eventually a one-shot rewrite still
   happens to consolidate, but the timing is decoupled
   from the introduction of the new layout.

The bot thinks **(1) one-shot rewrite** is the right
choice for vc-x1 — the project is small, the migration is
mechanical, and the read-both window adds parser
complexity for marginal benefit. Larger projects with
years of flat-layout history might prefer (2) for safety;
vc-x1 isn't there yet.

Migration ought to ride on a `--scope`-aware
`vc-x1 migrate-bot-layout --scope=bot` invocation once the
scope work continues.

## The `.claude` → `.bot` rename

Once vendor-subdir layout exists, `.claude/` is misleading
as the directory name — it implies single-vendor lock-in
when the design is multi-vendor. The rename `.claude` →
`.bot` aligns directory naming with the vendor-neutral
intent.

The rename is decoupled from the layout migration:

- **Layout migration** is the substantive change (moves
  files into vendor subdirs; requires viewer updates).
- **Directory rename** is cosmetic (`.claude/` → `.bot/`).
  Once the symmetric `.vc-config.toml` schema lands (TODO
  item — see `notes/todo.md`), the role-name → directory
  mapping is config-driven; the rename becomes a config
  change, not a code change.

Order: ship the symmetric schema first, ship layout
migration second, rename `.claude/` → `.bot/` last (or
fold it into the layout migration as a single mechanical
change).

The bot thinks the rename has lower urgency than the
layout — `.claude/` works fine as long as it's understood
as a historical name. Defer until config-driven naming is
in place.

## Viewer / explorer layer (out of scope for vc-x1)

vc-x1 does not implement a bot-data viewer. The viewer
layer is a separate concern, likely a separate crate /
binary / TUI / GUI:

- **Reads** files from the bot repo's vendor subdirs.
- **Dispatches** on `<vendor>/<version>` to the right
  parser.
- **Renders** session content (text, diffs, tool calls,
  etc.) in whatever UI it ships.
- **May fetch** from URL-shaped ochid references for
  cross-repo bot data (per
  `forks-multi-user.md > Per-user bot repos via
  URL-shaped ochid`).

The bot thinks this is where future-derivative work the
user has hinted at lives — multi-window TUI / GUI for bot
conversation exploration. The architectural separation
(vc-x1 = plumbing, viewer = parsing / rendering) means the
viewer can iterate independently without affecting vc-x1's
stability.

## Open questions

- **Vendor name registry.** Who decides `claude` vs
  `claude-code` vs `anthropic` as the canonical
  short-name? The bot thinks: project-level convention
  (documented in CLAUDE.md or similar), with a default
  registry shipped by vc-x1 / viewers but overridable
  per project.
- **Sub-vendor variants.** Anthropic has Claude Code and
  Claude (api, web). Are these one vendor (`claude`)
  with multiple session formats, or distinct vendors
  (`claude-code`, `claude-api`)? The bot thinks distinct
  — different file shapes, different metadata — but
  worth deciding before the registry hardens.
- **Sidecar metadata.** Should each session file have
  an optional `.meta.toml` sidecar (vendor name, format
  version, started-at, completed-at)? Cleaner than
  parsing the file, but adds another file per session.
  The bot thinks no — the directory layout already
  carries vendor / version, and the file's first line
  can carry per-session metadata when needed.
- **Mixed-vendor sessions.** Could a single session
  span multiple vendors (e.g. a Claude conversation
  that invoked Cursor for a tool call)? The bot thinks
  no — each tool's session record is its own session,
  even if they're causally linked. Cross-references go
  through ochid trailers, not file aggregation.
- **Index for viewer search.** A viewer that wants to
  search across all sessions ("which conversation
  produced this code?") needs an index. Build at scan
  time, cache in `.bot/.index/`? Or rebuild per query?
  Out of scope for vc-x1, but worth flagging as a
  viewer-side design problem the layout enables.
