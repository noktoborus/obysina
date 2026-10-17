//! The mesh of the beads, kept from frame to frame.
//!
//! The tessellator of egui turns every rounded bead into triangles. That is
//! the cost of a frame, so it is paid only when what the beads look like
//! changes: the pattern, the size of a bead, the turn of the field or the
//! pixels of the screen. A move of the field only moves the mesh.

use std::sync::Arc;

use egui::epaint::{Mesh, Tessellator};
use egui::{Pos2, Ui};

/// What the mesh is built from, but its place.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Key {
    /// The revision of the pattern.
    pub revision: u64,
    /// The height of a bead, in points.
    pub bead: f32,
    /// How far the field is turned.
    pub degrees: u16,
    /// Physical pixels to a point.
    pub pixels_per_point: f32,
}

/// A mesh and what it was built from.
#[derive(Clone, Default)]
pub struct Kept {
    /// What it was built from.
    pub key: Key,
    /// The top left of the pattern the vertices stand at.
    pub origin: Pos2,
    /// The beads.
    pub mesh: Arc<Mesh>,
}

impl Kept {
    /// Moves the mesh to `origin`.
    ///
    /// The vertices move in place: the shape of the last frame, the other
    /// holder of the mesh, is gone by the next one, so nothing is copied.
    pub fn move_to(&mut self, origin: Pos2) {
        if self.origin != origin {
            Arc::make_mut(&mut self.mesh).translate(origin - self.origin);
            self.origin = origin;
        }
    }
}

/// Tessellates `shapes`, standing at `origin`, as egui would at the end of a
/// frame.
pub fn tessellate(ui: &Ui, key: Key, origin: Pos2, shapes: Vec<egui::Shape>) -> Kept {
    let options = ui.ctx().tessellation_options(|options| *options);
    let font_size = ui.ctx().fonts(|fonts| fonts.font_image_size());
    let mut tessellator = Tessellator::new(key.pixels_per_point, options, font_size, Vec::new());
    let mut mesh = Mesh::default();
    for shape in shapes {
        tessellator.tessellate_shape(shape, &mut mesh);
    }
    Kept {
        key,
        origin,
        mesh: Arc::new(mesh),
    }
}
