#!/usr/bin/env bash
# vc-dr-apply.sh: apply a source dual repo's rules and agent files, and its skeletons, as its
# working copy holds them or as landed, to a bare dual workspace that `vc-x1 init` made, new or
# adopted. It may live anywhere, ~/bin included, since the skeletons come from the source. A
# first cut: it writes only new files, refuses when any existing file would change, and commits
# nothing.
set -euo pipefail

die() {
    echo "vc-dr-apply: $*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: vc-dr-apply.sh [--to DIR] [--landed] [--rev REV] [--agent-rev REV] [--skel DIR]
                      [--lang NAME] [--dry-run] SOURCE [AGENT_SOURCE]
       vc-dr-apply.sh --lang [--skel DIR] [SOURCE]

Apply what SOURCE holds, one tree per repo, to the dual workspace DIR, made by `vc-x1 init`:

  SOURCE                a dual workspace, its agent repo found by its .vc-config.md
  SOURCE AGENT_SOURCE   a work repo and an agent repo, each given

  --to DIR         the workspace written [default: .]
  --landed         both repos at main, what landed, rather than the working copies
  --rev REV        the source work repo's commit, any jj revision naming one [default: @,
                   the working copy as on disk, which jj snapshots first]
  --agent-rev REV  the source agent repo's commit [default: @, as --rev]
  --skel DIR       the skeletons, a directory holding vc-dr-apply-base/ and vc-dr-apply-lang/,
                   as on disk [default: the source work repo's scripts/ at --rev, else this
                   script's own directory, symlinks resolved]
  --lang NAME      also apply the language skeleton vc-dr-apply-lang/NAME/, its [validate]
                   table appended to DIR's .vc-config.md; --lang alone lists them
  --dry-run        print the plan and write nothing

The default is the active version, the working copy with what it has not committed, so a test
repo gets the rules being worked on. --landed takes the trunk's, and --rev or --agent-rev, given
after it, overrides it for that repo. The skeletons follow the work repo's revision, so they
match the agent-files they come with.

From the work repo: AGENTS.md, agent-data/, and LICENSE-*. From the agent repo: LICENSE-*,
README.md, its first line renamed for DIR's agent repo, and memory/MEMORY.md, each when present.
From vc-dr-apply-base/: work/, the work repo's skeletons, {name} the workspace's directory name
and {module} it with each '-' an '_', in file contents and paths, and gitignore-lines, the
.gitignore lines.

A language skeleton, vc-dr-apply-lang/NAME/, holds the project's starting files, written like the
others, and three files that are not: validate.toml, the [validate] table appended to
.vc-config.md, refused when it has one already; gitignore-lines, more .gitignore lines; and
replaces, the base skeleton's paths it supersedes, left out, version.toml when the build system
holds the version. A new language is a new directory with a validate.toml.

Both repos of DIR must have an empty `@`, so every write is a diff `jj restore` undoes. Each
path is new, identical, or differs, judged on disk. New is written and identical skipped, and
any path that differs refuses the whole apply, nothing written. `.gitignore` takes the lines it
lacks. Nothing is committed: the end names the commit to make after review.
EOF
}

script_dir=$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")
to=.
rev=@
agent_rev=@
skel_opt=
dry_run=0
lang=
list_langs=0
sources=()
while (($#)); do
    case $1 in
    --to | --rev | --agent-rev | --skel)
        (($# >= 2)) || die "$1 takes a value"
        case $1 in
        --to) to=$2 ;;
        --rev) rev=$2 ;;
        --agent-rev) agent_rev=$2 ;;
        --skel) skel_opt=$2 ;;
        esac
        shift 2
        ;;
    --dry-run)
        dry_run=1
        shift
        ;;
    --landed)
        rev=main
        agent_rev=main
        shift
        ;;
    --lang)
        # A language name has no '/', so a path after --lang is the SOURCE to list from.
        if (($# < 2)) || [[ $2 == -* || $2 == */* || $2 == . || $2 == .. ]]; then
            list_langs=1
            shift
        else
            lang=$2
            shift 2
        fi
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
if ((list_langs)); then
    ((${#sources[@]} || ${#skel_opt})) || die "--lang alone lists from SOURCE or --skel DIR, see --help"
else
    ((${#sources[@]} == 1 || ${#sources[@]} == 2)) || die "takes SOURCE or SOURCE AGENT_SOURCE, see --help"
fi

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

# The one commit REV names in REPO, refused unless it is exactly one. A jj command snapshots the
# working copy first, so `@` is the tree on disk, and jj keeps its commit in the git store, where
# `git archive` reads it.
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

# What a revision is, for the plan and the commit body: the working copy by its change id, which
# survives the snapshot's rewrites, or the revision as given.
named() {
    local repo=$1 r=$2 at=$3
    if [[ $r == @ ]]; then
        echo "its working copy, change $(jj -R "$repo" log --no-graph -r @ -T 'change_id.short(12)')"
    else
        echo "${at:0:12} ($r)"
    fi
}

tmp=$(mktemp -d "${TMPDIR:-/tmp}/vc-dr-apply.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

# --- The source, and the commit of each repo ---------------------------------------------------

if ((${#sources[@]})); then
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
    src_at=$(named "$src" "$rev" "$sha")
    agent_at=$(named "$src_agent" "$agent_rev" "$agent_sha")
fi

# --- The skeletons -----------------------------------------------------------------------------

if [[ -n $skel_opt ]]; then
    skel=$(absolute "$skel_opt")
    skel_from="--skel $skel"
elif ((${#sources[@]})) && in_commit "$src" "$sha" scripts/vc-dr-apply-base; then
    mkdir -p "$tmp/skel"
    paths=(scripts/vc-dr-apply-base)
    if in_commit "$src" "$sha" scripts/vc-dr-apply-lang; then paths+=(scripts/vc-dr-apply-lang); fi
    git -C "$src" archive "$sha" "${paths[@]}" | tar -x -C "$tmp/skel"
    skel=$tmp/skel/scripts
    skel_from="the source's scripts/ at $src_at"
else
    skel=$script_dir
    skel_from="this script's directory, $script_dir, the source having none"
fi
langs_dir=$skel/vc-dr-apply-lang

# The language skeletons, one per directory with a validate.toml.
langs() {
    local d
    for d in "$langs_dir"/*/; do
        if [[ -f $d/validate.toml ]]; then basename "$d"; fi
    done
}

if ((list_langs)); then
    echo "vc-dr-apply: the language skeletons, from $skel_from:"
    langs | sed 's/^/  /'
    exit 0
fi
[[ -d $skel/vc-dr-apply-base/work ]] || die "no vc-dr-apply-base/work in $skel_from, see --skel"
lang_dir=
if [[ -n $lang ]]; then
    lang_dir=$langs_dir/$lang
    [[ -f $lang_dir/validate.toml ]] ||
        die "no language skeleton '$lang' in $skel_from, the choices: $(langs | paste -sd ' ')"
fi

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

stage=$tmp/stage
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
cp -R "$skel/vc-dr-apply-base/work/." "$stage/work/"
if [[ -n $lang_dir ]]; then
    if [[ -f $lang_dir/replaces ]]; then
        while IFS= read -r x; do
            if [[ -n $x ]]; then rm -f "$stage/work/$x"; fi
        done <"$lang_dir/replaces"
    fi
    cp -R "$lang_dir/." "$stage/work/"
    rm -f "$stage/work/validate.toml" "$stage/work/gitignore-lines" "$stage/work/replaces"
fi
module=${name//-/_}
while IFS= read -r -d '' f; do
    sed -i "s|{name}|$name|g; s|{module}|$module|g" "$f"
done < <(grep -rlZ -e '{name}' -e '{module}' "$stage/work" || true)
# A path naming {name} or {module} is renamed, the deepest first.
while IFS= read -r -d '' f; do
    base=$(basename "$f")
    base=${base//\{name\}/$name}
    mv "$f" "$(dirname "$f")/${base//\{module\}/$module}"
done < <(find "$stage/work" -depth \( -name '*{name}*' -o -name '*{module}*' \) -print0)
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
done < <(cat "$skel/vc-dr-apply-base/gitignore-lines" ${lang_dir:+"$lang_dir/gitignore-lines"} 2>/dev/null)

# The [validate] table a language skeleton brings, refused when the config has one.
validate_add=
if [[ -n $lang_dir ]]; then
    if grep -q '^\[validate\]' "$ws/.vc-config.md"; then
        differs+=("work .vc-config.md (it has a [validate] table, so $lang's is not added)")
    else
        validate_add=$(cat "$lang_dir/validate.toml")
    fi
fi

echo "vc-dr-apply: from $src at $src_at, its agent repo at $agent_at"
echo "vc-dr-apply: skeletons from $skel_from"
echo "vc-dr-apply: onto $ws, agent repo $agent"
for x in ${new[@]+"${new[@]}"}; do echo "  new       $x"; done
for x in ${same[@]+"${same[@]}"}; do echo "  identical $x"; done
for x in ${differs[@]+"${differs[@]}"}; do echo "  differs   $x"; done
for x in ${ignore_add[@]+"${ignore_add[@]}"}; do echo "  ignore    work .gitignore + $x"; done
[[ -z $validate_add ]] || echo "  config    work .vc-config.md + [validate], the $lang skeleton's"

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
if [[ -n $validate_add ]]; then
    printf '\nThe validation commands, from the %s language skeleton\n```toml\n%s\n```\n' \
        "$lang" "$validate_add" >>"$ws/.vc-config.md"
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

Solution: $src_name's agent-files and agent files, from $src_name at $src_at and its agent repo at $agent_at."
EOF
