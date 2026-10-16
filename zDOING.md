# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial (2026-10-09). The agent-files still name `TODO.md > ## In Progress` as the
record's home, and the cycle that makes this file the rule is [The cycle-record moves out of
TODO.md into zDOING.md](TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

_None._

## In Progress

### fix: a snapshot failing is reported to the user and in Result

(wink + agent, 2026-10-15) From iiac-perf's `m-10-0` in the messages repo: `vc-x1 push`
committed and pushed both repos without a session's transcript, and said nothing. The transcript
was a new file over `snapshot.max-new-file-size` at its first snapshot, and the `jj` command
reports that case as `Refused to snapshot some files`, where vc-x1 is silent.

- `RepoSession::snapshot` discards the list `snapshot_with` returns, the new files the limit left
  untracked with their sizes, so no caller of `snapshot` sees it. Only `commit_any_size`, used by
  an adoption, does.
- Push decides whether the agent-repo has changes with `jj::is_empty(bot, "@")`, which snapshots
  through `snapshot`, so a refused transcript leaves `@` empty and push skips the agent-repo's
  commit. squash-push snapshots the same way.
- The limit vc-x1 ships is jj's default, 1 MiB, and a session's transcript passes it within
  hours, so an adopter without a raised limit in their user config loses every transcript that
  outgrows it before the session's first push. Our agent-repo holds transcripts of 6 to 12.5 MB.
- A refused file stays untracked, so jj-lib reports it again at every snapshot until it is
  ignored or the limit raised, which is what lets an error raised there keep firing.
- Found at the work (2026-10-15): vc-x1 reads none of jj's per-repo config. Since jj 0.37 a
  repo's config is `<user config dir>/jj/repos/<config-id>/config.toml`, named by the repo's
  `.jj/repo/config-id`, and `jj config set --repo` writes there, but `stacked_config` still reads
  the old `.jj/repo/config.toml`. So the remedy jj's own hint and this cycle's error name, `jj
  config set --repo snapshot.max-new-file-size`, would not reach vc-x1, nor any other per-repo
  setting.

#### Deliberation

- vc-x1 reads jj's per-repo config the way the `jj` command does, through jj-lib's
  `secure_config::SecureConfig`, in place of the old `.jj/repo/config.toml`.
  - In this cycle (wink, 2026-10-15), since the error's remedy is a per-repo setting and would
    not reach vc-x1 without it. Its own rung, being a fix of its own.
  - The read is `maybe_load_config`, which creates no entry for a repo without a `config-id`,
    where `load_config` would. It may migrate an old-style `.jj/repo/config.toml`, as the `jj`
    command would.
- `snapshot` returns a typed error naming the refused paths and sizes, with jj's two remedies, a
  `.gitignore` entry or a larger `snapshot.max-new-file-size`, iiac-perf's proposal.
  - Every caller fails with it, the read-only ones included. The `jj` command only warns, and a
    warning is what was missed. The cost accepted: a work-repo holding an unignored large file
    stops every vc-x1 verb until it is dealt with, and an acquaint's `sync --dry-run` reports it
    as a finding.
  - `commit_any_size` keeps calling `snapshot_with` directly, and keeps letting a file of any size
    in.
  - The files that fit are already snapshotted when the error returns, since
    `snapshot_working_copy` commits the snapshot operation first, and the message says so.
- The agent-repo's remedy is the user's to choose, never written by vc-x1 (wink, 2026-10-15).
  When a refused file is in an agent-repo, the error adds a third remedy: `jj config set --repo
  snapshot.max-new-file-size 0` in the agent-repo, 0 meaning no limit to vc-x1 and the `jj`
  command alike, or a larger value.
  - vc-x1 reads the limit through jj-lib's layered config, defaults, then the user's, then the
    repo's, so a repo's value overrides whatever the user's config sets, wink's being 20 MiB, in
    that repo alone. That is why it is not written unasked: it overrides a limit the user chose.
  - Weighed and left: `vc-x1 init` and `vc-x1 clone` writing it, first silently and then on a
    prompt or a flag. Push has already stopped when the error is seen, so nothing is lost while
    the user decides.
  - Weighed and left: push lifting the limit for the agent-repo itself. squash-push does not know
    which repo is the agent-repo, and the `jj` command would keep refusing.
- The tests set a limit in-process, never through `jj config set --repo`, which writes an entry
  under the user's config dir for every test repo.
  - Found at the work: the user's `~/.config/jj/repos` held 17,228 entries, 16,965 of them for
    test fixture repos long gone, each holding only the `trunk()` alias jj writes at a repo's
    creation, or this cycle's first test runs' 1 KiB limit. Removed at wink's go, 2026-10-15.
  - The push test writes a file just over whatever limit the user's config resolves to, and
    skips when there is none.
- iiac-perf's proposed check under Before any push, that the review shows the agent-repo carrying
  the session's transcript, is declined (wink, 2026-10-15), `m-10-2`: the error makes the refusal
  seen, at a push and at every other snapshot.
- A ladder (wink, 2026-10-15). It opened as one, became single-step when the agent-repo's limit
  was left to the user, and is one again with the per-repo config's rung, nothing having pushed
  but the bookmark.
- The cycle is titled for what the user and a caller get, wink's title at the opening's
  description review, in place of "fix: a snapshot no longer drops files the size limit refused".
  The bookmark keeps the first title's slug, being published and deleted at Land.

#### Solution

vc-x1 reads a repo's own jj config where jj keeps it. A snapshot that refuses a file over the size
limit fails, naming the file, its size, and the remedies, so no vc-x1 verb goes on as if the file
were not there. A refused file in an agent-repo adds the remedy of lifting the limit in that
repo's own config, which the user chooses.

#### Acceptance check

- A value set by `jj config set --repo` is what `RepoSession` reads, in a test.
- A new file over the limit makes `RepoSession::snapshot` return the typed error, naming the file
  and its size, and a refused file in an agent-repo adds the `jj config set --repo` remedy, in
  tests.
- `vc-x1 push` with a new file over the limit in either repo fails before committing either repo,
  naming the file, in a test.
- `vc-x1 validate` passes.

#### Ladder

- [fix: a snapshot failing is reported to the user and in Result opening][1] (done)
- [fix: vc-x1 reads jj's per-repo config][2] (done)
- [fix: snapshot fails on a file the size limit refused][3] (done)
- [fix: a snapshot failing is reported to the user and in Result closing][4]

##### fix: a snapshot failing is reported to the user and in Result opening

The cycle's setup commit: create and publish the bookmark, write this block, bump the
version-of-record, and rename the package to its dev name.

##### fix: vc-x1 reads jj's per-repo config

vc-x1 reads a repo's config from `.jj/repo/config.toml`, where jj has not kept it since 0.37, so
every per-repo setting is lost to it. It reads it through jj-lib's `SecureConfig`, as jj does.

- The repo layer is its own function, given the user's config dir, so a test points it at a temp
  dir and the user's is never written. `stacked_config` passes `$XDG_CONFIG_HOME` or
  `~/.config`, the dir it already read the user's own config from.
- `maybe_load_config`, never `load_config`: a read mints no `config-id` and no entry for a repo
  without one. It does migrate an old-style `.jj/repo/config.toml` on first read, as jj does, and
  jj's warnings, the migration's among them, are logged as warnings.
- The rng `SecureConfig` takes, to mint an id when migrating, comes from `rand` and
  `rand_chacha` at jj-lib's versions, seeded from the OS as jj-cli seeds it. jj-cli's
  `JJ_RANDOMNESS_SEED` override is left out, being a test hook of its own.
- The workspace layer, `.jj/workspace-config-id`, stays out, named as a deviation beside the
  command-line and `EnvBase` layers. No vc-x1 setting is per-workspace.

##### fix: snapshot fails on a file the size limit refused

`snapshot` drops the list of files the limit refused, so every verb proceeds without them and
push commits without a word. It returns a typed error naming them instead, with the remedies.

- `FilesTooLarge` carries the repo, the refused files with their sizes, the limit, and whether
  the repo is an agent-repo, so a caller can match on it, and its message is jj's hint adapted:
  a `.gitignore` entry, or `jj -R <repo> config set --repo snapshot.max-new-file-size
  <largest>`, the command that now reaches vc-x1 by the rung before. An agent-repo's adds `... 0`.
- Push stops at its first stage, the change check, before committing either repo, since the
  check snapshots, and squash-push, sync, and every other verb stop the same way.
- The limit's read is one function, `max_new_file_size`, its 0 read as no limit, shared by the
  snapshot and the error.
- The session tests set a 1 KiB limit in-process, on a `CommandArg` layer as `jj --config` sets
  one, and read trees with `--ignore-working-copy`, so the `jj` command's own limit takes no part.
- Push's test is a CLI test, `tests/cli_push.rs`: the binary runs in a `CliFixture` whose
  isolated home holds the jj config, the 1 KiB limit included, so the test owns its limit and its
  per-repo entries die with the fixture. It asserts no rollback ran in either repo, so push
  stopped before its first commit, and it fails with the old `snapshot`, checked by hand.
  - Weighed first and left: an in-crate test writing a sparse file one byte over the user's own
    limit, which skipped at no limit, could not tell a stop from a rolled-back commit, and tied
    the file's size to the user's config. Then a test-only thread-local config layer, needless
    once the CLI fixture was seen to hold the config.
  - The fixture's children also get `XDG_CONFIG_HOME` set and `JJ_CONFIG` removed, either of
    which, set in the environment, would carry jj's config reads past the isolated home.

##### fix: a snapshot failing is reported to the user and in Result closing

Closing out the cycle.

# References

[1]: #fix-a-snapshot-failing-is-reported-to-the-user-and-in-result-opening
[2]: #fix-vc-x1-reads-jjs-per-repo-config
[3]: #fix-snapshot-fails-on-a-file-the-size-limit-refused
[4]: #fix-a-snapshot-failing-is-reported-to-the-user-and-in-result-closing
