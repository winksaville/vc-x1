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

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)).

## Waiting

Important work that cannot start yet. Each entry names what it waits on, in a form that can be
checked, and the rank it takes in `## Todo` once unblocked. Every opening checks each condition and
promotes what is met ([Opening](AGENTS.md#opening)).

## Todo

Entries are in priority order, the first highest, and reprioritizing is moving an entry. Each is a
`###` heading, so a citation is a link to its anchor. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the [Prose form](agent-data/prose.md#prose-form).

### Configure this workspace

`vc-x1 init` writes a bare `.vc-config.md`, with no validation and no closing body, so `vc-x1
validate` and a bare `commit --closing` refuse. Add to `.vc-config.md`:

```toml
[validate]
full = ["<the project's full check>"]
fast = ["<the project's quick check>"]

[closing]
body = 'Closes the cycle "{title}". See details in TODO.md.'
```

## Ideas

Items not yet solid enough for `## Todo`. Triaged at the next opening: promote, fold into a
picked-up cycle, or drop.

## Bugs

_See [bugs.md](notes/bugs.md)._

# References
