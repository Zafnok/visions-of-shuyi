---
id: "0903"
title: "Steam readiness: steamworks feature flag, depot build script, Deck check"
type: research
milestone: M8 Release
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0816", "0902", "0906", "0908", "1064"]
nick_input: setup
completed:
---

# 0903 — Steam readiness

## Context

Steam is the end goal after Chapter 1 proves fun
([ADR-0009](../../docs/adr/0009-distribution.md)). Steam needs a Steamworks
partner account and a $100 app fee — Nick's decision and money.

**Blockers added 2026-10-01:** 0906 (the title's trademark check must be
done before the Steam page is created) and 0816 (step 3 assumes controller
input, button names and button rebinding all exist; 0816 is the last of
them and waits for 0220 and 0815).

**Blocker added 2026-10-16:** 0908, the polish of the title's sparks and
the threat arrows that Nick wants "before release" (Chapter 1 playtest
round 3).

**Machine-made content (added 2026-10-03, either order with 0907):** if the
build has AI-generated voices, a machine-translated language or both
(`docs/design/voices-languages-and-script.md`), Steam's content survey must
describe them (pre-generated AI content) and the store page's language
boxes must match what ships. If 0907 is done, paste its
`docs/release/store-disclosure.md`; if not, write that paragraph here from
ADR-0045 and ADR-0046 and add a line to 0907.

**After the Act 1 playtest** (re-gated 2026-10-21, 0129): this waited
for the Chapter 1 playtest (0804), which was folded into the Act 1
playtest; it now waits for 1064.

## Nick input

**Setup (when Nick decides to go for Steam):** register at
<https://partner.steamgames.com>, pay the app fee, create the app, and share the
**App ID** and a build account for CI (a dedicated Steam account with 2FA via
the `steamcmd` guard flow — the session will explain exact steps then).
Until then this ticket can do everything with Valve's test App ID 480.

## Implementation steps

1. Add optional Cargo feature `steam` to `trpg-app` using the `steamworks`
   crate: init on startup (fail gracefully → run without Steam), run callbacks
   each frame, show overlay-compatibility check. App ID 480 for development.
   The default build must not require Steam or its SDK. The Steamworks SDK
   redistributable (`steam_api64.dll`) is allowed under Valve's free SDK
   agreement per ADR-0013: record it in `THIRD_PARTY_ASSETS.md` and confirm
   `cargo deny check licenses` passes with the feature enabled.
2. Research and write `docs/steam.md`: Steamworks setup, depot layout, redistributing
   `steam_api64.dll`, `steamcmd` + `app_build.vdf` upload flow, Steam Cloud for
   saves (maps to our storage keys), achievements idea list (don't implement).
3. **Steam Deck:** controller support already exists (0219 input, 0220
   button names, 0816 rebinding; decided in 0032). Document what's still
   needed for Deck Verified and ticket each gap: readable text at
   1280×800; controller prompts matching the Deck (0220's names or Steam
   Input glyphs); an on-screen keyboard where the player types (renaming
   the lead, `docs/design/setting-and-tone.md`, and profile names,
   0837): **Steam's own gamepad keyboard** (`ShowGamepadTextInput` or the
   floating keyboard), replacing our letter grid in the Steam build
   (Nick, 2026-10-14, `docs/design/options.md`; build it here or ticket
   it); and whether to ship a Steam Input configuration that passes the
   pad through as a gamepad (so 0219's gilrs input keeps working) or adopt
   the Steam Input API with action sets (Steam-only; would need an ADR).
4. Write a new ADR only if the integration approach departs from ADR-0009.

## Acceptance criteria

- [ ] `cargo build -p trpg-app --features steam` works in CI on Windows; default build unchanged.
- [ ] With Steam running (App ID 480), the overlay opens in-game (manual check, screenshot).
- [ ] `docs/steam.md` complete; a ticket for each Deck Verified gap found.

## Completion notes

