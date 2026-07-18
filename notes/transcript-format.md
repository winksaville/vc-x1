# Session transcript format

A Claude Code session is recorded as a `.jsonl` file — one JSON
object per line. `vc-x1 bot-session` reads these files; this note
is the single source of truth for what the format looks like and
how to explore it, and the README
[bot-session](../README.md#bot-session) section points here.

Every command below runs against
[`transcript-sample.jsonl`](transcript-sample.jsonl) — a small
sample committed next to this note, so each example is
reproducible. The transcript format is undocumented by Anthropic
and evolves, so `bot-session` parses tolerantly (see
[Two-layer parse](#two-layer-parse)) and doubles as a tool for
exploring the format itself.

## Entries and entry types

- Each line is an **entry** — one JSON object.
- Every entry carries a top-level `type` field, and that value is
  its **entry type**: `mode`, `permission-mode`,
  `file-history-snapshot`, `user`, `assistant`, `system`, … An
  entry with no `type` is bucketed as `<none>`.
- The `--fields` / `--unknown` views group their inventory by
  entry type — that is why the output reads `=== user (2 lines)
  ===` followed by the fields seen across all `user` entries.

## Field names

A field is named by its position in the entry's JSON:

- levels are joined with `.` — `snapshot.timestamp`,
  `message.role`.
- `[]` marks an array element — `message.content[].text` is the
  `text` of each object in the `message.content` array.
- the two compose to any depth —
  `message.usage.iterations[].output_tokens`.

These names are the first column of the `--fields` / `--per-line`
views. They can get long: the sample's deepest is
`message.usage.iterations[].cache_creation.ephemeral_1h_input_tokens`
at 67 characters — which is why `--col-width` defaults to 68.

## Two-layer parse

`bot-session` keeps every field the source writes (the raw entry)
while a typed layer consumes a known subset. The gap between the
two is the unexplored surface:

- `--fields` — the full inventory: every field, per entry type,
  with a count and up to three sample values.
- `--unknown` — the same, minus the fields the typed layer
  already consumes: the unmodeled / new surface.
- malformed lines (for example a live session's truncated last
  line) warn to stderr and never fail the run.

## The sample

[`transcript-sample.jsonl`](transcript-sample.jsonl) holds 7
entries across five entry types, from an 82-character `mode` line
to a ~1.3 KB `assistant` entry:

| Index | Entry type | What it is |
|-------|------------|------------|
| 0 | `mode` | session mode marker |
| 1 | `permission-mode` | permission mode marker |
| 2 | `file-history-snapshot` | file-backup snapshot |
| 3 | `file-history-snapshot` | another snapshot |
| 4 | `user` | a `/effort` slash command |
| 5 | `user` | that command's stdout |
| 6 | `assistant` | a one-line reply, with usage stats |

The **Index** is the 0-based source-line number `bot-session`
prints as `Index N` and the unit `--lines` slices by.

## bot-session example

### The conversation — default view

`vc-x1 bot-session transcript-sample.jsonl` renders the essentials
and skips bookkeeping entries:

```
=== user 2026-07-17 16:10:49Z ===
<command-name>/effort</command-name>
            <command-message>effort</command-message>
            <command-args></command-args>
<local-command-stdout>Set effort level to high …</local-command-stdout>

=== assistant 2026-07-17 17:38:40Z ===
Now the module itself:

bot-session: 2 turns shown; skipped: 4 bookkeeping entries
```

The four skipped entries are the `mode`, `permission-mode`, and
two `file-history-snapshot` lines — bookkeeping, never rendered as
conversation.

### One entry's fields — `--per-line`

`--per-line` prints one section per entry — `=== Index N: <type>
[time] ===` then a row per field (name, value kind, value). The
`mode` entry (index 0) is about as simple as an entry gets (shown
with `--col-width 14`, since the default 68 is overkill here):

```
=== Index 0: mode ===
  mode           str       normal
  sessionId      str       2a51d281-2438-486b-b91f-f80c735cf842
  type           str       mode
```

The `assistant` entry (index 6) is the other extreme — this is
where `.` nesting and `[]` arrays appear (excerpted; at the
default width):

```
  message.content[].text                                 str   Now the module itself:
  message.content[].type                                 str   text
  message.usage.input_tokens                             num   1
  message.usage.cache_creation.ephemeral_1h_input_tokens num   716
  message.usage.iterations[].output_tokens               num   4013
  message.usage.iterations[].cache_creation.ephemeral_1h_input_tokens  num  716
```

Reading the names: `message.content[]` is the array of content
blocks, so `message.content[].type` is each block's `type`;
`message.usage.iterations[]` is a second, deeper array, and
`.cache_creation.ephemeral_1h_input_tokens` nests two more objects
below it.

### The inventory — `--fields`

`--fields` aggregates across the whole file, grouped by entry
type, with a per-field count (`x2`) and up to three distinct
sample values (`|`-separated). The two `file-history-snapshot`
entries collapse into one group:

```
=== file-history-snapshot (2 lines) ===
  isSnapshotUpdate             bool   x2  false
  messageId                    str    x2  7202061f-… | 3ab3a592-…
  snapshot.messageId           str    x2  7202061f-… | 3ab3a592-…
  snapshot.timestamp           str    x2  2026-…925Z | 2026-…010Z
  snapshot.trackedFileBackups  obj{}  x2
```

`obj{}` is an empty object (no backups tracked in this sample);
the `x2` shows both snapshots share the field.

### The unmodeled surface — `--unknown`

`--unknown` drops every field the typed layer already consumes,
leaving what `bot-session` does not model. On this sample it
surfaces, among others, that the `assistant` entry carries both
`session_id` and `sessionId` (a real quirk), plus `effort`,
`slug`, and `requestId`:

```
=== assistant (1 lines) ===
  effort      str  x1  medium
  requestId   str  x1  req_011Cd838YGaPHox1wRkfLCiv
  slug        str  x1  declarative-orbiting-quokka
  …
  bot-session: 45 unknown paths across 7 entries; 0 malformed lines
```

### Verbatim — `--raw`

`--raw` pretty-prints the source line itself; pair it with
`--lines` to isolate one entry. `--raw --lines 0,1`:

```
{
  "mode": "normal",
  "sessionId": "2a51d281-2438-486b-b91f-f80c735cf842",
  "type": "mode"
}
```

### Slicing — `--lines`

`--lines` selects by source-line number, the same unit in every
view (default, `--fields`, `--per-line`, `--raw`):

- `--lines 0,1` — one line starting at index 0 (the `mode` entry).
- `--lines 4,1` — one line at index 4 (the `/effort` user entry).
- `--lines 2` — the first two lines; `--lines -1` — the last.
- `--lines 0` — no entries, just the summary line.

Because it slices *source lines*, a range can land entirely on
bookkeeping entries (the default view then shows nothing but the
skip summary); `--per-line` and `--fields` show those entries
regardless.
