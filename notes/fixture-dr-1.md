# The dr-1 fixture

The `vc-x1 lookup` tests need a dual workspace in which every relationship is known before the
test runs: which transcript write produced a work line, which push carried it, and which agent
commit is that push's partner. A real workspace has these only by reconstruction, which is what
lookup does, so a test over one would check lookup against itself. `dr-1` is made instead by a
scripted agent that records each relationship as it creates it. `dr` is for dual repo, and `1`
for the simple shape, one work repo to one agent repo. A one-work-to-many-agents fixture would be
a numbered sibling.

The lookup design it serves is [transcript-write.md](transcript-write.md).

## What it holds

- A dual workspace made by `vc-x1 init`: the work repo at the root and the agent repo at
  `.claude`, each with a local bare origin. The agent directory is named because the fixture
  predates init's `.agent-session` default.
- Three work files the cycles edit, `notes.md`, `design.md`, and `TODO.md`, and two the build
  writes last, `README.md` and `relationships.json`.
- Two session files in the agent repo, written in the real transcript shape. Every file edit is a
  Write or Edit tool call and every command a Bash call, so each work line has exactly one
  transcript write.
- One clock for both sides: it starts at 2026-09-12T20:00:00Z, advances a second per transcript
  line, and jumps by set gaps between pushes and between cycles. Commit times come from the same
  clock, so a commit's time sits among the transcript lines around it, and the gaps the tests
  rely on are chosen, not measured.
- A fixed committer, `dr-1 builder`, so no developer's identity or signing key reaches it.

## The cycles, and the case each exists for

1. A single-step cycle, `docs: add the notes file`: one commit, landed by a fast-forward. The
   plain pairing, a push and its partner.
2. A multi-step cycle, `feat: the design note`, an opening, two rungs, and a closing, landed as a
   trapezoid:
   - Set aside: a `notes.md` line written during the opening is set aside as a patch and
     restored in the second rung, so its transcript write is in the opening's window and its
     commit is the second rung's.
   - Moved: the closing moves the cycle-record from In Progress to Closed past the Todo entries,
     so blame names the closing while the write is in the opening's window.
   - Rewritten: the trapezoid rewrites the closing, and its change id is still what the trailer
     finds.
3. A restart: a second session file begins, so the next opening's window spans both files.
4. A multi-step cycle, `fix: the design note`, landed kept separate, with a rung amended after
   its push: folded into the pushed rung, the bookmark re-pushed, and the agent side
   squash-pushed. Both sides have a predecessor, and the amended line's write is in the amended
   partner.
5. A commit pair made by hand, work first and the agent 20 seconds later, with no `ochid:`
   trailers, so lookup can only name candidates by time.
6. A single-step `docs: describe the fixture` pushing `README.md` and `relationships.json`, after
   everything they describe, so it is not in their record.

## The record

`relationships.json` is what the tests read, and `README.md` is the same facts as tables:

- Pushes: title, case, the change id on each side, the window of transcript lines the agent
  commit added, and whether the push was rewritten or has a predecessor.
- Writes: each work line's file and text, its line on `main`, the push whose window holds its
  write, the push whose commit first carries it, the closing that moved it, the write's session
  and line, and the change id jj's annotate gives it on `main`.
- Discussion: the transcript lines that are talk rather than writes, with their pushes.

The record's ids, windows, and blame are read with the jj CLI, never with the crate's own jj-lib
calls: lookup is built on jj-lib, so a record taken through it would check lookup against itself.
