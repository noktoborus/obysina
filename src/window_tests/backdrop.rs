//! Tests of the picture under the pattern: chosen, edited by its corners,
//! hidden, removed, and found again when the pattern opens.

use egui::{Event, Modifiers, PointerButton, Pos2};
use egui_kittest::kittest::Queryable;
use rust_i18n::t;

use super::{FREE, SIZE, Window, window, window_in};
use crate::pick::Picked;

/// A PNG of `width` by `height` pixels of one colour.
fn png(width: u32, height: u32) -> Vec<u8> {
    let picture = image::RgbaImage::from_pixel(width, height, image::Rgba([200, 60, 60, 255]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    picture
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("a PNG is written");
    bytes.into_inner()
}

/// Puts a picture of 40 by 20 pixels under the pattern, as a choice of the
/// person would.
fn choose(window: &mut Window) {
    window.harness.state_mut().take_picture(Picked {
        name: Some("photo.png".to_owned()),
        bytes: png(40, 20),
    });
    window.harness.run();
}

/// Lays the picture in the middle of the field, away from the panels.
fn in_the_middle(window: &mut Window) {
    window.harness.state_mut().backdrop.corners = [
        egui::vec2(-6.0, -4.0),
        egui::vec2(6.0, -4.0),
        egui::vec2(6.0, 4.0),
        egui::vec2(-6.0, 4.0),
    ];
    window.harness.run();
}

/// The files of the store of pictures.
fn stored(window: &Window) -> Vec<String> {
    let dir = window.directory.path().join(crate::backdrop::DIRECTORY);
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// Opens the menu of the picture and chooses `entry`.
fn menu(window: &mut Window, entry: &str) {
    window.click(&t!("backdrop.button"));
    window.click(entry);
}

/// The handle of a corner of the picture on the screen.
fn handle(window: &Window, corner: usize) -> Pos2 {
    let field = window.harness.get_by_label(&t!("field")).rect();
    let app = window.harness.state();
    obusina_canvas::handles(field, &app.view, &app.backdrop.corners)[corner]
}

/// The first button pressed at `start`, dragged by `by`, and let go.
fn drag(window: &mut Window, start: Pos2, by: egui::Vec2) {
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    window.harness.event(Event::PointerMoved(start));
    window.harness.event(button(start, true));
    window.harness.step();
    for step in 1..=10 {
        window
            .harness
            .event(Event::PointerMoved(start + by * (step as f32 / 10.0)));
        window.harness.step();
    }
    window.harness.event(button(start + by, false));
    window.harness.run();
}

/// A picture chosen lies over the sheet with its sides as they are, is kept
/// in the store, and is found again when the pattern opens.
#[test]
fn a_chosen_picture_is_found_again() {
    let mut window = window();
    choose(&mut window);

    let app = window.harness.state();
    assert!(app.backdrop.has());
    let [top_left, _, bottom_right, _] = app.backdrop.corners;
    let size = bottom_right - top_left;
    assert!((size.x / size.y - 2.0).abs() < 1e-3, "{size:?}");
    assert_eq!(stored(&window), ["1.png", "1.yaml"]);
    let corners = app.backdrop.corners;
    window.click(&t!("file.save"));
    let Window { harness, directory } = window;
    drop(harness);

    let window = window_in(directory, SIZE);
    assert!(window.harness.state().backdrop.has());
    assert_eq!(window.harness.state().backdrop.corners, corners);
}

/// A handle moves its corner; "put back" puts it back, "keep" keeps and
/// writes it. A press on the field while the corners are edited paints
/// nothing.
#[test]
fn the_corners_are_kept_or_put_back() {
    let mut window = window();
    choose(&mut window);
    in_the_middle(&mut window);
    let before = window.harness.state().backdrop.corners;

    menu(&mut window, &t!("backdrop.edit"));
    assert!(window.harness.state().backdrop.editing());
    window.press(FREE, PointerButton::Primary);
    assert_eq!(window.harness.state().pattern.get(FREE), None, "no bead");
    let corner = handle(&window, 0);
    drag(&mut window, corner, egui::vec2(-40.0, -30.0));
    let moved = window.harness.state().backdrop.corners;
    assert!(
        moved[0].x < before[0].x && moved[0].y < before[0].y,
        "{moved:?}"
    );
    assert_eq!(moved[2], before[2], "the other corners stay");
    window.click(&t!("backdrop.cancel"));
    assert_eq!(window.harness.state().backdrop.corners, before);

    menu(&mut window, &t!("backdrop.edit"));
    let corner = handle(&window, 2);
    drag(&mut window, corner, egui::vec2(30.0, 20.0));
    let kept = window.harness.state().backdrop.corners;
    window.click(&t!("backdrop.done"));
    assert!(!window.harness.state().backdrop.editing());
    let text = std::fs::read_to_string(
        window
            .directory
            .path()
            .join(crate::backdrop::DIRECTORY)
            .join("1.yaml"),
    )
    .expect("the description");
    assert!(text.contains(&kept[2].x.to_string()), "{text}");
}

/// A drag that begins inside the picture moves all of it.
#[test]
fn a_drag_inside_moves_the_whole_picture() {
    let mut window = window();
    choose(&mut window);
    in_the_middle(&mut window);
    let before = window.harness.state().backdrop.corners;
    menu(&mut window, &t!("backdrop.edit"));

    let middle = Pos2::new(
        (handle(&window, 0).x + handle(&window, 2).x) / 2.0,
        (handle(&window, 0).y + handle(&window, 2).y) / 2.0,
    );
    drag(&mut window, middle, egui::vec2(50.0, 0.0));

    let after = window.harness.state().backdrop.corners;
    let moved = after[0] - before[0];
    assert!(moved.x > 0.0 && moved.y.abs() < 1e-3, "{moved:?}");
    assert_eq!(after[2] - before[2], moved);
}

/// Hidden, the picture is kept and written so; the opacity of the slider is
/// written when the menu closes.
#[test]
fn hidden_and_opacity_are_written() {
    let mut window = window();
    choose(&mut window);

    menu(&mut window, &t!("backdrop.hide"));
    assert!(window.harness.state().backdrop.hidden());
    window.click(&t!("backdrop.button"));
    *window
        .harness
        .state()
        .backdrop
        .opacity
        .lock()
        .expect("the opacity") = 0.3;
    window.harness.key_press(egui::Key::Escape);
    window.harness.run();

    let text = std::fs::read_to_string(
        window
            .directory
            .path()
            .join(crate::backdrop::DIRECTORY)
            .join("1.yaml"),
    )
    .expect("the description");
    assert!(text.contains("hidden: true"), "{text}");
    assert!(text.contains("opacity: 0.3"), "{text}");
    menu(&mut window, &t!("backdrop.show"));
    assert!(!window.harness.state().backdrop.hidden());
}

/// "Remove" asks first; "keep" keeps it, "remove" takes it and its copy away.
#[test]
fn a_removed_picture_leaves_nothing() {
    let mut window = window();
    choose(&mut window);

    menu(&mut window, &t!("backdrop.remove"));
    window.click(&t!("backdrop.remove_no"));
    assert!(window.harness.state().backdrop.has());
    menu(&mut window, &t!("backdrop.remove"));
    window.click(&t!("backdrop.remove_yes"));

    assert!(!window.harness.state().backdrop.has());
    assert!(stored(&window).is_empty(), "{:?}", stored(&window));
}

/// A file that is no PNG, JPEG or WebP is refused, and nothing is kept.
#[test]
fn a_picture_of_another_kind_is_refused() {
    let mut window = window();

    window.harness.state_mut().take_picture(Picked {
        name: Some("cat.gif".to_owned()),
        bytes: b"GIF89a....".to_vec(),
    });
    window.harness.run();

    let app = window.harness.state();
    assert!(!app.backdrop.has());
    assert_eq!(app.message.as_deref(), Some(&*t!("error.backdrop.kind")));
    assert!(stored(&window).is_empty());
}
