#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
// The code of the original is kept as it is, so the copy is read against it
// (`RULE notify.patch`); edition 2024 would fold its nested `if`s.
#![allow(clippy::collapsible_if)]

mod toast;
pub use toast::*;
mod anchor;
pub use anchor::*;

#[doc(hidden)]
pub use egui::__run_test_ctx;
use egui::text::TextWrapping;
use egui::{
    Align, Color32, Context, CornerRadius, FontId, FontSelection, Id, LayerId, Order, Rect, Sense,
    Shadow, Stroke, TextWrapMode, Vec2, WidgetText, vec2,
};

pub(crate) const TOAST_WIDTH: f32 = 180.;
pub(crate) const TOAST_HEIGHT: f32 = 34.;

const ERROR_COLOR: Color32 = Color32::from_rgb(200, 90, 90);
const INFO_COLOR: Color32 = Color32::from_rgb(150, 200, 210);
const WARNING_COLOR: Color32 = Color32::from_rgb(230, 220, 140);
const SUCCESS_COLOR: Color32 = Color32::from_rgb(140, 230, 140);

/// Main notifications collector.
/// # Usage
/// You need to create [`Toasts`] once and call `.show(ctx)` in every frame.
/// ```
/// # use std::time::Duration;
/// use egui_notify::Toasts;
///
/// # egui_notify::__run_test_ctx(|ctx| {
/// let mut t = Toasts::default();
/// t.info("Hello, World!").duration(Some(Duration::from_secs(5))).closable(true);
/// // More app code
/// t.show(ctx);
/// # });
/// ```
pub struct Toasts {
    toasts: Vec<Toast>,
    anchor: Anchor,
    margin: Vec2,
    spacing: f32,
    padding: Vec2,
    reverse: bool,
    speed: f32,
    font: Option<FontId>,
    shadow: Option<Shadow>,
    held: bool,
    /// The identifier the next toast takes.
    next_id: u64,
}

impl Toasts {
    /// Creates new [`Toasts`] instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            anchor: Anchor::TopRight,
            margin: vec2(8., 8.),
            toasts: vec![],
            spacing: 8.,
            padding: vec2(10., 10.),
            held: false,
            speed: 4.,
            reverse: false,
            font: None,
            shadow: None,
            next_id: 1,
        }
    }

    /// Adds new toast to the collection.
    /// By default adds toast at the end of the list, can be changed with `self.reverse`.
    #[allow(clippy::unwrap_used)] // We know that the index is valid
    pub fn add(&mut self, mut toast: Toast) -> &mut Toast {
        toast.id = ToastId(self.next_id);
        self.next_id += 1;
        if self.reverse {
            self.toasts.insert(0, toast);
            return self.toasts.get_mut(0).unwrap();
        }
        self.toasts.push(toast);
        let l = self.toasts.len() - 1;
        self.toasts.get_mut(l).unwrap()
    }

    /// Dismisses the oldest toast
    pub fn dismiss_oldest_toast(&mut self) {
        if let Some(toast) = self.toasts.get_mut(0) {
            toast.dismiss();
        }
    }

    /// Dismisses the most recent toast
    pub fn dismiss_latest_toast(&mut self) {
        if let Some(toast) = self.toasts.last_mut() {
            toast.dismiss();
        }
    }

    /// Dismisses all toasts
    pub fn dismiss_all_toasts(&mut self) {
        for toast in &mut self.toasts {
            toast.dismiss();
        }
    }

    /// Returns the number of toast items.
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Returns `true` if there are no toast items.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Shortcut for adding a toast with info `success`.
    pub fn success(&mut self, caption: impl Into<WidgetText>) -> &mut Toast {
        self.add(Toast::success(caption))
    }

    /// Shortcut for adding a toast with info `level`.
    pub fn info(&mut self, caption: impl Into<WidgetText>) -> &mut Toast {
        self.add(Toast::info(caption))
    }

    /// Shortcut for adding a toast with warning `level`.
    pub fn warning(&mut self, caption: impl Into<WidgetText>) -> &mut Toast {
        self.add(Toast::warning(caption))
    }

    /// Shortcut for adding a toast with error `level`.
    pub fn error(&mut self, caption: impl Into<WidgetText>) -> &mut Toast {
        self.add(Toast::error(caption))
    }

    /// Shortcut for adding a toast with no level.
    pub fn basic(&mut self, caption: impl Into<WidgetText>) -> &mut Toast {
        self.add(Toast::basic(caption))
    }

    /// Shortcut for adding a toast with custom `level`.
    pub fn custom(
        &mut self,
        caption: impl Into<WidgetText>,
        level_string: String,
        level_color: egui::Color32,
    ) -> &mut Toast {
        self.add(Toast::custom(
            caption,
            ToastLevel::Custom(level_string, level_color),
        ))
    }

    /// Should toasts be added in reverse order?
    pub const fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Where toasts should appear.
    pub const fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets spacing between adjacent toasts.
    pub const fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Margin or distance from screen to toasts' bounding boxes
    pub const fn with_margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Enables the use of a shadow for toasts.
    pub const fn with_shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    /// Padding or distance from toasts' bounding boxes to inner contents.
    pub const fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }

    /// Changes the default font used for all toasts.
    pub fn with_default_font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }
}

impl Toasts {
    /// Displays toast queue, and gives back the toasts whose action was
    /// pressed in this frame.
    pub fn show(&mut self, ctx: &Context) -> Vec<ToastId> {
        let mut acted = Vec::new();
        let Self {
            anchor,
            margin,
            spacing,
            padding,
            toasts,
            held,
            speed,
            ..
        } = self;

        let mut pos = anchor.screen_corner(ctx.input(|i| i.content_rect().max), *margin);
        let p = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("egui-notify")));

        // `held` used to prevent sticky removal
        if ctx.input(|i| i.pointer.primary_released()) {
            *held = false;
        }

        let visuals = ctx.global_style().visuals.widgets.noninteractive;
        let mut update = false;

        toasts.retain_mut(|toast| {
            // Start disappearing expired toasts
            if let Some((_initial_d, current_d)) = toast.duration {
                if current_d <= 0. {
                    toast.state = ToastState::Disappear;
                }
            }

            let anim_offset = toast.width * (1. - ease_in_cubic(toast.value));
            pos.x += anim_offset * anchor.anim_side();
            let rect = toast.calc_anchored_rect(pos, *anchor);

            if let Some((_, d)) = toast.duration.as_mut() {
                // Check if we hover over the toast and if true don't decrease the duration
                let hover_pos = ctx.input(|i| i.pointer.hover_pos());
                let is_outside_rect = hover_pos.is_none_or(|pos| !rect.contains(pos));

                if is_outside_rect && toast.state.idling() {
                    *d -= ctx.input(|i| i.stable_dt);
                    update = true;
                }
            }

            let caption_galley = toast.caption.clone().into_galley_impl(
                ctx,
                ctx.global_style().as_ref(),
                TextWrapping::from_wrap_mode_and_width(TextWrapMode::Extend, f32::INFINITY),
                FontSelection::Default,
                Align::LEFT,
            );

            let (caption_width, caption_height) =
                (caption_galley.rect.width(), caption_galley.rect.height());

            let line_count = caption_galley.rows.len().max(1);
            let icon_width = caption_height / line_count as f32;
            let rounding = CornerRadius::same(4);

            // Create toast icon
            let icon_font = FontId::proportional(icon_width);
            let icon_galley =
                match &toast.level {
                    ToastLevel::Info => {
                        Some(ctx.fonts_mut(|f| {
                            f.layout("ℹ".into(), icon_font, INFO_COLOR, f32::INFINITY)
                        }))
                    }
                    ToastLevel::Warning => Some(ctx.fonts_mut(|f| {
                        f.layout("⚠".into(), icon_font, WARNING_COLOR, f32::INFINITY)
                    })),
                    ToastLevel::Error => Some(ctx.fonts_mut(|f| {
                        f.layout("！".into(), icon_font, ERROR_COLOR, f32::INFINITY)
                    })),
                    ToastLevel::Success => Some(ctx.fonts_mut(|f| {
                        f.layout("✅".into(), icon_font, SUCCESS_COLOR, f32::INFINITY)
                    })),
                    ToastLevel::Custom(s, c) => {
                        Some(ctx.fonts_mut(|f| f.layout(s.clone(), icon_font, *c, f32::INFINITY)))
                    }
                    ToastLevel::None => None,
                };

            let (action_width, action_height) =
                icon_galley.as_ref().map_or((0., 0.), |icon_galley| {
                    (icon_galley.rect.width(), icon_galley.rect.height())
                });

            // Create closing cross
            let cross_galley = if toast.closable {
                let cross_fid = FontId::proportional(icon_width);
                let cross_galley = ctx.fonts_mut(|f| {
                    f.layout(
                        "❌".into(),
                        cross_fid,
                        visuals.fg_stroke.color,
                        f32::INFINITY,
                    )
                });
                Some(cross_galley)
            } else {
                None
            };

            let (cross_width, cross_height) =
                cross_galley.as_ref().map_or((0., 0.), |cross_galley| {
                    (cross_galley.rect.width(), cross_galley.rect.height())
                });

            // Create the words of the action
            let action_galley = toast.action.clone().map(|words| {
                words.into_galley_impl(
                    ctx,
                    ctx.global_style().as_ref(),
                    TextWrapping::from_wrap_mode_and_width(TextWrapMode::Extend, f32::INFINITY),
                    FontSelection::Default,
                    Align::LEFT,
                )
            });
            let button_width = action_galley
                .as_ref()
                .map_or(0., |galley| galley.rect.width() + 2. * padding.x);

            let icon_x_padding = (0., padding.x);
            let cross_x_padding = (padding.x, 0.);

            let icon_width_padded = if icon_width == 0. {
                0.
            } else {
                icon_width + icon_x_padding.0 + icon_x_padding.1
            };
            let cross_width_padded = if cross_width == 0. {
                0.
            } else {
                cross_width + cross_x_padding.0 + cross_x_padding.1
            };

            toast.width = padding.x.mul_add(
                2.,
                icon_width_padded + caption_width + button_width + cross_width_padded,
            );
            toast.height = padding
                .y
                .mul_add(2., action_height.max(caption_height).max(cross_height));

            // Required due to positioning of the next toast
            pos.x -= anim_offset * anchor.anim_side();

            // Draw shadow
            if let Some(shadow) = self.shadow {
                let s = shadow.as_shape(rect, rounding);
                p.add(s);
            }

            // Draw background
            p.rect_filled(rect, rounding, visuals.bg_fill);

            // Paint icon
            if let Some((icon_galley, true)) =
                icon_galley.zip(Some(toast.level != ToastLevel::None))
            {
                let oy = toast.height / 2. - action_height / 2.;
                let ox = padding.x + icon_x_padding.0;
                p.galley(
                    rect.min + vec2(ox, oy),
                    icon_galley,
                    visuals.fg_stroke.color,
                );
            }

            // Paint caption
            let oy = toast.height / 2. - caption_height / 2.;
            let o_from_icon = if action_width == 0. {
                0.
            } else {
                action_width + icon_x_padding.1
            };
            let o_from_cross = if cross_width == 0. {
                0.
            } else {
                cross_width + cross_x_padding.0
            };
            let ox = (toast.width / 2. - caption_width / 2.) + o_from_icon / 2.
                - o_from_cross / 2.
                - button_width / 2.;
            p.galley(
                rect.min + vec2(ox, oy),
                caption_galley,
                visuals.fg_stroke.color,
            );

            // Take the presses on the toast, and paint and answer its action
            let area = egui::Area::new(Id::unique(("egui-notify-toast", toast.id.0)))
                .order(Order::Foreground)
                .movable(false)
                .constrain(false)
                .current_pos(rect.min)
                .show(ctx, |ui| {
                    let (_, _) = ui.allocate_exact_size(rect.size(), Sense::click());
                    let galley = action_galley.clone()?;
                    let right = toast.width - padding.x - o_from_cross;
                    let button = Rect::from_min_max(
                        rect.min + vec2(right - button_width, 0.),
                        rect.min + vec2(right, toast.height),
                    );
                    let response = ui.interact(
                        button,
                        Id::unique(("egui-notify-action", toast.id.0)),
                        Sense::click(),
                    );
                    let words = galley.text().to_owned();
                    response.widget_info(move || {
                        egui::WidgetInfo::labeled(egui::Role::Button, true, &words)
                    });
                    Some((button, galley, response))
                });
            if let Some((button, galley, response)) = area.inner {
                let colour = if response.hovered() {
                    ctx.global_style()
                        .visuals
                        .hyperlink_color
                        .gamma_multiply(1.3)
                } else {
                    ctx.global_style().visuals.hyperlink_color
                };
                let at = button.center() - galley.rect.size() / 2.;
                p.galley(at, galley, colour);
                if response.clicked() && !toast.state.disappearing() {
                    acted.push(toast.id);
                    toast.dismiss();
                }
            }

            // Paint cross
            if let Some(cross_galley) = cross_galley {
                let cross_rect = cross_galley.rect;
                let oy = toast.height / 2. - cross_height / 2.;
                let ox = toast.width - cross_width - cross_x_padding.1 - padding.x;
                let cross_pos = rect.min + vec2(ox, oy);
                p.galley(cross_pos, cross_galley, Color32::BLACK);

                let screen_cross = Rect {
                    max: cross_pos + cross_rect.max.to_vec2(),
                    min: cross_pos,
                };

                if let Some(pos) = ctx.input(|i| i.pointer.press_origin()) {
                    if screen_cross.contains(pos) && !*held {
                        toast.dismiss();
                        *held = true;
                    }
                }
            }

            // Draw duration
            if toast.show_progress_bar {
                if let Some((initial, current)) = toast.duration {
                    if !toast.state.disappearing() {
                        p.line_segment(
                            [
                                rect.min + vec2(0., toast.height),
                                rect.max - vec2((1. - (current / initial)) * toast.width, 0.),
                            ],
                            Stroke::new(4., visuals.fg_stroke.color),
                        );
                    }
                }
            }

            toast.adjust_next_pos(&mut pos, *anchor, *spacing);

            // Animations
            if toast.state.appearing() {
                update = true;
                toast.value += ctx.input(|i| i.stable_dt) * (*speed);

                if toast.value >= 1. {
                    toast.value = 1.;
                    toast.state = ToastState::Idle;
                }
            } else if toast.state.disappearing() {
                update = true;
                toast.value -= ctx.input(|i| i.stable_dt) * (*speed);

                if toast.value <= 0. {
                    toast.state = ToastState::Disappeared;
                }
            }

            // Remove disappeared toasts
            !toast.state.disappeared()
        });

        if update {
            ctx.request_repaint();
        }
        acted
    }
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

fn ease_in_cubic(x: f32) -> f32 {
    1. - (1. - x).powi(3)
}
