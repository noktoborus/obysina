//! The pictures of the cards: each pattern drawn by the code of the field
//! (`obusina_canvas::still`) once into a texture of the graphics chip, kept
//! until its file or the size of its card changes. The list then shows the
//! texture only.
//!
//! ```text
//! frame N    a card asks ──► still(): the shapes, which load the sprites of
//!                           the beads into the textures of egui; a frame more
//! frame N+1  still() again ──► tessellate ──► the renderer of egui draws them
//!                             into a texture of their own ──► registered;
//!                             the card shows it from now on
//! ```
//!
//! Why a frame between: a sprite a shape draws by reaches the graphics chip
//! at the end of the frame it was made in. Why the renderer of the window:
//! its pipeline and its textures are those the field is drawn by, so the
//! picture is the field.

use std::collections::HashMap;
use unifs::SystemTime;

use eframe::egui_wgpu::{RenderState, ScreenDescriptor};
use eframe::wgpu;
use egui::{TextureId, Vec2};
use obusina_pattern::{Pattern, PatternId};

/// What a picture was drawn for: the time of the file and the size in
/// pixels.
type Key = (SystemTime, [u32; 2]);

/// A picture drawn, or asked for.
struct Kept {
    key: Key,
    /// The texture shown; `None` until it is drawn.
    shown: Option<(TextureId, wgpu::Texture)>,
    /// The frame the picture of `key` was asked in, until it is drawn.
    asked: Option<u64>,
}

/// The pictures of the cards.
#[derive(Default)]
pub struct Thumbs {
    kept: HashMap<PatternId, Kept>,
    /// The renderer of the window; `None` where it is not wgpu.
    pub render: Option<RenderState>,
}

impl Thumbs {
    /// The picture of `pattern`, whose file changed at `changed`, for a
    /// place of `size` points; a picture of an earlier change while the new
    /// one is drawn, `None` before the first one is.
    pub fn of(
        &mut self,
        context: &egui::Context,
        pattern: &Pattern,
        changed: SystemTime,
        (size, glow): (Vec2, bool),
    ) -> Option<TextureId> {
        let render = self.render.clone()?;
        let pixels = (size * context.pixels_per_point()).round();
        let key = (
            changed,
            [pixels.x.max(1.0) as u32, pixels.y.max(1.0) as u32],
        );
        let frame = context.cumulative_frame_nr();
        let kept = self.kept.entry(pattern.id()).or_insert(Kept {
            key,
            shown: None,
            asked: None,
        });
        let drawn = kept.shown.is_some() && kept.asked.is_none() && kept.key == key;
        if !drawn {
            match kept.asked {
                Some(asked) if kept.key == key && asked < frame => {
                    let Some(texture) = draw(context, &render, pattern, (key.1, glow)) else {
                        context.request_repaint();
                        return kept.shown.as_ref().map(|(id, _)| *id);
                    };
                    if let Some((id, _)) = kept.shown.replace(texture) {
                        render.renderer.write().free_texture(&id);
                    }
                    kept.asked = None;
                }
                Some(_) if kept.key == key => {}
                _ => {
                    kept.key = key;
                    kept.asked = Some(frame);
                    let _ = obusina_canvas::still(context, pattern, size, glow);
                    context.request_repaint();
                }
            }
        }
        kept.shown.as_ref().map(|(id, _)| *id)
    }
}

impl Thumbs {
    /// How many pictures are drawn, for a test.
    #[cfg(test)]
    pub fn drawn(&self) -> usize {
        self.kept
            .values()
            .filter(|kept| kept.shown.is_some())
            .count()
    }
}

/// Draws `pattern` into a new texture of `pixels`, by the renderer of the
/// window, and registers it with the renderer; `None` while a texture its
/// shapes draw by has not reached the renderer yet.
fn draw(
    context: &egui::Context,
    render: &RenderState,
    pattern: &Pattern,
    (pixels, glow): ([u32; 2], bool),
) -> Option<(TextureId, wgpu::Texture)> {
    let ppp = context.pixels_per_point();
    let size = Vec2::new(pixels[0] as f32, pixels[1] as f32) / ppp;
    let whole = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
    let shapes = obusina_canvas::still(context, pattern, size, glow)
        .into_iter()
        .map(|shape| egui::epaint::ClippedShape::new(whole, shape))
        .collect();
    let primitives = context.tessellate(shapes, ppp);
    let missing = primitives
        .iter()
        .any(|primitive| match &primitive.primitive {
            egui::epaint::Primitive::Mesh(mesh) => {
                render.renderer.read().texture(&mesh.texture_id).is_none()
            }
            egui::epaint::Primitive::Callback(_) => false,
        });
    if missing {
        return None;
    }
    let format = render.target_format;
    let texture = render.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("obusina-thumb"),
        size: wgpu::Extent3d {
            width: pixels[0],
            height: pixels[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let screen = ScreenDescriptor {
        size_in_pixels: pixels,
        pixels_per_point: ppp,
    };
    let mut renderer = render.renderer.write();
    let mut encoder = render
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("obusina-thumb"),
        });
    let mut buffers = renderer.update_buffers(
        &render.device,
        &render.queue,
        &mut encoder,
        &primitives,
        &screen,
    );
    {
        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("obusina-thumb"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            })
            .forget_lifetime();
        renderer.render(&mut pass, &primitives, &screen);
    }
    buffers.push(encoder.finish());
    render.queue.submit(buffers);
    let id = renderer.register_native_texture(&render.device, &target, wgpu::FilterMode::Linear);
    Some((id, texture))
}
