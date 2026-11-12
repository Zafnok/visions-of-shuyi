//! Small scripted battles for this crate's tests: generic swordsmen on the
//! plain row (`y = 4`, `x` 0 to 8) of the game's test map, with the game's
//! own tables.

use std::sync::LazyLock;

use trpg_content::Content;
use trpg_core::{
    BattlePack, BattleSetup, BattleState, ClassId, Command, Faction, GameMode, ItemId, LoadoutDef,
    Objective, Phase, Pos, Stock, Unit, UnitAction, UnitId,
};

use crate::BattleMeasures;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| trpg_content::load_embedded().unwrap_or_else(|e| panic!("content: {e}")));

/// The game's content.
pub(crate) fn content() -> &'static Content {
    &CONTENT
}

/// The test files' story, for the tests of how the bots work (ADR-0118):
/// the content with the test files' chapters and battles only, and a New
/// Game of the test world's three chapters (`test_wm1`, `test_wm2`,
/// `test_wm3`, in that order) with the lead and the test knight. The
/// first chapter's victory has a scene in which the test rogue joins, so
/// the army counts a recruit from the second chapter on.
pub(crate) fn test_story() -> Content {
    let mut content = content().clone();
    let test = |id: &String| id.starts_with("test") || id == "quick";
    content.battles.retain(|id, _| test(id));
    content.chapters.retain(|id, _| test(id));
    content.new_game = trpg_content::NewGameDef {
        first_chapter: "test_wm1".into(),
        ..trpg_content::NewGameDef::test_chapter()
    };
    let id = TEST_JOINS.to_owned();
    let joins = trpg_content::Stage::Joins(trpg_core::CharacterId(TEST_RECRUIT.into()));
    let steps = vec![trpg_content::Step::Directive(joins)];
    let scene = trpg_content::Scene {
        id: id.clone(),
        steps,
    };
    content.dialogue.scenes.insert(id.clone(), scene);
    let first = content.chapters.get_mut("test_wm1");
    first
        .unwrap_or_else(|| panic!("no test_wm1"))
        .victory_scenes
        .push(id);
    content
}

/// The scene of [`test_story`] in which [`TEST_RECRUIT`] joins.
pub(crate) const TEST_JOINS: &str = "test_rogue_joins";
/// Who joins in [`test_story`]'s first chapter's victory.
pub(crate) const TEST_RECRUIT: &str = "test_rogue";

/// The plain row's tile `x`.
pub(crate) fn tile(x: i32) -> Pos {
    Pos::new(x, 4)
}

fn swordsman(id: u32, faction: Faction, x: i32) -> Unit {
    let content = content();
    let loadout = LoadoutDef {
        weapons: vec![ItemId::new("iron_sword")],
        ..LoadoutDef::default()
    };
    let class = ClassId("swordsman".into());
    Unit::generic(UnitId(id), &class, &content.classes, 1, faction, tile(x))
        .unwrap_or_else(|e| panic!("swordsman: {e}"))
        .with_loadout(&loadout, &content.classes, &content.items)
        .unwrap_or_else(|e| panic!("loadout: {e}"))
}

/// A player swordsman at `x`.
pub(crate) fn player(id: u32, x: i32) -> Unit {
    swordsman(id, Faction::Player, x)
}

/// An enemy swordsman at `x`.
pub(crate) fn enemy(id: u32, x: i32) -> Unit {
    swordsman(id, Faction::Enemy, x)
}

/// `unit`, never missing and felling anything it strikes.
pub(crate) fn rigged(mut unit: Unit) -> Unit {
    unit.stats.dex = 99;
    unit.stats.str = 99;
    unit
}

/// `unit` at 1 HP.
pub(crate) fn weak(mut unit: Unit) -> Unit {
    unit.hp = 1;
    unit
}

/// A rout of `units`, with a potion, an elixir and a potion in the pack.
pub(crate) fn start(units: Vec<Unit>) -> BattleState {
    BattleState::new(setup(units)).0
}

/// The setup of [`start`], to change before starting it.
pub(crate) fn setup(units: Vec<Unit>) -> BattleSetup {
    let content = content();
    let tables = content.tables();
    let pack = ["potion", "elixir", "potion"];
    BattleSetup {
        map: content.maps["test_small"].map.clone(),
        terrain: tables.terrain,
        classes: tables.classes,
        items: tables.items,
        spells: tables.spells,
        skills: tables.skills,
        arts: tables.arts,
        supports: tables.supports,
        bonds: trpg_core::SupportBook::default(),
        pack: BattlePack {
            items: pack.iter().map(|i| ItemId::new(i)).collect(),
            cap: pack.len(),
        },
        gold: 0,
        stock: Stock::default(),
        units,
        reinforcements: vec![],
        objective: Objective::Rout { turn_limit: None },
        rewind_charges: 0,
        seed: 1,
        triggers: vec![],
        battle_notes: vec![],
        village: None,
        mode: GameMode::Classic,
    }
}

/// `unit`, a lord.
pub(crate) fn lord(mut unit: Unit) -> Unit {
    unit.is_lord = true;
    unit
}

/// `unit`, which never moves and strikes only what stands next to it.
pub(crate) fn stationary(mut unit: Unit) -> Unit {
    unit.ai = trpg_core::AiBehavior::Stationary;
    unit
}

/// Attack `target` with the first weapon.
pub(crate) fn attack(target: u32) -> UnitAction {
    UnitAction::Attack {
        target: UnitId(target),
        slot: 0,
        active: None,
        art: None,
    }
}

/// A battle played by hand, every command observed.
pub(crate) struct Scripted {
    pub(crate) state: BattleState,
    pub(crate) measures: BattleMeasures,
}

impl Scripted {
    pub(crate) fn new(state: BattleState) -> Self {
        let measures = BattleMeasures::new(&state);
        Self { state, measures }
    }

    /// Applies `command`, which must be accepted.
    pub(crate) fn step(&mut self, command: &Command) {
        let events = self
            .state
            .apply(command)
            .unwrap_or_else(|e| panic!("{command:?}: {e}"));
        self.measures.observe(&self.state, &events);
    }

    /// Unit `id` does `action` at `x`.
    pub(crate) fn act(&mut self, id: u32, x: i32, action: UnitAction) {
        self.step(&Command::Act {
            unit: UnitId(id),
            dest: tile(x),
            action,
        });
    }

    pub(crate) fn end_phase(&mut self) {
        self.step(&Command::EndPhase);
    }

    /// Ends phases, nobody acting, until the next Player phase.
    pub(crate) fn next_player_phase(&mut self) {
        self.end_phase();
        while self.state.phase() != Phase::Player {
            self.end_phase();
        }
    }
}

/// Battle `id` of the game's (or the test files') battles, as the playtest
/// fields it: the characters of its player slots, fresh.
pub(crate) fn stage(id: &str) -> crate::BattleStage {
    let content = content();
    let def = &content.battles[id];
    let setup = |mode| {
        let lead = trpg_core::LeadProfile::new("Lead", trpg_core::LeadGender::Male);
        let campaign = trpg_content::battle_campaign(content, def, mode, lead);
        campaign.battle_setup(def, &content.tables())
    };
    crate::BattleStage {
        classic: setup(GameMode::Classic),
        casual: setup(GameMode::Casual),
    }
}
