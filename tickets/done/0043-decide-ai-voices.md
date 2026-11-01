---
id: "0043"
title: "Decide: AI voices — what is voiced, each voice by ear, on or off, the tool"
type: design-decision
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-01
---

# 0043 — Decide: AI voices — what is voiced, each voice by ear, on or off, the tool

## Context

Nick, 2026-10-03 (`docs/design/voices-languages-and-script.md`): voice
acting by AI-generated voices for now, "with options menu to turn it off,
maybe even ask upon starting a new game", replaced by a real cast once the
game earns enough. Not a blocker for Chapter 1 or Act 1. The technical
plan is [ADR-0046](../../docs/adr/0046-voice-clips-by-line-id.md). This
ticket settles everything a player hears or sees, the way 0020 settled
music: by ear, over several rounds.

## Nick input

**Decision.** `ask-nick` skill. Steps 1–2 are the agent's research; Nick
answers from step 3. He may need to do a **setup** step if a tool needs an
account or an install on his machine: give exact clicks, and never ask him
for a key in chat.

## Scope

**In:** the tool research and its licence check; a listening page; Nick's
answers recorded in `docs/design/voices-languages-and-script.md` (a
*Voices* section) and `docs/design/README.md`; the cast list (character →
voice) as a table in that doc.

**Out (do not do):** game code; the generation tool (0721); generating a
chapter (0722). No sample that clones or imitates a real person, actor or
existing character. No commissioning anyone (Nick's rule).

## Implementation steps

1. **Research the tools.** For each candidate, read its terms on the day
   and note: who owns the output, commercial use, royalties, price, whether
   it runs on Nick's machine (Windows 11, no MSVC; say what hardware it
   needs) or as a service, Japanese support, and whether a designed voice
   stays the same across hundreds of lines. Cover both kinds:
   - free models that run locally under a licence allowing commercial use
     of the output (check the *weights'* licence, not just the code's;
     many are non-commercial);
   - paid services (monthly price for one chapter's worth and for a whole
     game's worth of lines).
   ADR-0046 rule 8 is the bar. Drop anything that fails and say why.
2. **Build a listening page** (an Artifact, like 0020's): for each tool
   that passed, the same four Chapter 1 lines (`assets/dialogue/ch01.dlg`)
   for three characters with different voices in their sheets
   (`docs/story/characters/*.md`, *Voice notes*): Hollis (old, dry), the
   lead's short lines in both genders, Harl (the boss). Keep the page's
   source in `assets-src/audio/`.
3. **Ask Nick**, one at a time:
   1. **Which tool**, by ear, with its price beside it. Paying is his call.
   2. **What is voiced.** Options: every story line (Triangle Strategy,
      Unicorn Overlord); only the big scenes, the rest silent text; a
      short reaction per text box, a grunt or a word (Fire Emblem
      Awakening); narration read or not. Give an honest size for each (how
      many lines in Chapter 1).
   3. **Battle lines.** None yet; or boss, crit and death lines only (the
      scripts already have those scenes).
   4. **The lead.** Unvoiced (Byleth in Three Houses); voiced in both
      genders, with lines that say the player's name left silent or
      reworded.
   5. **On or off at the start, and asking.** Options: a question when a
      new game starts ("Play with AI-generated voices?"); on, with the
      switch in Options; off, with the switch in Options. Note that Steam
      shows an AI notice on the store page either way.
   6. **How it is labelled** in the game: the Options row's wording, a
      line in the credits.
   7. **Text and voice together.** Options: text types out as now and the
      clip plays over it; advancing cuts the clip (most games); an
      auto-advance mode when the clip ends (a later ticket if wanted).
   8. **Japanese and Chinese** (0045 added Chinese; ask per language).
      English voices under Japanese text; Japanese voices
      too (double the work, and nobody here can check them); no voices in
      Japanese.
   9. **The cast.** Per Chapter 1 speaking character, two or three voice
      candidates from the chosen tool reading that character's sample
      lines; Nick picks. Later chapters' characters get their own round
      when they are written.
4. Record answers verbatim, then as rules. Write the tool's row in
   `THIRD_PARTY_ASSETS.md` (terms quoted, date read).
5. Update 0720, 0721, 0722 and 0826 where an answer changes them, and
   write tickets for anything new (auto-advance, battle voice lines).

## Acceptance criteria

- [x] Every question has Nick's answer, verbatim, in the design doc.
- [x] The chosen tool's terms are quoted with date and URL and meet ADR-0046 rule 8.
- [x] A cast table covers every Chapter 1 speaker the answers say is voiced.
- [x] The listening page's source is in `assets-src/audio/`.
- [x] Dependent tickets are updated; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

Result: `docs/design/voices-languages-and-script.md`, *Voices*, with Nick's words,
the rules, the cast table and the research; ADR-0107 for how the tools run. Free
tools, per character: Qwen3-TTS designs every voice, and Qwen3-TTS or Chatterbox
reads it. Nothing to pay. The 12 chosen voices are in
`assets-src/audio/voice-refs/`.

Listening pages: https://claude.ai/artifact/USzpopFocrPyqido9Ny5xC (the tools) and
https://claude.ai/artifact/WQspqiQaEt5Wy4UUCdMCNn (the cast). Their sources are in
`assets-src/audio/`. The clips and bought portraits are only in the published
pages. The casting page's source shows only the last round, Vix's.

Deviations from the plan:

- The tools' terms are under *AI voice tools* in `THIRD_PARTY_ASSETS.md`, not in its
  table: a table row needs a shown credit, and nothing they make ships until 0722.
- Questions went in batches of up to three, not one at a time (the `ask-nick`
  limit, since Nick's turns are expensive).
- Nick answered question 1 per character, so there are two reading tools.
- Nick added Vix to the voiced cast, and said button tutorials need no voices.
  That made tickets 0475 and 0476.
- Paid services were not sampled: Nick chose to hear the free tools first and was
  satisfied. They are in reserve at the design doc's prices.
- Mistakes corrected along the way: a question said 54 of 281 narration boxes and
  17 lead lines; the real counts are 62 of 271 and 16. A woman's voice reached
  Dace's first round, so pitch is now measured for men's parts. Rue's and Vix's
  pitch-shifted candidates sounded bad and are ruled out; no chosen voice is one.

Not verified: the script check ran on rounds 1 and 2 only, not on Rue's or Vix's
later voices. Chapter 1's clips get it in 0722. No agent has heard any clip; Nick's
ear is the check.

Follow-up tickets:

- **0077** decide the voices of the characters after Chapter 1
- **0475** Vix's lines come apart from the objectives, with no key in what she says
- **0476** Vix speaks her lessons and tips
- **0754** the voice says another word for the player's name
- **0755**, **0756** Japanese and Chinese voices for Chapter 1
- **0757** generate Chapters 2 to 8's voices
- **0859** auto-advance when the voice ends
- **0860** a New Game options menu

Updated: 0042, 0045, 0720, 0721, 0722, 0752, 0826, 0907, `docs/ROADMAP.md`,
`docs/design/tutorial.md`, Vix's character sheet.

**The agent's starting rules** (Nick can veto any):

- The voice says "the exile" for the player's name for now, and a fitting word
  later.
- The New Game question comes after the familiarity question, until 0860.
- A voice speaks the language of the text, with no separate setting.
- Vix's sexy, upbeat direction is how she sounds, and her wording stays short and dry.
- Auto-advance starts Off (0859). A player with no answer to the New Game question
  yet hears voices (0826).
