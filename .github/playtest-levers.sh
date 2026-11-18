#!/usr/bin/env bash
# Decides whether the playtest bots must play (ticket 0514): reads changed
# file paths, one per line, on stdin and prints `play=true` if any of them
# is a balance lever, else `play=false`. A path that plays is echoed as
# `plays: <path>`.
#
# usage: playtest-levers.sh [BASE HEAD]
#
# Balance levers are what can change how a battle plays out: the battles
# and maps, the number tables in assets/data, the rules (crates/core), how
# content becomes a battle (crates/content), the bots and their settings
# (crates/bots), the engine pin, and the playtest command and this
# workflow themselves. A later chapter is played with the army the earlier
# ones leave (ticket 0520), so the chapters, the New Game file, the basic
# shop and the recorded playthroughs that win the earlier chapters are
# levers too. Tests, docs, dialogue's art and sound, fonts, text and the
# other data files are not.
#
# crates/content's modules that no battle reads (ticket 0173) are not
# levers either: credits, tips, the title, languages, audio, voice, fonts,
# images, palettes, the Playdate's pictures and palette, portraits,
# cinematics and the keymap. Neither crates/core, crates/bots nor the
# playtest command uses their types; the battle modules name them only to
# check references or to draw. dialogue stays a lever: a scene can bring a
# unit in (the bots' army reads `joins`).
#
# Given BASE and HEAD (two revisions), a `.rs` or `.ron` lever whose edit
# touches only comments, blank lines or an in-file test module (a column-0
# `#[cfg(test)]` right before a column-0 `mod <name> {`, through the next
# line that is exactly `}`) does not play: it prints `skips: <path>
# (comments or tests only)`. A file added or deleted plays.
set -euo pipefail

base=${1:-}
head=${2:-}

# The text of stdin without comments, blank lines and in-file test modules.
strip() {
  awk '
    { sub(/\r$/, "") }
    skipping { if ($0 == "}") skipping = 0; next }
    pending {
      pending = 0
      if ($0 ~ /^mod [A-Za-z_][A-Za-z0-9_]* \{$/) { skipping = 1; next }
      print held
    }
    $0 == "#[cfg(test)]" { pending = 1; held = $0; next }
    /^[ \t]*$/ { next }
    /^[ \t]*\/\// { next }
    { print }
    END { if (pending) print held }
  '
}

# Whether `path` changed between BASE and HEAD in comments or tests only.
only_comments_or_tests() {
  local path=$1 before after
  [ -n "$base" ] && [ -n "$head" ] || return 1
  case "$path" in *.rs|*.ron) ;; *) return 1 ;; esac
  before=$(git show "$base:$path" 2>/dev/null) || return 1
  after=$(git show "$head:$path" 2>/dev/null) || return 1
  [ "$(strip <<< "$before")" = "$(strip <<< "$after")" ]
}

play=false
while IFS= read -r path; do
  [ -z "$path" ] && continue
  case "$path" in
    # The recorded playthroughs the bots' army plays (ticket 0520).
    crates/ui/tests/it/fixtures/*_winning.ron) ;;
    # Tests and docs change no balance.
    */tests/*|*/tests.rs|*_tests.rs|*/testkit.rs|*.md) continue ;;
    assets/battles/*|assets/maps/*) ;;
    assets/data/ai.ron|assets/data/arts.ron|assets/data/characters.ron) ;;
    assets/data/classes.ron|assets/data/items.ron|assets/data/new_game.ron) ;;
    assets/data/skills.ron|assets/data/spells.ron|assets/data/supports.ron) ;;
    assets/data/terrain.ron|assets/data/basic_shop.ron|assets/chapters/*) ;;
    # The content modules no battle reads (ticket 0173).
    crates/content/src/credits.rs|crates/content/src/credits/*) continue ;;
    crates/content/src/tip.rs|crates/content/src/tip/*) continue ;;
    crates/content/src/title.rs|crates/content/src/title/*) continue ;;
    crates/content/src/lang.rs|crates/content/src/lang/*) continue ;;
    crates/content/src/audio.rs|crates/content/src/audio/*) continue ;;
    crates/content/src/voice.rs|crates/content/src/voice/*) continue ;;
    crates/content/src/font.rs|crates/content/src/font/*) continue ;;
    crates/content/src/image.rs|crates/content/src/image/*) continue ;;
    crates/content/src/palette.rs|crates/content/src/palette/*) continue ;;
    crates/content/src/playdate_images.rs|crates/content/src/playdate_images/*) continue ;;
    crates/content/src/playdate_palette.rs|crates/content/src/playdate_palette/*) continue ;;
    crates/content/src/portrait.rs|crates/content/src/portrait/*) continue ;;
    crates/content/src/cinematic.rs|crates/content/src/cinematic/*) continue ;;
    crates/content/src/keymap.rs|crates/content/src/keymap/*) continue ;;
    crates/core/src/*|crates/content/src/*|crates/bots/*) ;;
    shuyi-engine.rev) ;;
    crates/tools/src/playtest.rs|.github/workflows/playtest.yml|.github/playtest-levers.sh) ;;
    .github/playtest-inputs.sh) ;;
    *) continue ;;
  esac
  if only_comments_or_tests "$path"; then
    echo "skips: $path (comments or tests only)"
    continue
  fi
  echo "plays: $path"
  play=true
done
echo "play=$play"
