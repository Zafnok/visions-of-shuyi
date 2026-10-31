---
id: "1052"
title: "Playdate: package the builds and sell them on itch.io (one purchase, every language)"
type: infra
milestone: Post–Chapter 1
model: reasoning
effort: medium
status: todo
blocked_by: ["0901", "0062", "1064"]
nick_input: setup
completed:
---

# 1052 — Package the Playdate builds and sell them on itch.io

## Context

Nick: sell the Playdate version on **itch.io first**, beside the Windows
build (players sideload it), then apply to Panic's Catalog (1053). One
purchase gets all the language builds: "On itch.io that is one page with
a download per language" (`docs/design/playdate.md`, *Selling it*, Q6).

0901 publishes the PC releases to itch.io with butler. A Playdate game
also needs launcher pictures (a card and an icon, shown in the device's
game list); 0062 decided how they look. The Playdate SDK licence lets us
sell the game anywhere but forbids "Playdate" or "Panic" in the game's
name without permission (ADR-0070, *Context*).

**After the Act 1 playtest** (re-gated 2026-10-21, 0129): this waited
for Nick to play Chapter 1 on his Playdate (1050), which was folded
into the Act 1 playtest; it now waits for 1064.

## Nick input

**Setup:** Nick decides with the agent whether the Playdate build goes on
the same itch.io page as the PC game (another download on the same
purchase) or a page of its own, and the price if separate (a store
choice, not a design rule; ask with the question pop-up). If the page is
new, Nick creates it on itch.io (exact clicks written in the PR) and adds
its name to the release secrets as 0901 did.

## Scope

**In:**
- **Launcher assets:** `card.png` (350 × 155), `icon.png` (32 × 32) and
  the launch image as 0062 decided (`playdate.md`, *One-bit pictures*:
  the card is the Bayer-dithered title backdrop Fields B at dusk with the
  logo in white; the icon the block S of `SHUYI`,
  `docs/design/playdate/0062-icon.png`; the launch image the card's look
  at 400 × 240; no animation, no wrapping paper), in the `.pdx`'s `launcher/` folder
  (`pdxinfo`'s `imagePath`); made from public art only (or kept in the
  private repository if made from bought art, ADR-0040).
- **`pdxinfo`:** name (no "Playdate" or "Panic"), author, description,
  bundle ID per language (1049), version and build number from the
  release tag.
- **Release workflow** (`.github/workflows/release.yml`): build each
  language's `.pdx` (the CI `playdate` job's steps), zip each
  (`VisionsOfShuyi-playdate-en.pdx.zip`, …), attach to the GitHub
  release, and push each to its own butler channel
  (`playdate-en`, `playdate-ja`, `playdate-zh`) on the page Nick chose.
  Languages whose build doesn't exist yet (1049 not done) are left out;
  1049 adds them (add a line to 1049 if this lands first).
- **Store text:** a Playdate section on the itch.io page: how to sideload
  (Panic's sideload page at `play.date/account/sideload/`), which
  device revisions it was tested on (Rev B), and screenshots of the
  running game (allowed, CLAUDE.md *Bought art*).
- Checks that the `.pdx` zips are what Panic's sideload page accepts
  (upload one to Nick's own account as a test: Nick does this).

**Out (do not do):**
- Catalog (1053).
- Changing the PC release.

**Carve:** none.

## Implementation steps

1. Read 0901's workflow, ADR-0070, 0062's launcher answer.
2. Ask Nick where it sells (pop-up); write the setup steps if needed.
3. Launcher assets, `pdxinfo`, the release jobs.
4. A test release (a pre-release tag) to check the channels; Nick
   sideloads the downloaded zip.

## Acceptance criteria

- [ ] A release builds and attaches the Playdate zips and pushes them to
      their butler channels.
- [ ] The downloaded zip sideloads and runs on Nick's Playdate.
- [ ] The itch.io page has the Playdate downloads and text.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `pdxinfo` text from a tag (xtask).
- Manual: test release; Nick's sideload.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
