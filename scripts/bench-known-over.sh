#!/usr/bin/env bash
# What CI's `bench` job makes of `cargo shuyi bench`'s exit code (ticket
# 1244): the numbers over budget its first run found, KNOWN_OVER (one
# `target/what/scenario`, or `target/package`, per word), are warnings until tickets
# 1247, 1248 and 1249 bring them under and 1250 removes this script. Any
# other number over its budget is an error, and so is a failed run with
# an error of its own (`error[SH…]` in its log) or no report.
#
# Usage: bench-known-over.sh <bench's exit code> <its log> <bench-report.json>
set -euo pipefail

code=$1
log=$2
report=$3
[ "$code" -eq 0 ] && exit 0

py=$(command -v python3 || command -v python)
over=""
if [ -f "$report" ]; then
  # One line per finding: its key, a tab, what it says.
  over=$("$py" -c '
import json, sys
for o in json.load(open(sys.argv[1]))["over"]:
    name = o["scenario"] or o["package"]
    frame = " (frame {})".format(o["frame"]) if o["frame"] is not None else ""
    # A target has one package, whatever its version: keyed without its name.
    key = "/".join([o["target"], o["what"]] + ([o["scenario"]] if o["scenario"] else []))
    print("{}\t{} on {}: {} {:,} over the budget of {:,}{}".format(
        key, name, o["target"], o["what"], o["value"], o["budget"], frame))
' "$report" | tr -d '\r')
fi
if grep -q 'error\[SH' "$log" || [ -z "$over" ]; then
  echo "::error::bench failed (exit $code) with an error, not only numbers over budget"
  exit "$code"
fi
new=0
while IFS=$'\t' read -r key says; do
  if printf '%s\n' ${KNOWN_OVER:-} | grep -qxF -- "$key"; then
    echo "::warning::$says (known: ticket 1248)"
  else
    echo "::error::$says"
    new=1
  fi
done <<< "$over"
exit "$new"
