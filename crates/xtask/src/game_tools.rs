//! What Shuyi's tools (`shuyi-cli`, ticket 1117) need from this game:
//! [`Trpg`], the game's `GameTools`. xtask is the game's tool program
//! (`shuyi.ron`'s `game`): every `cargo shuyi` command that runs the game
//! (`frame-png`, `lang-status`, `run`, `scene`, the MCP server's session)
//! runs as `cargo xtask shuyi <command>`, the engine's commands with these.
//!
//! `check`, `bots` and `replay` (ticket 1193) are [`plays`]'s;
//! `autobalance` (ticket 0509) is [`balance`]'s.
//!
//! The session (ticket 1181) is the game started as `frame-png` starts it,
//! its scene the battle's map and its picture `frame-png`'s.
//!
//! `frame-png` starts the game in the test Harness with the game's own
//! options: the layout picked before, the Playdate's screen (ADR-0074),
//! the map alone, the prompt start, the tips and a light for every battle.

use shuyi_assets::ContentError;
use shuyi_cli::contract::autobalance::AutobalanceInput;
use shuyi_cli::contract::bots::{BotsInput, BotsOutput};
use shuyi_cli::contract::replay::{Record, ReplayRun};
use shuyi_cli::contract::session::{Headless, Scene, Session};
use shuyi_cli::frame_png::{Image, Painter, Script, Shot, one_bit_picture};
use shuyi_cli::{AutobalancePlan, GameArg, GameLang, GameOption, GameTools};
use shuyi_core::bots::playtest::Tries;
use trpg_content::font::ATLAS_1BIT_PATH;
use trpg_core::BattleState;
use trpg_ui::harness::{self, Core, Harness};
use trpg_ui::input::{Device, LEFT_HANDED, Layout, PadKind, RIGHT_HANDED};
use trpg_ui::onebit::{self, FRAME_BYTES};
use trpg_ui::screen::{Ctx, Keyboard, Shuyi, ShuyiCtx};
use trpg_ui::screens::BattleScreen;
use trpg_ui::{Cell, ConsoleSizes, ConsoleSpec, GlyphBuffer, Palette, Rect, UiColor};

/// The game's `frame-png` options.
const OPTIONS: [GameOption; 7] = [
    GameOption::new("--layout", "the key layout picked before (default right)")
        .with_value(Some("right|left")),
    GameOption::new(
        "--playdate",
        "the Playdate's screen: a 50×15 console in the Playdate's\n\
               palette, drawn black and white by the one-bit renderer,\n\
               with the one-bit pictures the Playdate's build makes (1040);\n\
               no keyboard and the Playdate's buttons, as on the device\n\
               (steer it with --pad)",
    ),
    GameOption::new(
        "--map",
        "the battle's map alone over the whole screen, painted by\n\
               the game's map skin (with --playdate the Playdate's), with\n\
               no windows over it",
    ),
    GameOption::new(
        "--prompt",
        "start as the game does (the title waits for a key or button)",
    ),
    GameOption::new(
        "--tips",
        "show the guide's tips and lessons, as the game does",
    ),
    GameOption::new(
        "--light",
        "fight every battle in this light of assets/data/lighting.ron,\n\
               whatever its map and its battle file say",
    )
    .with_value(Some("NAME")),
    // engine-friction: engine#2540
    GameOption::new(
        "--then-pad",
        "after the steps, press and release each button in turn, as --pad",
    )
    .with_value(Some("BUTTONS")),
];

/// What `frame-png --help` adds about the game's pictures.
const NOTES: &str = "\
The picture is 800×512 times the scale (2 by default), or 400×240 times
it with --playdate (1 by default).

With the private-assets feature the picture shows the bought art: commit
it only as a screenshot of the game (CLAUDE.md, Bought art).";

/// What the summary warns about a picture built with the bought art.
const BOUGHT_ART: &str = "built with private-assets: the picture shows bought art; commit it \
                          only as a screenshot of the game (CLAUDE.md, Bought art)";

/// What a picture shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shows {
    /// The game's screen.
    Screen,
    /// The battle's map alone, over the whole screen (`--map`).
    MapAlone,
}

/// The game's options as given.
#[derive(Debug, Clone, PartialEq)]
struct Start {
    /// The layout picked before.
    layout: Layout,
    /// The Playdate's console, palette and one-bit renderer.
    playdate: bool,
    /// What the picture shows.
    shows: Shows,
    /// Start as the game does: the title waits for a key or button.
    prompt: bool,
    /// The light every battle is fought in, instead of its own.
    light: Option<String>,
    /// Show the guide's one-time tips and checklist lessons, as `app`
    /// does.
    tips: bool,
    /// What is done after the script's steps, in turn (`--then-pad`).
    /// Ticket 1039 added it beside a `--crank` of the game's own, before
    /// the engine's script had a crank step; the engine's `--crank` now
    /// takes that flag, and 1182 deletes this.
    // engine-friction: engine#2540
    after: Vec<After>,
}

/// A step played after the script's.
#[derive(Debug, Clone, PartialEq)]
enum After {
    /// Each button pressed and released in turn.
    Pad(String),
}

impl Start {
    /// The start `options` ask for.
    fn of(options: &[GameArg]) -> Result<Self, String> {
        let mut start = Self {
            layout: RIGHT_HANDED,
            playdate: false,
            shows: Shows::Screen,
            prompt: false,
            light: None,
            tips: false,
            after: Vec::new(),
        };
        for option in options {
            let value = option.value.clone().unwrap_or_default();
            match option.flag.as_str() {
                "--layout" => {
                    start.layout = match value.as_str() {
                        "right" => RIGHT_HANDED,
                        "left" => LEFT_HANDED,
                        other => return Err(format!("--layout: right or left, not {other}")),
                    };
                }
                "--playdate" => start.playdate = true,
                "--map" => start.shows = Shows::MapAlone,
                "--prompt" => start.prompt = true,
                "--tips" => start.tips = true,
                "--light" => start.light = Some(value),
                "--then-pad" => start.after.push(After::Pad(value)),
                other => return Err(format!("unknown option: {other}")),
            }
        }
        Ok(start)
    }
}

/// This game's tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct Trpg;

impl GameTools for Trpg {
    fn frame_options(&self) -> &[GameOption] {
        &OPTIONS
    }

    fn frame_notes(&self) -> &'static str {
        NOTES
    }

    fn frame(
        &self,
        options: &[GameArg],
        script: &Script,
        scale: Option<u32>,
    ) -> Result<Shot, String> {
        frame(&Start::of(options)?, script, scale)
    }

    fn session(&self, options: &[GameArg]) -> Result<Box<dyn Session>, String> {
        Ok(Box::new(session(Start::of(options)?)?))
    }

    fn lang(&self) -> Result<GameLang, String> {
        let content = trpg_content::load_embedded().map_err(|e| e.to_string())?;
        Ok(GameLang::new(
            (**content.lang).clone(),
            trpg_content::lang::RULES
                .variants
                .iter()
                .map(|&variant| variant.to_owned())
                .collect(),
        ))
    }

    fn check(&self) -> Result<Vec<ContentError>, String> {
        Ok(plays::check())
    }

    fn replay(&self, record: &Record) -> Result<ReplayRun, String> {
        plays::replay(record)
    }

    fn bots(&self, input: &BotsInput) -> Result<BotsOutput, String> {
        // The options are about the look, which the bots never see
        // (ADR-0038 §5): checked, and the same battle played whatever they
        // say.
        Start::of(&input.start)?;
        plays::bots(input)
    }

    /// The battles, by id: what `autobalance --stage` takes.
    fn playtest_stages(&self) -> Vec<String> {
        balance::stages()
    }

    /// 0506's playtest: 100 tries from seed 1, each stopped at turn 60.
    fn playtest_defaults(&self) -> Tries {
        Tries::new(100, 1, 60)
    }

    fn autobalance_plan(
        &self,
        stage: Option<&str>,
        input: &AutobalanceInput,
    ) -> Result<AutobalancePlan, String> {
        balance::plan(&crate::repo_root(), stage, input)
    }
}

/// Plays `script` in a Harness started as `start` says and paints the
/// frame.
fn frame(start: &Start, script: &Script, scale: Option<u32>) -> Result<Shot, String> {
    let mut h = harness(start)?;
    script.play(&mut h);
    // engine-friction: engine#2540
    for step in &start.after {
        match step {
            After::Pad(buttons) => h.pad(buttons),
        };
    }
    let mut shot = Shot::new(picture(start, &h, scale)?, h.top_screen());
    shot.warning = cfg!(feature = "private-assets").then(|| BOUGHT_ART.to_owned());
    Ok(shot)
}

/// The session (`cargo shuyi run`, `scene`, the MCP server's; ticket
/// 1181): the game started as `frame` starts it, its scene the battle's
/// map and its picture `frame`'s.
fn session(start: Start) -> Result<Headless<Shuyi>, String> {
    let h = harness(&start)?.into_core();
    Ok(Headless::new(h)
        .with_scene(scene)
        .with_picture(move |h, scale| picture(&start, h, scale)))
}

/// The battle on `h`'s screen: its map scene as text, its state as data
/// ([`battle_data`]).
fn scene(h: &Core) -> Result<Scene, String> {
    let battle = h.screen::<BattleScreen>().ok_or_else(|| {
        format!(
            "no battle on screen (the top screen is `{}`)",
            h.top_screen()
        )
    })?;
    let mut scene = Scene::new(h.top_screen(), "grid map", harness::map_text(h));
    scene.data = battle_data(battle.state());
    Ok(scene)
}

/// `state` as JSON: all of it but its map, which is the scene's text (JSON
/// can't key the map's features by tile).
fn battle_data(state: &BattleState) -> serde_json::Value {
    serde_json::json!({
        "turn": state.turn(),
        "phase": state.phase(),
        "units": state.units(),
        "fallen": state.fallen(),
        "down": state.down(),
        "escaped": state.escaped(),
        "outcome": state.outcome(),
    })
}

/// A Harness started as `start` says.
fn harness(start: &Start) -> Result<Harness, String> {
    let playdate = start.playdate.then(playdate_palette).transpose()?;
    let setup = |ctx: &mut Ctx| {
        if let Some(palette) = playdate {
            // As `crates/playdate/src/start.rs` starts the game.
            ctx.console = ConsoleSpec::PLAYDATE;
            ctx.palette = palette;
            ctx.platform.keyboard = Keyboard::Absent;
            ctx.device = Device::Pad(PadKind::Playdate);
            ctx.use_playdate_buttons();
            ctx.use_playdate_skins();
        }
    };
    let h = if start.prompt {
        Harness::at_prompt_with_layout_and(start.layout, setup)
    } else {
        Harness::with_layout_and(start.layout, setup)
    };
    // The game as players get it: New Game starts the story.
    let mut h = h.with_shipped_story();
    // And as `app` shows it: with the idle motion on.
    h.ctx_mut().idle_motion = trpg_ui::screen::IdleMotion::On;
    if start.tips {
        h.with_tips().with_lessons();
    }
    if let Some(light) = &start.light {
        let content = &mut h.ctx_mut().game.content;
        if content.lighting.get(light).is_none() {
            let known = content.lighting.names();
            return Err(format!(
                "--light: no light \"{light}\"; the lights are {known}"
            ));
        }
        let battles: Vec<String> = content.battles.keys().cloned().collect();
        content
            .battle_lights
            .extend(battles.into_iter().map(|id| (id, light.clone())));
    }
    Ok(h)
}

/// `h`'s frame painted as `start` says.
fn picture(start: &Start, h: &Core, scale: Option<u32>) -> Result<Image, String> {
    let ctx = h.game().ctx();
    let map;
    let buffer = if start.shows == Shows::MapAlone {
        map = map_alone(h)?;
        &map
    } else {
        h.game().buffer()
    };
    if start.playdate {
        let bits = trpg_content::bundle::bytes(ATLAS_1BIT_PATH).unwrap_or_default();
        let font = onebit::Font::new(&ctx.game.content.font, bits)?;
        let mut frame = Box::new([0; FRAME_BYTES]);
        // The pictures the Playdate's build makes (ticket 1040).
        let pictures = crate::playdate_images::Pictures::of_the_bundle()?;
        onebit::render(buffer, &font, &pictures, &mut frame);
        Ok(one_bit_picture(&frame, scale.unwrap_or(1)))
    } else {
        painted(ctx, buffer, scale.unwrap_or(2))
    }
}

/// `buffer` painted with the game's font and images, each console pixel
/// `scale` × `scale`.
fn painted(ctx: &Ctx, buffer: &GlyphBuffer, scale: u32) -> Result<Image, String> {
    let painter = Painter::new(
        ctx.game.content.font.clone(),
        trpg_content::bundle::bytes(trpg_content::font::ATLAS_PNG_PATH).unwrap_or_default(),
        &ctx.game.content.images,
        trpg_content::bundle::bytes,
        ctx.palette.get(UiColor::Black),
    )?;
    Ok(painter.render(buffer, scale))
}

/// The battle on `h`'s screen, its map alone over the whole console as the
/// game's map skin paints it, the view centred on the cursor (`--map`).
fn map_alone(h: &Core) -> Result<GlyphBuffer, String> {
    let ctx = h.game().ctx();
    let console = ctx.console;
    let screen = Rect::new(0, 0, i32::from(console.cols), i32::from(console.rows));
    let scene = harness::map_scene_in(h, screen)
        .ok_or_else(|| format!("--map: no battle on screen ({})", h.top_screen()))?;
    let blank = Cell::new(
        ' ',
        ctx.palette.get(UiColor::Text),
        ctx.palette.get(UiColor::Black),
    );
    let mut buf = GlyphBuffer::with_spec(console, blank);
    ctx.game.map_skin.paint(ctx, &scene, screen, &mut buf);
    Ok(buf)
}

/// The Playdate's palette (ADR-0074), over the default theme's names.
fn playdate_palette() -> Result<Palette, String> {
    let shown = |errors: Vec<trpg_content::ContentError>| {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    };
    let base = trpg_content::palette::load().map_err(shown)?;
    let def = trpg_content::ink::load_playdate(&base).map_err(shown)?;
    Palette::new::<UiColor>(&def)
        .map_err(|missing| format!("the Playdate palette lacks {missing:?}"))
}

mod balance;
mod plays;

#[cfg(test)]
mod tests;
