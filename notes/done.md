# Done

As todo.md `## Done` sections fills move them to here.

- Add --version and -V flags using std lib (clap not a dependency)
- Use git trailers for inter/intra repo info: ochid trailer, changeID path syntax, .vc-config.toml[[2]]
- Document git trailer convention (ochid:) and .vc-config.toml for workspace identity
- Document why jj log shows fewer commits than gitk (refs/jj/keep, obslog, ::@ revset)
- Create a binary that lists jj info[[1]]
- Convert CLI to subcommand structure with `list` command
- Add finalize subcommand arg parsing (0.6.0-dev1) [3]
- Add finalize daemonize with debug logging (0.6.0-dev2) [3]
- Implement finalize exec with squash/push logic (0.6.0-dev3) [3]
- Add --ignore-immutable and unique log paths (0.6.0-dev4) [3]
- Finalize subcommand complete (0.6.0) [3]
- Plan refactor and desc subcommand (0.7.0-dev0) [4],[5]
- Extract common.rs and refactor list (0.7.0-dev1) [4],[5]
- Refactor finalize into src/finalize.rs (0.7.0-dev2) [4],[5]
- Implement desc subcommand (0.7.0-dev3) [4]
- Refactor and desc subcommand complete (0.7.0) [4]
- Migrate CLI parsing to clap derive (0.8.0) [6]
- Move subcommand args into per-module structs (0.9.0) [7]
- Add --revision/-r, --repo/-R, --limit/-l to list (0.10.0-dev1) [8]
- Add --revision/-r, --repo/-R, --limit/-l to desc (0.10.0-dev2) [8]
- Revision and repo options complete (0.10.0) [8]
- Show changeID and commitID in desc output (0.11.0) [9]
- Add chid subcommand (0.12.0) [10]
- Add --limit to chid subcommand (0.13.0) [11]
- Add positional `..` revision notation (0.14.0) [12]
- Add required `--bookmark` to finalize (0.14.0) [13]
- Bold primary revision in chid, list, desc output (0.15.0) [14]
- Indent desc body lines with --indent/-i, default 3 spaces (0.16.0) [15]
- Finalize: replace --foreground with --detach, document manual recovery (0.17.0) [16]
- jj commit organization and traversal mechanisms (0.17.0) [17]
- Add show subcommand with header, bookmarks, and diff summary (0.18.0) [18]
- Flesh out show header to match gitk, add .. notation and file limiting (0.18.1) [18]
- Unify `..` notation and CLI across all subcommands (0.19.0) [18]
- Reorganize notes: move older done items to done.md (0.19.1)
- Multi-repo `-R` support with `-l`/`--label` and `-L`/`--no-label` for chid, desc, list, show (0.20.0) [18]
- Disperse CLI parsing tests from main.rs into per-subcommand files (0.20.1) [19]
- Show ochid in list output, clean up CLI help defaults (0.21.0) [19]
- Deduplicate common CLI flags with `#[command(flatten)]` (0.21.1) [19]
- Add fix-ochid subcommand with validation and --fallback (0.22.0) [19]
- Fix fix-ochid prefix bug: read workspace.path from .vc-config.toml (0.22.1) [19]
- Fix fix-ochid short ID extension, add notes to pre-commit checklist (0.22.2) [19]
- Add --add-missing to fix-ochid for inferring ochid from title+timestamp (0.23.0) [19]
- Add --max-fixes to fix-ochid to limit commits actually changed (0.24.0) [19]
- Add validate-desc subcommand, extract desc_helpers (0.25.0-dev1) [21]
- Add fix-desc subcommand using shared helpers (0.25.0-dev2) [22]
- Add lost/none special ochid status, improved error messages (0.25.0-dev2) [22],[23]
- Read other-repo from .vc-config.toml, make positional arg a --other-repo flag (0.25.0-dev3) [24]
- Run fix-desc on both repos to fix ochid trailers with --fallback for lost IDs (0.25.0) [20]
- Remove deprecated fix-ochid subcommand (0.25.0) [25]
- Add shell completion via clap_complete env (0.26.0) [26]
- Fix validate-desc/fix-desc other-repo resolution with -R flag (0.26.2) [30]
- Add `fn claude-symlink` and `symlink` subcommand (0.27.0) [31]
- Add `init` subcommand for dual-repo project creation (0.28.0) [32]
- Add `clone` command + fix init submodule/ochid bug (0.29.0) [33]
- Universal --verbose, common::run() refactor, chid bold removal (0.30.0) [34]
- Adopt `log` crate with per-module runtime filtering (0.31.0) [35]
- Remove submodule from init/clone (0.31.1) [38]
- Audit `unwrap`/`unwrap_or` usage, add `// OK: …` convention (0.32.0) [39]
- Make `finalize` failures visible — pre-flight, subprocess logging, tty reconnect, status marker (0.33.0) [40]
- Fix deprecated `jj bookmark track <bookmark>@<remote>` syntax for jj 0.40.0 (0.33.1) [41]
- Silence untracked-remote hint in `init` step 9 (0.33.2) [42]
- Compatible dep refresh via `cargo update` (0.33.3) [43]
- Add `--use-template` to `init` and `test-fixture` (0.34.0) [44]
- Bump `jj-lib` to 0.40 + tighten `clap` floor to 4.6 (0.34.1) [45]
- Add `sync` subcommand — fetch + classify + rebase both repos (0.35.0) [46]
- Show bookmarks in `list`, `show`, `desc` output (0.36.0) [47]

# References

[1]: /notes/chores-01.md#create-a-binary-that-lists-jj-info
[2]: /notes/chores-01.md#git-trailer-convention
[3]: /notes/chores-01.md#finalize-subcommand-for-session-repo-coherence
[4]: /notes/chores-01.md#refactor-and-add-desc-subcommand
[5]: /notes/chores-01.md#claude-repo-issue-070-dev0-through-dev2
[6]: /notes/chores-01.md#migrate-cli-parsing-to-clap-080
[7]: /notes/chores-01.md#move-subcommand-args-into-modules-090
[8]: /notes/chores-01.md#add-revision-and-repo-options-to-list-and-desc-0100
[9]: /notes/chores-01.md#show-changeid-and-commitid-in-desc-output-0110
[10]: /notes/chores-01.md#add-chid-subcommand-0120
[11]: /notes/chores-01.md#add---limit-to-chid-subcommand-0130
[12]: /notes/chores-01.md#add-positional--revision-notation-0140
[13]: /notes/chores-01.md#add-required---bookmark-to-finalize-0140
[14]: /notes/chores-01.md#bold-primary-revision-in-output-0150
[15]: /notes/chores-01.md#indent-desc-body-lines-0160
[16]: /notes/chores-01.md#finalize-detach-and-manual-recovery-0170
[17]: /notes/chores-02.md#jj-commit-organization-and-traversal-mechanisms-0170
[18]: /notes/chores-02.md#0180--initial-show-subcommand
[19]: /notes/chores-02.md#0200--multi-repo-support
[20]: /notes/chores-02.md#0250--refactor-into-validate-desc--fix-desc
[21]: /notes/chores-02.md#0250-dev1--add-validate-desc-extract-desc_helpers
[22]: /notes/chores-02.md#0250-dev2--add-fix-desc-subcommand
[23]: /notes/chores-02.md#special-ochid-values-lost-and-none
[24]: /notes/chores-02.md#0250-dev3--read-other-repo-from-config
[25]: /notes/chores-02.md#0250--remove-deprecated-fix-ochid
[26]: /notes/chores-02.md#0260--shell-completion-via-clap_complete-env
[30]: /notes/chores-02.md#0262--fix-validate-descfix-desc-other-repo-resolution-with--r
[31]: /notes/chores-03.md#add-fn-claude-symlink-0270
[32]: /notes/chores-03.md#add-init-command-0280
[33]: /notes/chores-03.md#add-clone-command-0290
[34]: /notes/chores-03.md#universal---verbose-and-commonrun-refactor-0300
[35]: /notes/chores-03.md#adopt-log-crate-with-per-module-filtering-0310
[38]: /notes/chores-03.md#remove-submodule-from-initclone-0311
[39]: /notes/chores-04.md#audit-unwrapunwrap_or-usage-0320
[40]: /notes/chores-04.md#make-finalize-failures-visible-0330
[41]: /notes/chores-04.md#fix-deprecated-jj-bookmark-track-syntax-0331
[42]: /notes/chores-04.md#silence-untracked-remote-hint-in-init-step-9-0332
[43]: /notes/chores-04.md#compatible-dep-refresh-0333
[44]: /notes/chores-04.md#add---use-template-to-init--test-fixture-0340
[45]: /notes/chores-04.md#bump-jj-lib-to-040--tighten-clap-floor-0341
[46]: /notes/chores-04.md#add-sync-subcommand-0350
[47]: /notes/chores-04.md#show-bookmarks-in-list-show-desc-output-0360
