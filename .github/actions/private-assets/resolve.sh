#!/usr/bin/env bash
# The effective pin of the private repository (ADR-0135): the base,
# assets-private.rev, merged with each change's own pin file,
# assets-private.d/<branch>.rev, in the order given (the callers sort them
# by file name). Puts the checkout at it, detached, and prints its commit.
#
#   resolve.sh <checkout> <base> [<pin>...]
#
# A pin the result already holds is skipped; one that comes after it is
# taken as it is (no merge, so a single pin of a branch made from the base
# is that commit); any other is merged with a fixed author and date, so
# the same pins give the same commit on every run. A merge conflict fails
# with the files and the fix. A missing commit is fetched from origin (the
# caller provides the credentials).
#
# Used by .github/actions/private-assets/action.yml and by
# `cargo xtask assets-pin --resolve`.
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: resolve.sh <checkout> <base> [<pin>...]" >&2
  exit 2
fi
dir=$1
base=$2
shift 2

g() { git -C "$dir" "$@"; }

have() { g cat-file -e "$1^{commit}" 2> /dev/null; }

for commit in "$base" "$@"; do
  if ! have "$commit"; then
    g fetch -q origin "$commit" 2> /dev/null || true
  fi
  if ! have "$commit"; then
    echo "error: $commit is not in the private repository's history: push its branch there first (the content-change skill)" >&2
    exit 1
  fi
done

if [ -n "$(g status --porcelain --untracked-files=no)" ]; then
  echo "error: $dir/ has changes that aren't committed: commit and push them first" >&2
  exit 1
fi

g -c advice.detachedHead=false switch -q --detach "$base"
for pin in "$@"; do
  if g merge-base --is-ancestor "$pin" HEAD; then
    echo "$pin is already in the result" >&2
  elif g merge-base --is-ancestor HEAD "$pin"; then
    g -c advice.detachedHead=false switch -q --detach "$pin"
    echo "took $pin as it is" >&2
  else
    if ! GIT_AUTHOR_DATE="2000-01-01T00:00:00+0000" \
      GIT_COMMITTER_DATE="2000-01-01T00:00:00+0000" \
      g -c user.name="assets-pin" -c user.email="assets-pin@localhost" \
      -c commit.gpgsign=false \
      merge -q --no-ff --no-edit -m "Merge the pin $pin" "$pin" > /dev/null 2>&1; then
      conflicts=$(g diff --name-only --diff-filter=U | sed 's/^/  /')
      g merge --abort 2> /dev/null || true
      branches=$(g branch -r --contains "$pin" --format='%(refname:short)' 2> /dev/null | grep -v '/main$' | head -3 | tr '\n' ' ')
      cat >&2 << EOF
error: the pin $pin (${branches:-no branch}) conflicts with the base and the pins before it in:
$conflicts
Its private branch must take the private main first, then pin again:
  git -C assets-private fetch origin
  git -C assets-private merge origin/main   # on that branch; resolve, commit
  git -C assets-private push origin HEAD:refs/heads/<branch>
  cargo xtask assets-pin --add
EOF
      exit 1
    fi
    echo "merged $pin" >&2
  fi
done
g rev-parse HEAD
