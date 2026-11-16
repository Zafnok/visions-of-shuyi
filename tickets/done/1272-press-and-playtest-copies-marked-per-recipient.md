---
id: "1272"
title: "Press and playtest copies are marked for their recipient"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-15
---

# 1272 — Press and playtest copies are marked for their recipient

## Context

The engine pin move of 2026-11-15 passed engine PR #417 (engine ticket
3302): `cargo shuyi export <target> --release --recipient <id> --label
<text> --signing-key <key>` copies a release package into
`dist/recipients/<id>/<target>`, with the recipient's 8-character code
in the program, the pack and one picture named by `shuyi.ron`'s
`export: (overlay_picture: …)`, and records it in a signed ledger;
`export --detect-recipient` finds whose copy a leaked file is. The
`shuyi-ship` skill's step 10 says how, and its limit: a mark is a
deterrent and circumstantial evidence, never proof.

The game will hand out closed builds before its release: press and
publisher copies around the store page (0901, 0903) and playtest builds
to people outside the team.

## Nick input

None for the work. Who gets a copy is Nick's call; the PR tells him
plainly what a mark can and can't show (the skill's limit), and records
nothing as decided.

## Scope

**In:**
- `overlay_picture` in `shuyi.ron`'s `export` part: a picture of the
  game's with at least seven colours (never text), shipped in every
  release package. A bought picture is fine (the package carries it
  already); none is committed.
- `docs/releasing.md`: making a recipient's copy, where the signed
  ledger and the signing key live (the private repository's checkout,
  outside `content/`, never this repository), and checking a leaked
  file with `--detect-recipient`.
- One test copy made and detected locally, its result in the PR.

**Out (do not do):**
- Sending any copy to anyone.
- A key or ledger in this repository.

Estimate: About 80 lines (a `shuyi.ron` line and a page of docs).

## Implementation steps

1. Read the `shuyi-ship` skill's step 10 and the engine's ADR for 3302.
2. Pick the picture, add `overlay_picture`, run an export with
   `--recipient` and detect it.
3. Write the docs; `run-gates`.

## Acceptance criteria

- A local release export with `--recipient test` makes a copy that
  `--detect-recipient` names.
- `docs/releasing.md` says how, and the mark's limit.

## Tests required

None beyond the gates; the test copy's export and detection output is in the PR.

## Completion notes

- `shuyi.ron`'s `export` part names `overlay_picture:
  "portraits/lead_m/sad.png"`, the lead's committed placeholder portrait
  (20 colors that carry the code; in the release's, the demo's and the
  script preview's packs).
- `docs/releasing.md`, "A press or playtest copy": the mark's limit first,
  the one-time setup (the reveal kit, a signing key of its own), making a
  copy, the signed ledger, `--detect-recipient`.
- The test copy, made and detected locally (`--recipient test`, Windows,
  0.3.0, a throwaway key and a throwaway reveal kit): the copy's three
  marks name `9N3P1ES8`, which the signed ledger gave `test`; the base
  package says "no mark found". The output is in the PR. The test key,
  kit, ledger and copy were deleted afterwards.

**Deviations.**
- *The picture is not a bought one.* The ticket allowed one, but the
  bought art is embedded in the program, not the pack (ADR-0120), and the
  mark goes in a picture of the pack: the first try, the bought fields
  battleback's path, got the pack's 4-color placeholder and `SH1002`. Of
  the pack's pictures only the ten lead placeholder portraits have the 7
  colors a code needs; `lead_m/sad.png` has the most. No picture was added.
- *The signing key lives in `~/.ssh/`, not in the private repository's
  checkout.* CI's build key reads the private repository, so a key
  committed there would let any workflow draw codes and sign the ledger.
  The ledger itself does live in the private repository's checkout,
  beside the kit, outside `content/`.
- *The reveal kit is needed first* (`--recipient` refuses without one that
  verifies, and keeps the ledger beside it), and it is ticket 1268, still
  open. The test used a throwaway local kit, not committed. 1268's scope
  now says to set the kit's `out` inside `assets-private/`, so the ledger
  lands there. No real copy can be made before 1268 is done.

**For Nick.** Who gets a copy is yours to decide; nothing here decides it.
What a mark can show: which copy a leaked file came from, as
circumstantial evidence, never proof. Two recipients who compare copies
can strip every mark, and re-saving the pack or the picture loses a mark.
A store key per recipient (Steam, itch.io) is the stronger record.
Nothing changes for a player: the public build has no mark, and the game
draws no recipient line (`shuyi::app::recipient()` would allow one; that
is a look decision, not made).

No follow-up tickets.
