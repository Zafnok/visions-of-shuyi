---
id: "0772"
title: The legendary beasts in the story documents
type: content
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0087", "0766"]
nick_input: answer-first
completed:
---

# 0772 — The legendary beasts in the story documents

## Context

The 2026-11-16 hook chain (0085–0088, 0772, 1091–1099, 1275–1281) adds
legendary beasts. They hide at the bottom of dungeon delves, a rival
enemy column hunts them too, and they're fought as a raid with their own
Beast Phase. 0087 decides what they are in the story. Nick set the limit:

> "if our story doesnt revolve around dragons or there's no room for it
> in the arcs or beats I dont wanna totally rewrite it but if there is
> room for something like this, maybe adjusted to something else then I
> think its ok"

A research pass on 2026-11-16 found room in the existing bible: one new
line, no beats moved. Its framings and the canon they must not
contradict are kept in the private repository, because this repository
is public. This ticket writes Nick's answer to 0087's question 1 into
the story documents, so the writer, the content and the later tickets
share one canon.

**Dialogue writing is paused** (Nick, 2026-11-08) until the writer is
hired (0766). That's why this ticket waits for 0766 and works with the
writer:
- **Facts** are recorded as design: what the beasts are, who hunts them,
  where they sleep, and what's off-limits. Agents may write these in
  `story/design/`.
- **Prose** waits for the writer: the bible's wording, rumor lines in
  towns, and any scene. The `story-writing` skill says what is allowed.

## Nick input

**Answer first:** 0087 (what the beasts are in the story, whether the
rival column's hunt has stakes beyond loot, and what happens to a beast
whose act locks behind you). If 0087 left the story part open, ask it
here with the same options before writing.

## Scope

**In** (all in the private repository, through the `content-change`
skill):
- `story/design/beasts.md`, or the file 0087 created: the decided
  facts.
  - What the beasts are.
  - Which faction hunts them, and why, in each act.
  - Roughly where each act's beasts sleep, by region, matched to the
    world map's regions (`docs/design/world-structure.md`).
  - What the canon rules out.
  - What the rewards are called in the world (relics, materials).
- The bible's line that makes beasts possible (one line, per the research
  pass). The writer words it, or an agent adds it plainly marked "to be
  worded by the writer".
- The outline: a note per act on where a beast rumor can first appear
  (for example "Ch6: the first rumor rides on the chapter that
  introduces the relevant magic"). No beat moves, and no beat sheet or
  scene is written.
- A list for the writer: the rumor and lore lines 1279 and 1280 will
  need (count, where shown, length limits). The lines themselves are
  the writer's.
- Moving `assets-private.rev` here in the same PR (`content-change`).

**Out (do not do):**
- Dialogue, `.dlg` scripts, beat sheets or character sheets (paused).
- Any game code or data (1095–1099, 1275–1281).
- Story detail in this public repository. Ticket text and the PR body
  here name files and decisions only, never plot.

## Implementation steps

1. Read 0087's answers (`docs/design/beasts.md` and the private story
   part), then the private bible, outline, beats and cheat sheet, and
   `story/design/`.
2. Write the facts file and the bible line (marked for the writer if the
   writer hasn't worded it), and the outline notes.
3. Check every new fact against the canon and the writer's guide. List
   anything that might clash in the private PR for Nick and the writer.
4. Open the private PR, move the pin here (`cargo shuyi private-assets
   --pin`), and move this ticket to `done/`.

## Acceptance criteria

- [ ] The private repository has the facts file, the bible line and the
      outline notes, merged.
- [ ] `assets-private.rev` points at that commit.
- [ ] No plot detail appears in this repository's diff (`cargo xtask
      check-leaks` passes).
- [ ] The writer's list of lines for 1279 and 1280 is in the private
      PR.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None beyond the gates (`check-leaks`, the content checks after the pin
  moves).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
