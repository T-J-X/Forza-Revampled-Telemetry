//! Frame-time and latency statistics with no allocation after startup.
use std::time::Duration;

/// How many recent samples each series keeps: about 2 minutes at 144 fps, so a 60 s parked bench
/// at a high refresh counts every frame (64 KiB per series, allocated once).
pub const SAMPLE_CAPACITY: usize = 16384;

/// A ring of the most recent samples, in milliseconds.
#[derive(Debug, Clone)]
pub struct SampleRing {
    samples: Box<[f32]>,
    len: usize,
    next: usize,
    total: u64,
}

impl Default for SampleRing {
    fn default() -> Self {
        Self::new()
    }
}

/// Percentiles of one series.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Percentiles {
    /// Samples currently held.
    pub count: usize,
    /// Samples ever recorded.
    pub total: u64,
    /// Median, ms.
    pub p50: f32,
    /// 99th percentile, ms.
    pub p99: f32,
    /// Maximum, ms.
    pub max: f32,
}

impl SampleRing {
    /// An empty ring of [`SAMPLE_CAPACITY`] (allocated once here).
    pub fn new() -> Self {
        Self {
            samples: vec![0.0; SAMPLE_CAPACITY].into_boxed_slice(),
            len: 0,
            next: 0,
            total: 0,
        }
    }

    /// Records one sample.
    pub fn record(&mut self, value: Duration) {
        self.samples[self.next] = value.as_secs_f32() * 1000.0;
        self.next = (self.next + 1) % self.samples.len();
        self.len = (self.len + 1).min(self.samples.len());
        self.total += 1;
    }

    /// Forgets every sample.
    pub fn clear(&mut self) {
        self.len = 0;
        self.next = 0;
        self.total = 0;
    }

    /// Percentiles over the held samples. `scratch` must have [`SAMPLE_CAPACITY`] capacity
    /// so no allocation happens.
    pub fn percentiles(&self, scratch: &mut Vec<f32>) -> Percentiles {
        if self.len == 0 {
            return Percentiles::default();
        }
        scratch.clear();
        scratch.extend_from_slice(&self.samples[..self.len]);
        let rank = |p: f32| (((self.len - 1) as f32) * p).round() as usize;
        let pick = |s: &mut Vec<f32>, k: usize| *s.select_nth_unstable_by(k, f32::total_cmp).1;
        let p50 = pick(scratch, rank(0.50));
        let p99 = pick(scratch, rank(0.99));
        let max = pick(scratch, self.len - 1);
        Percentiles {
            count: self.len,
            total: self.total,
            p50,
            p99,
            max,
        }
    }
}

/// Every series the render loop measures.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Packet receive to the return of `present`, for frames that showed a new packet.
    pub receive_to_present: SampleRing,
    /// Packet receive to the first vblank after `present` (estimated on-screen time).
    pub receive_to_vblank: SampleRing,
    /// CPU time per presented frame: the whole iteration from the wake, less the latency wait.
    pub cpu_frame: SampleRing,
    /// The same, for frames the OLED idle dim turned down (a parked car).
    pub cpu_frame_dimmed: SampleRing,
    /// Time blocked on the swapchain's frame-latency wait.
    pub acquire_wait: SampleRing,
    /// Building the frame: config, layout, signals and the cluster's `update`.
    pub build: SampleRing,
    /// Drawing the frame into the kit's arena: scrim, wash, the cluster's `draw`, the dim and
    /// the OSD.
    pub draw: SampleRing,
    /// Uploading, recording and submitting.
    pub encode_submit: SampleRing,
    /// Of `encode_submit`: the swapchain view, the encoder, the kit's render and `finish`.
    pub record: SampleRing,
    /// Of `record`: the kit's own render (pack, the upload copy, pass recording).
    pub kit: SampleRing,
    /// The loop's checks between the wake and the build.
    pub before_build: SampleRing,
    /// After `present` returned: the kit's staging hand-back and the failure checks.
    pub after_present: SampleRing,
    /// Of `encode_submit`: `Queue::submit`.
    pub submit: SampleRing,
    /// The `present` call.
    pub present: SampleRing,
    /// Render passes the kit recorded, summed over the presented frames.
    pub passes: u64,
    /// Draw calls the kit recorded, summed over the presented frames.
    pub draw_calls: u64,
    /// New packets presented before the first vblank after they arrived.
    pub vsync_hits: u64,
    /// New packets checked for a vsync hit (DWM timing available).
    pub vsync_checked: u64,
}

impl FrameStats {
    /// Clears every series (after warm-up).
    pub fn clear(&mut self) {
        self.receive_to_present.clear();
        self.receive_to_vblank.clear();
        self.cpu_frame.clear();
        self.cpu_frame_dimmed.clear();
        self.acquire_wait.clear();
        self.build.clear();
        self.draw.clear();
        self.encode_submit.clear();
        self.record.clear();
        self.kit.clear();
        self.before_build.clear();
        self.after_present.clear();
        self.submit.clear();
        self.present.clear();
        self.passes = 0;
        self.draw_calls = 0;
        self.vsync_hits = 0;
        self.vsync_checked = 0;
    }
}

/// True when a packet received `to_present` before `present` returned is shown at the first
/// vblank after it arrived: it arrived after the last vblank (which was `period - until_next`
/// before present).
pub fn caught_next_vsync(to_present: Duration, until_next: Duration, period: Duration) -> bool {
    to_present < period.saturating_sub(until_next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_of_a_ramp() {
        let mut ring = SampleRing::new();
        for ms in 1..=100u64 {
            ring.record(Duration::from_millis(ms));
        }
        let mut scratch = Vec::with_capacity(SAMPLE_CAPACITY);
        let p = ring.percentiles(&mut scratch);
        assert_eq!(p.count, 100);
        assert!((p.p50 - 51.0).abs() < 0.01, "{p:?}");
        assert!((p.p99 - 99.0).abs() < 0.01, "{p:?}");
        assert!((p.max - 100.0).abs() < 0.01, "{p:?}");
    }

    #[test]
    fn ring_keeps_only_recent_samples_without_growing() {
        let mut ring = SampleRing::new();
        let mut scratch = Vec::with_capacity(SAMPLE_CAPACITY);
        let cap = scratch.capacity();
        for _ in 0..SAMPLE_CAPACITY {
            ring.record(Duration::from_millis(100));
        }
        for _ in 0..SAMPLE_CAPACITY {
            ring.record(Duration::from_millis(1));
        }
        let p = ring.percentiles(&mut scratch);
        assert_eq!(p.count, SAMPLE_CAPACITY);
        assert_eq!(p.total, 2 * SAMPLE_CAPACITY as u64);
        assert!((p.max - 1.0).abs() < 0.01, "old samples were overwritten");
        assert_eq!(scratch.capacity(), cap, "scratch never reallocates");
    }

    #[test]
    fn vsync_hit_rule() {
        let period = Duration::from_micros(8_333);
        // Present 2 ms before the next vblank: the last vblank was 6.3 ms ago.
        let until_next = Duration::from_millis(2);
        assert!(caught_next_vsync(Duration::from_millis(1), until_next, period));
        assert!(!caught_next_vsync(Duration::from_millis(7), until_next, period));
    }

    #[test]
    fn empty_ring_is_zero() {
        let mut scratch = Vec::new();
        assert_eq!(SampleRing::new().percentiles(&mut scratch), Percentiles::default());
    }
}
