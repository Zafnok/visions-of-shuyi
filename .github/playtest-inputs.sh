#!/usr/bin/env bash
# Prints one sha256 of everything a playtest report can depend on (ticket
# 0173): the toolchain and the lock file, the engine pin, the rules, the
# content and the bots (their recorded wins too, crates/bots/wins/), the
# playtest command, the public and test files, the private content's tree (assets-private/
# must be checked out) and the workflow and its scripts. The same hash
# gives the same reports (the seeds are fixed), so the Playtest workflow
# reuses a report whose inputs hash the same.
#
# It is wider than the levers (playtest-levers.sh) on purpose: a miss only
# costs a play, while a wrong hit would hide a balance change.
set -euo pipefail

# Fails when the private content isn't checked out: no hash without it.
content=$(git -C assets-private rev-parse HEAD:content)
{
  git ls-files -s -- \
    Cargo.toml Cargo.lock rust-toolchain.toml shuyi-engine.rev \
    crates/core crates/content crates/bots \
    crates/tools/Cargo.toml crates/tools/src/playtest.rs \
    assets assets-test crates/ui/tests/it/fixtures \
    .github/workflows/playtest.yml .github/playtest-levers.sh .github/playtest-inputs.sh
  echo "content $content"
} | sha256sum | cut -d' ' -f1
