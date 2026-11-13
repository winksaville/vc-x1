# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record, read first at
acquaint, acted on, and reset to `_None._` by the reader.

_None._

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's shape is
the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in [The In Progress
block](agent-data/notes.md#the-in-progress-block).

_No cycle currently in progress._

## Waiting

Important work that cannot start yet. Each entry names what it waits on, in a form that can be
checked, and the rank it takes in `## Todo` once unblocked. Every opening checks each condition and
promotes what is met ([Opening](AGENTS.md#opening)).

## Todo

Entries are in priority order, the first highest, and reprioritizing is moving an entry. Each is a
`###` heading, so a citation is a link to its anchor. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the [Prose form](agent-data/prose.md#prose-form).

### Configure this workspace

`vc-x1 init` writes a bare `.vc-config.md`, with no `[validate]` unless `vc-dr-apply.sh --lang`
added one, so `vc-x1 validate` may refuse, and with no `[version]`, so no commit is checked for
its version bump.

- `[validate]`: when `.vc-config.md` has none, ask the user for one: `Skip` leaves it for later,
  `None` is the doc checks alone, and `Rust`, `C/C++`, and `Python` are the language skeletons, each
  with a starting project that validates. A skeleton is `vc-dr-apply.sh --lang NAME`, run by the
  user from the source repo once this cycle is committed, and `--lang` alone lists them. A source
  may add its own under `scripts/vc-dr-apply-lang/`. The project's own commands are a choice too.
- `[closing]`: add

  ```toml
  [closing]
  body = 'Closes the cycle "{title}". See details in TODO.md.'
  ```

- Commit: when the work `@` holds uncommitted changes, as it does after `vc-dr-apply.sh`, fold
  them into this cycle and commit them with the config. Otherwise commit the config.

## Ideas

Items not yet solid enough for `## Todo`. Triaged at the next opening: promote, fold into a
picked-up cycle, or drop.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)).

# References
