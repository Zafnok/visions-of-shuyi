---
id: "1023"
title: "Modding guide and an example mod, checked in CI"
type: infra
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1016"]
nick_input: none
completed:
---

# 1023 — Modding guide and example mod

## Context

The modding tickets (1012–1022) build the machinery; modders need one place
that tells them how to use it, written for someone who has never seen the
repository, and a small working mod to copy. The format references already
exist per folder (`assets/maps/README.md`, `assets/battles/README.md`,
`assets/chapters/README.md`, `assets/dialogue/README.md`, and others); the
guide links them rather than copying them.

## Nick input

None.

## Scope

**In:**
- `docs/modding/README.md`: what a mod is; installing one; `vos-modkit new`
  and `check`; the manifest; replacing files versus adding entries by id
  (1013); a worked example for each common change (a new item, a stronger
  class, a new map and battle); what the rules forbid (0054's policy and
  `LICENSE` section 4, quoted, not paraphrased); never ship copies of the
  game's art.
- Sections for scripting (1017, 1018), the map editor (1019, 1020),
  `playtest` (1021) and campaigns (1022), each written when its ticket is
  done; a ticket that lands after this one adds its own section (add that
  line to each of those tickets' acceptance criteria when you finish this
  one, if they're still open).
- `examples/mods/example_mod/`: a mod that adds one item, changes one
  class's growth and adds one small battle, with a README.
- CI: a step that runs `vos-modkit check examples/mods/example_mod` so the
  example never rots.

**Out (do not do):** a website or wiki; translating the guide; video
tutorials.

## Implementation steps

1. Read 1011's ADR, 0054's answers, `LICENSE` section 4 and every
   `assets/*/README.md`.
2. Write the guide and the example mod; run `vos-modkit check` on it.
3. Add the CI step to the existing build workflow (not a new workflow).

## Acceptance criteria

- [ ] `vos-modkit check examples/mods/example_mod` exits 0 locally and in
      CI.
- [ ] The guide links every format README it relies on, and quotes
      `LICENSE` section 4.
- [ ] The still-open modding tickets each have a criterion to add their
      section to the guide.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the CI check of the example mod.

## Completion notes
