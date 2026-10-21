---
id: "0738"
title: "Conversation places: longer openings, no outline on actors, a lighter study, units left standing at the toll-fort"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0738 — Conversation places: Nick's round 3 notes

## Context

The places behind conversations and their moving openings (0029,
0732–0734; `docs/design/look-and-feel.md`, *Places behind
conversations*). Chapter 1 playtest round 3 (0804, 2026-10-16):

| Asked | Nick |
| ----- | ---- |
| The sparring opening, ~1.5 s | "they can spar for maybe 2s... anyway an issue is that they are yellow outlined? don't think I signed off on that, I want them with no outline." |
| Dace pacing the study, ~1.5 s | "Longer" |
| The study at night is very dark (night light + 40%) | "Darken less in this scene only" |
| After the win, the talk over the empty toll-fort, darkened | "show who's left standing, and fade to the darkened, rather than being darkened immediately, it looks like early morning not noon." |

Kept (Nick, round 3): the yard feels right; the study feels right; battle
lines over the darkened live battle work.

What the code does today (checked 2026-10-16):

- Opening lengths come from their actions' last end
  (`crates/content/src/cinematic/opening.rs:179`): `ch01_sparring.ron`
  1.55 s, `ch01_pacing.ron` 1.45 s. Darkening `DARKEN_MS = 500`
  (`crates/ui/src/cinematic/opening.rs:24`).
- Opening actors are `Faction::Neutral` (`cinematic/opening.rs:64`),
  painted with the neutral outline `#e0c870` (`assets/data/palette.ron:24`)
  by the sprite skin (`crates/ui/src/map_view/sprite/units.rs:33-38`,
  `354`) and the glyph skin (`map_view/scene.rs:591`).
- No per-scene brightness: `@place` takes a map and a tile only
  (`assets/dialogue/ch01.dlg`), `OpeningDef` has no brightness field.
- `ch01_victory` uses the battle map empty of units (0732's starting
  rule), darkened from its first frame.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Play the opening, the victory talk and the study; are the
> lengths, the outline, the lighting and the units right?

## Scope

**In:**
1. **Sparring ~2.0 s** (`assets/openings/ch01_sparring.ron`: one more
   exchange or slower steps, *Claude's choice*, still facing each other).
2. **Dace pacing ~2.0–2.5 s** (`ch01_pacing.ron`: one more turn across
   the rug, *Claude's starting rule*).
3. **No outline on opening actors**, in every map skin (ADR-0038): a
   "no side" look for actors that aren't on a side, or an `outline: None`
   on the cast (say which). Not the neutral yellow.
4. **Per-scene brightness:** an optional brightness on `@place` (e.g.
   `@place study 12 7 bright 0.6`; pick the directive form that fits
   the `.dlg` format, ADR-0011) used by `ch01_tbc` only. *Claude's
   starting value:* 0.6 for the study. Default stays 0.4.
5. **Victory at the toll-fort:**
   - **Who's left standing:** the battle's surviving player units drawn
     where they stood when the battle ended (the enemies are gone, since
     the battle was won). Replaces 0732's "empty of units" rule for
     after-battle scenes.
   - **Fade into the darkening:** the scene starts at full brightness and
     darkens over 0.5 s (the openings' `DARKEN_MS`) before the first box.
   - **Noon, not early morning:** check the lighting the scene uses
     (`assets/data/lighting.ron`; the battle's time of day) and set it to
     the time the beat sheet gives for the victory (`docs/story/chapters/
     ch01.md`); if the beat sheet doesn't say, it's Nick's: ask him.
6. Update `look-and-feel.md` with every changed rule.

**Out (do not do):** the faces' dimming (0739); the chapel place (0741);
the script (0737).

## Implementation steps

1. Retime the two openings.
2. Add the no-outline look for actors; tests read the scene's unit
   entries (their side / outline), not cells.
3. Add the brightness option to `@place`, parse it in content, use it in
   the dialogue view.
4. Victory: pass the surviving units' positions from the battle result
   into the scene's place; add the fade-in-to-dark; fix the lighting.
5. Tests and snapshots.

## Acceptance criteria

- [ ] Openings last ~2.0 s and ~2.0–2.5 s (content test on `length_ms`).
- [ ] Opening actors have no outline in both skins (scene test).
- [ ] `ch01_tbc` is drawn at its own brightness; other scenes at 0.4.
- [ ] `ch01_victory` shows the surviving party where they ended, starts
      bright and darkens over 0.5 s, with the right time of day (tests).
- [ ] `look-and-feel.md` updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/content: `@place` with and without a brightness; opening lengths.
- Integration: the victory scene's place units after a won battle.
- Snapshot: the yard opening, the study, the victory talk.

## Completion notes

*(Filled in by the session that completes the ticket.)*
