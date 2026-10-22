#!/usr/bin/env bash
# Decides whether the playtest bots must play (ticket 0514): reads changed
# file paths, one per line, on stdin and prints `play=true` if any of them
# is a balance lever, else `play=false`. A path that plays is echoed as
# `plays: <path>`.
#
# Balance levers are what can change how a battle plays out: the battles
# and maps, the number tables in assets/data, the rules (crates/core), how
# content becomes a battle (crates/content), the bots and their settings
# (crates/bots), the engine pin, and the playtest command and this
# workflow themselves. Tests, docs, dialogue, art, audio, fonts, text and
# the other data files are not.
set -euo pipefail

play=false
while IFS= read -r path; do
  [ -z "$path" ] && continue
  case "$path" in
    # Tests and docs change no balance.
    */tests/*|*/tests.rs|*/testkit.rs|*.md) continue ;;
    assets/battles/*|assets/maps/*) ;;
    assets/data/ai.ron|assets/data/arts.ron|assets/data/characters.ron) ;;
    assets/data/classes.ron|assets/data/items.ron|assets/data/new_game.ron) ;;
    assets/data/skills.ron|assets/data/spells.ron|assets/data/supports.ron) ;;
    assets/data/terrain.ron) ;;
    crates/core/src/*|crates/content/src/*|crates/bots/*) ;;
    shuyi-engine.rev) ;;
    crates/xtask/src/playtest.rs|.github/workflows/playtest.yml|.github/playtest-levers.sh) ;;
    *) continue ;;
  esac
  echo "plays: $path"
  play=true
done
echo "play=$play"
