# Chores-17

Continuation of `chores-16.md` (closed after `0.78.10`, the refactor-program retirement, at
just over 1000 lines). This file covers the cycles from `0.79.0-0` onward, opening with the
jj-spawn retirement cycle.

Reference numbering is file-local, per
[`agent-data/notes.md`](../../agent-data/notes.md#reference-numbering), and chores-17 starts
at `[1]`.

## Table of Contents

- [refactor: retire the remaining jj spawns](#refactor-retire-the-remaining-jj-spawns)

## refactor: retire the remaining jj spawns

- [[N]] [refactor: retire the remaining jj spawns opening][1]
- [[N]] [refactor: port push and facade reads to jj-lib][2]
- [[N]] [refactor: port sync repositioning to jj-lib][3]
- [[N]] [refactor: port op recovery and squash to jj-lib][4]
- [[N]] [refactor: port init and clone plumbing to jj-lib][5]
- [[N]] [chore: ban process spawning outside the version gate][6]
- [[N]] [refactor: retire the remaining jj spawns closing][7]

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

# References

[1]: #refactor-retire-the-remaining-jj-spawns-opening
[2]: #refactor-port-push-and-facade-reads-to-jj-lib
[3]: #refactor-port-sync-repositioning-to-jj-lib
[4]: #refactor-port-op-recovery-and-squash-to-jj-lib
[5]: #refactor-port-init-and-clone-plumbing-to-jj-lib
[6]: #chore-ban-process-spawning-outside-the-version-gate
[7]: #refactor-retire-the-remaining-jj-spawns-closing
