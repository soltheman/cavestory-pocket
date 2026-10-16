//! Phone vibration for the game's rumble events: getting hit and quakes. Only iOS drives a motor;
//! elsewhere the taps are just logged at debug level.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(true);

/// Ticks between taps while a quake shakes the phone.
const QUAKE_TAP_INTERVAL: u32 = 4;
/// Long scripted quakes stop buzzing after this many ticks (2 seconds at 50 ticks/s).
const MAX_QUAKE_TICKS: u32 = 100;

thread_local! {
    /// The quake being felt: ticks left and tap strength.
    static QUAKE: Cell<(u32, f32)> = Cell::new((0, 0.0));
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
    if !enabled {
        QUAKE.with(|q| q.set((0, 0.0)));
    }
}

fn tap(intensity: f32) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }

    #[cfg(target_os = "ios")]
    crate::util::ios::haptic_impact(intensity);

    #[cfg(not(target_os = "ios"))]
    log::debug!("Haptic tap at {:.2}", intensity);
}

/// The player got hurt: one firm tap, firmer the less health is left (rumble 0x4000 to 0x6000).
pub fn hit(strength: u16) {
    tap((strength as f32 / 0x6000 as f32).clamp(0.5, 1.0));
}

/// A quake: light taps for its length (rumble 0x3000 normal, 0x5000 super).
pub fn quake(strength: u16, ticks: u32) {
    let intensity = (strength as f32 / 0x7000 as f32).clamp(0.2, 0.8);
    QUAKE.with(|q| {
        let (remaining, current) = q.get();
        q.set((remaining.max(ticks.min(MAX_QUAKE_TICKS)), current.max(intensity)));
    });
    tap(intensity);
}

/// Called every game tick to keep a quake tapping.
pub fn tick() {
    QUAKE.with(|q| {
        let (remaining, intensity) = q.get();
        if remaining == 0 {
            return;
        }

        let remaining = remaining - 1;
        q.set((remaining, if remaining == 0 { 0.0 } else { intensity }));
        if remaining > 0 && remaining % QUAKE_TAP_INTERVAL == 0 {
            tap(intensity);
        }
    });
}
