#!/usr/bin/env bash
# vc-dr-apply.sh: apply a source dual repo's committed rules and agent files, and the skeletons
# beside this script, to a bare dual workspace that `vc-x1 init` made, new or adopted. A first
# cut: it writes only new files, refuses when any existing file would change, and commits nothing.
set -euo pipefail

die() {
    echo "vc-dr-apply: $*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: vc-dr-apply.sh [--to DIR] [--rev REV] [--agent-rev REV] [--dry-run] SOURCE [AGENT_SOURCE]

Apply what SOURCE holds at one commit per repo to the dual workspace DIR, made by `vc-x1 init`:

  SOURCE                a dual workspace, its agent repo found by its .vc-config.md
  SOURCE AGENT_SOURCE   a work repo and an agent repo, each given

  --to DIR         the workspace written [default: .]
  --rev REV        the source work repo's commit, any jj revision naming one [default: main]
  --agent-rev REV  the source agent repo's commit [default: main]
  --dry-run        print the plan and write nothing

From the work repo: AGENTS.md, agent-data/, and LICENSE-*. From the agent repo: LICENSE-*,
README.md, its first line renamed for DIR's agent repo, and memory/MEMORY.md, each when present.
From scripts/vc-dr-apply/: the work repo's skeletons, {name} the workspace's directory name, and
the .gitignore lines.

Both repos of DIR must have an empty `@`, so every write is a diff `jj restore` undoes. Each
path is new, identical, or differs, judged on disk. New is written and identical skipped, and
any path that differs refuses the whole apply, nothing written. `.gitignore` takes the lines it
lacks. Nothing is committed: the end names the commit to make after review.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
skel=$script_dir/vc-dr-apply
to=.
rev=main
agent_rev=main
dry_run=0
sources=()
while (($#)); do
    case $1 in
    --to | --rev | --agent-rev)
        (($# >= 2)) || die "$1 takes a value"
        case $1 in
        --to) to=$2 ;;
        --rev) rev=$2 ;;
        --agent-rev) agent_rev=$2 ;;
        esac
        shift 2
        ;;
    --dry-run)
        dry_run=1
        shift
        ;;
    -h | --help)
        usage
        exit 0
        ;;
    -*) die "unknown option '$1', see --help" ;;
    *)
        sources+=("$1")
        shift
        ;;
    esac
done
((${#sources[@]} == 1 || ${#sources[@]} == 2)) || die "takes SOURCE or SOURCE AGENT_SOURCE, see --help"

# The agent repo a workspace's .vc-config.md names, absolute.
agent_of() {
    local ws=$1 dir
    [[ -f $ws/.vc-config.md ]] || die "'$ws' has no .vc-config.md"
    dir=$(sed -n '/^\[repos\]/,/^\[/{s/^agent *= *"\(.*\)"/\1/p}' "$ws/.vc-config.md" | head -n1)
    [[ -n $dir ]] || die "'$ws' declares no [repos] agent: it is not a dual workspace"
    case $dir in
    /*) echo "$dir" ;;
    *) echo "$ws/$dir" ;;
    esac
}

# The one commit REV names in REPO, refused unless it is exactly one.
commit_of() {
    local repo=$1 r=$2 sha
    sha=$(jj -R "$repo" log --no-graph -r "$r" -T 'commit_id ++ "\n"')
    [[ -n $sha && $(wc -l <<<"$sha") -eq 1 ]] || die "'$r' must name exactly one commit in '$repo'"
    echo "$sha"
}

# Whether PATH exists in REPO at SHA.
in_commit() {
    git -C "$1" cat-file -e "$2:$3" 2>/dev/null
}

absolute() {
    [[ -d $1 ]] || die "'$1' is not a directory"
    (cd "$1" && pwd)
}

# --- The source, and the commit of each repo ---------------------------------------------------

src=$(absolute "${sources[0]}")
if ((${#sources[@]} == 2)); then
    src_agent=$(absolute "${sources[1]}")
else
    src_agent=$(agent_of "$src")
fi
for repo in "$src" "$src_agent"; do
    [[ -d $repo/.jj ]] || die "the source '$repo' is not a jj repo"
done
sha=$(commit_of "$src" "$rev")
agent_sha=$(commit_of "$src_agent" "$agent_rev")

# --- The target workspace ----------------------------------------------------------------------

ws=$(absolute "$to")
agent=$(agent_of "$ws")
[[ -d $agent/.jj ]] || die "the agent repo '$agent' is not a jj repo"
[[ $ws != "$src" ]] || die "the source and the target are the same workspace"
name=$(basename "$ws")
agent_name=$(sed -n '/^\[remote\]/,/^\[/{s/^agent-repo *= *"\(.*\)"/\1/p}' "$ws/.vc-config.md" | head -n1)
[[ -n $agent_name ]] || agent_name=$name.agent-session

for repo in "$ws" "$agent"; do
    [[ -z $(jj -R "$repo" diff --summary) ]] ||
        die "'$repo' has uncommitted changes: commit or restore them first, so the apply is one diff"
done

# --- Stage what would be written ---------------------------------------------------------------

stage=$(mktemp -d "${TMPDIR:-/tmp}/vc-dr-apply.XXXXXX")
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/work" "$stage/agent"

# Archive the PATHS present in REPO at SHA into DEST.
take() {
    local repo=$1 at=$2 dest=$3 p
    shift 3
    local present=()
    for p in "$@"; do
        if in_commit "$repo" "$at" "$p"; then present+=("$p"); fi
    done
    ((${#present[@]})) || return 0
    git -C "$repo" archive "$at" "${present[@]}" | tar -x -C "$dest"
}

in_commit "$src" "$sha" AGENTS.md || die "the source work repo has no AGENTS.md at ${sha:0:12}"
take "$src" "$sha" "$stage/work" AGENTS.md agent-data LICENSE-APACHE LICENSE-MIT
take "$src_agent" "$agent_sha" "$stage/agent" LICENSE-APACHE LICENSE-MIT README.md memory/MEMORY.md
if [[ -f $stage/agent/README.md ]]; then
    sed -i "1s|.*|# $agent_name|" "$stage/agent/README.md"
fi
cp -R "$skel/work/." "$stage/work/"
while IFS= read -r -d '' f; do
    sed -i "s|{name}|$name|g" "$f"
done < <(grep -rlZ '{name}' "$stage/work" || true)
[[ -z $(find "$stage" -type l) ]] || die "the staged files hold a symlink, refused"

# --- The plan ----------------------------------------------------------------------------------

new=()
same=()
differs=()

# Why REL cannot be written under DEST: it, or a directory it needs, is a symlink or the wrong
# kind of entry. Empty when it can.
blocked() {
    local dest=$1 rel=$2 path=$1 part
    local parts
    IFS=/ read -r -a parts <<<"$rel"
    for part in "${parts[@]::${#parts[@]}-1}"; do
        path=$path/$part
        if [[ -L $path || (-e $path && ! -d $path) ]]; then
            echo "$part is not a directory"
            return
        fi
    done
    if [[ -L $dest/$rel ]]; then
        echo "a symlink"
    elif [[ -d $dest/$rel ]]; then
        echo "a directory"
    fi
}

plan_side() {
    local side=$1 dest=$2 f rel why
    while IFS= read -r -d '' f; do
        rel=${f#"$stage/$side/"}
        why=$(blocked "$dest" "$rel")
        if [[ -n $why ]]; then
            differs+=("$side $rel ($why)")
        elif [[ ! -e $dest/$rel ]]; then
            new+=("$side $rel")
        elif cmp -s "$f" "$dest/$rel"; then
            same+=("$side $rel")
        else
            differs+=("$side $rel")
        fi
    done < <(find "$stage/$side" -type f -print0 | sort -z)
}

plan_side work "$ws"
plan_side agent "$agent"

ignore_add=()
while IFS= read -r line; do
    [[ -f $ws/.gitignore ]] && grep -qxF -- "$line" "$ws/.gitignore" && continue
    ignore_add+=("$line")
done <"$skel/gitignore-lines"

echo "vc-dr-apply: from $src at ${sha:0:12} ($rev), its agent repo at ${agent_sha:0:12} ($agent_rev)"
echo "vc-dr-apply: onto $ws, agent repo $agent"
for x in ${new[@]+"${new[@]}"}; do echo "  new       $x"; done
for x in ${same[@]+"${same[@]}"}; do echo "  identical $x"; done
for x in ${differs[@]+"${differs[@]}"}; do echo "  differs   $x"; done
for x in ${ignore_add[@]+"${ignore_add[@]}"}; do echo "  ignore    work .gitignore + $x"; done

if ((${#differs[@]})); then
    die "${#differs[@]} path(s) differ from what would be written, so nothing was written"
fi
if ((dry_run)); then
    echo "vc-dr-apply: dry run, nothing written"
    exit 0
fi

# --- Apply -------------------------------------------------------------------------------------

for x in ${new[@]+"${new[@]}"}; do
    side=${x%% *}
    rel=${x#* }
    if [[ $side == work ]]; then dest=$ws; else dest=$agent; fi
    mkdir -p "$(dirname "$dest/$rel")"
    cp "$stage/$side/$rel" "$dest/$rel"
done
if ((${#ignore_add[@]})); then
    printf '%s\n' "${ignore_add[@]}" >>"$ws/.gitignore"
fi

echo "vc-dr-apply: applied, nothing committed. custom.md is written and gitignored, local."
echo "--- work repo"
(cd "$ws" && jj diff --stat)
echo "--- agent repo"
(cd "$agent" && jj diff --stat)
src_name=$(basename "$src")
cat <<EOF
Review both diffs, then commit, for example:
  vc-x1 commit -s b -R $ws --title "chore: apply $src_name's rules" \\
    --body "A bare workspace has no rules to work by.

Solution: $src_name's agent-files and agent files, from $src_name at ${sha:0:12} and its agent repo at ${agent_sha:0:12}."
EOF
