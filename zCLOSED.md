# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record.

## Closed

### fix: no test sets HOME

(wink + agent, 2026-10-15) `init::tests::expand_vars_tilde` sets `HOME` to `/h` for the whole
test process and puts it back, and a test that signs a commit while it is set has git look for
the signing key under `/h`, so `vc-x1 validate` fails now and then for no reason in the change
under it. [Bug 21](notes/bugs.md#bugs), seen a second time 2026-10-15, its victim
`init::tests::dual_fixture_writes_work_and_bot_config_files`.

#### Deliberation

- Fixed now, ahead of the Todo's first entry, wink's call: a validation that fails at random
  stops every cycle's push.
- The fix is the bug's own fix direction: `expand_vars` reads `HOME` and hands it to an inner
  function that takes the home directory as an argument, and the test calls the inner one.
  - Running the tests single-threaded, as the test's comment suggests, was not taken: it slows
    every run to hide one test's write.
- The cause was reproduced before the fix was trusted, wink's ask at review: the failure itself
  was not, a race seen once in about a dozen runs.
  - A three-second sleep inside the old test's window, `main`'s test file under the fixed
    code, failed 18 sync integration tests: the `jj` they spawn inherited `HOME=/h`, found no
    config, and died with "Failed to determine the secure config for a repo". A different
    symptom from the signing key, the same cause.
  - The fixed test with the same sleep: all 653 passed.
  - The sleep was a demonstration and is not committed: after the fix there is no window for it
    to widen.
- Two guards keep the cause out, wink's call at review, both:
  - The test `no_test_writes_home` scans `src/` and `tests/` for a write of `HOME`, so it fails
    on `main`'s test file, naming its three writes, and passes on the fix. It matches text, so
    a variable holding `"HOME"` passes it.
  - Clippy bans `std::env::set_var` and `std::env::remove_var`, as it bans `Command::new`, with
    a register in `clippy.toml` and an `#[allow]` at each granted site. That covers the variable
    the test misses. The two remaining writes, each of a variable no other test reads, are the
    register's two entries.
  - Entry 1, `expand_vars_envvar`, still writes while other tests' threads read the
    environment, which libc does not make safe. Passing it a variable lookup, as `HOME` is
    passed, would retire it, and it stays, wink's call: the register leaves it visible.
- Single-step, one function split, its guards, and its record.

#### Solution

`expand_vars` splits so the expansion takes the home directory as an argument, and the test of
`~` passes `/h` instead of writing `HOME`, so no test writes the process's `HOME`. A test fails
on any write of `HOME`, and clippy bans environment writes but for the two sites its register
names.

#### Acceptance check

- `rg 'set_var\("HOME"' src/` prints nothing. Passed.
- `no_test_writes_home` fails on `main`'s `src/init/tests.rs` and passes on the fix. Passed.
- `cargo clippy --all-targets` rejects an environment write without an `#[allow]`: the ban
  flagged the four writes in `src/init/tests.rs` and `tests/cli_keep.rs` before their
  `#[allow]`s. Passed.
- `vc-x1 validate` passes. Passed.

#### Ladder

- fix: no test sets HOME (done)

Close-out shape: none to choose, single-step. Land fast-forwards `main` over the one commit.

# References
