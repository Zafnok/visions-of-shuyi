---
id: "0526"
title: The Casual bot wins the Envoy's House too often, and no bot holds its door
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0851"]
nick_input: none
completed:
---

# 0526 — The Casual bot wins the Envoy's House too often, and no bot holds its door

## Context

Chapter 5's story battle (the Envoy's House, 0851) is the first hold: a
Survive of 6 turns in a narrow house with one street door onto the quay
and a roof hatch. Nick gave holds their own Casual band, 40–70% on a
Normal-tier map, and asked for the hold to be made harder to meet it
(`docs/design/playtest-bots.md`, *Holds have their own Casual band*).
0851 balanced Normal and Hardcore into their bands on seeds 1–100 and
101–200 (Normal 65 and 70, Hardcore 62 and 57) but Casual stays at 86
and 80: too easy (`docs/playtesting.md`, Chapter 5).

What 0851 saw, without tracing it:

- The Casual bot guards only the lead, who waits the hold out indoors,
  so it loses only when the lead falls. More Hounds, more turns, a
  second opening (unbarred shutters), archers on the quay: each one got
  companions killed (Normal down to 7–42) and left Casual at 78–91.
- The only thing that made the lead fall was the officer reaching the
  street door on turn 1 (starting on the near pier): at Str 12 and Spd
  11 the lead fell on turn 1 in 94–97 tries of each bot, at Str 11 and
  Spd 10 in about 11–15. **No bot stood a unit on the door tile (13, 8)
  to stop him**, though a unit there takes him alone. Hardcore, with the
  biggest thinking budget, didn't either. So the officer's strength
  moves all three bots together and can't tell a careless player from a
  careful one, which is what the Casual band is for.

A bot that held a chokepoint against a threat it can see in reach
(0523's kind of fault: the moves worth making missing from what it
tries) would let the officer be dangerous to a careless lead only.

## Nick input

None (a bots fault; the band is Nick's).

## Scope

**In:**
- Trace the three bots on the Envoy's House with the officer in turn-1
  reach (`assets/battles/ch05.ron`, the officer at (13, 13); `cargo
  xtask playtest ch05`, a few seeds command by command, as 0523 did) and
  find why none blocks the door.
- Fix it in the bots (`crates/bots`), with a test that fails without the
  fix, if the cause is a bot's; or say why it can't be.
- Then retune the Envoy's House so Casual lands in 40–70% with Normal
  and Hardcore still in their bands, both seed ranges; record a new
  winning playthrough if it changes (`ch05_winnable`).
- Chapters 1 to 4 still in their bands after it.

**Out (do not do):**
- Changing the bands (Nick's).

## Acceptance criteria

- [ ] `cargo xtask playtest ch05 --bot all` meets every band on seeds
      1–100 and 101–200; Chapters 1 to 4 likewise.
- [ ] A test of the fix fails without it.
- [ ] All gates in the `run-gates` skill pass.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
