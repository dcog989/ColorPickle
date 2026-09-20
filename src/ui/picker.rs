use std::sync::mpsc::{self, Receiver, TryRecvError};

use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::overlay::{self, PickOutcome};

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

    pub fn request(&mut self, ctx: &egui::Context) -> bool {
        if self.is_busy() {
            return false;
        }
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = sender.send(overlay::CapturedFrame::capture());
            ctx.request_repaint();
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
                Err(TryRecvError::Empty) => self.capture = Some(receiver),
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
