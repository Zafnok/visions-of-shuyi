//! How the two still pictures of the combat stage move (ticket 0413,
//! `docs/design/look-and-feel.md`, *Combat screen*): the striker runs up
//! to its target and back, a target that is hit flashes and shakes, one
//! that is missed steps back out of the way, a critical winds up first
//! and flashes the whole stage, and a fighter that falls fades and sinks.
//!
//! Every function here is pure and has no unit of length: the skin turns
//! a [`Pose`] into pixels. Times are in seconds. *Tunable.*

/// How far a critical's striker leans back before it runs, in
/// [`Pose::lunge`]'s negative unit.
pub const WIND_UP: f32 = 1.0;
/// The part of the run-up a critical spends winding up.
pub const WIND_UP_PART: f32 = 0.4;
/// How long the striker stays at its target after the strike lands.
pub const HOLD_S: f32 = 0.08;
/// How long the striker takes to go back.
pub const RETURN_S: f32 = 0.3;
/// How long a target that is hit shakes.
pub const SHAKE_S: f32 = 0.3;
/// How long the target stays on each side while it shakes.
pub const SHAKE_STEP_S: f32 = 0.04;
/// How much harder a critical shakes its target.
pub const CRITICAL_SHAKE: f32 = 2.0;
/// How long a target that is hit shows as a white shape.
pub const FLASH_S: f32 = 0.1;
/// The same for a critical.
pub const CRITICAL_FLASH_S: f32 = 0.2;
/// How long the whole stage flashes at a critical.
pub const VEIL_S: f32 = 0.07;
/// How long a target that is missed takes to step back and return.
pub const DODGE_S: f32 = 0.3;

/// Where a fighter's picture is and how it is drawn, relative to standing
/// still.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose {
    /// How far it is toward its opponent: 0 at home, 1 at the opponent;
    /// below 0 it leans back (−[`WIND_UP`] at most).
    pub lunge: f32,
    /// How far it has stepped back out of the way: 0 to 1.
    pub dodge: f32,
    /// How far it is shaken to a side: −1 to 1 for a hit,
    /// ±[`CRITICAL_SHAKE`] for a critical.
    pub shake: f32,
    /// Whether it shows as a white shape (it was just hit).
    pub flash: bool,
    /// How far it has fallen: 0 standing, 1 gone.
    pub fade: f32,
}

/// What a strike did to its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blow {
    /// It missed.
    Miss,
    /// It hit.
    Hit,
    /// It hit critically.
    Critical,
    /// It healed its target (Absorb).
    Heal,
}

/// `t` clamped to 0..=1, slowing toward 1 (a bad number counts as 0).
fn ease_out(t: f32) -> f32 {
    let t = if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.0
    };
    1.0 - (1.0 - t) * (1.0 - t)
}

/// `t` clamped to 0..=1, speeding up from 0 (a bad number counts as 0).
fn ease_in(t: f32) -> f32 {
    let t = if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.0
    };
    t * t
}

/// The striker's [`Pose::lunge`] `part` (0 to 1) of the way through its
/// run-up: straight to the target, slowing as it arrives; a critical
/// leans back first, then dashes.
pub fn run_up(part: f32, critical: bool) -> f32 {
    if !critical {
        return ease_out(part);
    }
    // Each part stays at its end outside its own time, so they add up:
    // the lean back is done when the dash starts.
    let back = ease_out(part / WIND_UP_PART);
    let dash = ease_in((part - WIND_UP_PART) / (1.0 - WIND_UP_PART));
    -WIND_UP * back + (1.0 + WIND_UP) * dash
}

/// The striker's [`Pose::lunge`] `since` seconds after its strike landed:
/// at the target for [`HOLD_S`], then back home within [`RETURN_S`].
pub fn go_back(since: f32) -> f32 {
    1.0 - ease_out((since - HOLD_S) / RETURN_S)
}

/// The target's pose `since` seconds after `blow` landed.
pub fn struck(since: f32, blow: Blow) -> Pose {
    let since = if since.is_finite() {
        since.max(0.0)
    } else {
        0.0
    };
    match blow {
        Blow::Heal => Pose::default(),
        Blow::Miss => {
            let part = (since / DODGE_S).min(1.0);
            Pose {
                // Out and back: 0 at both ends, 1 half way.
                dodge: 4.0 * part * (1.0 - part),
                ..Pose::default()
            }
        }
        Blow::Hit | Blow::Critical => {
            let critical = blow == Blow::Critical;
            let force = if critical { CRITICAL_SHAKE } else { 1.0 };
            let left = (1.0 - since / SHAKE_S).max(0.0);
            // Whole steps since the blow: away first, then toward, …
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let step = (since / SHAKE_STEP_S) as u32;
            let side = if step.is_multiple_of(2) { 1.0 } else { -1.0 };
            let flash = if critical { CRITICAL_FLASH_S } else { FLASH_S };
            Pose {
                shake: side * force * left,
                flash: since < flash,
                ..Pose::default()
            }
        }
    }
}

/// Whether the whole stage flashes `since` seconds after `blow` landed.
pub fn veil(since: f32, blow: Blow) -> bool {
    blow == Blow::Critical && (0.0..VEIL_S).contains(&since)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn a_strike_runs_straight_to_its_target_slowing_down() {
        assert!(close(run_up(0.0, false), 0.0));
        assert!(close(run_up(0.25, false), 0.4375));
        assert!(close(run_up(0.5, false), 0.75));
        assert!(close(run_up(0.75, false), 0.9375));
        assert!(close(run_up(1.0, false), 1.0));
        // Outside the run-up, and bad numbers.
        assert!(close(run_up(-1.0, false), 0.0));
        assert!(close(run_up(2.0, false), 1.0));
        assert!(close(run_up(f32::NAN, false), 0.0));
    }

    #[test]
    fn a_critical_leans_back_then_dashes() {
        assert!(close(run_up(0.0, true), 0.0));
        // Half way through the wind-up: three quarters back.
        assert!(close(run_up(0.2, true), -0.75));
        // Fully back as the dash starts.
        assert!(close(run_up(0.4, true), -1.0));
        // A quarter, a half and all of the dash: 2 × t² − 1.
        assert!(close(run_up(0.55, true), -0.875));
        assert!(close(run_up(0.7, true), -0.5));
        assert!(close(run_up(1.0, true), 1.0));
        assert!(close(run_up(3.0, true), 1.0));
        assert!(close(run_up(-1.0, true), 0.0));
    }

    #[test]
    fn the_striker_holds_then_goes_home() {
        assert!(close(go_back(0.0), 1.0));
        assert!(close(go_back(HOLD_S), 1.0));
        // A third, a half and all of the way back in time.
        assert!(close(go_back(HOLD_S + RETURN_S / 3.0), 4.0 / 9.0));
        assert!(close(go_back(HOLD_S + RETURN_S / 2.0), 0.25));
        assert!(close(go_back(HOLD_S + RETURN_S), 0.0));
        assert!(close(go_back(9.0), 0.0));
        assert!(close(go_back(f32::NAN), 1.0));
    }

    /// Frame by frame at 50 frames a second (0.02 s): the shake of a hit.
    #[test]
    fn a_hit_flashes_white_and_shakes_less_and_less() {
        let frames: Vec<Pose> = (0..17_u8)
            .map(|f| struck(f32::from(f) * 0.02, Blow::Hit))
            .collect();
        let flash: Vec<bool> = frames.iter().map(|p| p.flash).collect();
        // White for 0.1 s: frames 0 to 4 (frame 5 is on the edge).
        assert_eq!(flash[..5], [true; 5]);
        assert!(flash[6..].iter().all(|f| !f));
        // Two frames on each side, a fifteenth weaker each frame.
        let expected = [
            15.0, 14.0, -13.0, -12.0, 11.0, 10.0, -9.0, -8.0, 7.0, 6.0, -5.0, -4.0, 3.0, 2.0, -1.0,
            0.0, 0.0,
        ];
        for (f, (pose, e)) in frames.iter().zip(expected).enumerate() {
            // Frames on a step's edge may land on either side.
            assert!(
                close(pose.shake.abs(), f32::abs(e) / 15.0),
                "frame {f}: {pose:?}"
            );
            assert!(pose.lunge == 0.0 && pose.dodge == 0.0 && pose.fade == 0.0);
        }
        // Mid-step, the side is certain: away, toward, away.
        assert!(struck(0.01, Blow::Hit).shake > 0.0);
        assert!(struck(0.05, Blow::Hit).shake < 0.0);
        assert!(struck(0.09, Blow::Hit).shake > 0.0);
        assert!(close(struck(0.01, Blow::Hit).shake, 1.0 - 0.01 / SHAKE_S));
        assert!(close(
            struck(0.05, Blow::Hit).shake,
            -(1.0 - 0.05 / SHAKE_S)
        ));
        assert_eq!(struck(SHAKE_S, Blow::Hit), Pose::default());
        assert_eq!(struck(1.0, Blow::Hit), Pose::default());
        // A bad time is the moment of the blow.
        assert_eq!(struck(f32::NAN, Blow::Hit), struck(0.0, Blow::Hit));
        assert_eq!(struck(-1.0, Blow::Hit), struck(0.0, Blow::Hit));
    }

    #[test]
    fn a_critical_shakes_twice_as_hard_and_flashes_twice_as_long() {
        assert!(close(struck(0.0, Blow::Critical).shake, 2.0));
        assert!(close(
            struck(0.05, Blow::Critical).shake,
            -2.0 * (1.0 - 0.05 / SHAKE_S)
        ));
        assert!(struck(0.15, Blow::Critical).flash);
        assert!(!struck(0.15, Blow::Hit).flash);
        assert!(struck(0.19, Blow::Critical).flash);
        assert!(!struck(0.2, Blow::Critical).flash);
        assert!(!struck(0.1, Blow::Hit).flash);
        assert!(struck(0.09, Blow::Hit).flash);
    }

    #[test]
    fn a_miss_steps_back_and_returns_and_a_heal_stands_still() {
        let dodge = |t: f32| struck(t, Blow::Miss).dodge;
        assert!(close(dodge(0.0), 0.0));
        assert!(close(dodge(DODGE_S / 4.0), 0.75));
        assert!(close(dodge(DODGE_S / 2.0), 1.0));
        assert!(close(dodge(DODGE_S * 0.75), 0.75));
        assert!(close(dodge(DODGE_S), 0.0));
        assert!(close(dodge(2.0), 0.0));
        let miss = struck(DODGE_S / 2.0, Blow::Miss);
        assert!(!miss.flash && miss.shake == 0.0);
        for t in [0.0, 0.05, 0.5] {
            assert_eq!(struck(t, Blow::Heal), Pose::default());
        }
    }

    #[test]
    fn only_a_critical_flashes_the_stage_and_only_at_first() {
        assert!(veil(0.0, Blow::Critical));
        assert!(veil(0.06, Blow::Critical));
        assert!(!veil(VEIL_S, Blow::Critical));
        assert!(!veil(0.5, Blow::Critical));
        assert!(!veil(-0.01, Blow::Critical));
        assert!(!veil(f32::NAN, Blow::Critical));
        for blow in [Blow::Hit, Blow::Miss, Blow::Heal] {
            assert!(!veil(0.0, blow));
        }
    }
}
