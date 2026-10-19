//! What Shuyi's tools (`shuyi-cli`, ticket 1117) need from this game:
//! [`Trpg`], the game's `GameTools`. xtask is the game's tool program
//! (`shuyi.ron`'s `game`): `cargo shuyi frame-png` and `cargo shuyi
//! lang-status` run `cargo xtask frame-png` and `cargo xtask lang-status`,
//! which run the engine's commands with these.
//!
//! `frame-png` starts the game in the test Harness with the game's own
//! options: the layout picked before, the Playdate's screen (ADR-0074),
//! the map alone, the prompt start, the tips and a light for every battle.

use shuyi_cli::frame_png::{Image, Painter, Script, Shot, one_bit_picture};
use shuyi_cli::{GameArg, GameLang, GameOption, GameTools};
use trpg_content::font::ATLAS_1BIT_PATH;
use trpg_ui::harness::Harness;
use trpg_ui::input::{Device, LEFT_HANDED, Layout, PadKind, RIGHT_HANDED};
use trpg_ui::onebit::{self, FRAME_BYTES};
use trpg_ui::screen::{Ctx, Keyboard, ShuyiCtx};
use trpg_ui::{Cell, ConsoleSizes, ConsoleSpec, GlyphBuffer, Palette, Rect, UiColor};

/// The game's `frame-png` options.
const OPTIONS: [GameOption; 6] = [
    GameOption {
        flag: "--layout",
        value: Some("right|left"),
        help: "the key layout picked before (default right)",
    },
    GameOption {
        flag: "--playdate",
        value: None,
        help: "the Playdate's screen: a 50×15 console in the Playdate's\n\
               palette, drawn black and white by the one-bit renderer,\n\
               with the one-bit pictures the Playdate's build makes (1040);\n\
               no keyboard and the Playdate's buttons, as on the device\n\
               (steer it with --pad)",
    },
    GameOption {
        flag: "--map",
        value: None,
        help: "the battle's map alone over the whole screen, painted by\n\
               the game's map skin (with --playdate the Playdate's: its\n\
               battle screen keeps the PC's layout until 1038)",
    },
    GameOption {
        flag: "--prompt",
        value: None,
        help: "start as the game does (the title waits for a key or button)",
    },
    GameOption {
        flag: "--tips",
        value: None,
        help: "show the guide's tips and lessons, as the game does",
    },
    GameOption {
        flag: "--light",
        value: Some("NAME"),
        help: "fight every battle in this light of assets/data/lighting.ron,\n\
               whatever its map and its battle file say",
    },
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

    fn lang(&self) -> Result<GameLang, String> {
        let content = trpg_content::load_embedded().map_err(|e| e.to_string())?;
        Ok(GameLang {
            lang: (**content.lang).clone(),
            variants: trpg_content::lang::RULES
                .variants
                .iter()
                .map(|&variant| variant.to_owned())
                .collect(),
        })
    }
}

/// Plays `script` in a Harness started as `start` says and paints the
/// frame.
fn frame(start: &Start, script: &Script, scale: Option<u32>) -> Result<Shot, String> {
    let playdate = start.playdate.then(playdate_palette).transpose()?;
    let setup = |ctx: &mut Ctx| {
        if let Some(palette) = playdate {
            // As `crates/playdate/src/start.rs` starts the game.
            ctx.console = ConsoleSpec::PLAYDATE;
            ctx.palette = palette;
            ctx.keyboard = Keyboard::Absent;
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
    script.play(&mut h);
    let ctx = h.game().ctx();
    let map;
    let buffer = if start.shows == Shows::MapAlone {
        map = map_alone(&h)?;
        &map
    } else {
        h.game().buffer()
    };
    let image = if start.playdate {
        let bits = trpg_content::bundle::bytes(ATLAS_1BIT_PATH).unwrap_or_default();
        let font = onebit::Font::new(&ctx.game.content.font, bits)?;
        let mut frame = Box::new([0; FRAME_BYTES]);
        // The pictures the Playdate's build makes (ticket 1040).
        let pictures = crate::playdate_images::Pictures::of_the_bundle()?;
        onebit::render(buffer, &font, &pictures, &mut frame);
        one_bit_picture(&frame, scale.unwrap_or(1))
    } else {
        painted(ctx, buffer, scale.unwrap_or(2))?
    };
    Ok(Shot {
        image,
        shows: h.top_screen().to_owned(),
        warning: cfg!(feature = "private-assets").then(|| BOUGHT_ART.to_owned()),
    })
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
fn map_alone(h: &Harness) -> Result<GlyphBuffer, String> {
    let ctx = h.game().ctx();
    let console = ctx.console;
    let screen = Rect::new(0, 0, i32::from(console.cols), i32::from(console.rows));
    let scene = h
        .map_scene_in(screen)
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

#[cfg(test)]
mod tests;
