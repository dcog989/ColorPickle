use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::ui::overlay::{self, PickOutcome};

const CAPTURE_TIMEOUT_SECONDS: f64 = 5.0;
const CAPTURE_WATCHDOG_MILLIS: u64 = 250;
const HIDE_SETTLE_MILLIS: u64 = 150;
const PICKER_VIEWPORT_SALT: &str = "colorpickle-picker";

pub enum Event {
    Ready,
    CaptureFailed(String),
    ThreadStopped,
    Picked(Okhsl),
    Dismissed,
}

enum CaptureUpdate {
    WaitingForUser,
    Finished(overlay::CaptureOutcome),
}

pub struct PickerController {
    session: Option<overlay::Session>,
    capture: Option<Receiver<CaptureUpdate>>,
    capture_started: f64,
    waiting_for_user: bool,
    generation: u64,
    viewport: egui::ViewportId,
}

impl PickerController {
    pub fn new() -> Self {
        Self {
            session: None,
            capture: None,
            capture_started: 0.0,
            waiting_for_user: false,
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
        self.waiting_for_user = false;
        tracing::info!(generation = self.generation, "picker: capture requested");

        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));

        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            tracing::debug!("picker: capture worker starting");
            // Give the compositor time to unmap the main window so it is not captured.
            std::thread::sleep(Duration::from_millis(HIDE_SETTLE_MILLIS));
            let result = overlay::CapturedFrame::capture_with(|| {
                let _ = sender.send(CaptureUpdate::WaitingForUser);
                ctx.request_repaint();
            });
            tracing::debug!(ok = result.is_ok(), "picker: capture worker finished");
            let _ = sender.send(CaptureUpdate::Finished(result));
            ctx.request_repaint();
        });
        self.capture = Some(receiver);
        true
    }

    pub fn update(&mut self, ctx: &egui::Context) -> Option<Event> {
        if let Some(receiver) = self.capture.take() {
            match receiver.try_recv() {
                Ok(CaptureUpdate::Finished(Ok(captured))) => {
                    tracing::info!("picker: frame ready, showing overlay");
                    self.session = Some(overlay::Session::new(captured));
                    // Run another frame immediately so the overlay is shown.
                    ctx.request_repaint();
                    return Some(Event::Ready);
                }
                Ok(CaptureUpdate::Finished(Err(error))) => {
                    tracing::warn!(%error, "picker: capture failed");
                    show_main_window(ctx);
                    return Some(Event::CaptureFailed(error.to_string()));
                }
                Ok(CaptureUpdate::WaitingForUser) => {
                    tracing::info!("picker: waiting for the user to answer the permission prompt");
                    self.waiting_for_user = true;
                    self.capture = Some(receiver);
                    ctx.request_repaint_after(Duration::from_millis(CAPTURE_WATCHDOG_MILLIS));
                }
                Err(TryRecvError::Empty) => {
                    let now = ctx.input(|input| input.time);
                    if !self.waiting_for_user && now - self.capture_started > CAPTURE_TIMEOUT_SECONDS {
                        tracing::warn!(
                            elapsed = now - self.capture_started,
                            "picker: capture timed out"
                        );
                        show_main_window(ctx);
                        return Some(Event::CaptureFailed("timed out".to_owned()));
                    }
                    self.capture = Some(receiver);
                    ctx.request_repaint_after(Duration::from_millis(CAPTURE_WATCHDOG_MILLIS));
                }
                Err(TryRecvError::Disconnected) => {
                    tracing::warn!("picker: capture thread disconnected");
                    show_main_window(ctx);
                    return Some(Event::ThreadStopped);
                }
            }
        }

        if let Some(mut session) = self.session.take() {
            match overlay::show(ctx, &mut session, self.viewport) {
                Some(outcome) => {
                    tracing::info!(?outcome, "picker: overlay closed");
                    show_main_window(ctx);
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

fn show_main_window(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
}
