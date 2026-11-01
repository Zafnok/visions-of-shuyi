//! Data schemas, parsers and validation for game content. See ADR-0004 and
//! ADR-0005. All content comes from the embedded asset bundle ([`bundle`]);
//! [`load_embedded`] loads and validates everything, reporting every error.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// What `std`'s prelude gives for free, for the `no_std` build.
mod prelude {
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
    pub use alloc::{format, vec};
}
use prelude::*;

pub mod ai;
pub mod art;
pub mod audio;
pub mod basic_shop;
pub mod battle;
pub mod battler;
pub mod bundle;
pub mod camp;
pub mod chapter;
pub mod character;
pub mod cinematic;
pub mod class;
pub mod credits;
pub mod dialogue;
mod enums;
pub mod error;
pub mod font;
pub mod formats;
pub mod image;
pub mod ink;
pub mod item;
pub mod keymap;
pub mod lang;
pub mod lesson;
pub mod lighting;
pub mod map;
pub mod names;
pub mod palette;
#[cfg(feature = "std")]
pub mod playdate_images;
pub mod portrait;
pub mod ron_loader;
pub mod skill;
pub mod skirmish;
pub mod spell;
pub mod support;
pub mod terrain;
pub mod tileset;
pub mod tip;
pub mod title;
pub mod trigger;
pub mod voice;
pub mod worldmap;

use alloc::collections::BTreeMap;

use alloc::sync::Arc;

use trpg_core::{
    AiWeights, ArtTable, BasicStock, BattleDef, BattleMap, CampEvent, ClassTable, GameTables,
    ItemTable, SkillTable, SpellTable, SupportTable,
};

pub use audio::{AudioManifest, Credit, CreditRef, MusicCue, SoundCue};
pub use battle::BattleRefs;
pub use battler::{Battler, BattlerTable};
pub use bundle::layers;
pub use chapter::{ChapterDef, NewGameDef, battle_campaign, new_campaign, roster_units};
pub use character::{CharacterTable, GenericTemplate, character_unit, check_map_labels};
pub use cinematic::{
    Act, ActionDef, CastMember, CinematicDef, CombatShot, Fight, Heading, OpeningDef, Shot,
    ShotAction, ShotDef, TilePoint,
};
pub use credits::{CreditEntry, CreditGroup, Credits};
pub use dialogue::{
    ChoiceOption, DialogueTable, LineId, MusicLine, PlaceLine, Present, Scene, Side, Stage, Step,
    deaths, joins,
};
pub use error::{ContentError, ContentErrors};
pub use font::FontAtlasDef;
pub use image::{ImageId, ImageInfo, ImageTable};
pub use ink::Ink;
pub use keymap::{
    Action, ActionSet, Bindings, Button, ButtonChord, Chord, ChordDef, Key, KeyGroup, KeymapDef,
    LEFT_HANDED, Layout, LayoutKeys, PadKeys, RIGHT_HANDED, RepeatDef, SLOTS, StickDef, UiAction,
    load_keymap,
};
pub use lang::{
    Lang, LangCode, LangInfo, LangPack, LangStatus, LineEntry, LineText, MadeBy, Orphan,
};
pub use lesson::{DoneWhen, Lesson, LessonItem, LessonLevel, LessonTable};
pub use lighting::{Light, Lighting};
pub use map::{MapDef, MapLegend, MapLook};
pub use names::Names;
pub use palette::{PaletteDef, Theme};
pub use portrait::Portrait;
/// One-bit pictures, the Playdate's in place of its PNGs (ticket 1040,
/// ADR-0085): the format is Shuyi Engine's (`shuyi_assets::bitmap`).
pub use shuyi_assets::bitmap;
pub use terrain::{TerrainDef, TerrainDisplay, TerrainDisplayTable};
pub use tileset::{CornerLayer, ImageRect, Layer, Look, Picture, TerrainTiles, TileLayer, Tileset};
pub use tip::{Tip, TipTable, TipTrigger};
pub use title::TitleArt;
pub use trigger::check_triggers;
pub use voice::{Playable, Variant, VoiceManifest};
pub use worldmap::WorldMapDef;

/// All validated game content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    /// Named colours of the default theme (ADR-0012).
    pub palette: PaletteDef,
    /// The palette of every colour theme (ticket 0806), the default's too.
    pub themes: BTreeMap<Theme, PaletteDef>,
    /// Key bindings for every layout, and repeat timings (ADR-0015).
    pub keymap: KeymapDef,
    /// Font atlas layout; the image is `bundle::bytes(font::ATLAS_PNG_PATH)`.
    pub font: FontAtlasDef,
    /// Every other image in the bundle, by path, with its size (ADR-0038).
    pub images: ImageTable,
    /// What sprite map skins paint battle maps with, by id (ADR-0038).
    pub tilesets: BTreeMap<String, Tileset>,
    /// Terrain rules and looks.
    pub terrain: TerrainDef,
    /// Battle maps by id (file stem).
    pub maps: BTreeMap<String, MapDef>,
    /// The class tree and progression tables.
    pub classes: ClassTable,
    /// Items and item rules.
    pub items: ItemTable,
    /// What the Preparations shop sells, and from when (ticket 0443).
    pub basic_shop: BasicStock,
    /// Spells.
    pub spells: SpellTable,
    /// Class skills.
    pub skills: SkillTable,
    /// Combat Arts.
    pub arts: ArtTable,
    /// Display names by name id (ticket 0709).
    pub names: Names,
    /// Named characters and generic unit templates.
    pub characters: CharacterTable,
    /// Character portraits by character id (file stem).
    pub portraits: BTreeMap<String, Portrait>,
    /// Dialogue scenes by id.
    pub dialogue: DialogueTable,
    /// The AI's numbers.
    pub ai: AiWeights,
    /// One-time contextual tips.
    pub tips: TipTable,
    /// Checklist lessons (ticket 0456).
    pub lessons: LessonTable,
    /// Sound and music cues, pools and credits (ADR-0026).
    pub audio: AudioManifest,
    /// Every third-party work, for the credits screen (ticket 0808).
    pub credits: Credits,
    /// Battles by id (file stem).
    pub battles: BTreeMap<String, BattleDef>,
    /// Chapters by id (file stem).
    pub chapters: BTreeMap<String, ChapterDef>,
    /// How a new game starts.
    pub new_game: NewGameDef,
    /// Support rules and pairs (ticket 1002).
    pub supports: SupportTable,
    /// Camp events, in the order camp lists them (ticket 1003).
    pub camp: Vec<CampEvent>,
    /// English screen text and the language packs (ADR-0045).
    pub lang: Arc<Lang>,
    /// Combat pictures and the backdrops behind them (ADR-0056).
    pub battlers: BattlerTable,
    /// The lights a map can be in (ADR-0058).
    pub lighting: Lighting,
    /// The light a battle is fought in instead of its map's, by battle
    /// id, for the battles that name one. Look data: it isn't in the
    /// [`BattleDef`].
    pub battle_lights: BTreeMap<String, String>,
    /// The title screen's logo and backdrops (ADR-0059).
    pub title: TitleArt,
    /// Cinematics by id (file stem), ADR-0060.
    pub cinematics: BTreeMap<String, CinematicDef>,
    /// World maps by id (file stem), one an act (ticket 1007, ADR-0099).
    pub worldmaps: BTreeMap<String, WorldMapDef>,
    /// The openings a conversation's script may ask for, by id (file
    /// stem): ticket 0734, ADR-0076.
    pub openings: BTreeMap<String, OpeningDef>,
}

impl Content {
    /// How `map` looks: the look of the map file it is, compared whole (a
    /// battle's map as its file has it, before anything changed its
    /// terrain); the look of a map that names none if it is no file's.
    pub fn map_look(&self, map: &BattleMap) -> MapLook {
        let file = self.maps.values().find(|def| def.map == *map);
        file.map(|def| def.look.clone()).unwrap_or_default()
    }

    /// How `battle`'s map looks: [its map's look](Self::map_look), in the
    /// battle's own light if its file names one.
    pub fn battle_look(&self, battle: &BattleDef) -> MapLook {
        let light = self.battle_lights.get(&battle.id);
        self.map_look(&battle.map).lit(light.map(String::as_str))
    }

    /// The battle scene `scene` is played for: its chapter's battle, for a
    /// chapter's intro or victory scene, else the battle with a trigger
    /// that plays it. `None` for a scene of no battle.
    pub fn battle_of_scene(&self, scene: &str) -> Option<&BattleDef> {
        let chapter = self.chapters.values().find(|c| {
            let mut scenes = c.intro_scenes.iter().chain(&c.victory_scenes);
            scenes.any(|s| s == scene)
        });
        if let Some(battle) = chapter.and_then(|c| self.battles.get(&c.battle)) {
            return Some(battle);
        }
        let mut battles = self.battles.values();
        battles.find(|b| b.triggers.iter().any(|t| t.scene == scene))
    }

    /// What a battle file refers to, as [`battle::from_source`] checks it.
    pub fn battle_refs(&self) -> BattleRefs<'_> {
        BattleRefs {
            maps: &self.maps,
            terrain: &self.terrain.rules,
            terrain_looks: &self.terrain.display,
            classes: &self.classes,
            items: &self.items,
            characters: &self.characters,
            dialogue: &self.dialogue,
            audio: &self.audio,
        }
    }

    /// The tables battles read, shared.
    pub fn tables(&self) -> GameTables {
        GameTables {
            terrain: Arc::new(self.terrain.rules.clone()),
            classes: Arc::new(self.classes.clone()),
            items: Arc::new(self.items.clone()),
            spells: Arc::new(self.spells.clone()),
            skills: Arc::new(self.skills.clone()),
            arts: Arc::new(self.arts.clone()),
            supports: Arc::new(self.supports.clone()),
        }
    }
}

/// Loads and validates every content type from the embedded bundle. Runs all
/// loaders and returns every error found, not just the first. Checks that
/// depend on another file (terrain colours, map legends) are skipped when that
/// file failed, so one broken file doesn't flood the report. Installs the
/// game's bundle first ([`bundle::install`]).
pub fn load_embedded() -> Result<Content, ContentErrors> {
    bundle::install();
    load::<&str>(None)
}

/// [`load_embedded`] with `scripts`, `(file name, source)` pairs, as the
/// dialogue files instead of the bundle's: every check a script goes
/// through, on scripts as they are on disk (`cargo xtask check-script`).
pub fn load_with_scripts<F: AsRef<str>>(scripts: &[(F, &str)]) -> Result<Content, ContentErrors> {
    bundle::install();
    load(Some(scripts))
}

/// Loads and validates everything; the dialogue files are `scripts`, or
/// the bundle's when `None`.
fn load<F: AsRef<str>>(scripts: Option<&[(F, &str)]>) -> Result<Content, ContentErrors> {
    let palette = crate::palette::load();
    let terrain = TerrainDef::load(palette.as_ref().ok());
    let maps = load_maps(terrain.as_ref().ok());
    let movement = terrain.as_ref().ok().map(|t| &t.rules.movement_types[..]);
    let classes = class::load(movement);
    let (items, basic_shop) = load_items(classes.as_ref().ok());
    let maps = check_map_features(maps, items.as_ref().ok(), terrain.as_ref().ok());
    let (lighting, names) = (lighting::load(), names::load());
    let lit = lighting.as_ref().ok();
    let (maps, battle_lights) = (check_map_lights(maps, lit), load_battle_lights(lit));
    let characters = character::load(
        classes.as_ref().ok(),
        items.as_ref().ok(),
        names.as_ref().ok(),
    );
    let spells = check_spell_references(
        spell::load(terrain.as_ref().ok().map(|t| &t.display)),
        classes.as_ref().ok(),
        characters.as_ref().ok(),
    );
    let skills = check_skill_references(skill::load(), classes.as_ref().ok());
    let arts = check_art_references(art::load(), items.as_ref().ok());
    let (images, audio) = (crate::image::load(), audio::load());
    let portraits = load_portraits(images.as_ref().ok());
    let stage_art = load_battlers(images.as_ref().ok(), terrain.as_ref().ok(), lit);
    let title_art = load_title(images.as_ref().ok());
    let (cast, faces) = (characters.as_ref().ok(), portraits.as_ref().ok());
    let (known, cues) = (names.as_ref().ok(), audio.as_ref().ok());
    let scripts = match scripts {
        Some(files) => dialogue::scripts_from_sources(files, cast, faces, known, cues),
        None => dialogue::load_scripts(cast, faces, known, cues),
    };
    let scenes = scripts.as_ref().ok().map(|s| &s.table);
    let (battles, chapters, new_game) = load_story(
        maps.as_ref().ok(),
        terrain.as_ref().ok(),
        classes.as_ref().ok(),
        items.as_ref().ok(),
        characters.as_ref().ok(),
        scenes,
        audio.as_ref().ok(),
    );
    let (fields, rules) = (battles.as_ref().ok(), (&classes, &terrain));
    let cinematics = load_cinematics(fields, cues, scenes, (cast, faces, known), rules);
    let openings = load_openings(maps.as_ref().ok(), cast, rules);
    let (supports, camp) = load_camp_talks(cast, scenes, fields);
    let story = (fields, chapters.as_ref().ok(), new_game.as_ref().ok());
    let places = (maps.as_ref().ok(), openings.as_ref().ok());
    let away = (supports.as_ref().ok(), camp.as_ref().ok());
    let dialogue = check_dialogue(scripts, places, story, away);
    let tilesets = load_tilesets(
        images.as_ref().ok(),
        terrain.as_ref().ok(),
        classes.as_ref().ok(),
        characters.as_ref().ok(),
    );
    let mut units = Loaded {
        classes,
        items,
        basic_shop,
        spells,
        skills,
        arts,
        names,
        characters,
        // Before the portraits move in: checked against the guide's.
        tips: tip::load(faces),
        lessons: load_lessons(battles.as_ref().ok(), faces),
        portraits,
        dialogue,
        ai: ai::load(),
        credits: credits::load(audio.as_ref().ok()),
        audio,
        images,
        tilesets,
        battles,
        chapters,
        new_game,
        supports,
        camp,
        lang: Ok(Lang::default()),
        battlers: stage_art,
        lighting,
        battle_lights,
        title: title_art,
        cinematics,
        openings,
        worldmaps: Ok(BTreeMap::new()),
    };
    load_worldmaps_and_lang(&mut units, terrain.as_ref().ok(), maps.as_ref().ok());
    let themes = palette::load_themes(palette.as_ref().ok());
    let content = assemble(
        palette,
        keymap::load_keymap(),
        crate::font::load(),
        terrain,
        maps,
        units,
    );
    // The combat shots' fights need everything else (ADR-0062).
    with_themes(content.and_then(cinematic::check_fights), themes)
}

/// Loads the world maps into `units` (checked against its battles,
/// chapters, scenes, characters, items and music cues), then the
/// languages, whose data text names the maps' places.
fn load_worldmaps_and_lang(
    units: &mut Loaded,
    terrain: Option<&TerrainDef>,
    maps: Option<&BTreeMap<String, MapDef>>,
) {
    units.worldmaps = load_worldmaps(units);
    units.lang = lang::load(data_text(units, terrain, maps));
}

/// The world maps, checked against what `units` loaded; none when one of
/// the files they need failed (its errors are reported already).
fn load_worldmaps(units: &Loaded) -> Result<BTreeMap<String, WorldMapDef>, Vec<ContentError>> {
    let (Ok(battles), Ok(chapters), Ok(dialogue)) =
        (&units.battles, &units.chapters, &units.dialogue)
    else {
        return Ok(BTreeMap::new());
    };
    let (Ok(characters), Ok(items), Ok(audio)) = (&units.characters, &units.items, &units.audio)
    else {
        return Ok(BTreeMap::new());
    };
    worldmap::load_all(worldmap::WorldRefs {
        battles,
        chapters,
        dialogue,
        characters,
        items,
        audio,
    })
}

/// The supports and the camp events, each loaded or its errors.
type CampTalks = (
    Result<SupportTable, Vec<ContentError>>,
    Result<Vec<CampEvent>, Vec<ContentError>>,
);

/// What camp plays: the supports and the camp events, checked against
/// the `characters`, the dialogue `scenes` and the `battles` that loaded.
fn load_camp_talks(
    characters: Option<&CharacterTable>,
    scenes: Option<&DialogueTable>,
    battles: Option<&BTreeMap<String, BattleDef>>,
) -> CampTalks {
    let supports = support::load(characters, scenes);
    let camp = camp::load(camp::CampRefs {
        characters,
        dialogue: scenes,
        battles,
    });
    (supports, camp)
}

/// The portraits in `images`; none if the images failed to load.
fn load_portraits(
    images: Option<&ImageTable>,
) -> Result<BTreeMap<String, Portrait>, Vec<ContentError>> {
    images.map_or_else(|| Ok(BTreeMap::new()), portrait::load_all)
}

/// The lessons, checked against `battles` and the guide's portrait in
/// `faces`. No battle is checked when there are none: the battles load
/// empty, without an error, when a file they need failed.
fn load_lessons(
    battles: Option<&BTreeMap<String, BattleDef>>,
    faces: Option<&BTreeMap<String, Portrait>>,
) -> Result<LessonTable, Vec<ContentError>> {
    let battles = battles.filter(|b| !b.is_empty());
    lesson::load(lesson::Refs { battles, faces })
}

/// `content` with the colour themes added, or the errors of both.
fn with_themes(
    content: Result<Content, ContentErrors>,
    themes: Result<BTreeMap<Theme, PaletteDef>, Vec<ContentError>>,
) -> Result<Content, ContentErrors> {
    match (content, themes) {
        (Ok(mut content), Ok(themes)) => {
            content.themes = themes;
            Ok(content)
        }
        (Ok(_), Err(errors)) => Err(ContentErrors(errors)),
        (Err(mut content), Err(errors)) => {
            content.0.extend(errors);
            Err(content)
        }
        (Err(content), Ok(_)) => Err(content),
    }
}

/// The English a language pack's data and dialogue text is checked
/// against ([`lang::DataText`]); `None` if a file it is read from failed
/// to load. The story files load empty, without an error, when a file
/// they depend on failed (the maps, the characters, the audio): their
/// text is then no base to check a pack on either.
fn data_text(
    units: &Loaded,
    terrain: Option<&TerrainDef>,
    maps: Option<&BTreeMap<String, MapDef>>,
) -> Option<lang::DataText> {
    maps?;
    units.characters.as_ref().ok()?;
    units.audio.as_ref().ok()?;
    Some(lang::from_tables(&lang::Tables {
        classes: units.classes.as_ref().ok()?,
        items: units.items.as_ref().ok()?,
        spells: units.spells.as_ref().ok()?,
        skills: units.skills.as_ref().ok()?,
        arts: units.arts.as_ref().ok()?,
        terrain: terrain?,
        names: units.names.as_ref().ok()?,
        tips: units.tips.as_ref().ok()?,
        lessons: units.lessons.as_ref().ok()?,
        chapters: units.chapters.as_ref().ok()?,
        battles: units.battles.as_ref().ok()?,
        dialogue: units.dialogue.as_ref().ok()?,
        camp: units.camp.as_ref().ok()?,
        worldmaps: units.worldmaps.as_ref().ok()?,
    }))
}

/// The battles, chapters and New Game file, each if it loaded.
type StoryRefs<'a> = (
    Option<&'a BTreeMap<String, BattleDef>>,
    Option<&'a BTreeMap<String, ChapterDef>>,
    Option<&'a NewGameDef>,
);

/// The maps and the openings, each if it loaded.
type PlaceFiles<'a> = (
    Option<&'a BTreeMap<String, MapDef>>,
    Option<&'a BTreeMap<String, OpeningDef>>,
);

/// Adds the checks that need the maps, the openings and the story to the
/// dialogue's result: the places and openings ([`check_places`]), then who
/// is there ([`check_presence`]).
fn check_dialogue(
    scripts: Result<dialogue::Scripts, Vec<ContentError>>,
    (maps, openings): PlaceFiles,
    (battles, chapters, new_game): StoryRefs,
    (supports, camp): AwayRefs,
) -> Result<DialogueTable, Vec<ContentError>> {
    let places = dialogue::PlaceRefs::new(maps, battles, chapters, supports);
    let places = places.with_openings(openings).with_camp(camp);
    let scripts = check_places(scripts, &places);
    check_presence(scripts, battles, chapters, new_game, (supports, camp))
}

/// The supports and the camp events, each if it loaded: the scenes played
/// at camp, away from any battle.
type AwayRefs<'a> = (Option<&'a SupportTable>, Option<&'a Vec<CampEvent>>);

/// Adds the place checks ([`dialogue::check_places`]: every `@place` line's
/// map and tile are there, `@place battle` only in a battle's scenes) to
/// the dialogue's result.
fn check_places(
    scripts: Result<dialogue::Scripts, Vec<ContentError>>,
    places: &dialogue::PlaceRefs,
) -> Result<dialogue::Scripts, Vec<ContentError>> {
    let scripts = scripts?;
    let errors: Vec<ContentError> = scripts
        .parsed
        .iter()
        .flat_map(|scene| dialogue::check_places(scene, places))
        .collect();
    if errors.is_empty() {
        Ok(scripts)
    } else {
        Err(errors)
    }
}

/// Adds the presence checks ([`dialogue::check_presence`]: nobody who may
/// be gone is shown outside an `@if` block for them, ADR-0055) to the
/// dialogue's result. Skipped when the dialogue, the battles, the
/// chapters, the New Game file, the supports or the camp events failed to
/// load.
fn check_presence(
    scripts: Result<dialogue::Scripts, Vec<ContentError>>,
    battles: Option<&BTreeMap<String, BattleDef>>,
    chapters: Option<&BTreeMap<String, ChapterDef>>,
    new_game: Option<&NewGameDef>,
    (supports, camp): AwayRefs,
) -> Result<DialogueTable, Vec<ContentError>> {
    let scripts = scripts?;
    let (Some(battles), Some(chapters), Some(new_game), Some(supports), Some(camp)) =
        (battles, chapters, new_game, supports, camp)
    else {
        return Ok(scripts.table);
    };
    let joins = scripts
        .parsed
        .iter()
        .map(|scene| (scene.scene.id.clone(), dialogue::joined(scene)))
        .collect();
    let cast = dialogue::Cast::new(new_game, battles, chapters, supports)
        .with_camp(camp)
        .with_joins(&joins, battles, chapters);
    let errors: Vec<ContentError> = scripts
        .parsed
        .iter()
        .flat_map(|scene| dialogue::check_presence(scene, &cast))
        .collect();
    if errors.is_empty() {
        Ok(scripts.table)
    } else {
        Err(errors)
    }
}

/// The items, their seals checked against `classes`, and the basic shop,
/// checked against the items.
fn load_items(
    classes: Option<&ClassTable>,
) -> (
    Result<ItemTable, Vec<ContentError>>,
    Result<BasicStock, Vec<ContentError>>,
) {
    let items = check_seals(item::load(), classes);
    let basic_shop = basic_shop::load(items.as_ref().ok());
    (items, basic_shop)
}

/// Adds the seal checks ([`item::check_seals`]: a seal for every tier a
/// class promotes into, and a Reclass Seal) to the items' result. Skipped
/// when the items or classes failed to load.
fn check_seals(
    items: Result<ItemTable, Vec<ContentError>>,
    classes: Option<&ClassTable>,
) -> Result<ItemTable, Vec<ContentError>> {
    match (items, classes) {
        (Ok(items), Some(classes)) => {
            let errors = item::check_seals(&items, classes);
            if errors.is_empty() {
                Ok(items)
            } else {
                Err(errors)
            }
        }
        (items, _) => items,
    }
}

/// Loads the tilesets, checked against the images, terrain, classes and
/// characters. Skipped (none, no errors) when one of those failed.
fn load_tilesets(
    images: Option<&ImageTable>,
    terrain: Option<&TerrainDef>,
    classes: Option<&ClassTable>,
    characters: Option<&CharacterTable>,
) -> Result<BTreeMap<String, Tileset>, Vec<ContentError>> {
    let (Some(images), Some(terrain), Some(classes), Some(characters)) =
        (images, terrain, classes, characters)
    else {
        return Ok(BTreeMap::new());
    };
    tileset::load_all(&tileset::TilesetRefs {
        images,
        terrain: &terrain.display,
        classes,
        characters,
    })
}

/// Loader results for the battles, chapters and New Game file.
type Story = (
    Result<BTreeMap<String, BattleDef>, Vec<ContentError>>,
    Result<BTreeMap<String, ChapterDef>, Vec<ContentError>>,
    Result<NewGameDef, Vec<ContentError>>,
);

/// Loads the battles, then the chapters, then the New Game file, each
/// checked against what it refers to. Each is skipped (empty, no errors)
/// when a file it depends on failed, so one broken file doesn't flood the
/// report.
fn load_story(
    maps: Option<&BTreeMap<String, MapDef>>,
    terrain: Option<&TerrainDef>,
    classes: Option<&ClassTable>,
    items: Option<&ItemTable>,
    characters: Option<&CharacterTable>,
    dialogue: Option<&DialogueTable>,
    audio: Option<&AudioManifest>,
) -> Story {
    let (
        Some(maps),
        Some(terrain),
        Some(classes),
        Some(items),
        Some(characters),
        Some(dialogue),
        Some(audio),
    ) = (maps, terrain, classes, items, characters, dialogue, audio)
    else {
        return (
            Ok(BTreeMap::new()),
            Ok(BTreeMap::new()),
            Ok(NewGameDef::default()),
        );
    };
    let refs = BattleRefs {
        maps,
        terrain: &terrain.rules,
        terrain_looks: &terrain.display,
        classes,
        items,
        characters,
        dialogue,
        audio,
    };
    let battles = battle::load_all(&refs);
    let Ok(loaded) = &battles else {
        return (battles, Ok(BTreeMap::new()), Ok(NewGameDef::default()));
    };
    let chapters = chapter::load_all(loaded, dialogue);
    let new_game = match &chapters {
        Ok(chapters) => chapter::load_new_game(chapters, characters, items),
        Err(_) => Ok(NewGameDef::default()),
    };
    (battles, chapters, new_game)
}

/// Adds the art reference checks ([`art::check_references`]: every
/// weapon's arts exist, are weapon arts and match its kind) to the arts'
/// result. Skipped when the arts or items failed to load.
fn check_art_references(
    arts: Result<ArtTable, Vec<ContentError>>,
    items: Option<&ItemTable>,
) -> Result<ArtTable, Vec<ContentError>> {
    match (arts, items) {
        (Ok(arts), Some(items)) => {
            let errors = art::check_references(&arts, items);
            if errors.is_empty() {
                Ok(arts)
            } else {
                Err(errors)
            }
        }
        (arts, _) => arts,
    }
}

/// Adds the skill reference checks ([`skill::check_references`]: every
/// class active and passive exists, of the right kind) to the skills'
/// result. Skipped when the skills or classes failed to load.
fn check_skill_references(
    skills: Result<SkillTable, Vec<ContentError>>,
    classes: Option<&ClassTable>,
) -> Result<SkillTable, Vec<ContentError>> {
    match (skills, classes) {
        (Ok(skills), Some(classes)) => {
            let errors = skill::check_references(&skills, classes);
            if errors.is_empty() {
                Ok(skills)
            } else {
                Err(errors)
            }
        }
        (skills, _) => skills,
    }
}

/// Adds the spell reference checks ([`spell::check_references`]: every class
/// and personal spell exists) to the spells' result. Skipped when the
/// spells, classes or characters failed to load.
fn check_spell_references(
    spells: Result<SpellTable, Vec<ContentError>>,
    classes: Option<&ClassTable>,
    characters: Option<&CharacterTable>,
) -> Result<SpellTable, Vec<ContentError>> {
    match (spells, classes, characters) {
        (Ok(spells), Some(classes), Some(characters)) => {
            let errors = spell::check_references(&spells, classes, characters);
            if errors.is_empty() {
                Ok(spells)
            } else {
                Err(errors)
            }
        }
        (spells, ..) => spells,
    }
}

/// Adds the map feature checks ([`map::check_features`]) to the maps'
/// result. Skipped when the maps, items or terrain failed to load.
fn check_map_features(
    maps: Result<BTreeMap<String, MapDef>, Vec<ContentError>>,
    items: Option<&ItemTable>,
    terrain: Option<&TerrainDef>,
) -> Result<BTreeMap<String, MapDef>, Vec<ContentError>> {
    match (maps, items, terrain) {
        (Ok(maps), Some(items), Some(terrain)) => {
            let errors = map::check_features(&maps, items, &terrain.rules);
            if errors.is_empty() {
                Ok(maps)
            } else {
                Err(errors)
            }
        }
        (maps, ..) => maps,
    }
}

/// Adds the light check ([`map::check_lights`]: every map's light is one
/// of the lights' file) to the maps' result. Skipped when the maps or the
/// lights failed to load.
fn check_map_lights(
    maps: Result<BTreeMap<String, MapDef>, Vec<ContentError>>,
    lighting: Option<&Lighting>,
) -> Result<BTreeMap<String, MapDef>, Vec<ContentError>> {
    match (maps, lighting) {
        (Ok(maps), Some(lighting)) => {
            let errors = map::check_lights(&maps, lighting);
            if errors.is_empty() {
                Ok(maps)
            } else {
                Err(errors)
            }
        }
        (maps, _) => maps,
    }
}

/// The maps ([`map::load_all`]): none if the terrain failed to load.
fn load_maps(terrain: Option<&TerrainDef>) -> Result<BTreeMap<String, MapDef>, Vec<ContentError>> {
    terrain.map_or_else(|| Ok(BTreeMap::new()), |_| map::load_all())
}

/// The light each battle names instead of its map's
/// ([`battle::load_lights`]): none if the lights failed to load.
fn load_battle_lights(
    lighting: Option<&Lighting>,
) -> Result<BTreeMap<String, String>, Vec<ContentError>> {
    lighting.map_or_else(|| Ok(BTreeMap::new()), battle::load_lights)
}

/// The combat pictures and backdrops (ADR-0056): none if the images failed
/// to load (their errors are reported once, by the image table). The
/// backdrops' terrain and lights are checked against `terrain` and
/// `lighting`, each if it loaded.
fn load_battlers(
    images: Option<&ImageTable>,
    terrain: Option<&TerrainDef>,
    lighting: Option<&Lighting>,
) -> Result<BattlerTable, Vec<ContentError>> {
    match images {
        Some(images) => battler::load_all(images, terrain.map(|t| &t.display), lighting),
        None => Ok(BattlerTable::default()),
    }
}

/// The title screen's art (ADR-0059): none if the images failed to load
/// (their errors are reported once, by the image table).
fn load_title(images: Option<&ImageTable>) -> Result<TitleArt, Vec<ContentError>> {
    match images {
        Some(images) => title::load(images),
        None => Ok(TitleArt::default()),
    }
}

/// The cinematics, checked against the battles, the music, the scenes and
/// the characters (`people`: the characters, their portraits, the names)
/// they name, and their actions against the tiles a unit can walk on
/// (`rules`: the classes and the terrain). Skipped (none, no errors) when
/// one of those failed, or when there are no battles: they failed, or
/// were skipped because what they need failed.
fn load_cinematics(
    battles: Option<&BTreeMap<String, BattleDef>>,
    audio: Option<&AudioManifest>,
    scenes: Option<&DialogueTable>,
    people: People<'_>,
    rules: Rules<'_>,
) -> Result<BTreeMap<String, CinematicDef>, Vec<ContentError>> {
    let battles = battles.filter(|b| !b.is_empty());
    let (Some(battles), Some(audio), Some(scenes)) = (battles, audio, scenes) else {
        return Ok(BTreeMap::new());
    };
    let (Some(characters), Some(portraits), Some(names)) = people else {
        return Ok(BTreeMap::new());
    };
    let (Ok(classes), Ok(terrain)) = rules else {
        return Ok(BTreeMap::new());
    };
    cinematic::load_all(cinematic::Refs {
        battles,
        audio,
        scenes,
        characters,
        portraits,
        names,
        classes,
        terrain: &terrain.rules,
    })
}

/// The openings ([`cinematic::load_openings`]): none when the maps, the
/// characters, the classes or the terrain failed to load.
fn load_openings(
    maps: Option<&BTreeMap<String, MapDef>>,
    characters: Option<&CharacterTable>,
    (classes, terrain): Rules<'_>,
) -> Result<BTreeMap<String, OpeningDef>, Vec<ContentError>> {
    let (Some(maps), Some(characters), Ok(classes), Ok(terrain)) =
        (maps, characters, classes, terrain)
    else {
        return Ok(BTreeMap::new());
    };
    cinematic::load_openings(cinematic::OpeningRefs {
        maps,
        characters,
        classes,
        terrain: &terrain.rules,
    })
}

/// The classes and the terrain as their loaders left them.
type Rules<'a> = (
    &'a Result<ClassTable, Vec<ContentError>>,
    &'a Result<TerrainDef, Vec<ContentError>>,
);

/// The characters, their portraits and the names table, each if it loaded.
type People<'a> = (
    Option<&'a CharacterTable>,
    Option<&'a BTreeMap<String, Portrait>>,
    Option<&'a Names>,
);

/// Loader results for the unit data (kept together to keep [`assemble`]'s
/// argument list short).
struct Loaded {
    classes: Result<ClassTable, Vec<ContentError>>,
    items: Result<ItemTable, Vec<ContentError>>,
    basic_shop: Result<BasicStock, Vec<ContentError>>,
    spells: Result<SpellTable, Vec<ContentError>>,
    skills: Result<SkillTable, Vec<ContentError>>,
    arts: Result<ArtTable, Vec<ContentError>>,
    names: Result<Names, Vec<ContentError>>,
    characters: Result<CharacterTable, Vec<ContentError>>,
    portraits: Result<BTreeMap<String, Portrait>, Vec<ContentError>>,
    dialogue: Result<DialogueTable, Vec<ContentError>>,
    ai: Result<AiWeights, Vec<ContentError>>,
    tips: Result<TipTable, Vec<ContentError>>,
    lessons: Result<LessonTable, Vec<ContentError>>,
    audio: Result<AudioManifest, Vec<ContentError>>,
    credits: Result<Credits, Vec<ContentError>>,
    images: Result<ImageTable, Vec<ContentError>>,
    tilesets: Result<BTreeMap<String, Tileset>, Vec<ContentError>>,
    battles: Result<BTreeMap<String, BattleDef>, Vec<ContentError>>,
    chapters: Result<BTreeMap<String, ChapterDef>, Vec<ContentError>>,
    new_game: Result<NewGameDef, Vec<ContentError>>,
    supports: Result<SupportTable, Vec<ContentError>>,
    camp: Result<Vec<CampEvent>, Vec<ContentError>>,
    lang: Result<Lang, Vec<ContentError>>,
    battlers: Result<BattlerTable, Vec<ContentError>>,
    lighting: Result<Lighting, Vec<ContentError>>,
    battle_lights: Result<BTreeMap<String, String>, Vec<ContentError>>,
    title: Result<TitleArt, Vec<ContentError>>,
    cinematics: Result<BTreeMap<String, CinematicDef>, Vec<ContentError>>,
    openings: Result<BTreeMap<String, OpeningDef>, Vec<ContentError>>,
    worldmaps: Result<BTreeMap<String, WorldMapDef>, Vec<ContentError>>,
}

/// Takes a loader's value, or moves its errors into `errors` and returns a
/// default.
fn take<T: Default>(result: Result<T, Vec<ContentError>>, errors: &mut Vec<ContentError>) -> T {
    result.unwrap_or_else(|e| {
        errors.extend(e);
        T::default()
    })
}

/// Combines each loader's result into [`Content`], collecting the errors of
/// every loader that failed.
fn assemble(
    palette: Result<PaletteDef, Vec<ContentError>>,
    keymap: Result<KeymapDef, Vec<ContentError>>,
    font: Result<FontAtlasDef, Vec<ContentError>>,
    terrain: Result<TerrainDef, Vec<ContentError>>,
    maps: Result<BTreeMap<String, MapDef>, Vec<ContentError>>,
    units: Loaded,
) -> Result<Content, ContentErrors> {
    let mut errors = Vec::new();
    let content = Content {
        palette: take(palette, &mut errors),
        themes: BTreeMap::new(),
        keymap: take(keymap, &mut errors),
        font: take(font, &mut errors),
        terrain: take(terrain, &mut errors),
        maps: take(maps, &mut errors),
        classes: take(units.classes, &mut errors),
        items: take(units.items, &mut errors),
        spells: take(units.spells, &mut errors),
        skills: take(units.skills, &mut errors),
        arts: take(units.arts, &mut errors),
        names: take(units.names, &mut errors),
        characters: take(units.characters, &mut errors),
        portraits: take(units.portraits, &mut errors),
        dialogue: take(units.dialogue, &mut errors),
        ai: take(units.ai, &mut errors),
        tips: take(units.tips, &mut errors),
        audio: take(units.audio, &mut errors),
        credits: take(units.credits, &mut errors),
        images: take(units.images, &mut errors),
        tilesets: take(units.tilesets, &mut errors),
        battles: take(units.battles, &mut errors),
        chapters: take(units.chapters, &mut errors),
        new_game: take(units.new_game, &mut errors),
        supports: take(units.supports, &mut errors),
        lang: Arc::new(take(units.lang, &mut errors)),
        battlers: take(units.battlers, &mut errors),
        lighting: take(units.lighting, &mut errors),
        battle_lights: take(units.battle_lights, &mut errors),
        title: take(units.title, &mut errors),
        cinematics: take(units.cinematics, &mut errors),
        lessons: take(units.lessons, &mut errors),
        openings: take(units.openings, &mut errors),
        basic_shop: take(units.basic_shop, &mut errors),
        camp: take(units.camp, &mut errors),
        worldmaps: take(units.worldmaps, &mut errors),
    };
    if errors.is_empty() {
        Ok(content)
    } else {
        Err(ContentErrors(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_terrain() -> Result<TerrainDef, Vec<ContentError>> {
        TerrainDef::load(crate::palette::load().ok().as_ref())
    }

    #[test]
    fn a_maps_look_is_its_files_until_its_terrain_changes() {
        let mut content = load_embedded().unwrap();
        let map = content.maps["test_small"].map.clone();
        // The test map names no look.
        assert_eq!(content.map_look(&map), MapLook::default());
        let indoor = MapLook {
            tiles: "indoor".to_owned(),
            ..MapLook::default()
        };
        if let Some(def) = content.maps.get_mut("test_small") {
            def.look = indoor.clone();
        }
        assert_eq!(content.map_look(&map), indoor);
        // A map that is no file's (here: another name) has the look of a
        // map that names none.
        let mut other = map;
        other.name = "Elsewhere".to_owned();
        assert_eq!(content.map_look(&other), MapLook::default());
    }

    #[test]
    fn a_battles_look_is_its_maps_in_its_own_light_if_it_names_one() {
        let mut content = load_embedded().unwrap();
        let battle = content.battles["test"].clone();
        let dusk = MapLook {
            tiles: "indoor".to_owned(),
            light: "dusk".to_owned(),
        };
        for def in content.maps.values_mut().filter(|d| d.map == battle.map) {
            def.look = dusk.clone();
        }
        // The test battle's file names noon: fought at noon on a dusk map.
        let noon = Some("noon".to_owned());
        assert_eq!(content.battle_lights.get("test"), noon.as_ref());
        let look = content.battle_look(&battle);
        assert_eq!(
            (look.tiles.as_str(), look.light.as_str()),
            ("indoor", "noon")
        );
        // No light of its own: its map's.
        assert_eq!(content.battle_lights.remove("test"), noon);
        assert_eq!(content.battle_look(&battle), dusk);
        // Its own light, on its map's tiles; another battle's is not its.
        content
            .battle_lights
            .insert("other".to_owned(), "noon".to_owned());
        assert_eq!(content.battle_look(&battle), dusk);
        content
            .battle_lights
            .insert("test".to_owned(), "night".to_owned());
        let look = content.battle_look(&battle);
        assert_eq!(
            (look.tiles.as_str(), look.light.as_str()),
            ("indoor", "night")
        );
    }

    #[test]
    fn battle_lights_are_read_when_the_lights_loaded() {
        let lights = lighting::load().ok();
        let read = load_battle_lights(lights.as_ref()).unwrap();
        assert_eq!(read.get("test").map(String::as_str), Some("noon"));
        assert_eq!(load_battle_lights(None), Ok(BTreeMap::new()));
    }

    #[test]
    fn the_shipped_maps_and_battles_name_lights_that_exist() {
        let content = load_embedded().unwrap();
        for def in content.maps.values() {
            assert!(content.lighting.get(&def.look.light).is_some());
        }
        for light in content.battle_lights.values() {
            assert!(content.lighting.get(light).is_some());
        }
        // The test map is at noon.
        assert_eq!(content.maps["test_small"].look.light, lighting::NOON);
    }

    #[test]
    fn a_maps_light_is_checked_when_the_maps_and_the_lights_loaded() {
        let lights = lighting::load().ok();
        let odd = || {
            let mut maps = ok_maps().unwrap_or_default();
            if let Some(def) = maps.get_mut("test_small") {
                def.look.light = "midnight".to_owned();
            }
            maps
        };
        assert_eq!(check_map_lights(ok_maps(), lights.as_ref()), ok_maps());
        let errors = check_map_lights(Ok(odd()), lights.as_ref())
            .err()
            .unwrap_or_default();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "assets-test/maps/test_small.map");
        assert!(errors[0].message.contains("\"midnight\""), "{errors:?}");
        // Unchecked without the lights; the maps' own errors pass through.
        assert_eq!(check_map_lights(Ok(odd()), None), Ok(odd()));
        let bad = || Err(vec![ContentError::new("m", "bad")]);
        assert_eq!(check_map_lights(bad(), lights.as_ref()), bad());
    }

    fn ok_maps() -> Result<BTreeMap<String, MapDef>, Vec<ContentError>> {
        map::load_all()
    }

    fn ok_classes() -> Result<ClassTable, Vec<ContentError>> {
        class::load(Some(&ok_terrain().unwrap_or_default().rules.movement_types))
    }

    fn ok_characters() -> Result<CharacterTable, Vec<ContentError>> {
        character::load(
            ok_classes().ok().as_ref(),
            item::load().ok().as_ref(),
            names::load().ok().as_ref(),
        )
    }

    fn ok_spells() -> Result<SpellTable, Vec<ContentError>> {
        spell::load(Some(&ok_terrain().unwrap_or_default().display))
    }

    fn ok_skills() -> Result<SkillTable, Vec<ContentError>> {
        skill::load()
    }

    fn ok_battlers() -> Result<BattlerTable, Vec<ContentError>> {
        battler::load_all(&crate::image::load().unwrap_or_default(), None, None)
    }

    fn ok_title() -> Result<TitleArt, Vec<ContentError>> {
        title::load(&crate::image::load().unwrap_or_default())
    }

    fn ok_openings() -> Result<BTreeMap<String, OpeningDef>, Vec<ContentError>> {
        let (maps, cast) = (load_maps(ok_terrain().as_ref().ok()), ok_characters());
        let (classes, terrain) = (ok_classes(), ok_terrain());
        load_openings(maps.as_ref().ok(), cast.as_ref().ok(), (&classes, &terrain))
    }

    fn ok_cinematics() -> Result<BTreeMap<String, CinematicDef>, Vec<ContentError>> {
        let faces = ok_portraits().ok();
        let (classes, terrain) = (ok_classes(), ok_terrain());
        load_cinematics(
            ok_story().0.ok().as_ref(),
            audio::load().ok().as_ref(),
            ok_dialogue().ok().as_ref(),
            (
                ok_characters().ok().as_ref(),
                faces.as_ref(),
                names::load().ok().as_ref(),
            ),
            (&classes, &terrain),
        )
    }

    fn ok_tips() -> Result<TipTable, Vec<ContentError>> {
        tip::load(ok_portraits().ok().as_ref())
    }

    fn ok_lessons() -> Result<LessonTable, Vec<ContentError>> {
        let (battles, faces) = (ok_story().0.ok(), ok_portraits().ok());
        lesson::load(lesson::Refs {
            battles: battles.as_ref(),
            faces: faces.as_ref(),
        })
    }

    fn ok_portraits() -> Result<BTreeMap<String, Portrait>, Vec<ContentError>> {
        portrait::load_all(&crate::image::load().unwrap_or_default())
    }

    fn ok_dialogue() -> Result<DialogueTable, Vec<ContentError>> {
        dialogue::load(
            ok_characters().ok().as_ref(),
            ok_portraits().ok().as_ref(),
            names::load().ok().as_ref(),
            audio::load().ok().as_ref(),
        )
    }

    fn ok_story() -> Story {
        let terrain = ok_terrain().ok();
        load_story(
            ok_maps().ok().as_ref(),
            terrain.as_ref(),
            ok_classes().ok().as_ref(),
            item::load().ok().as_ref(),
            ok_characters().ok().as_ref(),
            ok_dialogue().ok().as_ref(),
            audio::load().ok().as_ref(),
        )
    }

    fn ok_credits() -> Result<Credits, Vec<ContentError>> {
        credits::load(audio::load().ok().as_ref())
    }

    fn ok_supports() -> Result<SupportTable, Vec<ContentError>> {
        support::load(ok_characters().ok().as_ref(), ok_dialogue().ok().as_ref())
    }

    fn ok_camp() -> Result<Vec<CampEvent>, Vec<ContentError>> {
        let (battles, ..) = ok_story();
        camp::load(camp::CampRefs {
            characters: ok_characters().ok().as_ref(),
            dialogue: ok_dialogue().ok().as_ref(),
            battles: battles.ok().as_ref(),
        })
    }

    fn ok_tilesets() -> Result<BTreeMap<String, Tileset>, Vec<ContentError>> {
        load_tilesets(
            crate::image::load().ok().as_ref(),
            ok_terrain().ok().as_ref(),
            ok_classes().ok().as_ref(),
            ok_characters().ok().as_ref(),
        )
    }

    fn ok_units() -> Loaded {
        let (battles, chapters, new_game) = ok_story();
        Loaded {
            classes: ok_classes(),
            items: item::load(),
            basic_shop: basic_shop::load(None),
            spells: ok_spells(),
            skills: ok_skills(),
            arts: art::load(),
            names: names::load(),
            characters: ok_characters(),
            portraits: ok_portraits(),
            dialogue: ok_dialogue(),
            ai: ai::load(),
            tips: ok_tips(),
            lessons: ok_lessons(),
            audio: audio::load(),
            credits: ok_credits(),
            images: crate::image::load(),
            tilesets: ok_tilesets(),
            battles,
            chapters,
            new_game,
            supports: ok_supports(),
            lang: lang::load(None),
            battlers: ok_battlers(),
            lighting: lighting::load(),
            battle_lights: Ok(BTreeMap::new()),
            title: ok_title(),
            cinematics: ok_cinematics(),
            openings: ok_openings(),
            worldmaps: Ok(BTreeMap::new()),
            camp: ok_camp(),
        }
    }

    #[test]
    fn assemble_ok_keeps_content() {
        let content = assemble(
            crate::palette::load(),
            keymap::load_keymap(),
            crate::font::load(),
            ok_terrain(),
            ok_maps(),
            ok_units(),
        )
        .ok();
        assert_eq!(
            content.as_ref().map(|c| &c.palette),
            crate::palette::load().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.keymap),
            keymap::load_keymap().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.font),
            crate::font::load().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.terrain),
            ok_terrain().ok().as_ref()
        );
        assert_eq!(content.as_ref().map(|c| &c.maps), ok_maps().ok().as_ref());
        assert_eq!(
            content.as_ref().map(|c| &c.classes),
            ok_classes().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.items),
            item::load().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.spells),
            ok_spells().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.skills),
            ok_skills().ok().as_ref()
        );
        assert_eq!(content.as_ref().map(|c| &c.arts), art::load().ok().as_ref());
        assert_eq!(
            content.as_ref().map(|c| &c.names),
            names::load().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.characters),
            ok_characters().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.portraits),
            ok_portraits().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.dialogue),
            ok_dialogue().ok().as_ref()
        );
        assert_eq!(content.as_ref().map(|c| &c.ai), ai::load().ok().as_ref());
        assert_eq!(
            content.as_ref().map(|c| &c.audio),
            audio::load().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.credits),
            ok_credits().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.images),
            crate::image::load().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.tilesets),
            ok_tilesets().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.supports),
            ok_supports().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &*c.lang),
            lang::load(None).ok().as_ref()
        );
        assert!(
            content.as_ref().is_some_and(
                |c| c.maps.contains_key("test_small") && c.dialogue.get("test").is_some()
            )
        );
        assert!(
            content.is_some_and(
                |c| !c.classes.classes.is_empty() && !c.characters.characters.is_empty()
            )
        );
    }

    const NAMES: [&str; 35] = [
        "p", "k", "f", "t", "m", "c", "i", "s", "x", "a", "n", "u", "o", "d", "w", "y", "v", "r",
        "j", "z", "b", "h", "g", "q", "l", "A", "L", "B", "T", "C", "E", "O", "S", "Q", "W",
    ];

    /// `ok`, or the error of file `name` if `bad`.
    fn pick<T>(
        bad: bool,
        name: &str,
        ok: Result<T, Vec<ContentError>>,
    ) -> Result<T, Vec<ContentError>> {
        if bad {
            Err(vec![ContentError::new(name, "bad")])
        } else {
            ok
        }
    }

    #[test]
    fn assemble_reports_loader_errors() {
        let e = |f: &str| vec![ContentError::new(f, "bad")];
        assert_eq!(
            assemble(
                Err(e("p")),
                Err(e("k")),
                Err(e("f")),
                Err(e("t")),
                Err(e("m")),
                Loaded {
                    classes: Err(e("c")),
                    items: Err(e("i")),
                    basic_shop: Err(e("S")),
                    spells: Err(e("s")),
                    skills: Err(e("x")),
                    arts: Err(e("a")),
                    names: Err(e("n")),
                    characters: Err(e("u")),
                    portraits: Err(e("o")),
                    dialogue: Err(e("d")),
                    ai: Err(e("w")),
                    tips: Err(e("y")),
                    lessons: Err(e("E")),
                    audio: Err(e("v")),
                    credits: Err(e("r")),
                    images: Err(e("j")),
                    tilesets: Err(e("z")),
                    battles: Err(e("b")),
                    chapters: Err(e("h")),
                    new_game: Err(e("g")),
                    supports: Err(e("q")),
                    lang: Err(e("l")),
                    battlers: Err(e("A")),
                    lighting: Err(e("L")),
                    battle_lights: Err(e("B")),
                    title: Err(e("T")),
                    cinematics: Err(e("C")),
                    openings: Err(e("O")),
                    camp: Err(e("Q")),
                    worldmaps: Err(e("W")),
                },
            ),
            Err(ContentErrors(NAMES.iter().flat_map(|f| e(f)).collect()))
        );
    }

    #[test]
    fn assemble_reports_each_loaders_errors_alone() {
        let e = |f: &str| vec![ContentError::new(f, "bad")];
        let only = |i: usize| {
            assemble(
                if i == 0 {
                    Err(e("p"))
                } else {
                    crate::palette::load()
                },
                if i == 1 {
                    Err(e("k"))
                } else {
                    keymap::load_keymap()
                },
                if i == 2 {
                    Err(e("f"))
                } else {
                    crate::font::load()
                },
                if i == 3 { Err(e("t")) } else { ok_terrain() },
                if i == 4 { Err(e("m")) } else { ok_maps() },
                Loaded {
                    classes: if i == 5 { Err(e("c")) } else { ok_classes() },
                    items: if i == 6 { Err(e("i")) } else { item::load() },
                    spells: if i == 7 { Err(e("s")) } else { ok_spells() },
                    skills: if i == 8 { Err(e("x")) } else { ok_skills() },
                    arts: if i == 9 { Err(e("a")) } else { art::load() },
                    names: if i == 10 { Err(e("n")) } else { names::load() },
                    characters: if i == 11 {
                        Err(e("u"))
                    } else {
                        ok_characters()
                    },
                    portraits: if i == 12 { Err(e("o")) } else { ok_portraits() },
                    dialogue: if i == 13 { Err(e("d")) } else { ok_dialogue() },
                    ai: if i == 14 { Err(e("w")) } else { ai::load() },
                    tips: if i == 15 { Err(e("y")) } else { ok_tips() },
                    audio: if i == 16 { Err(e("v")) } else { audio::load() },
                    credits: if i == 17 { Err(e("r")) } else { ok_credits() },
                    images: if i == 18 {
                        Err(e("j"))
                    } else {
                        crate::image::load()
                    },
                    tilesets: if i == 19 { Err(e("z")) } else { ok_tilesets() },
                    battles: if i == 20 { Err(e("b")) } else { ok_story().0 },
                    chapters: if i == 21 { Err(e("h")) } else { ok_story().1 },
                    new_game: if i == 22 { Err(e("g")) } else { ok_story().2 },
                    supports: if i == 23 { Err(e("q")) } else { ok_supports() },
                    lang: pick(i == 24, "l", lang::load(None)),
                    battlers: if i == 25 { Err(e("A")) } else { ok_battlers() },
                    lighting: pick(i == 26, "L", lighting::load()),
                    battle_lights: pick(i == 27, "B", Ok(BTreeMap::new())),
                    title: pick(i == 28, "T", ok_title()),
                    cinematics: pick(i == 29, "C", ok_cinematics()),
                    lessons: pick(i == 30, "E", ok_lessons()),
                    openings: pick(i == 31, "O", ok_openings()),
                    basic_shop: pick(i == 32, "S", basic_shop::load(None)),
                    camp: pick(i == 33, "Q", ok_camp()),
                    worldmaps: pick(i == 34, "W", Ok(BTreeMap::new())),
                },
            )
        };
        for (i, f) in NAMES.iter().enumerate() {
            assert_eq!(only(i), Err(ContentErrors(e(f))));
        }
    }

    #[test]
    fn the_cinematics_are_skipped_without_what_they_name() {
        let (battles, audio) = (ok_story().0.ok(), audio::load().ok());
        let (scenes, cast) = (ok_dialogue().ok(), ok_characters().ok());
        let (faces, names) = (ok_portraits().ok(), names::load().ok());
        let (classes, terrain, failed) = (ok_classes(), ok_terrain(), Err(Vec::new()));
        let (b, a, s) = (battles.as_ref(), audio.as_ref(), scenes.as_ref());
        let people = (cast.as_ref(), faces.as_ref(), names.as_ref());
        let rules = (&classes, &terrain);
        let none = BTreeMap::new();
        let skipped = [
            load_cinematics(None, a, s, people, rules),
            load_cinematics(Some(&none), a, s, people, rules),
            load_cinematics(b, None, s, people, rules),
            load_cinematics(b, a, None, people, rules),
            load_cinematics(b, a, s, (None, people.1, people.2), rules),
            load_cinematics(b, a, s, (people.0, None, people.2), rules),
            load_cinematics(b, a, s, (people.0, people.1, None), rules),
            load_cinematics(b, a, s, people, (&Err(Vec::new()), rules.1)),
            load_cinematics(b, a, s, people, (rules.0, &failed)),
        ];
        for loaded in skipped {
            assert_eq!(loaded, Ok(BTreeMap::new()));
        }
        let loaded = load_cinematics(b, a, s, people, rules);
        assert!(loaded.is_ok_and(|c| c.contains_key("test")));
    }

    #[test]
    fn the_title_art_is_skipped_when_the_images_failed() {
        assert_eq!(load_title(None), Ok(TitleArt::default()));
        let images = crate::image::load().ok();
        assert!(load_title(images.as_ref()).is_ok_and(|t| t.logo.is_some()));
    }

    #[test]
    fn battlers_are_skipped_when_the_images_failed() {
        assert_eq!(load_battlers(None, None, None), Ok(BattlerTable::default()));
        let images = crate::image::load().ok();
        let lights = lighting::load().ok();
        let loaded = load_battlers(images.as_ref(), ok_terrain().ok().as_ref(), lights.as_ref());
        assert!(loaded.is_ok_and(|t| !t.battlers.is_empty()));
    }

    #[test]
    fn tilesets_are_skipped_when_what_they_need_failed() {
        let (images, terrain) = (crate::image::load().ok(), ok_terrain().ok());
        let (classes, characters) = (ok_classes().ok(), ok_characters().ok());
        let all = [
            images.is_some(),
            terrain.is_some(),
            classes.is_some(),
            characters.is_some(),
        ];
        assert_eq!(all, [true; 4]);
        for missing in 0..4 {
            let loaded = load_tilesets(
                images.as_ref().filter(|_| missing != 0),
                terrain.as_ref().filter(|_| missing != 1),
                classes.as_ref().filter(|_| missing != 2),
                characters.as_ref().filter(|_| missing != 3),
            );
            assert_eq!(loaded, Ok(BTreeMap::new()), "{missing}");
        }
        let loaded = load_tilesets(
            images.as_ref(),
            terrain.as_ref(),
            classes.as_ref(),
            characters.as_ref(),
        );
        assert!(loaded.is_ok_and(|t| t.contains_key("test")));
    }

    /// A pack's data and dialogue text is checked only when every file
    /// its English comes from loaded, the ones the story files need too.
    #[test]
    fn the_english_of_the_data_needs_every_file_it_comes_from() {
        fn failed<T>() -> Result<T, Vec<ContentError>> {
            Err(vec![ContentError::new("f", "broken")])
        }
        let terrain = ok_terrain().ok();
        let maps = ok_maps().ok();
        let text = |units: &Loaded| data_text(units, terrain.as_ref(), maps.as_ref());
        let all = text(&ok_units()).expect("everything loaded");
        assert_eq!(
            all.data.get("names.lead").map(String::as_str),
            Some("Ellery")
        );
        assert!(all.lines.contains_key("test_5da5d147"));
        let mut units = ok_units();
        units.characters = failed();
        assert_eq!(text(&units), None);
        let mut units = ok_units();
        units.audio = failed();
        assert_eq!(text(&units), None);
        let mut units = ok_units();
        units.tips = failed();
        assert_eq!(text(&units), None);
        let mut units = ok_units();
        units.lessons = failed();
        assert_eq!(text(&units), None);
        let units = ok_units();
        assert_eq!(data_text(&units, terrain.as_ref(), None), None);
        assert_eq!(data_text(&units, None, maps.as_ref()), None);
    }

    #[test]
    fn story_files_are_skipped_when_what_they_need_failed() {
        let (battles, chapters, new_game) = load_story(None, None, None, None, None, None, None);
        assert_eq!(battles, Ok(BTreeMap::new()));
        assert_eq!(chapters, Ok(BTreeMap::new()));
        assert_eq!(new_game, Ok(NewGameDef::default()));
        // Battles name their music: no audio manifest, no battles.
        let terrain = ok_terrain().ok();
        let (battles, chapters, new_game) = load_story(
            ok_maps().ok().as_ref(),
            terrain.as_ref(),
            ok_classes().ok().as_ref(),
            item::load().ok().as_ref(),
            ok_characters().ok().as_ref(),
            ok_dialogue().ok().as_ref(),
            None,
        );
        assert_eq!(battles, Ok(BTreeMap::new()));
        assert_eq!(chapters, Ok(BTreeMap::new()));
        assert_eq!(new_game, Ok(NewGameDef::default()));
        let (battles, chapters, new_game) = ok_story();
        assert!(battles.is_ok_and(|b| b.contains_key("test")));
        assert!(chapters.is_ok_and(|c| c.contains_key("test")));
        assert!(new_game.is_ok_and(|n| n.first_chapter == "ch01"));
    }

    #[test]
    fn map_feature_checks_join_the_map_errors() {
        let terrain = ok_terrain().ok();
        let items = item::load().ok();
        let mut maps = ok_maps().unwrap_or_default();
        assert_eq!(
            check_map_features(Ok(maps.clone()), items.as_ref(), terrain.as_ref()),
            Ok(maps.clone())
        );
        if let Some(def) = maps.get_mut("test_small") {
            def.map.features.insert(
                trpg_core::Pos::new(0, 0),
                trpg_core::TileFeature::Chest(trpg_core::Loot::Item(trpg_core::ItemId::new("x"))),
            );
        }
        let errors = check_map_features(Ok(maps.clone()), items.as_ref(), terrain.as_ref());
        assert_eq!(errors.map_err(|e| e.len()), Err(1));
        // Skipped when another file failed.
        assert_eq!(
            check_map_features(Ok(maps.clone()), None, terrain.as_ref()),
            Ok(maps.clone())
        );
        assert_eq!(
            check_map_features(Ok(maps.clone()), items.as_ref(), None),
            Ok(maps)
        );
        let failed = Err(vec![ContentError::new("m", "bad")]);
        assert_eq!(
            check_map_features(failed.clone(), items.as_ref(), terrain.as_ref()),
            failed
        );
    }

    #[test]
    fn spell_reference_checks_join_the_spell_errors() {
        let classes = ok_classes().ok();
        let characters = ok_characters().ok();
        let spells = ok_spells();
        assert_eq!(
            check_spell_references(spells.clone(), classes.as_ref(), characters.as_ref()),
            spells
        );
        let empty = Ok(SpellTable::default());
        let errors = check_spell_references(empty.clone(), classes.as_ref(), characters.as_ref());
        assert!(errors.is_err_and(|e| !e.is_empty()));
        // Skipped when another file failed.
        assert_eq!(
            check_spell_references(empty.clone(), None, characters.as_ref()),
            empty
        );
        assert_eq!(
            check_spell_references(empty.clone(), classes.as_ref(), None),
            empty
        );
        let failed = Err(vec![ContentError::new("s", "bad")]);
        assert_eq!(
            check_spell_references(failed.clone(), classes.as_ref(), characters.as_ref()),
            failed
        );
    }

    #[test]
    fn skill_reference_checks_join_the_skill_errors() {
        let classes = ok_classes().ok();
        let skills = ok_skills();
        assert_eq!(
            check_skill_references(skills.clone(), classes.as_ref()),
            skills
        );
        let empty = Ok(SkillTable::default());
        let errors = check_skill_references(empty.clone(), classes.as_ref());
        assert!(errors.is_err_and(|e| !e.is_empty()));
        // Skipped when another file failed.
        assert_eq!(check_skill_references(empty.clone(), None), empty);
        let failed = Err(vec![ContentError::new("x", "bad")]);
        assert_eq!(
            check_skill_references(failed.clone(), classes.as_ref()),
            failed
        );
    }

    /// Given the bundle's own dialogue files, [`load_with_scripts`] loads
    /// what [`load_embedded`] does; given others, their scenes and their
    /// problems are the game's.
    #[test]
    fn scripts_given_replace_the_bundles() {
        let paths = bundle::files_in(dialogue::DIALOGUE_DIR);
        let files: Vec<(String, &str)> = paths
            .into_iter()
            .filter(|p| p.ends_with(dialogue::DIALOGUE_EXTENSION))
            .map(|p| (bundle::display_path(p), bundle::file(p).unwrap_or_default()))
            .collect();
        assert!(files.len() > 1);
        let embedded = load_embedded().unwrap();
        let same = load_with_scripts(&files).unwrap();
        assert_eq!(same.dialogue, embedded.dialogue);
        assert_eq!(same.battles, embedded.battles);
        // A scene more.
        let mut more = files.clone();
        more.push(("draft.dlg".to_owned(), "@scene draft\n> Hello.\n@end\n"));
        let with_draft = load_with_scripts(&more).unwrap();
        assert!(with_draft.dialogue.get("draft").is_some());
        assert_eq!(
            with_draft.dialogue.scenes.len(),
            embedded.dialogue.scenes.len() + 1
        );
        // A broken script is reported under the name given.
        more.push(("bad.dlg".to_owned(), "@scene bad\n@end\n"));
        let errors = load_with_scripts(&more).unwrap_err();
        assert_eq!(
            errors.0,
            [
                ContentError::new("bad.dlg", "scene \"bad\" has no speech or narration")
                    .at(1, None)
            ]
        );
        // Without the game's scripts, what plays their scenes fails.
        let errors = load_with_scripts(&more[files.len()..=files.len()]).unwrap_err();
        assert!(errors.0.iter().any(|e| e.file.starts_with("assets/")));
    }

    /// The embedded scripts pass the presence check (ADR-0055); with the
    /// test lord in the starting roster too, his lines outside `@if`
    /// blocks are errors.
    #[test]
    fn presence_checks_join_the_dialogue_errors() {
        let scripts = || {
            dialogue::load_scripts(
                ok_characters().ok().as_ref(),
                ok_portraits().ok().as_ref(),
                names::load().ok().as_ref(),
                audio::load().ok().as_ref(),
            )
        };
        let (battles, chapters, new_game) = ok_story();
        let (battles, chapters, supports) = (battles.ok(), chapters.ok(), ok_supports().ok());
        let camp = ok_camp().ok();
        let check = |new_game: Option<&NewGameDef>| {
            check_presence(
                scripts(),
                battles.as_ref(),
                chapters.as_ref(),
                new_game,
                (supports.as_ref(), camp.as_ref()),
            )
        };
        let mut new_game = new_game.unwrap_or_default();
        assert_eq!(check(Some(&new_game)), scripts().map(|s| s.table));
        assert!(scripts().is_ok());
        new_game
            .roster
            .push(trpg_core::CharacterId("test_lord".into()));
        let errors = check(Some(&new_game)).err().unwrap_or_default();
        let first = errors.first().map(ToString::to_string);
        assert_eq!(
            first.as_deref(),
            Some(
                "assets-test/dialogue/test.dlg:10: \"test_lord\" may have fallen or left the army by \
                 now; put their lines inside \"@if test_lord\" ... \"@endif\" \
                 (assets/dialogue/README.md, \"Who is still there\")"
            )
        );
        // His support conversations and the scenes of his own triggers
        // can't play without him.
        let files: std::collections::BTreeSet<&str> =
            errors.iter().map(|e| e.file.as_str()).collect();
        assert_eq!(
            files.into_iter().collect::<Vec<_>>(),
            [
                "assets-test/dialogue/test.dlg",
                "assets-test/dialogue/test_triggers.dlg"
            ]
        );
        // Skipped when another file failed, or the dialogue did.
        assert_eq!(check(None), scripts().map(|s| s.table));
        let failed = Err(vec![ContentError::new("d", "bad")]);
        let checked = check_presence(
            failed,
            battles.as_ref(),
            chapters.as_ref(),
            Some(&new_game),
            (supports.as_ref(), camp.as_ref()),
        );
        assert_eq!(checked, Err(vec![ContentError::new("d", "bad")]));
    }

    /// A chapter's intro and victory scenes are its battle's, a trigger's
    /// scene its battle's; a support conversation is played away from
    /// any. A wrong place in a script is one of the dialogue's errors.
    #[test]
    fn place_checks_know_each_scenes_battle_and_join_the_dialogue_errors() {
        let maps = ok_maps().ok();
        let (battles, chapters, _) = ok_story();
        let (battles, chapters, supports) = (battles.ok(), chapters.ok(), ok_supports().ok());
        let refs = dialogue::PlaceRefs::new(
            maps.as_ref(),
            battles.as_ref(),
            chapters.as_ref(),
            supports.as_ref(),
        );
        assert!(refs.maps.contains_key("ch01"));
        let of = |scene: &str| -> Vec<&str> {
            let known = refs.battles.as_ref().and_then(|b| b.get(scene));
            known.into_iter().flatten().map(|(id, _)| *id).collect()
        };
        assert_eq!(of("ch01_prebattle"), ["ch01"]);
        assert_eq!(of("ch01_victory"), ["ch01"]);
        assert_eq!(of("test_intro"), ["test"]);
        assert_eq!(of("test_fort"), ["quick"]);
        assert_eq!(of("test_support_lord_knight_c"), [] as [&str; 0]);
        assert!(refs.elsewhere.contains("test_support_lord_knight_c"));
        assert!(!refs.elsewhere.contains("test_intro"));
        // Without the chapters (or with none loaded), no battle is known.
        let none = BTreeMap::new();
        let without = dialogue::PlaceRefs::new(maps.as_ref(), battles.as_ref(), Some(&none), None);
        assert!(without.battles.is_none());
        // A script's wrong place fails the load.
        let bad = "@scene bad\n@place ch01 30,2\n> x\n@end\n";
        let mut files = bundle::files_in(dialogue::DIALOGUE_DIR)
            .into_iter()
            .filter(|p| p.ends_with(dialogue::DIALOGUE_EXTENSION))
            .map(|p| (bundle::display_path(p), bundle::file(p).unwrap_or_default()))
            .collect::<Vec<_>>();
        files.push(("bad.dlg".to_owned(), bad));
        let errors = load_with_scripts(&files).unwrap_err();
        assert_eq!(
            errors.0,
            [ContentError::new(
                "bad.dlg",
                "tile 30,2 is off map \"ch01\" (24 by 16 tiles, from 0,0 to 23,15)"
            )
            .at(2, None)]
        );
    }

    /// A chapter's intro and victory scenes are its battle's, a trigger's
    /// scene its battle's; a support conversation and an unknown scene are
    /// no battle's.
    #[test]
    fn a_scenes_battle_is_its_chapters_or_its_triggers() {
        let content = load_embedded().unwrap();
        let of = |scene: &str| content.battle_of_scene(scene).map(|b| b.id.as_str());
        assert_eq!(of("ch01_prebattle"), Some("ch01"));
        assert_eq!(of("ch01_victory"), Some("ch01"));
        assert_eq!(of("test_intro"), Some("test"));
        assert_eq!(of("test_victory"), Some("test"));
        assert_eq!(of("test_fort"), Some("quick"));
        assert_eq!(of("test_rogue_falls"), Some("quick"));
        assert_eq!(of("test_support_lord_knight_c"), None);
        assert_eq!(of("no_such_scene"), None);
    }

    #[test]
    fn seal_checks_join_the_item_errors() {
        let classes = ok_classes().ok();
        let items = item::load();
        assert_eq!(check_seals(items.clone(), classes.as_ref()), items);
        // Without its seals, the class tree's promotions can't happen.
        let mut bare = items.clone().unwrap_or_default();
        bare.items
            .retain(|_, d| !matches!(d, trpg_core::ItemDef::Seal(_)));
        let errors: Vec<String> = check_seals(Ok(bare.clone()), classes.as_ref())
            .err()
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            errors,
            [
                "assets/data/items.ron: no seal of kind Tier(2), but classes promote into tier 2",
                "assets/data/items.ron: no seal of kind Tier(3), but classes promote into tier 3",
                "assets/data/items.ron: no seal of kind Reclass",
            ]
        );
        // Skipped when another file failed.
        assert_eq!(check_seals(Ok(bare.clone()), None), Ok(bare));
        let failed = Err(vec![ContentError::new("i", "bad")]);
        assert_eq!(check_seals(failed.clone(), classes.as_ref()), failed);
    }

    #[test]
    fn art_reference_checks_join_the_art_errors() {
        let items = item::load().ok();
        let arts = art::load();
        assert_eq!(check_art_references(arts.clone(), items.as_ref()), arts);
        // A weapon naming an art that isn't there.
        let mut broken = items.clone().unwrap_or_default();
        if let Some(trpg_core::ItemDef::Weapon(w)) =
            broken.items.get_mut(&trpg_core::ItemId::new("iron_sword"))
        {
            w.arts.push(trpg_core::ArtId::new("nope"));
        }
        let errors = check_art_references(arts.clone(), Some(&broken));
        assert!(errors.is_err_and(|e| e.len() == 1));
        // Skipped when another file failed.
        assert_eq!(check_art_references(arts.clone(), None), arts);
        let failed = Err(vec![ContentError::new("a", "bad")]);
        assert_eq!(check_art_references(failed.clone(), items.as_ref()), failed);
    }

    #[test]
    fn embedded_content_loads() {
        let content = load_embedded();
        assert!(content.is_ok(), "{content:?}");
        let content = content.ok();
        assert_eq!(
            content.as_ref().map(|c| &c.terrain),
            ok_terrain().ok().as_ref()
        );
        assert_eq!(content.as_ref().map(|c| &c.maps), ok_maps().ok().as_ref());
        assert_eq!(
            content.as_ref().map(|c| &c.classes),
            ok_classes().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.characters),
            ok_characters().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.skills),
            ok_skills().ok().as_ref()
        );
        assert_eq!(content.as_ref().map(|c| &c.arts), art::load().ok().as_ref());
        assert_eq!(
            content.as_ref().map(|c| &c.portraits),
            ok_portraits().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.dialogue),
            ok_dialogue().ok().as_ref()
        );
        assert_eq!(
            content.as_ref().map(|c| &c.supports),
            ok_supports().ok().as_ref()
        );
        // The battles' tables hold the supports.
        assert_eq!(
            content.as_ref().map(|c| c.tables().supports),
            ok_supports().ok().map(Arc::new)
        );
    }
}
