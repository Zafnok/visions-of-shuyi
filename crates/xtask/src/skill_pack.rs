//! Shuyi Engine's skill pack, copied into `.claude/skills/` at the pinned
//! engine (ADR-0097, ticket 1192).
//!
//! The pack's skills (`shuyi-*`) teach the engine's API, so they must be
//! the pinned engine's, not its `main`'s. xtask builds with `shuyi-cli` at
//! the pin, which embeds its pack ([`Pack::embedded`]): `cargo xtask
//! skill-pack` writes that pack beside the game's own skills, and
//! `--check` (in CI) fails when the copy differs from it. A folder named
//! `shuyi-*` that the pack doesn't have is an old skill of the pack.

use std::fs;
use std::path::Path;

use shuyi_cli::skills::Pack;

/// The game's skills folder, from the repository's root.
pub const DIR: &str = ".claude/skills";
/// How each of the pack's skills is named; the game's own never are.
pub const PREFIX: &str = "shuyi-";

pub const USAGE: &str = "usage: cargo xtask skill-pack [--check]\n\n\
skill-pack          copy Shuyi Engine's skill pack, at the pinned engine, into .claude/skills/\n\
skill-pack --check  fail when the copy there isn't the pinned engine's pack";

/// A file's bytes with Windows line endings made `\n`, when it is text.
fn normalized(bytes: &[u8]) -> Vec<u8> {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.replace("\r\n", "\n").into_bytes(),
        Err(_) => bytes.to_vec(),
    }
}

/// Every file under `dir`, by its `/`-separated path from `root`.
fn files_under(root: &Path, dir: &Path, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_under(root, &path, found);
        } else if let Ok(relative) = path.strip_prefix(root) {
            let parts: Vec<_> = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            found.push(parts.join("/"));
        }
    }
}

/// The `shuyi-*` folders in `skills`.
fn copied_skills(skills: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(skills) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(PREFIX))
        .collect();
    names.sort();
    names
}

/// How the copy of `pack` in `root`'s skills folder differs from it: a
/// file missing or changed, a file the pack lacks, an old skill.
pub fn problems(root: &Path, pack: &Pack) -> Vec<String> {
    let skills = root.join(DIR);
    let mut found = Vec::new();
    for skill in &pack.skills {
        let folder = skills.join(&skill.name);
        for (path, bytes) in &skill.files {
            let shown = format!("{DIR}/{}/{path}", skill.name);
            match fs::read(folder.join(path)) {
                Err(_) => found.push(format!("{shown}: missing")),
                Ok(have) if normalized(&have) != normalized(bytes) => {
                    found.push(format!("{shown}: not the pinned engine's"));
                }
                Ok(_) => {}
            }
        }
        let mut on_disk = Vec::new();
        files_under(&folder, &folder, &mut on_disk);
        for path in on_disk.iter().filter(|p| !skill.files.contains_key(*p)) {
            found.push(format!(
                "{DIR}/{}/{path}: not in the pinned engine's pack",
                skill.name
            ));
        }
    }
    for name in copied_skills(&skills) {
        if pack.skill(&name).is_none() {
            found.push(format!(
                "{DIR}/{name}/: the pinned engine's pack has no such skill"
            ));
        }
    }
    found
}

/// `skill-pack`: writes `pack` into `root`'s skills folder, after taking
/// away every `shuyi-*` folder there.
pub fn write(root: &Path, pack: &Pack) -> Result<String, String> {
    let skills = root.join(DIR);
    for name in copied_skills(&skills) {
        fs::remove_dir_all(skills.join(&name)).map_err(|e| format!("{DIR}/{name}: {e}"))?;
    }
    for skill in &pack.skills {
        if !skill.name.starts_with(PREFIX) {
            return Err(format!(
                "the pack's skill `{}` isn't named `{PREFIX}*`, so it can't be told from the game's own",
                skill.name
            ));
        }
        for (path, bytes) in &skill.files {
            let file = skills.join(&skill.name).join(path);
            let shown = format!("{DIR}/{}/{path}", skill.name);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{shown}: {e}"))?;
            }
            fs::write(&file, normalized(bytes)).map_err(|e| format!("{shown}: {e}"))?;
        }
    }
    Ok(format!(
        "{DIR}/ now has the pinned engine's {} skills: commit them",
        pack.skills.len()
    ))
}

/// `skill-pack --check`.
pub fn check(root: &Path, pack: &Pack) -> Result<String, Vec<String>> {
    let found = problems(root, pack);
    if found.is_empty() {
        Ok(format!(
            "skill-pack: OK, {DIR}/ has the pinned engine's {} skills",
            pack.skills.len()
        ))
    } else {
        Err(found)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use shuyi_cli::skills::Skill;

    use super::*;

    fn pack() -> Pack {
        let skill = |name: &str, files: &[(&str, &str)]| Skill {
            name: name.to_owned(),
            files: files
                .iter()
                .map(|(p, t)| ((*p).to_owned(), t.as_bytes().to_vec()))
                .collect::<BTreeMap<_, _>>(),
        };
        Pack {
            skills: vec![
                skill("shuyi-gates", &[("SKILL.md", "gates\n")]),
                skill(
                    "shuyi-test",
                    &[("SKILL.md", "test\n"), ("example/walk.rs", "walk\n")],
                ),
            ],
            not_skills: Vec::new(),
        }
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("skill-pack-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(DIR).join("run-gates")).unwrap();
        fs::write(dir.join(DIR).join("run-gates/SKILL.md"), "ours\n").unwrap();
        dir
    }

    #[test]
    fn writing_the_pack_leaves_the_game_s_own_skills_and_passes_the_check() {
        let dir = scratch("write");
        assert_eq!(problems(&dir, &pack()).len(), 3);
        let said = write(&dir, &pack()).unwrap();
        assert!(said.contains("2 skills"), "{said}");
        assert!(check(&dir, &pack()).is_ok());
        let walk = fs::read_to_string(dir.join(DIR).join("shuyi-test/example/walk.rs")).unwrap();
        assert_eq!(walk, "walk\n");
        let ours = fs::read_to_string(dir.join(DIR).join("run-gates/SKILL.md")).unwrap();
        assert_eq!(ours, "ours\n");
    }

    #[test]
    fn the_check_names_each_difference() {
        let dir = scratch("check");
        write(&dir, &pack()).unwrap();
        let skills = dir.join(DIR);
        fs::write(skills.join("shuyi-gates/SKILL.md"), "edited\n").unwrap();
        fs::remove_file(skills.join("shuyi-test/example/walk.rs")).unwrap();
        fs::write(skills.join("shuyi-test/notes.md"), "extra\n").unwrap();
        fs::create_dir_all(skills.join("shuyi-gone")).unwrap();
        let found = check(&dir, &pack()).unwrap_err();
        assert_eq!(
            found,
            vec![
                format!("{DIR}/shuyi-gates/SKILL.md: not the pinned engine's"),
                format!("{DIR}/shuyi-test/example/walk.rs: missing"),
                format!("{DIR}/shuyi-test/notes.md: not in the pinned engine's pack"),
                format!("{DIR}/shuyi-gone/: the pinned engine's pack has no such skill"),
            ]
        );
        // Writing again takes the old skill away and mends the rest.
        write(&dir, &pack()).unwrap();
        assert!(!skills.join("shuyi-gone").exists());
        assert!(check(&dir, &pack()).is_ok());
    }

    #[test]
    fn windows_line_endings_are_the_same_file() {
        let dir = scratch("crlf");
        write(&dir, &pack()).unwrap();
        fs::write(dir.join(DIR).join("shuyi-gates/SKILL.md"), "gates\r\n").unwrap();
        assert!(check(&dir, &pack()).is_ok());
    }

    #[test]
    fn a_pack_skill_not_named_shuyi_is_refused() {
        let dir = scratch("name");
        let mut odd = pack();
        odd.skills[0].name = "gates".to_owned();
        assert!(write(&dir, &odd).unwrap_err().contains("`gates`"));
    }

    #[test]
    fn the_real_repo_has_the_pinned_engine_s_pack() {
        let root = crate::repo_root();
        // With the local checkout on, xtask embeds the checkout's pack.
        if root.join(crate::engine::PATCH_FILE).exists() {
            return;
        }
        assert_eq!(problems(&root, &Pack::embedded()), Vec::<String>::new());
    }
}
