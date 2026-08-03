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

## style: typeable punctuation + line-width source sweep

- [[N]] 0.78.2 style: typeable punctuation + line-width source sweep

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
