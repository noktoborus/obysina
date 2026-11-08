//! The floating panel of beads: the colours of the palette shown, each with
//! the number of its place, and the empty place `0`, shown by `icons::EMPTY`
//! on the ground of the panel, outlined;
//! the one in hand marked
//! by a bar under it, a hidden one hatched.
//! A row or a column runs along the panel and goes on in the next line where
//! the window ends; a block stands as the keys of a numpad (decision 10 of
//! `docs/palette.md`). The menu of the settings chooses among them
//! (`crate::ui::panels`).
//!
//! ```text
//! line   < 1 2 3 4 5 6 7 8 9 0 > 🎨
//!
//! block  7 8 9      two palettes  7 8 9      three or more  7 8 9
//!        4 5 6                    4 5 6                     4 5 6
//!        1 2 3                    1 2 3                     1 2 3
//!        0 🎨                     0 🎨 ↔                    [ 0 ]
//!                                                           < 🎨 >
//! ```
//!
//! With two palettes one button `↔` goes to the other, after `0`; with more,
//! `<` and `>`, in the block on a row of their own; with one, none. `🎨`
//! opens the window of the palettes. A tap on a colour or `0` takes it in
//! hand; a second tap does nothing more.

use egui::{Color32, vec2};
use rust_i18n::t;

use obusina_panel::{FloatingPanel, Orientation, Room};
use obusina_pattern::{ColourId, Rgb};

use crate::app::App;
use crate::settings::PanelView;
use crate::ui::palette_window::{block, step};
use crate::ui::panels::{self, Stood};
use crate::ui::{button, icons};

/// The name the place of the panel is kept by.
pub const NAME: &str = "palette";
/// The views the menu offers.
pub const VIEWS: [PanelView; 3] = [PanelView::Row, PanelView::Column, PanelView::Block];
/// The space between two swatches.
const SPACING: egui::Vec2 = vec2(4.0, 4.0);

/// One button of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Entry {
    /// A colour: its place and its id.
    Colour(usize, ColourId),
    /// The empty place, `0`.
    Empty,
    /// The empty place, `0`, as wide as two swatches: the block with more
    /// than two palettes.
    WideEmpty,
    /// The previous palette.
    Back,
    /// The next palette.
    On,
    /// The other palette, of two.
    Other,
    /// The window of the palettes.
    Palettes,
}

/// Shows the panel of beads.
pub fn show(app: &mut App, context: &egui::Context) {
    let Stood {
        placement,
        view,
        posture,
    } = panels::stand(app, context, NAME);
    let content = (view, entries(app));
    let shown = FloatingPanel::new(NAME, t!("panel.palette"))
        .placement(placement)
        .orientation(view.orientation())
        .content(content)
        .show(context, |ui, room| {
            ui.spacing_mut().item_spacing = SPACING;
            match view {
                PanelView::Block => numpad(app, ui),
                _ => line(app, ui, room),
            }
        });
    panels::after(app, posture, NAME, view, &shown);
}

/// The size of the panel in `view`, for the panels to be stood
/// (`crate::ui::arrange`).
pub fn size(app: &App, context: &egui::Context, view: PanelView) -> egui::Vec2 {
    let margin = egui::Frame::window(&context.global_style())
        .total_margin()
        .sum();
    let step = button::SIZE + SPACING;
    let along = |count: usize, step: f32, spacing: f32| count as f32 * step - spacing;
    let count = entries(app).len();
    let size = match view {
        PanelView::Row | PanelView::TwoRows => {
            egui::vec2(along(count, step.x, SPACING.x), button::SIZE.y)
        }
        PanelView::Column | PanelView::TwoColumns => {
            egui::vec2(button::SIZE.x, along(count, step.y, SPACING.y))
        }
        PanelView::Block => {
            let switch_row = usize::from(app.pattern.palettes().len() > 2);
            let rows = shown(app).len().div_ceil(3) + 1 + switch_row;
            egui::vec2(along(3, step.x, SPACING.x), along(rows, step.y, SPACING.y))
        }
    };
    size + margin
}

/// The buttons of the line, in order.
fn entries(app: &App) -> Vec<Entry> {
    let palettes = app.pattern.palettes().len();
    let colours = shown(app)
        .into_iter()
        .map(|(place, id)| Entry::Colour(place, id));
    let mut entries = Vec::new();
    if palettes > 2 {
        entries.push(Entry::Back);
    }
    entries.extend(colours);
    entries.push(Entry::Empty);
    match palettes {
        1 => {}
        2 => entries.push(Entry::Other),
        _ => entries.push(Entry::On),
    }
    entries.push(Entry::Palettes);
    entries
}

/// How many of `count` buttons stand in one line, so they fit the length of
/// `room`. Along a row they are one row, two or more when the window is too
/// narrow; along a column they are one column, two or more when the window
/// is too short.
fn per_line(room: Room, count: usize) -> usize {
    let step = button::SIZE + SPACING;
    match room.orientation {
        Orientation::Horizontal => {
            let fit = ((room.length + SPACING.x) / step.x).floor() as usize;
            count.min(fit).max(1)
        }
        Orientation::Vertical => {
            let lines = (((room.length + SPACING.y) / step.y).floor() as usize).max(1);
            count.div_ceil(lines).max(1)
        }
    }
}

/// The line: the buttons in one line along the panel. The lines are one
/// block, laid out like any other widget of the panel, so the block stands in
/// the middle across the panel with the rest.
fn line(app: &mut App, ui: &mut egui::Ui, room: Room) {
    let entries = entries(app);
    let per_line = per_line(room, entries.len());
    ui.vertical(|ui| {
        for chunk in entries.chunks(per_line) {
            ui.horizontal(|ui| {
                for entry in chunk {
                    button_of(app, ui, *entry);
                }
            });
        }
    });
}

/// The block: the colours as the keys of a numpad, rows with no colour
/// folded, a gap for an empty place. Under them `0` and `🎨`, and `↔` with
/// two palettes; with more, a wide `0`, and `< 🎨 >` on a row under it.
fn numpad(app: &mut App, ui: &mut egui::Ui) {
    let colours = shown(app);
    let rows = block(obusina_pattern::MAX_COLOURS);
    let palettes = app.pattern.palettes().len();
    ui.vertical(|ui| {
        for row in rows {
            if !row
                .iter()
                .any(|place| colours.iter().any(|(at, _)| at == place))
            {
                continue;
            }
            ui.horizontal(|ui| {
                for place in row {
                    match colours.iter().find(|(at, _)| *at == place) {
                        Some((place, id)) => button_of(app, ui, Entry::Colour(*place, *id)),
                        None => {
                            ui.allocate_exact_size(button::SIZE, egui::Sense::hover());
                        }
                    }
                }
            });
        }
        if palettes > 2 {
            ui.horizontal(|ui| button_of(app, ui, Entry::WideEmpty));
            ui.horizontal(|ui| {
                button_of(app, ui, Entry::Back);
                button_of(app, ui, Entry::Palettes);
                button_of(app, ui, Entry::On);
            });
        } else {
            ui.horizontal(|ui| {
                button_of(app, ui, Entry::Empty);
                button_of(app, ui, Entry::Palettes);
                if palettes == 2 {
                    button_of(app, ui, Entry::Other);
                }
            });
        }
    });
}

/// One button of the panel and what a tap on it does.
fn button_of(app: &mut App, ui: &mut egui::Ui, entry: Entry) {
    match entry {
        Entry::Colour(place, id) => {
            let colour = app.pattern.colour(id);
            let rgb = colour.and_then(|colour| colour.rgb).unwrap_or(Rgb([0; 3]));
            let name = t!(
                "palette.bead",
                key = place + 1,
                rgb = rgb,
                count = app.pattern.count(id)
            );
            let fill = colour
                .and_then(obusina_pattern::Colour::seen)
                .map(|Rgb([red, green, blue])| Color32::from_rgb(red, green, blue));
            let in_hand = app.colour == Some(id);
            let face = (fill, app.settings.glow());
            if button::colour(ui, face, &(place + 1).to_string(), in_hand, &name).clicked() {
                tap(app, Some(id));
            }
        }
        Entry::Empty | Entry::WideEmpty => {
            let fill = ui.visuals().window_fill;
            let in_hand = app.colour.is_none();
            let name = t!("palette.empty");
            let response = if entry == Entry::WideEmpty {
                button::wide(ui, fill, icons::EMPTY, in_hand, &name)
            } else {
                button::colour(ui, (Some(fill), false), icons::EMPTY, in_hand, &name)
            };
            if response.clicked() {
                tap(app, None);
            }
        }
        Entry::Back => {
            if button::icon(ui, "<", &t!("palette.previous"), true).clicked() {
                step(app, -1);
            }
        }
        Entry::On => {
            if button::icon(ui, ">", &t!("palette.next"), true).clicked() {
                step(app, 1);
            }
        }
        Entry::Other => {
            if button::icon(ui, icons::SWITCH, &t!("palette.switch"), true).clicked() {
                step(app, 1);
            }
        }
        Entry::Palettes => {
            if button::icon(ui, icons::PALETTE, &t!("palette.settings"), true).clicked() {
                app.palette_window.open = true;
            }
        }
    }
}

/// A tap on a colour or the empty place: it goes in hand.
fn tap(app: &mut App, colour: Option<ColourId>) {
    app.colour = colour;
}

/// The colours the panel shows: the places of the palette shown, each with
/// its id. A removed colour does not stand on the panel.
fn shown(app: &App) -> Vec<(usize, ColourId)> {
    app.pattern.palettes()[app.palette]
        .places()
        .iter()
        .enumerate()
        .filter_map(|(place, colour)| {
            let colour = colour.as_ref().filter(|colour| colour.rgb.is_some())?;
            Some((place, colour.id))
        })
        .collect()
}
