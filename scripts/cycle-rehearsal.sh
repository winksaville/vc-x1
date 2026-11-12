#!/usr/bin/env bash
# cycle-rehearsal.sh - run the cycle flow end to end, by `cycle`'s flags.
#
# Makes a throwaway dual workspace with local bare remotes and no cargo
# project, and runs three cycles through it, open to Land, checking
# what each command left:
#
#   1. a single-step cycle with nothing committed before its closing
#   2. a single-step cycle whose opening is committed first
#   3. a ladder, closed as a trapezoid, with each way back from a fault
#      found after the closing tried: fix inside, fix on top, reopen
#
# Usage: scripts/cycle-rehearsal.sh [DIR]
#   DIR  where the workspace and its remotes go, removed first
#        [default: ~/tmp/cycle-rehearsal]
#   VC   the binary under test [default: vc-x1-dev]
#
# Exits zero when every check passed. The ways back the tool refuses
# are reported, not failed: the rehearsal's job there is to say which.

set -uo pipefail

VC=${VC:-vc-x1-dev}
ROOT=${1:-$HOME/tmp/cycle-rehearsal}
WS=$ROOT/ws
AGENT=$WS/.agent-session
LOG=$ROOT/rehearsal.log
LINK=$HOME/.claude/projects/$(printf '%s' "$WS" | tr '/.' '--')

checks=0
failed=0
refused=()

say() { printf '%s\n' "$*"; }
step() { say; say "== $*"; }

# Run a command that must succeed, its output kept in the log.
run() {
    say "  \$ $*" | tee -a "$LOG"
    if ! "$@" >>"$LOG" 2>&1; then
        say "  FAILED (exit $?): $*, see $LOG"
        failed=$((failed + 1))
        return 1
    fi
}

# Run a command that is expected to be refused, and say which way it went.
tries() {
    local what=$1
    shift
    say "  \$ $*" | tee -a "$LOG"
    if "$@" >>"$LOG" 2>&1; then
        say "  done: $what"
    else
        say "  refused: $what"
        refused+=("$what")
    fi
}

# check <what> <actual> <expected>
check() {
    checks=$((checks + 1))
    if [ "$2" = "$3" ]; then
        say "  ok: $1"
    else
        say "  NOT OK: $1"
        say "    expected: $3"
        say "    actual:   $2"
        failed=$((failed + 1))
    fi
}

# check_has <what> <text> <fragment>
check_has() {
    checks=$((checks + 1))
    case "$2" in
    *"$3"*) say "  ok: $1" ;;
    *)
        say "  NOT OK: $1"
        say "    missing: $3"
        failed=$((failed + 1))
        ;;
    esac
}

jjw() { jj --ignore-working-copy -R "$WS" "$@"; }
jja() { jj --ignore-working-copy -R "$AGENT" "$@"; }
title() { jjw log -r "$1" --no-graph -T 'description.first_line()'; }
cid() { jjw log -r "$1" --no-graph -T 'commit_id'; }
agent_cid() { jja log -r "$1" --no-graph -T 'commit_id'; }
bookmarks() { jjw bookmark list -T 'name ++ "\n"' | sort -u | tr '\n' ' '; }
remote_bookmarks() {
    git -C "$ROOT/remotes/ws.git" for-each-ref --format='%(refname:short)' refs/heads | tr '\n' ' '
}
above_main() { jjw log -r "$1..$2" --no-graph --reversed -T 'description.first_line() ++ "\n"'; }
nparents() { jjw log -r "$1" --no-graph -T 'parents.len()'; }
ladder() { sed -n '/^## Ladder$/,/^## The TOC$/p' "$WS/zTOC.md" | grep '^- '; }

# What every Land leaves: the destination published in both repos, the
# cycle's bookmark gone from both sides, the record closed, and the
# work repo clean.
check_landed() {
    local name=$1 slug=$2
    check "$name: main is published" "$(cid main)" "$(cid main@origin)"
    check "$name: the agent main is published" "$(agent_cid main)" "$(agent_cid main@origin)"
    check "$name: only main is left locally" "$(bookmarks)" "main "
    check "$name: only main is left on origin" "$(remote_bookmarks)" "main "
    check_has "$name: nothing is in progress" "$(cat "$WS/TODO.md")" \
        "_No cycle currently in progress._"
    check "$name: the work @ is empty" "$(jj -R "$WS" log -r @ --no-graph -T 'empty')" "true"
    check_has "$name: the landed commit names its partner" \
        "$(jjw log -r main --no-graph -T description)" "ochid: "
    : "$slug"
}

step "Workspace: $WS, with $VC"
if [ -L "$LINK" ]; then rm "$LINK"; fi
rm -rf "$ROOT"
mkdir -p "$ROOT"
: >"$LOG"
run "$VC" init --target "$WS" --test-repo "$ROOT/remotes" || exit 1
cd "$WS" || exit 1

cat >TODO.md <<'EOF'
# Todo list

## Continuation notes

_None._

## In Progress

_No cycle currently in progress._

## Todo

### feat: a ladder cycle

The ladder's problem, moved into the record by the open.

## Closed

# References
EOF
cat >zTOC.md <<'EOF'
# TOC

## Ladder

_None._

## The TOC

- [TODO.md](TODO.md): the todo list.
EOF
run "$VC" commit -s b --title "chore: the record's skeleton" --body "The skeleton." \
    --bookmark main --push
check "the skeleton is on main" "$(title main)" "chore: the record's skeleton"

step "Cycle 1: single-step, nothing committed before the closing"
run "$VC" cycle --open --title "feat: one commit"
check "open made the bookmark" "$(bookmarks)" "feat-one-commit main "
check_has "open wrote the scaffold" "$(cat TODO.md)" "### feat: one commit"
check "open wrote the ladder" "$(ladder | head -1)" \
    "- 0 (current) [feat: one commit opening](TODO.md#feat-one-commit-opening)"
echo "one" >work.txt
run "$VC" cycle --close -s b --body "The problem.

Solution: the one commit." --land main --push
check "one commit above the old main" "$(above_main 'main@origin-' main)" "feat: one commit"
check "the ladder is one unlinked rung" "$(ladder)" "- 0 (done) feat: one commit"
check_has "the record is in Closed" "$(sed -n '/^## Closed$/,$p' TODO.md)" "### feat: one commit"
check_landed "cycle 1" feat-one-commit

step "Cycle 2: single-step, its opening committed first"
base=$(cid main)
run "$VC" cycle --open --title "fix: opened first"
run "$VC" cycle --commit -s b
check "the opening is committed" "$(title @-)" "fix: opened first opening"
check_has "the opening has the default body" "$(jjw log -r @- --no-graph -T description)" \
    'Opens the cycle "fix: opened first".'
opening=$(jjw log -r @- --no-graph -T 'change_id.short(12)')
echo "two" >>work.txt
run "$VC" cycle --close -s b --default-body
check "the closing is squashed into the opening" "$(above_main "$base" @-)" "fix: opened first"
check "the opening's change is kept" "$(jjw log -r @- --no-graph -T 'change_id.short(12)')" \
    "$opening"
run "$VC" cycle --land main -s b --push
check "the one commit landed" "$(title main)" "fix: opened first"
check_landed "cycle 2" fix-opened-first

step "Cycle 3: a ladder, closed as a trapezoid"
base=$(cid main)
run "$VC" cycle --open --title "feat: a ladder cycle"
check_has "open moved the Todo entry" "$(sed -n '/^## In Progress$/,/^## Todo$/p' TODO.md)" \
    "The ladder's problem, moved into the record by the open."
# Plan two work rungs: a heading each, above the closing's.
sed -i 's/^##### feat: a ladder cycle closing$/##### feat: the first rung\n\nIts intent.\n\n##### feat: the second rung\n\nIts intent.\n\n&/' TODO.md
run "$VC" cycle --ladder
check "the ladder has the rungs planned" "$(ladder | wc -l | tr -d ' ')" "4"
run "$VC" cycle --commit -s b
echo "first" >first.txt
run "$VC" cycle --commit -s b --title "feat: the first rung" --body "Why.

Solution: how."
check "the first rung is done and the second current" "$(ladder | sed -n 2,3p)" \
    "- 1 (done) [feat: the first rung](TODO.md#feat-the-first-rung)
- 2 (current) [feat: the second rung](TODO.md#feat-the-second-rung)"
echo "stray" >stray.txt
tries "a commit whose title is no rung's" \
    "$VC" cycle --commit -s b --title "feat: unplanned" --body "b"
rm stray.txt
echo "second" >second.txt
run "$VC" cycle --commit -s b --title "feat: the second rung" --body "Why.

Solution: how."
check "main has not moved" "$(cid main)" "$base"
run "$VC" push -s b
check "the draft push published the cycle's bookmark" "$(remote_bookmarks)" \
    "feat-a-ladder-cycle main "
run "$VC" cycle --close -s b --trapezoid main
check "the closing is titled" "$(title @-)" "feat: a ladder cycle closing"
check "the closing is a merge of two" "$(nparents @-)" "2"
check "every rung is done" "$(ladder | grep -c '(done)')" "4"

step "Cycle 3: the ways back from a fault found after the closing"
say "  fix inside: the fix folded into the closing commit"
echo "inside" >inside.txt
tries "fix inside, by \`squash -s b\`" "$VC" squash -s b
check "the closing holds the fix" "$(jjw file list -r @- 'root:"inside.txt"')" "inside.txt"
check "the closing is still a merge of two" "$(nparents @-)" "2"

say "  fix on top: a rung above the closing"
echo "on top" >ontop.txt
tries "fix on top, by \`cycle --commit\`" \
    "$VC" cycle --commit -s b --title "fix: on top" --body "b"
rm -f ontop.txt

say "  reopen: the closing dropped, a fix rung added, the cycle closed again"
closing=$(cid @-)
tip=$(jjw log -r 'heads(@-- ~ main)' --no-graph -T 'commit_id' 2>/dev/null)
tip=${tip:-$(jjw log -r '@-' --no-graph -T 'parents.map(|p| p.commit_id()).join(" ")' | awk '{print $2}')}
run jj -R "$WS" bookmark set feat-a-ladder-cycle -r "$tip" --allow-backwards
run jj -R "$WS" abandon "$closing"
run jj -R "$WS" new feat-a-ladder-cycle
check_has "the record is in progress again" "$(sed -n '/^## In Progress$/,/^## Todo$/p' TODO.md)" \
    "### feat: a ladder cycle"
sed -i 's/^##### feat: a ladder cycle closing$/##### fix: the fault\n\nIts intent.\n\n&/' TODO.md
echo "fixed" >fix.txt
tries "reopen, a fix rung by \`cycle --commit\`" \
    "$VC" cycle --commit -s b --title "fix: the fault" --body "Why.

Solution: how."
tries "reopen, the closing made again" "$VC" cycle --close -s b --trapezoid main
check "the cycle closed again" "$(title @-)" "feat: a ladder cycle closing"
check "the fix is a rung of it" "$(above_main "$base" @- | grep -c '^fix: the fault$')" "1"

step "Cycle 3: the Land"
run "$VC" cycle --land main -s b --push
check "the closing landed" "$(title main)" "feat: a ladder cycle closing"
check "the ladder is under the closing" "$(above_main "$base" main | head -1)" \
    "feat: a ladder cycle opening"
check_landed "cycle 3" feat-a-ladder-cycle

step "Result"
if [ -L "$LINK" ]; then rm "$LINK"; fi
say "  $checks checks, $failed failed, the log in $LOG"
if [ ${#refused[@]} -gt 0 ]; then
    say "  refused by the tool:"
    for r in "${refused[@]}"; do say "    - $r"; done
fi
[ "$failed" -eq 0 ]
