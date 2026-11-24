# Voices, languages and who writes the script

Decided: 2026-10-03 (the direction; the details are open, see below);
Chinese added 2026-10-04; voices decided 2026-11-01 (0043); Chinese,
picking a language and the machine-translation label decided 2026-11-24
(0045)
Source: Nick, in conversation; tickets 0042, 0043 and 0045 settle the details

## Nick's words

> few high level directives I'm now thinking about. we added CC music, we are
> adding sprites. How about VA? For now I don't have the budget for a VA
> cast. So we can use AI generated with options menu to turn it off, maybe
> even ask upon starting a new game. This is NOT a blocker for ch1 or even
> act 1... but it's something im thinking about. And once we make enough
> money we can hire real VA cast to replace it.
>
> Additionally, I'm thinking, SRPGs seem fairly popular in Japan. So we can
> also add ML translated Japanese option, with human translation later if we
> make enough money as well.

> we should also have the same goals for the script. rn it's fully AI
> generated with minimal input from me bc I don't trust my script writing
> skills. But if we get interest & money I can hire a scriptwriter to make a
> more compelling story. And since our scripts are human readable/writable
> (I hope) then it shouldn't be a huge hassle for them to also add cues for
> i.e. music, portrait transitions, etc.

2026-10-04:

> I know I mentioned Japanese support -- since Japanese love SRPGs
> apparently -- but I guess I totally glossed over that my wife is Chinese
> and I already named the game after her so I may as well add ML translated
> Chinese (Mandarin / Simple) too

2026-10-23, at the end of ticket 0736 (the research on dialogue craft),
on its final sample scenes:

> this is serviceable... still reads incredibly AI written but at least
> there's something here. It has the facade of a soul behind it.
> Eventually, we'll strip all the script for a real human made script but
> we'll ship with this AI script until I make enough money for a rewrite.
> One thing - when writing up the script, I think our dialogue format
> should have cues for voice acting synthesis that do not render in the
> game dialogue viewer (i.e. Wenna talks calmly, Wenna shouts loudly) this
> way it can be mass fed into AI voice synthesis like Suno or Elevenlabs.

The cues are ticket **0752**.

## What this decides

One idea, three times: **ship a machine-made version now, built so that a
paid person can replace it later without rebuilding anything.**

1. **Voices.** Story lines may be spoken by AI-generated voices. The player
   can turn them off in Options. A hired cast replaces the generated clips
   later, line by line.
2. **Japanese and Chinese.** The game gets a Japanese language option and
   a Chinese one (Mandarin, Simplified characters), each translated by
   machine first. A human translation replaces it later.
3. **The script.** Today's script is written by Claude. A hired scriptwriter
   may rewrite it later, working in the same `.dlg` files, cues (music,
   portraits, captions) included.
4. **None of this blocks Chapter 1 or Act 1.**

"AI-generated voices" is an exception Nick is making to his own rule that
shipped art must have had a human work on it (`look-and-feel.md`, ADR-0032
rule 2). It covers voices only; the rule for pictures is unchanged.

## Voices

Decided: 2026-11-01. Source: ticket 0043, by ear, on two listening pages
(`assets-private/story/audio/voice-audition.html`, `assets-src/audio/voice-cast.html`).

### Nick's words

On downloading the free tools, and on money:

> we can download all of these, ensure they are the latest versions of each,
> but also, I think it's ok to use paid services if none of these work well.
> Audio stays free is a preference not a rule

Free first, then decide: "Free first, then decide (Recommended)".

**1. The tool**, after hearing four free tools read the same lines:

> chatterbox is best for hollis, qwen3 seems best for lead, qwen3 seems best
> for harl,

**2. What is voiced:** "Spoken lines and narration". On the tutorial guide:

> also, I think we can voice Vix too.

> and for the button tutorials they don't need voices, but make sure Vix has
> lines throughout tutorial, so a clear delineation between <Vix's line>
> <tutorial objectives> should be made if it's not already entangled...

**3. Battle lines:** "Boss and death scenes only (Recommended)".

**4. The lead:** "Voiced, but not replies". The player's name in a line:
"Voice says another word".

**5. On or off:**

> yes ask at new game. probably this will need engine support at some point,
> like NewGameOptions or some struct etc to build out a menu before new
> game... lots of games have this new game options menu

**6. The label:** "Both, and promise a cast".

**7. Text and voice:** "Same, plus auto-advance".

**8. Other languages:** Japanese, "Japanese voices too"; Chinese, "Chinese
voices too".

**9. The cast:**

> for Hollis A
> for lead man A
> for lead woman B
> for Harl C

> for Tamsin A / Qwen3

> for Rue I don't like any option for her since A and C are too high pitched
> and B too deep

> I don't like any voices for Dace and tbh B is a literal female voice bruh
>
> for Messenger we can go A / Qwen3
>
> for narrator I envision a man, so more man options pls
>
> for Aske C / Chatterbox
>
> for Rue F / Qwen3

> Maud A / Qwen3
> Dace D / Qwen3
>
> narrator E / Qwen3
>
> for Vix I want a more sexy but upbeat voice idk how
>
> basically, A is upbeat and B is sexy but they're both a bit wrong for a
> tutorial char

> im noticing a simple pitch up/pitch down doesn't make much sense it usually
> sounds quite bad. out of these E / chatterbox sounds best but I would like
> one more round of iteration with new voices don't pitch up / down.

> ill go with K

**10. Directions** (0752, by ear, 2026-11-01; "excitedly" should change the
tone and never be read aloud). On four ways to make a directed line:

> A is out of the running bc it too drastically alters the character's voice.
> D is bad too bc the direction doesn't change the delivery whatsoever. Somehow
> the slowly, close to tears delivery is haunted and always corrupts the voices
> which reflects poorly on B even though in general B sounds about on par with C.

No direction was heard read aloud. In the next round he picked **B**, "Low, full
of grief" for sadness, and "Yes, use them".

### The rules

**What is heard**

- Every speech line in a story scene has a voice, the lead's scripted lines
  included, and so does narration. Boss and death scenes are story scenes, so
  they are voiced like any other. Nothing else in a battle speaks: no crit
  shouts.
- The replies a player picks (`*` lines) stay silent text.
- The lead has a man's voice and a woman's voice, by the gender the player
  picked.
- A line that says the player's name shows the name in the text box while the
  voice says another word. So text and voice differ on those lines. Across
  Chapters 1–8 that is 62 of the 271 narration boxes and 4 spoken lines.
- Vix is voiced. Her talk and the objectives are separate: her line is spoken,
  and what names a key or a button is silent text. She has spoken lines all
  through the tutorial (tickets 0475, 0476).

**Starting a game, and saying it is AI**

- New Game asks **"Play with AI-generated voices?"**, with Yes highlighted. The
  answer sets the Options switch. Nick expects a New Game options menu later,
  which may need engine support (0860).
- The Options row reads **"Voices (AI-generated)"**, On or Off.
- The credits say **"Voices generated with AI: Qwen3-TTS, Chatterbox."** and
  **"We hope to replace these with a human cast."**

**Text and voice together**

- Text types out as it does today, with the clip playing over it.
- The first press fills in the box and the voice keeps going. The next press
  goes to the next line and stops the voice.
- An Options setting moves to the next line by itself when the clip ends
  (0859).

**How a line is said**

- A script line can carry a direction ("whispering, frightened"). It changes
  how the voice says the line, and is never read aloud or shown (0752,
  ADR-0109).
- A quiet direction stays quiet: a whisper isn't brought up to the level of
  a shout.
- For sadness the script says "in a low voice, full of grief".

**Other languages**

- Japanese text has Japanese voices and Chinese text has Chinese voices,
  generated with the same tools (0755, 0756). Nobody here can check spoken
  Japanese. Nick's wife checks the Chinese text (*Chinese (Simplified)*,
  below); whether she also listens to the voices was not asked.

**The tool**

- Free tools, chosen per character: **Qwen3-TTS** designs every voice from a
  description, and **Qwen3-TTS** or **Chatterbox** reads the lines, as the cast
  table says. No paid service. Paid services stay allowed if the free tools
  stop being good enough.
- Candidates are newly designed voices near the one Nick liked, never a clip
  pitched up or down.

### The cast

The story part of this decision is in the private assets repository, `story/design/voices-languages-and-script.md`.

### The agent's starting rules

Nick can veto any of these.

- The word the voice says for the player's name is "the exile" for now. A
  later line takes a word that fits the lead's situation at that point (0754).
- The New Game question comes after the familiarity question, in today's
  new-game flow, until 0860 gathers them into a menu.
- A voice speaks the language of the text. There is no separate voice language
  setting.
- Vix's sexy, upbeat direction is how she sounds. Her wording stays by her
  character sheet (`assets-private/story/characters/guide.md`): short, dry, plain.

### What the tools research found

Read on 2026-11-01. The chosen tools' terms are in `THIRD_PARTY_ASSETS.md`.
Quality below is what sources report, unless Nick heard it.

| Tool or group | Result | Why |
| ------------- | ------ | --- |
| Qwen3-TTS, Chatterbox | **Chosen** | Apache 2.0 and MIT weights; run on Nick's machine |
| VoxCPM2, Ming-3.0-tts | Passed; heard, not chosen | Both permissive. Ming's built-in voices copy known characters, so only designed voices were tried |
| MOSS-TTS, Maya1, Zonos, Dia, CosyVoice, GPT-SoVITS, Kokoro, MeloTTS, Kitten TTS, Parler-TTS, Pocket TTS | Passed the licenses; not tried | Weaker fit: no voice design, no direction or no Japanese and Chinese, or nothing reported better than those tried |
| Fish Audio Drama 3, Higgs Audio v2, IndexTTS-2.5, Supertonic 3, Style-Bert-VITS2, NVIDIA Magpie, Orpheus | Dropped | A revenue or user cap, a required notice or credit, voices that may imitate known characters, or terms that couldn't be read |
| Fish S2 Pro, Fish Speech 1.5, F5-TTS, XTTS v2, Higgs Audio v3, VibeVoice, Breeze TTS 2, Voxtral, Piper by default | Failed | Non-commercial or research-only weights or voices |
| ElevenLabs, Gemini 3.8 Flash TTS | Passed; held in reserve | With retakes, about $6 and $1 for Chapter 1, $66–$99 and $14–$27 for the whole game. Not generated, because Nick was satisfied with the free tools |
| MiniMax, Azure, OpenAI, Cartesia, Murf, xAI | Conditional | A required AI mark or violence ban, models closing in January 2027, or rights that don't clearly survive cancelling |
| Inworld, Fish Audio (service), Resemble, Hume, Replica, Play.ht | Failed | Outputs deleted at termination, no ownership clause, no output rights found, or the service is closed or closing |

## Picking a language and saying it is machine-made (every language)

Decided: 2026-11-24. Source: ticket 0045. Asked for Chinese first and
shared with Japanese: 0042 doesn't ask these again.

### Nick's words

On the order of the two tickets:

> why should chinese be blocked by japanese? lets do chinese first.

**Picking a language** (offered: follow the system language with a row
in Options; ask once on first launch; Options only):

> Start with sys lang and also ask on first launch and in options

**Saying it is machine-translated** (offered: a note beside the
language's name where it is picked; a one-time notice; store page and
credits only):

> Lang name and store page and credits

Whether the note also says the Chinese was checked: "Say it's checked
(Recommended)".

### The rules

- **First launch** asks for the language, once, before anything else is
  asked (the keyboard-layout picker comes after it). The computer's
  language is already highlighted when the game has a pack for it
  (Chinese Windows: Chinese); otherwise English is. One press accepts
  it.
- **Options** has a Language row, which changes the language at any time.
- **A machine-translated language says so beside its name**, wherever a
  language is picked (the first-launch list and the Options row):
  - Chinese, which a native reader checks: **简体中文 (机器翻译，母语者审校)**
    ("machine translation, checked by a native reader").
  - Japanese, which nobody checks: **日本語 (機械翻訳)**.
  - A language a person translated has no note.
- **The store pages and the credits** also say which languages are
  machine-translated, and that the Chinese is checked (0907).

### The agent's starting rules

Nick can veto any of these.

- The first-launch list shows each language by its own name (English,
  简体中文, 日本語), whatever language the screen is in.

## Chinese (Simplified)

Decided: 2026-11-24. Source: ticket 0045, on the rendered screens
[`0045-font-compare.png`](../screenshots/0045-font-compare.png),
[`0045-info-fusion12.png`](../screenshots/0045-info-fusion12.png) and
[`0045-info-unifont.png`](../screenshots/0045-info-unifont.png) (the
rest are beside them, `docs/screenshots/0045-*.png`). His wife, who is
Chinese and whom the game is named after, answered some of these with
him.

### Nick's words

**1. How "Shuyi" is written:**

> first name：姝怡（Shuyi
> last name：[the surname is left out of this public repository]

Whether the game may use her exact characters, or a pair that sounds the
same: "Use 姝怡 (Recommended)".

**2. The title in Chinese.** He first picked "Chinese title: 姝怡幻境
(Recommended)" from three machine-made candidates, then:

> My wife changed her mind
>
> 姝怡的魔幻之境 as it’s full name( more attractive and more
> understandable)and 姝怡幻境as it’s nick name

**3. Who checks the Chinese:** "Wife: lists + plays Ch1" (she reads the
title, the names and the glossary of game terms, and also plays
Chapter 1 in Chinese and says what reads wrong). Her credit: "Credit,
first name only".

**4. The font.** Asked to pick from A and B (GNU Unifont 16px, lines
touching or with a blank row between them) and C (Fusion Pixel 12px):

> Wait for a sec

> Her favorite is C with B close behind so prob both should be options
> but are there any other options

Shown D (Fusion Pixel 10px), the only other font that passed, and told
why the rest failed: "C default, B in Options (Recommended)".

**5. Names:** "Chinese characters (Recommended)": every name written in
characters chosen for their sound, as Fire Emblem's Chinese releases do,
and his wife sees the list before it is used.

**6. Naming the lead:** "Type with system keyboard". Until that is
built: "Latin letters for now (Recommended)".

**7. Voices with Chinese text** was answered in 0043: Chinese voices
(0756).

### The rules

**Her name and the title**

- "Shuyi" is written **姝怡**, exactly, everywhere the Chinese text says
  it. Her surname is never used.
- The game's Chinese title is **姝怡的魔幻之境** ("Shuyi's Magical
  Realm"). Its short name is **姝怡幻境**. Both go to 0906's trademark
  check.

**Who checks it, and the credit**

- His wife reads three short lists before Chapter 1 is translated with
  them: the title, the names and the glossary of game terms (0725's
  review page). Chapter 1's translation doesn't start until her changes
  are recorded (0726).
- She plays Chapter 1 in Chinese once it is translated, and says what
  reads wrong. Each thing she finds becomes a ticket.
- Nothing else waits on her, and she is not asked to read every line.
- The credits say **中文校对：姝怡** in Chinese and **Chinese review:
  Shuyi** in English. First name only.

**The font**

- Chinese text starts in **Fusion Pixel 12px** (its Simplified Chinese
  shapes), each character in the 16×16 space of two cells, with the
  space around it that mockup C shows.
- A font row in Options, shown when the language is Chinese, switches to
  **GNU Unifont 16px**, with a blank row between lines of text so they
  don't touch (B). Both fonts ship.
- Latin letters, numbers, `HP` and `Lv` stay in Terminus, as today.

**Names**

- Every character's name is written in Chinese characters chosen for
  their sound (Hollis is 霍利斯 in the mockup), as Fire Emblem's
  official Chinese releases do.
- His wife sees the names list before it is used (above), since a pair
  that sounds right can carry an unlucky meaning.

**Naming the lead**

- A Chinese player types the lead's name in Chinese with the computer's
  own input method (pinyin), as in any Chinese program. That is a large
  job of its own (ticket 0262) and not part of Chapter 1.
- Until it is built, the naming screen works as it does today, in Latin
  letters.

### The agent's starting rules

Nick can veto any of these.

- The full title, 姝怡的魔幻之境, is on the title screen and the store
  page. The short name, 姝怡幻境, is used where the full one doesn't fit
  (a window title, a list of games, an icon's label).
- The lead's default name is shown in its Chinese form, from the names
  list. A name the player typed is shown as typed, Latin letters
  included, inside the Chinese text.
- The font row's wording and its place in Options are proposed by the
  font ticket (0261), through `ask-nick` if they aren't obvious.

### What the font research found

Read at the source on 2026-11-24. Coverage is counted from the font
files against GB 2312: level 1 is the 3,755 common characters, level 2
the 3,008 less common ones. Both characters of 姝怡 are in level 2, and
both chosen fonts have them.

| Font | Size | Licence | Level 1 | Level 2 | Result |
| ---- | ---- | ------- | ------- | ------- | ------ |
| **Fusion Pixel 12px** (`zh_hans`, monospaced), 2026.11.10 | 12×12 | OFL-1.1 | 3,755 | 2,972 | **Chosen** (C), the default |
| **GNU Unifont** 18.0.01 | 16×16 | OFL-1.1, or GPL-2+ with the font exception (we take the OFL) | 3,755 | 3,008 | **Chosen** (B), in Options |
| Fusion Pixel 10px (`zh_hans`), 2026.11.10 | 10×10 | OFL-1.1 | 3,755 | 2,190 | Passed; shown (D), not chosen: complex characters blur |
| Ark Pixel 12px, 2026.11.10 | 12×12 | OFL-1.1 | 3,641 (reported) | — | Dropped for gaps. Fusion Pixel is Ark Pixel with the gaps filled |
| Ark Pixel 16px, 2026.09.01 | 16×16 | OFL-1.1 | 65 (reported) | — | Dropped: the size is retired and never covered Chinese |
| Cubic 11, v1.500 | 12×12 | OFL-1.1 | full (reported) | — | Not offered on its own: Traditional shapes, TTF only; Fusion Pixel uses it to fill gaps |
| Vonwaon Bitmap, 1.02 | 12, 16 | CC0-1.0 | 3,755 (reported) | — | Dropped: its characters trace back to a company's commercial DOS fonts |
| Zpix | 12×12 | Its own terms: US$1,000 for commercial use | — | — | Failed |
| WenQuanYi Bitmap Song | 12–16 | GPL-2+ only | — | — | Failed (its characters reach Unifont under the OFL) |
| IPix | 16 | Its own terms | — | — | Failed: an unreviewed licence |

Licence notes, for 0239:

- **Fusion Pixel** (https://github.com/TakWolf/fusion-pixel-font/releases/tag/2026.11.10):
  the release's `OFL.txt` and `LICENSES/` folder. Its parts are Ark
  Pixel (OFL-1.1), Galmuri (OFL-1.1) and Cubic 11 (OFL-1.1). Cubic 11's
  licence file also carries the notices of the fonts it grew from (M+
  bitmap fonts, JF Dot M+H 12), which grant "Unlimited permission ...
  either commercially or noncommercially". That is permissive and passes
  ADR-0013's review.
- **GNU Unifont** (https://unifoundry.com/LICENSE.txt): the compiled
  fonts are "covered by the SIL Open Font License version 1.1" and by the
  GPL with the font exception. We ship it under the OFL.

## Still open (Nick decides; not yet asked)

Japanese, ticket **0042** (picking a language and the label are decided
above, for every language):

- The Japanese font, picked from rendered screens. Fusion Pixel 12px has
  Japanese shapes too (`ja`), the cheapest way to make the two languages
  look alike.
- How a Japanese player names the lead.
- Whether names are written in katakana.
- The game's title in Japanese.

Until those are answered, nothing about Japanese is a rule a ticket may
build on beyond the four points at the top and the shared rules above.
Traditional Chinese is not asked for.

## How the technical side supports it

- Text in other languages: [ADR-0045](../adr/0045-languages-text-by-key-and-line-ids.md).
- Voice clips: [ADR-0046](../adr/0046-voice-clips-by-line-id.md).
- A human scriptwriter: ticket 0723 (a writer's kit: guide, checker, preview).
- Saying what is machine-made on the credits screen and the store pages:
  ticket 0907.
