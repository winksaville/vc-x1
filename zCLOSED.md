# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### chore: jj-lib moves to 0.46.0

(wink, 2026-10-09, picked 2026-10-13) jj-lib 0.46.0 is published and the project builds against
0.45. The installed `jj` is 0.46.0, so the version gate, which compares the `jj` CLI with the
linked jj-lib, refuses every run of the installed `vc-x1`. The Todo entry picked is "jj-lib moves
to 0.46.0":

- The `gix` line in `Cargo.toml` follows, 0.87 to 0.88, since it names the gix jj-lib resolves.
- The facade in `src/jj/` takes whatever the API changed between the two releases.
- The check `Cargo.toml` asks for at a jj-lib bump is already made for this one: 0.46.0 has no
  remote-ref query, so the `## Waiting` entry stays.

#### Deliberation

- A single-step cycle, the agent's recommendation and wink's go: a build against 0.46 with gix
  0.88 fails at 19 sites in six groups, five of them a mechanical follow-up of an API that moved.
- The version gate is waived for this cycle, wink's go, 2026-10-13: the `vc-x1` runs of the cycle
  pass `--allow-jj-mismatch` until Land installs a binary linked against 0.46. The waiver covers
  sync, validate, and push, and it is no approval of a push.
  - The alternative, putting `jj` back to 0.45.1 until the cycle lands, was offered and not taken.
- The package is `vc-x1-dev` until Land, wink's correction. The agent had kept the plain name on
  the precedent of the single-step cycles before this one, which were documents alone: this one
  changes code, and validation's `cargo install` would have replaced the installed `vc-x1` with
  an unreviewed build.
- Lock contention is told by classification and I/O kind, wink's choice of the narrow form. gix
  0.88 has no `gix::lock::acquire::Error` to downcast to: a held lockfile is an untyped error
  classified retryable over an `io::Error` of kind `AlreadyExists`.
  - The broad form, any error gix classifies retryable, was offered and not taken, since gix
    marks transport and pack errors retryable too.
  - The classification is on the gix error itself, and jj-lib holds that error in
    `#[error(transparent)]` variants whose `source()` skips it. So the classifier names the two
    jj-lib errors the retried writes return, `GitResetHeadError` and `GitExportError`, and asks
    the payload of their `Git` variant. A retried write returning a third error type would need
    naming there, and the test of a held `.git/index.lock` is what would say so.
- The `## Waiting` conditions are unmet, so nothing promotes: `vc-x1 closed` is not a command,
  and jj-lib 0.46.0 has no remote-ref query.

#### Solution

The project builds against jj-lib 0.46 and gix 0.88, and the call sites take the APIs that
changed.

- A `UserSettings` is built with the home directory, which jj-lib now takes from its caller to
  expand a leading `~/` in a config path. One function, `home_dir`, reads `HOME` for all of them.
- A ref target is a plain merge of commit ids, so its ids are read with `adds().flatten()`.
- A revset is evaluated through jj-lib's `ResolvedRevset`, which carries what an `at_operation()`
  brings into context. Blame's domain still takes the bare expression.
- A gix error that `?` no longer converts goes through `gix::Exn::into_error`.
- Lock contention is told by gix's classification and the I/O kind, asked of the gix error inside
  jj-lib's `Git` variants.
- The semicolons in the comments of `src/show.rs` are paid, the file being touched.

#### Acceptance check

- `vc-x1-dev sync --scope=both --dry-run` runs with no `--allow-jj-mismatch` beside `jj` 0.46.0,
  so the version gate finds the linked jj-lib and the CLI agree. Passed.
- `vc-x1 validate` passes. Passed, run by the installed `vc-x1` under the waiver.

#### Ladder

- chore: jj-lib moves to 0.46.0 (done)

# References
