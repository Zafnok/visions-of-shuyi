//! Data text in the player's language (ADR-0045, ticket 0235): the names
//! of classes, items, spells, skills, arts, terrain, units and people,
//! tips, chapter titles, battle notes and dialogue. The data and `core`
//! hold English; a screen shows any of it through [`Words`]
//! ([`Ctx::words`](crate::Ctx::words)), never by reading a `name` field
//! itself (`cargo xtask check-text` fails on one).

use std::sync::Arc;

use trpg_content::{ChapterDef, Lang, LangCode, Names, Scene, Tip};
use trpg_core::lead::LEAD_ID;
use trpg_core::{
    ArtDef, BattleNote, ClassDef, ClassId, ClassTable, ItemDef, ItemId, ItemTable, LeadGender,
    SkillDef, SpellDef, TerrainRules, Unit,
};

/// The data's text in one language. Cheap to copy: pass it to whatever
/// builds a line of text.
#[derive(Debug, Clone, Copy)]
pub struct Words<'a> {
    /// The languages and the code of the one in use; `None` for English,
    /// which is what the data holds.
    pack: Option<(&'a Lang, &'a LangCode)>,
}

/// [`Words`] to keep: what a screen holds to name things between frames,
/// where it has no [`Ctx`](crate::Ctx) (a battle applying a command).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Language {
    /// The languages and the code of the one in use; `None` for English.
    pack: Option<(Arc<Lang>, LangCode)>,
}

impl Language {
    /// The language `code` of `lang`.
    pub fn new(lang: &Arc<Lang>, code: &LangCode) -> Self {
        let pack = lang.pack(code).map(|_| (Arc::clone(lang), code.clone()));
        Self { pack }
    }

    /// Whether it is the language `code` of `lang`: nothing to change.
    pub fn is(&self, lang: &Arc<Lang>, code: &LangCode) -> bool {
        match &self.pack {
            Some((mine, my_code)) => Arc::ptr_eq(mine, lang) && my_code == code,
            None => lang.pack(code).is_none(),
        }
    }

    /// The data's text in this language.
    pub fn words(&self) -> Words<'_> {
        Words {
            pack: self.pack.as_ref().map(|(lang, code)| (&**lang, code)),
        }
    }
}

impl<'a> Words<'a> {
    /// English: every text as the data has it.
    pub const ENGLISH: Words<'static> = Words { pack: None };

    /// The text of `lang` in language `code`.
    pub fn new(lang: &'a Lang, code: &'a LangCode) -> Self {
        let pack = lang.pack(code).map(|_| (lang, code));
        Self { pack }
    }

    /// Whether this is English: the data's own text.
    pub fn is_english(self) -> bool {
        self.pack.is_none()
    }

    /// The name of `class`.
    pub fn class(self, class: &'a ClassDef) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.class_name(code, &class.id, &class.name),
            None => &class.name,
        }
    }

    /// The name of class `id` of `classes` (its id if the table lacks it).
    pub fn class_of(self, id: &'a ClassId, classes: &'a ClassTable) -> &'a str {
        classes.get(id).map_or(id.0.as_str(), |c| self.class(c))
    }

    /// The name of item `id` of `items` (its id if the table lacks it).
    pub fn item(self, id: &'a ItemId, items: &'a ItemTable) -> &'a str {
        let Some(english) = items.get(id).map(ItemDef::name) else {
            return &id.0;
        };
        match self.pack {
            Some((lang, code)) => lang.item_name(code, id, english),
            None => english,
        }
    }

    /// The name of `spell`.
    pub fn spell(self, spell: &'a SpellDef) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.spell_name(code, spell),
            None => &spell.name,
        }
    }

    /// The name of `skill`.
    pub fn skill(self, skill: &'a SkillDef) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.skill_name(code, skill),
            None => &skill.name,
        }
    }

    /// The name of `art`.
    pub fn art(self, art: &'a ArtDef) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.art_name(code, art),
            None => &art.name,
        }
    }

    /// The name of `terrain`.
    pub fn terrain(self, terrain: &'a TerrainRules) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.terrain_name(code, &terrain.name),
            None => &terrain.name,
        }
    }

    /// The name of `unit`. A named character's is the names table's and a
    /// generic unit's its class's, each translated while the unit still
    /// carries the English the translation was made from; the lead's is
    /// the player's own.
    pub fn unit(self, unit: &'a Unit) -> &'a str {
        let english = unit.name.as_str();
        let Some((lang, code)) = self.pack else {
            return english;
        };
        match &unit.character {
            Some(id) if id.0 == LEAD_ID => english,
            Some(id) => lang.name(code, &id.0, english),
            None => lang.class_name(code, &unit.class, english),
        }
    }

    /// The display name `id` of `names` (the id itself if the table lacks
    /// it).
    pub fn name(self, names: &'a Names, id: &'a str) -> &'a str {
        let Some(english) = names.get(id) else {
            return id;
        };
        match self.pack {
            Some((lang, code)) => lang.name(code, id, english),
            None => english,
        }
    }

    /// The names table `names`, for filling the name tokens of a line.
    pub fn names(self, names: &Names) -> Names {
        match self.pack {
            Some((lang, code)) => lang.names(code, names),
            None => names.clone(),
        }
    }

    /// The title of `tip`.
    pub fn tip_title(self, tip: &'a Tip) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.tip_title(code, tip),
            None => &tip.title,
        }
    }

    /// The text of `tip`, placeholders and all.
    pub fn tip_text(self, tip: &'a Tip) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.tip_text(code, tip),
            None => &tip.text,
        }
    }

    /// The title of `chapter`.
    pub fn chapter_title(self, chapter: &'a ChapterDef) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.chapter_title(code, chapter),
            None => &chapter.title,
        }
    }

    /// The text of battle note `note`.
    pub fn battle_note(self, note: &'a BattleNote) -> &'a str {
        match self.pack {
            Some((lang, code)) => lang.battle_note(code, note),
            None => &note.text,
        }
    }

    /// `scene` as a lead of `gender` reads it: its lines, replies and
    /// captions in the language, with their ids and tokens kept.
    #[must_use]
    pub fn scene(self, scene: &Scene, gender: LeadGender) -> Scene {
        match self.pack {
            Some((lang, code)) => lang.scene(code, scene, gender),
            None => scene.clone(),
        }
    }
}

/// Languages made for tests.
#[cfg(test)]
pub(crate) mod testing {
    use std::collections::BTreeMap;

    use trpg_content::lang::{PackSources, pack_from_files};
    use trpg_content::{Content, Lang, LangCode};

    /// The code of the pack [`shouting`] makes.
    pub(crate) fn shout() -> LangCode {
        LangCode::new("shout").unwrap()
    }

    /// `content`'s data with a pack, [`shout`], that has every name, title
    /// and note of the data in capitals (the tips aside: their
    /// placeholders must stay as written). A test that shows a name in it
    /// sees at once whether the name went through [`Words`](super::Words).
    pub(crate) fn shouting(content: &Content) -> Lang {
        let source = content.lang.source().clone();
        let entries: Vec<String> = source
            .data
            .iter()
            .filter(|(key, _)| !key.starts_with("tips."))
            .map(|(key, english)| {
                let text = english.to_uppercase();
                format!("(key: {key:?}, source: {english:?}, text: {text:?})")
            })
            .collect();
        let data = format!("[{}]", entries.join(","));
        let sources = PackSources {
            info: ("lang.ron", r#"(name: "SHOUT", made_by: Machine)"#),
            ui: ("ui.ron", "[]"),
            data: Some(("data.ron", &data)),
            dialogue: Vec::new(),
        };
        let pack = pack_from_files(&sources, &BTreeMap::new(), Some(&source));
        let pack = pack.unwrap_or_else(|errors| panic!("{errors:?}"));
        Lang::with_data(BTreeMap::new(), source, [(shout(), pack)].into())
    }
}

#[cfg(test)]
mod tests {
    use trpg_content::lang::TEST;
    use trpg_core::{BattleNote, CharacterId, ItemId, LeadProfile};

    use super::testing::{shout, shouting};
    use super::*;
    use crate::screen::tests::ctx;
    use crate::screens::battle::quick_battle;

    fn test() -> LangCode {
        LangCode::new(TEST).unwrap()
    }

    #[test]
    fn english_is_the_text_of_the_data() {
        let c = ctx();
        let content = &c.content;
        let english = Words::ENGLISH;
        assert!(english.is_english());
        assert!(Words::new(&content.lang, &LangCode::english()).is_english());
        // A code with no pack is English too.
        let zz = LangCode::new("zz").unwrap();
        assert!(Words::new(&content.lang, &zz).is_english());
        assert!(!Words::new(&content.lang, &test()).is_english());
        let guard = &content.classes.classes[&ClassId("guard".to_owned())];
        assert_eq!(english.class(guard), "Guard");
        let potion = ItemId::new("potion");
        assert_eq!(english.item(&potion, &content.items), "Potion");
        let fire = content.spells.spells.values().find(|s| s.id.0 == "fire");
        assert_eq!(fire.map(|s| english.spell(s)), Some("Fire"));
        let skill = content
            .skills
            .skills
            .values()
            .find(|s| s.id.0 == "keen_edge");
        assert_eq!(skill.map(|s| english.skill(s)), Some("Keen Edge"));
        let art = content.arts.arts.values().find(|a| a.id.0 == "guard_break");
        assert_eq!(art.map(|a| english.art(a)), Some("Guard Break"));
        let terrains = &content.terrain.rules.terrains;
        let forest = terrains.iter().find(|t| t.name == "Forest");
        assert_eq!(forest.map(|t| english.terrain(t)), Some("Forest"));
        let tips = &content.tips.tips;
        let tip = tips.iter().find(|t| t.id == "battle_start").unwrap();
        assert_eq!(english.tip_title(tip), "Your move");
        assert_eq!(english.tip_text(tip), tip.text);
        let chapter = &content.chapters["test"];
        assert_eq!(english.chapter_title(chapter), "Test Chapter");
        let note = &content.battles["test"].battle_notes[1];
        assert_eq!(english.battle_note(note), note.text);
        assert_eq!(english.name(&content.names, "test_knight"), "Test Knight");
        assert_eq!(english.names(&content.names), content.names);
        let scene = content.dialogue.get("test").unwrap();
        assert_eq!(&english.scene(scene, LeadGender::Male), scene);
    }

    #[test]
    fn the_test_pack_names_what_it_has_and_english_names_the_rest() {
        let c = ctx();
        let content = &c.content;
        let code = test();
        let words = Words::new(&content.lang, &code);
        let class = |id: &str| &content.classes.classes[&ClassId(id.to_owned())];
        assert_eq!(words.class(class("guard")), "GUARD");
        assert_eq!(words.class(class("archer")), "Archer");
        let (guard, nope) = (ClassId("guard".to_owned()), ClassId("nope".to_owned()));
        assert_eq!(words.class_of(&guard, &content.classes), "GUARD");
        // A class the table lacks shows as its id.
        assert_eq!(words.class_of(&nope, &content.classes), "nope");
        assert_eq!(Words::ENGLISH.class_of(&nope, &content.classes), "nope");
        let item = |id: &str| words.item(&ItemId::new(id), &content.items).to_owned();
        assert_eq!(item("potion"), "POTION");
        assert_eq!(item("iron_sword"), "IRON SWORD");
        assert_eq!(item("steel_sword"), "Steel Sword");
        // Stale: English. Not an item: its id.
        assert_eq!(item("iron_axe"), "Iron Axe");
        assert_eq!(item("nope"), "nope");
        let nope = ItemId::new("nope");
        assert_eq!(Words::ENGLISH.item(&nope, &content.items), "nope");
        let spell = |id: &str| content.spells.spells.values().find(|s| s.id.0 == id);
        assert_eq!(spell("fire").map(|s| words.spell(s)), Some("FIRE"));
        assert_eq!(spell("frost").map(|s| words.spell(s)), Some("Frost"));
        let skill = |id: &str| content.skills.skills.values().find(|s| s.id.0 == id);
        let keen = skill("keen_edge").map(|s| words.skill(s));
        assert_eq!(keen, Some("KEEN EDGE"));
        assert_eq!(skill("flurry").map(|s| words.skill(s)), Some("Flurry"));
        let art = |id: &str| content.arts.arts.values().find(|a| a.id.0 == id);
        let guard_break = art("guard_break").map(|a| words.art(a));
        assert_eq!(guard_break, Some("GUARD BREAK"));
        let flowing = art("flowing_cut").map(|a| words.art(a));
        assert_eq!(flowing, Some("Flowing Cut"));
        let terrain = |name: &str| {
            let terrains = &content.terrain.rules.terrains;
            let found = terrains.iter().find(|t| t.name == name);
            found.map(|t| words.terrain(t))
        };
        assert_eq!(terrain("Forest"), Some("FOREST"));
        assert_eq!(terrain("Plain"), Some("Plain"));
        let tip = |id: &str| content.tips.tips.iter().find(|t| t.id == id).unwrap();
        assert_eq!(words.tip_title(tip("battle_start")), "YOUR MOVE");
        let text = words.tip_text(tip("battle_start"));
        assert!(text.starts_with("STEER THE CURSOR"), "{text}");
        assert_eq!(words.tip_title(tip("unit_selected")), "Moving a unit");
        let chapters = &content.chapters;
        assert_eq!(words.chapter_title(&chapters["test"]), "TEST CHAPTER");
        assert_eq!(words.chapter_title(&chapters["quick"]), "Quick Battle");
        let notes = &content.battles["test"].battle_notes;
        assert!(words.battle_note(&notes[1]).starts_with("SEIZE THE FORT"));
        assert_eq!(words.battle_note(&notes[0]), notes[0].text);
        let made_up = BattleNote {
            text: "Run.".to_owned(),
            units: Vec::new(),
        };
        assert_eq!(words.battle_note(&made_up), "Run.");
    }

    #[test]
    fn people_are_named_by_the_names_table_of_the_language() {
        let c = ctx();
        let code = test();
        let words = Words::new(&c.content.lang, &code);
        let names = &c.content.names;
        assert_eq!(words.name(names, "test_knight"), "TEST KNIGHT");
        assert_eq!(words.name(names, "test_lord"), "Test Lord");
        // Someone the table lacks shows as their id, in any language.
        assert_eq!(words.name(names, "nobody"), "nobody");
        assert_eq!(Words::ENGLISH.name(names, "nobody"), "nobody");
        let theirs = words.names(names);
        assert_eq!(theirs.get("test_knight"), Some("TEST KNIGHT"));
        assert_eq!(theirs.get("test_archer"), Some("TEST ARCHER"));
        assert_eq!(theirs.get("test_lord"), Some("Test Lord"));
        assert_eq!(theirs.names.len(), names.names.len());
    }

    #[test]
    fn a_unit_is_named_by_who_it_is() {
        let c = ctx();
        let state = quick_battle(&c.content).unwrap();
        let code = test();
        let words = Words::new(&c.content.lang, &code);
        let unit = |name: &str| state.units().iter().find(|u| u.name == name).unwrap();
        // A named character: the names table's entry for their id.
        assert_eq!(words.unit(unit("Test Knight")), "TEST KNIGHT");
        assert_eq!(words.unit(unit("Test Lord")), "Test Lord");
        assert_eq!(Words::ENGLISH.unit(unit("Test Knight")), "Test Knight");
        // A generic unit: its class's name.
        let brigand = unit("Brigand");
        assert_eq!(brigand.character, None);
        assert_eq!(words.unit(brigand), "BRIGAND");
        assert_eq!(Words::ENGLISH.unit(brigand), "Brigand");
        // One named something else keeps its name: a generic unit a
        // battle renamed, a character an old save calls otherwise.
        let mut renamed = brigand.clone();
        renamed.name = "Guard".to_owned();
        assert_eq!(words.unit(&renamed), "Guard");
        let mut knight = unit("Test Knight").clone();
        knight.name = "Brigand".to_owned();
        assert_eq!(words.unit(&knight), "Brigand");
        // The lead's name is the player's, whatever the pack calls anyone.
        let lang = shouting(&c.content);
        let code = shout();
        let shouted = Words::new(&lang, &code);
        assert_eq!(shouted.unit(unit("Test Lord")), "TEST LORD");
        let mut lead = unit("Test Lord").clone();
        lead.character = Some(CharacterId(LEAD_ID.to_owned()));
        lead.name = "Ellery".to_owned();
        assert_eq!(lang.name(&code, LEAD_ID, "Ellery"), "ELLERY");
        assert_eq!(shouted.unit(&lead), "Ellery");
    }

    #[test]
    fn a_scene_is_told_in_the_language_for_the_leads_gender() {
        let c = ctx();
        let code = test();
        let words = Words::new(&c.content.lang, &code);
        let scene = c.content.dialogue.get("test").unwrap();
        let last = |gender| {
            let told = words.scene(scene, gender);
            let lines = told.lines();
            lines.last().map(|line| line.text.to_owned()).unwrap()
        };
        assert!(last(LeadGender::Male).contains("TIGHTENS HIS GRIP"));
        assert!(last(LeadGender::Female).contains("TIGHTENS HER GRIP"));
        let told = words.scene(scene, LeadGender::Male);
        assert_ne!(&told, scene);
        assert_eq!(told.lines().len(), scene.lines().len());
        assert_eq!(told.lines()[0].id, scene.lines()[0].id);
    }

    #[test]
    fn a_kept_language_gives_the_same_words() {
        let c = ctx();
        let lang = &c.content.lang;
        let english = Language::default();
        assert!(english.words().is_english());
        assert_eq!(Language::new(lang, &LangCode::english()), english);
        assert_eq!(Language::new(lang, &LangCode::new("zz").unwrap()), english);
        let kept = Language::new(lang, &test());
        assert_ne!(kept, english);
        let guard = &c.content.classes.classes[&ClassId("guard".to_owned())];
        assert_eq!(kept.words().class(guard), "GUARD");
        assert_eq!(kept.clone().words().class(guard), "GUARD");
        // Whether it is already the language asked for.
        assert!(kept.is(lang, &test()));
        assert!(!kept.is(lang, &LangCode::english()));
        assert!(!kept.is(lang, &shout()));
        assert!(english.is(lang, &LangCode::english()));
        assert!(english.is(lang, &LangCode::new("zz").unwrap()));
        assert!(!english.is(lang, &test()));
        // The same code of other languages is another language.
        let other = Arc::new(shouting(&c.content));
        assert!(!Language::new(&other, &shout()).is(lang, &shout()));
        assert!(Language::new(&other, &shout()).is(&other, &shout()));
        assert!(!kept.is(&other, &test()));
    }

    #[test]
    fn the_context_gives_the_words_of_the_language_in_use() {
        let mut c = ctx();
        let guard = c.content.classes.classes[&ClassId("guard".to_owned())].clone();
        assert!(c.words().is_english());
        assert_eq!(c.language(), Language::default());
        assert_eq!(c.effective_lang(), &LangCode::english());
        c.lang = test();
        assert_eq!(c.effective_lang(), &test());
        assert_eq!(c.words().class(&guard), "GUARD");
        assert_eq!(c.language().words().class(&guard), "GUARD");
        // The test pack is only for builds with debug tools.
        c.debug_tools = false;
        assert_eq!(c.effective_lang(), &LangCode::english());
        assert_eq!(c.words().class(&guard), "Guard");
        assert_eq!(c.language(), Language::default());
        c.lead = LeadProfile::new("Mara", LeadGender::Female);
        assert_eq!(c.text("title.new_game"), "New Game");
    }
}
