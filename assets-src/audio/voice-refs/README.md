# Voice reference clips

The game's voices, one clip per cast entry
([ADR-0107](../../../docs/adr/0107-free-local-voice-tools-with-designed-voices-and-a-reader-per-character.md)).
Who reads each one is in `docs/design/voices-languages-and-script.md`, *The
cast*. Every clip says:

> The road runs north past the mill and over the ford. We keep moving until
> dark, and we eat when we get there.

Each was made by `Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign` (revision
`5ecdb67327fd`, Qwen3-TTS code `022e286`), in English, at `torch.bfloat16`,
with `torch.manual_seed` set to the seed below and `instruct` set to the
description. Clips are FLAC copies of the model's WAV. No real person's voice
went in, and none was asked for.

| File | Seed | Description |
| ---- | ---- | ----------- |
| `retainer.flac` | 7 | A man of fifty-six, an old soldier. Low, dry, weathered and slightly hoarse voice. Unhurried, plain, gruff, keeps his feelings hidden. |
| `lead_m.flac` | 7 | A young man in his early twenties. Clear, even, quiet voice of middle pitch. Few words, calm and steady. |
| `lead_f.flac` | 25 | A young woman of twenty-two, a noblewoman in exile. A low, dry, quiet voice, slightly wry, few words. |
| `red_captain.flac` | 28 | A big man of forty-six with a rough English accent, a mercenary captain. A loud, booming, gravelly voice used to carrying across a battlefield. |
| `sergeant.flac` | 31 | A woman of twenty-seven, a cavalry sergeant. A fast, loud, bossy, hot-tempered voice with a sharp edge, always talking over people. |
| `heretic.flac` | 61 | A young woman of nineteen with a natural, medium-pitched voice, neither high nor deep. Sharp, angry and quick, a little rough at the edges, rude when she is scared. |
| `keeper.flac` | 37 | A woman of twenty-seven, a village healer and priest. An anxious, warm, kind voice that talks too much and apologizes. |
| `poacher.flac` | 43 | A girl of seventeen with a Scottish highland accent, a proud hunter. A clear, hard, serious voice, jaw set, unused to talking. |
| `rival.flac` | 101 | An adult man of twenty-two with a male tenor voice. Warm, quick and easy, a natural talker with a smile in his voice. |
| `messenger.flac` | 47 | A young man, a servant carrying a message. A plain, nervous, respectful voice. |
| `narrator.flac` | 108 | A man of about fifty with a deep male voice. Gravelly, steady and unhurried, a soldier telling an old story. |
| `guide.flac` | 214 | An adult woman in her late twenties with a smooth, velvety, rich voice. Bright and energetic, playfully teasing, enjoying herself. |

The seed reproduces a voice only on the same model, code and machine, so these
files, not the table, are the voices. Don't replace one without a round with
Nick (0043).
