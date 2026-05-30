# Max review #1 — 0.61.0 por/dual parity design

Captured 2026-05-26 (max effort) after the close-out commit
of 0.61.0 (`c0f3376f`, pre-push to `main`). Independent
review of the design output (audit doc, copying stub,
chores narrative). Each item below carries a **Status** —
open (needs decision), resolved (with the agreed
disposition), or applied (already changed).

## Scope reviewed

- [`por-dual-parity-audit.md`](por-dual-parity-audit.md) —
  canonical design doc post-0.61.0 (≈1090 lines).
- [`copying.md`](copying.md) — copying mechanism design
  stub from `0.61.0-4`.
- [`por-dual-parity.md`](por-dual-parity.md) — older
  forward-looking doc (read for context only; not
  critiqued).
- [`/notes/chores/chores-12.md > ## docs: por/dual parity
  design (0.61.0)`](../chores/chores-12.md#docs-pordual-parity-design-0610) —
  cycle narrative.

Line numbers in this review are **as of the 0.61.0
close-out snapshot**; section / heading names are the
durable references.

## Strengths

For balance — what the bot thinks works well:

- **Three-pass methodology** (audit → commonality
  inversion → axis decomposition). The `-2` inversion is
  the cycle's pivotal move: reframed "we have a problem
  everywhere" to "the right pattern already exists, extend
  it." That mental flip made the rest tractable.
- **Axis decomposition is genuine decomposition.** Calling
  out `--por` as a bundle of six axes opens design space
  the bundle hid. The litmus is `audit.md:752` — "A user
  wanting a fully-plain single repo today can't spell it" —
  a real defect of the bundle framing.
- **A2 collapse was earned.** `.vc-config.toml` write was
  initially listed as independent; the design pass
  noticed presence-of-`.vc-config.toml` ≡ Topology and
  retired the axis. Honest housekeeping.
- **Two-class principle ("defaults where natural; errors
  only for user-specific keys") is sound.** Correct
  generalization of what `resolve_repo` already does.
  Baked-in default config + `vc-x1 config dump` closes
  the "what does layer-4-empty mean?" question
  elegantly.
- **T7-only promotion.** Resisting the urge to seed all
  14 Gaps into Todo is the right restraint — cheapest
  concrete equalization that validates the
  topology-from-config rule as the prototype.
- **Status note + "implementation will diverge" framing**
  (`audit.md:19-37`) is humility the bot will thank
  itself for in 0.65.0.

## Concerns

_All concerns applied and removed from this list; the
Disposition table below and the git log are the record._

## Nits

_All nits (N1–N4) applied and removed from this list; the
Disposition table below and the git log are the record._

## Process observation

_Applied: the original-user-framing acknowledgment landed
in the `0.61.0` chores narrative
([chores-12.md](../chores/chores-12.md#docs-pordual-parity-design-0610)),
closing the loop that the deferred user-config topology
default reached the design by close-out._

## Disposition table

_Fully drained — every item (concerns 1–6, nits N1–N4,
process observation) is applied and removed. The git log
(`0.62.0-2` … `0.62.0-10`) holds the per-item commits._

## TL;DR

All review items — concerns 1–6, nits N1–N4, and the
process observation — are applied across `0.62.0`
(`-2` … `-10`) and removed from this list. What survives
in this doc is the record: Strengths, Scope reviewed, and
the drained-section markers. The git log is the
authoritative per-item trail.

# References

[1]: /notes/design-cli/por-dual-parity-audit.md
[2]: /notes/design-cli/copying.md
[3]: /notes/design-cli/por-dual-parity.md
[4]: /notes/chores/chores-12.md#docs-pordual-parity-design-0610
