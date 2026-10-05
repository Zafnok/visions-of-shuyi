---
id: "1021"
title: "vos-modkit playtest: the bots' balance report for a mod's battle"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1016", "0506"]
nick_input: none
completed:
---

# 1021 — `vos-modkit playtest`: the bots' balance report for modders

## Context

The game's battles are tested by bots that play each one many times as a
Casual, Normal and Hardcore player and report win rates against bands per
map tier (`docs/design/playtest-bots.md`, ADR-0033, `cargo xtask
playtest`, 0505, 0506). Giving modders the same report ("your map, played
100 times") is something no other tactics game offers them.

The bots are dev tooling and are never linked into the game (`CLAUDE.md`,
ADR-0033). 1011's ADR puts them in `vos-modkit`, a separate tool shipped
beside the game (1016).

## Nick input

None.

## Scope

**In:**
- `vos-modkit playtest <mod folder> <battle id> [--tries N]`: layers the
  mod (as `check` does), then runs the same report `cargo xtask playtest`
  prints, with the bands for the battle's tier and every try listed.
- The army the bots bring: the battle's player slots as written; a flag
  for the generated-army rule if 0510 is done (a level N army).
- A short guide section (in 1023's guide; add it there if 1023 is done,
  else note it in 1023).

**Out (do not do):** autobalancing a modder's battle (0509's tool stays a
dev tool unless asked); the trained bot (0507); a GUI.

## Implementation steps

1. Read ADR-0033, `crates/bots/`, `crates/xtask/src/playtest*` and 1016's
   crate.
2. Move the report's shared parts so both `xtask` and `modkit` call one
   function (keep `cargo xtask playtest`'s output unchanged).
3. Add the subcommand; keep the default tries modest (the ADR or 0505's
   default) so it finishes in minutes.

## Acceptance criteria

- [ ] `vos-modkit playtest` on a test mod battle prints the same report
      `cargo xtask playtest` prints for the same battle and seeds (test).
- [ ] `cargo xtask playtest` output is unchanged (its existing tests pass).
- [ ] The game binary does not depend on `trpg-bots` (`cargo tree -p
      trpg-app` shows no `trpg-bots`; add a test or CI step if there isn't
      one).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the subcommand on a test mod.

## Completion notes
