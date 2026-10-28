---
id: "0844"
title: "Options: a setting that turns the title's intro cinematic off"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-28
---

# 0844 — Options: turn the intro cinematic off

## Context

In ticket 0036 Nick said "Later" to a setting that turns the title's
cinematic off, to be asked again once he had seen the real one
(`docs/design/title-screen.md`, the line "A setting in Options to turn the
cinematic off (yes / no / later): "Later""). Asked in Chapter 1 playtest
round 3 (0804, 2026-10-16) with the choices "No setting needed", "Yes, an
on/off setting" and "Yes, and play it only the first time by default", he
picked **"Yes, an on/off setting"**.

Options belong to the player's profile (0058, 0836). Leaving Options with
a change asks Keep / Discard / Back (0839).

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

> Turn the intro off, wait on the title, and say whether what
> the title does instead feels right.

## Scope

**In:**
- An Options row **"Intro: On / Off"**, default **On**, kept in the
  profile like the other options. *Claude's starting rule:* it sits with
  the other display rows, after Map look / Map zoom; on the title's
  Options and in battle alike (it does nothing in battle, which is fine).
- With it Off, the title never goes idle into the cinematic: after the
  first press it shows the logo, the backdrop, the sparks and the menu
  with the song, and stays (*Claude's starting rule*). The song keeps
  looping.
- Discard on leaving Options brings the old value back (0839's rules).
- The Playdate's Options (1045) get the row too if they have the title
  cinematic (1044); if 1045 is done, add the row there in this ticket.
- Update `docs/design/title-screen.md` and `docs/design/options.md`.

**Out (do not do):**
- "Only the first time" (Nick didn't pick it).
- Any change to the cinematic's content (0843, 0063).

## Implementation steps

1. Add the option to the profile's options (the struct 0836 saves),
   with its default and its text id (`assets/lang/en/ui.ron`; no literal
   screen text, ADR-0045, `cargo xtask check-text`).
2. Add the row to the Options view (`crates/ui/src/screens/options.rs`
   and its view/skin, ADR-0054).
3. In `crates/ui/src/screens/title.rs`, skip the idle-to-cinematic step
   when the option is Off.
4. Tests and snapshot.

## Acceptance criteria

- [ ] The row shows, cycles On/Off, and is kept per profile across a
      restart.
- [ ] With Off, the title stays on the logo and menu after 60 s idle (a
      harness test steps the time).
- [ ] With On, today's behaviour is unchanged (existing tests pass).
- [ ] Discard restores the old value.
- [ ] Design docs updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the option's save/load and default.
- Integration: the title with the option Off and On; Options keep/discard.
- Snapshot: the Options screen with the new row.

## Completion notes

Done as scoped: an `intro` setting (`ShuyiSettings::intro`, default On),
an **Intro** row after Map zoom, and the title (`screens/title.rs`, `play`)
no longer fades out to the cinematic when it is Off. Estimate ~60
production lines (a toggle row like Auto-end, plus one condition).

- Saved form: `intro` is written only when Off (`skip_serializing_if`), so
  every existing save, and the byte-for-byte fixtures from before ticket
  1109, are unchanged; a save without it loads as On.
- Tests: settings round trip, the row's keep/discard, the title with the
  intro Off (stays on the menu through the 15 s idle, the fade and the
  song's loop; On again fades out), and the Options snapshots (one more
  row, so the integration tests' key counts moved by one).
- **Deviation:** the Playdate has no title cinematic yet (1044 is open),
  so the row is in `PC_ONLY` and the Playdate list is unchanged. Ticket
  1044 should take `Row::Intro` out of `PC_ONLY` when it lands.
- *Claude's starting rules* (from the ticket): the row sits after
  Map zoom, shows in battle's Options too, and Off keeps the song looping.
  Restore defaults puts it back On.
- Left for the Act 1 playtest (1064): turn the intro off, wait on the
  title, and say whether what the title does instead feels right.
