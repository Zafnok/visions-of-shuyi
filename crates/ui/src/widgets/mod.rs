//! Reusable pieces screens are built from. The menu and word wrap are
//! Shuyi Engine's (`shuyi_ui::widgets`, ticket 1112), re-exported here
//! under their old names; typing a name is this game's.

pub mod help;
pub mod menu;
pub mod name_entry;

pub use menu::{Menu, MenuEvent, MenuItem, MenuItemView, MenuView};
pub use shuyi_ui::widgets::wrap;
pub use wrap::word_wrap;
