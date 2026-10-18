//! The widgets of the last frame, read from the tree of accesskit egui
//! builds: every widget a screen reader is told of, with its name and where
//! it stands. A widget of another crate, an entry of a `PlateMenu` or a
//! part of the picker of egui, is seen as well as one of a mockup.

use std::sync::{Arc, Mutex};

use egui::accesskit;

use crate::note::Widget;

/// The widgets of the last frame, filled by [`Seen`] after each frame.
pub type Widgets = Arc<Mutex<Vec<Widget>>>;

/// A plugin of egui that keeps the widgets of each frame.
pub struct Seen(pub Widgets);

impl egui::Plugin for Seen {
    fn debug_name(&self) -> &'static str {
        "mockups-seen"
    }

    fn setup(&mut self, context: &egui::Context) {
        context.enable_accesskit();
    }

    fn output_hook(&mut self, _context: &egui::Context, output: &mut egui::FullOutput) {
        let Some(update) = &output.platform_output.accesskit_update else {
            return;
        };
        let widgets = update
            .nodes
            .iter()
            .filter_map(|(_, node)| {
                let bounds = node.bounds()?;
                let name = name(node, &update.nodes)?;
                Some(Widget {
                    name,
                    rect: [bounds.x0, bounds.y0, bounds.x1, bounds.y1].map(|side| side as f32),
                })
            })
            .collect();
        *self.0.lock().expect("the widgets are never poisoned") = widgets;
    }
}

/// What a screen reader calls a node: its label, its value, or the label
/// of the node that labels it. A node with none is no widget a person
/// names.
fn name(node: &accesskit::Node, nodes: &[(accesskit::NodeId, accesskit::Node)]) -> Option<String> {
    let own = node
        .label()
        .or_else(|| node.value())
        .map(str::to_string)
        .filter(|name| !name.trim().is_empty());
    own.or_else(|| {
        node.labelled_by().iter().find_map(|id| {
            nodes
                .iter()
                .find(|(other, _)| other == id)
                .and_then(|(_, other)| other.label())
                .map(str::to_string)
        })
    })
}
