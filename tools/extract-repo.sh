#!/usr/bin/env bash
# extract-repo.sh <agent-cockpit|vitals> <destination>: a new repository at <destination> holding
# the paths of tools/extract/<name>.paths with their whole history (git filter-repo), plus one
# commit with the files of tools/extract/<name>/ (Makefile, README, CLAUDE.md, the workspace).
# Runs on the host, from the running copy (~/dotfiles): its committed state, never the work tree.
# Used once per repository for the move out of dotfiles (todo f4f67e5a); deleted afterwards.
set -euo pipefail

name=${1:?usage: extract-repo.sh <agent-cockpit|vitals> <destination>}
destination=${2:?usage: extract-repo.sh <agent-cockpit|vitals> <destination>}
source_repo=$(cd "$(dirname "$0")/.." && pwd -P)
paths=$source_repo/tools/extract/$name.paths
history=$source_repo/tools/extract/$name.history-paths
overlay=$source_repo/tools/extract/$name

[[ -f $paths && -d $overlay ]] || { echo "extract-repo.sh: no $paths or $overlay" >&2; exit 2; }
[[ ! -e $destination ]] || { echo "extract-repo.sh: $destination exists already" >&2; exit 1; }
command -v git-filter-repo >/dev/null || { echo "extract-repo.sh: git-filter-repo is missing (pacman -S git-filter-repo)" >&2; exit 1; }

git clone -q --no-local "$source_repo" "$destination"
cd "$destination"
git remote remove origin
# The paths to keep, and the names some of them had before (renames keep their history).
{ cat "$paths"; [[ ! -f $history ]] || cat "$history"; } | grep -v '^[[:space:]]*$' > .git/extract.paths
git filter-repo --quiet --force --paths-from-file .git/extract.paths

# An old name can be a live file of dotfiles today (apps/desktop/src/fifo.rs): only the current
# list stays at HEAD.
while IFS= read -r -d '' file; do
    keep=false
    while IFS= read -r path; do
        [[ -z $path ]] && continue
        if [[ $file == "$path" || $file == "${path%/}"/* ]]; then keep=true; break; fi
    done < "$paths"
    "$keep" || git rm -q --cached -- "$file"
done < <(git ls-files -z)
git clean -fdq

cp -rT "$overlay" .
git add -A
git -c core.hooksPath=/dev/null commit -q -m "Make $name a repository of its own

Its files and their history come from dotfiles (tools/extract-repo.sh, todo f4f67e5a);
this adds what it needs alone: the Makefile, the README, CLAUDE.md and the Cargo workspace."
echo "$destination: $(git rev-list --count HEAD) commits, $(git ls-files | wc -l) files"
