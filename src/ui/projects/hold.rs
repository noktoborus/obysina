//! The long press of a finger on a card, which chooses it.
//!
//! ```text
//! a finger down on a card ──SECONDS, within SLOP of where it fell──► fired
//!                         ──further than SLOP──► nothing: the list scrolls
//! fired ──the finger up──► the tap of that release is no tap
//! ```
//!
//! Why not the long touch of egui: it needs the finger within
//! `max_click_dist`, 6 points, for 0.8 seconds; a finger on a large card
//! drifts more, and the list starts to scroll. Here the finger may drift
//! `SLOP`, and the list scrolls under it meanwhile. The mouse keeps the
//! second button.

use egui::Pos2;
use obusina_pattern::PatternId;

/// The seconds a finger stays down before a card is chosen; the long press
/// of android.
pub const SECONDS: f64 = 0.5;
/// The points a finger may drift from where it fell.
pub const SLOP: f32 = 24.0;

/// A finger down on a card.
#[derive(Debug, Clone, Copy)]
struct Press {
    id: PatternId,
    start: f64,
    origin: Pos2,
    fired: bool,
}

/// What a frame of a hold gives.
#[derive(Debug, Clone, Copy, Default)]
pub struct Step {
    /// The card held long enough, on the one frame it is.
    pub fired: Option<PatternId>,
    /// The card whose tap is no tap: its hold fired, the finger is up.
    pub swallowed: Option<PatternId>,
}

/// The long press of a finger on a card.
#[derive(Debug, Default)]
pub struct Hold {
    press: Option<Press>,
}

impl Hold {
    /// Follows the finger, once a frame before the cards are drawn.
    pub fn step(&mut self, context: &egui::Context) -> Step {
        let Some(mut press) = self.press else {
            return Step::default();
        };
        let (down, at, time) = context.input(|input| {
            let pointer = &input.pointer;
            (pointer.primary_down(), pointer.latest_pos(), input.time)
        });
        let mut step = Step::default();
        if !down {
            step.swallowed = press.fired.then_some(press.id);
            self.press = None;
            return step;
        }
        if at.is_some_and(|at| at.distance(press.origin) > SLOP) {
            self.press = None;
            return step;
        }
        if !press.fired {
            let left = press.start + SECONDS - time;
            if left <= 0.0 {
                press.fired = true;
                step.fired = Some(press.id);
            } else {
                context.request_repaint_after_secs(left as f32);
            }
        }
        self.press = Some(press);
        step
    }

    /// Starts a hold when a finger falls on the card `id`.
    pub fn press_on(&mut self, context: &egui::Context, response: &egui::Response, id: PatternId) {
        if self.press.is_some() || !response.is_pointer_button_down_on() {
            return;
        }
        let fell = context.input(|input| {
            let pointer = &input.pointer;
            match (input.any_touches(), pointer.press_origin()) {
                (true, Some(origin)) => Some((origin, pointer.press_start_time())),
                _ => None,
            }
        });
        if let Some((origin, Some(start))) = fell {
            self.press = Some(Press {
                id,
                start,
                origin,
                fired: false,
            });
        }
    }
}
