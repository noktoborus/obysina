//! The place and the view of the two panels, the tools and the beads, kept
//! apart for a window that stands and one that lies (`Posture`).
//!
//! ```text
//! a frame ──► the posture of the window
//!             ├─ the settings keep a place for the panel ──► that place
//!             └─ none ──► both panels stood (`arrange`), and kept
//! a panel let go ──► its place kept for the posture
//! a view chosen in the menu ──► kept for the posture
//! a panel taken ──► a short vibration, on android
//! ```

use obusina_panel::{PanelResponse, Placement};

use crate::app::App;
use crate::settings::{PanelView, Posture};
use crate::ui::arrange::{self, arrange};

/// Where a panel stands in this frame, and how.
pub struct Stood {
    /// Its place.
    pub placement: Placement,
    /// How its buttons stand.
    pub view: PanelView,
    /// The posture of the window it is kept for.
    pub posture: Posture,
}

/// The place and the view of the panel `name` in the posture of the window
/// now. When the settings keep none, both panels are stood and kept.
pub fn stand(app: &mut App, context: &egui::Context, name: &str) -> Stood {
    let window = context.content_rect();
    let posture = Posture::of(window.size());
    if let Some((placement, view)) = app.settings.panel(posture, name) {
        return Stood {
            placement,
            view,
            posture,
        };
    }
    let tools_size = |view| crate::ui::tools::size(context, view);
    let beads_size = |view| crate::ui::palette::size(app, context, view);
    let tools = arrange::Panel {
        line: [PanelView::Row, PanelView::Column],
        compact: [PanelView::TwoRows, PanelView::TwoColumns],
        size: &tools_size,
    };
    let beads = arrange::Panel {
        line: [PanelView::Row, PanelView::Column],
        compact: [PanelView::Block, PanelView::Block],
        size: &beads_size,
    };
    let stands = arrange(window, posture, &tools, &beads);
    let names = [crate::ui::tools::NAME, crate::ui::palette::NAME];
    if window.is_positive() {
        for (panel, (placement, view)) in names.into_iter().zip(stands) {
            if app.settings.panel(posture, panel).is_none() {
                app.keep_panel(posture, panel, placement, view);
            }
        }
    }
    let (placement, view) = names
        .into_iter()
        .zip(stands)
        .find(|(panel, _)| *panel == name)
        .map(|(_, stand)| stand)
        .unwrap_or((Placement::new(egui::Vec2::ZERO), PanelView::Row));
    Stood {
        placement,
        view,
        posture,
    }
}

/// Keeps the place of a panel let go, and answers a panel taken.
pub fn after<R>(
    app: &mut App,
    posture: Posture,
    name: &str,
    view: PanelView,
    shown: &PanelResponse<R>,
) {
    if shown.changed {
        app.keep_panel(posture, name, shown.placement, view);
    }
    if shown.taken {
        #[cfg(target_os = "android")]
        crate::activity::tick();
    }
}

/// Changes the view of the panel `name` in the posture of the window now.
pub fn set_view(app: &mut App, context: &egui::Context, name: &str, view: PanelView) {
    let Stood {
        placement, posture, ..
    } = stand(app, context, name);
    app.keep_panel(posture, name, placement, view);
}
