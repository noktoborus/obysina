//! A shake of the grip: the pointer goes to and fro quickly while the panel
//! is held. A caller gives it a meaning of its own.

use egui::Pos2;

/// The least move, in points, that counts as a step in a direction.
const STEP: f32 = 6.0;
/// The turns of direction a shake takes.
const TURNS: usize = 3;
/// The time, in seconds, those turns take at most.
const WINDOW: f64 = 0.8;

/// The turns of direction of a held pointer, along either axis.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Shake {
    /// Where the pointer was when it last made a step, on each axis.
    last: Option<[f32; 2]>,
    /// The direction of the last step on each axis: -1, 0 or 1.
    sign: [i8; 2],
    /// The times of the last turns, the oldest first.
    turns: [Option<f64>; TURNS],
}

impl Shake {
    /// Feeds the pointer at `time`; true when the turns make a shake. The
    /// turns are then forgotten, so a long shake is one shake per
    /// [`TURNS`].
    pub(crate) fn feed(&mut self, time: f64, pointer: Pos2) -> bool {
        let now = [pointer.x, pointer.y];
        let Some(last) = &mut self.last else {
            self.last = Some(now);
            return false;
        };
        for axis in 0..2 {
            let moved = now[axis] - last[axis];
            if moved.abs() < STEP {
                continue;
            }
            let sign = if moved > 0.0 { 1 } else { -1 };
            if self.sign[axis] != 0 && self.sign[axis] != sign {
                self.turns.rotate_left(1);
                self.turns[TURNS - 1] = Some(time);
            }
            self.sign[axis] = sign;
            last[axis] = now[axis];
        }
        let shaken = self
            .turns
            .iter()
            .all(|turn| turn.is_some_and(|turn| time - turn <= WINDOW));
        if shaken {
            self.turns = [None; TURNS];
        }
        shaken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pointer at `x` for each step, a step every `every` seconds.
    fn shake(steps: &[f32], every: f64) -> bool {
        let mut shake = Shake::default();
        steps
            .iter()
            .enumerate()
            .any(|(step, x)| shake.feed(step as f64 * every, Pos2::new(*x, 0.0)))
    }

    #[test]
    fn to_and_fro_quickly_is_a_shake() {
        assert!(shake(&[0.0, 30.0, 0.0, 30.0, 0.0], 0.1));
    }

    #[test]
    fn to_and_fro_slowly_is_no_shake() {
        assert!(!shake(&[0.0, 30.0, 0.0, 30.0, 0.0], 0.5));
    }

    #[test]
    fn a_drag_one_way_is_no_shake() {
        assert!(!shake(&[0.0, 30.0, 60.0, 90.0, 120.0], 0.05));
    }

    #[test]
    fn a_tremble_is_no_shake() {
        assert!(!shake(&[0.0, 2.0, 0.0, 2.0, 0.0, 2.0], 0.05));
    }
}
