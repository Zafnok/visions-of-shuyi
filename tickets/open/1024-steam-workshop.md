---
id: "1024"
title: "Steam Workshop: upload a mod from vos-modkit, subscribed mods appear in the game"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: blocked
blocked_by: ["0903", "0054", "1015", "1016"]
nick_input: setup
completed:
---

# 1024 — Steam Workshop

## Context

Steam Workshop (Steamworks UGC) lets players subscribe to mods with one
click; Steam downloads them and tells the game where they are. Whether the
game uses it, and from when, is 0054's answer: if Nick said "never", close
this ticket with a note. It needs the Steamworks integration from 0903 (an
optional `steamworks` cargo feature; the default build never needs Steam).

## Nick input

**Setup:** in the Steamworks partner site, on the game's app: Workshop →
enable it, and set its visibility as 0054 decided. The ticket's session
gives the exact clicks from Valve's current documentation on the day.

## Scope

**In:**
- `vos-modkit publish <mod folder>`: checks the mod (1016), then creates or
  updates its Workshop item (title, description, preview image, tags,
  version note), storing the Workshop id in `mod.ron`.
- In the game (Steam builds only): subscribed items' install folders are
  added to the folders 1014 scans; they show on the Mods screen (1015)
  with a Workshop mark.
- Both behind the `steamworks` feature; every other build is unchanged.

**Out (do not do):** a Workshop browser inside the game (Steam's overlay
does that); paid mods (`LICENSE` forbids them unless Nick changes it).

## Implementation steps

1. Read 0903's work, Valve's Steamworks UGC documentation (re-checked on
   the day) and the `steamworks` crate's licence and API (ADR-0013).
2. Add the scan of subscribed items to `crates/app/src/mods.rs`.
3. Add `publish` to `vos-modkit` behind the feature.
4. Test what can be tested without Steam: the folder merge with fake
   subscribed folders; `publish`'s metadata building.

## Acceptance criteria

- [ ] With fake subscribed folders, the mods appear in the scan with a
      Workshop mark (test).
- [ ] `vos-modkit publish --dry-run` prints the item it would create
      (test).
- [ ] A build without the feature has no Steam code (`cargo tree`).
- [ ] Nick (or Claude with his account's build user) uploads the example
      mod once and subscribes to it from a second account, and it loads.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: metadata building.
- Integration: the scan with fake subscribed folders.

## Completion notes
