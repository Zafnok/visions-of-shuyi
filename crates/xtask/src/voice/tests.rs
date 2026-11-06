use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use trpg_content::dialogue::{self as dlg, DialogueTable};
use trpg_content::names::Names;
use trpg_content::voice::{MadeBy, Variant, VoiceClip};

use super::files::{self, Cast, Reader, Say, Sent, SentMap, Voice};
use super::plan::{Inputs, Plan, Why, plan};
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
    let voice = |clip: &str, reader| Voice {
        clip: clip.to_owned(),
        reader,
    };
    Cast {
        voices: BTreeMap::from([
            (">".to_owned(), voice("narrator", Reader::Qwen3)),
            ("test_lord".to_owned(), voice("rival", Reader::Qwen3)),
            (
                "test_knight".to_owned(),
                voice("retainer", Reader::Chatterbox),
            ),
            ("lead_m".to_owned(), voice("lead_m", Reader::Qwen3)),
            ("lead_f".to_owned(), voice("lead_f", Reader::Qwen3)),
        ]),
    }
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
            .map(|v| (v.clip.clone(), "00000000".to_owned()))
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
            let voice = &self.cast.voices[&w.entry];
            let tool = if w.direction.is_some() {
                "directed".to_owned()
            } else {
                format!("{:?}", voice.reader)
            };
            self.clips
                .retain(|c| (&c.line, c.variant) != (&w.line, w.variant));
            self.clips.push(VoiceClip {
                line: w.line.clone(),
                variant: w.variant,
                spoken: w.spoken.clone(),
                direction: w.direction.clone(),
                voice: voice.clip.clone(),
                made_by: MadeBy::Generated {
                    tool,
                    model: "m".to_owned(),
                    date: "2026-11-03".to_owned(),
                },
            });
            let sent = Sent {
                text: w.sent.clone(),
                reference: self.references[&voice.clip].clone(),
            };
            self.sent.insert(w.file(), sent);
        }
        self.clips.retain(|c| !plan.remove.contains(c));
        self
    }
}

/// The plan's clips to make, as `file why` rows.
fn to_make(plan: &Plan) -> Vec<String> {
    plan.make
        .iter()
        .map(|&(i, why)| format!("{} {why:?}", plan.wanted[i].file()))
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
                w.entry,
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
            "a/a_36a692b7.m.ogg lead_m - | Let's move.",
            "a/a_36a692b7.f.ogg lead_f - | Let's move.",
            "a/a_c9e4741d.m.ogg > - | The exile grips his sword.",
            "a/a_c9e4741d.f.ogg > - | The exile grips her sword.",
            "a/a_16d9be18.ogg test_knight - | Good.",
        ]
    );
    assert!(plan.make.iter().all(|&(_, why)| why == Why::Missing));
    assert_eq!(plan.make.len(), 8);
    assert_eq!(
        plan.wanted[3].before,
        Some(("test_knight".to_owned(), "You're late.".to_owned()))
    );
}

#[test]
fn once_made_nothing_is_left_to_do() {
    let plan = World::new().made().plan();
    assert_eq!(
        plan,
        Plan {
            wanted: plan.wanted.clone(),
            ..Plan::default()
        }
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
    world.cast.voices.get_mut("test_lord").unwrap().clip = "messenger".to_owned();
    world
        .references
        .insert("messenger".to_owned(), "1".to_owned());
    let lord = ["a/a_2748b275.ogg Voice"];
    assert_eq!(to_make(&world.plan()), lord);

    // Another reader, or a new recording of the same clip, is a new voice.
    let mut world = World::new().made();
    world.cast.voices.get_mut("test_lord").unwrap().reader = Reader::Chatterbox;
    assert_eq!(to_make(&world.plan()), lord);
    let mut world = World::new().made();
    world
        .references
        .insert("rival".to_owned(), "ffffffff".to_owned());
    assert_eq!(to_make(&world.plan()), lord);
}

#[test]
fn a_new_spelling_in_say_ron_remakes_the_lines_with_that_word_and_keeps_the_words() {
    let mut world = World::new().made();
    world.say.insert("town".to_owned(), "Hayth".to_owned());
    let plan = world.plan();
    assert_eq!(to_make(&plan), ["a/a_5912b92e.ogg Spelling"]);
    let w = &plan.wanted[plan.make[0].0];
    assert_eq!(
        (w.spoken.as_str(), w.sent.as_str()),
        ("The rain fell on Heth.", "The rain fell on Hayth.")
    );
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
    world.cast.voices.get_mut("test_lord").unwrap().clip = "messenger".to_owned();
    world.dialogue = dialogue(&SCRIPT.replace("(angrily)", "(wearily)"));
    let plan = world.plan();
    assert_eq!(plan.make, []);
    let record: Vec<String> = plan
        .record
        .iter()
        .map(|&(i, why)| format!("{} {why:?}", plan.wanted[i].file()))
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
    let uncast: Vec<&str> = plan
        .uncast
        .iter()
        .map(|(_, entry)| entry.as_str())
        .collect();
    assert_eq!(uncast, ["test_knight", "test_knight"]);
    assert_eq!((plan.make.len(), plan.remove.len()), (0, 0));
}

// ---- The files ----

#[test]
fn the_cast_names_characters_narration_and_the_leads_voices() {
    let source = "(voices: {\">\": (clip: \"narrator\", reader: Qwen3), \"lead_m\": (clip: \"lead_m\", \
                  reader: Qwen3), \"retainer\": (clip: \"retainer\", reader: Chatterbox)})";
    let cast = files::cast(source, |id| id == "retainer").unwrap();
    assert_eq!(cast.voices["retainer"].reader, Reader::Chatterbox);
    let bad = "(voices: {\"lead\": (clip: \"x\", reader: Qwen3), \"nobody\": (clip: \"\", reader: Qwen3)})";
    let errors = files::cast(bad, |_| true).unwrap_err();
    assert!(
        errors.contains("\"lead\" is not a character that speaks"),
        "{errors}"
    );
    assert!(errors.contains("\"nobody\" has no clip"), "{errors}");
    assert!(files::cast("(voices: {\"x\": \"y\"})", |_| true).is_err());
    assert_eq!(Cast::entry("lead", Variant::F), "lead_f");
    assert_eq!(Cast::entry("lead", Variant::M), "lead_m");
    assert_eq!(Cast::entry("hollis", Variant::M), "hollis");
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
    assert!(cast.voices.values().all(|v| v.clip == "test_tone"));
}

#[test]
fn respelling_replaces_whole_words_and_names_by_id() {
    let say = files::say("{\"town\": \"Hayth\", \"Aske\": \"Ask\"}").unwrap();
    let name = |id: &str| (id == "town").then(|| "Heth".to_owned());
    assert_eq!(
        files::respell(
            "Aske left Heth; Askew and Hethers stayed. Aske's bow.",
            &say,
            name
        ),
        "Ask left Hayth; Askew and Hethers stayed. Aske's bow."
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
    assert!(synth::says("Tamsen rides at dawn", "Tamsin rides at dawn!"));
    assert!(!synth::says("You are late, sir", "You're late."));
    assert!(!synth::says("", "No."));
    assert!(synth::says("", "..."));
}

#[test]
fn a_short_line_passes_when_its_words_sound_the_same() {
    // 0722: the transcriber spells the lead's "Dace's." its own ways.
    for heard in ["Dases.", "Daces", "Days."] {
        assert!(synth::says(heard, "Dace's."), "{heard}");
    }
    assert!(synth::says("Kales", "Kael's!"));
    assert!(!synth::says("Bases.", "Dace's."));
    assert!(!synth::says("Dace is.", "Dace's."));
    assert!(!synth::says("Hold the boat.", "Hold the gate."));
    // Sounding the same is not enough past a few words.
    let long = "Dace's, Kael's, Tamsin's, Ellis's, Ferro's.";
    let heard = "Dases, Kales, Tamsens, Elises, Feros.";
    assert!(!synth::says(heard, long));
    assert!(synth::says(
        "Dases, Kales, Tamsens, Elises.",
        "Dace's, Kael's, Tamsin's, Ellis's."
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
    \">\": (clip: \"narrator\", reader: Qwen3),\n\
    \"lead_m\": (clip: \"lead_m\", reader: Qwen3),\n\
    \"lead_f\": (clip: \"lead_f\", reader: Qwen3),\n\
    \"test_knight\": (clip: \"retainer\", reader: Chatterbox),\n\
    \"test_lord\": (clip: \"rival\", reader: Qwen3),\n\
    \"test_archer\": (clip: \"poacher\", reader: Chatterbox),\n\
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
    assert!(
        report.contains("and 2 directed (65 characters)"),
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
    let scenes = story_scenes(&root()).unwrap();
    assert!(scenes.contains("ch01_intro"));
    assert!(!scenes.contains("test"));
}
