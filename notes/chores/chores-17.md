# Chores-17

Continuation of `chores-16.md` (closed after `0.78.10`, the refactor-program retirement, at
just over 1000 lines). This file covers the cycles from `0.79.0-0` onward, opening with the
jj-spawn retirement cycle.

Reference numbering is file-local, per
[`agent-data/notes.md`](../../agent-data/notes.md#reference-numbering), and chores-17 starts
at `[1]`.

## Table of Contents

- [refactor: retire the remaining jj spawns](#refactor-retire-the-remaining-jj-spawns)
- [docs: pin two rules and close the convergence record](#docs-pin-two-rules-and-close-the-convergence-record)
- [docs: empty custom-family into the pinned set and config](#docs-empty-custom-family-into-the-pinned-set-and-config)
- [docs: fold the cycle agent-files into AGENTS.md](#docs-fold-the-cycle-agent-files-into-agentsmd)
- [docs: halve AGENTS.md into rationale.md](#docs-halve-agentsmd-into-rationalemd)
- [docs: fix dev artifacts](#docs-fix-dev-artifacts)

## refactor: retire the remaining jj spawns

- [[9]] 0.79.0-0 [refactor: retire the remaining jj spawns opening][1]
- [[10]] 0.79.0-1 [refactor: port push and facade reads to jj-lib][2]
- [[11]] 0.79.0-2 [refactor: port sync repositioning to jj-lib][3]
- [[12]] 0.79.0-3 [refactor: port op recovery and squash to jj-lib][4]
- [[13]] 0.79.0-4 [refactor: port init and clone plumbing to jj-lib][5]
- [[14]] 0.79.0-5 [chore: ban process spawning outside the version gate][6]
- [[15]] 0.79.0 [refactor: retire the remaining jj spawns closing][7]

### Problem

The 0.78.0 jj-lib migration's commit body claimed "ending jj and git subprocess spawning",
but non-test spawns of the jj CLI remained (found 2026-08-06 at the 0.78.3 review), and
nothing stopped new ones from appearing. The picked-up entry's 2026-08-06 inventory also
undercounted: pickup found five more sites, the facade's two `bookmark_list` reads,
squash-push's `jj squash`, init's `jj git remote add`, clone's `jj git clone --colocate`,
and two debug `git rev-parse` reads.

### Solution

Ported in four rungs (reads first, then mutations, then provisioning), then enforced:
push's diff-stat reads and the bookmark-list parsers became typed view queries, sync's
repositioning and op recovery became session verbs inside the facade's index-lock retry,
squash-push's collapse became `squash_into`, and init/clone's provisioning became
`init_colocated` / `add_git_remote` / `clone_fetch` with `jj::git_clone_colocated`
composing the CLI clone flow, the network legs staying jj-lib's own git children. The
enforcement rung banned `std::process::Command::new` via clippy.toml `disallowed-methods`
(deny-level in Cargo.toml) over a documented, closed allowlist, dissolved `common::run`,
and audited the test spawns. A prerequisite for the safer revert's "identifiable sync
operations" (backlog "Stale `/.vc-x1` gitignore line: report it, and a safer revert, if
ever"). The process lesson from the gap (a program's header states its acceptance check at
open, close-out runs it) stays a template-proposal candidate for cycle-protocol.md's
Close-out.

### Acceptance check

1. Non-test code holds no spawn of `jj` or `git` except the version gate's
   `jj --version` / `jj -V` probes and push's `$EDITOR`, measured by reading the
   `grep '"jj"\|"git"' src/` inventory by hand.
2. clippy.toml `disallowed-methods` bans `std::process::Command::new`, the ban fails the
   build on a new spawn site (demonstrated once with a scratch violation), and the version
   gate, `$EDITOR`, and test helpers are the documented allowlist. The enforced half is
   enumerated by `rg '^\s*#.allow.*disallowed_methods' -A 1` (also recorded in
   clippy.toml), whose hits must map one-to-one onto the documented entries.
3. Test spawns of jj are integration-type only, real-jj fixture setup and interop
   verification, with none substituting for an in-process jj-lib assertion of our own code
   paths (audited at the enforcement rung, the finding recorded).
4. Full validation green at the close, with the existing push and sync integration tests
   passing unchanged over the ported paths.

Ran at close-out (2026-08-19), all four pass. 1: the sole surviving non-test jj/git spawn
is the gate's `jj -V`, stronger than the check asked, since the `jj --version` preflight
spelling retired at the init/clone rung. 2: the scratch `Command::new("true")` failed
`cargo clippy --all-targets` with the reason string while every allowlisted site passed,
and the rg one-liner's eight hits map onto the documented entries. The landed allowlist
has four entries, not the opening's three: init's gh provisioning was added (see
Deliberation) and the external CLI-test crates' launcher, which the demo run flushed out,
folded into the test-helpers entry. 3: after centralization the audit surface is five
helpers in two files, all integration-type, and the one violating class (the nine parser
text-fixture unit tests) had already retired at the push/facade rung. 4: fmt, clippy
`-D warnings`, all seven test suites, and `cargo install` green at the closing commit.

### Deliberation

- **Picked up ahead of ranks 1-4** (wink, 2026-08-18: "I'd like to finish the jj-lib
  port"), jumping the strict rank order. The convergence and config entries stay ranked
  where they were.
- **Version 0.79.0**, minor per scope-decides: a subsystem, the jj-CLI spawn path, is
  removed and the build enforces its absence. The retired program ladder's provisional
  "0.79.0 refactor: trapezoid-push" was never a reservation, since versions are stamps
  nothing dereferences, and the parked trapezoid opening re-versions at pickup.
- **chores-17 opened with this cycle** (wink, 2026-08-18: chores-16 passed 1000 lines).
  Created at the opening with its header only, gaining this first section at the
  close-out. The chores-16 backfill of the retirement cycle's rung rode this opening.
- **Network legs stay jj-lib's own `git` children** (fetch, push, clone, the
  default-branch query): that is jj-lib's implementation detail, not a vc-x1 spawn,
  recorded so the acceptance check's "no spawn" claim is measurable.
- **The allowlist grew beyond the opening's three members** (wink, 2026-08-19): keep gh,
  permanently allowlisted, the list specific and enumerated so people know. GitHub-side
  repo creation is the forge's REST API: no jj or git library can create a hosted repo,
  and gh is GitHub's authenticated client. Confined to init's GhCreate path, three
  commands through one helper, one file. A future in-process GitHub REST client would be
  its own Todo entry, not a rider.
- **Done sweep**: nothing migrated at this opening, the `0.78.5`..`0.78.10` run stays as
  nearby context.
- **The single-name guard earned its keep**: the opening's first build attempt used the
  suffixed version under the stable name and build.rs refused it, so the package rode the
  cycle as `vc-x1-dev` (renamed at the opening, back to `vc-x1` with bare `0.79.0` at the
  close), and rung installs never clobbered the stable binary.

### Ladder details

#### refactor: retire the remaining jj spawns opening

The six items, the chores-17 rollover, the chores-16 backfill, the refreshed inventory, and
the open-side `vc-x1-dev` rename beside the version bump.

#### refactor: port push and facade reads to jj-lib

push.rs's three `jj diff --stat` reads and the facade's `bookmark_list` /
`bookmark_list_all` become in-process jj-lib reads. Reads first: no mutation, so the risk
surface is output compatibility only. As landed:

- `jj::diff_stat` renders the CLI's stat shape in-process (`TreeDiffIterator` +
  `ContentDiff::by_line`, scaled graph, pluralized summary), keeping the constant
  `0 files changed` summary line push's completion sanity depends on. Accepted output
  delta: paths print repo-relative, so the bot side loses its cosmetic `.claude/` prefix
- the `bookmark list` spawns and their text-parser family (`find_tracked_remote`,
  `find_non_tracking_remote`) collapse into three typed view queries
  (`local_bookmark_exists`, `non_tracking_remote_of`, `has_tracked_remote`), so tracking
  state comes from `RemoteRef::is_tracked` rather than listing indentation
- the parsers' nine text-fixture unit tests retire with them, replaced by three
  fixture-driven integration-type tests per this cycle's test-spawn policy (real repos,
  real origin, the untrack transition exercised)

#### refactor: port sync repositioning to jj-lib

sync.rs's two `jj new` and two `jj rebase` spawns move in-process, where the facade's
index-lock retry can finally wrap them (the bug that motivated the program). As landed:

- two session verbs: `new_on` (via `MutableRepo::check_out`, whose edit path auto-abandons
  a discardable old `@` exactly as the CLI does) and `rebase_branch` (`-b` semantics: the
  roots of `dest..source` rebase onto dest, `finish_tx`'s `rebase_descendants` carrying
  descendants and the working-copy pointer along, conflicts written rather than refused so
  the call sites' post-rebase conflict checks keep their job)
- the reposition free functions take the one-shot wrappers, the diverged-rebase site inside
  `act_on_state` reuses the Context-cached session, and every verb's snapshot-first reload
  keeps cached sessions fresh across the mutation
- fixture tests pin the two behaviors the call sites lean on: empty-`@` auto-abandon vs
  non-empty sibling survival, and a diverged branch landing on its destination with chids
  preserved. The existing sync integration tests pass unchanged over the ported paths
- riding the rung at wink's direction: custom-family.md's new
  `## Experimental agent-file rules` section (the adopted-ahead home for family proposals),
  its two dogfood entries, and the first application of the comment-semicolon rule, 29
  conversions across the three code files this rung touches

#### refactor: port op recovery and squash to jj-lib

sync's `jj op log` read and `jj op restore`, and squash-push's `jj squash`, move
in-process. As landed:

- `jj::current_op_id` reads the head operation in-process (12-char short id, still
  printable for a user-typed `jj op restore`), and the restore becomes the session verb
  `restore_op` (op-string resolution, prefixes included, then a set-view transaction),
  with `jj::op_restore` the one-shot wrapper. The sync free functions and their `run`
  plumbing retire, push and the tests importing from the facade
- squash-push's spawn becomes the session verb `squash_into`: full-selection
  `squash_commits`, the destination's message kept (`--use-destination-message`
  semantics), the emptied source abandoned, and `finish_tx`'s `rebase_descendants`
  recreating the fresh empty `@`, matching the CLI
- the gotcha: a restored view must keep `git_refs` / `git_head` at their *current*
  values (as the CLI does), because they track what the colocated git side actually
  holds. Restoring the old records makes the next git import resurrect exactly the
  commits being undone, which is how the first draft failed
- three new fixture tests pin the snapshot/restore round trip (prefix resolution
  included), the whole-state file revert push's rollback relies on, and
  destination-message squash. The existing rollback and squash-push integration tests
  pass unchanged over the ported paths

#### refactor: port init and clone plumbing to jj-lib

repo_utils's `jj git init --colocate`, init's `jj git remote add`, clone's
`jj git clone --colocate`, and the two debug `git rev-parse` reads move to jj-lib, the
network leg staying jj-lib's own git child. As landed:

- three provisioning members join the session: `init_colocated` (the repo-creating
  constructor: `Workspace::init_colocated_git` plus the CLI's `.jj/.gitignore`, honoring
  `git.object-hash`), `add_git_remote`, and `clone_fetch` (fetch-all + import + track
  the default bookmark per `git.track-default-bookmark-on-clone`)
- `jj::git_clone_colocated` composes the whole CLI clone flow: absolutize a local-path
  source against the cwd before it is stored (the CLI's `absolute_git_url`, preserving
  bugs.md #2's fix in-process), init, add origin, reopen the session (gix caches the
  remote config it opened without, the same reason the CLI reloads), fetch/track,
  check out the default branch head, and remove a created target on failure
- the debug `git rev-parse HEAD` reads become `jj::cid_of(_, "@-")` (colocated, so the
  commit id is the git hash), and the two "is jj installed" preflight probes retire:
  init and clone no longer spawn jj, and main's version gate already errors before
  dispatch on a missing jj CLI
- every fixture-driven test already exercises the in-process init/remote-add (the
  Fixture drives `init::init`), and three new facade tests pin the colocated markers,
  a real clone's track-and-checkout shape, and failed-clone cleanup

#### chore: ban process spawning outside the version gate

clippy.toml `disallowed-methods` on `std::process::Command::new`, allowlisted at the
version gate, `$EDITOR`, and test helpers. `common::run` shrinks to the allowlisted callers
or moves into the gate module. The test-helper allowlist carries its policy (wink,
2026-08-18): a test may spawn jj for integration-type work, fixture setup with the real
installed jj and interop verification, and never as a substitute for asserting our jj-lib
paths in-process. The rung audits the existing test spawns against that line. As landed:

- the ban is two-layered: clippy.toml's `disallowed-methods` carries the reason string
  and the enumerated allowlist as comments, and Cargo.toml's
  `[lints.clippy] disallowed_methods = "deny"` makes a bare site fail plain
  `cargo clippy`. Every allowed site carries `#[allow(clippy::disallowed_methods)]`
  plus a comment naming its allowlist entry, and a recorded rg one-liner enumerates
  the enforced sites
- the allowlist grew two members beyond the opening's three, both surfaced and agreed
  during the rung: init's gh provisioning (entry 3, wink 2026-08-19: GitHub-side repo
  creation is the forge's REST API, no jj or git library can do it), and the demo run
  discovered the external CLI-test crates (`tests/common/mod.rs`), folded into the
  test-helpers entry (spawning the built binary is what a CLI test is)
- `common::run` dissolves rather than shrinks: the version gate owns its `jj -V` probe,
  init owns a `gh` helper, and the test spawns centralize in `src/test_helpers.rs`
  (`jj_ok`, new `jj_ok_at`, new `git_ok`), so no generic pass-through helper survives
  for a new caller to slip through
- the scratch-violation demo ran: a bare `Command::new("true")` in common.rs failed
  `cargo clippy --all-targets` with the reason string while every allowlisted site
  passed, and the demo also flushed out the tests/ sites the src/ grep had missed
- test-spawn audit finding: after centralization the audit surface is five helpers in
  two files, all integration-type (fixture setup, "second machine" clones kept
  deliberately independent of the clone facade under test, and interop verification
  reading real-jj/git views of our in-process writes). No spawn substitutes for an
  in-process assertion. The one violating class this cycle found, the nine text-fixture
  parser unit tests, was retired at the push/facade rung before this audit ran

#### refactor: retire the remaining jj spawns closing

Closing out the cycle: the acceptance check run and recorded above, the block moved here,
the Done entry written, the package renamed back to `vc-x1` at bare `0.79.0`, and
ARCHITECTURE.md's jj.rs passage refreshed (it still said "Mutations still spawn jj").
Gotchas, problem/solution form:

- The acceptance check's allowlist enumerated three members, but the chosen mechanism (a
  tool-agnostic `Command::new` ban) forced the gh question the jj/git-scoped opening never
  considered. Surfaced mid-rung (hard rule 10), resolved by wink: gh stays, permanently,
  with the allowlist explicit. The lesson: an acceptance check naming an allowlist should
  enumerate it against the mechanism's whole surface, not the cycle's topic.
- The opening's spawn inventory measured `src/` only, so the external CLI-test crates
  (`tests/common/mod.rs`) never appeared in it. The scratch-violation demo caught them
  because clippy lints `--all-targets`. The lesson rode into the acceptance record:
  measure with the enforcement tool's scope, not a narrower grep.
- Found while testing the closing tree (wink, 2026-08-19): init still writes
  `.vc-config.toml`, starting every new member on the legacy carrier the 0.78.8 cycle
  superseded. Not this cycle's scope: recorded as a new line on the "Finish the vc-config
  surface" Todo entry's regenerate rung.

## docs: pin two rules and close the convergence record

- [[16]] 0.79.1 [docs: pin two rules and close the convergence record][8]

A single-step cycle, so the one commit is the close-out. The cycle's own work is an
agent-file change, which is why it runs alone rather than as a rung of a feature cycle.

### Problem

Two records were owed at once. iiac-perf's 2026-08-15 convergence review, whose three proposals
were accepted without modification at the 0.78.8 trial rung, still had no `outcome-*` and so sat
as open traffic. And two rules adopted mid-cycle (the `(done)` flip on acceptance, 2026-08-18,
and comment semicolons converting on touch, 2026-08-19) were binding only from
`custom-family.md`'s `## Experimental agent-file rules`, a parking place whose own text calls a
long-lived entry a process failure. The pinned files still said the opposite: the checklist
flipped `(done)` before validation, and the semicolon rule asked at every alteration.

### Solution

Their record is closed in `../vc-x1-messages/vc-x1.md` with `outcome-*` pointing at the
landed trial rung (chores-16, permalinked at `main` e28cbd6b4983), and a reply record in
`iiac-perf.md` says so, naming the two rules below as proposals to follow in their own
record once this cycle lands. The review's remaining items moved to the 0816-proposal Todo
entry, which is where their reply rides.

Each rule now lives in the pinned file it named, and the diff against the payload carries
both to the family as proposals:

- the flip: cycle-checklists.md's per-commit step 3 keeps the rung `(current)` and step 7
  flips it once the work review completes, and cycle-protocol.md's per-commit flow says the
  same at its steps 3 and 7
- the conversion: prose.md's Semicolons history clause changed from ask-on-alter to
  convert-on-touch (whole file, code spans exempt, files outside the diff never touched),
  and code.md gained a `Comments are prose` section stating it for source files. The
  Typeable-punctuation cross-reference that contrasted itself with "ask-on-alter" now
  contrasts with "convert-on-touch"
- the two entries left `custom-family.md` (the section stays, `_None._`, since the pattern
  itself is still a proposal), and their dogfood entries retired, the 2026-08-19 entry
  trimmed to its in-flight second finding

### Acceptance check

Passed at the close-out: `grep -n 'before validation' agent-data/cycle-checklists.md` is
empty, prose.md no longer contains "ask the user whether", the experimental section holds no
entries, dogfood.md's only 2026-08-19 entry is the section-pattern finding, and
`grep -c '^- outcome-' ../vc-x1-messages/vc-x1.md` counts both records closed (four field lines).

### Deliberation

Single-step: the edits are four short passages plus retirements, so a ladder would only
spread one change over three commits. The rules landed as written in the experimental
section, with one scope note: the semicolon rule covered prose, and source-file comments are
prose that the history clause treated as ask-on-alter, so the change to that clause is the
rule change and code.md's section is its application. The section pattern (adopted-ahead
rules parked in the project layer) is not resolved here and rides the 0816-proposal.

The cycle opened as the pinning alone, with the reply to iiac-perf waiting on this cycle's
SHA. wink's correction at review: the acceptance without modification is the 0.78.8 trial
rung, already on `main`, so the record's outcome can point there today and the pinning is a
separate, later proposal. The close moved into this cycle and the title widened.

The Done sweep migrated nothing: the four 0.78.x entries are the convergence context the top
two Todo entries still cite.

### Ladder details

#### docs: pin two rules and close the convergence record

The whole cycle in one commit. Gotchas:

- Problem: the dogfood rewrite first named the pinning cycle by its version, which prose.md
  bans outside the version-of-record. Solution: it names the cycle title instead, the one
  identifier a step has.
- Problem: the block move's `## In Progress` reset spliced at the first `## Todo` match,
  which is a code-span mention in the file's intro rather than the heading, so the intro and
  the block came back duplicated and only the diff stat (+95 for a one-entry change) gave it
  away. Solution: anchor such splices on the heading line (`^## Todo$`), and read the stat
  before calling the move done.

## docs: empty custom-family into the pinned set and config

- [[30]] 0.80.0-0 [docs: empty custom-family into the pinned set and config opening][17]
- [[31]] 0.80.0-1 [feat: agent naming in config and CLI][18]
- [[32]] 0.80.0-2 [feat: add the family and validate tables to the schema][19]
- [[33]] 0.80.0-3 [feat: add the validate subcommand][20]
- [[34]] 0.80.0-4 [docs: pin messaging into agent-data][21]
- [[35]] 0.80.0-5 [docs: retire custom-family.md][22]
- [[36]] 0.80.0-6 [feat: rename validate-bot to validate-agent][24]
- [[37]] 0.80.0 [docs: empty custom-family into the pinned set and config closing][23]

### Problem
Both members' custom* files hold family infrastructure (the messaging behavior, the member
identity and sibling-repo paths, the validation commands) that lives there only because the
pinned set and the config schema had no home for it. While it sits there the agent-files
cannot converge byte-for-byte, every pinned checklist has to say "the commands are in
custom.md" instead of naming one command, and a session that skips the project layer misses
binding family behavior. The entry was raised 2026-08-16 (wink + bot) as the 0816-proposal,
to be implemented here first and proposed to iiac-perf as a working result.

### Solution
Each kind of content went to its proper home and `custom-family.md` is gone:

- the agent naming landed first (`repos.agent`, `[agent-session]`, `--scope=agent`, the
  `agent-session` subcommand), pulled forward from the "Finish the vc-config surface" Todo so
  the two new tables were born under the new vocabulary. Old spellings are rejected with
  fix-its, never aliased
- the family facts are a work-side `[family]` table (member, template, messages), and the
  validation commands a work-side `[validate]` table of `str-list` keys, a new kind the TOML
  reader learned arrays for
- `vc-x1 validate [--fast]` runs the chosen table, one invocation per element, stopping at
  the first failure, so the pinned checklists name one command for every medium. Its spawn is
  clippy.toml's allowlist entry 5, the list's one reopening
- `agent-data/messaging.md` pins the acquaint check and request-becomes-entry thinly, the
  messages repo's README staying the protocol authority and the paths read from `[family]`
- `custom.md` holds the medium prose (the version-bump promise, the single-name convention)
  in a `## Medium` section and nothing in its conventions section, and AGENTS.md states what
  the experimental-rules section resolved to: adopted-ahead rules live in the pinned file as
  the diff, never in a holding section

Outside this cycle, as planned: the payload update (backlog "Update the template payload, and
empty the three-way diff") and the reply to iiac-perf, which needs the landed SHA. The
install-then-respell order matters for both, since a 0.79.x binary reads a respelled config as
single-repo with six unknown keys, silently, while a 0.80.0 binary rejects the old spelling
loudly.

### Acceptance check
1. `diff` of every pinned file (`AGENTS.md`, `agent-data/*`) against the template payload is
   empty except for this cycle's own proposals, each of which the chores section names.
2. `custom.md` differs from the payload only inside its medium section, and
   `custom-family.md` does not exist.
3. `vc-x1 validate` runs the full table and `vc-x1 validate --fast` the fast one, each
   command a separate invocation whose exit status is checked, demonstrated once with a
   failing command in the table.
4. `vc-x1 config --validate` is clean on both sides with the new tables present.
5. The old spellings (`repos.bot`, `[bot-session]`, `--scope=bot`) are rejected, each
   rejection printing its fix-it, shown by a test, with values and the `/.claude/` ochid label
   unchanged.

Run at close-out (2026-08-21):

1. Passes, read against the payload's actual state. The diff holds this cycle's proposals
   (AGENTS.md: rule 0 and the project-layer section no longer list validation commands, the
   `messaging.md` file-map entry, the adopted-ahead rule, the new `agent-data/messaging.md`,
   and the checklists' and protocol's validation steps naming `vc-x1 validate`) and, beside
   them, the previous cycle's still-open proposals (the `(done)` flip timing, comments-are-prose,
   convert-on-touch), since the payload has not been updated since 0.79.1. Nothing else.
2. Fails as literally worded, for a reason the check did not anticipate: the payload's
   `custom.md` is an older shape (a medium section with the commands inline and a
   mailbox-parameters line), so `custom.md` differs from it in its header text and its
   sections, not only inside the medium section. `custom-family.md` does not exist. The
   payload update owns the fix, and the check's intent (custom.md carries only the medium
   prose and an empty conventions section) holds.
3. Passes. The full table ran at every rung from rung 4 on, the fast table at the close-out,
   and `["true", "false", "touch never"]` stops at 2/3 with `false` at exit 1, `never` not
   created, exit status 1.
4. Passes: `config: all checks passed` on both sides with `[family]` and `[validate]`
   present on the work side.
5. Passes: `old_agent_spellings_rejected_with_fixit`, `parse_scope_rejects_old_bot_keywords`,
   `parse_target_rejects_old_bot_keywords`, and `cli_bot_session_old_name_rejected`, with
   `repos.agent = ".claude"` and the `/.claude/` label unchanged.

### Deliberation
Multi-step: two feature rungs and two docs rungs, each reviewable alone, with the acceptance
check meaningful only once all four land. The one design fork was where the machine facts go.
The entry proposed a user-level `~/.config/vc-x1/settings.{md|toml}`, and the backlog's
"Drop the global config" entry wants the user tier gone. Decided (wink, 2026-08-21): no user
tier. The facts are sibling paths relative to the workspace, the same species as
`repos.bot = ".claude"`, and identity and credentials already live in jj's and git's own
config, so vc-x1 holding a third copy of either is the wrong shape. The same conversation
settled that `init` takes a URL or a path only: a bare name has no host-neutral meaning, and
delegating it to `gh` would make the convenience GitHub-only. That decision is recorded on
the "Drop the global config" entry, where it is acted on.

Inserted 2026-08-21 (wink): the agent-naming rung, ahead of the schema-tables rung. The
naming change was laddered as the first rung of "Finish the vc-config surface", but adding
`[family]` and `[validate]` under the old vocabulary would have had the later rename respell
them, so the rung moves here and that Todo shrinks to four rungs. Sized at one ordinary
feature rung: the schema is generated from `vc-config.md` at build time, so a missed site
fails the build or a test rather than passing silently. It is a breaking schema change for
the family's repos, called out in the commit and in the reply to iiac-perf.

Version: minor (0.80.0), since a subcommand is added, the config schema gains two tables, and
the agent-naming rung respells three keys.

### Ladder details

#### docs: empty custom-family into the pinned set and config opening
The block above, the chores-17 as-built backfill for the jj-spawns cycle (missed at the
previous close-out) and the pinning cycle, bugs.md #10 (init rejects a pre-created GitHub
repo) and #11 (init cannot publish to gitlab.com), and the "Drop the global
config" entry's decision notes, including the measured gitlab.com push-to-create refusal.

#### feat: agent naming in config and CLI
The agent side is `agent` everywhere a user types or reads it: the keys `repos.agent` and
`[agent-session].*`, the home `workspace-agent`, `--scope=agent` (and `work,agent` /
`agent,work`, the `config` target keywords included), and the `agent-session` subcommand. The
old spellings are rejected, never aliased: `legacy_vc_config::reject_old_agent_keys` runs
beside the legacy-schema rejection in every resolver and lists each old key found with its
replacement (one edit clears them all), `parse_scope` names the respelled keyword, and a hidden
`bot-session` subcommand prints the fix-it for any flags. Rust identifiers (`Side::Bot`,
`bot_session`, `Home::WorkspaceBot`) keep their names: the rung is vocabulary, and an identifier
sweep is a refactor of its own. Values untouched: `repos.agent = ".claude"` and the `/.claude/`
ochid label (test-pinned). Both sides' `.vc-config.md`, `vc-config.md`'s key docs, and README's
CLI examples are respelled (README's stale init-layout block updated to the md carrier in
passing). README's "bot repo" prose and the pinned files stay as written: the concept's name is
the pinned vocabulary's, a convention cycle's to change. Gotcha: the schema's generated
constants are named from the key path, so `BOT_SESSION_*_DEFAULT` became
`AGENT_SESSION_*_DEFAULT` on the rename, and the build pointed at every consumer.
Also in this rung: `config`'s per-side header reads `agent:` (wink spotted the old label
while reviewing, and that review raised the print-once Todo), and the touched source files'
comment semicolons were converted per code.md, a delegated mechanical pass.

#### feat: add the family and validate tables to the schema
The config schema had no place for the family facts (member name, template and messages paths)
or the validation commands, so they lived in `custom-family.md` prose, and the schema could
not hold a list of commands at all: its only list kind is a comma-separated string, and a
command line has commas and spaces of its own.

- five keys in `vc-config.md`: `family.member`, `family.template`, `family.messages` (strings)
  and `validate.full`, `validate.fast` (command lists), work side only
- a new `str-list` kind: a TOML array of strings, one element per command
- the TOML reader learns arrays, on one line or spread over several, and `toml_get_list`
  splits one into its elements, naming the key when the value is not a proper array
- `config --validate` accepts the keys on the work side, reports them unknown on the agent
  side, and checks that a command list is really a list
- the work side's `.vc-config.md` carries both tables with this repo's real values
- gotcha: `build.rs` expected quoted-string examples, so an array example is kept as its raw
  text and rendered as written

#### feat: add the validate subcommand
The checklists could only say "the commands are in custom.md", and a session had to read prose
and type each one, so the validation step depended on the reader and the commands had no
runner that checked every exit status the same way.

- `vc-x1 validate [--fast] [-R DIR]` reads `validate.full` (or `fast`) from the work side's
  config and runs each element as one command, in order, from the work repo root, printing
  each before it runs
- the first failure stops the run, naming the command, its exit status, and where in the
  table it stopped, and the subcommand exits non-zero (demonstrated: `["true", "false",
  "touch never"]` stops at 2/3 with `false` at exit 1, and `never` is not created)
- an empty or missing table is an error naming the key, since nothing to run is not a pass
- elements split on whitespace into a program and its arguments, no shell in between, so the
  status the run sees is the command's own
- the spawn needed an allowlist entry: clippy.toml's list was closed (wink, 2026-08-19), and
  this cycle's agreed plan opens it once, as entry 5, for the subcommand whose whole job is
  running the configured commands. Flagged at the rung, for the review of the delegation
- from this rung on, the per-commit validation is `vc-x1-dev validate` (the release `vc-x1`
  cannot read this workspace's 0.80.0 config), and the checklists are respelled at the next
  rung

#### docs: pin messaging into agent-data
The acquaint check and the request-becomes-entry rule were family policy parked in
`custom-family.md`, which a session that skipped the project layer never read, and the pinned
checklists could only point at custom.md for the validation commands.

- `agent-data/messaging.md`, thin: the repo and record come from `[family]`, the acquaint
  check reads the record file by its fields, a request becomes an entry and the reply cites
  it, and the messages repo's README stays the protocol authority on everything it covers
- AGENTS.md's file map gains it, read at acquaint when the work side's config has a
  `[family]` table, and rule 0 and the project-layer section stop listing "validation
  commands" among what custom.md holds
- the per-commit and ladder checklists, and the protocol's per-commit flow, name
  `vc-x1 validate` and `vc-x1 validate --fast` in place of "the commands are in custom.md",
  the protocol keeping the cargo cycle as its Rust example in one parenthesis
- all four pinned files swept to zero semicolons, as the agent-files are

#### docs: retire custom-family.md
With the facts in `[family]`, the commands in `[validate]`, and the messaging behavior pinned,
the file held only the medium prose and a section pattern, and its existence kept `custom.md`
one pointer line away from the payload's shape.

- `custom.md` gains a `## Medium` section with the artifact, the version-bump promise, and the
  single-name convention, and says where the validation commands went. Its conventions section
  is `_None._`, and the dogfood-log pointer moves in as its own section
- the experimental-rules section pattern lands in AGENTS.md's Changing the agent-files as the
  rule it resolved to: an adopted-ahead rule lives in the pinned file it belongs to, as the
  diff, never in a holding section, since the holding section hid rules from review and from
  sessions that skipped the layer (the measured failure, kept project-neutral in the pinned
  text)
- `custom-family.md` deleted, its pointer line gone. Backlog entry "`[private]` config
  table" deleted as superseded: the facts moved into a typed, validated `[family]` table,
  which is better than an opaque one. The backlog renumbered (`fix-todo`), and TODO.md's one
  numeric citation past it (`backlog #53` -> `#52`) followed
- the payload's `custom.md` still predates this shape (it carries a medium section with the
  commands inline and a mailbox-parameters line), which the backlog's "Update the template
  payload" entry takes, with the acceptance check's diff read against that

#### feat: rename validate-bot to validate-agent
Inserted at the close-out review (wink, 2026-08-21): the agent-naming rung had left
`validate-bot` as the one subcommand still naming the side by the old word, and the close-out's
README pass put it in front of the reviewer. `validate-agent` is the name, same flags, and
`validate-bot` is a hidden name printing the fix-it for any flags, as `bot-session` does. A
code change, so it is its own rung ahead of the close-out rather than a line in it, and the
rebase of the prepared close-out over it conflicted on the version, the lock, and the README,
resolved by hand.

#### docs: empty custom-family into the pinned set and config closing
Problem: rungs 4 to 6 and the close-out ran under a scoped delegation (wink, 2026-08-21:
describe and push each rung without approval, prepare the close-out but do not describe or
push it), so the per-rung review stops were waived. Solution: every other part of the flow ran
as usual (records, validation, the bookmark), the one judgment the delegation did not
anticipate (reopening clippy.toml's closed allowlist for the validate spawn) is flagged in its
rung and here, and the close-out's own review is the user's.

Problem: the rung-2 description was written before the user had accepted the work, because
the bot followed the squash-form "Ladder (sub-cycle)" checklist in a multi-commit cycle, the
word "ladder" naming both. Solution: caught at review, pushed under the multi-commit flow, and
Todo "Simplify and combine the cycle-* agent-files into AGENTS.md" records the cause and the
fix.

Problem: the release `vc-x1` (0.79.1) cannot drive this workspace once its config is
respelled, so every push in the cycle was `vc-x1-dev push`. Solution: the single-name
convention working as intended on a long-lived branch, over once this lands and `main`
installs as `vc-x1`.

## docs: fold the cycle agent-files into AGENTS.md

### Problem
The cycle protocol lives in `cycle-protocol.md` and its summaries in `cycle-checklists.md`,
and the two disagree just enough to mislead a reader who opens one and not the other.
Measured at the agent-naming rung: the bot read the checklists file, took the "Ladder
(sub-cycle) checklist" because the working record calls its rung list `#### Ladder`, skipped
the protocol file that tells the multi-commit and squash-form shapes apart, and so described
the rung and proposed "start the next rung" where the next step was `vc-x1 push`. Hard rule 7
and the checklists' own preamble already say to read both, so another rule is not the answer.
Beside it, the code and wink now say `agent` for the second repo while the Terminology section
still defines "bot repo" as the standard name, so a bot reading AGENTS.md keeps saying "bot".

### Solution
`AGENTS.md` gained `## Cycle protocol`, one account of the cycle in the order a cycle meets
it (bookmark, opening, per-rung flow, committing vs pushing, description, pushing with its
policy and the at-rest contract, drafts, close-out, chores sections with backfill, local
ladders), 330 lines against the two files' 1170, and the two files are deleted with every
live link repointed. The at-rest contract is one passage: the agent runs `vc-x1 push`, stops,
and the user runs `vc-x1 squash-push -R .claude`, so "clean" means both repos' `@` empty.
"Ladder (sub-cycle)" is retired for "local ladder", and the jj mechanics the account does not
carry (the trapezoid recipe, the ladder moves) moved to jj.md. The terminology moved with it:
"agent repo" is the standard name, `.claude` its path, "bot repo" retired, and the actor is
"the agent". The Cycle term was simplified (one change, opening to closing, as one commit or
a ladder of them) and the "Work" stage name retired. Convention work, so every change is a
family proposal as the diff against the payload.

### Acceptance check
1. `agent-data/cycle-protocol.md` and `agent-data/cycle-checklists.md` do not exist, and
   `grep -rn 'cycle-protocol\|cycle-checklists' AGENTS.md custom.md agent-data notes TODO.md
   README.md` finds nothing but historical records (chores and done.md), no live link.
2. `AGENTS.md` holds one account of the cycle: opening, per-rung flow, push, close-out, and
   the at-rest contract naming `vc-x1 push` and `vc-x1 squash-push` in one passage. The
   string "Ladder (sub-cycle)" appears in no agent-file.
3. `grep -rn 'bot repo\|bot-repo' AGENTS.md custom.md agent-data` finds only the retired-term
   note in Terminology, and "agent repo" is the defined name.
4. A fresh session's acquaint (read `AGENTS.md`, `custom.md`, and what they point at) reaches
   the per-rung `vc-x1 push` and the at-rest contract without opening a second cycle file.
5. `vc-x1 validate` passes at every rung.

Run at close-out (2026-08-21):

1. Passes, with one finding. The files are gone, and the grep over the named paths finds
   only historical mentions. Widening it to all of `notes/` found four live links in
   `notes/refactor-20260716.md` (a program record, three of them already broken by an
   earlier move of the file out of `notes/`), repointed at jj.md's recipe in this commit.
2. Passes in substance, fails as literally worded: the account is twelve subsections under
   `## Cycle protocol`, and `#### At rest: push, stop, squash-push` names both commands in
   one passage. "Ladder (sub-cycle)" appears once, as the retired-name note inside Local
   ladders, which the check did not anticipate and which is deliberate, so a reader meeting
   the old name in a chores file can find its successor.
3. Passes: the one hit is the dated retired-term note in Terminology, and "agent repo" is the
   defined name.
4. Passes, by reading: `AGENTS.md` reaches `vc-x1 push <bookmark>` at the per-rung flow's
   step 9 and the at-rest contract two sections later, and `custom.md` points at nothing
   cycle-related. Not measured with a fresh session, which the next cycle's acquaint is.
5. Passes: the full table ran and passed at the opening and each of the three rungs, and
   at this close-out.

### Ladder
- [[38]] 0.80.1-0 [docs: fold the cycle agent-files into AGENTS.md opening][25]
- [[39]] 0.80.1-1 [docs: write the cycle account into AGENTS.md][26]
- [[40]] 0.80.1-2 [docs: retire cycle-protocol.md and cycle-checklists.md][27]
- [[41]] 0.80.1-3 [docs: rename bot repo to agent repo in the agent-files][28]
- [[42]] 0.80.1 [docs: fold the cycle agent-files into AGENTS.md closing][29]

### Deliberation
Multi-step, three Work rungs: writing the merged account is the design work and reviewable on
its own, retiring the two files and repointing every cross-reference (jj.md, prose.md,
notes.md, versioning.md, messaging.md, custom.md, TODO.md, README.md) is mechanical and
separate so the diff of the account is not buried in link churn, and the terminology rename
touches every agent-file and is its own proposal to the family. The order puts the account
first so the retire rung can be a pure delete-and-repoint. Version: patch (0.80.1), docs only,
no CLI or schema change.

The cycle's bookmark is `fold-cycle-info`, not the title's slug (wink, 2026-08-21, at the
opening): a taken exception to hard rule 13's naming clause, the bookmark-per-cycle discipline
itself unchanged.

### Ladder details

#### docs: fold the cycle agent-files into AGENTS.md opening
The block above, the Done sweep of the 0.78.x entries into done.md, and the version bump.

#### docs: write the cycle account into AGENTS.md
Write one account of the cycle into `AGENTS.md`: the three stages, the bookmark, the six
provisional items, the per-rung flow ending in `vc-x1 push`, the review stops, the close-out
move and landing, and the at-rest contract (push both repos, stop, the user squash-pushes the
agent repo). The two source files stay in place for this rung so the review can diff the
account against them.

Landed as `## Cycle protocol`, 330 lines against the sources' 1170, twelve subsections in the
order a cycle meets them: bookmark, opening, per-rung flow, committing vs pushing, commit
description, pushing (policy, before any push, at rest), drafts, close-out, chores sections
with backfill, local ladders. Design points:

- the at-rest contract is the new `#### At rest: push, stop, squash-push`, three numbered
  parts with hard rule 3 as the middle one, absorbing the protocol's "After push or
  squash-push", ".claude cadence", and the squash-push half of its push Recovery. It is the
  first statement of the user's `squash-push` as a step of the workflow rather than a command
- "Ladder (sub-cycle)" is retired for **local ladder**, defined against the working record's
  `#### Ladder` in the section itself, and Iterative work folds into it as the one-commit case
- the hard rules' wording is quoted by number rather than restated (rules 1, 2, 3, 4, 5, 9,
  10, 13), so each rule has one text
- the section already says "agent repo", ahead of the rename rung, since writing "bot repo"
  into new text the next rung would respell was churn for nothing, and wink's review respelled
  two more (the file's title, "Agent Instructions", and the dual-repo model's "Agent repo"
  item). The Terminology section still defines the old name until that rung
- what did not merge: the trapezoid recipe with its details and recovery (a hundred lines of
  jj mechanics), the local-ladder navigation and recovery moves, and push's interim-push and
  out-of-band recovery notes. The retire rung moves the first two into jj.md and drops the
  third as covered by "rerunning is safe". Until then the account links the protocol for
  them, the only two links into the files this cycle retires
- a pre-existing dead anchor surfaced: `prose.md#cycle-bookend-titles` is a bold paragraph,
  not a heading, and notes.md and the protocol link it too. The account links the enclosing
  heading. The other two sites are the retire rung's

#### docs: retire cycle-protocol.md and cycle-checklists.md
Delete the two files and repoint every live link, including hard rules 1, 2, 3, 7, and 13 and
the File map, at the AGENTS.md sections. Recovery procedures the account does not carry move
to jj.md or are dropped with a note.

Landed. The two files are deleted and every live link repoints at the account: AGENTS.md's
hard rules 1, 2, 3, 7, and 13 and its File map, jj.md, prose.md, notes.md, versioning.md,
README.md, notes/README.md, the In Progress intro here, and one backlog entry. Points:

- hard rule 7 reworded from "read the checklist" to "read the protocol step", naming the
  per-rung flow and Before any push, since the checklists file it named is gone and the
  account is what is read at the moment of action
- jj.md gains `## Trapezoid close-out recipe` (the recipe, its details, its recovery) and
  `## Local ladders` (the navigation moves, the squash, the recovery), each opening with a
  pointer to the AGENTS.md section that states the rule. The recipe's diagram was redrawn
  with typeable characters, the original having box-drawing glyphs the punctuation rule
  forbids
- dropped, not moved: the protocol's interim-push and out-of-band recovery notes (covered by
  "rerunning is safe" in Committing vs pushing), the ESC-ESC override (the user can interrupt
  is stated in the per-rung flow), the `/exit` note in ".claude cadence", and the version
  numbers in Work-N's heading, which the manifest owns
- the File map's "checklists first, rationale after" ordering note went with the files, and
  the cycle protocol is named as in this file
- notes.md's link to the dead `prose.md#cycle-bookend-titles` anchor repointed at the
  enclosing heading, the same fix the account took
- historical mentions of the file names in chores, done.md, dogfood.md, and older Todo entries
  stay as written: records, not links
- at review (wink): the stage name "Work" and its terms "Work rung" / "Work commit" /
  "Work-N" retired. A rung between the bookends is a commit, and the version scheme's middle
  is "the commits between". "Work-repo commit" already carried the repo sense unambiguously,
  so the Terminology note shrinks to that one line

#### docs: rename bot repo to agent repo in the agent-files
Terminology defines "agent repo" as the standard name with `.claude` as its path and "bot
repo" as retired, and every pinned file follows.

Landed: 33 sites across AGENTS.md, jj.md, prose.md, and notes.md (custom.md and messaging.md
had none). Points:

- the actor renamed with the repo: "the rules bind the agent", "the agent runs `vc-x1 push`".
  One word for the thing the file instructs, matching its title
- what stays `bot` is what the code spells that way, quoted in code spans: the `commit-bot`
  and `squash-push-bot` push stages, and the pre-0.80.0 config spelling the validator names
  in its fix-it. The Terminology note says so, so a reader meeting `commit-bot` in push
  output does not take it for drift
- the Terminology note's CLI facts were stale and corrected on the way: `--scope` takes
  `agent`, the config is `.vc-config.md`, `[repos]` spells the side `agent`, and jj.md's
  two-sided config example now matches
- left alone: jj.md's `## .vc-config.toml` heading, a stale file name that other links may
  depend on, for a correction of its own

#### docs: fold the cycle agent-files into AGENTS.md closing
Closing out the cycle.

- Problem: the acceptance check's grep named its paths (`AGENTS.md custom.md agent-data notes
  TODO.md README.md`) but excused "chores and done.md" by name, so a records file outside
  that excuse with live links, `notes/refactor-20260716.md`, was found only because the
  closing widened the grep. Solution: an acceptance grep excuses record files by a pattern it
  runs, not by a list it remembers, and the four links were repointed here.
- Problem: a check worded as "appears in no agent-file" cannot pass when the rung that
  retires a name also has to say what the name was. Solution: recorded as a substantive
  pass, and a future check of a rename says "appears only as the retired-name note".
- Problem: the acceptance grep also treated `TODO.md`'s mentions as records, and four of
  them were live instructions naming the retired file as the place a future cycle would
  edit, beside two "Shared-doc sync" Todos whose whole ask (as-built `[[N]]` rungs, the
  per-commit narrative, a coordinated family sync) is now pinned in AGENTS.md and the
  convergence model. Solution: the four repointed at AGENTS.md's sections, the two Todos
  dissolved (the Done entry says so), and the wider sweep for pre-0.80 vocabulary in the
  Todo files filed as a backlog entry with its species listed, since changing what an entry
  asks for is a cycle of its own, not close-out bookkeeping. History lines keep their names.

## docs: halve AGENTS.md into rationale.md

### Problem

AGENTS.md is the one file every session loads, and about half of it is argument rather than
rule: the "Why" paragraphs, the incident stories behind the acceptance check and the
https-remote rule, the delegation tiers' justification, the Terminology notes' "because"
clauses. A session needs the rule. The argument is for whoever would change the rule, and for
the family at convergence, and it keeps the rule from being simplified away by an editor who
does not know its cost, so it moves rather than dies.

### Solution

AGENTS.md went from 593 lines to 340 in nine rungs. The why moved to the new pinned
`agent-data/rationale.md`, whose headings mirror AGENTS.md's so `[why](...#<same-slug>)` reaches
an entry or an explicit blank, with the evidence (chores sections, dates, the spawn story) riding
along. The mechanics AGENTS.md restated moved to the satellites that own them (jj.md: vc-x1 push
behaviors, close-out shapes, the local-ladder contract, bookmark reshaping; notes.md: the In
Progress block, the close-out move; prose.md: commit description details). Two rules were added
along the way, backfill as the Opening's first step and prose.md's unmarked label-colon lead,
and two tightening passes, the agent's and wink's, cut the rule text itself, the second settling
eight points item by item and changing several rules (repos always hyphenated, Bump before Work,
push expected every rung, shared title and body across the repos). The satellites' own why is
still inline and is the next convention cycle.

### Acceptance check

Run at close-out, 2026-08-22:

- `wc -l AGENTS.md`: 340, from 593, 57%. The original 297 was not reached and the check was
  revised at the first move rung to report the count (Deliberation). What remains is rule text
  at the length wink's pass left it
- every `##` / `###` / `####` heading in rationale.md has a same-slug heading in AGENTS.md:
  pass (diff of the heading lists is empty), six entries `_None recorded._`
- `vc-x1 validate` clean, and a scratch anchor check over the agent-files and TODO.md found
  only example placeholders: pass
- the two move rungs read as words moved: pass, each reviewed as deletions plus links
- the backfill and label-colon rungs: pass, the Opening's step 1 is Backfill with its `rg`
  check, and every lead in AGENTS.md is an unmarked label ending in a colon
- zero why-words in AGENTS.md outside its 17 `[why]` links ("because", "measured", "Why:"):
  pass

### Ladder

- [[53]] 0.80.2-0 [docs: halve AGENTS.md into rationale.md opening][43]
- [[54]] 0.80.2-1 [docs: make backfill the opening's first step][48]
- [[55]] 0.80.2-2 [docs: seed rationale.md and the Rationale term][44]
- [[56]] 0.80.2-3 [docs: move the cycle protocol's why into rationale.md][45]
- [[57]] 0.80.2-4 [docs: move the rest of AGENTS.md's why into rationale.md][46]
- [[58]] 0.80.2-5 [docs: point AGENTS.md's restated mechanics at the satellites][50]
- [[59]] 0.80.2-6 [docs: label-colon form for AGENTS.md's lists and rules][49]
- [[60]] 0.80.2-7 [docs: tighten AGENTS.md's prose][51]
- [[61]] 0.80.2-8 [docs: Winks AGENTS.md's tightening][52]
- [[62]] 0.80.2 [docs: halve AGENTS.md into rationale.md closing][47]

### Deliberation

Two move rungs rather than one, split at the `## Cycle protocol` tree, so each diff is
reviewable in a sitting. The judgment line is "boundary sentences stay": a sentence saying what
a rule does not cover is the rule. The borderline calls turned out to be few: the why was about
a tenth of the file. A patch bump rather than minor, as the fold cycle was: the agent-files set
gains a file but the shape of the system is unchanged, words moved. Satellites deferred per the
Todo that filed this.

A rule change added at the opening (wink, 2026-08-21): the 0.80.0 and 0.80.1 as-built rungs
were both found unfilled at this opening, the backfill missed at two consecutive openings.
Close-out step 8 says the edits "ride the next push", which names no owner, and the Opening's
steps never mention it, so the only place backfill is named is the one moment rule 3 forbids
doing it. Convention work does not ride a feature ladder, but this is an AGENTS.md cycle, so
the rule rides as its own rung and its own commit, and the acceptance check's "words moved"
clause narrows to the two move rungs rather than the branch: the check changed, and this is
why. The durable fix is tooling, a validate element failing on a `[[N]]` rung whose commit is
on `main`, filed at the closing as a backlog entry.

A second rule change, found at the backfill rung's review (wink, 2026-08-21): the new Opening
step 1 read as "every as-built ladder ...", its bold verb skipped as a label, so the sentence
lost its imperative. The fix is a form rule, label-colon with a sentence complete without the
label, stated in prose.md and applied across AGENTS.md's step lists and hard rules, the rules
gaining short names as their labels. Its own rung, after the two move rungs, so the sweep
runs over the halved file and the family sees the final shape in one diff. The acceptance
check's "one rule change" becomes two, this one named.

The line target revised at the first move rung's review (wink, 2026-08-21): the faithful
why-lift took the protocol tree from 348 to 319 lines and AGENTS.md from 593 to 585, so the
"about half is argument" premise does not hold for the tree, which is mostly rule and
mechanics. This is the revised-toward-what-was-achieved case the Opening warns about, argued
here rather than slipped: the cycle's real deliverable is zero words of why in AGENTS.md, the
line count was a guess, and the number stays in the check as a reported measure rather than
a pass/fail gate. Alongside, a further rung widens the move: mechanics that restate a
satellite (the jj diff commands, the rungs-are-named paragraph, the push behaviors, the
close-out shape definitions) become pointers, which is no longer words moved to rationale.md
and touches the satellites this cycle deferred, so it is its own rung with its own diff.

### Ladder details

#### docs: halve AGENTS.md into rationale.md opening

The cycle's bookmark, this block, the Done sweep, and the version bump.

#### docs: make backfill the opening's first step

Problem: backfill was specified only at close-out, as "the edits ride the next push", with no
owner and no place in the Opening's steps, so it was missed at two consecutive openings.
Solution: Opening step 1, ahead of the bookmark, backfills every as-built ladder whose commits
have landed, with the check spelled out (`rg '\[\[N\]\]' notes/chores/`, a hit outside a code
span is owed work), and close-out step 8 names the next opening as the owner of its debt.

- first, not folded into the Done sweep: the sweep is already a compound step, and a step with
  two halves is where the second half hides
- a lightweight cycle with no opening commit carries the step in its first commit, said in the
  Opening's own parenthesis rather than as a fourth place the rule lives
- the check is a grep, not a tool: a `validate` element that fails on an unfilled rung whose
  commit is on `main` is the durable fix, filed at the closing as a backlog entry

#### docs: seed rationale.md and the Rationale term

Problem: nothing links a rule to its why, and a why has no home outside the rule's own
paragraph. Solution: `agent-data/rationale.md` with the heading skeleton mirroring AGENTS.md
1:1, the **Rationale** term in Terminology stating the link pattern once, and the File map line.

- every AGENTS.md heading is mirrored, each holding `_None recorded._` until a move fills it,
  so an unfilled heading after the moves is a visible finding (a rule with no written why)
  rather than an absent anchor. Six empties remain at the closing, kept as findings
- the file's "How to read this file" states the three rules of the move once: headings
  mirror, an entry is why then evidence, a boundary sentence is not rationale. The move rungs
  apply them and do not restate them
- the term in AGENTS.md carries the boundary-sentence test, since that is what tells an
  editor which side of the line a sentence is on, and nothing else: the argument for having
  the file is the file's own first entry, once the Terminology why moves

#### docs: move the cycle protocol's why into rationale.md

Problem: the `## Cycle protocol` tree carries the bookmark why, the acceptance-check story, the
delegation tiers' justification, the at-rest explanation, and the one-home argument, the bulk
of AGENTS.md's argument. Solution: each moves under its mirrored heading, the rule keeping a
`[why]` link.

- thirteen of the tree's headings took an entry, ten stay `_None recorded._` (the intro-less
  ones: Pushing, Before any push, and the ones outside the tree)
- the lift was faithful to the boundary test and removed about 30 net lines from the
  protocol's 348: the tree is mostly rule and mechanics, and the Todo's "about half is
  argument" premise does not hold for it. Surfaced at this rung's review as a finding
  against the acceptance check's 297-line target, with the call on what to do left to
  the user
- the `[why]` link sits on the sentence it explains, or on the section intro when the entry
  covers several

#### docs: move the rest of AGENTS.md's why into rationale.md

Problem: the why outside the protocol is scattered: the Hard rules intro, the Terminology
"because" clauses, the Working practices stories (https remotes, the 2026-08-05 quoting note),
the measured line in Changing the agent-files, the custom.md section's argument. Solution: the
same move, and the size check against the target.

- five more headings filled (Hard rules, Terminology, Working practices, Changing the
  agent-files, custom.md), five stay `_None recorded._`: The dual-repo model, Pushing, Before
  any push, Close-out, File map, all of them pure rule or pure map
- the https-remote story is the longest single move, and its rule shrank to two sentences:
  the unconditional, and the first-thing-to-check boundary
- the exit-status sub-bullets kept their commands and lost their explanations, which read
  as one paragraph in the entry
- AGENTS.md 585 to 561: the outside-the-tree text was argument-heavier than the tree, as
  the Todo said, but the tree is most of the file

#### docs: point AGENTS.md's restated mechanics at the satellites

Problem: with the why gone, AGENTS.md is still twice the target because it restates
mechanics the satellites own: jj commands, the named-not-numbered rule, push behaviors, the
close-out shapes. Solution: each restatement becomes a one-line pointer to the satellite
section that holds it, the satellite gaining any sentence AGENTS.md alone had.

- AGENTS.md 561 to 456. The protocol tree is now rules plus step lists, each step pointing
  at the satellite that owns its mechanics, and nothing in it is stated a second time
- new satellite homes, each named for what it holds:
  - jj.md's "vc-x1 push: what it does and does not do", "Close-out shapes", and the
    per-commit contract in "Local ladders" and the reshape moves in "Cycle bookmarks"
  - notes.md's "The In Progress block" (the six items, the Ladder details area, the rung
    form) and "The close-out move" (the four transforms, the Done entry, the In Progress
    reset)
  - prose.md's "Commit description details"
- close-out steps 3 and 4 merged (the move and the Done entry are one act in notes.md), so
  backfill is step 7 and the Opening's pointer follows
- the Terminology "Retired" notes went to rationale.md as history, not rule
- no separate details file: every block had a satellite owner, so rationale.md stays why-only
- what is left is rule text: the hard rules, Pushing's three policy paragraphs, the at-rest
  contract, Working practices, the agent-file rules. Reaching 297 from here means shortening
  rules, which is a different cycle's decision

#### docs: label-colon form for AGENTS.md's lists and rules

Problem: a bold lead that is the sentence's own verb reads as a skippable label, and the
hard rules' bold sentences are the rule itself, skipped the same way. Solution: a prose.md
rule, a bold lead in a list is a short label ending in a colon and the sentence after it is
complete without it, applied across AGENTS.md, the hard rules taking short names as labels.

- prose.md's "Leads are labels, unmarked" holds the rule, its example, the inverted case (a rule
  stated in bold with commentary after it), definitions (`**Term:**`, not `**Term.**`), and
  the one-word redundancy as the accepted price. The measured miss stays inline there, since
  prose.md's why has not moved yet
- the label carries no markup (wink, 2026-08-21, at review): bold is what makes the eye
  skip, and an agent needs no emphasis, so the colon alone marks it
- every bold lead in AGENTS.md converted: the 14 hard rules (now named: Read custom.md first,
  Push commits, Approval per push, Hard stop after the final push, No re-describe without
  coordinating, No hand-written trailers, jj not git, Read the step before the action,
  Typeable punctuation, One title per step, Stop and ask, Alert on unwrap, Intent picks the
  file, One bookmark per cycle), the Terminology definitions, the four step lists, the Policy
  paragraphs, Working practices, Changing the agent-files, and the custom.md section
- the labels cost 15 lines (456 to 471), and a refill to the full 100 columns, links and code
  spans unbreakable, gave 24 back (447)
- a tightening rung added after this one (wink, 2026-08-21), since what is left is rule text, and
  the next cut is wording

#### docs: tighten AGENTS.md's prose

Problem: with the why and the mechanics gone, AGENTS.md is rule text at 447 lines, and the
rules are worded at the length they were argued, not the length they need. Solution: reword
each rule shorter without dropping a boundary, the diff reviewed sentence by sentence.

- AGENTS.md 447 to 400, every heading kept so the rationale mirror holds, every anchor and ref
  checked
- the cuts: restated subjects after a label ("Backfill: fill every ..." not "Backfill: backfill
  every ..."), parentheticals that named the obvious ("(medium, conventions)"), doubled
  qualifiers ("explicit", "specific", "mandatory" where the sentence already was), the
  Terminology and Cycle notes folded from bullet lists into paragraphs, and the File map's
  records list into one sentence
- boundaries kept, by check: every "never", "only", "not", and "exception" clause of the old
  text has a counterpart in the new
- `[rde]` added as a reference-style link, since Retiring Done entries is cited twice

#### docs: Winks AGENTS.md's tightening

Problem: the agent's tightening is one reader's cut, and the reader the rules are for is the
other one. Solution: wink's own pass over AGENTS.md, landed as its own rung so the two cuts
are separately reviewable.

- AGENTS.md 389 to 340. wink's cut, reviewed item by item with `--N--` markers in the file:
  fourteen points raised, six dropped as detail the linked section carries, eight settled
- rules changed by the pass:
  - repos are always hyphenated ("work-repo")
  - the per-rung Bump precedes Work
  - push is expected at every rung
  - both repos' commits carry the same title and body (prose.md's "agent-repo body" line
    retired)
  - "Project root" and "Short paths" retired from Working practices
  - `### Topic bookmarks are drafts` folded into Cycles run on a bookmark, its five links
    repointed and its rationale entry merged
- wording principle recorded (wink, 2026-08-21): shorter and direct, since redundancy and
  restated detail hinder the agent rather than protect it, and the linked section carries
  the detail. A mistake can be resolved, so a rule is stated once
- the validate rule binds the agent, not the user ("not the boss of me"): the full run is not
  advised mid-review because `cargo fmt` rewrites files, but `--fast` is safe at any time
- "work product" replaces "artifact" for what the work-repo holds
- filed Todo "`vc-x1 validate --full`: accept the default by name"

#### docs: halve AGENTS.md into rationale.md closing

Problem: the cycle's premise, "about half of AGENTS.md is argument", measured at about a
tenth, and the ladder grew from five rungs to ten finding where the rest of the length was.
Solution: recorded as found. The why was 8 lines net of 593, the restated mechanics 105, the
wording 108, and the rest is rule. The three cuts are three different moves with three
different reviews, and a future "halve X" cycle should open with that split measured, not
assumed. Also filed: the validate element that fails on an unfilled `[[N]]` rung whose commit
is on `main`, and the satellites' own why-move, both as backlog entries.

Exception taken after landing (wink, 2026-08-22): the closing had left the package name at
`vc-x1-dev`, the dev name the opening sets, so the name flip back to `vc-x1` was squashed into
the landed closing and `main` force-pushed, at wink's direction, for a clean history. The
0.80.1 closing had missed the same flip. The rule that the closing's bump restores the stable
name now lives in custom.md's single-name convention.

## docs: fix dev artifacts

### Problem

The agent-files leave six cycle-protocol gaps that this cycle met in one sitting, each a rule
the tightening either never placed or left one link short.

* The single-name convention says a cycle builds as `vc-x1-dev` and `main` as `vc-x1`, but no
  agent-file places the rename at a cycle beat, and two closings landed still named
  `vc-x1-dev`.
* The close-out shape list lost its link to the trapezoid recipe in the tightening, and the
  shapes read in a different order in AGENTS.md and jj.md.
* The per-rung flow's describe step reaches the commit-body form only at one remove, which let
  a description get drafted in the retired two-paragraph form.
* The Cycle term names single-step and multi-step without saying how to choose, which is how
  this cycle began as edits on `main`.
* The commit-body form lets a top-level `-` stand with no `*` above it, which reads as a
  solution to nothing, and it says nothing about a bookend commit, whose body has no problem
  to state.
* Close-out shapes names the three shapes without saying how to look at the net change before
  choosing one.

### Solution

The rename is named as an Opening step of its own and the restore as a step of the trapezoid
recipe, the recipe is relinked from the close-out shape list, the commit-body form is cued at
the describe step, the Cycle term says when a cycle is single-step or multi-step, the
commit-body form pairs every solution under a problem and makes a bookend body a pointer to
the record, and Close-out shapes says how to preview a squash before choosing.

### Acceptance check

- `grep -n '^name = ' Cargo.toml` reads `vc-x1-dev` on every rung of the bookmark and `vc-x1`
  on the landed close-out
- AGENTS.md's Opening lists Sweep, Bump, Rename as steps 4-6, and the trapezoid recipe's step 2
  restores the name
- every link added resolves: Close-out step 5 to the recipe, Opening step 6 and recipe step 2
  to versioning.md's dev artifact name
- `rg ';' AGENTS.md custom.md agent-data/jj.md` finds semicolons only inside code spans

### Ladder

- [[N]] [docs: fix dev artifacts opening][63]
- [[N]] [docs: rename at the opening and restore in the trapezoid recipe][64]
- [[N]] [docs: cue the commit-body form at the describe step][65]
- [[N]] [docs: say when a cycle is single-step or multi-step][66]
- [[N]] [docs: label the commit-body form][67]
- [[N]] [docs: preview a squash before choosing the close-out shape][68]
- [[N]] [docs: fix dev artifacts closing][69]

### Deliberation

The work began as uncommitted edits on `main`, reviewed as a docs interlude, and the Opening
ran only after wink named the miss: the recipe's "docs interlude" sentence reads as a waiver
of hard rule 13, and it is not one. The edits were parked and the Opening run in full, the
parked edits becoming the rungs. The bookmark is `fix-dev-artifacts`, wink's name, rather than
the title's slug, a scoped exception granted at the opening. A patch bump: agent-file rules
only, no change to the tool.

### Ladder details

#### docs: fix dev artifacts opening

The ten rungs of the halve cycle were unfilled and are backfilled, the 0.80.0 Done entry is
swept to done.md, and the manifest takes the opening's version under the dev name, the first
opening to do so by rule.

#### docs: rename at the opening and restore in the trapezoid recipe

The Opening's sweep-and-bump step splits into Sweep, Bump, and Rename, the recipe gains a
step that restores the plain name and squashes it into the close-out commit before the
reshape, and custom.md's single-name paragraph points at that step. The close-out shape list
links the recipe again, with the three shapes in the same order in AGENTS.md and jj.md, and
two semicolons in the first draft of these edits are reworded.

#### docs: cue the commit-body form at the describe step

Per-rung step 7 names the body's markers inline, so the shape is in front of the writer at the
moment of writing rather than two links away.

#### docs: say when a cycle is single-step or multi-step

The Cycle term gains the choice: single-step when the problem has one straightforward solution
step with its documentation in the same commit, otherwise multi-step, since development runs
on the bookmark under the dev name either way. "Cycles run on a bookmark" states that
development is not done on `main`, and the recipe's docs-interlude sentence is reworded to
match.

#### docs: label the commit-body form

Every `-` sits under a `*`, the trivial commit being one of each, so a body reads the same
with or without the rule open. The intro states the problem this commit resolves, never the
cycle's, and a bookend commit's body is an intro paragraph naming the cycle and pointing at
its record, since an opening or closing resolves nothing of its own.

#### docs: preview a squash before choosing the close-out shape

Close-out shapes says how to see what a squash would carry: diff `<base>` to the tip, at full
context to read the result rather than the edit, the same view `git log --first-parent` gives
a trapezoid once landed. The decision is then made on what `main` will carry.

#### docs: fix dev artifacts closing

Problem: the acceptance check's first line, `vc-x1-dev` on every rung of the bookmark, was
the cycle's own rule exercised for the first time, and the check's last line, the plain name
on the landed close-out, could only be met by the recipe's new step 2 running as written.
Solution: all four checks passed at the closing, the six bookmark commits all carrying the dev
name, and the restore ran as the recipe's step 2 between the close-out push and the reshape.
What closing taught: the close-out bump sets the bare version under the dev name still, which
the `build.rs` guard allows, so the version and the name change in different commits by
design. The 41 MB TODO.md (an unbounded `str.index` slice replaced as an empty string) cost a
zed restart and taught one script rule: bound every slice and assert it non-empty.
# References

[1]: #refactor-retire-the-remaining-jj-spawns-opening
[2]: #refactor-port-push-and-facade-reads-to-jj-lib
[3]: #refactor-port-sync-repositioning-to-jj-lib
[4]: #refactor-port-op-recovery-and-squash-to-jj-lib
[5]: #refactor-port-init-and-clone-plumbing-to-jj-lib
[6]: #chore-ban-process-spawning-outside-the-version-gate
[7]: #refactor-retire-the-remaining-jj-spawns-closing
[8]: #docs-pin-two-rules-and-close-the-convergence-record
[17]: #docs-empty-custom-family-into-the-pinned-set-and-config-opening
[18]: #feat-agent-naming-in-config-and-cli
[19]: #feat-add-the-family-and-validate-tables-to-the-schema
[20]: #feat-add-the-validate-subcommand
[21]: #docs-pin-messaging-into-agent-data
[22]: #docs-retire-custom-familymd
[23]: #docs-empty-custom-family-into-the-pinned-set-and-config-closing
[24]: #feat-rename-validate-bot-to-validate-agent
[9]: https://github.com/winksaville/vc-x1/commit/966214308f42 "966214308f42ed19aadc4c6c10a52e774379e71c"
[10]: https://github.com/winksaville/vc-x1/commit/4ec329664e1a "4ec329664e1aa348bdf65e955c4b5d0feba71c11"
[11]: https://github.com/winksaville/vc-x1/commit/3fc19a038602 "3fc19a0386027b016350b58962910acb5e88589c"
[12]: https://github.com/winksaville/vc-x1/commit/a3c0012c8f23 "a3c0012c8f23f28f4cb53639de655bf589127fc3"
[13]: https://github.com/winksaville/vc-x1/commit/47590a565aab "47590a565aab877028cd24aa28b301a75f1eb7ee"
[14]: https://github.com/winksaville/vc-x1/commit/fdbffa928c4e "fdbffa928c4eee0144c648f38e4bb1f891b7a33e"
[15]: https://github.com/winksaville/vc-x1/commit/e28cbd6b4983 "e28cbd6b498385104240ee423996f739618824b5"
[16]: https://github.com/winksaville/vc-x1/commit/92c398a91f8b "92c398a91f8b70e8962161f848926a8c7c6573f7"
[25]: #docs-fold-the-cycle-agent-files-into-agentsmd-opening
[26]: #docs-write-the-cycle-account-into-agentsmd
[27]: #docs-retire-cycle-protocolmd-and-cycle-checklistsmd
[28]: #docs-rename-bot-repo-to-agent-repo-in-the-agent-files
[29]: #docs-fold-the-cycle-agent-files-into-agentsmd-closing
[30]: https://github.com/winksaville/vc-x1/commit/63fe9e7cc85f "63fe9e7cc85f84cab3a54e32664883d98d8b327d"
[31]: https://github.com/winksaville/vc-x1/commit/704d246a6342 "704d246a6342335a09bef7244e5da94ff317b9fb"
[32]: https://github.com/winksaville/vc-x1/commit/5ce67fce6e7a "5ce67fce6e7a4998c4954ce69ddb9585d1defa8f"
[33]: https://github.com/winksaville/vc-x1/commit/e909117ac336 "e909117ac3366dc5aba4156dd4b3732ca6289db0"
[34]: https://github.com/winksaville/vc-x1/commit/bf177d42c81f "bf177d42c81f128887bbfb6c5e052fa7eda9786a"
[35]: https://github.com/winksaville/vc-x1/commit/72b2077f682d "72b2077f682d6d9fcac5eaf66d173813363b8f4e"
[36]: https://github.com/winksaville/vc-x1/commit/ee1fbfd28eca "ee1fbfd28ecad344b1fb4c6b2ece12d23c2caa99"
[37]: https://github.com/winksaville/vc-x1/commit/ef9aed26b238 "ef9aed26b238a87cba37ca93249e46a3321ad3af"
[38]: https://github.com/winksaville/vc-x1/commit/389c070c38b9 "389c070c38b9d7e3e8c9e173fd46960755f8fb75"
[39]: https://github.com/winksaville/vc-x1/commit/9be6c66aebd6 "9be6c66aebd6c91a31984246099125ef8e9ad6f8"
[40]: https://github.com/winksaville/vc-x1/commit/c441a2e37e84 "c441a2e37e84e5889c46de7ee1a34254788e5cc3"
[41]: https://github.com/winksaville/vc-x1/commit/7aaf783d57e6 "7aaf783d57e6ab9c43f9baaf97c8f61036ed3a97"
[42]: https://github.com/winksaville/vc-x1/commit/14540f84300e "14540f84300ecbd68ab28fdf24a18116a85bcdba"
[43]: #docs-halve-agentsmd-into-rationalemd-opening
[44]: #docs-seed-rationalemd-and-the-rationale-term
[45]: #docs-move-the-cycle-protocols-why-into-rationalemd
[46]: #docs-move-the-rest-of-agentsmds-why-into-rationalemd
[47]: #docs-halve-agentsmd-into-rationalemd-closing
[48]: #docs-make-backfill-the-openings-first-step
[49]: #docs-label-colon-form-for-agentsmds-lists-and-rules
[50]: #docs-point-agentsmds-restated-mechanics-at-the-satellites
[51]: #docs-tighten-agentsmds-prose
[52]: #docs-winks-agentsmds-tightening
[53]: https://github.com/winksaville/vc-x1/commit/3ee0b5c494a1 "3ee0b5c494a1111ec1a2cf0a721e4969cf480f6a"
[54]: https://github.com/winksaville/vc-x1/commit/232f3740cb36 "232f3740cb3632e3b1b44506382aaa912e7ba7ab"
[55]: https://github.com/winksaville/vc-x1/commit/a29f17baa4ab "a29f17baa4ab33e45fe3e530d78c1379b9f77af2"
[56]: https://github.com/winksaville/vc-x1/commit/157ef9c6f685 "157ef9c6f6855de9142d79002c049daea391214c"
[57]: https://github.com/winksaville/vc-x1/commit/1b5891c6480b "1b5891c6480b08d5889dec83802941b0ecee9013"
[58]: https://github.com/winksaville/vc-x1/commit/9da942d9352d "9da942d9352d751b5bceb1f311a746d951663d3c"
[59]: https://github.com/winksaville/vc-x1/commit/4fbfadf870d1 "4fbfadf870d126e22fe07f608c54e18c7ad757bf"
[60]: https://github.com/winksaville/vc-x1/commit/fc0cdd4917f5 "fc0cdd4917f5a41b859cc8b8c7719c7d32322a1d"
[61]: https://github.com/winksaville/vc-x1/commit/1b0827688e1a "1b0827688e1ac12a078aeb8f088474c140eb9e83"
[62]: https://github.com/winksaville/vc-x1/commit/4a530f43ca87 "4a530f43ca8724e74961e27cb391ed28003b4b2c"
[63]: #docs-fix-dev-artifacts-opening
[64]: #docs-rename-at-the-opening-and-restore-in-the-trapezoid-recipe
[65]: #docs-cue-the-commit-body-form-at-the-describe-step
[66]: #docs-say-when-a-cycle-is-single-step-or-multi-step
[67]: #docs-label-the-commit-body-form
[68]: #docs-preview-a-squash-before-choosing-the-close-out-shape
[69]: #docs-fix-dev-artifacts-closing
