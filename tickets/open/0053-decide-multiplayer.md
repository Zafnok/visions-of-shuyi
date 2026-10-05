---
id: "0053"
title: "Decide: multiplayer — which modes, how armies are set up, fairness, servers"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: high
status: blocked
blocked_by: ["0903"]
nick_input: decision
completed:
---

# 0053 — Decide: multiplayer

## Context

Nick, 2026-10-04 and 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Multiplayer*): "my idea was somehow having multiplayer but it might be
hard to make it work in a good way", and then:

> "for multiplayer we'll have a 00 ticket for designing through these
> options and their tradeoffs etc. before implementation details. and this
> will all be much after single-player launch anyway."

**Blocked until the single-player game has launched** (0903, Steam
readiness, is the last release ticket today; if a later launch ticket is
written, move this ticket's `blocked_by` to it).

His first reactions to the five options Claude listed:

| Option | What it is | Nick |
| ------ | ---------- | ---- |
| 1. Hotseat | Two players on one machine, Advance Wars style | "sure, but need to think about how we set up opposing armies (does it use save files, do we just let ppl pick units, is there an army budget, who goes first, how are unit placements determined)" |
| 2. Play by link | Each turn is a short code or link pasted into a chat; async; no servers | "maybe" |
| 3. Ghost armies | Fight someone's army run by the AI (Super Auto Pets, Hearthstone Battlegrounds) | "decent, need army budgets though or tiers like pokemon showdown to balance" |
| 4. Co-op skirmishes | Two players split one army's units | "maybe... but it might just be a nuisance... though I guess it might be ok." |
| 5. Live online PvP | Matchmaking, real time turns | "I do want this but I agree..." |

What makes it cheaper for us than for most games: the core is
deterministic (ADR-0004, ADR-0019), so a match is its start plus a list of
commands. Two machines only send each other moves (lockstep), never game
state, and a replay is free.

What Steam gives (Nick: "doesn't steam provide servers somehow"), as told
to him on 2026-10-05; **re-check against Valve's Steamworks documentation
on the day**:

- **Gives, free:** lobbies and matchmaking; connections between players
  relayed through Valve's network (Steam Datagram Relay), which hides IP
  addresses and gets through home routers; leaderboards that can carry a
  file; Steam Cloud; Workshop. Remote Play Together streams a local
  (hotseat) game to a friend online with no work from us.
- **Doesn't give:** a server that runs our game. Lockstep doesn't need one.
  It is needed for **ranked ratings** (Steam leaderboards trust the player's
  game, so they can be cheated).
- **Other costs:** a modded game could compute the dice before a move is
  committed, so each roll needs a secret part from both sides that is only
  revealed after the move is locked in (technical; Claude's job). The web
  build can't use Steam, so web ↔ Steam play needs a relay of our own.

## Nick input

**Decision**, after a written comparison of the options. Run with the
`ask-nick` skill, one question at a time, each with options from real games
and "describe your own".

## Scope

**In:**
- A written comparison of the five options: what each costs to build and
  run (networking, servers, anti-cheat, balance, content), what it needs
  from the single-player game, and what players get. Re-check the Steam
  facts above.
- The questions below; the answers in `replayability.md` (*Multiplayer*)
  or a new `docs/design/multiplayer.md` if it grows; the
  `docs/design/README.md` row.
- The implementation tickets for the first mode Nick picks.

**Out (do not do):** any game code or networking; an ADR for the network
design (the first implementation ticket writes it); anything before the
launch.

## Implementation steps

1. Write the comparison (step "In" above), with a rough size for each
   option (number of tickets, any monthly server cost).
2. **Ask Nick**, in this order:
   1. **Which modes, in which order.** From the comparison.
   2. **Where an army comes from.** Options: fixed armies per map (Advance
      Wars); a team imported from a campaign save (Fire Emblem Fates'
      castle battles, Awakening's StreetPass teams); a draft from a shared
      pool; built freely from any unit.
   3. **Keeping it fair.** Options: a point budget (Warhammer); tiers of
      units (Pokémon Showdown, Nick's example); every unit scaled to one
      level (Pokémon's Level 50 rule); mirrored armies.
   4. **Who goes first.** Options: alternate or random; the second player
      gets something back (an extra unit, more points, or the first player
      can't attack on turn 1).
   5. **Where units start.** Options: deploy zones the map sets; both
      players place their units in secret, then both are shown at once;
      fixed starting tiles.
   6. **Maps.** Symmetric maps made for PvP, or campaign maps; and whether
      the bots balance them by playing both sides until each wins about
      half the time (as 0509 does for single-player).
   7. **Ranked or not**, and whether to pay for a server to make ranks
      trustworthy.
   8. **The web build:** included (needs our own relay) or Steam only.
3. Record each answer verbatim and as a rule.
4. Write the implementation tickets for the first mode (`write-ticket`); the
   first of them writes the networking ADR.

## Acceptance criteria

- [ ] The comparison exists, covers all five options, and its Steam facts
      are re-checked with a source link.
- [ ] Every question above has Nick's answer, verbatim, in the design doc.
- [ ] The first mode's implementation tickets exist; `cargo xtask
      ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
