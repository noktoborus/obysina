//! A press on a panel, and when it takes the panel.
//!
//! ```text
//! press on the panel ─┬─ mouse ──moved past max_click_dist──► taken
//!                     └─ finger ─┬─ still for max_click_duration ──► taken
//!                                └─ moved before that ──► dropped
//! taken ──moved──► follows the pointer, the grab under it
//! let go ──after a move──► Let: where, and the edge strip it is in
//! ```
//!
//! A finger waits as long as egui waits before a touch is no click: a
//! button under a finger that took the panel is not pressed when it lets
//! go. A mouse moved past the click is no click either.

use egui::{Context, LayerId, Pos2, Rect, Vec2};

use crate::edge::Edge;

/// A press on a panel.
#[derive(Debug, Clone, Copy)]
pub struct Hold {
    /// Where the press began.
    origin: Pos2,
    /// When it began, in seconds of the input.
    start: f64,
    /// It is a finger, which has to be held still before it takes the panel.
    touch: bool,
    /// The point of the panel under the pointer, from its top left corner.
    grab: Vec2,
    /// The panel is taken: it follows the pointer.
    taken: bool,
    /// The pointer went past a click since the panel was taken.
    moved: bool,
    /// The top left corner of the panel, while it is taken.
    position: Pos2,
    /// The edge strip the pointer is in.
    edge: Option<Edge>,
}

impl Hold {
    /// The panel follows the pointer.
    pub fn taken(&self) -> bool {
        self.taken
    }

    /// A finger waits to take the panel: the time has to be watched.
    pub fn waits(&self) -> bool {
        self.touch && !self.taken
    }

    /// The top left corner of a panel that is taken.
    pub fn position(&self) -> Option<Pos2> {
        self.taken.then_some(self.position)
    }
}

/// What a press did in a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// Nothing to tell.
    Nothing,
    /// The panel was taken in this frame.
    Taken,
    /// The panel was let go after a move: its top left corner, and the edge
    /// strip the pointer was in.
    Let {
        /// The top left corner of the panel.
        position: Pos2,
        /// The edge strip the pointer was in.
        edge: Option<Edge>,
    },
}

/// Feeds the input of this frame to the press `hold` on a panel drawn at
/// `rect` on `layer` of `window`.
pub fn step(
    context: &Context,
    hold: &mut Option<Hold>,
    layer: LayerId,
    rect: Rect,
    window: Rect,
) -> Step {
    let (pressed, down, origin, pointer, time, touch) = context.input(|input| {
        (
            input.pointer.primary_pressed(),
            input.pointer.primary_down(),
            input.pointer.press_origin(),
            input.pointer.latest_pos(),
            input.time,
            input.any_touches(),
        )
    });
    let (click_distance, click_duration) = context.options(|options| {
        (
            options.input_options.max_click_dist,
            options.input_options.max_click_duration,
        )
    });

    if pressed
        && hold.is_none()
        && let Some(origin) = origin
        && rect.contains(origin)
        && context.layer_id_at(origin) == Some(layer)
    {
        *hold = Some(Hold {
            origin,
            start: time,
            touch,
            grab: origin - rect.min,
            taken: false,
            moved: false,
            position: rect.min,
            edge: None,
        });
    }
    let Some(held) = hold else {
        return Step::Nothing;
    };
    let Some(pointer) = pointer else {
        return Step::Nothing;
    };
    if !down {
        let held = hold.take().expect("a hold");
        return if held.taken && held.moved {
            Step::Let {
                position: held.position,
                edge: held.edge,
            }
        } else {
            Step::Nothing
        };
    }

    let far = (pointer - held.origin).length() > click_distance;
    let mut step = Step::Nothing;
    if !held.taken {
        let take = if held.touch {
            if far {
                *hold = None;
                return Step::Nothing;
            }
            time - held.start >= click_duration
        } else {
            far
        };
        if !take {
            return Step::Nothing;
        }
        held.taken = true;
        step = Step::Taken;
    }
    held.moved |= far;
    held.position = pointer - held.grab;
    if held.moved {
        held.edge = Edge::under(window, pointer, held.edge);
    }
    step
}
