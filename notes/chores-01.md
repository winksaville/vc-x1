# Chores-01

title syntax: `<tcid>-<ocid>-<short description>`

 - `<tcid>` is the jj change ID of this commit and is never `none`.
 - `<ocid>` is the jj change ID of the other commit and may be `none` if the bot was not involved.
 - `<short description>` is a brief one line description of the chore.

## zznknssm-none-Validate changeID consistency

Validate that when we clone a vc repo that the changeID remains the same
between the original and the clone for both the main and the claude repos.

Move the original local repo to vc-x1.ok and
clone to vc-x1 and then Initialize jj in vc-x1
finally use `jj st` to get the changeId `qvnyrzox`:
```
wink@3900x 26-03-04T18:56:36.701Z:~/data/prgs/rust
$ mv vc-x1 vc-x1.ok
wink@3900x 26-03-04T18:56:46.387Z:~/data/prgs/rust
$ git clone git@github.com:winksaville/vc-x1
Cloning into 'vc-x1'...
remote: Enumerating objects: 7, done.
remote: Counting objects: 100% (7/7), done.
remote: Compressing objects: 100% (6/6), done.
remote: Total 7 (delta 0), reused 7 (delta 0), pack-reused 0 (from 0)
Receiving objects: 100% (7/7), 8.21 KiB | 8.21 MiB/s, done.
wink@3900x 26-03-04T18:57:14.025Z:~/data/prgs/rust
$ cd vc-x1
wink@3900x 26-03-04T18:57:20.343Z:~/data/prgs/rust/vc-x1 (main)
$ jj git init
Done importing changes from the underlying Git repo.
Setting the revset alias `trunk()` to `main@origin`
Hint: The following remote bookmarks aren't associated with the existing local bookmarks:
  main@origin
Hint: Run the following command to keep local bookmarks updated on future pulls:
  jj bookmark track main --remote=origin
Initialized repo in "."
Hint: Running `git clean -xdf` will remove `.jj/`!
wink@3900x 26-03-04T18:57:26.594Z:~/data/prgs/rust/vc-x1 (main)
$ jj st
The working copy has no changes.
Working copy  (@) : zznknssm f24eadc5 (empty) (no description set)
Parent commit (@-): qvnyrzox 5bb219d1 main main@origin | Initial Commit
wink@3900x 26-03-04T18:58:34.324Z:~/data/prgs/rust/vc-x1 (main)
```

Clone the vc-x1.claude repo into .claude and get the changeId `ponzrznv`:
```
$ git clone git@github.com:winksaville/vc-x1.claude.git .claude
Cloning into '.claude'...
remote: Enumerating objects: 3, done.
remote: Counting objects: 100% (3/3), done.
remote: Total 3 (delta 0), reused 3 (delta 0), pack-reused 0 (from 0)
Receiving objects: 100% (3/3), done.
wink@3900x 26-03-04T18:58:34.324Z:~/data/prgs/rust/vc-x1 (main)
$ cd .claude
wink@3900x 26-03-04T18:58:42.307Z:~/data/prgs/rust/vc-x1/.claude (main)
$ jj git init
Done importing changes from the underlying Git repo.
Setting the revset alias `trunk()` to `main@origin`
Hint: The following remote bookmarks aren't associated with the existing local bookmarks:
  main@origin
Hint: Run the following command to keep local bookmarks updated on future pulls:
  jj bookmark track main --remote=origin
Initialized repo in "."
Hint: Running `git clean -xdf` will remove `.jj/`!
wink@3900x 26-03-04T18:58:46.948Z:~/data/prgs/rust/vc-x1/.claude (main)
$ jj st
The working copy has no changes.
Working copy  (@) : mqrnpozk 46adfe52 (empty) (no description set)
Parent commit (@-): ponzrznv 90244032 main main@origin | Initial Commit
wink@3900x 26-03-04T18:58:50.893Z:~/data/prgs/rust/vc-x1/.claude (main)
```

Cd into the original repo get the two changeIDs, `qvnyrzox` for vc-x1 and
`ponzrznv` for .claude:
```
wink@3900x 26-03-04T18:58:50.893Z:~/data/prgs/rust/vc-x1/.claude (main)
$ cd ../../vc-x1.ok/
wink@3900x 26-03-04T19:00:31.341Z:~/data/prgs/rust/vc-x1.ok ((main))
$ jj st
The working copy has no changes.
Working copy  (@) : qnxrzwrw bfe0bbd9 (empty) (no description set)
Parent commit (@-): qvnyrzox 5bb219d1 main | Initial Commit
wink@3900x 26-03-04T19:00:35.263Z:~/data/prgs/rust/vc-x1.ok ((main))
$ cd .claude
wink@3900x 26-03-04T19:00:49.904Z:~/data/prgs/rust/vc-x1.ok/.claude ((main))
$ jj st
The working copy has no changes.
Working copy  (@) : kttvktmm 0c376c25 (empty) (no description set)
Parent commit (@-): ponzrznv 90244032 main | Initial Commit
wink@3900x 26-03-04T19:00:56.656Z:~/data/prgs/rust/vc-x1.ok/.claude ((main))
```

## Use footers to track changeIDs or notes

This is more general as we can have multiple footers for multiple related
bits of information, such changeId, or URLs to files in this repo or other places.
In particular URLs to the naratives in chores-xx.md files and changeID (chid).

Since jj information is not stored in a repo but generated when `jj git init` is run
the changeID markdown URLs are only valid when using a yet to be implemented tool
vc-x1 (vibe coding) tool.

### Example of changeID footers

We'll explore two forms:

#### footer style:

- `[1]: .knxzszwu`
- `[2]: .claude/ponzrznv`

#### URL style:
- `[ChangeID in this repo, /.jj/ must exist](./knxzszwu)`
- `[ChangeID in a local repo, /.claude/.jj/ must exist](.claude/ponzrznv)`

## Create a binary that lists jj info

This binary should list the changeID, commitID, and description title
and using `jj-lib`

### Implementation

Created a Rust binary (`src/main.rs`) using `jj-lib` 0.39.0 that:

1. Opens the jj workspace from the current directory (or a path argument)
2. Loads the repo at head via `Workspace::load()` and `RepoLoader::load_at_head()`
3. Evaluates `RevsetExpression::all()` to iterate all commits
4. For each commit (excluding the root), prints: changeID (reverse hex, 12 chars),
   commitID (hex, 12 chars), and first line of description

Key dependencies: `jj-lib = "0.39.0"`, `pollster = "0.4"` (for async `.block_on()`).

Usage:
```
cargo run            # uses current directory
cargo run -- /path   # uses specified path
```
