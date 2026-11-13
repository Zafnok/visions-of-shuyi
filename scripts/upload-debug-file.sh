#!/usr/bin/env bash
# Puts a release's debug file on a release of the private assets repository
# (ticket 0917, docs/releasing.md "The debug files"): the release tagged TAG
# there, made when it isn't there yet, gets FILE as the asset NAME. An asset
# of that name already there (a rerun) is replaced.
#
#   upload-debug-file.sh <file> <tag> <name>
#
# DEBUG_FILES_TOKEN is a token that can write the private repository's
# contents; DEBUG_FILES_REPO names it (`<owner>/<repo>`). Needs curl and jq.
set -euo pipefail

file=$1 tag=$2 name=$3
repo=${DEBUG_FILES_REPO:?}
if [ -z "${DEBUG_FILES_TOKEN:-}" ]; then
  echo "::error::No DEBUG_FILES_TOKEN secret: the debug file can't be kept (docs/releasing.md, \"The debug files\")."
  exit 1
fi
test -s "$file"

api=https://api.github.com/repos/$repo
call() {
  curl --proto '=https' --tlsv1.2 -sS -H "Authorization: Bearer $DEBUG_FILES_TOKEN" \
    -H 'Accept: application/vnd.github+json' -H 'X-GitHub-Api-Version: 2022-11-28' "$@"
}

# The release, or a new one. The Windows and Linux jobs may make it at the
# same moment: the loser's create fails, and its next lookup finds it.
id=
for _ in 1 2 3; do
  id=$(call "$api/releases/tags/$tag" | jq -r '.id // empty')
  [ -n "$id" ] && break
  body=$(jq -n --arg tag "$tag" --arg run "${RUN_URL:-}" \
    '{tag_name: $tag, name: $tag, body: ("Debug files of this version, kept private (ticket 0917). Made by " + $run)}')
  id=$(call -X POST "$api/releases" -d "$body" | jq -r '.id // empty')
  [ -n "$id" ] && break
  sleep 5
done
[ -n "$id" ] || { echo "::error::No release $tag in $repo, and none could be made."; exit 1; }

old=$(call "$api/releases/$id/assets?per_page=100" | jq -r --arg n "$name" '.[] | select(.name == $n) | .id')
if [ -n "$old" ]; then
  call -X DELETE "$api/releases/assets/$old" -o /dev/null -w '%{http_code}\n' | grep -qx 204
fi

size=$(wc -c < "$file" | tr -d ' ')
echo "Uploading $name ($size bytes) to $repo's release $tag"
got=$(call -X POST -H 'Content-Type: application/octet-stream' --data-binary "@$file" \
  "https://uploads.github.com/repos/$repo/releases/$id/assets?name=$name" | jq -r '.size // empty')
if [ "$got" != "$size" ]; then
  echo "::error::The upload of $name didn't arrive whole ($got of $size bytes)."
  exit 1
fi
echo "Kept $name in $repo, release $tag"
