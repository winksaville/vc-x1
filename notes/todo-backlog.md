# Todo Backlog

 Lower-priority `## Todo` entries — the long tail. When an
 entry becomes a priority, move it (and any refs it cites)
 into `TODO.md > ## Todo` at its priority rank (the
 list is strict-ranked, #1 highest), then `fix-todo` to
 renumber.

 Same formatting rules as `TODO.md > ## Todo` — see
 `notes/README.md > ## Todo format`. Run
 `vc-x1 fix-todo --no-dry-run notes/todo-backlog.md` to
 renumber.

## Todo

1. **vc-x1 push: per-repo bookmark names.** Allow code-side
   and `.claude`-side bookmarks to differ; currently
   `.claude` is locked to `main` regardless of the
   app-side `<bookmark>`. Sibling generalization to the
   N:1 / `--squash` / `--merge` work — together they make
   push handle all close-out shapes with arbitrary
   bookmark layouts.
2. **Investigate `linkme` for subcommand registration.**
   Distributed-slice registry — each subcommand registers itself
   at link time; `main.rs` discovers them via the slice rather
   than matching a `Commands` enum. Reduces per-subcommand
   touchpoints from 3 (mod decl + enum variant + match arm) to 1
   (registration). Costs: loses compile-time exhaustiveness
   (missing registration = runtime gap); help-output ordering
   depends on link order unless sorted; macro-magic dependency.
   Revisit once the `0.50.0` trait sweep's per-arm cost has been
   felt under real "add a subcommand" load.
   <https://github.com/dtolnay/linkme>
3. **Investigate `inventory` as `linkme` alternative.** Same
   shape as `linkme` — runtime-iterable registry populated by
   `inventory::submit!` per subcommand. Trade-offs mirror
   linkme's. Pick one if/when the trait sweep's match becomes
   the felt bottleneck.
   <https://github.com/dtolnay/inventory>
4. **forks-multi-user + bot-data-formats follow-through.**
   Design captured across two notes; concrete work to
   land when a cycle picks it up. Major pieces:
   multi-line `ochid:` parser/emitter; bot-side
   immutability enforcement; URL-shaped ochid (per-user
   / cross-repo); vendor-subdir layout
   (`.bot/<vendor>/<version>/<id>.<ext>`) +
   flat-to-vendor migration. (The `.claude/` → `.bot/`
   rename piece moved into the refactor program's
   facade-owns-topology stage, config knob included.)
   Each piece is its own future TODO when the design
   hardens. [[2]],[[3]]
5. **`test_helpers::Fixture` migration + downstream callers.**
   Plus rename `Fixture` → `TestFixtureDual` and `FixturePor`
   → `TestFixturePor` so call sites carry the test-only
   signal that `#[cfg(test)] mod test_helpers` doesn't
   communicate. Was `0.41.1-7`. [[5]]
6. **`vc-x1 finalize --scope` flag.** Replace `--repo`
   with the role vocabulary used elsewhere
   (`work|bot|work,bot`). Carry-over from the 0.42.0
   `--scope` sweep (was 0.42.0-5; deferred at -4.7
   close-out). The paired `Single(_)` dogfood item
   (0.42.0-7) is moot after `0.53.0` — `Single(_)`
   deleted. Design lives in chores-07. [[6]]
7. **Cross-file `chores-NN.md` ordering sanity pass.**
   `chores-08.md` (the 0.41.1 cycle) landed on `main` via
   the `0.42.0-4.7` rebase; check that section ordering
   across `chores-06`/`-07`/`-08`/`-09` is chronologically
   coherent and normalize if not. Low priority.
8. **Add a vc-x1 validate-repo?**
9. **vc-x1 push: rework the two bookmark parameters.**
   `PushArgs` has `bookmark_pos` (positional `BOOKMARK`) +
   `bookmark` (`--bookmark` flag) for one logical value,
   forcing an `or_else` in `From<&PushArgs>`. Collapse to a
   single positional with `--bookmark` as a true clap alias,
   or drop one spelling. [[7]]
10. **vc-x1 push: `--recheck` — implement or remove.** Parsed by
    `PushArgs`, never read; mirrored into `PushParams` with
    `#[allow(dead_code)]`. Either wire the
    skip-preflight-on-resume behavior or drop the flag. [[8]]
11. **vc-x1 push: `--scope=work|bot|work,bot` flag.**
    Was 0.42.0-4 (deferred when cycle pivoted to icr
    rebase work; cycle closed at -4.7). State machine
    becomes scope-aware — single-side scope skips
    `commit-claude`/bookmark-claude/`finalize-claude`.
    [[9]],[[10]],[[11]],[[12]]
12. **vc-x1 clone: `--scope=work|bot|work,bot` flag.**
    Parallel to `init --scope` for role selection;
    topology (`--por` vs dual) is the separate `--por`
    boolean. Was 0.42.0-6 (deferred at -4.7
    close-out). [[10]],[[11]],[[12]]
13. **vc-x1 validate-desc / fix-desc:
    `--scope=work|bot|work,bot` flag.** Same role vocabulary
    as elsewhere — `work` validates the work repo's commits against
    bot, `bot` reverses, `work,bot` does both (new
    default). [[10]],[[11]],[[12]]
14. **Unify `.vc-config.toml` accessors onto Pattern B**
    (typed struct + `load_from(path)`, like new
    `config::UserConfig` and `push::resolve_state_layout`).
    Replaces the map-typed helpers in `desc_helpers.rs` /
    `fix_desc.rs` / `validate_desc.rs` with a typed
    `WorkspaceConfig` struct. ~50 LOC, mechanical.
    Candidate for 0.41.2. [[13]]
15. **Layered config precedence (user → workspace → CLI)**
    once `WorkspaceConfig` is typed. Workspace can
    override `[github].owner` etc. for a specific project;
    init can't use the layer (chicken-and-egg) but
    post-init commands can. Depends on the
    `WorkspaceConfig` typed-struct refactor above.
    Candidate for 0.41.2. [[13]]
16. **Help layout: force over-under everywhere.** Apply
    `next_line_help(true)` at the root (or via the existing
    `cli_with_banner` walker) so every subcommand's `-h` /
    `--help` uses the same layout. Today clap auto-picks
    per-command based on the widest flag spec, so
    `sync -h` is two-column but `init -h` is over-under —
    visual inconsistency.
17. **Replace "Step N" log prefixes with single-word
    `label: body` convention** (`bookmark`, `provision`,
    `colocate`, `cross-ref`, `symlink`, …); indent labels
    under per-side `code:` / `bot:` headers in dual.
    Originally planned as 0.41.1-6.7; deferred.
18. **"Stop saying workspace in user-facing surfaces" sweep.**
    The `[workspace]` → `[repos]` rename itself shipped in
    the 0.76.0 repo-registry cycle (legacy schemas hard-
    reject with a fix-it; `src/legacy_vc_config.rs` holds
    the compat surface until the repo migration sweep);
    what remains of the original entry is the broader
    wording sweep — prose, help text, and identifiers
    still say "workspace" for the dual-repo project root.
19. **Add `status` (alias `st`) subcommand: `jj st` across both
    repos in one shot.** Uses `--scope` from day one. This is
    natural home for the working-copy signal called out and
    it needs to include remotes, like remotes/origin/main. [[14]].
20. **`vc-x1 init --dry-run` should bypass the
    `--repo-remote` path-existence preflight** (currently fires
    before the dry-run early-return; observed dogfooding
    2026-04-24).
21. **vc-x1 push: `--squash` flag.** Squashes WC into `@-` via
    `--ignore-immutable` and force-pushes; needs
    `--force-with-lease`-equivalent + state-sanity preflight in
    place first. [[9]]
22. **vc-x1 push: `--message-file PATH` flag.** Git-style commit
    message file (first line = title, blank, rest = body).
    Alternative to `--title` + `--body`. [[15]]
23. **Mirror `--check` / `--no-check` onto `vc-x1 push`** (forwards
    through to the preflight `vc-x1 sync` invocation).
    0.37.1 hard-codes `--check`; default stays `--check`.
24. **Add `validate-repo` subcommand:** diagnostic that runs all
    `verify_*` checks (tracking, push state freshness, ochid
    integrity, conflicts, config sanity, working-copy state)
    plus chores↔commit consistency — every `[N]:` anchor
    resolves, and each chores `##` section's recorded title
    matches its `Commits:` commit's title — and reports
    per-check pass/fail. Exit code = number of failed checks.
    Implementation: promote `verify_state_sanity` /
    `verify_completion_sanity` from push.rs to `common.rs`. [[16]]
    - **A punctuation check cannot be a byte scan.** The
      typeable-punctuation rule forbids *authoring* the four
      characters, not their presence: transcribed tool output
      and published commit titles keep theirs, and no scanner
      distinguishes authored from transcribed. Record a
      per-file count baseline and fail when a count rises.
      This supersedes the chores-15 note asking the checker to
      read its character set from one place, which assumed a
      checkable zero.
25. **sync: surface working-copy state in the up-to-date summary**
    (per-repo pending-files count or compact stat). Wording-only
    fix shipped in 0.37.1; this is the design+impl. [[14]]
26. **bm-track silent-when-clean refinement.** Print on entry/exit
    only when state isn't fully tracked or when exit state
    differs from entry. [[17]]
27. **"Oh shit" revert — post-success undo via `.vc-x1-ops/`
    anchor dir.** Idea-stage; every repo-mutating command drops a
    pre-op snapshot, `vc-x1 undo` restores both repos. [[9]]
28. **Source-code design ref sweep + AGENTS.md codification:**
    adopt section-name + `blob/main/...` URL pattern for source
    code refs to designs; codify in AGENTS.md alongside the
    existing markdown ref conventions. Sweep targets:
    src/push.rs lines 4, 121, 645, 1219. [[18]]
29. **Richer bookmark enumeration:** per-bookmark remote presence + tracking status [[19]]
30. **Per-line/per-thread runtime log points** (future, maybe) [[20]]
31. **Add Windows symlink support** via `std::os::windows::fs::symlink_dir` [[21]]
32. **Add "::" revision syntax for jj compatibility**
33. **Add -p, --parents, -c, --children** so parent and child counts can be asymmetric
34. **Add integration tests in tests/ for subcommands** using temp jj repos (tempfile crate)
35. **Fix .claude repo history:** dev0 through dev2 sessions squashed into wrong commit
    [[22]],[[23]]
36. **Add `vc-x1 setup` subcommand:** completions install, .claude repo init, symlink setup
    [[24]]
37. **Add dynamic revision completion via `ArgValueCompleter`** (jj doesn't complete revsets
    either) [[25]],[[26]]
38. **Test-tempdir override resolution chain.** Both
    `src/test_helpers::unique_base` and
    `tests/common/unique_base` currently use
    `std::env::temp_dir()` (= `$TMPDIR`). Generalize to
    resolve in priority order: explicit env var (e.g.
    `VC_X1_TEST_TMPDIR`) → user config
    (`~/.config/vc-x1/config.toml`) → local
    `.vc-config.toml` → `std::env::temp_dir()` fallback.
    Useful when a developer wants tests on a tmpfs / SSD /
    project-local path without exporting `TMPDIR` globally.
    Open question: do we also expose a CLI parameter
    (e.g. `vc-x1 --workspace-tmp …`)? Test binaries can't
    easily accept arbitrary flags via `cargo test --`, so
    env is the realistic surface for tests; for the
    `vc-x1` binary itself a flag is feasible but unclear
    it adds value over the resolution chain.
39. **`validate-todo` / `fix-todo`: flag malformed lines.**
    A column-0 line inside `## Todo` / `## Bugs` that is
    neither an entry (`N. `) nor a heading is malformed;
    it's currently tolerated silently. Report it so stray
    lines / typos surface.
40. **`vc-x1` version-string ref resolution.** Today version
    strings (`0.58.0`, `0.58.0-3`) live in commit titles and
    `Cargo.toml` but aren't git refs, so
    `git diff 0.58.0^1 0.58.0` fails with "ambiguous
    argument". Auto-tagging at close-out clutters the tag
    namespace fast (one tag per cycle); a resolver is cleaner:
    - `vc-x1 sha <ref>` primitive: accepts version strings,
      standard git refs (pass-through), jj refs (chids, `@`)
      and outputs a SHA on stdout. Composable into
      `git diff $(vc-x1 sha X)^1 $(vc-x1 sha X)`.
    - `vc-x1 diff` / `vc-x1 log` thin wrappers that accept
      the version-string vocabulary and dispatch to git.
    - Cycle-relative aliases: `prev`, `cur`, `prev~1c` for
      "N cycles back along `--first-parent`".
    - `--first-parent` awareness: version refs resolve to
      the merge commit (bare `X.Y.Z`), so `0.58.0^1` walks
      first-parent to `0.57.0` naturally.

    Builds on existing `vc-x1 chid` (which resolves jj
    revsets to chids). Separate gap on the jj side: no clean
    first-parent revset operator in jj 0.40 — equivalent
    today is `jj diff --from <fp-chid> --to <merge-chid>`.
41. **`vc-x1 push --squash`: symmetric squash on both repos.**
    Automate Option F (manually exercised in the 0.59.0
    close-out [[27]]): app-side squash + bot-side description
    rewrite + force-push, atomically. Demoted from `## Todo`
    after 0.67.0 — merge non-ff is the routine close-out
    shape and a pre-publication squash needs no tooling, so
    this entry's domain (squashing already-published cycle
    commits) is off the routine path.
    - App side: squash cycle commits into one new commit;
      capture the squashed chid.
    - Bot side: rewrite the prior push commits' descriptions
      — replace their per-commit `ochid:` trailers with one
      pointing at the squashed chid; add a rewrite-note
      acknowledging the change (preserves historical truth
      for future readers).
    - Force-push bot `main` (rewrites the published commit;
      chid preserved via `jj describe`).
    - Push app `main`. The new bot commit paired with this
      push receives `ochid: /<squashed-chid>` as normal — K
      prior bot records plus the new one gives (K+1):1
      bot→code (2:1 in the 0.59.0 case).
42. **`vc-x1 clone`: graceful single-repo fallback when no
    companion `.claude` remote.** Default dual clone errors
    mid-way when `<source>.claude` doesn't exist; `--por` is
    the workaround (works, but you must know to pass it).
    - After the code clone, probe the session URL with
      `git ls-remote <url> HEAD`; on failure report "no
      companion session repo — cloning code repo only"
      (GitHub reports missing and no-access identically) and
      skip the bot clone + symlink; done message says single
      repo.
    - A clone failure after a successful probe stays a real
      error.
    - Dry-run text notes steps 2/3 are skipped when no
      companion exists.
    - Integration-testable offline via path-form sources
      (code bare repo present, no `.claude` bare repo).
43. **Sketch cross-repo ochid migration in
    cycle-protocol.md.** Remnant of the retired Ideas entry
    "Codify ochid invariant + bot-repo rules + squash gating
    + cross-repo migration" (rest folded into the 0.72.0
    merge close-out cycle's docs step): in a
    multi-contributor flow, ochids change at every merge
    until the change reaches the canonical repo's `main` —
    document the migration story.
44. **Refactor stage: por → dual conversion.** Attach a bot
    companion + `.vc-config.toml` to an existing por
    workspace as a routine subcommand; see
    [the stage](refactor-20260716.md#stage-por--dual-conversion).
    Last program stage on purpose — leans on facade-owns-
    topology and the in-process init pieces.
45. **Write down what an interlude is.** A docs or planning
    commit that lands between cycles on the trunk line, taking
    a patch bump rather than a `-N` rung. The facts exist but
    are scattered across four places: `TODO.md`'s program
    ladder, cycle-protocol's trapezoid details, the chores
    `Commits:` convention, and the numbering visible in the
    shipped versions. Consolidation, not new information, so
    the rank is low.
    - The one load-bearing fact is already written down: a
      trapezoid's `<base>` is the parent of the ladder's first
      rung, not the last close-out, because an interlude sits
      on the trunk line and must stay there. It bit at 0.76.0
      and again at 0.77.0, and now lives in
      [the trapezoid details](../agent-data/cycle-protocol.md#details).
    - What is missing is the decision rule: when unplanned
      work becomes an interlude rather than a rung appended to
      the running cycle.

46. **`init` template ergonomics.** Take-or-leave ideas from the template restructure (mailbox,
    2026-07-31), triaged here 2026-08-02:
    - `--use-template <repo-root>` auto-detecting `work/` / `work.claude/`, so the caller
      passes the template repo rather than its payload subdirectory
    - a no-remote init mode for local testing
    - an empty-dir bot template only needs a `.gitkeep`, since init writes hidden files itself

47. **Warn on the legacy `[workspace]` config schema from any command.** A repo on the old
    schema reads, logs, and diffs fine for weeks and discovers the problem only when `push`
    hard-errors, the least convenient moment. Cheap fix: any command warns once on a legacy
    config rather than only `config --validate` and `push`. iiac-perf finding (mailbox,
    2026-08-07).

48. **Revset pass-through: stop translating the house dialect.** The CLI still converts
    house-convention revsets before issuing jj commands. It should pass revsets to jj
    verbatim, so one dialect exists and `jj help -k revsets` is the single authority
    (decided 2026-08-03). The docs side landed with the agent-files adoption; this is the
    CLI side.

49. **Promote body shape into hard rule 9?** Extending title identity to cover the
    problem/solution body shape was deferred at the 2026-08-07 convergence review: the
    convention was two days old, and "a pushed body is coordinate-first to fix" would
    promote every prose rule that touches a body. Concrete trigger: revisit after
    iiac-perf's measure-reproducibility cycle closes under the convention.
    - the shape itself stopped being vague at the "docs: pin the commit-body form" rung
      (2026-08-12), which pinned the intro / `*` facets / `-` solutions form in prose.md. What
      is left here is only the promotion question, on a rule that now has text to promote

50. **OSC 8 hyperlinks in `config` TTY output** (wink, 2026-08-09). When stdout is a TTY,
    `vc-x1 config` renders each key's name as an OSC 8 hyperlink to its reference url, so the
    printed schema is clickable in supporting terminals; suppressed when piped, with an
    `ls`-style `--hyperlink=auto|always|never` override.
    - the stored files keep plain urls: TOML forbids control characters in comments, and
      editors plus most modern terminals linkify plain urls already
    - builds on the vc-config.toml prototype's per-key reference field (the "docs: freshen
      vc-config and config subcmd" cycle), so it waits for that cycle to land

51. **init distributes vc-config.md; reference-base points at the member** (wink + bot,
    2026-08-10). `vc-x1 init` seeds a new member with a copy of `vc-config.md` from the
    template payload and stamps `[vc-config] reference-base` with the member's own repo url,
    so every generated doc-reference web link in the member's `.vc-config.toml` lands on a
    copy the member owns. The copy is a pinned family file: a member's edits diff against the
    payload and are its doc proposals, folded back at convergence.
    - the instance `.vc-config.toml` stays the only behavior-changing file: pinning and
      value-carrying are mutually exclusive, since workspace values in a pinned copy would
      pollute the diff-as-proposal property
    - the prototype `vc-config.toml` stays vc-x1-only build source (schema proposals travel by
      family channel or fork), and workspace-local keys wait on the `[private]` table
    - `validate-anchors` gains member-side work: the carried copy must still have the headings
      the instance's `reference:` links anchor at
    - settled at the "chore: regenerate stale config files" rung of the "docs: freshen
      vc-config and config subcmd" cycle; the fuller ownership model is recorded there
    - the `localfile://./vc-config.md#<anchor>` entry in each key's `doc references:` list
      lands here, not in the doc-references rung it was drafted in (cut at review, wink
      2026-08-10): the entry is useless until the file it names exists locally, which is
      this entry's seeding. Editors and terminals need a taught handler for the scheme
      (claude-web: straightforward in neovim / vscode, harder in zed); the rendered list
      form already leaves room, so adding the entry is append-only
    - a possible example of the localfile link (the doc one-liner will wrap at <=100 rather
      than this example's width):
      ```
      # bot-session.col-width: Default --col-width: first-column width in the
      #   --fields / --unknown / --per-line views
      #   used by: bot-session --col-width
      #   default: 68
      #   doc references:
      #    - https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#bot-sessioncol-width
      #    - localfile://./vc-config.md#bot-sessioncol-width
      # col-width = 68
      ```

52. **Reference defs: go file-relative, with anchors** (wink + bot, 2026-08-10). The house
    `[N]:` form `/notes/<file>.md` resolves by *convention*, not by any markdown standard:
    no spec says what a leading `/` means, so each viewer picks. Wink's measurements
    (2026-08-10, todo-backlog's `[2]` / `[3]`): GitHub resolves it against the repo root
    (works), VS Code's preview against the workspace root (works, though preview "back"
    navigation is weak, a VS Code limitation no path form fixes), and Zed treats it as a
    filesystem-absolute path (dead). The bot's initial claim that GitHub 404s was wrong,
    eliminated by the same measurements.
    - only file-relative paths (`<file>.md` from a sibling, `../notes/<file>.md` from
      elsewhere) resolve identically everywhere, since relative-to-the-containing-file is
      the one interpretation needing no convention
    - so the direction is relative paths plus `#anchor` fragments where a def targets a
      section; costs: a notes.md convention change, a sweep of existing defs, and defs
      becoming location-dependent (the close-out move's transform list already rewrites
      relative links, and validate-anchors' cross-file check keeps the sweep honest)
    - sweep inventory (2026-08-10): ~167 root-absolute defs across 9 files (done.md 122,
      TODO.md 7, refactor-20260716.md 5, design-cli/por-dual-parity-audit.md 3, chores-07 2,
      chores-06 / chores-09 / design-cli/copying.md 1 each, todo-backlog.md 25), plus ~5
      inline `](/...)` links
    - wink trial-converted todo-backlog.md's 25 defs to the `./` form (2026-08-10): all
      target files verified present, links confirmed working on GitHub and in VS Code (we
      think Zed resolves the relative form too, untested). Reverted the same day so the
      records stay uniform with the pinned convention until this entry's sweep flips both
      together

53. **vc-config.md per-key worked examples** (2026-08-11). The per-key sections mostly restate
    each key's one-liner, and a reference link has to reward the click. Cut from the
    "docs: freshen vc-config and config subcmd" ladder at its freeze: no acceptance item
    needs it, since item 3 asks only that each generated link land on a real section, which
    it does.
    - a worked TOML example per section, as it would appear in an actual config file (for
      `bot-session.items`, a couple of named recipes)
    - effects, not just meaning: what visibly changes when the key is set
    - decide there: extend the anchor drift test to require a toml fence per key section,
      making "has an example" suite-enforced (quality itself stays a review judgment)
    - the entry shrank when it moved: its `repo.category.<cat>` and `account.*` bullets
      described keys that `## Todo` #1 deletes
    - a conventions change, so its own small cycle per AGENTS.md's Changing the agent-files,
      sequenced after validate-anchors lands and grows the cross-file check, so the ~145-edit
      sweep is machine-verified rather than hand-checked

54. **Config provenance names the schema, not just the binary** (iiac-perf + bot, 2026-08-12).
    The schema is generated at build time from `vc-config.md`, so an installed binary validates
    against its build's prototype rather than the workspace's, and a key added after that build
    is reported unknown with the config blamed. Member repos run a binary built from this one,
    so the exposure is the family's.
    - provenance already prints, keyed to the binary: `--validate` opens with the banner
      (`vc-x1-dev 0.78.8-8`) and `print_schema` with
      `# vc-x1 settable config keys (from vc-x1 0.78.8-8)`. So this is one field on two lines
      that exist, not a new flag
    - the gap is that a version identifies the *build* while the question at an unknown-key
      complaint is whether that build's `vc-config.md` equals the workspace's. A content hash
      of the prototype, baked by build.rs beside the schema and printed next to the version,
      answers it exactly
    - **not covered by `## Todo`'s Tiered exit status for `config --validate`** (#5), which was
      asked and is worth recording: an unknown key is tier 1 whether it is a typo or a stale
      binary, so the exit status is the same either way. That entry carries severity (could the
      check run at all), this one attribution (whose fault the unknown key is), and only the
      second tells a reader to fix a spelling or to rebuild. It does shrink what is left here:
      once tier 1 reads as "non-fatal findings"
      rather than a bare failure, an unknown key looks like a note instead of a fault, and
      being unsure why it appeared costs less
    - decide there: hash or a schema version. A hash is free, exact, and unreadable; a version
      is readable and someone has to remember to bump it

55. **Cite a Todo or backlog entry by its bold title, not its number** (wink, 2026-08-12).
    Numbers are addresses that change on every insert. Ranking two entries in one session
    renumbered 15 in `TODO.md` and slid a backlog entry from 56 to 55, invalidating references
    written minutes earlier, one of them already drafted into another member's mailbox. Titles
    do not move.
    - the doctrine already exists one level up: hard rule 9 makes a cycle step's title its only
      identifier, carrying no number and unambiguous within its cycle. This is the same rule
      applied to the notes files, which is the argument for it being pinned rather than local
    - a number may still ride along as a convenience ("**title** (#5)"), but the title is what
      the reference *is*, so a stale number beside a live title is a hint gone slightly out of
      date rather than a pointer at the wrong entry
    - long titles make a citation unwieldy, which is an argument for keeping them short and a
      reason to allow citing an unambiguous prefix. Commit titles have a 50-char cap and these
      have none, which is where the friction comes from. Decide there whether to cap them
    - the rule has a precondition the files do not meet: every entry needs a bold title and
      titles must be locally unique. Several backlog entries open in bare prose with no bold
      title at all, so the sweep is real work rather than a wording change
    - **the cost is clickability.** A bold title inside a numbered list item is not a heading,
      so it has no anchor and a citation by title cannot be a link. Numbers were never linkable
      either, so nothing regresses, but the fix (entries become `###` headings, which anchors
      and lets `validate-anchors` reach them) is a much larger change and should be decided
      here rather than assumed by whoever does the sweep
    - mechanical check once validate-anchors lands: every entry carries a bold title, and
      titles are unique within their file
    - the rule belongs in pinned `agent-data/notes.md` beside Notes references and Reference
      numbering, so it is a family proposal; a conventions change gets its own small cycle per
      AGENTS.md's Changing the agent-files. Sequenced after validate-anchors, like
      **vc-config.md per-key worked examples**, so the uniqueness sweep is machine-verified
    - the bold-title precondition is done (2026-08-12): all 56 entries here carry one, none
      duplicated. What remains is the rule, the citation sweep, and the check

56. **What carries a Todo entry: numbered list, heading, or a tracker outside the repo?**
    (wink, 2026-08-12). The numbered-list form is what makes numbers unstable and titles
    unlinkable, so **Cite a Todo or backlog entry by its bold title, not its number** (#56)
    treats a symptom of it. Three routes, and they are not the same size of decision.
    - **`###` headings per entry.** Every entry gains an anchor, so a citation becomes a link,
      `validate-anchors` reaches it, and the bold-title rule enforces itself structurally
      because a heading *is* the title. The cost is the numbering machinery: `fix-todo` and
      validate-numbering key on `^\d+\. `, and `TODO.md`'s own header tells intro paragraphs to
      lead with a space so they miss that pattern. Strict rank becomes document order, which
      loses nothing. The move between files stays a cut-paste
    - **GitHub issues.** A real tracker: state, search, assignment, comments, and cross-repo
      references between members. It would also subsume the mailbox, which exists only because
      a member cannot write into another member's repo
    - **A database.** Everything the issues route costs, plus building and maintaining the tool,
      minus a free tracker. It wins only under a query load that 78 entries do not present
    - **the crux is doctrine, not format.** AGENTS.md keeps durable context in committed files
      because "easy for everyone to find beats convenient for the bot alone". Issues and a db
      both leave the repo: not in the diff, not in jj history, not in a clone, not offline, not
      carried by the template payload, and the one-home-per-record move (a `## In Progress`
      block becoming a chores section at close-out) turns into a sync problem across two
      systems. So headings are a change inside the doctrine and the other two are a change *to*
      it, which is a decision to take deliberately rather than as a file-format tweak
    - a narrower version of the issues route may survive that objection: issues for
      *cross-member coordination*, where the mailbox is already a workaround for repo
      boundaries, and files for in-repo records. Worth separating before the whole idea is
      accepted or rejected
    - **decide this before the citation sweep**, since headings would change what the sweep
      does: with anchors, citations become links and the bold-title rule is restated rather
      than applied as written
    - **a second motivation, from the mailbox rather than the notes files** (wink,
      2026-08-12): handling a message deletes it, which is heavy-handed for anything a commit
      might later want to cite, and the copy-into-records-first rule exists precisely because
      deletion loses the original. A durable, referenceable issue would serve both surfaces at
      once, which strengthens the tracker option relative to headings, since headings fix
      citation inside a repo and do nothing for cross-member traffic

57. **Update the template payload, and empty the three-way diff** (2026-08-12, replanned
    2026-08-16, the baseline landed by the "chore: update vc-x1-template" rung). The payload
    tip is now the family's agreed state, and what remains is keeping it that way:
    - the payload takes the 0816-proposal result when that program completes (TODO.md "Empty
      the custom* files into the pinned set and config"), both members re-sync, and the
      three-way comparison going empty is the acceptance check
    - `jj-tips.md` needs its own fix: it still glosses `@..` as "descendants of @, excluding
      @", the wrong reading that triggered the revset decision, and it sits at the template
      root rather than under `work/`, so no payload sync will carry it. Its tutorial
      transcripts (around lines 259-290) are the real rewrite
    - the template's own `.vc-config.toml` is a fossil (wink, 2026-08-17): the pre-0.75.0
      `[workspace]` schema plus the retired `[push]` comment block, working today only
      because read-only surfaces (an explicit-path `bot-session`) never consult the
      resolvers' legacy rejection. We think the agent-naming rung's `[bot-session]` rejection
      widens the breakage, so regenerate it (as `.vc-config.md`, per the per-repo carrier
      rule)
    - template governance (wink, 2026-08-16): the template stays agent-less, wink as
      operator, writes and pushes only at convergence moments at wink's direction, recorded
      in the acting member's records


# References

[2]: /notes/forks-multi-user.md
[3]: /notes/bot-data-formats.md
[5]: /notes/chores/chores-08.md#cycle-structure--multi-step
[6]: /notes/chores/chores-07.md#--scope-enum-refactor-0420
[7]: /notes/chores/chores-09.md#push-dual-bookmark-parameters
[8]: /notes/chores/chores-09.md#push-unimplemented-recheck-flag
[9]: /notes/chores/chores-05.md#capture-squash-mode--scope-design-for-push-0374
[10]: /notes/chores/chores-06.md#generalize---scope-to-all-commands-design
[11]: /notes/chores/chores-06.md#--scope-continuation-0410
[12]: /notes/chores/chores-08.md#init--clone-redesign-0411
[13]: /notes/chores/chores-08.md#user-config-0411-3
[14]: /notes/chores/chores-05.md#open-sync-up-to-date-should-mention-working-copy-state
[15]: /notes/chores/chores-05.md#capture---message-file-design-for-push-0375
[16]: /notes/chores/chores-06.md#vc-x1-validate-repo-command-design
[17]: /notes/chores/chores-06.md#bm-track-silent-when-clean-design
[18]: /notes/chores/chores-06.md#source-code-design-ref-convention-design
[19]: /notes/chores/chores-05.md#open-questions--tbd
[20]: /notes/chores/chores-03.md#per-lineper-thread-runtime-log-points-future
[21]: /notes/chores/chores-03.md#windows-symlink-support
[22]: /notes/chores/chores-01.md#refactor-and-add-desc-subcommand
[23]: /notes/chores/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[24]: /notes/chores/chores-02.md#0260--shell-completion-via-clap_complete-env
[25]: /notes/chores/chores-02.md#testing-results
[26]: /notes/chores/chores-02.md#shell-completion-discovery
[27]: /notes/chores/chores-12.md#docs-extract-cycle-protocol-0590
