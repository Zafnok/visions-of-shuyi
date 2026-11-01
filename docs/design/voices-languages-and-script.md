# Voices, languages and who writes the script

Decided: 2026-10-03 (the direction; the details are open, see below);
Chinese added 2026-10-04
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
(`assets-src/audio/voice-audition.html`, `assets-src/audio/voice-cast.html`).

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

**Other languages**

- Japanese text has Japanese voices and Chinese text has Chinese voices,
  generated with the same tools (0755, 0756). Nobody here can check spoken
  Japanese, and who may check Chinese is 0045's question.

**The tool**

- Free tools, chosen per character: **Qwen3-TTS** designs every voice from a
  description, and **Qwen3-TTS** or **Chatterbox** reads the lines, as the cast
  table says. No paid service. Paid services stay allowed if the free tools
  stop being good enough.
- Candidates are newly designed voices near the one Nick liked, never a clip
  pitched up or down.

### The cast

Chapter 1's speakers and Vix. A character from a later chapter gets a round of
their own when they are written (0077). "Asked for" is the description the
voice was designed from, shortened; the voices themselves are in
`assets-src/audio/voice-refs/`.

| Character | Id | Pick | Reader | Asked for |
| --------- | -- | ---- | ------ | --------- |
| Hollis | `retainer` | A | Chatterbox | Man of 56, old soldier; low, dry, weathered, gruff |
| The lead, man | `lead_m` | A | Qwen3-TTS | Man in his early 20s; clear, even, quiet, calm |
| The lead, woman | `lead_f` | B | Qwen3-TTS | Woman of 22, in exile; low, dry, quiet, slightly wry |
| Harl | `red_captain` | C | Qwen3-TTS | Big man of 46, rough English accent; loud, booming, gravelly |
| Tamsin | `sergeant` | A | Qwen3-TTS | Woman of 27, cavalry sergeant; fast, loud, bossy, hot-tempered |
| Rue | `heretic` | F | Qwen3-TTS | Woman of 19, medium pitch; sharp, angry, quick, a little rough |
| Maud | `keeper` | A | Qwen3-TTS | Woman of 27, healer; anxious, warm, kind |
| Aske | `poacher` | C | Chatterbox | Girl of 17, Scottish highland accent; clear, hard, serious |
| Dace | `rival` | D | Qwen3-TTS | Man of 22, tenor; warm, quick, easy charm |
| The messenger | `messenger` | A | Qwen3-TTS | Young man, a servant; plain, nervous, respectful |
| The narrator | `>` | E | Qwen3-TTS | Man of about 50; deep, gravelly, steady, unhurried |
| Vix | `guide` | K | Chatterbox | Woman in her late 20s; smooth, velvety, bright, playfully teasing |

### The agent's starting rules

Nick can veto any of these.

- The word the voice says for the player's name is "the exile" for now. A
  later line takes a word that fits the lead's situation at that point (0754).
- The New Game question comes after the familiarity question, in today's
  new-game flow, until 0860 gathers them into a menu.
- A voice speaks the language of the text. There is no separate voice language
  setting.
- Vix's sexy, upbeat direction is how she sounds. Her wording stays by her
  character sheet (`docs/story/characters/guide.md`): short, dry, plain.

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

## Still open (Nick decides; not yet asked)

Japanese, ticket **0042**:

- The Japanese font, picked from rendered screens.
- How the player picks a language (asked on first launch, or only in Options).
- How the game says the translation is machine-made.
- How a Japanese player names the lead.
- Whether names are written in katakana.

Chinese, ticket **0045** (after 0042; picking a language and the
machine-translation label are 0042's answers and apply to both):

- Which characters write "Shuyi", and the game's title in Chinese.
- Whether Nick's wife checks any of the Chinese, and how she is credited.
- The Chinese font, picked from rendered screens.
- Whether names are written in Chinese characters.
- How a Chinese player names the lead.

Until those are answered, nothing here is a rule a ticket may build on
beyond the four points above. Traditional Chinese is not asked for.

## How the technical side supports it

- Text in other languages: [ADR-0045](../adr/0045-languages-text-by-key-and-line-ids.md).
- Voice clips: [ADR-0046](../adr/0046-voice-clips-by-line-id.md).
- A human scriptwriter: ticket 0723 (a writer's kit: guide, checker, preview).
- Saying what is machine-made on the credits screen and the store pages:
  ticket 0907.
