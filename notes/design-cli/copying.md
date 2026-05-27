# copying (design stub)

Forward-looking design capture from the 0.61.0 parity audit
cycle. No implementation yet — concrete work lands when a
follow-on cycle picks it up.

Generalizes "copy these files into the new workspace at
init time" into one composable mechanism that subsumes
today's `--config <path>` (deprecated under A2 collapse —
see [[1]]), a hypothetical `--gitignore <path>`, and
today's `--use-template <CODE[,BOT]>`.

The mechanism is **pure file copy** — no variable
substitution, no transformation. The bot deliberately
avoided "templating" / "seeding" as concept names; "seed"
implies growth and "template" implies parameterization,
neither of which this mechanism does. It just copies.

## Goal

One mechanism for "copy these files into the workspace at
init time" rather than per-file flags. The user supplies
one or more source paths; init copies them into the new
workspace during the copy phase, before commits.

## Surface

- `--init-from-code=<glob>` — copy files matched by
  `<glob>` into the code repo (non-recursive).
- `--init-from-bot=<glob>` — same, into the `.claude/`
  companion. Only meaningful under dual.
- `--init-from=<glob>` — shorthand for `--init-from-code`
  (since por only has code, and dual users typically copy
  into code more often than bot).
- `--init-from-recursive=<glob>` /
  `--init-from-code-recursive=<glob>` /
  `--init-from-bot-recursive=<glob>` — recursive variants.

Each flag accepts a **shell glob** (expansion done by the
shell; `vc-x1` receives the expanded path list). Each may
be specified multiple times — the result is the additive
union of all sources. Order matters for collisions (see
below).

## Behavior

- **Copy semantics** — files only; preserves relative
  layout under the source root. The bot's working
  assumption is `cp -a` semantics (preserve symbolic
  links and executable bits); confirm at design time.
- **Collision: last writer wins, with a warning.** If two
  `--init-from*` flags resolve to the same destination
  file, the later one overwrites the earlier; `vc-x1`
  emits a warning naming both sources. The bot thinks
  warnings are right here — silent overwrite is a
  footgun; erroring would block valid layering use cases
  (a base directory + per-project tweaks).
- **Canned writes suppressed when copying is engaged.**
  Today `init` writes a canned `.vc-config.toml` and
  `.gitignore`. With any `--init-from*` present, **both
  canned writes are suppressed entirely**. The user opted
  in to full control of the copied content; init doesn't
  layer canned defaults underneath.
- **Dual post-copy check (deferred).** After copying
  completes for a dual workspace, if `.vc-config.toml`
  is missing or lacks `[workspace] other-repo`, init
  **warns** (does not error). The workspace looks dual
  (has `.claude/`) but acts por at runtime; downstream
  subcommands enforce the invariant. See
  [Deferred-validation](#deferred-validation).

## Deferred-validation

Init's `.vc-config.toml` check is a **warning**, not an
error, to support workflows where the user plans to
supply the config post-init (e.g. checking in the
workspace shell first, then populating config files in a
follow-up commit).

The error happens at the **first downstream invocation**:
`default_scope` (or a new explicit check) detects the
broken-dual state — `.claude/` exists but `.vc-config.toml`
doesn't (or lacks `other-repo`) — and refuses to proceed
with a clear error pointing at the missing file.

Today `default_scope` silently falls back to
`Scope([Code])` when `.vc-config.toml` is absent
(`src/common.rs:594`). That silent fallback is exactly
the "looks dual, acts por" surprise this design avoids;
fixing it is a prerequisite for the post-init copy
workflow.

## Fixed filename

`.vc-config.toml` stays the fixed name; overriding it is
deferred until a real use case surfaces (would ripple
through every subcommand's lookup code — `default_scope`,
`find_workspace_root_from`, `validate-desc`, `fix-desc`,
`push`, `clone`, …).

## Subsumes

- `--config <path>` — never really built; A2's collapse
  left the capability un-claimed. Equivalent under
  copying: `--init-from=<src>/.vc-config.toml`.
- `--gitignore <path>` — never built; equivalent under
  copying: `--init-from=<src>/.gitignore`.
- `--use-template <CODE[,BOT]>` — today's coarse
  whole-repo flag. Subsumed by recursive variants:
  - `--use-template code-tpl/` →
    `--init-from-code-recursive=code-tpl/*`.
  - `--use-template code-tpl/,bot-tpl/` → two flags:
    `--init-from-code-recursive=code-tpl/*` +
    `--init-from-bot-recursive=bot-tpl/*`.

The bot thinks `--use-template` can be retired once
copying ships, but a back-compat shim that translates
old form → new form is cheap and worth keeping for one
release cycle.

## Naming rationale

The mechanism is named **copying** (not "templating," not
"seeding," not "scaffolding") because all three imply
something the mechanism doesn't do:

- "Templating" implies parameterization / variable
  substitution. We deliberately don't do that.
- "Seeding" implies growth (a seed grows into a plant);
  the copied files don't grow — they sit where they
  landed.
- "Scaffolding" implies project-structure generation
  (Rails generators, Yeoman); the mechanism is just
  `cp -r`, no structure-aware logic.

"Copying" is literal: source files go to destination
paths. The flag family `--init-from*` keeps the verb out
of the surface — the flag describes *what* (source-to-
destination), not *what kind of operation*. If a future
mechanism does layer substitution on top of copy, it's a
separate feature with its own name.

## Open questions (resolve at design time)

- **Variable substitution.** Should the copy step support
  `{{name}}` / `{{account}}` substitution? The bot
  thinks **no** — keep the mechanism pure copy. Users
  who need substitution can pre-process and pass the
  result. Adding substitution invites a substitution-
  language rabbit hole and breaks the "copying" framing
  we just settled on.
- **Symbolic links and executable bits.** Preserve, or
  flatten? Probably preserve — `cp -a` semantics.
  Worth confirming at design time.
- **Source-not-found behavior.** Hard error (typo
  protection) or soft skip (let globs return empty)?
  Probably hard error — a missing source is almost
  always user mistake. Empty glob expansion (the shell
  passing zero paths) is fine.
- **Interaction with `vc-x1 clone`.** Clone seeds from
  an existing remote, so copying overlays *on top of*
  the cloned content. Same flag set, same collision
  rules.
- **Mechanism beyond init?** A future
  `vc-x1 <subcommand> --copy-from=<glob>` to layer
  files into an existing workspace would be the same
  mechanism with a different host command. The bot
  thinks defer until a real use surfaces; `--init-from*`
  is the only surface today.

## See also

- `notes/design-cli/por-dual-parity-audit.md > ## Feature axes > A2`
  — the collapse that motivates this design [[1]].
- `notes/design-cli/por-dual-parity-audit.md > ## Feature axes > A5`
  — `--use-template` today; this design subsumes it.

# References

[1]: /notes/design-cli/por-dual-parity-audit.md#a2-vc-configtoml-write--collapsed
