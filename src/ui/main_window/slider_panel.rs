use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::ui::slider;
use crate::ui::theme::color32;

const SLIDER_WIDTH: f32 = 30.0;
const SLIDER_PANEL_MARGIN: f32 = 12.0;
const SLIDER_PANEL_ID: &str = "colorpickle-sliders";
const HUE_MAX_DEGREES: f32 = 360.0;
const HUE_FRACTION_MAX: f32 = 1.0 - f32::EPSILON;

pub(super) fn show(
    ui: &mut egui::Ui,
    color: &mut Okhsl,
    background: egui::Color32,
    foreground: egui::Color32,
) -> f32 {
    let mut hue = color.hue() / HUE_MAX_DEGREES;
    let mut saturation = color.saturation();
    let mut lightness = color.lightness();

    let panel_width =
        3.0 * SLIDER_WIDTH + 2.0 * ui.spacing().item_spacing.x + 2.0 * SLIDER_PANEL_MARGIN;
    egui::Panel::right(SLIDER_PANEL_ID)
        .resizable(false)
        .exact_size(panel_width)
        .frame(
            egui::Frame::NONE
                .fill(background)
                .inner_margin(SLIDER_PANEL_MARGIN),
        )
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let hue_gradient = channel_gradient(SliderChannel::Hue, hue, saturation, lightness);
                slider::column(ui, "H", foreground, &mut hue, SLIDER_WIDTH, hue_gradient);

                let saturation_gradient =
                    channel_gradient(SliderChannel::Saturation, hue, saturation, lightness);
                slider::column(
                    ui,
                    "S",
                    foreground,
                    &mut saturation,
                    SLIDER_WIDTH,
                    saturation_gradient,
                );

                let lightness_gradient =
                    channel_gradient(SliderChannel::Lightness, hue, saturation, lightness);
                slider::column(
                    ui,
                    "L",
                    foreground,
                    &mut lightness,
                    SLIDER_WIDTH,
                    lightness_gradient,
                );
            });
        });

    *color = Okhsl::new(
        hue.min(HUE_FRACTION_MAX) * HUE_MAX_DEGREES,
        saturation,
        lightness,
    );

    panel_width
}

#[derive(Clone, Copy)]
enum SliderChannel {
    Hue,
    Saturation,
    Lightness,
}

fn channel_gradient(
    channel: SliderChannel,
    hue: f32,
    saturation: f32,
    lightness: f32,
) -> impl Fn(f32) -> egui::Color32 {
    move |value| {
        let color = match channel {
            SliderChannel::Hue => Okhsl::new(value * HUE_MAX_DEGREES, saturation, lightness),
            SliderChannel::Saturation => Okhsl::new(hue * HUE_MAX_DEGREES, value, lightness),
            SliderChannel::Lightness => Okhsl::new(hue * HUE_MAX_DEGREES, saturation, value),
        };
        color32(color)
    }
}
