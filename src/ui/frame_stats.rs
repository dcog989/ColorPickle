//! Frame pacing for the picker overlay.
//!
//! A stutter is either our per-frame work or the event/present pipeline, and the
//! two look the same on screen. This separates them: `draw_ms` is the time our
//! UI code takes, `gap_ms` is what the user waits between frames, and
//! `motion_per_frame` says whether pointer motion arrived at all. A long gap with
//! a short draw means egui, wgpu or the compositor dropped the frame, not us.

use std::time::{Duration, Instant};

use eframe::egui;

/// A frame slower than this reads as a hitch at any refresh rate.
const STALL_MILLIS: f64 = 20.0;
const REPORT_INTERVAL_SECONDS: f64 = 1.0;
const MILLIS_PER_SECOND: f64 = 1000.0;

#[derive(Debug, Default)]
pub struct FrameStats {
    last_frame: Option<Instant>,
    last_pointer: Option<egui::Pos2>,
    report_start: Option<Instant>,
    frames: u32,
    motion_events: u32,
    stalls: u32,
    gap_total_millis: f64,
    max_gap_millis: f64,
    draw_total_millis: f64,
    max_draw_millis: f64,
}

impl FrameStats {
    pub fn record(&mut self, draw: Duration, motion_events: usize, pointer: Option<egui::Pos2>) {
        self.record_at(Instant::now(), draw, motion_events, pointer);
    }

    fn record_at(
        &mut self,
        now: Instant,
        draw: Duration,
        motion_events: usize,
        pointer: Option<egui::Pos2>,
    ) {
        let draw_millis = draw.as_secs_f64() * MILLIS_PER_SECOND;
        // A repeated pointer on a slow frame means input was stale, not that our
        // frame was late.
        let same_pointer = pointer.is_some() && pointer == self.last_pointer;
        self.last_pointer = pointer;
        self.frames += 1;
        self.motion_events += motion_events as u32;
        self.draw_total_millis += draw_millis;
        self.max_draw_millis = self.max_draw_millis.max(draw_millis);

        let gap = self
            .last_frame
            .replace(now)
            .map(|last| (now - last).as_secs_f64() * MILLIS_PER_SECOND);
        if let Some(gap) = gap {
            self.gap_total_millis += gap;
            self.max_gap_millis = self.max_gap_millis.max(gap);
            if gap >= STALL_MILLIS {
                self.stalls += 1;
                tracing::debug!(
                    gap_ms = gap,
                    draw_ms = draw_millis,
                    motion_events,
                    same_pointer,
                    ?pointer,
                    "overlay: slow frame"
                );
            }
        }

        let start = *self.report_start.get_or_insert(now);
        let span = (now - start).as_secs_f64();
        if span < REPORT_INTERVAL_SECONDS {
            return;
        }
        let frames = f64::from(self.frames);
        tracing::debug!(
            frames = self.frames,
            fps = frames / span,
            motion_per_frame = f64::from(self.motion_events) / frames,
            avg_gap_ms = self.gap_total_millis / (frames - 1.0).max(1.0),
            max_gap_ms = self.max_gap_millis,
            avg_draw_ms = self.draw_total_millis / frames,
            max_draw_ms = self.max_draw_millis,
            stalls = self.stalls,
            "overlay: frame stats"
        );
        let last_frame = self.last_frame;
        *self = Self {
            last_frame,
            report_start: Some(now),
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::FrameStats;
    use super::STALL_MILLIS;
    use std::time::{Duration, Instant};

    #[test]
    fn counts_a_gap_as_a_stall_only_past_the_threshold() {
        let mut stats = FrameStats::default();
        let start = Instant::now();
        let slow = Duration::from_millis((STALL_MILLIS as u64) + 10);
        let fast = Duration::from_millis(5);

        stats.record_at(start, Duration::ZERO, 1, None);
        stats.record_at(start + slow, Duration::ZERO, 1, None);
        stats.record_at(start + slow + fast, Duration::ZERO, 1, None);

        assert_eq!(stats.frames, 3);
        assert_eq!(stats.stalls, 1);
        assert_eq!(stats.motion_events, 3);
    }
}
