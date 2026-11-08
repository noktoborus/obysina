//! The floating panel of tools. While the corners of the picture under the
//! pattern are edited, it holds only the two buttons that end the editing.
//! A locked pattern is only looked at: the panel holds the settings, the
//! shadows, the underlay and the way back.
//!
//! ```text
//! row          ↶ ↷  ⚙ ▓  🖼  ⬅          a line goes on in the next line
//! column       ↶                         where the window ends
//!              ↷ …
//! two rows     ↶ ⚙ 🖼                    two columns  ↶ ↷
//!              ↷ ▓ ⬅                                 ⚙ ▓
//!                                                     🖼 ⬅
//! ```
//!
//! A space stands between two groups: undo and redo, the settings and the
//! shadows, the underlay, the way back.

use obusina_panel::{FloatingPanel, Orientation, Room};
use rust_i18n::t;

use crate::app::App;
use crate::settings::PanelView;
use crate::ui::panels::{self, Stood};
use crate::ui::{button, icons};

/// The name the place of the panel is kept by.
pub const NAME: &str = "tools";
/// The views the menu offers.
pub const VIEWS: [PanelView; 4] = [
    PanelView::Row,
    PanelView::Column,
    PanelView::TwoRows,
    PanelView::TwoColumns,
];

/// The space between two groups, in item spacings.
const GAP: f32 = 2.0;

/// A button of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tool {
    Undo,
    Redo,
    Settings,
    Shadows,
    Picture,
    Back,
}

/// The groups of buttons of the panel, as the pattern is now.
fn groups(app: &App) -> Vec<Vec<Tool>> {
    let mut groups = Vec::new();
    if !app.pattern.locked() {
        groups.push(vec![Tool::Undo, Tool::Redo]);
    }
    groups.extend([
        vec![Tool::Settings, Tool::Shadows],
        vec![Tool::Picture],
        vec![Tool::Back],
    ]);
    groups
}

/// The size of the panel of the tools of a pattern that is not locked in
/// `view`, for the panels to be stood (`crate::ui::arrange`).
pub fn size(context: &egui::Context, view: PanelView) -> egui::Vec2 {
    let spacing = context.global_style().spacing.item_spacing;
    let margin = egui::Frame::window(&context.global_style())
        .total_margin()
        .sum();
    let (buttons, groups) = (6.0, 4.0);
    let along = |count: f32, gaps: f32, size: f32, spacing: f32| {
        count * size + (count - 1.0) * spacing + gaps * GAP * spacing
    };
    let (b, s) = (button::SIZE, spacing);
    let size = match view {
        PanelView::Row | PanelView::Block => {
            egui::vec2(along(buttons, groups - 1.0, b.x, s.x), b.y)
        }
        PanelView::Column => egui::vec2(b.x, along(buttons, groups - 1.0, b.y, s.y)),
        PanelView::TwoRows => egui::vec2(along(3.0, 2.0, b.x, s.x), 2.0 * b.y + s.y),
        PanelView::TwoColumns => egui::vec2(2.0 * b.x + s.x, along(3.0, 2.0, b.y, s.y)),
    };
    size + margin
}

/// Shows the panel of tools.
pub fn show(app: &mut App, context: &egui::Context) {
    let Stood {
        placement,
        view,
        posture,
    } = panels::stand(app, context, NAME);
    let editing = app.backdrop.editing();
    let groups = groups(app);
    let shown = FloatingPanel::new(NAME, t!("panel.tools"))
        .placement(placement)
        .orientation(view.orientation())
        .content((editing, &groups, view))
        .show(context, |ui, room| {
            if editing {
                crate::ui::backdrop_menu::editing(app, ui);
            } else {
                match view {
                    PanelView::TwoRows | PanelView::TwoColumns => pairs(app, ui, &groups, view),
                    _ => lines(app, ui, &groups, room),
                }
            }
            if let Some(message) = &app.message {
                ui.label(message);
            }
        });
    panels::after(app, context, (posture, NAME, view), &shown);
}

/// The buttons in one line along the panel, going on in the next line where
/// the window ends. The lines are one block, laid out like any other widget
/// of the panel, so the block stands in the middle across the panel.
fn lines(app: &mut App, ui: &mut egui::Ui, groups: &[Vec<Tool>], room: Room) {
    let spacing = ui.spacing().item_spacing;
    let step = match room.orientation {
        Orientation::Horizontal => button::SIZE.x + spacing.x,
        Orientation::Vertical => button::SIZE.y + spacing.y,
    };
    let space = match room.orientation {
        Orientation::Horizontal => spacing.x,
        Orientation::Vertical => spacing.y,
    };
    let gap = GAP * space;
    let mut lines: Vec<Vec<(Tool, bool)>> = vec![Vec::new()];
    let mut length = 0.0;
    for group in groups {
        for (index, tool) in group.iter().enumerate() {
            let starts_group = index == 0 && !lines[0].is_empty();
            let wanted = step + if starts_group { gap } else { 0.0 };
            let line = lines.last_mut().expect("a line");
            if !line.is_empty() && length + wanted > room.length + space {
                lines.push(Vec::new());
                length = 0.0;
            }
            let line = lines.last_mut().expect("a line");
            let gap_before = starts_group && !line.is_empty();
            length += step + if gap_before { gap } else { 0.0 };
            line.push((*tool, gap_before));
        }
    }
    let across = |ui: &mut egui::Ui, add: &mut dyn FnMut(&mut egui::Ui)| match room.orientation {
        Orientation::Horizontal => ui.vertical(|ui| add(ui)),
        Orientation::Vertical => ui.horizontal(|ui| add(ui)),
    };
    across(ui, &mut |ui| {
        for line in &lines {
            let along =
                |ui: &mut egui::Ui, add: &mut dyn FnMut(&mut egui::Ui)| match room.orientation {
                    Orientation::Horizontal => ui.horizontal(|ui| add(ui)),
                    Orientation::Vertical => ui.vertical(|ui| add(ui)),
                };
            along(ui, &mut |ui| {
                for (tool, gap_before) in line {
                    if *gap_before {
                        gap_in(ui, room.orientation);
                    }
                    draw(app, ui, *tool);
                }
            });
        }
    });
}

/// The buttons in two lines along the panel: each group of two stands
/// across it, as a pair.
fn pairs(app: &mut App, ui: &mut egui::Ui, groups: &[Vec<Tool>], view: PanelView) {
    let flat: Vec<Tool> = groups.iter().flatten().copied().collect();
    let orientation = view.orientation();
    for (index, pair) in flat.chunks(2).enumerate() {
        if index > 0 {
            gap_in(ui, orientation);
        }
        let mut add = |ui: &mut egui::Ui| {
            for tool in pair {
                draw(app, ui, *tool);
            }
        };
        match orientation {
            Orientation::Horizontal => ui.vertical(|ui| add(ui)),
            Orientation::Vertical => ui.horizontal(|ui| add(ui)),
        };
    }
}

/// A space between two groups of buttons. A separator is not used: in a
/// column it would take all the width it is offered.
pub fn gap_in(ui: &mut egui::Ui, orientation: Orientation) {
    let space = ui.spacing().item_spacing * GAP;
    match orientation {
        Orientation::Horizontal => ui.add_space(space.x),
        Orientation::Vertical => ui.add_space(space.y),
    }
}

/// One button and what a press on it does.
fn draw(app: &mut App, ui: &mut egui::Ui, tool: Tool) {
    match tool {
        Tool::Undo => {
            if button::icon(ui, icons::UNDO, &t!("edit.undo"), app.history.can_undo()).clicked() {
                app.undo();
            }
        }
        Tool::Redo => {
            if button::icon(ui, icons::REDO, &t!("edit.redo"), app.history.can_redo()).clicked() {
                app.redo();
            }
        }
        Tool::Settings => {
            if button::icon(ui, icons::SETTINGS, &t!("settings.open"), true).clicked() {
                crate::ui::settings_menu::open(app);
            }
        }
        Tool::Shadows => {
            let shadows = app.settings.shadows();
            if button::toggle(ui, shadows, icons::SHADOW, &t!("view.shadows")).clicked() {
                app.keep_shadows(!shadows);
            }
        }
        Tool::Picture => crate::ui::backdrop_menu::button(app, ui),
        Tool::Back => {
            let listed = app.library.is_some();
            if button::icon(ui, icons::BACK, &t!("projects.back"), listed).clicked() {
                let time = ui.input(|input| input.time);
                app.back_to_list(ui.ctx(), time);
            }
        }
    }
}
