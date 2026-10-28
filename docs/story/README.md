# Story

How the story gets written:
[ADR-0011](../adr/0011-story-authoring-pipeline.md) and the `story-writing`
skill. Short version: Nick gives beats → Claude builds a bible and a cast with
real arcs → outline → per-chapter beat sheets → scripts in `assets/dialogue/`
→ a separate critique pass. Nick approves at two gates: cast summary and outline.

| File | What | Created by | Status |
| ---- | ---- | ---------- | ------ |
| [`beats.md`](beats.md) | Nick's beats, verbatim. **Canon.** | 0007 | ✅ 2026-09-25 |
| [`bible.md`](bible.md) | World, factions, themes, tone, magic rules, glossary | 0701 | ✅ gate 1 approved 2026-09-28 |
| [`names.md`](names.md) | Name registry: a stable id for every proper noun, so names can be renamed | 0701 | ✅ 2026-09-28 |
| [`characters/<id>.md`](characters/) | One sheet per character: want/need/flaw/arc, voice, portrait brief, supports | 0701, 0743 | ✅ Chapter 1's cast, 2026-09-28 (0701) · Act 1's new cast (Joss, Gil, Hedda, Ragna, Oriel) and named bosses, 2026-10-28 (0743) |
| [`outline.md`](outline.md) | Acts and chapters; which arcs each chapter advances; the twists | 0701 | ✅ gate 2 approved 2026-09-28 |
| [`chapters/chNN.md`](chapters/) | Scene-by-scene beat sheet per chapter | 0701 (ch01), 0744 (ch02), later tickets | ✅ ch01 · ch02 (0744) · later chapters ⏳ |
| [`ledger.md`](ledger.md) | Continuity: what each character knows/did as of each chapter | 0701, updated by each script ticket | ✅ start of Chapter 1 · after Chapter 1 (0707, 0737) · after Chapter 2 (0744) |
| [`craft.md`](craft.md) | Research on what makes fantasy game dialogue compelling: sources, what went wrong in Chapter 1, what each studied game does, scene lengths. The rules are the `story-writing` skill's *Craft* | 0736 | ✅ 2026-10-23 |

Scripts themselves live in `assets/dialogue/*.dlg` (format: ticket 0702).
Written so far: Chapter 1, `ch01.dlg` (0707; played straight by 0724, reworked scene by scene by 0728, written again from the outline by 0737); Chapter 2, `ch02.dlg` (0744).

## For a human writer

If Nick hires a scriptwriter (`docs/design/voices-languages-and-script.md`),
this is what to read, in order:

1. [`writers-guide.md`](writers-guide.md): how scripts are written, checked
   (`cargo xtask check-script`) and watched (`--scene <id>`), what is fixed,
   and what a rewrite costs. Start here.
2. [`beats.md`](beats.md): Nick's beats. Canon.
3. [`bible.md`](bible.md): the world, its factions, themes and tone.
4. [`characters/`](characters/): one sheet per character; the voice notes
   are what dialogue is written from.
5. [`outline.md`](outline.md): the acts and chapters, and the twists.
6. [`chapters/chNN.md`](chapters/): the beat sheet of the chapter being
   written.
7. [`ledger.md`](ledger.md): what each character knows and has done as of
   each chapter.
8. [`names.md`](names.md): the id of every name, to look up while writing.
9. [`craft.md`](craft.md): why the first Chapter 1 scripts were rejected,
   and what the studied games do instead.

**The ledger must be updated after a rewrite** that changes who knows or
did what: add to (or correct) the section for that chapter, so the next
chapter isn't written from a wrong picture.
