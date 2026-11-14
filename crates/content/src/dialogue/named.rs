//! Whether a scene a file names is missing (ticket 0764): a chapter's, a
//! battle trigger's, a cinematic's talk shot's, a camp event's, a support
//! conversation's or a world-map place's. Every load holds those files to
//! the scenes being there, but the load of only some scripts
//! ([`crate::load_with_some_scripts`], the writer's preview on a trial
//! packet of two scenes), which notes each scene named that isn't there
//! instead.

#[cfg(feature = "std")]
use crate::prelude::*;
#[cfg(feature = "std")]
use alloc::collections::BTreeSet;

use super::DialogueTable;

#[cfg(feature = "std")]
std::thread_local! {
    /// While a load of only some scripts runs on this thread: the scenes
    /// named that weren't there. `None` in every other load.
    static NOT_HERE: core::cell::RefCell<Option<BTreeSet<String>>> =
        const { core::cell::RefCell::new(None) };
}

/// Whether `scene`, named by a file, is a problem: it isn't in `dialogue`,
/// and the load isn't of only some scripts, which notes it instead.
pub(crate) fn missing(dialogue: &DialogueTable, scene: &str) -> bool {
    if dialogue.get(scene).is_some() {
        return false;
    }
    #[cfg(feature = "std")]
    if NOT_HERE
        .with_borrow_mut(|n| n.as_mut().map(|n| n.insert(scene.to_owned())))
        .is_some()
    {
        return false;
    }
    true
}

/// Puts back the load before it: the one a [`with_some_scenes`] ran in,
/// if any, even when the load panics.
#[cfg(feature = "std")]
struct Restore(Option<BTreeSet<String>>);

#[cfg(feature = "std")]
impl Drop for Restore {
    fn drop(&mut self) {
        let before = self.0.take();
        NOT_HERE.with_borrow_mut(|n| *n = before);
    }
}

/// `load` run as a load of only some scripts: a scene a file names that
/// isn't there is no problem. Gives its result and the scenes named that
/// weren't there.
#[cfg(feature = "std")]
pub(crate) fn with_some_scenes<T>(load: impl FnOnce() -> T) -> (T, BTreeSet<String>) {
    let before = NOT_HERE.with_borrow_mut(|n| n.replace(BTreeSet::new()));
    let restore = Restore(before);
    let value = load();
    let not_here = NOT_HERE.with_borrow_mut(Option::take).unwrap_or_default();
    drop(restore);
    (value, not_here)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    fn table() -> DialogueTable {
        let scene = super::super::Scene {
            id: "here".to_owned(),
            ..Default::default()
        };
        DialogueTable::new([("here".to_owned(), scene)].into_iter().collect())
    }

    #[test]
    fn a_scene_not_there_is_missing_but_in_a_load_of_some_scripts() {
        let table = table();
        assert!(!missing(&table, "here"));
        assert!(missing(&table, "gone"));
        let (seen, not_here) = with_some_scenes(|| {
            [
                missing(&table, "here"),
                missing(&table, "gone"),
                missing(&table, "gone"),
            ]
        });
        assert_eq!(seen, [false, false, false]);
        assert_eq!(not_here, BTreeSet::from(["gone".to_owned()]));
        assert!(missing(&table, "gone"), "the load after is strict again");
    }

    #[test]
    fn a_load_inside_another_keeps_its_own_scenes_and_puts_the_first_back() {
        let table = table();
        let (inner, outer) = with_some_scenes(|| {
            let _ = missing(&table, "a");
            let (_, inner) = with_some_scenes(|| missing(&table, "b"));
            assert!(!missing(&table, "c"));
            inner
        });
        assert_eq!(inner, BTreeSet::from(["b".to_owned()]));
        assert_eq!(outer, BTreeSet::from(["a".to_owned(), "c".to_owned()]));
    }

    #[test]
    fn a_load_that_panics_leaves_the_next_strict() {
        let table = table();
        let caught = std::panic::catch_unwind(|| with_some_scenes(|| panic!("broken")));
        assert!(caught.is_err());
        assert!(missing(&table, "gone"));
    }
}
