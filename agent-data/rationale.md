# Rationale

The why behind [AGENTS.md](../AGENTS.md), one entry per rule that has one. AGENTS.md holds the
rule and its boundaries, and a session needs only that. The argument is for whoever would change
a rule, and for the family at convergence, and it is kept so a rule is not simplified away by an
editor who does not know its cost.

Universal file, shared with the template repository. A proposed change is edited here and
converges at the template ([Changing the agent-files](../AGENTS.md#changing-the-agent-files)).
Project-local content goes in [custom.md](../custom.md).

## How to read this file

- **Headings mirror AGENTS.md's**, same text, same level, so the anchors line up 1:1 and a rule
  reaches its why by one fixed pattern, `[why](agent-data/rationale.md#<same-slug>)`. A heading
  with nothing under it but `_None recorded._` is a rule whose why was never written down, which
  is a finding, not a gap to fill with a guess.
- **An entry is the why, then the evidence**: back references to the chores section where the
  rule was paid for, the dogfood entry, the messages-repo record, the commit. Mostly pointers,
  not a re-telling. The "measured YYYY-MM-DD" lines live here, with the story.
- **A boundary sentence is not rationale.** A sentence saying what a rule does not cover is the
  rule, and stays in AGENTS.md. What moves here is argument: why the rule exists, what it cost
  to learn, what the alternatives were.
- **Speculation is marked** as everywhere else ("We think ...", prose.md's
  [Speculation marker](prose.md#speculation-marker)), so a reader can tell the measured from
  the inferred.

## Hard rules

_None recorded._

## Terminology

**Rationale.** The headings mirror so that a missing entry is a grep away: a rule's why is
either at its own slug or nowhere, and nothing has to be searched for by wording. The file
exists so the argument can leave AGENTS.md without dying: a session needs the rule, whoever
would change the rule needs the argument, and a rule whose cost is not written down is the one
an editor simplifies away. Filed as the Todo "Halve AGENTS.md: move its rationale into
`agent-data/rationale.md`" (wink, 2026-08-21).

## The dual-repo model

_None recorded._

## Cycle protocol

_None recorded._

### Cycles run on a bookmark

_None recorded._

### Opening

_None recorded._

### The per-rung flow

_None recorded._

### Committing vs pushing

_None recorded._

### Commit description

_None recorded._

### Pushing

_None recorded._

#### Policy

_None recorded._

#### Before any push

_None recorded._

#### At rest: push, stop, squash-push

_None recorded._

### Topic bookmarks are drafts

_None recorded._

### Close-out

_None recorded._

### Chores sections

_None recorded._

#### Commits backfill

_None recorded._

### Local ladders

_None recorded._

## Working practices

_None recorded._

## File map

_None recorded._

## Changing the agent-files

_None recorded._

## custom.md: the project layer

_None recorded._
