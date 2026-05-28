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

### 5. List-valued CLI-vs-config "wins" semantics deserves an explicit callout

**Critique:** `audit.md:697-699` says: "If user passes
any `--init-from*` on CLI, the config-pinned list is
ignored — match the broader chain rule."

The "broader chain rule" is about *scalar* values where
"wins" unambiguously means "replaces." For list-valued
axes, the bot thinks users coming from `LD_LIBRARY_PATH`
/ `PATH` will expect merge, not replace. The replace
choice is defensible (predictability beats expressivity)
but it's not the obvious choice.

**Proposed action:** keep the replace semantics, but add
an explicit "list-typed-axis rule" sentence — "for
list-valued axes like Copying, CLI replaces config; to
merge, pass the config globs on CLI too." Prevents the
same conversation in 0.62.0.

**Status:** open. Docs-only edit.

### 6. Gap-list ordering hides one structural dependency

**Critique:** `audit.md:1010-1080` ranks gaps by "blast
radius." But Gap #7 (`default_scope` broken-dual
detection) is a *prerequisite* for Gap #9 (copying
mechanism), per `copying.md:81-92` — copying defers
validation to the first downstream subcommand, which
means broken-dual detection has to exist first.

**Proposed action:** one-line note above the list:
"ranking is rough; some structural prerequisites (e.g.
#7 before #9) override size ordering." Doesn't require
renumbering.

**Status:** open. Docs-only edit.

## Nits

Lower-priority items; do as time allows.

- **`finalize` matrix row** marks T+SC support based on
  the body being topology-neutral. True, but no command
  path exercises it today. The bot thinks adding a
  footnote — "currently latent — body supports it, no
  caller surfaces it" — would prevent a future cycle
  hardening a use case nobody asked for.

- **Reading guide at top of audit doc.** The doc is ≈1090
  lines and will be cited from many cycles. A `## Reading
  guide` (matrix → §X; one axis → §Y; new axis → §Z)
  costs 10 lines and pays back across many readings. The
  bot thinks this is worth doing before 0.62.0 starts.

- **"Por's view of the chain"** (`audit.md:897-913`)
  duplicates the per-axis Decisions blocks. The bot
  thinks it could be dropped or reframed as a
  debugging cheat-sheet ("when does topology actually
  get consulted?") — a different angle.

- **`validate-todo` / `fix-todo`** are topology-blind by
  design. Worth one sentence somewhere ("notes/-family
  commands operate outside the workspace shape") so a
  future reader doesn't suspect they were missed.

## Process observation

The user-proposed-then-deferred `~/.config/vc-x1/config.
toml` topology default that **opened** the cycle is, by
close-out, **in** the design (`audit.md:763`:
`[default].topology = "single" | "dual"`). The bot's
initial "audit first, expose defaults later" pushback
was about *ordering*, not the idea itself. Net: the cycle
delivered what the user originally proposed, just through
a more thorough process.

The bot thinks this is worth noting because it shows the
pushback shape ("don't ship that *yet*") working
correctly — but also that the original ask was the right
ask. The Decisions blocks currently read as if the design
emerged from the bot's analysis alone; an acknowledgment
of the original user framing would balance the record.

**Status:** open. Optional chores narrative tweak (no
action implied for the design itself).

## Disposition table

| # | Title | Status | Action surface |
| --- | --- | --- | --- |
| 5 | List-valued CLI-vs-config "wins" | Open | `audit.md` Copying Decisions |
| 6 | Gap-list ordering hides one prereq | Open | `audit.md` Gap list intro |
| N1 | `finalize` matrix row footnote | Open | `audit.md` matrix |
| N2 | Reading guide at top of audit doc | Open | `audit.md` top |
| N3 | "Por's view of the chain" redundancy | Open | `audit.md` § "Por's view" |
| N4 | `validate-todo` / `fix-todo` topology-blind note | Open | `audit.md` (anywhere appropriate) |
| Pr | Original user framing acknowledgment | Open | Either `chores-12.md` cycle narrative or `audit.md` Decisions blocks |

## TL;DR

The two highest-conviction concerns — 1 (runtime `--por`
semantics) and 3 (copying-surface doubling) — are applied
and removed from this list. What remains (concerns #4–#6,
nits N1–N4, and the process observation) is docs-only: low
risk to defer, but cheaper to apply now in one design pass
than after the next ten cycles cite the current text.

# References

[1]: /notes/design-cli/por-dual-parity-audit.md
[2]: /notes/design-cli/copying.md
[3]: /notes/design-cli/por-dual-parity.md
[4]: /notes/chores/chores-12.md#docs-pordual-parity-design-0610
