use std::time::Duration;

use eframe::egui;

const TOAST_DURATION: f64 = 2.5;
const TOAST_BOTTOM_MARGIN: f32 = 84.0;
const TOAST_MARGIN_X: i8 = 16;
const TOAST_MARGIN_Y: i8 = 10;
const TOAST_ID: &str = "colorpickle-toast";

pub(super) struct Toast {
    message: String,
    expires_at: f64,
}

impl Toast {
    pub(super) fn new(message: impl Into<String>, now: f64) -> Self {
        Self {
            message: message.into(),
            expires_at: now + TOAST_DURATION,
        }
    }

    pub(super) fn is_expired(&self, now: f64) -> bool {
        now >= self.expires_at
    }

    pub(super) fn remaining(&self, now: f64) -> Duration {
        Duration::from_secs_f64((self.expires_at - now).max(0.0))
    }

    pub(super) fn show(&self, ctx: &egui::Context) {
        egui::Area::new(egui::Id::new(TOAST_ID))
            .anchor(
                egui::Align2::CENTER_BOTTOM,
                egui::vec2(0.0, -TOAST_BOTTOM_MARGIN),
            )
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Margin::symmetric(TOAST_MARGIN_X, TOAST_MARGIN_Y))
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(self.message.clone()).extend());
                    });
            });
    }
}
