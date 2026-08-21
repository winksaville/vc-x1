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

The record has one home at a time so it is never written twice. The alternative keeps a
working ladder in `TODO.md` and an as-built ladder in chores, so every rung is written twice
and every backfill applied twice, and detail written twice drifts (the same argument that
keeps the edit list out of the commit body, notes.md's
[Chores section content](notes.md#chores-section-content-no-edit-list-git-is-the-record)).

### Cycles run on a bookmark

A cycle that pushes `main` directly makes every correction a coordinated force-push of
published history. Landing costs one command and buys free rewrites for the whole cycle. A
single-step cycle gets a bookmark for the same reason: a one-commit line is exactly where a
pre-landing rewrite is cheapest.

### Opening

**Backfill first.** The 0.80.0 and 0.80.1 as-built rungs were both found unfilled at the
0.80.2 opening (measured 2026-08-21): backfill was named only at close-out, as "the edits ride
the next push", which names no owner, and the Opening's steps never mentioned it, so the only
place the rule lived was the one moment hard rule 3 forbids acting on it. The previous cycle's
rungs are the usual hits of the check. Not folded into the Done sweep, which is already a
compound step, and a step with two halves is where the second half hides. Recorded in the
"docs: halve AGENTS.md into rationale.md" chores section.

**The bookmark create is a push** because `vc-x1 push` requires the bookmark's remote refs to
be tracked, so the create has to publish, and a publish takes push approval.

**The solution statement is provisional** because it is written before the work.

**Why the acceptance check, and why it is provisional.** A cycle's per-commit checklists can
all pass while its banner claim is false: a seven-cycle program opened against "end subprocess
spawning" and its close-out claimed the goal met, with about twenty spawn sites surviving, two
inside the facade the program built (found 2026-08-06 at the 0.78.3 review, and retired by
the 0.79.0 cycle, chores-17's
[refactor: retire the remaining jj spawns](../notes/chores/chores-17.md#refactor-retire-the-remaining-jj-spawns)).
Being provisional, the check can also be revised
*toward* what was achieved, which is the same failure by a slower route, so a changed check is
one of the things the deliberation exists to justify.

### The per-rung flow

**Validate at every commit, doc-only ones included**, because step 4 changed the version, and
running the validation is how that is verified. **No validation while a review iterates**
because a formatter mutates files in ways that interact badly with the user's mid-review edits,
so it runs once, on the settled state, after the last edit.

**The work-review stop carries no description** because a description beside the work review
collapses two stops into one and describes work the review may still change.

**The `(done)` flip waits for "done" to be true** because before it the user may still reject
or reshape the work the marker would claim.

**Never `jj edit -r @-` to view a past commit**: it marks the commit mutable and shifts `@`.

### Committing vs pushing

Push's commit stages commit both repos and stamp each new commit's `ochid:` trailer, so a
pre-committed rung leaves `@` empty and push mints a stamped empty duplicate (the empty-`@`
push minting orphan agent-repo commits was measured 2026-08-15, in the "docs: trial the
iiac-perf convergence proposals" chores section). **No checks of the project's own** because
vc-x1 assumes nothing about a repo beyond `.jj` and its config.

### Commit description

No version in title or body because a version is stable only once it lands, and a history
rewrite can renumber it. No file list because the diff is the mechanical record. No
deliberation because chores, todo, and the session the `ochid:` trailer names hold that, each
reachable from the commit by construction.

### Pushing

_None recorded._

#### Policy

**Delegation waives stops, never flow**, because the stops are the synchronous half of review
and the flow (the records, the validation, the bookmark discipline) is what deferred review
reads. A delegated cycle that skipped a record would leave the deferred reviewer nothing to
read.

#### Before any push

_None recorded._

#### At rest: push, stop, squash-push

The agent repo (`.claude`) is a live journal, so everything after a `vc-x1 push` invocation,
its own record and any closing words, lands in the agent repo's `@` as a trailing tail. That
tail is why the contract has three parts: the push cannot include its own record, the agent
cannot fold the tail (its own squash-push is itself session data, so `@` refills the moment it
runs), so only the user can. The fold keeps the change id, so the work-side `ochid:` keeps
resolving. The user repeats the squash-push if new writes land because the agent's back end
may consolidate session data minutes later.

### Topic bookmarks are drafts

Pushing to the bookmark makes the work durable and visible, but landing on `main` is
publication, and that is the line the rules divide at. The series is kept self-consistent
before landing so the branch reads as one coherent ladder. Amending content rather than
re-describing keeps hard rule 4 intact and lets the `ochid:` trailers ride along: they carry
change ids, which survive a rewrite.

### Close-out

_None recorded._

### Chores sections

Anchors survive the heading-level shift because GitHub slugs derive from the heading's text,
not its level. The renumbered refs and the rebased links are checked by hand because both fail
silently: a mis-renumbered ref and an un-rebased link render as plain text or a 404 rather than
erroring.

#### Commits backfill

An as-built rung cites its commit by SHA and records the version that commit carried, and
neither is stable until the commit lands on a permanent branch: a rebase or squash rewrites
the SHA on the way, and a history rewrite can renumber the version. A commit cannot record its
own SHA, which is why the fill is always one push later.

### Local ladders

Retired name: "Ladder (sub-cycle)", which collided with the working record's `#### Ladder`.
That ladder is the cycle's rung list, and a local ladder is one rung's scratch history. The
fast validation per ladder commit is non-negotiable because a regression in an early ladder
commit otherwise goes uncaught until a later commit runs the full suite, raising bisection
cost. The scratch `jj describe` is the one permitted describe because the commit is never
published and never carries a trailer.

## Working practices

_None recorded._

## File map

_None recorded._

## Changing the agent-files

_None recorded._

## custom.md: the project layer

_None recorded._
