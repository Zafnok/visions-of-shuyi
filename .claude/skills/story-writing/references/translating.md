# Translating

A translation is a step after the script (ADR-0045 §6): a language pack
laid over the English, made by machine a chapter at a time, by a written
process for each language. Nobody writing it can judge it as a native
reader does, so the process is the quality: a glossary, a names table, a
style sheet per speaker, and a second session that translates the result
back without seeing the English.

- **Chinese (Simplified), pack `zhhans`** (ticket 0725): follow
  `assets-private/story/zh/README.md` from top to bottom. Beside it:
  `glossary.md` (one term, one translation), `names.md` (the Chinese
  form of every `names.ron` id; "Shuyi" is 姝怡, exactly), `styles.md`
  (how each speaker says "I" and "you", what they call each other, their
  particles) and `for-review.md` (the title, the names and the glossary
  on one page for Nick's wife, who reads them before they are used and
  plays Chapter 1 in Chinese:
  `docs/design/voices-languages-and-script.md`, *Chinese (Simplified)*).
- **Japanese** has no process yet: ticket 0718 writes its twin, and moves
  the steps both languages share into one page.
- **Translate from the English**, never from another pack, and never
  change the English to make it easier: write a ticket for the line.
- **Don't translate a script that is about to be rewritten.** A line's id
  is a hash of its words, so a reworded line loses its translation. While
  the writer is being hired (0766), ask Nick before starting a chapter.
- **A pack that isn't translated isn't in the build**: the moment a
  language's `lang.ron` is there, the first launch offers it to every
  player. It lands with its first chapter.
- **A rename** gives the name its new form in each language's names list
  in the same change (`assets-private/story/names.md`).
