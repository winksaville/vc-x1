# Chores-01

## Initial commit with dual jj-git repos

Note: the two .gitignore files are explicitly ignoring .jj and .git
to be obvious although they maybe unnecessary.

```
wink@3900x 26-03-04T16:08:16.365Z:~/data/prgs/rust
$ git clone git@github.com:winksaville/hw-jjq-bot vc-x1
Cloning into 'vc-x1'...
remote: Enumerating objects: 26, done.
remote: Counting objects: 100% (26/26), done.
remote: Compressing objects: 100% (15/15), done.
remote: Total 26 (delta 10), reused 25 (delta 9), pack-reused 0 (from 0)
Receiving objects: 100% (26/26), 10.30 KiB | 10.30 MiB/s, done.
Resolving deltas: 100% (10/10), done.
wink@3900x 26-03-04T16:10:37.171Z:~/data/prgs/rust
$ cd vc-x1/
wink@3900x 26-03-04T17:15:13.281Z:~/data/prgs/rust/vc-x1
$ rm -rf .git Cargo.toml Cargo.lock src
wink@3900x 26-03-04T17:15:23.347Z:~/data/prgs/rust/vc-x1
$ jj git init
Initialized repo in "."
Hint: Running `git clean -xdf` will remove `.jj/`!
$ jj st
Working copy changes:
A .gitignore
A CLAUDE.md
A LICENSE-APACHE
A LICENSE-MIT
A README.md
Working copy  (@) : qvnyrzox f53c3497 (no description set)
Parent commit (@-): zzzzzzzz 00000000 (empty) (no description set)
wink@3900x 26-03-04T17:17:29.325Z:~/data/prgs/rust/vc-x1 (main)
$ claude-symlink.sh .claude
Created target directory: .claude
Created: /home/wink/.claude/projects/-home-wink-data-prgs-rust-vc-x1 -> /home/wink/data/prgs/rust/vc-x1/.claude
wink@3900x 26-03-04T17:19:31.408Z:~/data/prgs/rust/vc-x1 (main)
$ printf ".git\n.jj\n" > .claude/.gitignore
wink@3900x 26-03-04T17:19:56.680Z:~/data/prgs/rust/vc-x1 (main)
$ cd .claude/
wink@3900x 26-03-04T17:20:15.182Z:~/data/prgs/rust/vc-x1/.claude
$ jj git init
Initialized repo in "."
Hint: Running `git clean -xdf` will remove `.jj/`!
wink@3900x 26-03-04T17:20:35.081Z:~/data/prgs/rust/vc-x1/.claude (main)
$ jj st
Working copy changes:
A .gitignore
Working copy  (@) : ponzrznv 526882d0 (no description set)
Parent commit (@-): zzzzzzzz 00000000 (empty) (no description set)
wink@3900x 26-03-04T17:20:46.906Z:~/data/prgs/rust/vc-x1/.claude (main)
$ jj describe -m "ponzrznv-qvnyrzox-Initial Commit" -m $'Future commits will contain claude-code session files`
Working copy  (@) now at: ponzrznv 90244032 ponzrznv-qvnyrzox-Initial Commit
Parent commit (@-)      : zzzzzzzz 00000000 (empty) (no description set)
wink@3900x 26-03-04T17:55:06.459Z:~/data/prgs/rust/vc-x1/.claude (main)
$ cd ..
wink@3900x 26-03-04T17:55:11.933Z:~/data/prgs/rust/vc-x1 (main)
$ jj describe -m "qvnyrzox-ponzrznv-Initial Commit" -m $'An experiment on creating a Vibe Coding environment'
Working copy  (@) now at: qvnyrzox 9d954685 qvnyrzox-ponzrznv-Initial Commit
Parent commit (@-)      : zzzzzzzz 00000000 (empty) (no description set)
wink@3900x 26-03-04T17:56:04.166Z:~/data/prgs/rust/vc-x1 (main)
```

The last step is to push the two repo to github, I used the web interface
on github to create vc-x1.git and vc-x1.claude.git. I then created the
remote reference, a main bookmark and push.

Here I'm pushing vc-x1:
```
wink@3900x 26-03-04T17:56:04.166Z:~/data/prgs/rust/vc-x1 (main)
$ jj git remote add origin git@github.com:winksaville/vc-x1.git
wink@3900x 26-03-04T18:09:48.521Z:~/data/prgs/rust/vc-x1 (main)
$ jj bookmark create main -r @
Created 1 bookmarks pointing to qvnyrzox 7517ee24 main | qvnyrzox-ponzrznv-Initial Commit
wink@3900x 26-03-04T18:10:03.265Z:~/data/prgs/rust/vc-x1 (refs/jj/root)
$ jj git push --bookmark main
Changes to push to origin:
  Add bookmark main to 7517ee249e03
git: Enumerating objects: 7, done.
git: Counting objects: 100% (7/7), done.
git: Delta compression using up to 24 threads
git: Compressing objects: 100% (6/6), done.
git: Writing objects: 100% (7/7), 7.69 KiB | 7.69 MiB/s, done.
git: Total 7 (delta 0), reused 0 (delta 0), pack-reused 0 (from 0)
Warning: The working-copy commit in workspace 'default' became immutable, so a new commit has been created on top of it.
Working copy  (@) now at: sxwktqsz a1103fe8 (empty) (no description set)
Parent commit (@-)      : qvnyrzox 7517ee24 main | qvnyrzox-ponzrznv-Initial Commit
wink@3900x 26-03-04T18:10:17.354Z:~/data/prgs/rust/vc-x1 ((main))
```

Here is vc-x1.claude:
```
wink@3900x 26-03-04T18:10:17.354Z:~/data/prgs/rust/vc-x1 ((main))
$ cd .claude
wink@3900x 26-03-04T18:13:17.830Z:~/data/prgs/rust/vc-x1/.claude (main)
$ jj git remote add origin git@github.com:winksaville/vc-x1.claude.git
wink@3900x 26-03-04T18:13:41.018Z:~/data/prgs/rust/vc-x1/.claude (main)
$ jj bookmark create main -r @
Created 1 bookmarks pointing to ponzrznv 90244032 main | ponzrznv-qvnyrzox-Initial Commit
wink@3900x 26-03-04T18:13:49.904Z:~/data/prgs/rust/vc-x1/.claude (refs/jj/root)
$ jj git push --bookmark main
Changes to push to origin:
  Add bookmark main to 90244032218c
git: Enumerating objects: 3, done.
git: Counting objects: 100% (3/3), done.
git: Writing objects: 100% (3/3), 307 bytes | 307.00 KiB/s, done.
git: Total 3 (delta 0), reused 0 (delta 0), pack-reused 0 (from 0)
Warning: The working-copy commit in workspace 'default' became immutable, so a new commit has been created on top of it.
Working copy  (@) now at: kttvktmm 0c376c25 (empty) (no description set)
Parent commit (@-)      : ponzrznv 90244032 main | ponzrznv-qvnyrzox-Initial Commit
wink@3900x 26-03-04T18:14:05.349Z:~/data/prgs/rust/vc-x1/.claude ((main))
```

## Use commit titles to cross-reference changeIDs

This was tried but I decided footers were more flexible and less noice than the commit title.
See [ChangeID footer syntax](#changeid-footer-syntax) for the final approach.

A **critical item** in this experiment is each commit can contain the changeID
of the other commit. Here I'm trying out providing both "this commit id" (tcid)
and "other commit id" (ocid) in the commit title to make it easier for tools to
navigate and synchronize the two repos.

title syntax: `<tcid>-<ocid>-<short description>`

 - `<tcid>` is the jj change ID of this commit and is never `none`.
 - `<ocid>` is the jj change ID of the other commit and may be `none` if the bot was not involved.
 - `<short description>` is a brief one line description of the chore.

The intent is that these might be able to make the tool that shows the
evolution of the code more clear by showing the relationship between the two repos

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

Commit footers can reference changeIDs and notes files using markdown
reference-link syntax. See [ChangeID footer syntax](#changeid-footer-syntax)
for the path rules.

Since jj changeIDs are generated at `jj git init` time (not stored in the
repo), changeID references are only resolvable by tools that have access
to the local jj repo (e.g. vc-x1).

## ChangeID footer syntax

Commit footers use markdown reference-link syntax to cross-reference
jj changeIDs across repos:

- `/changeID` — workspace-root repo (e.g. `/upxmvpyz`)
- `/<path>/changeID` — sub-repo within the workspace (e.g. `/.claude/vxmpmzsy`)

All changeID references must be workspace-root relative (start with `/`).

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
