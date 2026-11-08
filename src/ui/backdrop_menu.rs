//! The picture under the pattern: its button of the tools, its menu and the
//! buttons that end the editing of its corners (`docs/backdrop.md`).
//!
//! ```text
//! picture, none ──────────► the choice of a file of the platform
//! picture, hidden ────────► shown again
//! picture, shown or loads ► PlateMenu: the opacity on top, then
//! "background picture" of the menu of the settings ──► the same PlateMenu,
//!               with or without a picture, hidden or not
//!               │──choose a picture…─► the dialog of the platform; the
//!               │                      picture takes the place of the old one
//!               │                      and its corners
//!               │──edit the corners──► a handle on each corner; ✔ keeps, ✖ puts back
//!               │──hide / show───────► kept, not drawn
//!               └──remove…───────────► asks; yes takes it and its copy away
//! closed ──► the opacity is written
//! ```

use std::sync::Arc;

use plate_menu::{MenuItem, Notice};
use rust_i18n::t;

use crate::app::App;
use crate::pick::Choice;
use crate::ui::access::Named;
use crate::ui::{button, icons};

/// The least opacity of the slider: a picture is never quite gone by it.
const LEAST_OPACITY: f32 = 0.1;

/// The button of the picture: with none, the choice of a file; hidden, it
/// shows the picture again; shown, or while it loads, it opens its menu.
pub fn button(app: &mut App, ui: &mut egui::Ui) {
    let present = app.backdrop.has() || app.backdrop.loading();
    let shown = present && !app.backdrop.hidden();
    if !button::toggle(ui, shown, icons::PICTURE, &t!("backdrop.button")).clicked() {
        return;
    }
    if !present {
        app.picker.start(
            ui.ctx(),
            &t!("backdrop.kinds"),
            crate::pick::Kinds::Pictures,
        );
    } else if app.backdrop.hidden() {
        app.backdrop.set_hidden(ui.ctx(), false);
    } else {
        open(app);
    }
}

/// The step a press makes the angles of the picture squarer by: ten degrees.
const SQUARE_STEP: f32 = 10.0 * std::f32::consts::PI / 180.0;

/// While the corners are edited, the buttons that turn the picture a quarter
/// either way and make its corners squarer by `SQUARE_STEP`, then the ones
/// that keep the
/// corners and put them back, which the panel of tools holds in place of its
/// tools; nothing otherwise.
pub fn editing(app: &mut App, ui: &mut egui::Ui) {
    if app.backdrop.editing() {
        let quarter = std::f32::consts::FRAC_PI_2;
        if button::icon(ui, icons::TURN_LEFT, &t!("backdrop.turn_left"), true).clicked() {
            obusina_canvas::turn_corners(&mut app.backdrop.corners, -quarter);
        }
        if button::icon(ui, icons::TURN_RIGHT, &t!("backdrop.turn_right"), true).clicked() {
            obusina_canvas::turn_corners(&mut app.backdrop.corners, quarter);
        }
        if button::icon(ui, icons::SQUARE, &t!("backdrop.square"), true).clicked() {
            obusina_canvas::square_corners(&mut app.backdrop.corners, SQUARE_STEP);
        }
        if button::icon(ui, icons::DONE, &t!("backdrop.done"), true).clicked() {
            app.backdrop.end_editing(ui.ctx(), true);
        }
        if button::icon(ui, icons::CANCEL, &t!("backdrop.cancel"), true).clicked() {
            app.backdrop.end_editing(ui.ctx(), false);
        }
    }
}

/// Opens the menu of the picture: from the button of the picture, and from
/// "background picture" of the menu of the settings.
pub fn open(app: &mut App) {
    let has = app.backdrop.has();
    let hide = if app.backdrop.hidden() {
        t!("backdrop.show")
    } else {
        t!("backdrop.hide")
    };
    let items = vec![
        MenuItem::new("choose", t!("backdrop.choose")),
        MenuItem::new("edit", t!("backdrop.edit")).enabled(has),
        MenuItem::new("hide", hide).enabled(has),
        MenuItem::new("remove", t!("backdrop.remove")).enabled(has),
    ];
    if let Err(error) = app.backdrop_menu.open(items) {
        log::error!("the menu of the picture does not open: {error}");
        return;
    }
    if has {
        let opacity = Arc::clone(&app.backdrop.opacity);
        let name = t!("backdrop.opacity").to_string();
        app.backdrop_menu.notice(Notice::widget(move |ui| {
            let mut value = opacity.lock().expect("the opacity is never poisoned");
            ui.spacing_mut().slider_width = ui.available_width();
            let slider = egui::Slider::new(&mut *value, LEAST_OPACITY..=1.0).show_value(false);
            ui.label(name.as_str());
            ui.add(slider).named(&name);
        }));
    }
}

/// Asks whether to take the picture away.
fn ask_remove(app: &mut App) {
    let items = vec![
        MenuItem::new("remove-yes", t!("backdrop.remove_yes")),
        MenuItem::new("remove-no", t!("backdrop.remove_no")),
    ];
    if let Err(error) = app.backdrop_menu.open(items) {
        log::error!("the question of the picture does not open: {error}");
        return;
    }
    app.backdrop_menu.notice(t!("backdrop.remove_question"));
}

/// The state of the picture, right of its entry in the menu of the settings.
pub fn state(app: &App) -> String {
    let state = if !app.backdrop.has() && !app.backdrop.loading() {
        t!("backdrop.none")
    } else if app.backdrop.hidden() {
        t!("backdrop.hidden")
    } else {
        t!("backdrop.shown")
    };
    state.to_string()
}

/// Draws the menu while it is open, does what is chosen, and takes the answer
/// of a choice of a file once it has come.
pub fn show(app: &mut App, context: &egui::Context) {
    match app.picker.take() {
        Some(Choice::Picked(picked)) => app.take_picture(context, picked),
        Some(Choice::Failed) => app.message = Some(t!("error.pick").to_string()),
        Some(Choice::GivenUp) | None => {}
    }
    app.backdrop_menu.nothing_found(t!("menu.nothing_found"));
    let was_open = app.backdrop_menu.is_open();
    if was_open {
        crate::ui::blocker::under(context, "backdrop-menu", &app.backdrop_menu);
    }
    let chosen = app.backdrop_menu.show(context);
    if was_open && !app.backdrop_menu.is_open() {
        app.backdrop.save(context);
    }
    let Some(chosen) = chosen else {
        return;
    };
    match chosen.id.as_str() {
        "choose" => app
            .picker
            .start(context, &t!("backdrop.kinds"), crate::pick::Kinds::Pictures),
        "edit" => app.backdrop.begin_editing(),
        "hide" => {
            let hidden = !app.backdrop.hidden();
            app.backdrop.set_hidden(context, hidden);
        }
        "remove" => ask_remove(app),
        "remove-yes" => app.backdrop.remove(context),
        _ => {}
    }
}
