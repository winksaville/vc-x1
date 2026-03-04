# vc-x1

This is experiment 1 to explore creating a Vibe Coding (vc) environment.
We will investigate ways of using the dual jj-git repo concept, explored
in [hw-jjg-bot](https://github.com/winksaville/hw-jjg-bot.git) to
initially make it easy to see how the code base evolved. This is
made possible by the fact that we have two repos one with the code
and one with the conversation with the bot.

I've chosen the jj-git environment because jj provides the concept that
each commit has an immutable changeID as well as the mutable commitID
of git. The idea is that each commit made on repo A writes the
changeID in the commit message to repo B. Thus there is a cross reference
between the two repos and this will allow vc-x1 to show how the repo
evolved and the entity (bot or human) can more clearly understand **how** and
most importantly **why** the code evolved.

The solution space is wide open, from trivial CLI, web or app based
(mobile/non-mobile). In addition, I could see this as an extension to
existing programming editors like vscode and zed or even creating our
own IDE for vc.

Initial creation steps is below, these were followed by creating this README.md.

A **critical item** in this experiment is each commit can contain the changeID
of the other commit. This is written on the last line description body as
"ocid: xxxxyyyy". The intent is that these are the cross-references to the
other commit allowing for guaranteed synchronization between the two repos.

Note: the two .gitignore files are explicitly ignoring .jj and .git
to be obvious although it maybe unnecessary.

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
$ jj describe -m "Initial Commit" -m $'Future commits will contain claude-code session files\n\nocid: qvnyrzox'
Working copy  (@) now at: ponzrznv 90244032 Initial Commit
Parent commit (@-)      : zzzzzzzz 00000000 (empty) (no description set)
wink@3900x 26-03-04T17:55:06.459Z:~/data/prgs/rust/vc-x1/.claude (main)
$ cd ..
wink@3900x 26-03-04T17:55:11.933Z:~/data/prgs/rust/vc-x1 (main)
$ jj describe -m "Initial Commit" -m $'An experiment on creating a Vibe Coding environment\n\nocid: ponzrznv'
Working copy  (@) now at: qvnyrzox 9d954685 Initial Commit
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
Created 1 bookmarks pointing to qvnyrzox 7517ee24 main | Initial Commit
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
Parent commit (@-)      : qvnyrzox 7517ee24 main | Initial Commit
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
Created 1 bookmarks pointing to ponzrznv 90244032 main | Initial Commit
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
Parent commit (@-)      : ponzrznv 90244032 main | Initial Commit
wink@3900x 26-03-04T18:14:05.349Z:~/data/prgs/rust/vc-x1/.claude ((main))
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
