#!/usr/bin/env bash
# The verdict of a workflow's one required check (`ci-result`,
# `codeql-result`, `mutants (diff)`; ADR-0014) from its legs' results
# (ticket 0150).
#
# - A failed leg fails it.
# - A cancelled leg fails it, unless a newer commit has replaced this
#   run's: then a newer push cancelled the run (`cancel-in-progress`), the
#   newer run judges the newer commit, and this one passes with a notice
#   instead of a red that was never about the change. A run cancelled by
#   hand, or a leg past its time limit, on the commit still at the head
#   stays red, so a cancelled run never lets a commit merge unchecked.
# - Anything else (success, skipped) passes.
#
# Env: NEEDS (`toJSON(needs)`), EVENT (`github.event_name`), REPO, PR
# (the pull request's number, on `pull_request`), REF_NAME (the branch, on
# `push`), RUN_SHA (the commit this run checks: the pull request's head, or
# `github.sha`), GH_TOKEN.
set -euo pipefail

echo "$NEEDS" | jq -r 'to_entries[] | "\(.key): \(.value.result)"'

if echo "$NEEDS" | jq -e '[.[].result] | any(. == "failure")' > /dev/null; then
  echo "::error::A required job failed."
  exit 1
fi
if ! echo "$NEEDS" | jq -e '[.[].result] | any(. == "cancelled")' > /dev/null; then
  exit 0
fi

# The commit at the head now. Empty when it can't be read: then the run
# counts as not replaced, and the cancelled leg fails it.
head=""
case "${EVENT:-}" in
  pull_request)
    head=$(gh api "repos/$REPO/pulls/$PR" --jq .head.sha 2> /dev/null || true)
    ;;
  push)
    head=$(gh api "repos/$REPO/git/ref/heads/$REF_NAME" --jq .object.sha 2> /dev/null || true)
    ;;
esac

if [ -n "$head" ] && [ "$head" != "$RUN_SHA" ]; then
  echo "::notice::Cancelled because $head replaced $RUN_SHA; that commit's run is the check."
  exit 0
fi
echo "::error::A required job was cancelled on the commit still at the head ($RUN_SHA): re-run it."
exit 1
