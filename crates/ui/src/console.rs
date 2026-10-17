//! The console sizes this game picks (ADR-0070 §6). The console itself
//! and fitting it to a window are Shuyi Engine's
//! (`shuyi_ui::frame::console`, ticket 1108).

pub use shuyi_ui::frame::console::{ConsoleSpec, Layout, layout};

/// This game's console sizes, as `ConsoleSpec::PC` and
/// `ConsoleSpec::PLAYDATE` (import the trait to name them).
pub trait ConsoleSizes {
    /// The PC (and web) console: 100 × 32 cells of 8 × 16 pixels.
    const PC: ConsoleSpec;

    /// The Playdate's 400 × 240 screen: 50 × 15 cells of 8 × 16 pixels
    /// (`docs/design/playdate.md`, *Size*).
    const PLAYDATE: ConsoleSpec;
}

impl ConsoleSizes for ConsoleSpec {
    const PC: ConsoleSpec = ConsoleSpec::text(100, 32);
    const PLAYDATE: ConsoleSpec = ConsoleSpec::text(50, 15);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(scale: u32, offset_x: f32, offset_y: f32) -> Layout {
        Layout {
            scale,
            offset_x,
            offset_y,
        }
    }

    #[test]
    fn the_pc_console_fits_1080p_at_2x() {
        assert_eq!(ConsoleSpec::PC.px(), (800, 512));
        assert_eq!(ConsoleSpec::PC.cell_px(), (8, 16));
        assert_eq!(layout(ConsoleSpec::PC, 800.0, 512.0), l(1, 0.0, 0.0));
        // 1080p: 2x (1600x1024), bars of 160 px left/right and 28 top/bottom.
        assert_eq!(layout(ConsoleSpec::PC, 1920.0, 1080.0), l(2, 160.0, 28.0));
    }

    #[test]
    fn the_playdate_console_fits_its_own_size() {
        let pd = ConsoleSpec::PLAYDATE;
        assert_eq!(pd.px(), (400, 240));
        assert_eq!(pd.cell_px(), (8, 16));
        assert_eq!(layout(pd, 400.0, 240.0), l(1, 0.0, 0.0));
        // The PC's 1080p window takes it at 4x: 1600 × 960, bars of 160 and 60.
        assert_eq!(layout(pd, 1920.0, 1080.0), l(4, 160.0, 60.0));
    }
}
