//! The beads are tessellated only when what they look like changes; a move of
//! the field moves the mesh.

use egui_kittest::Harness;
use obusina_canvas::{Canvas, Turn, View};
use obusina_pattern::{Bead, Cell, Grid, Pattern, Rgb};

/// The field and what each frame of it said.
struct Field {
    pattern: Pattern,
    view: View,
    tessellated: Vec<bool>,
}

/// A harness over a small pattern of one colour.
fn harness() -> Harness<'static, Field> {
    let bead = Bead {
        key: 'A',
        rgb: Rgb([0x1b, 0x1b, 0x1b]),
        catalog: None,
        code: None,
    };
    let mut pattern = Pattern::new(Grid::Peyote, 1.0, vec![bead]).expect("a pattern");
    pattern.set(Cell { column: 0, row: 0 }, Some(0));
    pattern.set(Cell { column: 5, row: 4 }, Some(0));
    let field = Field {
        pattern,
        view: View::default(),
        tessellated: Vec::new(),
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(240.0, 200.0))
        .build_ui_state(
            |ui, field: &mut Field| {
                let shown = Canvas::new(&field.pattern, &mut field.view).show(ui);
                field.tessellated.push(shown.tessellated);
            },
            field,
        );
    harness.run();
    harness
}

/// Steps one frame and says whether it tessellated.
fn frame(harness: &mut Harness<'_, Field>) -> bool {
    harness.step();
    *harness
        .state()
        .tessellated
        .last()
        .expect("a frame was drawn")
}

#[test]
fn a_frame_with_nothing_changed_tessellates_nothing() {
    let mut harness = harness();
    assert!(harness.state().tessellated.iter().any(|done| *done));

    assert!(!frame(&mut harness));
    assert!(!frame(&mut harness));
}

#[test]
fn a_move_of_the_field_moves_the_mesh() {
    let mut harness = harness();

    harness.state_mut().view.offset += egui::vec2(30.0, -12.0);

    assert!(!frame(&mut harness));
}

/// The mesh holds half a screen more on every side: a move of a whole
/// screen leaves it.
#[test]
fn a_move_past_the_cells_kept_tessellates() {
    let mut harness = harness();

    harness.state_mut().view.offset.x += 240.0;

    assert!(frame(&mut harness));
    assert!(!frame(&mut harness));
}

#[test]
fn a_change_of_the_pattern_tessellates_once() {
    let mut harness = harness();

    harness
        .state_mut()
        .pattern
        .set(Cell { column: 2, row: 1 }, Some(0));

    assert!(frame(&mut harness));
    assert!(!frame(&mut harness));
}

#[test]
fn a_zoom_and_a_turn_tessellate() {
    let mut harness = harness();

    harness.state_mut().view.bead *= 1.5;
    assert!(frame(&mut harness));

    harness.state_mut().view.turn(Turn::Clockwise);
    assert!(frame(&mut harness));
    assert!(!frame(&mut harness));
}
