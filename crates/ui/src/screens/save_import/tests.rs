use super::*;
use crate::screen::tests::ctx;
use crate::screens::actions::assert_declares;
use shuyi_ui::StoragePlace;

/// The screen declares the keys it reads, and opens only where the
/// platform has files a player reaches.
#[test]
fn the_screen_declares_its_keys_and_opens_only_with_files() {
    let mut c = ctx();
    assert!(ImportScreen::open(&mut c, Channel::Profile).is_none());
    c.platform.storage_place = StoragePlace::Folder("C:/saves".to_owned());
    let screen = ImportScreen::open(&mut c, Channel::Profile);
    let screen = screen.unwrap_or_else(|| panic!("no screen"));
    assert_declares(&screen, ACTIONS);
    assert!(screen.is_waiting());
}
