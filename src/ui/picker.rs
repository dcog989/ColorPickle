use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::overlay::{self, PickOutcome};

const CAPTURE_POLL_MILLIS: u64 = 50;

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
}

impl PickerController {
    pub fn new() -> Self {
        Self {
            session: None,
            capture: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.session.is_some() || self.capture.is_some()
    }

    pub fn request(&mut self) -> bool {
        if self.is_busy() {
            return false;
        }
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(overlay::CapturedFrame::capture());
        });
        self.capture = Some(receiver);
        true
    }

    pub fn update(&mut self, ctx: &egui::Context, config: &Config) -> Option<Event> {
        if let Some(receiver) = self.capture.take() {
            match receiver.try_recv() {
                Ok(Ok(captured)) => {
                    self.session = Some(overlay::Session::new(captured));
                    return Some(Event::Ready);
                }
                Ok(Err(error)) => return Some(Event::CaptureFailed(error.to_string())),
                Err(TryRecvError::Empty) => {
                    self.capture = Some(receiver);
                    ctx.request_repaint_after(Duration::from_millis(CAPTURE_POLL_MILLIS));
                }
                Err(TryRecvError::Disconnected) => return Some(Event::ThreadStopped),
            }
        }

        if let Some(mut session) = self.session.take() {
            match overlay::show(ctx, &mut session, config) {
                Some(outcome) => {
                    return Some(match outcome {
                        PickOutcome::Picked(color) => Event::Picked(color),
                        PickOutcome::Dismissed => Event::Dismissed,
                    });
                }
                None => self.session = Some(session),
            }
        }

        None
    }
}
