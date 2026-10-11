//! The command line (native only; the web build has none). Without
//! arguments the game starts at the title. `--scene <id>` opens it
//! straight on one dialogue scene, for whoever writes scripts (ticket
//! 0723, `docs/story/writers-guide.md`); `--lead f` gives that scene the
//! female lead.

use trpg_core::LeadGender;

/// What a wrong command line is answered with.
pub const USAGE: &str = "usage: visions-of-shuyi [--scene <scene id> [--lead m|f]]";

/// What a wrong `--lead` is answered with.
const LEAD_VALUES: &str = "--lead needs male or female (or the first letter)";

/// Where the game starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Startup {
    /// At the title screen.
    Title,
    /// On a dialogue scene, with the default lead of this gender.
    Scene {
        /// The scene's id.
        id: String,
        /// The lead's gender.
        gender: LeadGender,
    },
}

/// The start-up `args` (without the program's name) ask for, or what is
/// wrong with them.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Startup, String> {
    let mut args = args.into_iter();
    let (mut scene, mut gender) = (None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scene" => {
                scene = Some(args.next().ok_or("--scene needs a scene id")?);
            }
            "--lead" => {
                gender = Some(match args.next().as_deref() {
                    Some("m" | "male") => LeadGender::Male,
                    Some("f" | "female") => LeadGender::Female,
                    _ => return Err(LEAD_VALUES.to_owned()),
                });
            }
            other => return Err(format!("unknown argument \"{other}\"")),
        }
    }
    match (scene, gender) {
        (Some(id), gender) => Ok(Startup::Scene {
            id,
            gender: gender.unwrap_or(LeadGender::Male),
        }),
        (None, Some(_)) => Err("--lead needs --scene".to_owned()),
        (None, None) => Ok(Startup::Title),
    }
}

/// The program's own command line. Always the title on the web.
pub fn startup() -> Result<Startup, String> {
    if cfg!(target_arch = "wasm32") {
        return Ok(Startup::Title);
    }
    parse(std::env::args().skip(1)).map_err(|e| format!("{e}\n{USAGE}"))
}

/// `text` with every line broken at spaces so that none is longer than
/// `width` characters (a single longer word stays whole).
pub fn wrap(text: &str, width: usize) -> String {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut current = String::new();
        for word in line.split(' ') {
            let fits = current.chars().count() + 1 + word.chars().count() <= width;
            if !current.is_empty() && !fits {
                out.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        out.push(current);
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(args: &[&str]) -> Result<Startup, String> {
        parse(args.iter().map(|&a| a.to_owned()))
    }

    fn scene(id: &str, gender: LeadGender) -> Startup {
        Startup::Scene {
            id: id.to_owned(),
            gender,
        }
    }

    #[test]
    fn no_arguments_start_at_the_title() {
        assert_eq!(parsed(&[]), Ok(Startup::Title));
    }

    #[test]
    fn scene_opens_a_scene_with_the_male_lead_unless_asked() {
        let (m, f) = (LeadGender::Male, LeadGender::Female);
        let intro = parsed(&["--scene", "ch01_intro"]);
        assert_eq!(intro, Ok(scene("ch01_intro", m)));
        assert_eq!(parsed(&["--scene", "a", "--lead", "f"]), Ok(scene("a", f)));
        assert_eq!(parsed(&["--lead", "f", "--scene", "a"]), Ok(scene("a", f)));
        assert_eq!(parsed(&["--scene", "a", "--lead", "m"]), Ok(scene("a", m)));
        let long = |gender| parsed(&["--scene", "a", "--lead", gender]);
        assert_eq!(long("female"), Ok(scene("a", f)));
        assert_eq!(long("male"), Ok(scene("a", m)));
    }

    #[test]
    fn wrong_arguments_say_what_is_wrong() {
        let err = |args: &[&str]| parsed(args).unwrap_err();
        assert_eq!(err(&["--scene"]), "--scene needs a scene id");
        assert_eq!(err(&["--scene", "a", "--lead"]), LEAD_VALUES);
        assert_eq!(err(&["--scene", "a", "--lead", "x"]), LEAD_VALUES);
        assert!(LEAD_VALUES.starts_with("--lead needs male or female"));
        assert_eq!(err(&["--lead", "f"]), "--lead needs --scene");
        assert_eq!(err(&["ch01_intro"]), "unknown argument \"ch01_intro\"");
    }

    #[test]
    fn long_lines_are_broken_at_spaces() {
        assert_eq!(wrap("aa bb cc", 5), "aa bb\ncc");
        assert_eq!(wrap("aa bb cc", 4), "aa\nbb\ncc");
        assert_eq!(wrap("aa bb cc", 8), "aa bb cc");
        assert_eq!(wrap("aa bb cc", 7), "aa bb\ncc");
        assert_eq!(wrap("abcdefgh ij", 3), "abcdefgh\nij");
        assert_eq!(wrap("one\n\ntwo three", 5), "one\n\ntwo\nthree");
        assert_eq!(wrap("", 5), "");
    }
}
