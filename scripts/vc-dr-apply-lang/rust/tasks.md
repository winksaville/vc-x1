| bump | sets the version in `Cargo.toml` and refreshes `Cargo.lock` ([Suffix scheme](agent-data/versioning.md#suffix-scheme)) | first, at each commit of a cycle | `cargo xtask bump next`, or `bump X.Y.Z` |
| validate | formats the code, runs the lints, and runs every test | before each review, and before every commit | `cargo xtask validate` |
| validate fast | runs the binaries' unit tests only, changing no file | while a review iterates | `cargo xtask validate --fast` |
| run | builds the dev binary, then runs it with the arguments | to try the change | `cargo xtask run <args>` |
| install | builds the dev binary and installs it as `{name}-dev`, and with `--release` the release one as `{name}` | after each rung's commit, and with `--release` at Land | `cargo xtask install` |
