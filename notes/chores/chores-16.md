# Chores-16

Continuation of `chores-15.md` (closed after `0.78.0`, the jj-lib migration close-out; the
`0.78.1` instruction-set interlude between the files is recorded in its TODO.md program-ladder
rung and commit body). This file covers `0.78.2` onward, back on `main` with the refactor
program's long-lived bookmark retired and the 20260803 baseline pin set in force.

Reference numbering is file-local; see
[`agent-data/notes.md`](../../agent-data/notes.md#reference-numbering); chores-16 starts at
`[1]`.

## Table of Contents

- [style: typeable punctuation + line-width source sweep](#style-typeable-punctuation--line-width-source-sweep)
- [refactor: trapezoid-push + body-intro validation](#refactor-trapezoid-push--body-intro-validation)

## style: typeable punctuation + line-width source sweep

- [[1]] 0.78.2 style: typeable punctuation + line-width source sweep

`src/` + `tests/` predate the typeable-punctuation rule and code.md's <=100-col comment wrap:
863 banned-character sites across 63 files and 60 overlong lines at pickup. A single-commit
cycle (wink's call at the opening review: the sweep is one homogeneous transformation, so a
ladder would checkpoint nothing a review gate and jj's op log don't already cover).

- the four named characters went to zero: `->` and `...` by mechanical replace, the 708 em
  dashes each given their structural decision (colon for term-elaboration, comma for
  continuation, parentheses for asides, sentence splits for contrasts), converted by four
  parallel lesser-model agents on disjoint file batches and audited after
- the sweep surfaced four *more* untypeable species the ban list does not name: `=>` was
  written `⇒` (13 sites), the config/show output drew rules with `─` (56), plus one `≥` and
  one U+2212 minus; all converted, output tests updated in the same pass (see the dogfood
  finding on the enumeration)
- the ellipsis in `truncate_chars` was load-bearing: appending one-char `…` kept truncated
  gists at cap+1, and the mechanical `...` conversion broke the length test; the function now
  reserves room for the suffix and stays within `max` (contract tightened, test to `<= CAP`)
- `config_cmd` section headers (`# ── x ──`) and `show`'s separator rule are user-visible
  output; they render as `-` runs now, and the README's config samples were regenerated from
  the installed binary so the docs match what the code emits
- line width: string literals split with backslash continuation (content byte-identical),
  comments re-wrapped, trailing `// OK:` comments moved above their statement where wrapping
  was awkward; exempt as literal rows: the JSONL fixture strings in the transcript /
  bot-session tests, and comment URLs

Riding in the same commit: the cycle records, the Done-retirement sweep of four
refactor-program entries into done.md, the `0.78.1` ladder-ref backfill, and the first
generic-custom.md-test findings logged in [notes/dogfood.md](../dogfood.md).

### Delegation notes

Four Sonnet agents converted the em-dash batches and a fifth did the line-width pass; two
incidents worth remembering when delegating sweeps:

- one batch agent "corrected" scope creep it thought it saw, reverting 29 already-converted
  arrows back to `→` because it compared against a pre-sweep baseline; caught by the
  post-batch character audit. A delegated sweep needs the invariant stated as "the end state
  is X", not "convert only Y": agents infer baselines
- the line-width agent first placed the continuation break-space after the backslash, which
  Rust strips with the leading whitespace, silently corrupting a fixture string; its own
  `cargo test` run caught it. String splits verify by test, not by eye

## refactor: trapezoid-push + body-intro validation

- [[N]] 0.79.0-0 refactor: trapezoid-push + body-intro validation opening

The trapezoid close-out is a four-step manual recipe, and its warts are all artifacts of the
shape having no command: an interim published shape, a sideways bookmark move, a backfill
embargo window. `vc-x1 trapezoid-push` makes it first-party, as a subcommand rather than a flag
on `push`, so `push` keeps a stateable invariant: it never produces a merge. Body-intro
validation rides as the first rung, turning jj's opaque `-m "-..."` arg-parse failure
([bugs.md](../bugs.md) #7) into a clear error naming the offending line. The last rung of the
jj-lib refactor program; at this close-out the program's `## In Progress` block retires here.

### 0.79.0-0 refactor: trapezoid-push + body-intro validation opening

Cycle open, plus the records work the previous push left outstanding.

- the `0.78.0` and `0.78.2` as-built rungs were both still `[[N]]`, each being its cycle's
  close-out, which nothing inside that cycle can backfill; the accompanying dogfood entry
  proposes a greppable acceptance check rather than a per-file recall
- the parked `support-trapezoid-commits` bookmark is scheduled for deletion rather than
  rebasing: its one commit extracts `push/state.rs`, the module `0.77.0` deleted, so the
  quarry the disposition note anticipated has no ore in it. The `0.72.0-0` chore commit is
  abandoned with it
- `custom.md` gains its first real content since the `0.78.1` reset to the bare skeleton: the
  validation commands, the dogfood-binary rule, and the mailbox parameters, all three named as
  missing by the generic-custom.md test's first findings
- the `por -> dual conversion` stage leaves the refactor program for its own `## Todo`. It is
  init/setup work, its stated dependencies (facade topology, the in-process init pieces) have
  both shipped, and holding a program open for one independent feature buys nothing
- the cycle runs on the topic bookmark `trapezoid-push-vc-x1`, created at `0.78.2`, rather
  than landing on `main` the way `0.78.2` did. The close-out is a trapezoid and a trapezoid
  needs a branch to merge from, so `<base>` is fixed at cycle open as the parent of the `-0`
  rung. A per-cycle topic bookmark, deleted once merged, not a second long-lived program line
  under jj.md's discipline
- `notes/refactor-20260716.md` is scheduled for deletion at the close-out, not conversion to a
  record. A plan file is scaffolding for work in flight, and each cycle it covered has already
  written its own chores section, so keeping it would be a second copy of an existing record.
  Its one homeless fact, the `0.72.0` version gap, moves to a design subsection here; the
  deletion commit is the citation for the rest
- the revset item was rescoped at wink's review: it is not a translation layer to switch off
  (`resolve_revset` already passes verbatim to jj), but a house `..` notation that inverts
  jj's. It becomes its own cycle after this one, adopting jj's notation and moving the
  windowing to `-A` / `-D` / `-C` flags

# References

[1]: https://github.com/winksaville/vc-x1/commit/a8b43a18999e "a8b43a18999ece30e7b807650ba45eb9b236ebdc"
