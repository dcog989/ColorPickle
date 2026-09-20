use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::overlay::{self, PickOutcome};

const CAPTURE_TIMEOUT_SECONDS: f64 = 5.0;
const CAPTURE_WATCHDOG_MILLIS: u64 = 250;
const PICKER_VIEWPORT_SALT: &str = "colorpickle-picker";

pub enum Event {
    Ready,
    CaptureFailed(String),
    ThreadStopped,
    Picked(Okhsl),
    Dismissed,
}

pub struct PickerController {
    session: Option<overlay::Session>,
    capture: Option<Receiver<overlay::CaptureOutcome>>,
    capture_started: f64,
    generation: u64,
    viewport: egui::ViewportId,
}

impl PickerController {
    pub fn new() -> Self {
        Self {
            session: None,
            capture: None,
            capture_started: 0.0,
            generation: 0,
            viewport: egui::ViewportId::from_hash_of((PICKER_VIEWPORT_SALT, 0_u64)),
        }
    }

    pub fn is_busy(&self) -> bool {
        self.session.is_some() || self.capture.is_some()
    }

    pub fn request(&mut self, ctx: &egui::Context) -> bool {
        if self.is_busy() {
            return false;
        }
        self.generation = self.generation.wrapping_add(1);
        self.viewport = egui::ViewportId::from_hash_of((PICKER_VIEWPORT_SALT, self.generation));
        self.capture_started = ctx.input(|input| input.time);
        tracing::info!(generation = self.generation, "picker: capture requested");

        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            tracing::debug!("picker: capture worker starting");
            let result = overlay::CapturedFrame::capture();
            tracing::debug!(ok = result.is_ok(), "picker: capture worker finished");
            let _ = sender.send(result);
            ctx.request_repaint();
        });
        self.capture = Some(receiver);
        true
    }

    pub fn update(&mut self, ctx: &egui::Context, config: &Config) -> Option<Event> {
        if let Some(receiver) = self.capture.take() {
            match receiver.try_recv() {
                Ok(Ok(captured)) => {
                    tracing::info!("picker: frame ready, showing overlay");
                    self.session = Some(overlay::Session::new(captured));
                    // Run another frame immediately so the overlay is shown.
                    ctx.request_repaint();
                    return Some(Event::Ready);
                }
                Ok(Err(error)) => {
                    tracing::warn!(%error, "picker: capture failed");
                    return Some(Event::CaptureFailed(error.to_string()));
                }
                Err(TryRecvError::Empty) => {
                    let now = ctx.input(|input| input.time);
                    if now - self.capture_started > CAPTURE_TIMEOUT_SECONDS {
                        tracing::warn!(elapsed = now - self.capture_started, "picker: capture timed out");
                        return Some(Event::CaptureFailed("timed out".to_owned()));
                    }
                    self.capture = Some(receiver);
                    ctx.request_repaint_after(Duration::from_millis(CAPTURE_WATCHDOG_MILLIS));
                }
                Err(TryRecvError::Disconnected) => {
                    tracing::warn!("picker: capture thread disconnected");
                    return Some(Event::ThreadStopped);
                }
            }
        }

        if let Some(mut session) = self.session.take() {
            match overlay::show(ctx, &mut session, config, self.viewport) {
                Some(outcome) => {
                    tracing::info!(?outcome, "picker: overlay closed");
                    return Some(match outcome {
                        PickOutcome::Picked(color) => Event::Picked(color),
                        PickOutcome::Dismissed => Event::Dismissed,
                    });
                }
                None => {
                    self.session = Some(session);
                    ctx.request_repaint();
                }
            }
        }

        None
    }
}
