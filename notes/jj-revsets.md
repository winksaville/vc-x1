Learning about revsets to develop a mental model of jj
and I used notes/substep-test.sh written by the bot to
create the repo here.

To develop a mental model look at the examples and
develop explanations for them. 

The basic information is that revsets, set of revisions,
are used to address commits. A revision can be identified
using "@", chid or cid and revsets can be created using
those identifiers plus operators "-", "+", ".." and "::".
There is a complete language available, see
`jj help -k revsets` for more information.

Summary:
 - A revision (rev) used for addressing/referring to commits
 - jj support a change_id (chid) and commit_id (cid)
   - The chid is permanent and doesn't change
   - the cid is the sha of the current commit and
     changes with any change to it or its ancestors
 - A revset is a set of revisions
 - In many jj commands revsets can be use to address multiple commits
 - There is one root commit; chid=zzz sha=0 owner=root
 - The root commit never has content nor description
 - There can be multiple independent lineages of commits off root()
   - to create:
     - `jj new 'root()'`
     - `git switch --orphan <name>`
   - to see these:
     - `jj log -r 'roots(~root())'`
     - `git log --all --max-parents=0 --oneline`

The examples below show these primitives in action.

## A jj repo with some commits

Some examples of revisions/rev:
   - @, chid=tkpvsrop, cid=ad2ba9b4 is the current commit
   - @-, chid=wxtmosqz, cid=35ac2422 is the ancestor of the current commit
   - @--, chid=vktlnyvm, cid=a0b0285e is the ancestor of the ancestor and so on

```
wink@3900x 26-05-01T16:44:41.913Z:/tmp/substep-test (@tkpvsrop)
$ jj log -r 'all()'
@  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
○  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
◆  zzzzzzzz root() 00000000
wink@3900x 26-05-01T16:45:21.527Z:/tmp/substep-test (@tkpvsrop)
```

Move @ to vktlnyvm. (This could also have been done with `jj edit -r @--`.)

```
wink@3900x 26-05-01T16:45:21.527Z:/tmp/substep-test (@tkpvsrop)
$ jj edit -r vkt
Working copy  (@) now at: vktlnyvm a0b0285e count 1
Parent commit (@-)      : nvwytkrw 54fd18f7 base | count 0
Added 0 files, modified 1 files, removed 0 files
wink@3900x 26-05-01T16:58:46.466Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r 'all()'
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
◆  zzzzzzzz root() 00000000
wink@3900x 26-05-01T16:58:57.546Z:/tmp/substep-test (@vktlnyvm)
```

## Example of referencing commits with relative revsets


```
wink@3900x 26-05-01T17:16:46.893Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
~
wink@3900x 26-05-01T17:16:49.873Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @-
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
~
wink@3900x 26-05-01T17:16:52.804Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @--
◆  zzzzzzzz root() 00000000
wink@3900x 26-05-01T17:16:56.477Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @+
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
~
wink@3900x 26-05-01T17:17:02.607Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @++
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
~
wink@3900x 26-05-01T17:17:07.256Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @+++
wink@3900x 26-05-01T17:17:18.436Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @---
wink@3900x 26-05-01T17:17:24.021Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @..
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
~
wink@3900x 26-05-01T17:18:05.778Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r @::
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
~
wink@3900x 26-05-01T17:18:22.096Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r ..@
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
~
wink@3900x 26-05-01T17:18:50.526Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r ::@
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
◆  zzzzzzzz root() 00000000
wink@3900x 26-05-01T17:19:07.714Z:/tmp/substep-test (@vktlnyvm)
```

### Interpretation

- `@-` and `@--` resolve to single revisions: parent and
  grandparent of @. `@+` and `@++` are children. The
  blank output for `@+++` and `@---` is the empty revset
  — there is no revision that far away in this chain.
- `@..` and `@::` are ranges going outward from @ toward
  descendants:
  - `@..` is the *open* form: descendants of @, **excluding**
    @ itself (here: count 2 and count 3).
  - `@::` is the *closed* form: descendants of @, **including**
    @ itself (here: @=count 1, count 2, count 3).
- `..@` and `::@` are ranges going inward from some implicit
  start toward ancestors of @:
  - `..@` includes @ and its ancestors but **excludes** the
    root commit (here: @=count 1, base=count 0).
  - `::@` includes the root commit as well (here: @=count 1,
    base=count 0, root).

Mnemonic: `..` and `::` both produce ranges; `::` includes
the implicit endpoint (root or visible heads), `..` excludes
it. The named operand is always part of the result on the
target side; on the source side it depends on which dot-form
is used (excluded by `..`, included by `::`).

## Example of referencing commits with absolute revsets

```
wink@3900x 26-05-01T17:19:07.714Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r v
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
~
wink@3900x 26-05-01T17:24:10.385Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r vktln
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
~
wink@3900x 26-05-01T17:24:26.707Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r w+
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
~
wink@3900x 26-05-01T17:25:10.856Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r w--
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
~
wink@3900x 26-05-01T17:25:22.852Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r w..
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
~
wink@3900x 26-05-01T17:25:51.559Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r ..w
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
~
wink@3900x 26-05-01T17:26:01.771Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r w::
○  tkpvsrop wink@saville.com 2026-05-01 08:41:05 ad2ba9b4
│  count 3
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
~
wink@3900x 26-05-01T17:26:13.067Z:/tmp/substep-test (@vktlnyvm)
$ jj log -r ::w
○  wxtmosqz wink@saville.com 2026-05-01 08:41:05 35ac2422
│  count 2
@  vktlnyvm wink@saville.com 2026-05-01 08:41:05 a0b0285e
│  count 1
○  nvwytkrw wink@saville.com 2026-05-01 08:41:05 base 54fd18f7
│  count 0
◆  zzzzzzzz root() 00000000
wink@3900x 26-05-01T17:26:27.937Z:/tmp/substep-test (@vktlnyvm)
```

### Interpretation

- `v` and `vktln` both resolve to chid `vktlnyvm` because they
  are unambiguous prefixes within this repo — no other chid
  starts with those letters.
- `w+` resolves in two stages: `w` matches `wxtmosqz` (count 2)
  by prefix, then `+` takes its child, giving count 3.
- `w--` grandparent of w.
- `w..` descendants of w, excluding w.
- `..w` ancestors of w, not including root.
- `::w` ancestors of w, including root.
- `w::` descendants of w, including w.

Prefix rule: jj rejects ambiguous prefixes. If two chids
both start with the same letters, the prefix must be
lengthened until exactly one chid matches.


