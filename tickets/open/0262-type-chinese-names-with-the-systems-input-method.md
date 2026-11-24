---
id: "0262"
title: "Type a name in Chinese with the system's input method"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0068", "0239"]
nick_input: none
completed:
---

# 0262 — Type a name in Chinese with the system's input method

## Context

Nick, 0045 (2026-11-24, `docs/design/voices-languages-and-script.md`,
*Chinese (Simplified)*, *Naming the lead*): a Chinese player types the
lead's name in Chinese with the computer's own input method (pinyin),
"Type with system keyboard". He was told it is a large job of its own
and not part of Chapter 1; until it lands the naming screen stays in
Latin letters ("Latin letters for now").

Typing Chinese goes through the system's input method editor (IME): the
player types pinyin, the IME shows candidates in its own window, and the
chosen characters arrive as committed text. The game today takes typed
characters one by one (`crates/ui/src/screens/lead_select.rs`, its
`max_chars`), with no composition and no IME window placement, on
Windows, the web build, Linux, macOS and the Steam Deck (whose on-screen
keyboard has its own path through Steam). The name box is the engine's
text input, so most of this is likely engine work.

0065 (open) may change the naming screen to suggested names plus free
text, for the lead and Vix. Whatever it decides, the free-text box this
ticket teaches Chinese is the same one.

## Nick input

None: 0045 decided it. A choice a player would see that the design doc
doesn't cover (how the composing text looks, how long a Chinese name may
be) is asked through `ask-nick` before building.

## Scope

**In:** composed text from the system IME into the game's name box on
Windows and the web build first, the composing text shown in the box,
the IME's candidate window placed near the box; Chinese names counted in
cells for the length limit; the other platforms listed with what they
need, as tickets where they need more.

**Out (do not do):** typing Japanese (0042 decides how a Japanese player
names the lead); any text box other than names; translating; the
Playdate (it has no keyboard).

## Implementation steps

1. Find how the engine's window layer (macroquad / miniquad, its version
   at the pin) and the web build see IME input today: composition
   events, committed text, the IME window's position. Write down per
   platform what arrives when a player types 你好 with Microsoft Pinyin
   (Windows) and in Chrome (web).
2. Follow the `engine-change` skill for what the engine lacks: an engine
   ticket for IME text in its text input (committed text, the composing
   string, a caret rectangle for the candidate window), then this
   ticket's game half. Use the engine's `engine-friction` estimate
   rules.
3. The name box draws the composing string (e.g. underlined) until it is
   committed; committed Chinese characters are measured with
   `text_width` (two cells each) against the name's limit in cells
   (0068's answer on cutting a long name).
4. Every place that shows the lead's name draws Chinese characters
   (0239's atlas already has every character of GB 2312 level 1;
   a name using a character outside the atlas: decide the fallback,
   following 0068).
5. Steam Deck: list what Steam's on-screen keyboard gives in Chinese,
   and file a ticket if it needs more.

## Acceptance criteria

- [ ] On Windows with Microsoft Pinyin, typing `nihao` and picking 你好 names the lead 你好 (a manual check written up in the PR with a screenshot, plus a Harness test feeding committed text).
- [ ] The same in the web build in Chrome.
- [ ] A Chinese name longer than the limit in cells is refused or cut as 0068 says (test).
- [ ] The lead's Chinese name shows in a Chinese dialogue line and on unit info (Harness test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: name length in cells with wide characters.
- Snapshot / integration: the name box with composing and committed text (Harness, with IME events fed in).

## Completion notes

