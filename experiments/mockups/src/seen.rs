//! The widgets of the last frame, read from the tree of accesskit egui
//! builds: every widget a screen reader is told of, with its name and where
//! it stands. A widget of another crate, an entry of a `PlateMenu` or a
//! part of the picker of egui, is seen as well as one of a mockup.

use std::collections::HashMap;
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
        let nodes = &update.nodes;
        let parents: HashMap<accesskit::NodeId, accesskit::NodeId> = nodes
            .iter()
            .flat_map(|(id, node)| node.children().iter().map(move |child| (*child, *id)))
            .collect();
        let widgets = nodes
            .iter()
            .filter_map(|(id, node)| {
                let bounds = node.bounds()?;
                let name = name(node, nodes)?;
                Some(Widget {
                    name,
                    rect: [bounds.x0, bounds.y0, bounds.x1, bounds.y1].map(|side| side as f32),
                    path: path(*id, &parents, nodes),
                    layer: layer(*id, &parents),
                    covered: false,
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

/// The names of the nodes a node stands in, from the outermost; a node with
/// no name is left out.
fn path(
    id: accesskit::NodeId,
    parents: &HashMap<accesskit::NodeId, accesskit::NodeId>,
    nodes: &[(accesskit::NodeId, accesskit::Node)],
) -> Vec<String> {
    let mut path = Vec::new();
    let mut at = id;
    while let Some(parent) = parents.get(&at) {
        if let Some((_, node)) = nodes.iter().find(|(other, _)| other == parent)
            && let Some(name) = name(node, nodes)
        {
            path.push(name);
        }
        at = *parent;
    }
    path.reverse();
    path
}

/// The layer of egui a node is drawn on: the node right under the root
/// that holds it, whose id is the `Id` of the layer.
fn layer(
    id: accesskit::NodeId,
    parents: &HashMap<accesskit::NodeId, accesskit::NodeId>,
) -> Option<u64> {
    let mut at = id;
    while let Some(parent) = parents.get(&at) {
        if !parents.contains_key(parent) {
            return Some(at.0);
        }
        at = *parent;
    }
    None
}
