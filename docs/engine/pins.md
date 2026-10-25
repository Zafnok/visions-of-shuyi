# Engine pins

The ledger of the game's engine pin ([ADR-0096](../adr/0096-a-watcher-on-the-engines-main-moves-the-pin-when-its-public-api-changes.md)):
one section per pin move, newest last, whether the engine watcher filed it
(an `engine-pin` issue) or a ticket made it (`engine-change` §3.6 and §7).
The `engine-pin` skill writes each section.

A section is headed `## YYYY-MM-DD: <from 8 hex> → <to 8 hex> (N engine
PRs), #<game PR>`. Its table has a row for every *Breaking*, *Unblocks* and
*New for games* engine PR the move passed, each with a decision:

- *Breaking:* `fixed: <what the game changed>`;
- *Unblocks:* `unblocks NNNN`;
- *New for games:* `adopt → NNNN` (a game ticket that moves the game onto
  the engine's piece, or an open ticket changed to use it; `adopt → NNNN
  (after engine#MMMM)` when the engine must change first), or `not for the
  game: <why>`.

A line under the table counts the rest: `Nothing for the game: N PRs (#…,
#…)`.

`cargo xtask engine-pin --check` (in CI) fails when the last section isn't
the pin in `shuyi-engine.rev`, when a row has no decision, or when an
`adopt → NNNN` names no ticket in `tickets/open/` or `tickets/done/`.

## 2026-10-25: baseline → 3f2256d4

The pins before this one were moved by tickets (1177 and 1188, both
2026-10-22) without this review. The engine watcher's first report starts
from here.
