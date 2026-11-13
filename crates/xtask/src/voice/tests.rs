use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use trpg_content::dialogue::{self as dlg, DialogueTable};
use trpg_content::names::Names;
use trpg_content::voice::{MadeBy, Variant, VoiceClip};

use super::audit;
use shuyi_assets::voice::CastVoice;
use shuyi_assets::voice::plan::Why;

use super::files::{self, Cast, Reader, Say, Sent, SentMap};
use super::plan::{self as voice_plan, Inputs, Plan, plan};
use super::synth::{self, Level, Tools};
use super::*;

// ---- The plan, as a pure function ----

const SCRIPT: &str = "@scene a
> (slowly): The rain fell on {n:town}.
@left test_lord neutral
@right test_knight neutral
test_lord: Hello there.
test_knight (angrily): You're late.
@left lead neutral
lead: Let's move.
> {lead|The exile} grips {their} sword.
@choice
* wry: Fine.
  test_knight: Good.
* blunt: No.
@endchoice
@end
@scene b
@left test_lord neutral
test_lord: Elsewhere.
@end
";

fn names(town: &str) -> Names {
    Names {
        names: BTreeMap::from([("town".to_owned(), town.to_owned())]),
    }
}

fn dialogue(script: &str) -> DialogueTable {
    let scripts = dlg::scripts_from_sources(&[("t.dlg", script)], None, None, None, None).unwrap();
    dlg::written(&scripts.table, &scripts.stand_ins)
}

fn cast() -> Cast {
    let voice = |clip: &str, reader| CastVoice::new(clip, reader);
    Cast::new(BTreeMap::from([
        (">".to_owned(), voice("narrator", Reader::Qwen3)),
        ("test_lord".to_owned(), voice("rival", Reader::Qwen3)),
        (
            "test_knight".to_owned(),
            voice("retainer", Reader::Chatterbox),
        ),
        ("lead.m".to_owned(), voice("lead_m", Reader::Qwen3)),
        ("lead.f".to_owned(), voice("lead_f", Reader::Qwen3)),
    ]))
}

/// Everything a plan is made from, each part changeable.
struct World {
    dialogue: DialogueTable,
    scenes: BTreeSet<String>,
    names: Names,
    cast: Cast,
    say: Say,
    clips: Vec<VoiceClip>,
    sent: SentMap,
    references: BTreeMap<String, String>,
}

impl World {
    fn new() -> Self {
        let cast = cast();
        let references = cast
            .voices
            .values()
            .map(|v| (v.voice.clone(), "00000000".to_owned()))
            .collect();
        Self {
            dialogue: dialogue(SCRIPT),
            scenes: BTreeSet::from(["a".to_owned()]),
            names: names("Heth"),
            cast,
            say: Say::new(),
            clips: Vec::new(),
            sent: SentMap::new(),
            references,
        }
    }

    fn plan(&self) -> Plan {
        plan(&Inputs {
            dialogue: &self.dialogue,
            scenes: &self.scenes,
            names: &self.names,
            cast: &self.cast,
            say: &self.say,
            clips: &self.clips,
            sent: &self.sent,
            references: &self.references,
            tool: &|reader, directed| match (reader, directed) {
                (_, true) => "directed".to_owned(),
                (r, false) => format!("{r:?}"),
            },
        })
    }

    /// Makes every clip the plan says to, as `generate` records them.
    fn made(mut self) -> Self {
        let plan = self.plan();
        for &(i, _) in &plan.make {
            let w = &plan.wanted[i];
            let voice = &self.cast.voices[&w.key];
            let tool = if w.direction.is_some() {
                "directed".to_owned()
            } else {
                format!("{:?}", voice.settings)
            };
            self.clips
                .retain(|c| (&c.line, c.variant) != (&w.line, w.variant));
            self.clips.push(VoiceClip {
                line: w.line.clone(),
                variant: w.variant,
                spoken: w.spoken.clone(),
                direction: w.direction.clone(),
                voice: voice.voice.clone(),
                made_by: MadeBy::Generated {
                    tool,
                    model: "m".to_owned(),
                    date: "2026-11-03".to_owned(),
                },
            });
            let sent = Sent {
                text: self.sent_of(w),
                reference: self.references[&voice.voice].clone(),
            };
            self.sent.insert(w.file(), sent);
        }
        self.clips.retain(|c| !plan.remove.contains(c));
        self
    }

    /// The words sent to the reader for `w`.
    fn sent_of(&self, w: &voice_plan::Wanted) -> String {
        voice_plan::sent(&w.spoken, &self.say, &self.names)
    }
}

/// The plan's clips to make, as `file why` rows.
fn to_make(plan: &Plan) -> Vec<String> {
    plan.make
        .iter()
        .map(|(i, why)| format!("{} {why:?}", plan.wanted[*i].file()))
        .collect()
}

#[test]
fn every_spoken_and_narrated_line_needs_a_clip_and_replies_none() {
    let plan = World::new().plan();
    let rows: Vec<String> = plan
        .wanted
        .iter()
        .map(|w| {
            format!(
                "{} {} {} | {}",
                w.file(),
                w.key,
                w.direction.as_deref().unwrap_or("-"),
                w.spoken
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            "a/a_5912b92e.ogg > slowly | The rain fell on Heth.",
            "a/a_2748b275.ogg test_lord - | Hello there.",
            "a/a_21833604.ogg test_knight angrily | You're late.",
            "a/a_36a692b7.m.ogg lead.m - | Let's move.",
            "a/a_36a692b7.f.ogg lead.f - | Let's move.",
            "a/a_c9e4741d.m.ogg > - | The exile grips his sword.",
            "a/a_c9e4741d.f.ogg > - | The exile grips her sword.",
            "a/a_16d9be18.ogg test_knight - | Good.",
        ]
    );
    assert!(plan.make.iter().all(|(_, why)| *why == Why::Missing));
    assert_eq!(plan.make.len(), 8);
    assert_eq!(
        plan.wanted[3].before,
        Some(("test_knight".to_owned(), "You're late.".to_owned()))
    );
}

#[test]
fn once_made_nothing_is_left_to_do() {
    let plan = World::new().made().plan();
    assert_eq!(plan.wanted.len(), 8);
    assert_eq!(
        (plan.make, plan.record, plan.remove, plan.uncast),
        (vec![], vec![], vec![], vec![])
    );
}

#[test]
fn rewording_a_line_makes_its_new_clip_and_removes_the_old() {
    let mut world = World::new().made();
    world.dialogue = dialogue(&SCRIPT.replace("Hello there.", "Hello again."));
    let plan = world.plan();
    assert_eq!(to_make(&plan), ["a/a_72405b07.ogg Missing"]);
    let removed: Vec<String> = plan.remove.iter().map(|c| c.spoken.clone()).collect();
    assert_eq!(removed, ["Hello there."]);
}

#[test]
fn renaming_a_name_remakes_only_the_lines_that_say_it() {
    let mut world = World::new().made();
    world.names = names("Hethel");
    assert_eq!(to_make(&world.plan()), ["a/a_5912b92e.ogg Words"]);
}

#[test]
fn changing_a_direction_remakes_only_that_line() {
    let mut world = World::new().made();
    world.dialogue = dialogue(&SCRIPT.replace("(angrily)", "(wearily)"));
    assert_eq!(to_make(&world.plan()), ["a/a_21833604.ogg Direction"]);
}

#[test]
fn changing_a_speakers_voice_remakes_only_their_lines() {
    let mut world = World::new().made();
    world.cast.voices.get_mut("test_lord").unwrap().voice = "messenger".to_owned();
    world
        .references
        .insert("messenger".to_owned(), "1".to_owned());
    assert_eq!(to_make(&world.plan()), ["a/a_2748b275.ogg Recast"]);

    // Another reader, or a new recording of the same clip, is a new voice.
    let lord = |why: &str| [format!("a/a_2748b275.ogg Voice({why:?})")];
    let mut world = World::new().made();
    world.cast.voices.get_mut("test_lord").unwrap().settings = Reader::Chatterbox;
    assert_eq!(to_make(&world.plan()), lord(voice_plan::READER));
    let mut world = World::new().made();
    world
        .references
        .insert("rival".to_owned(), "ffffffff".to_owned());
    assert_eq!(to_make(&world.plan()), lord(voice_plan::REFERENCE));
}

#[test]
fn a_new_spelling_in_say_ron_remakes_the_lines_with_that_word_and_keeps_the_words() {
    let mut world = World::new().made();
    world.say.insert("town".to_owned(), "Hayth".to_owned());
    let plan = world.plan();
    let spelling = format!("a/a_5912b92e.ogg Voice({:?})", voice_plan::SPELLING);
    assert_eq!(to_make(&plan), [spelling]);
    let w = &plan.wanted[plan.make[0].0];
    assert_eq!(
        (w.spoken.as_str(), world.sent_of(w).as_str()),
        ("The rain fell on Heth.", "The rain fell on Hayth.")
    );
}

/// Acceptance (0762): a line's trailing-offs and breaks reach the reader
/// as commas and full stops; the line on screen and the manifest's words
/// keep them. A clip made before the change is made again.
#[test]
fn trailing_offs_and_breaks_are_sent_as_punctuation_and_the_line_keeps_them() {
    let script = "@scene a\n@left test_lord neutral\ntest_lord: Wait... no. Not yet.\n\
                  test_lord: Aye. ...Fold it. I folded it and she didn't even-- never mind.\n@end\n";
    let mut world = World::new();
    world.dialogue = dialogue(script);
    let plan = world.plan();
    let sent: Vec<String> = plan.wanted.iter().map(|w| world.sent_of(w)).collect();
    let said: Vec<(&str, &str)> = plan
        .wanted
        .iter()
        .zip(&sent)
        .map(|(w, sent)| (w.spoken.as_str(), sent.as_str()))
        .collect();
    assert_eq!(
        said,
        [
            ("Wait... no. Not yet.", "Wait, no. Not yet."),
            (
                "Aye. ...Fold it. I folded it and she didn't even-- never mind.",
                "Aye. Fold it. I folded it and she didn't even\u{2014} never mind."
            ),
        ]
    );
    let shown = world.dialogue.get("a").unwrap().lines()[0].text;
    assert_eq!(shown, "Wait... no. Not yet.");

    // Made when the words were sent as written: made again, same words.
    let mut world = world.made();
    assert_eq!(world.plan().make, []);
    for clip in &world.clips {
        let file = trpg_content::voice::clip_file(&clip.line, clip.variant);
        world.sent.get_mut(&file).unwrap().text = clip.spoken.clone();
    }
    let plan = world.plan();
    assert_eq!(plan.make.len(), 2);
    let spelling = Why::Voice(voice_plan::SPELLING.to_owned());
    assert!(plan.make.iter().all(|(_, why)| *why == spelling));
}

#[test]
fn a_reader_is_sent_trailing_offs_and_breaks_as_punctuation() {
    let cases = [
        // After a sentence's end the mark goes: the full stop pauses.
        ("Aye. ...Fold it.", "Aye. Fold it."),
        (
            "Whose cup? ...It's the cook's.",
            "Whose cup? It's the cook's.",
        ),
        (
            "\"Go home.\" ...That's not it.",
            "\"Go home.\" That's not it.",
        ),
        ("Keeper: ...the bell.", "Keeper: the bell."),
        // At the start or the end of the line.
        ("...Aye. Well.", "Aye. Well."),
        ("Should've waited for--", "Should've waited for."),
        ("\"I never--\"", "\"I never.\""),
        // A trailing-off mid-line: a comma, or a full stop before a new
        // sentence.
        ("Wait... no. Not yet.", "Wait, no. Not yet."),
        ("Wait... Bran!", "Wait. Bran!"),
        ("Wait... I'll go.", "Wait, I'll go."),
        ("Go. ...", "Go."),
        // A break mid-line is an em dash, whatever follows.
        ("Lad-- no. Leave it.", "Lad\u{2014} no. Leave it."),
        ("No-- I mean-- yes.", "No\u{2014} I mean\u{2014} yes."),
        ("all they-- Your Grace.", "all they\u{2014} Your Grace."),
        (
            "it's no trouble-- ...No. I won't.",
            "it's no trouble\u{2014} No. I won't.",
        ),
        ("Help me up -- no, don't.", "Help me up\u{2014} no, don't."),
        // Before other punctuation, and the one-character ellipsis.
        ("What...? Now?", "What? Now?"),
        ("Wait\u{2026} no.", "Wait, no."),
        ("Well\u{2026}", "Well."),
        // Nothing else changes.
        ("The toll-fort, at noon. Go!", "The toll-fort, at noon. Go!"),
        ("", ""),
    ];
    for (written, sent) in cases {
        assert_eq!(files::for_reader(written), sent, "{written}");
    }
}

#[test]
fn a_recorded_clip_is_listed_for_recording_and_never_made() {
    let mut world = World::new().made();
    for clip in &mut world.clips {
        clip.made_by = MadeBy::Recorded {
            actor: "A. Person".to_owned(),
        };
    }
    // A new voice doesn't touch a recording; new words or direction do.
    world.cast.voices.get_mut("test_lord").unwrap().voice = "messenger".to_owned();
    world.dialogue = dialogue(&SCRIPT.replace("(angrily)", "(wearily)"));
    let plan = world.plan();
    assert_eq!(plan.make, []);
    let record: Vec<String> = plan
        .record
        .iter()
        .map(|(i, why)| format!("{} {why:?}", plan.wanted[*i].file()))
        .collect();
    assert_eq!(record, ["a/a_21833604.ogg Direction"]);
}

#[test]
fn clips_of_gone_lines_go_and_clips_of_scenes_not_voiced_stay() {
    let mut world = World::new().made();
    let mut gone = world.clips[0].clone();
    gone.line = dlg::LineId::new("a_00000000".to_owned());
    let mut elsewhere = world.clips[1].clone();
    elsewhere.line = dialogue(SCRIPT).get("b").unwrap().lines()[0].id.clone();
    world.clips.extend([gone.clone(), elsewhere]);
    assert_eq!(world.plan().remove, [gone]);
}

#[test]
fn a_line_whose_clips_change_kind_loses_the_old_ones() {
    let mut world = World::new().made();
    world.dialogue = dialogue(&SCRIPT.replace("grips {their} sword", "grips the sword"));
    let plan = world.plan();
    assert_eq!(to_make(&plan), ["a/a_730ca1ea.ogg Missing"]);
    let removed: Vec<Variant> = plan.remove.iter().map(|c| c.variant).collect();
    assert_eq!(removed, [Variant::M, Variant::F]);
}

#[test]
fn a_speaker_without_a_voice_is_listed_and_keeps_their_clips() {
    let mut world = World::new().made();
    world.cast.voices.remove("test_knight");
    let plan = world.plan();
    let uncast: Vec<&str> = plan.uncast.iter().map(|u| u.speaker.as_str()).collect();
    assert_eq!(uncast, ["test_knight", "test_knight"]);
    assert_eq!((plan.make.len(), plan.remove.len()), (0, 0));
}

// ---- The files ----

#[test]
fn the_cast_names_characters_narration_and_the_leads_voices() {
    let source = "(voices: {\">\": (voice: \"narrator\", settings: Qwen3), \"lead.m\": (voice: \
                  \"lead_m\", settings: Qwen3), \"retainer\": (voice: \"retainer\", settings: \
                  Chatterbox)})";
    let speaks = |id: &str| id == "retainer" || id == "lead";
    let cast = files::cast(source, speaks).unwrap();
    assert_eq!(cast.voices["retainer"].settings, Reader::Chatterbox);
    assert_eq!(cast.voices["lead.m"].voice, "lead_m");
    let errors = files::cast(
        "(voices: {\"nobody\": (voice: \"x\", settings: Qwen3)})",
        speaks,
    )
    .unwrap_err();
    assert!(errors.starts_with(files::CAST_FILE), "{errors}");
}

/// The game's test voice folder keeps the tool's form of the cast.
#[test]
fn the_test_voice_folders_cast_is_read_by_the_tool() {
    let path = root()
        .join("crates/app/tests/voice/en")
        .join(files::CAST_FILE);
    let source = fs::read_to_string(&path).unwrap();
    let characters = content().characters;
    let cast = files::cast(&source, |id| {
        characters.can_speak(&trpg_core::CharacterId(id.to_owned()))
    })
    .unwrap();
    assert_eq!(cast.voices.len(), 3);
    assert!(cast.voices.values().all(|v| v.voice == "test_tone"));
}

#[test]
fn respelling_replaces_whole_words_and_names_by_id() {
    let say = files::say("{\"town\": \"Hayth\", \"Bran\": \"Brahn\"}").unwrap();
    let name = |id: &str| (id == "town").then(|| "Heth".to_owned());
    assert_eq!(
        files::respell(
            "Bran left Heth; Brandon and Hethers stayed. Bran's bow.",
            &say,
            name
        ),
        "Brahn left Hayth; Brandon and Hethers stayed. Bran's bow."
    );
}

#[test]
fn the_written_manifest_is_one_the_game_accepts() {
    let world = World::new().made();
    let mut clips = world.clips.clone();
    clips[0].made_by = MadeBy::Recorded {
        actor: "A \"Quoted\" Person".to_owned(),
    };
    let text = files::manifest_text(&clips);
    assert_eq!(files::manifest(&text).unwrap().len(), clips.len());
    let game = trpg_content::voice::from_source("voice.ron", &text, &world.dialogue).unwrap();
    let mut sorted = clips.clone();
    sorted.sort_by(|a, b| (a.line.as_str(), a.variant).cmp(&(b.line.as_str(), b.variant)));
    assert_eq!(game.clips, sorted);
    let sent = files::sent_text(&world.sent);
    assert_eq!(files::sent(&sent).unwrap(), world.sent);
    assert!(!sent.contains('\r'), "{sent:?}");
}

#[test]
fn fingerprints_are_fnv1a_32() {
    assert_eq!(files::fingerprint(b""), "811c9dc5");
    assert_eq!(files::fingerprint(b"a"), "e40c292c");
}

// ---- Words, levels and dates ----

#[test]
fn a_take_says_the_words_when_they_are_nearly_the_same() {
    assert!(synth::says(" You're late.", "You're late."));
    assert!(synth::says("Halwen rides at dawn", "Halwin rides at dawn!"));
    assert!(!synth::says("You are late, sir", "You're late."));
    assert!(!synth::says("", "No."));
    assert!(synth::says("", "..."));
}

#[test]
fn a_short_line_passes_when_its_words_sound_the_same() {
    // 0722: the transcriber spells a name's "Mace's." its own ways.
    for heard in ["Mases.", "Maces", "Mays."] {
        assert!(synth::says(heard, "Mace's."), "{heard}");
    }
    assert!(synth::says("Kales", "Kael's!"));
    assert!(!synth::says("Bases.", "Mace's."));
    assert!(!synth::says("Mace is.", "Mace's."));
    assert!(!synth::says("Hold the boat.", "Hold the gate."));
    // Sounding the same is not enough past a few words.
    let long = "Mace's, Kael's, Halwin's, Ellis's, Ferro's.";
    let heard = "Mases, Kales, Halwens, Elises, Feros.";
    assert!(!synth::says(heard, long));
    assert!(synth::says(
        "Mases, Kales, Halwens, Elises.",
        "Mace's, Kael's, Halwin's, Ellis's."
    ));
}

#[test]
fn a_words_sound_spells_alike_letters_one_way() {
    assert_eq!(synth::sound("dace's"), "ds");
    assert_eq!(synth::sound("dases"), "ds");
    assert_eq!(synth::sound("evan"), "avn");
    assert_eq!(synth::sound("ivan"), "avn");
    assert_eq!(synth::sound("phlox"), "flks");
    assert_eq!(synth::sound("shack"), "xk");
    assert_eq!(synth::sound("thicket"), "0kt");
    assert_eq!(synth::sound("quiz"), "ks");
    assert_eq!(synth::sound("chirp"), "xrp");
    assert_eq!(synth::sound(""), "");
}

#[test]
fn undirected_clips_meet_one_level_and_directed_ones_keep_their_distance() {
    let at = |lufs, peak| Level { lufs, peak };
    assert!((synth::gain_for(at(-24.0, -10.0), synth::VOICE_LUFS) - 6.0).abs() < 1e-9);
    // The peak ceiling holds a loud peak back.
    assert!((synth::gain_for(at(-24.0, -3.0), synth::VOICE_LUFS) - 2.0).abs() < 1e-9);
    let voice = at(-20.0, -5.0);
    assert!(
        (synth::directed_target(at(-30.0, -15.0), voice) - (synth::VOICE_LUFS - 10.0)).abs() < 1e-9
    );
    let whisper = synth::directed_target(at(-45.0, -30.0), voice);
    assert!((whisper - (synth::VOICE_LUFS + synth::DIRECTED_FLOOR_DB)).abs() < 1e-9);
    let shout = synth::directed_target(at(-5.0, -1.0), voice);
    assert!((shout - (synth::VOICE_LUFS + synth::DIRECTED_CEILING_DB)).abs() < 1e-9);
}

#[test]
fn the_level_is_read_from_ffmpegs_summary() {
    let log = "[Parsed_ebur128_0] t: 0.1 M: -40.1 S:-120.7 I: -40.1 LUFS\n\
               [Parsed_ebur128_0] Summary:\n\n  Integrated loudness:\n    I:         -21.3 LUFS\n    \
               Threshold: -31.4 LUFS\n\n  True peak:\n    Peak:       -2.5 dBFS\n";
    assert_eq!(
        synth::parse_level(log),
        Some(Level {
            lufs: -21.3,
            peak: -2.5
        })
    );
    assert_eq!(synth::parse_level("no summary"), None);
    let silent = "Summary:\n I: -70.0 LUFS\n Peak: -inf dBFS";
    assert_eq!(
        synth::parse_level(silent).map(|l| l.peak),
        Some(f64::NEG_INFINITY)
    );
    assert!(synth::trim_filters().contains("start_silence=0.1"));
}

#[test]
fn dates_are_civil() {
    assert_eq!(civil_date(0), "1970-01-01");
    assert_eq!(civil_date(20_760), "2026-11-03");
    assert_eq!(civil_date(11_016), "2000-02-29");
    assert_eq!(today().len(), 10);
}

#[test]
fn the_arguments_parse() {
    let args = |a: &[&str]| parse(&a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>());
    let status = args(&["status", "--scene", "test", "--lang", "ja"]).unwrap();
    assert_eq!(
        (
            status.command,
            status.scene.as_deref(),
            status.lang.as_str()
        ),
        (Command::Status, Some("test"), "ja")
    );
    let generate = args(&["generate", "--dry-run", "--dir", "x", "--lab", "y"]).unwrap();
    assert_eq!(generate.command, Command::Generate { dry_run: true });
    assert_eq!(
        (generate.dir, generate.lab),
        (Some("x".into()), Some("y".into()))
    );
    let script = args(&["script", "--character", "retainer"]).unwrap();
    assert_eq!(
        script.command,
        Command::Script {
            character: "retainer".to_owned()
        }
    );
    assert!(args(&["script"]).is_err());
    assert!(args(&["status", "--dry-run"]).is_err());
    assert!(args(&["status", "--scene"]).is_err());
    assert!(args(&["play"]).is_err());
}

// ---- A run in a temporary folder, with the fake tools ----

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The test scene's cast, with real reference clips.
const TEST_CAST: &str = "(voices: {\n\
    \">\": (voice: \"narrator\", settings: Qwen3),\n\
    \"lead.m\": (voice: \"lead_m\", settings: Qwen3),\n\
    \"lead.f\": (voice: \"lead_f\", settings: Qwen3),\n\
    \"test_knight\": (voice: \"retainer\", settings: Chatterbox),\n\
    \"test_lord\": (voice: \"rival\", settings: Qwen3),\n\
    \"test_archer\": (voice: \"poacher\", settings: Chatterbox),\n\
})";

/// A fresh voice folder named `name` with the test cast.
fn folder(name: &str) -> Folder {
    let dir = std::env::temp_dir().join(format!("xtask-voice-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(files::CAST_FILE), TEST_CAST).unwrap();
    Folder {
        dir,
        refs: root().join(REFS_DIR),
    }
}

fn content() -> trpg_content::Content {
    trpg_content::load_embedded().unwrap()
}

fn test_scene(content: &trpg_content::Content) -> Scripts<'_> {
    Scripts::new(content, BTreeSet::from(["test".to_owned()]))
}

/// The `.ogg` files under `dir`, relative to it.
fn oggs(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for scene in fs::read_dir(dir).unwrap().filter_map(Result::ok) {
        if scene.path().is_dir() {
            for file in fs::read_dir(scene.path()).unwrap().filter_map(Result::ok) {
                out.push(format!(
                    "{}/{}",
                    scene.file_name().to_string_lossy(),
                    file.file_name().to_string_lossy()
                ));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn generate_makes_one_clip_per_voiced_line_and_a_second_run_none() {
    let content = content();
    let scripts = test_scene(&content);
    let folder = folder("twice");
    let mut tools = Tools::fake(&[]);
    let report = generate(&folder, &scripts, &mut tools, false, "2026-11-03").unwrap();
    // 12 lines in one voice, the lead's line in two voices, and a
    // narration line that says "his" or "her"; the 3 replies are silent.
    assert!(
        report.starts_with("voice: made 16 clips, removed 0"),
        "{report}"
    );
    assert_eq!(oggs(&folder.dir).len(), 16);
    // The game plays every one of them.
    let manifest = fs::read_to_string(folder.dir.join(MANIFEST_FILE)).unwrap();
    let game = trpg_content::voice::from_source("voice.ron", &manifest, &content.dialogue).unwrap();
    let playable = trpg_content::voice::playable(&game, &scripts.dialogue, &content.names);
    assert_eq!(playable.len(), 16);
    assert!(
        manifest.contains("direction: Some(\"warmly, teasing\")"),
        "{manifest}"
    );
    assert!(manifest.contains("tool: \"test tones\""), "{manifest}");

    let again = generate(&folder, &scripts, &mut tools, false, "2026-11-04").unwrap();
    assert!(
        again.starts_with("voice: made 0 clips, removed 0"),
        "{again}"
    );
    let status = status(&folder, &scripts, &tools).unwrap();
    assert_eq!(
        status,
        "0 clips to make, 0 to re-record, 0 to remove, 0 lines with no voice"
    );
    let _ = fs::remove_dir_all(&folder.dir);
}

#[test]
fn a_dry_run_counts_and_writes_nothing() {
    let content = content();
    let folder = folder("dry");
    let report = generate(
        &folder,
        &test_scene(&content),
        &mut Tools::fake(&[]),
        true,
        "2026-11-03",
    )
    .unwrap();
    assert!(
        report.starts_with("voice --dry-run: would make 16 clips: 14 undirected ("),
        "{report}"
    );
    // "Better late than... well." is sent as "Better late than, well."
    assert!(
        report.contains("and 2 directed (63 characters)"),
        "{report}"
    );
    assert!(report.ends_with("would remove 0; made nothing"), "{report}");
    let left: Vec<_> = fs::read_dir(&folder.dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name())
        .collect();
    assert_eq!(left, [files::CAST_FILE]);
    let _ = fs::remove_dir_all(&folder.dir);
}

#[test]
fn a_recorded_clip_is_never_overwritten_and_a_line_with_no_good_take_is_not_saved() {
    let content = content();
    let scripts = test_scene(&content);
    let folder = folder("recorded");
    generate(
        &folder,
        &scripts,
        &mut Tools::fake(&[]),
        false,
        "2026-11-03",
    )
    .unwrap();
    // One clip is an actor's, recorded before its line was reworded; one
    // is gone, and its line takes no good take.
    let path = folder.dir.join(MANIFEST_FILE);
    let mut clips = files::manifest(&fs::read_to_string(&path).unwrap()).unwrap();
    clips[0].made_by = MadeBy::Recorded {
        actor: "A. Person".to_owned(),
    };
    clips[0].spoken = "Older words.".to_owned();
    let recorded = trpg_content::voice::clip_file(&clips[0].line, clips[0].variant);
    let lost = clips.remove(1);
    let lost_file = trpg_content::voice::clip_file(&lost.line, lost.variant);
    fs::write(&path, files::manifest_text(&clips)).unwrap();
    fs::write(folder.dir.join(&recorded), b"an actor's take").unwrap();
    fs::remove_file(folder.dir.join(&lost_file)).unwrap();

    let status = status(&folder, &scripts, &Tools::fake(&[])).unwrap();
    assert!(
        status.contains(&format!("re-record  {recorded}")),
        "{status}"
    );
    assert!(
        status.ends_with("1 clips to make, 1 to re-record, 0 to remove, 0 lines with no voice"),
        "{status}"
    );

    let key = lost_file
        .trim_end_matches(".ogg")
        .rsplit('/')
        .next()
        .unwrap();
    let report = generate(
        &folder,
        &scripts,
        &mut Tools::fake(&[key]),
        false,
        "2026-11-03",
    )
    .unwrap();
    assert!(report.starts_with("voice: made 0 clips"), "{report}");
    assert!(
        report.contains(&format!("not saved:\n  {lost_file}")),
        "{report}"
    );
    assert!(!folder.dir.join(&lost_file).exists());
    assert_eq!(
        fs::read(folder.dir.join(&recorded)).unwrap(),
        b"an actor's take"
    );
    let _ = fs::remove_dir_all(&folder.dir);
}

#[test]
fn a_clip_of_a_line_that_is_gone_is_removed() {
    let content = content();
    let scripts = test_scene(&content);
    let folder = folder("gone");
    generate(
        &folder,
        &scripts,
        &mut Tools::fake(&[]),
        false,
        "2026-11-03",
    )
    .unwrap();
    let path = folder.dir.join(MANIFEST_FILE);
    let mut clips = files::manifest(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut gone = clips[0].clone();
    gone.line = dlg::LineId::new("test_00000000".to_owned());
    clips.push(gone);
    fs::write(&path, files::manifest_text(&clips)).unwrap();
    fs::write(folder.dir.join("test/test_00000000.ogg"), b"old").unwrap();
    let report = generate(
        &folder,
        &scripts,
        &mut Tools::fake(&[]),
        false,
        "2026-11-03",
    )
    .unwrap();
    assert!(
        report.starts_with("voice: made 0 clips, removed 1"),
        "{report}"
    );
    assert!(!folder.dir.join("test/test_00000000.ogg").exists());
    assert_eq!(
        files::manifest(&fs::read_to_string(&path).unwrap())
            .unwrap()
            .len(),
        16
    );
    let _ = fs::remove_dir_all(&folder.dir);
}

#[test]
fn the_recording_script_of_a_test_character() {
    let content = content();
    let folder = folder("script");
    let script = script(
        &folder,
        &test_scene(&content),
        &Tools::fake(&[]),
        "test_archer",
    )
    .unwrap();
    insta::assert_snapshot!(script);
    let _ = fs::remove_dir_all(&folder.dir);
}

#[test]
fn without_a_cast_the_tool_says_so() {
    let content = content();
    let folder = folder("nocast");
    fs::remove_file(folder.dir.join(files::CAST_FILE)).unwrap();
    let error = status(&folder, &test_scene(&content), &Tools::fake(&[])).unwrap_err();
    assert!(error.starts_with("no cast.ron in "), "{error}");
    let _ = fs::remove_dir_all(&folder.dir);
}

#[test]
fn the_story_scenes_are_the_scenes_of_assets_dialogue() {
    // The game's own scenes (ADR-0118: a test of the content).
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let scenes = story_scenes(&root()).unwrap();
    assert!(scenes.contains("ch01_intro"));
    assert!(scenes.contains("vix"), "the guide's lessons and tips");
    assert!(!scenes.contains("test"));
}

/// A script of the content (ADR-0117) replaces `assets/`'s of its name, and
/// one only the content has is read too.
#[test]
fn the_story_scenes_are_read_with_the_content_over_assets() {
    let repo = crate::content_layer::tests::repo(
        "voice",
        &[
            (
                "assets/dialogue/ch01.dlg",
                "@scene public_one
> Hi.
@end
",
            ),
            (
                "assets/dialogue/ch02.dlg",
                "@scene public_two
> Hi.
@end
",
            ),
            (
                "assets-private/content/dialogue/ch01.dlg",
                "@scene private_one
> Hi.
@end
",
            ),
            (
                "assets-private/content/dialogue/ch03.dlg",
                "@scene private_three
> Hi.
@end
",
            ),
        ],
    );
    let scenes = story_scenes(&repo).unwrap();
    fs::remove_dir_all(&repo).unwrap();
    let expected = ["private_one", "private_three", "public_two", "vix"];
    assert_eq!(scenes, expected.map(str::to_owned).into());
}

/// Acceptance (0476): `status` reads the lessons' and tips' text as the
/// guide's lines, scene `vix`, in her cast voice; made, none is left.
#[test]
fn status_and_generate_cover_the_guides_lessons_and_tips() {
    let content = content();
    let scripts = Scripts::new(&content, BTreeSet::from(["vix".to_owned()]));
    let folder = folder("vix");
    let cast = "(voices: {\"guide\": (voice: \"guide\", settings: Chatterbox)})";
    fs::write(folder.dir.join(files::CAST_FILE), cast).unwrap();
    let lines = trpg_content::voice::guide_scene(&shipped_lessons(), &content.tips)
        .lines()
        .len();
    // Never the test lesson's: a build that ships has no such line.
    let test = content.lessons.get("test_cursor").map(|l| l.text.as_str());
    let test = trpg_content::voice::guide_line(test.unwrap_or_default());
    assert_eq!(
        lines + 1,
        content.lessons.lessons.len() + content.tips.tips.len()
    );
    let tools = Tools::fake(&[]);
    let report = status(&folder, &scripts, &tools).unwrap();
    assert!(
        report.starts_with(
            "vix
  make       vix/vix_"
        ),
        "{report}"
    );
    assert!(!report.contains(test.as_str()), "{report}");
    let counts =
        format!("{lines} clips to make, 0 to re-record, 0 to remove, 0 lines with no voice");
    assert!(report.ends_with(&counts), "{report}");
    let mut tools = Tools::fake(&[]);
    generate(&folder, &scripts, &mut tools, false, "2026-11-08").unwrap();
    let report = status(&folder, &scripts, &tools).unwrap();
    assert!(report.starts_with("0 clips to make"), "{report}");
    let _ = fs::remove_dir_all(&folder.dir);
}

// ---- The audit (ticket 0762) ----

/// A piece of `silencedetect`'s log: quiet at the start, one pause ffmpeg
/// split in two, a short one, and quiet to the end it never closed.
const SILENCES: &str = "\
[silencedetect @ 0x1] silence_start: 0
[silencedetect @ 0x1] silence_end: 0.21 | silence_duration: 0.21
size=N/A time=00:00:01.00
[silencedetect @ 0x1] silence_start: 1.226644
[silencedetect @ 0x1] silence_end: 1.73127 | silence_duration: 0.504626
[silencedetect @ 0x1] silence_start: 1.731338
[silencedetect @ 0x1] silence_end: 1.968639 | silence_duration: 0.237302
[silencedetect @ 0x1] silence_start: 2.681315
[silencedetect @ 0x1] silence_end: 2.972177 | silence_duration: 0.290862
[silencedetect @ 0x1] silence_start: 4.8
";

#[test]
fn pauses_inside_a_clip_are_read_from_silencedetect_joined_and_without_its_ends() {
    let pauses = audit::inner_pauses(SILENCES, 5.0);
    let expected = [1.968_639 - 1.226_644, 2.972_177 - 2.681_315];
    assert_eq!(pauses.len(), expected.len(), "{pauses:?}");
    for (got, want) in pauses.iter().zip(expected) {
        assert!((got - want).abs() < 1e-9, "{pauses:?}");
    }
    // Closed at the very end, the last stretch is still the end's.
    let closed =
        format!("{SILENCES}[silencedetect @ 0x1] silence_end: 5 | silence_duration: 0.2\n");
    assert_eq!(audit::inner_pauses(&closed, 5.0).len(), 2);
    assert_eq!(audit::inner_pauses("no silence here", 3.0), [0.0_f64; 0]);
    // A stretch starting 0.02 s in, or ending 0.02 s before the end, is
    // the end's; one a hair further in is a pause.
    assert!((0.52_f64 - 0.02 - 0.5).abs() < f64::EPSILON);
    let at = |start: f64, end: f64| {
        let log = format!(
            "silence_start: {start}
silence_end: {end} | x
"
        );
        audit::inner_pauses(&log, 0.52).len()
    };
    assert_eq!((at(0.02, 0.3), at(0.021, 0.3)), (0, 1));
    assert_eq!((at(0.2, 0.5), at(0.2, 0.499)), (0, 1));
    assert!(audit::silence_filter().starts_with("silencedetect=noise=-40dB:d=0.15"));
}

#[test]
fn a_clip_is_flagged_for_a_long_pause_or_a_slow_pace_of_a_long_enough_line() {
    let measure = |length_s, pauses: &[f64]| audit::Measure {
        length_s,
        pauses: pauses.to_vec(),
    };
    // 40 letters in 3.2 s: 0.08 s a letter.
    let words = "Hold the gate and watch the river road until dawn.";
    assert_eq!(audit::letters(words), 40);
    let fine = audit::judge("a/a_1.ogg", words, &measure(3.2, &[0.4, 0.55]));
    assert!(!fine.flagged(), "{fine:?}");
    assert!((fine.longest_pause - 0.55).abs() < 1e-9);
    assert!((fine.pace - 0.08).abs() < 1e-9);

    let halting = audit::judge("a/a_1.ogg", words, &measure(3.2, &[0.2, 0.9]));
    assert!(halting.halting && !halting.slow);
    assert_eq!(halting.flags(), "pause 0.90 s");
    // How far over: the larger of the pause's and the pace's shares.
    assert!((halting.over() - 0.9 / audit::LONGEST_PAUSE_S).abs() < 1e-9);
    assert!((fine.over() - 0.55 / audit::LONGEST_PAUSE_S).abs() < 1e-9);
    let slow = audit::judge("a/a_1.ogg", words, &measure(8.0, &[]));
    assert!(slow.slow && !slow.halting);
    assert_eq!(slow.flags(), "pace 0.200 s/letter");
    assert!((slow.over() - 0.2 / audit::SLOWEST_PACE_S).abs() < 1e-9);
    // At the limits exactly, nothing is flagged; a line of 20 letters is
    // judged, of 19 not.
    let twenty = "Hold the gates, watchman.";
    assert_eq!(audit::letters(twenty), 20);
    assert!((3.4_f64 / 20.0 - audit::SLOWEST_PACE_S).abs() < f64::EPSILON);
    assert!(!audit::judge("a/a_1.ogg", twenty, &measure(3.4, &[0.8])).flagged());
    assert!(audit::judge("a/a_1.ogg", twenty, &measure(3.5, &[])).slow);
    assert!(!audit::judge("a/a_1.ogg", "Hold the gate, watchman.", &measure(3.5, &[])).slow);
    // A word or two is mostly breath: its pace isn't judged.
    let short = audit::judge("a/a_1.ogg", "Not alone.", &measure(1.5, &[]));
    assert!(!short.flagged(), "{short:?}");
    assert!(
        short.over().abs() < 1e-9,
        "a short line's pace counts for nothing"
    );
    let both = audit::judge("a/a_1.ogg", words, &measure(9.0, &[1.2]));
    assert_eq!(both.flags(), "pause 1.20 s, pace 0.225 s/letter");
}

#[test]
fn the_audit_report_marks_the_flagged_clips_and_counts_them() {
    let measure = |length_s, pauses: Vec<f64>| audit::Measure { length_s, pauses };
    let words = "Hold the gate and watch the river road until dawn.";
    let rows = [
        audit::judge("a/a_1.ogg", words, &measure(3.2, vec![0.3])),
        audit::judge("a/a_2.ogg", words, &measure(3.6, vec![0.9])),
        audit::judge("a/a_3.ogg", words, &measure(9.0, Vec::new())),
        audit::judge("a/a_4.ogg", "Go.", &measure(2.0, Vec::new())),
    ];
    let report = audit::report(&rows);
    let lines: Vec<&str> = report.lines().collect();
    assert!(lines[1].starts_with("  a/a_1.ogg"), "{report}");
    assert!(lines[2].starts_with("! a/a_2.ogg"), "{report}");
    assert!(lines[2].ends_with("  pause 0.90 s"), "{report}");
    assert!(lines[3].starts_with("! a/a_3.ogg"), "{report}");
    assert!(lines[4].starts_with("  a/a_4.ogg"), "{report}");
    assert!(
        lines[5].starts_with(
            "4 clips; pace of lines of 20+ letters: median 0.090, 90th percentile 0.225"
        ),
        "{report}"
    );
    assert_eq!(
        lines[6],
        "2 flagged: 1 with a pause over 0.8 s, 1 slower than 0.17 s/letter"
    );
}

/// Acceptance (0762): `audit` lists every clip of the scenes asked for,
/// and `generate` warns of each clip it made over a limit.
#[test]
fn generate_warns_of_a_halting_clip_and_audit_lists_every_clip() {
    let content = content();
    let scripts = test_scene(&content);
    let folder = folder("audit");
    let plan = plan_of(
        &scripts,
        &load(&folder, &scripts).unwrap(),
        &Tools::fake(&[]),
    );
    // The first line halts at every seed; the second only at the first,
    // so its next take is kept.
    let halting = voice_plan::key(&plan.wanted[0]);
    let first_take = format!("{}@{}", voice_plan::key(&plan.wanted[1]), SEEDS[0]);
    let mut tools = Tools::fake(&[]).halting(&[&halting, &first_take]);
    let report = generate(&folder, &scripts, &mut tools, false, "2026-11-09").unwrap();
    assert!(report.starts_with("voice: made 16 clips"), "{report}");
    let warning = format!("\nwarning: {}: pause 1.00 s", plan.wanted[0].file());
    assert!(report.ends_with(&warning), "{report}");
    assert_eq!(report.matches("warning:").count(), 1, "{report}");

    let audit = audit(&folder, &scripts, &tools).unwrap();
    assert_eq!(audit.lines().filter(|l| l.contains(".ogg")).count(), 16);
    assert!(
        audit.contains(&format!("! {}", plan.wanted[0].file())),
        "{audit}"
    );
    assert!(audit.ends_with("1 flagged: 1 with a pause over 0.8 s, 0 slower than 0.17 s/letter"));
    // Another scene's clips aren't this audit's.
    let other = Scripts::new(&content, BTreeSet::from(["vix".to_owned()]));
    assert!(audit_of_none(&folder, &other, &tools));
    let _ = fs::remove_dir_all(&folder.dir);
}

fn audit_of_none(folder: &Folder, scripts: &Scripts<'_>, tools: &Tools) -> bool {
    audit(folder, scripts, tools).unwrap().starts_with("  clip")
        && audit(folder, scripts, tools)
            .unwrap()
            .contains("\n0 clips;")
}

// ---- Every lesson and tip the guide says has a clip (ticket 0494) ----

/// The `text` of each shipped lesson and tip of `content` that has no
/// playable clip in the manifest `source`: a box she would show in
/// silence.
fn unvoiced_guide_texts(source: &str, content: &trpg_content::Content) -> Vec<String> {
    let scripts = Scripts::new(content, BTreeSet::new());
    let manifest = voice::from_source(MANIFEST_FILE, source, &scripts.dialogue)
        .unwrap_or_else(|e| panic!("{MANIFEST_FILE}: {:?}", e.iter().map(|e| &e.message)));
    let playable = voice::playable(&manifest, &scripts.dialogue, &content.names);
    let lessons = shipped_lessons();
    let lessons = lessons.lessons.iter().map(|l| &l.text);
    let texts = lessons.chain(content.tips.tips.iter().map(|t| &t.text));
    texts
        .filter(|t| {
            let line = voice::guide_line(t);
            playable.variant_for(&line, Variant::None).is_none()
        })
        .cloned()
        .collect()
}

/// A manifest with a clip of every line of the guide's scene of
/// `content`'s shipped lessons and tips.
fn guide_manifest(content: &trpg_content::Content) -> String {
    let scene = voice::guide_scene(&shipped_lessons(), &content.tips);
    let clip = |l: &dlg::Line| {
        format!(
            "(line: \"{}\", spoken: \"{}\", voice: \"guide\", \
             made_by: Generated(tool: \"t\", model: \"m\", date: \"2026-11-12\")),\n",
            l.id, l.text
        )
    };
    let clips: String = scene.lines().iter().map(clip).collect();
    format!("(clips: [\n{clips}])")
}

/// A tip added with no clip is found: the check below would fail on it.
#[test]
fn a_tip_with_no_clip_is_found() {
    let mut content = content();
    let manifest = guide_manifest(&content);
    assert_eq!(unvoiced_guide_texts(&manifest, &content), [] as [String; 0]);
    let mut tip = content.tips.tips[0].clone();
    tip.text = "A tip nobody has voiced yet.".to_owned();
    content.tips.tips.push(tip);
    assert_eq!(
        unvoiced_guide_texts(&manifest, &content),
        ["A tip nobody has voiced yet."]
    );
}

/// Acceptance (0494): every lesson's and tip's `text` has a playable clip
/// in the private repository's manifest, so a new one without a clip fails
/// CI and not a playtest. Skipped without its copy in `voice/` (`cargo
/// shuyi private-assets`), as the disclosure list's check of it is.
#[test]
fn every_lesson_and_tip_the_guide_says_has_a_clip() {
    let path = root().join("voice").join(SOURCE_LANG).join(MANIFEST_FILE);
    let Ok(source) = fs::read_to_string(&path) else {
        return;
    };
    let content = content();
    assert_eq!(
        unvoiced_guide_texts(&source, &content),
        [] as [String; 0],
        "make their clips: cargo xtask voice generate --scene vix (docs/voice.md)"
    );
}
