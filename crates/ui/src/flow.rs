//! The game flow (ticket 0801): from `New Game` through each chapter's
//! scenes and battle to the result; and saving and loading it (0802).
//!
//! ```text
//! New Game → mode → familiarity → lead → [chapter: intro scenes → (Camp → Preparations) → battle
//!     ├─ victory → apply the result → the results (0810: gold, rewind
//!     │            bonus, level ups) → victory scenes
//!     │            → "Save your progress?" (→ slot picker)
//!     │            → (Camp →) next chapter, or "To be continued" → title
//!     ├─ defeat  → Game Over → Retry (the battle again) | Title
//!     └─ Suspend → the suspend save → title]
//! Load Game → slot picker → (Camp →) the chapter after the one the save cleared
//! Continue  → the suspended battle, where it was (the save is deleted)
//! ```
//!
//! [`FlowScreen`] is one screen on the stack that owns the campaign and
//! pushes the flow's screens over itself, one at a time: when one pops,
//! the flow reads what it popped with ([`Screen::uncovered`]) and pushes
//! the next, in the same frame; with none left, it pops itself (back to
//! the title). Game Over is the battle's own, over it (the battle pops
//! once the player picks `Retry` or `Title`). A battle is played from its
//! [`BattleSetup`](trpg_core::BattleSetup), kept so that `Restart Battle`
//! (map menu) and `Retry` (Game Over) rebuild it exactly, every rewind
//! charge back.
//!
//! A battle with `preparations: true` opens the Preparations screen (0408)
//! first, which changes the setup's loadouts and pack (and the gear of the
//! roster's units left out of the battle); `Fight!` starts the battle with
//! it. Both restarts go back to Preparations, as the player
//! left it (Nick, 0408), so they can change their gear before trying again.
//! Only the Quick Battle may leave Preparations (back to the title).
//!
//! **Camp** (1003, `supports.md`, `world-structure.md`): the Camp screen
//! opens between two chapters, before the next one's scenes, and before
//! the Preparations of a battle that has them; but once between two
//! battles: a battle whose chapter just came from camp goes straight to
//! Preparations. Camp with nothing to list (no support conversation
//! unlocked or seen, no camp event) is skipped (*Claude's starting
//! rules*). It works on a copy of the campaign, which the flow takes back
//! when camp breaks. Restarting a battle, `Retry` and `Continue` don't
//! open it. (The world map's `Camp`, 1007, opens the same screen.)
//!
//! **Saves** (`death-and-difficulty.md`, ADR-0039). A chapter save holds
//! the campaign once its chapter is won, so its
//! [`chapter`](Campaign::chapter) is the one just cleared until the next
//! begins; loading one starts the chapter after it. The suspend save holds
//! the campaign as the battle started with it and the battle's history;
//! continuing rebuilds the battle's setup from the two, so a restart after
//! it is the same as before.
//!
//! A suspended battle that had Preparations continues with the loadouts,
//! stock and pack it started with (they are in its history), and a restart
//! after it goes back to Preparations as they were left.
//!
//! Music (ticket 0807, `docs/design/audio.md`): each battle file names its
//! music, asked for once each time the battle starts (also on `Retry` and
//! `Restart Battle`, where a pool picks again); it already plays on the
//! battle's Preparations screen. The battle screen asks for
//! none until the battle ends, so the track stays through both phases,
//! combat and rewinds. At the outcome's banner the battle screen asks for
//! the victory sting, or for `defeat` (0809), which plays on through Game
//! Over: that screen asks for nothing. `Rewind` there keeps `defeat` until
//! a point is picked; then the battle's track (the same one, which the
//! flow gives it) starts again. "To be continued" stops the music.

use crate::prelude::*;
use alloc::collections::VecDeque;
use core::any::Any;

use trpg_content::{ChapterDef, MapLook, Present, Scene, battle_campaign, deaths, new_campaign};
use trpg_core::{
    BattleDef, BattleMusic, BattleRewards, BattleState, Campaign, GameMode, LeadProfile,
    Preparations, SaveFile, SavePoint, Shop, TutorialLevel,
};

use crate::glyph_buffer::GlyphBuffer;
use crate::map_view::Place;
use crate::save::{self, SUSPEND_KEY, SaveError};
use crate::screen::{Ctx, FrameInput, ModeSwitch, Screen, ScreenResult, Shuyi, Transition};
use crate::screens::CampScreen;
use crate::screens::battle::BattleEnd;
use crate::screens::familiarity::FamiliarityScreen;
use crate::screens::game_over::ToBeContinuedScreen;
use crate::screens::lead_select::LeadSelectScreen;
use crate::screens::mode_select::ModeSelectScreen;
use crate::screens::preparations::PreparationsScreen;
use crate::screens::save::{SavePromptScreen, SlotOutcome, SlotPickerScreen};
use crate::screens::{BattleScreen, DialogueScreen, ResultsScreen};
use crate::storage::Storage;

/// The Preparations shop of `chapter`'s battle (ticket 0443): what its
/// number unlocked; `None` for a chapter of no number.
fn basic_shop(ctx: &Ctx, chapter: &ChapterDef) -> Option<Shop> {
    let number = chapter.number?;
    Some(ctx.game.content.basic_shop.shop(number))
}

/// Asks for battle `def`'s music: its cue, or a track picked from its
/// pool. The pick is the UI's, not the battle's RNG (ADR-0019): a rewind or
/// a replay never changes the track. An empty pool leaves the music as it
/// is. Returns the cue asked for.
fn play_battle_music(ctx: &mut Ctx, def: &BattleDef) -> Option<String> {
    let cue = match &def.music {
        BattleMusic::Cue(cue) => Some(cue.clone()),
        BattleMusic::Pool(pool) => ctx.pick_music(pool),
    };
    if let Some(cue) = &cue {
        ctx.audio.play_music(cue);
    }
    cue
}

/// The chapter the debug Quick Battle plays (`assets-test/chapters/quick.ron`).
pub const QUICK_CHAPTER: &str = "quick";

/// The screen the flow has open over it, with what the flow needs to know
/// to read what it pops with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// Classic or Casual.
    Mode,
    /// How familiar the player is (the tutorial level), in the mode
    /// picked.
    Familiarity(GameMode),
    /// The lead's gender and name, in the mode and tutorial level picked.
    Lead(GameMode, TutorialLevel),
    /// A chapter's scene.
    Scene,
    /// Camp: the supports and camp events, between battles (1003).
    Camp,
    /// Loadouts and the pack, before a battle that has Preparations.
    Preparations,
    /// The chapter's battle (and its Game Over).
    Battle,
    /// What a won battle gave (0810).
    Results,
    /// "Save your progress?", after a chapter's victory scenes.
    SavePrompt,
    /// The slot picker: saving after a victory (`true`), or loading from
    /// the title.
    Slots(bool),
    /// After the last chapter.
    ToBeContinued,
}

/// What comes after the scenes being played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Then {
    /// The chapter's battle (after the intro scenes).
    Battle,
    /// The next chapter (after the victory scenes).
    NextChapter,
}

/// The chapter's battle, as it started (as last prepared, if it has
/// Preparations).
#[derive(Debug, Clone)]
struct Fight {
    def: BattleDef,
    /// The battle's setup, and the army's units left out of it.
    prep: Preparations,
    /// How the battle's map looks, as its file says (ADR-0052).
    look: MapLook,
}

/// The game flow: one chapter after another. See the module docs.
pub struct FlowScreen {
    stage: Stage,
    /// The screen to push over the flow next ([`show`](Self::show)): on
    /// its first update, or as soon as the screen above pops.
    next: Option<Box<dyn Screen<Shuyi>>>,
    campaign: Option<Campaign>,
    chapter: Option<ChapterDef>,
    fight: Option<Fight>,
    /// Scenes still to play before [`then`](Self::then).
    scenes: VecDeque<String>,
    then: Then,
    /// The last won battle's rewards.
    rewards: Option<BattleRewards>,
    /// [`Ctx::clock_s`] when the campaign began, for its playtime.
    started_at: f64,
    /// Whether Preparations may be left (the Quick Battle: to the title).
    can_leave: bool,
    /// The battle's music cue, as asked for when it last started.
    music: Option<String>,
    /// Whether camp was offered since the last battle (once between two).
    camped: bool,
    /// The chapter to start once camp breaks; `None`: the chapter's
    /// battle.
    after_camp: Option<String>,
}

impl FlowScreen {
    /// Name reported by [`Screen::name`] once the flow's first screen is
    /// over it.
    pub const NAME: &'static str = "game_flow";

    /// A flow with nothing to show yet.
    fn empty() -> Self {
        Self {
            stage: Stage::ToBeContinued,
            next: None,
            campaign: None,
            chapter: None,
            fight: None,
            scenes: VecDeque::new(),
            then: Then::Battle,
            rewards: None,
            started_at: 0.0,
            can_leave: false,
            music: None,
            camped: false,
            after_camp: None,
        }
    }

    /// `New Game`: the mode screen first, in `ctx`'s language.
    pub fn new_game(ctx: &Ctx) -> Self {
        let mut flow = Self::empty();
        flow.show(Stage::Mode, Box::new(ModeSelectScreen::new(ctx)));
        flow
    }

    /// `Load Game`: the slot picker first. Going back from it ends the
    /// flow.
    pub fn load_game(ctx: &Ctx) -> Self {
        let mut flow = Self::empty();
        flow.show(Stage::Slots(false), Box::new(SlotPickerScreen::load(ctx)));
        flow
    }

    /// `Continue`: the suspended battle, exactly where it was. The suspend
    /// save is deleted (it can't be loaded twice,
    /// `death-and-difficulty.md`); a save that can't be continued is left
    /// alone.
    pub fn resume(ctx: &mut Ctx) -> Result<Self, SaveError> {
        let file = save::read(&ctx.storage, SUSPEND_KEY)?.ok_or(SaveError::Missing)?;
        let SaveFile {
            campaign, point, ..
        } = file;
        let SavePoint::Battle(mut history) = point else {
            return Err(SaveError::Corrupt);
        };
        // A chapter or battle the game no longer has can't be continued.
        let chapter = ctx.game.content.chapters.get(&campaign.chapter);
        let chapter = chapter.cloned().ok_or(SaveError::Corrupt)?;
        let def = ctx.game.content.battles.get(&chapter.battle);
        let def = def.cloned().ok_or(SaveError::Corrupt)?;
        let tables = ctx.game.content.tables();
        // What Preparations set up is in the battle's first state.
        let mut setup = campaign.battle_setup(&def, &tables);
        setup.prepared_as(&history.state_at(0));
        let prep = Preparations {
            setup,
            bench: campaign.bench(&def),
            shop: basic_shop(ctx, &chapter),
        };
        history.restore_tables(&tables);
        // If deleting fails the battle still continues.
        let _ = ctx.storage.delete(SUSPEND_KEY);
        // The battle's music again (a pool picks afresh).
        let music = play_battle_music(ctx, &def);
        let mut flow = Self::empty();
        flow.music.clone_from(&music);
        flow.adopt(ctx, campaign);
        flow.chapter = Some(chapter);
        let look = ctx.game.content.battle_look(&def);
        let battle = BattleScreen::resume(*history)
            .with_look(look.clone())
            .in_battle(&def.id)
            .with_exit_name(def.exit_name.clone())
            .with_music(music);
        flow.fight = Some(Fight { def, prep, look });
        flow.show(Stage::Battle, Box::new(battle));
        Ok(flow)
    }

    /// The debug Quick Battle: the [`QUICK_CHAPTER`] with its battle's own
    /// characters, in Classic. `None` if the content lacks it.
    pub fn quick_battle(ctx: &mut Ctx) -> Option<Self> {
        let chapter = ctx.game.content.chapters.get(QUICK_CHAPTER)?;
        let def = ctx.game.content.battles.get(&chapter.battle)?;
        let mut campaign = battle_campaign(
            &ctx.game.content,
            def,
            GameMode::Classic,
            ctx.game.lead.clone(),
        );
        QUICK_CHAPTER.clone_into(&mut campaign.chapter);
        let mut flow = Self::empty();
        flow.can_leave = true;
        flow.begin(ctx, campaign);
        Some(flow)
    }

    /// The campaign, once the lead is made.
    pub fn campaign(&self) -> Option<&Campaign> {
        self.campaign.as_ref()
    }

    /// The chapter being played.
    pub fn chapter(&self) -> Option<&ChapterDef> {
        self.chapter.as_ref()
    }

    /// What the last won battle gave.
    pub fn rewards(&self) -> Option<&BattleRewards> {
        self.rewards.as_ref()
    }

    /// Opens `screen` over the flow next, as its `stage`.
    fn show(&mut self, stage: Stage, screen: Box<dyn Screen<Shuyi>>) {
        self.stage = stage;
        self.next = Some(screen);
    }

    /// Pushes the screen to show next, with the switch to Casual open only
    /// on Preparations ([`Ctx::mode_switch`]): a switch made on the Options
    /// screen there (0805) goes into the campaign and into the battle being
    /// prepared, so it starts (and restarts) in Casual. With nothing to
    /// show, the flow is over: back to the title, with no campaign.
    fn push_next(&mut self, ctx: &mut Ctx) -> Transition {
        let open = self.stage == Stage::Preparations && self.campaign.is_some();
        ctx.game.mode_switch = if open {
            ModeSwitch::Open
        } else {
            ModeSwitch::Closed
        };
        if let Some(screen) = self.next.take() {
            return Transition::Push(screen);
        }
        ctx.game.campaign_mode = None;
        ctx.game.campaign_tutorial = None;
        ctx.game.campaign_lessons.clear();
        Transition::Pop
    }

    /// Makes `campaign` the flow's: dialogue from here on uses its lead,
    /// [`Ctx`] owns its mode, tutorial level and lessons from here on
    /// ([`sync`](Self::sync) reads them back), and its playtime counts on
    /// from what it had.
    fn adopt(&mut self, ctx: &mut Ctx, campaign: Campaign) {
        ctx.game.lead = campaign.lead.clone();
        ctx.game.campaign_mode = Some(campaign.mode);
        ctx.game.campaign_tutorial = Some(campaign.tutorial);
        ctx.game.campaign_lessons.clone_from(&campaign.lessons_done);
        #[expect(clippy::cast_precision_loss, reason = "exact below 2^53 s")]
        let played = campaign.playtime_s as f64;
        self.started_at = ctx.clock_s - played;
        self.campaign = Some(campaign);
    }

    /// Starts `campaign` at its chapter.
    fn begin(&mut self, ctx: &mut Ctx, campaign: Campaign) {
        let chapter = campaign.chapter.clone();
        self.adopt(ctx, campaign);
        self.start_chapter(ctx, &chapter);
    }

    /// Goes on with the loaded chapter save `campaign`: the chapter after
    /// the one it cleared ("To be continued" if the game has none yet).
    fn begin_loaded(&mut self, ctx: &mut Ctx, campaign: Campaign) {
        self.chapter = ctx.game.content.chapters.get(&campaign.chapter).cloned();
        self.adopt(ctx, campaign);
        self.next_chapter(ctx);
    }

    /// Starts chapter `id` with its intro scenes. A chapter missing from
    /// the content (validation rules it out) ends the flow at "To be
    /// continued".
    fn start_chapter(&mut self, ctx: &mut Ctx, id: &str) {
        let Some(chapter) = ctx.game.content.chapters.get(id).cloned() else {
            self.the_end(ctx);
            return;
        };
        self.scenes = chapter.intro_scenes.iter().cloned().collect();
        self.then = Then::Battle;
        self.chapter = Some(chapter);
        self.next_scene(ctx);
    }

    /// Plays the next scene waiting, or goes on to what follows them.
    /// Scenes missing from the content (validation rules it out) are
    /// skipped, and so is one with nothing to say for the army as it is
    /// (every line of it was someone's who is gone, ADR-0055).
    fn next_scene(&mut self, ctx: &mut Ctx) {
        while let Some(id) = self.scenes.pop_front() {
            if let Some(scene) = ctx.game.content.dialogue.get(&id)
                && self.play(scene, ctx)
            {
                return;
            }
        }
        match self.then {
            Then::Battle => self.start_battle(ctx),
            // The chapter is over: the save prompt, then the next one.
            Then::NextChapter => {
                self.show(Stage::SavePrompt, Box::new(SavePromptScreen::new(ctx)));
            }
        }
    }

    /// Plays `scene` for the army as it is: the characters in the
    /// campaign's roster are there (ADR-0055), so a companion who died in
    /// Classic isn't, and one who retreated in Casual is. Returns `false`,
    /// playing nothing, if the scene has nothing to say for them. Its
    /// deaths (`@dies`, Hollis, 0319) happen as it starts, in every mode,
    /// in the version for those there, played or not, so skipping the
    /// scene can't skip them; the scenes after it are played without them.
    fn play(&mut self, scene: &Scene, ctx: &Ctx) -> bool {
        let lead = self
            .campaign
            .as_ref()
            .map_or_else(|| ctx.game.lead.clone(), |c| c.lead.clone());
        let present = self.campaign.as_ref().map_or(Present::Everyone, |c| {
            Present::only(c.roster.iter().filter_map(|u| u.character.clone()))
        });
        if let Some(campaign) = &mut self.campaign {
            let supports = &ctx.game.content.supports;
            for character in deaths(&scene.resolved(&present)) {
                campaign.dies_in_scene(&character, supports);
            }
        }
        let mut screen = DialogueScreen::told(ctx, scene, lead, &present);
        // `@place battle` in a chapter's scenes is its battle's field.
        let battle = self.chapter.as_ref();
        if let Some(def) = battle.and_then(|c| ctx.game.content.battles.get(&c.battle)) {
            screen = screen.with_battle(Place::of_battle(&ctx.game.content, def));
        }
        let plays = screen.has_text();
        if plays {
            self.show(Stage::Scene, Box::new(screen));
        }
        plays
    }

    /// Opens camp, if it wasn't offered since the last battle and has
    /// anything to list, to go on with `then` (the chapter to start, or
    /// the chapter's battle) once it breaks. Returns whether it opened.
    fn camp(&mut self, ctx: &Ctx, then: Option<String>) -> bool {
        let Some(campaign) = &self.campaign else {
            return false;
        };
        if self.camped || !CampScreen::has_anything(ctx, campaign) {
            return false;
        }
        self.camped = true;
        self.after_camp = then;
        let screen = CampScreen::new(ctx, campaign.clone());
        self.show(Stage::Camp, Box::new(screen));
        true
    }

    /// Starts the chapter's battle with the campaign's army: camp first,
    /// before a battle with Preparations ([`camp`](Self::camp)).
    fn start_battle(&mut self, ctx: &mut Ctx) {
        let def = self
            .chapter
            .as_ref()
            .and_then(|c| ctx.game.content.battles.get(&c.battle));
        let (Some(def), Some(_), Some(_)) = (def, &self.campaign, &self.chapter) else {
            self.the_end(ctx);
            return;
        };
        if def.preparations && self.camp(ctx, None) {
            return;
        }
        let (Some(campaign), Some(chapter)) = (&self.campaign, &self.chapter) else {
            return;
        };
        let prep = Preparations {
            setup: campaign.battle_setup(def, &ctx.game.content.tables()),
            bench: campaign.bench(def),
            shop: basic_shop(ctx, chapter),
        };
        self.fight = Some(Fight {
            def: def.clone(),
            prep,
            look: ctx.game.content.battle_look(def),
        });
        self.restart(ctx);
    }

    /// (Re)starts the battle with its music (a pool picks again):
    /// Preparations first if it has them (as the player last left them),
    /// else straight to turn 1.
    fn restart(&mut self, ctx: &mut Ctx) {
        let Some(fight) = &self.fight else {
            return;
        };
        self.music = play_battle_music(ctx, &fight.def);
        if fight.def.preparations {
            let screen = PreparationsScreen::new(fight.prep.clone(), self.can_leave);
            self.show(Stage::Preparations, Box::new(screen));
        } else {
            self.fight();
        }
    }

    /// Starts the battle from its setup: turn 1, every rewind charge.
    fn fight(&mut self) {
        let Some(fight) = &self.fight else {
            return;
        };
        let (state, events) = BattleState::new(fight.prep.setup.clone());
        let battle = BattleScreen::start(state, &events)
            .with_look(fight.look.clone())
            .in_battle(&fight.def.id)
            .with_exit_name(fight.def.exit_name.clone())
            .with_music(self.music.clone());
        self.show(Stage::Battle, Box::new(battle));
    }

    /// `Fight!` on Preparations: the battle with what the player set up.
    fn prepared(&mut self, prep: Preparations) {
        if let Some(fight) = &mut self.fight {
            fight.prep = prep;
            // The benched units keep what Preparations left them with.
            if let Some(campaign) = &mut self.campaign {
                campaign.set_members(&fight.prep.bench);
            }
        }
        self.fight();
    }

    /// "To be continued", in silence.
    fn the_end(&mut self, ctx: &mut Ctx) {
        ctx.audio.stop_music();
        self.show(Stage::ToBeContinued, Box::new(ToBeContinuedScreen));
    }

    /// Writes the suspend save: the campaign as the battle started with it
    /// and the battle's history. If it can't, the battle goes on, showing
    /// why; if it can, the flow is over (back to the title).
    fn suspend(&mut self, ctx: &mut Ctx, mut battle: Box<BattleScreen>) {
        let written = match &self.campaign {
            Some(campaign) => {
                let file = SaveFile::suspended(campaign.clone(), battle.history().clone());
                save::write(&mut ctx.storage, SUSPEND_KEY, &file)
            }
            None => Err(SaveError::Missing),
        };
        if let Err(e) = written {
            battle.suspend_failed(e.text(ctx));
            self.show(Stage::Battle, battle);
        }
    }

    /// Applies the won battle (`state`, with `unused` rewind charges) to
    /// the campaign, then its results and the victory scenes (Nick, 0810:
    /// the results come first).
    fn won(&mut self, ctx: &mut Ctx, state: &BattleState, unused: u8) {
        self.camped = false;
        let mut results = None;
        if let (Some(campaign), Some(fight)) = (&mut self.campaign, &self.fight) {
            self.rewards = campaign.apply_result(&fight.def, state, unused).ok();
            let charges = fight.prep.setup.rewind_charges;
            let told = (state, ctx.words());
            let screen = |r| ResultsScreen::new(r, told, charges, campaign.gold);
            results = self.rewards.as_ref().map(screen);
        }
        self.scenes = self
            .chapter
            .iter()
            .flat_map(|c| c.victory_scenes.iter().cloned())
            .collect();
        self.then = Then::NextChapter;
        match results {
            Some(results) => self.show(Stage::Results, Box::new(results)),
            None => self.next_scene(ctx),
        }
    }

    /// After a chapter's save prompt (or loading its save): its next
    /// chapter, or "To be continued".
    fn next_chapter(&mut self, ctx: &mut Ctx) {
        let next = self.chapter.as_ref().and_then(|c| c.next.clone());
        match next {
            Some(id) => {
                if let Some(campaign) = &mut self.campaign {
                    campaign.chapter.clone_from(&id);
                }
                if !self.camp(ctx, Some(id.clone())) {
                    self.start_chapter(ctx, &id);
                }
            }
            None => self.the_end(ctx),
        }
    }

    /// The screen over the flow popped with `result`: what comes next
    /// ([`show`](Self::show)); nothing when the flow is over.
    fn advance(&mut self, ctx: &mut Ctx, result: Option<ScreenResult>) {
        match self.stage {
            Stage::Mode => {
                if let Some(mode) = result.and_then(ScreenResult::take::<GameMode>) {
                    let screen = FamiliarityScreen::new(ctx);
                    self.show(Stage::Familiarity(mode), Box::new(screen));
                }
            }
            Stage::Familiarity(mode) => match result.and_then(ScreenResult::take) {
                Some(level) => {
                    self.show(Stage::Lead(mode, level), Box::new(LeadSelectScreen::new()));
                }
                None => self.show(Stage::Mode, Box::new(ModeSelectScreen::new(ctx))),
            },
            Stage::Lead(mode, level) => {
                if let Some(lead) = result.and_then(ScreenResult::take::<LeadProfile>) {
                    let mut campaign = new_campaign(&ctx.game.content, mode, lead);
                    campaign.tutorial = level;
                    self.begin(ctx, campaign);
                } else {
                    let screen = FamiliarityScreen::new(ctx);
                    self.show(Stage::Familiarity(mode), Box::new(screen));
                }
            }
            Stage::Scene | Stage::Results => self.next_scene(ctx),
            Stage::Camp => {
                if let Some(mut campaign) = result.and_then(ScreenResult::take::<Campaign>) {
                    Self::take_in(&mut campaign, ctx, self.started_at);
                    self.campaign = Some(campaign);
                }
                match self.after_camp.take() {
                    Some(chapter) => self.start_chapter(ctx, &chapter),
                    None => self.start_battle(ctx),
                }
            }
            // Fight! (leaving pops with nothing: the flow is over).
            Stage::Preparations => {
                if let Some(prep) = result.and_then(ScreenResult::take) {
                    self.prepared(prep);
                }
            }
            Stage::Battle => match result.and_then(ScreenResult::take) {
                Some(BattleEnd::Won(state, unused)) => self.won(ctx, &state, unused),
                Some(BattleEnd::Restart) => self.restart(ctx),
                Some(BattleEnd::Suspend(battle)) => self.suspend(ctx, battle),
                Some(BattleEnd::Title) | None => {}
            },
            Stage::SavePrompt => match (result.and_then(ScreenResult::take), &self.campaign) {
                (Some(true), Some(campaign)) => {
                    let picker = SlotPickerScreen::save(ctx, campaign.clone());
                    self.show(Stage::Slots(true), Box::new(picker));
                }
                _ => self.next_chapter(ctx),
            },
            Stage::Slots(saving) => match result.and_then(ScreenResult::take) {
                Some(SlotOutcome::Saved(_)) => self.next_chapter(ctx),
                Some(SlotOutcome::Loaded(_, file)) => self.begin_loaded(ctx, file.campaign),
                // Back from saving: the question again. Back from
                // loading: the title.
                None if saving => {
                    self.show(Stage::SavePrompt, Box::new(SavePromptScreen::new(ctx)));
                }
                None => {}
            },
            Stage::ToBeContinued => {}
        }
    }

    /// Takes into `campaign` (begun at `started_at`) what changed while a
    /// screen was over the flow: the mode, if the Options screen on
    /// Preparations switched it to Casual (0805); the tutorial level picked
    /// on the Options screen (0838); the lessons a battle finished
    /// ([`Ctx::campaign_lessons`], which only grows from the campaign's
    /// own, 0456); its playtime. Its saves keep them. Returns whether it
    /// switched to Casual.
    fn take_in(campaign: &mut Campaign, ctx: &Ctx, started_at: f64) -> bool {
        let casual = ctx.game.campaign_mode == Some(GameMode::Casual) && campaign.downgrade_mode();
        if let Some(level) = ctx.game.campaign_tutorial {
            campaign.tutorial = level;
        }
        if campaign.lessons_done.len() != ctx.game.campaign_lessons.len() {
            campaign.lessons_done.clone_from(&ctx.game.campaign_lessons);
        }
        let played = (ctx.clock_s - started_at).max(0.0);
        // Whole seconds; a campaign never runs anywhere near 2^53 s.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "non-negative, far below u64::MAX"
        )]
        let played = played as u64;
        campaign.playtime_s = played;
        casual
    }

    /// The screen over the flow popped: the campaign takes in what changed
    /// ([`take_in`](Self::take_in); a switch to Casual goes into the
    /// battle being prepared too). Only this way: while the campaign runs,
    /// [`Ctx`] owns its mode, tutorial level and lessons.
    fn sync(&mut self, ctx: &Ctx) {
        if let Some(campaign) = &mut self.campaign
            && Self::take_in(campaign, ctx, self.started_at)
            && let Some(fight) = &mut self.fight
        {
            fight.prep.setup.mode = GameMode::Casual;
        }
    }

    /// The campaign as a save made now would hold it: with the mode,
    /// tutorial level, lessons and playtime as they are now, not as of the
    /// last screen that popped over the flow. For tests that look at it
    /// mid-battle.
    #[cfg(any(test, feature = "harness"))]
    pub fn campaign_now(&self, ctx: &Ctx) -> Option<Campaign> {
        let mut campaign = self.campaign.clone()?;
        Self::take_in(&mut campaign, ctx, self.started_at);
        Some(campaign)
    }
}

/// The flow is opaque, and only shows itself on the frame it is pushed:
/// as its first screen, which it pushes on its first update.
impl Screen<Shuyi> for FlowScreen {
    fn name(&self) -> &'static str {
        self.next.as_ref().map_or(Self::NAME, |s| s.name())
    }

    fn update(&mut self, ctx: &mut Ctx, _input: &FrameInput) -> Transition {
        self.push_next(ctx)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        if let Some(next) = &self.next {
            next.draw(ctx, buf);
        }
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn uncovered(&mut self, ctx: &mut Ctx, result: Option<ScreenResult>) -> Transition {
        // A screen pushed over the flow before its first one (the debug
        // menu's): that one is still to come.
        if self.next.is_none() {
            self.sync(ctx);
            self.advance(ctx, result);
        }
        self.push_next(ctx)
    }
}

#[cfg(test)]
mod tests;
