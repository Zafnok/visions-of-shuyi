//! The game's files on disk as the build embeds them (ADR-0117): the
//! content from the private repository, in `assets-private/content/`, laid
//! over `assets/` when it is there. The tools that read the files from disk
//! read this view: `check-script`, `voice` and `check-keys` (and `lint`,
//! which runs it). `lines` and `lang-status` read the embedded bundle,
//! which has the content in it already.
//!
//! The files themselves are listed by the process tool's
//! [`xtask::content_layer`], which `check-leaks` reads too (ADR-0129);
//! this adds the project `check-keys` scans.

use shuyi_cli::prelude::Project;
use trpg_content::bundle::DISPLAY_ROOT;

pub use xtask::content_layer::{CONTENT, STORY, files_in};

/// `project` with the content in what `check-keys` scans when it is there
/// (`check_keys.assets`), its keymap file left out as `assets/`'s is: the
/// project `check-keys` and `lint` run on.
pub fn project(mut project: Project) -> Project {
    if !project.root.join(CONTENT).is_dir() {
        return project;
    }
    if let Some(keys) = project.config.check_keys.as_mut() {
        let prefix = format!("{DISPLAY_ROOT}/");
        let content: Vec<String> = keys
            .not_text
            .iter()
            .filter_map(|path| path.strip_prefix(&prefix))
            .map(|path| format!("{CONTENT}/{path}"))
            .collect();
        keys.not_text.extend(content);
        keys.assets.insert(0, CONTENT.to_owned());
    }
    project
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// A repository with `assets/dialogue/` and, with `content`, a
    /// `assets-private/content/dialogue/`, under the system's temp folder.
    pub(crate) fn repo(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-content-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        dir
    }

    /// The process tool writes the bundle's two folders out, so it builds
    /// without the game (ADR-0129): they stay the bundle's.
    #[test]
    fn the_process_tool_s_folders_are_the_bundle_s() {
        use trpg_content::bundle::CONTENT_DISPLAY_ROOT;
        assert_eq!(CONTENT, CONTENT_DISPLAY_ROOT);
        assert_eq!(xtask::content_layer::ASSETS, DISPLAY_ROOT);
        assert_eq!(STORY, [CONTENT_DISPLAY_ROOT, DISPLAY_ROOT]);
    }

    /// `check-keys` scans the content's scripts and data too, but its
    /// keymap file, when the content is there; the project is as it was
    /// when not.
    #[test]
    fn check_keys_scans_the_content_when_it_is_there() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = Project::load(&repository).unwrap().config;
        let root = repo(
            "keys",
            &[
                ("assets/dialogue/a.dlg", "@scene a\n> Go.\n@end\n"),
                ("assets-test/dialogue/t.dlg", "@scene t\n> Go.\n@end\n"),
                (
                    "assets-private/content/dialogue/a.dlg",
                    "@scene a\n> Press Enter to go.\n@end\n",
                ),
                (
                    "assets-private/content/data/keymap.ron",
                    "(enter: \"Enter\")",
                ),
            ],
        );
        let layered = project(Project::new(&root, config.clone()));
        let keys = layered.config.check_keys.as_ref().unwrap();
        assert_eq!(keys.assets, [CONTENT, "assets", "assets-test"]);
        assert!(
            keys.not_text
                .contains(&format!("{CONTENT}/data/keymap.ron"))
        );
        let rules = shuyi_cli::project::CheckKeys {
            rust: Vec::new(),
            ..keys.clone()
        };
        let errors = shuyi_cli::check_keys::run(&rules, &root);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(
            errors[0].starts_with("assets-private/content/dialogue/a.dlg:2"),
            "{errors:?}"
        );
        fs::remove_dir_all(root.join("assets-private")).unwrap();
        let public = project(Project::new(&root, config.clone()));
        assert_eq!(public.config, config);
        fs::remove_dir_all(&root).unwrap();
    }
}
