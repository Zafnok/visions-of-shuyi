# Languages (`assets/lang/`)

English is the source; every other language is a **pack** that overlays it
([ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)).
Loaded and validated by `trpg_content::lang`.

```
assets/lang/
  en/ui.ron             English screen text: a map of key to text
  <code>/               one pack (2 to 8 lowercase letters: ja)
    lang.ron            its own name, and who made it
    ui.ron              screen text
    data.ron            names, tips, titles and notes of the data files
    dialogue/<file>.ron lines and captions, one file per .dlg file
  test/                 the pack the tests use (English in capitals);
                        never offered to players
```

A pack needs `lang.ron` and `ui.ron`; `data.ron` and `dialogue/` are
optional. Any other file is an error.

## Entries

`ui.ron`, `data.ron` and each `dialogue/*.ron` are lists of entries:

```ron
(key: "items.iron_sword.name", source: "Iron Sword", text: "IRON SWORD"),
```

- `key`: which text this is (below).
- `source`: a copy of the English the translation was made from.
- `text`: the translation.

What the game shows:

| The entry… | Shows | Fails a gate? |
| ---------- | ----- | ------------- |
| has the `source` English has today | the translation | |
| has another `source` (**stale**: English changed since) | English | no |
| isn't there (**missing**) | English | no |
| has a key English doesn't have (`ui.ron`, `data.ron`) | | **yes** |
| is for a dialogue line English no longer has (**orphan**) | nothing | no |

So editing English never blocks a PR. `cargo xtask lang-status <code>`
lists what is missing, stale and orphaned; it pairs each orphan with the
untranslated line of its scene whose English is nearest, with the old
translation, so a reworded line is quick to redo.

## Keys

English for all of these stays in the data files; a pack only overlays it.

### `ui.ron`: screen text

Keys chosen by hand, `screen.thing` (`title.new_game`), listed in
`en/ui.ron`. A translation must use the same `{placeholders}` as its
`source`.

### `data.ron`: text from the data files

| Text | Key | English lives in |
| ---- | --- | ---------------- |
| A class's name | `classes.<id>.name` | `data/classes.ron` |
| An item's name (weapon, armour, accessory, consumable, seal) | `items.<id>.name` | `data/items.ron` |
| A spell's name | `spells.<id>.name` | `data/spells.ron` |
| A skill's name | `skills.<id>.name` | `data/skills.ron` |
| A Combat Art's name | `arts.<id>.name` | `data/arts.ron` |
| A terrain's name | `terrain.<id>.name` | `data/terrain.ron` |
| A name of the names table (people, places, factions, terms) | `names.<id>` | `data/names.ron` |
| A tip's title and text | `tips.<id>.title`, `tips.<id>.text` | `data/tips.ron` |
| A chapter's title | `chapters.<id>.title` | `chapters/<id>.ron` |
| A battle note | `battles.<id>.note_<n>` (n from 1, in file order) | `battles/<id>.ron` |

Rules beyond the same placeholders as the `source`:

- A name isn't empty and holds no `{` or `}`.
- A tip keeps to the limits of English tips, counted in cells: a title of
  1 to 30, a text of 1 to 3 lines of at most 60.

Not here, because nothing shows them or they aren't translated:

- **Named characters and speakers** have no `name` field: their name is
  the names table's entry for their id (`names.<id>`).
- **Generic units** are named after their class (`classes.<id>.name`).
- **The lead's name** is the player's own.
- **A unit's two-letter map label** is part of the glyph look, not text.
- **A map's `name`** is shown nowhere.
- **Credits** (titles of works and their authors) stay as they are.

### `dialogue/<file>.ron`: lines and captions

| Text | Key |
| ---- | --- |
| A speech line, a narration line, a reply | its line id, `<scene>_<8 hex>` (`_2`… on a repeat) |
| A `@caption` | `caption.<scene>_<8 hex>`, hashed like a line id from the caption's text |

`cargo xtask lines [scene]` lists every line with its id. A pack's files
may be split any way; one per `.dlg` file keeps them easy to find. A key
is used once in the whole pack.

```ron
(key: "test_21833604", source: "You're late.", text: "YOU'RE LATE."),
(
    key: "test_4ea176cf",
    source: "{lead} tightens {their} grip on the sword.",
    text_m: "{lead} TIGHTENS HIS GRIP ON THE SWORD.",
    text_f: "{lead} TIGHTENS HER GRIP ON THE SWORD.",
),
```

- An entry has `text`, or both `text_m` and `text_f`: one text per gender
  of the lead, for a language where the lead's gender changes more than a
  pronoun. The dialogue screen picks by the lead's gender.
- **Tokens** follow the rule of English scripts: only `{lead}`, the
  pronoun tokens (`{they}`, `{Their}`…; they give the *English* words, so
  a translation rarely wants them) and name tokens `{n:<id>}` /
  `{N:<id>}` with an id of the names table. A translation need not use
  the tokens of its `source`.
- **A name token** is filled with the pack's `names.<id>`, or the English
  name if the pack lacks it. `{N:…}` capitalises only a name that starts
  with a Latin letter.
- **Length**, in cells, with every token as long as it can get (a name
  token as the pack's longest name): 200 for a speech or narration line,
  60 for a reply. English's 40-character limit on the lead's own lines is
  a rule for writing English, and its plain-ASCII and no-written-names
  rules are too.
- Changing a line's English words gives it a new id, so its entry becomes
  an **orphan**: nothing shows it, nothing fails, and `lang-status` lists
  it beside the line it most likely became.
