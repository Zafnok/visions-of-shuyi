//! The writer's script preview (ticket 0764): `story-preview`, the program
//! a writer with no source code is handed. It lists the scenes of the
//! `.dlg` files in a folder and plays the one picked in the game's own
//! [`DialogueScreen`], with the faces, the music and the place (a battle's
//! map for a scene a battle plays), and reads a file again when it is
//! saved. The screen and its tabs (scenes, switches, lines, problems) are
//! Shuyi Engine's ([`DialoguePreview`]); this module hands it what only the
//! game knows ([`ShuyiPreview`]): its lines, its checks, the lead and the
//! dialogue screen.
//!
//! The game's checks are `story-check`'s, every rule of the content load
//! with the folder's scripts as the game's
//! ([`trpg_content::load_with_some_scripts`]): a chapter or battle naming a
//! scene the folder hasn't is no problem here (a trial packet holds two
//! scenes), and the window says only the folder's scenes are checked.
//!
//! [`StoryPreview`] wraps the engine's screen with the game's page of names
//! and faces (Info, the Unit info key), that note, and a line naming each face's
//! expression under the strip. What it shows is a [`StoryPreviewView`];
//! [`paint`] draws it (ADR-0054).

use crate::prelude::*;
use core::cell::Cell as Count;

use shuyi_assets::bundle::Layer;
use shuyi_assets::dialogue::{DialogueFile, scripts_from_parsed};
use shuyi_platform::{Card, Look, Setup, Start};
use shuyi_ui::GameHooks;
use shuyi_ui::dialogue::preview::{
    DialogueFiles, DialogueFolder, DialoguePreview, Play, PreviewGame, PreviewScreen, Showing,
};
use trpg_content::dialogue::FormatRules;
use trpg_content::dialogue::ParsedScene;
use trpg_content::{ContentError, ContentErrors, Stage};
use trpg_core::lead::{DEFAULT_NAME, LEAD_ID, MAX_NAME_LEN, PORTRAIT_MALE};
use trpg_core::{LeadGender, LeadProfile};

use crate::color::UiColor;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::game::{self, Game};
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::input::Action;
use crate::map_view::Place;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, ShuyiCtx, Transition};
use crate::screens::DialogueScreen;

/// The lead's name with the long-name switch on: as long as a player may
/// type one ([`MAX_NAME_LEN`]), to see the longest text box.
// check-text: not player text (a name the lead may be given, as a player types one)
pub const LONG_NAME: &str = "Maximilianus";

/// The game's switches, in [`ShuyiPreview::switches`]' order.
const FEMALE: usize = 0;
/// The long name's switch.
const LONG: usize = 1;

/// Text key of the female lead's switch.
const FEMALE_KEY: &str = "preview.switch.female_lead";
/// Text key of the long name's switch.
const LONG_KEY: &str = "preview.switch.long_name";
/// Text key of the note when the game names scenes the folder hasn't.
const ONLY_HERE_KEY: &str = "preview.note.only_here";
/// Text key of the line naming each face's expression.
const FACES_KEY: &str = "preview.faces";
/// Text key of the names and faces page's title.
const REFERENCE_KEY: &str = "preview.names.title";
/// Text key of its heading over the names.
const NAMES_KEY: &str = "preview.names.names";
/// Text key of its heading over the characters.
const CHARACTERS_KEY: &str = "preview.names.characters";
/// Text key of its help line.
const REFERENCE_HELP_KEY: &str = "preview.names.help";

/// What the game hands the engine's preview: its lines ([`Stage`]), the
/// format's rules on them, `story-check`'s checks, the lead's switches and
/// the dialogue screen.
#[derive(Debug, Default)]
pub struct ShuyiPreview {
    /// How many scenes the game names that the folder hasn't, at the last
    /// check: when some, only the folder's scenes were checked.
    not_here: Count<usize>,
}

impl ShuyiPreview {
    /// How many scenes the game's chapters, battles and the rest name that
    /// the folder hasn't, at the last check.
    pub fn scenes_not_here(&self) -> usize {
        self.not_here.get()
    }
}

impl PreviewGame<Shuyi> for ShuyiPreview {
    type Directive = Stage;
    type Rules = FormatRules;

    fn rules(&self, _: &Ctx) -> FormatRules {
        FormatRules
    }

    /// `story-check`'s problems with the folder's files that parsed as the
    /// game's scripts, but those the format's checks (`rules`) found
    /// already.
    fn problems(&self, ctx: &Ctx, scenes: &[ParsedScene]) -> Vec<ContentError> {
        let files = ctx.module_state::<DialogueFiles>();
        let parsed = |file: &str| scenes.iter().any(|s| s.file == file);
        let sources: Vec<(&str, &str)> = files
            .into_iter()
            .flat_map(DialogueFiles::files)
            .filter(|(file, _)| parsed(file))
            .filter_map(|(file, bytes)| Some((file, core::str::from_utf8(bytes).ok()?)))
            .collect();
        let loaded = trpg_content::load_with_some_scripts(&sources);
        self.not_here.set(loaded.not_here.len());
        let Err(ContentErrors(mut problems)) = loaded.content else {
            return Vec::new();
        };
        let by_file = sources.iter().map(|(file, _)| DialogueFile {
            scenes: scenes.iter().filter(|s| s.file == *file).cloned().collect(),
        });
        if let Err(found) = scripts_from_parsed(by_file, &FormatRules) {
            problems.retain(|p| !found.contains(p));
        }
        problems
    }

    fn switches(&self, ctx: &Ctx) -> Vec<String> {
        [FEMALE_KEY, LONG_KEY]
            .into_iter()
            .map(|key| ctx.text(key).to_owned())
            .collect()
    }

    fn play(&self, ctx: &mut Ctx, play: &Play<'_, Stage>) -> Box<dyn PreviewScreen<Shuyi>> {
        Box::new(dialogue_screen(ctx, play))
    }
}

/// The lead of `play`'s switches: female or male, with the default name or
/// [`LONG_NAME`].
pub fn lead_of(play: &Play<'_, Stage>) -> LeadProfile {
    let gender = if play.switch(FEMALE) {
        LeadGender::Female
    } else {
        LeadGender::Male
    };
    let name = if play.switch(LONG) {
        LONG_NAME
    } else {
        DEFAULT_NAME
    };
    LeadProfile::new(name, gender)
}

/// The game's dialogue screen playing `play` from its text box, over its
/// battle's map for a scene a battle plays, as the game shows it: the
/// name where a line writes `{lead|<words>}`.
fn dialogue_screen(ctx: &Ctx, play: &Play<'_, Stage>) -> DialogueScreen {
    let names = ctx.game.content.names.clone();
    let scene = trpg_content::dialogue::as_shown(play.scene);
    let player = crate::dialogue::DialoguePlayer::at_text_box(
        &scene,
        lead_of(play),
        names,
        play.present,
        play.text_box,
    );
    let screen = DialogueScreen::playing(player).wrapped_for(ctx);
    match ctx.game.content.battle_of_scene(&play.scene.id) {
        Some(def) => screen.with_battle(Place::of_battle(&ctx.game.content, def)),
        None => screen,
    }
}

impl PreviewScreen<Shuyi> for DialogueScreen {
    fn showing(&self) -> Showing {
        self.player().showing()
    }
}

/// The engine's preview of the game's scenes.
pub type Preview = DialoguePreview<Shuyi, ShuyiPreview>;

/// What the preview shows besides the engine's screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryPreviewView {
    /// The page of names and faces, while it is open.
    pub reference: Option<ReferenceView>,
    /// Over the tabs, when the game names scenes the folder hasn't: that
    /// only the folder's are checked.
    pub only_here: Option<String>,
    /// Under the strip while a scene plays: the expression each face
    /// shows, left then right.
    pub faces: Option<String>,
}

/// The page of names and faces: what a writer looks up without the
/// repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceView {
    /// Its title.
    pub title: String,
    /// Its rows, top to bottom.
    pub rows: Vec<ReferenceRow>,
    /// The first row shown.
    pub first: usize,
    /// The help line.
    pub help: String,
}

/// A row of the page of names and faces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceRow {
    /// A heading.
    Heading(String),
    /// What a script writes, then what it is: `{n:<id>}` and today's name,
    /// or a character's id and their expressions.
    Entry(String, String),
}

/// The preview program's root screen: the engine's preview, and the
/// game's page of names and faces over it.
pub struct StoryPreview {
    /// The engine's screen.
    preview: Preview,
    /// The first row shown of the page of names and faces, while it is
    /// open.
    reference: Option<usize>,
}

impl core::fmt::Debug for StoryPreview {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // check-text: not player text (the type, for debugging)
        f.debug_struct("StoryPreview")
            .field("preview", &self.preview)
            .field("reference", &self.reference)
            .finish()
    }
}

impl StoryPreview {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "story_preview";

    /// The preview on `ctx`, over the files of its [`DialogueFolder`]
    /// module.
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            preview: DialoguePreview::new(ctx, ShuyiPreview::default()),
            reference: None,
        }
    }

    /// The engine's screen.
    pub fn preview(&self) -> &Preview {
        &self.preview
    }

    /// The game's dialogue screen playing the scene, if one plays.
    pub fn playing(&self) -> Option<&DialogueScreen> {
        let screen = self.preview.playing()?.as_any()?;
        screen.downcast_ref::<DialogueScreen>()
    }

    /// What it shows besides the engine's screen.
    pub fn view(&self, ctx: &Ctx) -> StoryPreviewView {
        let engine = self.preview.view(ctx);
        let not_here = self.preview.game().scenes_not_here();
        StoryPreviewView {
            reference: self.reference.map(|first| reference(ctx, first)),
            only_here: (engine.open && not_here > 0).then(|| ctx.text(ONLY_HERE_KEY).to_owned()),
            faces: engine.strip.and(self.faces(ctx)),
        }
    }

    /// The expression each face of the scene playing shows, if any face
    /// shows.
    fn faces(&self, ctx: &Ctx) -> Option<String> {
        let view = self.playing()?.player().current();
        let faces: Vec<String> = [view.left, view.right]
            .into_iter()
            .flatten()
            .map(|p| format!("{}[{}]", p.character.0, p.expression))
            .collect();
        let faces = faces.join("  ");
        (!faces.is_empty()).then(|| ctx.text_with(FACES_KEY, &[("faces", &faces)]))
    }

    /// One frame with the page of names and faces open.
    fn read_reference(&mut self, ctx: &Ctx, input: &FrameInput) {
        let rows = reference_rows(ctx).len();
        for &action in &input.actions {
            let Some(first) = &mut self.reference else {
                return;
            };
            match action {
                Action::CursorUp => *first = first.saturating_sub(1),
                Action::CursorDown => *first = (*first + 1).min(rows.saturating_sub(1)),
                Action::Info | Action::Cancel => self.reference = None,
                _ => {}
            }
        }
    }
}

/// The page of names and faces, from row `first`.
fn reference(ctx: &Ctx, first: usize) -> ReferenceView {
    ReferenceView {
        title: ctx.text(REFERENCE_KEY).to_owned(),
        rows: reference_rows(ctx),
        first,
        help: ctx.text_with(REFERENCE_HELP_KEY, &[]),
    }
}

/// Every name token with today's name, then every character a script may
/// place or have speak, with the expressions of their face (the lead's
/// male one: the female one has the same).
fn reference_rows(ctx: &Ctx) -> Vec<ReferenceRow> {
    let content = &ctx.game.content;
    let mut rows = vec![ReferenceRow::Heading(ctx.text(NAMES_KEY).to_owned())];
    for (id, name) in &content.names.names {
        // check-text: not player text (a script's token)
        rows.push(ReferenceRow::Entry(format!("{{n:{id}}}"), name.clone()));
    }
    rows.push(ReferenceRow::Heading(ctx.text(CHARACTERS_KEY).to_owned()));
    let cast = &content.characters;
    let mut ids: Vec<&str> = cast.characters.keys().map(|c| c.0.as_str()).collect();
    ids.extend(cast.speakers.iter().map(|c| c.0.as_str()));
    ids.sort_unstable();
    ids.dedup();
    for id in ids {
        let face = if id == LEAD_ID { PORTRAIT_MALE } else { id };
        let expressions = content.portraits.get(face).map(|p| {
            // check-text: not a data name (expression ids, as a script writes them)
            let names: Vec<&str> = p.expressions.iter().map(|e| e.name.as_str()).collect();
            names.join(", ")
        });
        rows.push(ReferenceRow::Entry(
            id.to_owned(),
            expressions.unwrap_or_default(),
        ));
    }
    rows
}

/// How far a row's first part sits from the page's left edge.
const ENTRY_X: i32 = 3;

/// The column of the rows' second parts: two cells past the longest first
/// part.
fn value_x(view: &ReferenceView) -> i32 {
    let longest = view.rows.iter().map(|row| match row {
        ReferenceRow::Entry(token, _) => token.chars().count(),
        ReferenceRow::Heading(_) => 0,
    });
    ENTRY_X + 2 + i32::try_from(longest.max().unwrap_or(0)).unwrap_or(0)
}

/// Paints `view` over the engine's preview in `buf`: the page of names and
/// faces over the whole of it, or the note on the tabs' second row and the
/// faces under the strip.
pub fn paint(ctx: &Ctx, view: &StoryPreviewView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, dim) = (c(UiColor::Black), c(UiColor::TextDim));
    let width = i32::from(buf.width());
    if let Some(reference) = &view.reference {
        return paint_reference(ctx, reference, buf);
    }
    if let Some(note) = &view.only_here {
        let w = i32::try_from(note.chars().count()).unwrap_or(width);
        buf.print(width - w - 1, 1, note, c(UiColor::TextHighlight), black);
    }
    if let Some(faces) = &view.faces {
        let bg = c(UiColor::PanelBg);
        buf.fill_rect(Rect::new(0, 1, width, 1), Cell::new(' ', bg, bg));
        buf.print(1, 1, faces, dim, bg);
    }
}

/// The page of names and faces: its title, its rows from the first shown,
/// and its help on the bottom row.
fn paint_reference(ctx: &Ctx, view: &ReferenceView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, text) = (c(UiColor::Black), c(UiColor::Text));
    let (dim, high) = (c(UiColor::TextDim), c(UiColor::TextHighlight));
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    let bottom = i32::from(buf.height()) - 1;
    buf.print(1, 0, &view.title, high, black);
    let value_x = value_x(view);
    let rows = (2..bottom).zip(view.rows.iter().skip(view.first));
    for (y, row) in rows {
        match row {
            ReferenceRow::Heading(heading) => {
                buf.print(1, y, heading, high, black);
            }
            ReferenceRow::Entry(token, value) => {
                buf.print(ENTRY_X, y, token, text, black);
                buf.print(value_x, y, value, dim, black);
            }
        }
    }
    buf.print(1, bottom, &view.help, dim, black);
}

impl Screen<Shuyi> for StoryPreview {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    /// The engine's preview's, and Info (Unit info) for the page of names and
    /// faces.
    fn actions(&self) -> Option<Vec<Action>> {
        let mut actions = self.preview.actions().unwrap_or_default();
        for action in [Action::CursorUp, Action::CursorDown, Action::Info] {
            if !actions.contains(&action) {
                actions.push(action);
            }
        }
        Some(actions)
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if self.reference.is_some() {
            self.read_reference(ctx, input);
            return Transition::None;
        }
        let tabs = self.preview.view(ctx);
        let info = input.actions.contains(&Action::Info);
        if info && tabs.open && !tabs.go_to.typing {
            self.reference = Some(0);
            return Transition::None;
        }
        self.preview.update(ctx, input)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        self.preview.draw(ctx, buf);
        paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}

/// The game's content with none of its scripts (the preview's build
/// embeds none), and the context over it, on a PC's console. A scene the
/// game names is no problem: the folder's scripts are checked when read.
///
/// # Errors
///
/// The content's problems, or the palette's.
pub fn ctx() -> Result<Ctx, String> {
    let loaded = trpg_content::load_with_some_scripts::<&str>(&[]);
    let content = loaded.content.map_err(|e| e.to_string())?;
    let keys = crate::lang_keys::check_ui_text_keys(trpg_content::bundle::install());
    keys.map_err(|e| ContentErrors(e).to_string())?;
    let mut ctx = Ctx::from_content(content).map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PC;
    Ok(ctx)
}

/// The preview on `ctx` over the folder read as `folder`, with the
/// layout picker on the first key until a layout is chosen, as in the
/// game, and no debug screens: a writer's program.
///
/// # Errors
///
/// If the folder's module can't join the game.
pub fn game(mut ctx: Ctx, folder: Layer) -> Result<Game, String> {
    game::use_saved_layout(&mut ctx);
    let root = StoryPreview::new(&ctx);
    let hooks = GameHooks {
        debug: None,
        ..game::hooks()
    };
    Game::new(ctx, Box::new(root), hooks)
        .with_debug_screens(false)
        .with_module(DialogueFolder::new(folder))
        .map_err(|e| e.to_string())
}

/// What the preview's frames are painted with: the game's font atlas and
/// pictures.
fn look(game: &Shuyi) -> Look<'_> {
    Look::new(&game.content.font, &game.content.images)
}

/// How the preview starts: `title` on its loading card, then the preview
/// over `folder` (read with `shuyi_app::read_watched_folder`, so a saved
/// file is read again in an optimized build), or the folder's error.
pub fn start(title: &'static str, folder: Result<Layer, String>) -> Start<Shuyi> {
    Start::new(Card::text(title), look, move |setup: Setup| {
        let folder = folder?;
        let ctx = setup.ctx(ctx()?);
        game(ctx, folder)
    })
}

/// The longest name a player may give the lead.
const _: () = assert!(LONG_NAME.len() == MAX_NAME_LEN);

#[cfg(test)]
mod tests;
