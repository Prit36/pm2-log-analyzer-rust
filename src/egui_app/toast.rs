use super::theme::{TailwindColors, font_medium};
use egui::{Align2, Color32, CornerRadius, Frame, Margin, Stroke, Vec2};
use std::time::{Duration, Instant};

pub struct ToastManager {
    message: Option<String>,
    expires_at: Option<Instant>,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self {
            message: None,
            expires_at: None,
        }
    }
}

impl ToastManager {
    pub fn show(&mut self, text: impl Into<String>) {
        self.message = Some(text.into());
        self.expires_at = Some(Instant::now() + Duration::from_millis(3200));
    }

    pub fn render(&mut self, ctx: &egui::Context, dark: bool) {
        if let Some(expires) = self.expires_at {
            let now = Instant::now();
            if now >= expires {
                self.message = None;
                self.expires_at = None;
                return;
            }

            // Ensure egui repaints when the toast is scheduled to expire
            ctx.request_repaint_after(expires - now);

            if let Some(msg) = &self.message {
                egui::Area::new(egui::Id::new("app_toast"))
                    .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-16.0, -16.0))
                    .order(egui::Order::Tooltip)
                    .show(ctx, |ui| {
                        let bg = if dark {
                            TailwindColors::SLATE_800
                        } else {
                            TailwindColors::SLATE_900
                        };
                        let border = if dark {
                            TailwindColors::SLATE_700
                        } else {
                            TailwindColors::SLATE_200
                        };
                        let text_color = if dark {
                            TailwindColors::SLATE_100
                        } else {
                            Color32::WHITE
                        };

                        Frame::NONE
                            .fill(bg)
                            .stroke(Stroke::new(1.0, border))
                            .corner_radius(CornerRadius::same(8))
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 4],
                                blur: 16,
                                spread: 0,
                                color: Color32::from_black_alpha(if dark { 120 } else { 70 }),
                            })
                            .inner_margin(Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(msg)
                                        .font(font_medium(12.0))
                                        .line_height(Some(super::theme::LH_XS))
                                        .color(text_color),
                                );
                            });
                    });
            }
        }
    }
}
