use crate::vad::{VadEvent, VadEventKind};

const SAMPLE_RATE: usize = 16_000;
const FAST_PREVIEW_STEP_SAMPLES: usize = SAMPLE_RATE / 4;
const BALANCED_PREVIEW_STEP_SAMPLES: usize = SAMPLE_RATE / 2;
const THROTTLED_PREVIEW_STEP_SAMPLES: usize = SAMPLE_RATE;
const STRESSED_DECODE_MS: u128 = 350;
const SEVERE_DECODE_MS: u128 = 900;
const HEALTHY_DECODE_MS: u128 = 150;
const STRESSED_QUEUE_PERCENT: u64 = 35;
const SEVERE_QUEUE_PERCENT: u64 = 75;
const HEALTHY_QUEUE_PERCENT: u64 = 10;
const HEALTHY_OBSERVATIONS_TO_RECOVER: u8 = 3;
const LATENCY_BUCKET_UPPER_BOUNDS_MS: [u64; 15] = [
    25, 50, 100, 150, 250, 350, 500, 750, 1_000, 1_500, 2_500, 5_000, 10_000, 30_000, 60_000,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeReason {
    Preview,
    Endpoint,
    ForcedLimit,
    Final,
}

#[derive(Debug, PartialEq)]
pub struct DecodeRequest {
    pub samples: Vec<f32>,
    pub reason: DecodeReason,
    pub requires_transcript_overlap: bool,
}

#[derive(Debug, Clone)]
pub struct DecodeLatencyHistogram {
    buckets: [u64; LATENCY_BUCKET_UPPER_BOUNDS_MS.len() + 1],
    count: u64,
    max_ms: u64,
}

impl Default for DecodeLatencyHistogram {
    fn default() -> Self {
        Self {
            buckets: [0; LATENCY_BUCKET_UPPER_BOUNDS_MS.len() + 1],
            count: 0,
            max_ms: 0,
        }
    }
}

impl DecodeLatencyHistogram {
    pub fn record(&mut self, elapsed_ms: u128) {
        let elapsed_ms = elapsed_ms.min(u128::from(u64::MAX)) as u64;
        let bucket_index = LATENCY_BUCKET_UPPER_BOUNDS_MS
            .iter()
            .position(|upper_bound| elapsed_ms <= *upper_bound)
            .unwrap_or(LATENCY_BUCKET_UPPER_BOUNDS_MS.len());
        self.buckets[bucket_index] = self.buckets[bucket_index].saturating_add(1);
        self.count = self.count.saturating_add(1);
        self.max_ms = self.max_ms.max(elapsed_ms);
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn percentile(&self, percentile: u8) -> Option<u64> {
        if self.count == 0 || !(1..=100).contains(&percentile) {
            return None;
        }
        let rank = (u128::from(self.count) * u128::from(percentile)).div_ceil(100);
        let mut cumulative = 0u128;
        for (index, bucket_count) in self.buckets.iter().enumerate() {
            cumulative = cumulative.saturating_add(u128::from(*bucket_count));
            if cumulative >= rank {
                let bucket_upper = LATENCY_BUCKET_UPPER_BOUNDS_MS
                    .get(index)
                    .copied()
                    .unwrap_or(self.max_ms);
                return Some(bucket_upper.min(self.max_ms));
            }
        }
        Some(self.max_ms)
    }

    pub fn max_ms(&self) -> Option<u64> {
        (self.count > 0).then_some(self.max_ms)
    }
}

#[derive(Debug, Default)]
pub struct StreamingMetrics {
    pub decode_requests: u64,
    pub total_decode_ms: u128,
    pub decode_latency: DecodeLatencyHistogram,
    pub socket_connects: u64,
    pub server_recoveries: u64,
    pub empty_endpoint_decodes: u64,
    pub recovered_endpoint_previews: u64,
    pub final_preview_reused: bool,
    pub max_active_samples: usize,
    pub max_queued_chunks: u64,
    pub max_preview_step_samples: usize,
}

#[derive(Debug)]
pub struct StreamingOutcome {
    pub generation: u64,
    pub transcript: String,
    pub dropped_chunks: u64,
    pub error: Option<String>,
    pub metrics: StreamingMetrics,
}

impl StreamingOutcome {
    pub fn reusable_transcript(&self, expected_generation: u64) -> Result<String, String> {
        if self.generation != expected_generation {
            return Err("streaming result belongs to a stale recording session".to_string());
        }
        if let Some(error) = &self.error {
            return Err(format!("streaming worker degraded: {error}"));
        }
        if self.dropped_chunks > 0 {
            return Err(format!(
                "streaming audio queue dropped {} chunks",
                self.dropped_chunks
            ));
        }
        if self.metrics.empty_endpoint_decodes > 0 {
            return Err(format!(
                "streaming recovered {} empty endpoint decodes; authoritative WAV verification is required",
                self.metrics.empty_endpoint_decodes
            ));
        }
        let transcript = self.transcript.trim();
        if transcript.is_empty() {
            return Err("streaming result is empty".to_string());
        }
        Ok(transcript.to_string())
    }
}

/// Adapts repeated offline preview work to the decoder's measured throughput.
///
/// It reacts immediately to overload, but requires several healthy decodes to
/// lower the cadence. Endpoint and final requests bypass this controller.
#[derive(Debug, Default)]
pub struct PreviewCadenceController {
    level: u8,
    healthy_observations: u8,
}

impl PreviewCadenceController {
    pub fn preview_step_samples(&self) -> usize {
        match self.level {
            0 => FAST_PREVIEW_STEP_SAMPLES,
            1 => BALANCED_PREVIEW_STEP_SAMPLES,
            _ => THROTTLED_PREVIEW_STEP_SAMPLES,
        }
    }

    pub fn observe_decode(&mut self, decode_ms: u128, queued_chunks: u64, queue_capacity: usize) {
        let queue_percent = if queue_capacity == 0 {
            0
        } else {
            queued_chunks
                .saturating_mul(100)
                .saturating_div(queue_capacity as u64)
        };

        if decode_ms >= SEVERE_DECODE_MS || queue_percent >= SEVERE_QUEUE_PERCENT {
            self.level = 2;
            self.healthy_observations = 0;
        } else if decode_ms >= STRESSED_DECODE_MS || queue_percent >= STRESSED_QUEUE_PERCENT {
            self.level = self.level.max(1);
            self.healthy_observations = 0;
        } else if decode_ms <= HEALTHY_DECODE_MS && queue_percent <= HEALTHY_QUEUE_PERCENT {
            self.healthy_observations = self.healthy_observations.saturating_add(1);
            if self.level > 0 && self.healthy_observations >= HEALTHY_OBSERVATIONS_TO_RECOVER {
                self.level -= 1;
                self.healthy_observations = 0;
            }
        } else {
            self.healthy_observations = 0;
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SegmentLimits {
    pre_roll_samples: usize,
    min_preview_samples: usize,
    long_preview_after_samples: usize,
    long_preview_step_samples: usize,
    max_active_samples: usize,
    forced_overlap_samples: usize,
    max_preview_reuse_tail_samples: usize,
}

impl Default for SegmentLimits {
    fn default() -> Self {
        Self {
            pre_roll_samples: SAMPLE_RATE / 5,
            min_preview_samples: SAMPLE_RATE / 2,
            long_preview_after_samples: SAMPLE_RATE * 15,
            long_preview_step_samples: SAMPLE_RATE / 2,
            max_active_samples: SAMPLE_RATE * 30,
            forced_overlap_samples: SAMPLE_RATE / 5,
            max_preview_reuse_tail_samples: SAMPLE_RATE * 3 / 4,
        }
    }
}

/// Keeps only the active VAD utterance in memory for repeated offline decoding.
/// Completed utterances are returned once and never decoded again.
pub struct SegmentAccumulator {
    limits: SegmentLimits,
    active: Vec<f32>,
    speech_started: bool,
    last_preview_len: usize,
    last_successful_preview_len: usize,
    last_successful_preview_speech_revision: Option<u64>,
    speech_revision: u64,
    active_has_overlap: bool,
}

impl SegmentAccumulator {
    pub fn new() -> Self {
        Self::with_limits(SegmentLimits::default())
    }

    fn with_limits(mut limits: SegmentLimits) -> Self {
        limits.max_active_samples = limits.max_active_samples.max(1);
        limits.forced_overlap_samples = limits
            .forced_overlap_samples
            .min(limits.max_active_samples.saturating_sub(1));
        limits.pre_roll_samples = limits.pre_roll_samples.max(1);
        limits.min_preview_samples = limits.min_preview_samples.max(1);
        limits.long_preview_step_samples = limits.long_preview_step_samples.max(1);
        Self {
            limits,
            active: Vec::with_capacity(limits.max_active_samples),
            speech_started: false,
            last_preview_len: 0,
            last_successful_preview_len: 0,
            last_successful_preview_speech_revision: None,
            speech_revision: 0,
            active_has_overlap: false,
        }
    }

    pub fn push(
        &mut self,
        samples: &[f32],
        events: &[VadEvent],
    ) -> Result<Vec<DecodeRequest>, String> {
        validate_events(samples.len(), events)?;

        let mut requests = Vec::new();
        let mut cursor = 0;
        for event in events {
            self.append_samples(&samples[cursor..event.end_offset]);
            if event.kind == VadEventKind::Speech {
                self.speech_started = true;
                self.speech_revision = self.speech_revision.saturating_add(1);
            }
            self.emit_forced_segments(&mut requests);

            if event.kind == VadEventKind::Endpoint {
                if let Some(request) = self.take_current(DecodeReason::Endpoint) {
                    requests.push(request);
                } else {
                    self.clear_active();
                }
            }
            cursor = event.end_offset;
        }

        self.append_samples(&samples[cursor..]);
        self.emit_forced_segments(&mut requests);
        Ok(requests)
    }

    pub fn take_preview_request(&mut self, requested_step_samples: usize) -> Option<DecodeRequest> {
        if !self.speech_started || self.active.len() < self.limits.min_preview_samples {
            return None;
        }

        let step = if self.active.len() >= self.limits.long_preview_after_samples {
            requested_step_samples.max(self.limits.long_preview_step_samples)
        } else {
            requested_step_samples.max(1)
        };
        if self.active.len().saturating_sub(self.last_preview_len) < step {
            return None;
        }

        self.last_preview_len = self.active.len();
        Some(DecodeRequest {
            samples: self.active.clone(),
            reason: DecodeReason::Preview,
            requires_transcript_overlap: false,
        })
    }

    pub fn finish(&mut self) -> Option<DecodeRequest> {
        self.take_current(DecodeReason::Final)
    }

    pub fn mark_preview_successful(&mut self) {
        if self.speech_started
            && self.last_preview_len > 0
            && self.last_preview_len == self.active.len()
        {
            self.last_successful_preview_len = self.last_preview_len;
            self.last_successful_preview_speech_revision = Some(self.speech_revision);
        } else {
            self.invalidate_successful_preview();
        }
    }

    pub fn invalidate_successful_preview(&mut self) {
        self.last_successful_preview_len = 0;
        self.last_successful_preview_speech_revision = None;
    }

    pub fn can_reuse_successful_preview(&self, pending_tail_has_speech: bool) -> bool {
        if pending_tail_has_speech
            || !self.speech_started
            || self.active_has_overlap
            || self.last_successful_preview_len == 0
        {
            return false;
        }
        let Some(preview_revision) = self.last_successful_preview_speech_revision else {
            return false;
        };
        preview_revision == self.speech_revision
            && self
                .active
                .len()
                .saturating_sub(self.last_successful_preview_len)
                <= self.limits.max_preview_reuse_tail_samples
    }

    pub fn discard_active(&mut self) {
        self.clear_active();
    }

    pub fn active_sample_count(&self) -> usize {
        self.active.len()
    }

    fn append_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        self.active.extend_from_slice(samples);
        if !self.speech_started && self.active.len() > self.limits.pre_roll_samples {
            let discard = self.active.len() - self.limits.pre_roll_samples;
            self.active.drain(..discard);
            self.last_preview_len = self.last_preview_len.saturating_sub(discard);
        }
    }

    fn emit_forced_segments(&mut self, requests: &mut Vec<DecodeRequest>) {
        while self.speech_started && self.active.len() >= self.limits.max_active_samples {
            let max = self.limits.max_active_samples;
            let samples = self.active[..max].to_vec();
            let requires_transcript_overlap = self.active_has_overlap;
            let keep_from = max - self.limits.forced_overlap_samples;
            self.active.drain(..keep_from);
            self.speech_started = true;
            self.last_preview_len = self.active.len();
            self.invalidate_successful_preview();
            self.active_has_overlap = true;
            requests.push(DecodeRequest {
                samples,
                reason: DecodeReason::ForcedLimit,
                requires_transcript_overlap,
            });
        }
    }

    fn take_current(&mut self, reason: DecodeReason) -> Option<DecodeRequest> {
        if !self.speech_started || self.active.is_empty() {
            return None;
        }
        let samples = std::mem::take(&mut self.active);
        let requires_transcript_overlap = self.active_has_overlap;
        self.speech_started = false;
        self.last_preview_len = 0;
        self.invalidate_successful_preview();
        self.speech_revision = 0;
        self.active_has_overlap = false;
        Some(DecodeRequest {
            samples,
            reason,
            requires_transcript_overlap,
        })
    }

    fn clear_active(&mut self) {
        self.active.clear();
        self.speech_started = false;
        self.last_preview_len = 0;
        self.invalidate_successful_preview();
        self.speech_revision = 0;
        self.active_has_overlap = false;
    }
}

impl Default for SegmentAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_events(sample_count: usize, events: &[VadEvent]) -> Result<(), String> {
    let mut previous_offset = 0;
    for event in events {
        if event.end_offset < previous_offset || event.end_offset > sample_count {
            return Err(format!(
                "Invalid VAD event offset {} for {sample_count} samples",
                event.end_offset
            ));
        }
        previous_offset = event.end_offset;
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct TranscriptState {
    committed: String,
    preview: String,
}

impl TranscriptState {
    pub fn set_preview(&mut self, text: impl Into<String>) {
        self.preview = text.into().trim().to_string();
    }

    pub fn commit(&mut self, text: &str) -> bool {
        let was_empty = self.committed.trim().is_empty();
        let (merged, overlap_tokens) = merge_transcripts(&self.committed, text);
        self.committed = merged;
        self.preview.clear();
        was_empty || overlap_tokens > 0
    }

    pub fn has_preview(&self) -> bool {
        !self.preview.trim().is_empty()
    }

    pub fn commit_preview(&mut self) -> Option<bool> {
        if !self.has_preview() {
            return None;
        }
        let preview = std::mem::take(&mut self.preview);
        Some(self.commit(&preview))
    }

    pub fn display_text(&self) -> String {
        merge_transcripts(&self.committed, &self.preview).0
    }

    pub fn final_text(&self) -> &str {
        self.committed.trim()
    }
}

fn merge_transcripts(left: &str, right: &str) -> (String, usize) {
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() {
        return (right.to_string(), 0);
    }
    if right.is_empty() {
        return (left.to_string(), 0);
    }

    let left_tokens: Vec<&str> = left.split_whitespace().collect();
    let right_tokens: Vec<&str> = right.split_whitespace().collect();

    // A segment entirely contained in the committed text is a repetition
    // (e.g. the user said the same word twice, or the decoder echoed an
    // earlier phrase): keep the longer string instead of duplicating the
    // contained words ("world" + "hello world" must not become
    // "world hello world").
    if contains_token_sequence(&left_tokens, &right_tokens) {
        return (left.to_string(), right_tokens.len());
    }
    if contains_token_sequence(&right_tokens, &left_tokens) {
        return (right.to_string(), left_tokens.len());
    }

    let max_overlap = left_tokens.len().min(right_tokens.len()).min(16);
    let overlap = (1..=max_overlap).rev().find(|&count| {
        left_tokens[left_tokens.len() - count..]
            .iter()
            .zip(&right_tokens[..count])
            .all(|(left_token, right_token)| {
                let left_normalized = normalize_token(left_token);
                !left_normalized.is_empty() && left_normalized == normalize_token(right_token)
            })
    });

    match overlap {
        Some(count) if count == right_tokens.len() => (left.to_string(), count),
        Some(count) => {
            let right_suffix = right_tokens[count..].join(" ");
            (
                crate::text_normalizer::smooth_conjunction_boundary(left, &right_suffix),
                count,
            )
        }
        None => (
            crate::text_normalizer::smooth_conjunction_boundary(left, right),
            0,
        ),
    }
}

fn contains_token_sequence(haystack: &[&str], needle: &[&str]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|window| {
        window.iter().zip(needle).all(|(a, b)| {
            normalize_token(a).eq(&normalize_token(b)) && !normalize_token(a).is_empty()
        })
    })
}

fn normalize_token(token: &str) -> String {
    token
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_limits() -> SegmentLimits {
        SegmentLimits {
            pre_roll_samples: 4,
            min_preview_samples: 6,
            long_preview_after_samples: 12,
            long_preview_step_samples: 4,
            max_active_samples: 10,
            forced_overlap_samples: 2,
            max_preview_reuse_tail_samples: 3,
        }
    }

    fn event(end_offset: usize, kind: VadEventKind) -> VadEvent {
        VadEvent { end_offset, kind }
    }

    fn accumulator_with_successful_preview() -> SegmentAccumulator {
        let mut limits = test_limits();
        limits.max_active_samples = 20;
        let mut accumulator = SegmentAccumulator::with_limits(limits);
        accumulator
            .push(&[0.0; 6], &[event(6, VadEventKind::Speech)])
            .expect("valid speech");
        accumulator
            .push(&[0.0; 2], &[])
            .expect("valid speech continuation");
        assert!(accumulator.take_preview_request(1).is_some());
        accumulator.mark_preview_successful();
        accumulator
    }

    #[test]
    fn latency_histogram_is_empty_until_a_decode_is_recorded() {
        let histogram = DecodeLatencyHistogram::default();

        assert_eq!(histogram.count(), 0);
        assert_eq!(histogram.percentile(50), None);
        assert_eq!(histogram.percentile(95), None);
        assert_eq!(histogram.max_ms(), None);
    }

    #[test]
    fn latency_histogram_reports_bounded_percentiles_and_exact_max() {
        let mut histogram = DecodeLatencyHistogram::default();
        for elapsed_ms in [1, 25, 26, 100, 101, 60_001] {
            histogram.record(elapsed_ms);
        }

        assert_eq!(histogram.count(), 6);
        assert_eq!(histogram.percentile(0), None);
        assert_eq!(histogram.percentile(50), Some(50));
        assert_eq!(histogram.percentile(95), Some(60_001));
        assert_eq!(histogram.percentile(100), Some(60_001));
        assert_eq!(histogram.max_ms(), Some(60_001));
    }

    #[test]
    fn latency_histogram_percentile_never_exceeds_exact_max() {
        let mut histogram = DecodeLatencyHistogram::default();
        for elapsed_ms in [610, 625, 640, 655, 680] {
            histogram.record(elapsed_ms);
        }

        assert_eq!(histogram.percentile(95), Some(680));
        assert_eq!(histogram.percentile(100), Some(680));
        assert_eq!(histogram.max_ms(), Some(680));
    }

    #[test]
    fn latency_histogram_counters_saturate() {
        let mut buckets = [0; LATENCY_BUCKET_UPPER_BOUNDS_MS.len() + 1];
        buckets[0] = u64::MAX;
        let mut histogram = DecodeLatencyHistogram {
            buckets,
            count: u64::MAX,
            max_ms: 0,
        };

        histogram.record(1);

        assert_eq!(histogram.count(), u64::MAX);
        assert_eq!(histogram.buckets[0], u64::MAX);
        assert_eq!(histogram.max_ms(), Some(1));
    }

    #[test]
    fn silence_is_bounded_to_pre_roll() {
        let mut accumulator = SegmentAccumulator::with_limits(test_limits());

        let requests = accumulator.push(&[0.0; 20], &[]).expect("valid silence");

        assert!(requests.is_empty());
        assert_eq!(accumulator.active_sample_count(), 4);
        assert!(accumulator.finish().is_none());
    }

    #[test]
    fn endpoint_returns_one_segment_and_clears_active_audio() {
        let mut accumulator = SegmentAccumulator::with_limits(test_limits());
        let samples: Vec<f32> = (0..8).map(|value| value as f32).collect();
        let events = [
            event(4, VadEventKind::Speech),
            event(8, VadEventKind::Endpoint),
        ];

        let requests = accumulator.push(&samples, &events).expect("valid events");

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].reason, DecodeReason::Endpoint);
        assert_eq!(requests[0].samples, samples);
        assert_eq!(accumulator.active_sample_count(), 0);
    }

    #[test]
    fn continuous_speech_is_cut_with_overlap_and_stays_bounded() {
        let mut accumulator = SegmentAccumulator::with_limits(test_limits());
        let samples: Vec<f32> = (0..20).map(|value| value as f32).collect();
        let events: Vec<VadEvent> = (2..=20)
            .step_by(2)
            .map(|offset| event(offset, VadEventKind::Speech))
            .collect();

        let requests = accumulator.push(&samples, &events).expect("valid events");

        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].samples,
            (0..10).map(|value| value as f32).collect::<Vec<_>>()
        );
        assert_eq!(
            requests[1].samples,
            (8..18).map(|value| value as f32).collect::<Vec<_>>()
        );
        assert!(!requests[0].requires_transcript_overlap);
        assert!(requests[1].requires_transcript_overlap);
        assert!(accumulator.active_sample_count() <= 4);
    }

    #[test]
    fn preview_requests_follow_sample_cadence() {
        let mut accumulator = SegmentAccumulator::with_limits(test_limits());
        accumulator
            .push(&[0.0; 6], &[event(6, VadEventKind::Speech)])
            .expect("valid speech");
        accumulator.push(&[0.0; 2], &[]).expect("valid audio");

        assert!(accumulator.take_preview_request(2).is_some());
        assert!(accumulator.take_preview_request(2).is_none());
        accumulator.push(&[0.0], &[]).expect("valid audio");
        assert!(accumulator.take_preview_request(2).is_none());
        accumulator.push(&[0.0], &[]).expect("valid audio");
        assert!(accumulator.take_preview_request(2).is_some());
    }

    #[test]
    fn fresh_preview_can_replace_final_decode_after_short_silent_tail() {
        let mut accumulator = accumulator_with_successful_preview();
        accumulator.push(&[0.0; 2], &[]).expect("valid silence");

        assert!(accumulator.can_reuse_successful_preview(false));
        accumulator.discard_active();
        assert!(accumulator.finish().is_none());
    }

    #[test]
    fn preview_reuse_rejects_new_speech_and_long_or_partial_tail() {
        let mut new_speech = accumulator_with_successful_preview();
        new_speech
            .push(&[0.0; 2], &[event(2, VadEventKind::Speech)])
            .expect("valid speech");
        assert!(!new_speech.can_reuse_successful_preview(false));

        let mut long_tail = accumulator_with_successful_preview();
        long_tail.push(&[0.0; 4], &[]).expect("valid long silence");
        assert!(!long_tail.can_reuse_successful_preview(false));

        let partial_speech = accumulator_with_successful_preview();
        assert!(!partial_speech.can_reuse_successful_preview(true));
    }

    #[test]
    fn preview_reuse_rejects_overlap_and_missing_successful_text() {
        let mut overlap = accumulator_with_successful_preview();
        overlap.active_has_overlap = true;
        assert!(!overlap.can_reuse_successful_preview(false));

        let mut missing_preview = SegmentAccumulator::with_limits(test_limits());
        missing_preview
            .push(&[0.0; 6], &[event(6, VadEventKind::Speech)])
            .expect("valid speech");
        assert!(!missing_preview.can_reuse_successful_preview(false));
    }

    #[test]
    fn adaptive_cadence_throttles_immediately_and_recovers_with_hysteresis() {
        let mut cadence = PreviewCadenceController::default();
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE / 4);

        cadence.observe_decode(400, 0, 256);
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE / 2);

        cadence.observe_decode(950, 0, 256);
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE);

        for _ in 0..2 {
            cadence.observe_decode(100, 0, 256);
        }
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE);

        cadence.observe_decode(100, 0, 256);
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE / 2);
        for _ in 0..3 {
            cadence.observe_decode(100, 0, 256);
        }
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE / 4);
    }

    #[test]
    fn adaptive_cadence_uses_queue_pressure_and_handles_zero_capacity() {
        let mut cadence = PreviewCadenceController::default();

        cadence.observe_decode(50, 35, 100);
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE / 2);

        cadence.observe_decode(50, 75, 100);
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE);

        for _ in 0..3 {
            cadence.observe_decode(50, u64::MAX, 0);
        }
        assert_eq!(cadence.preview_step_samples(), SAMPLE_RATE / 2);
    }

    #[test]
    fn invalid_event_offsets_are_rejected_without_mutation() {
        let mut accumulator = SegmentAccumulator::with_limits(test_limits());

        let result = accumulator.push(&[0.0; 4], &[event(5, VadEventKind::Speech)]);

        assert!(result.is_err());
        assert_eq!(accumulator.active_sample_count(), 0);
    }

    #[test]
    fn transcript_commits_segments_and_deduplicates_overlap() {
        let mut transcript = TranscriptState::default();
        assert!(transcript.commit("Hello brave world"));
        transcript.set_preview("brave world again");
        assert_eq!(transcript.display_text(), "Hello brave world again");

        assert!(transcript.commit("brave world again"));
        assert!(!transcript.commit("A new sentence"));
        assert_eq!(
            transcript.final_text(),
            "Hello brave world again A new sentence"
        );
    }

    #[test]
    fn transcript_can_commit_a_valid_preview_atomically() {
        let mut transcript = TranscriptState::default();
        assert_eq!(transcript.commit_preview(), None);

        transcript.set_preview("ready text");
        assert!(transcript.has_preview());
        assert_eq!(transcript.commit_preview(), Some(true));
        assert!(!transcript.has_preview());
        assert_eq!(transcript.final_text(), "ready text");
    }

    #[test]
    fn preview_commit_reports_overlap_safety_separately_from_presence() {
        let mut transcript = TranscriptState::default();
        assert!(transcript.commit("first segment"));
        transcript.set_preview("unrelated second segment");

        assert_eq!(transcript.commit_preview(), Some(false));
        assert_eq!(
            transcript.final_text(),
            "first segment unrelated second segment"
        );
    }

    #[test]
    fn reuse_policy_rejects_gaps_errors_empty_and_stale_results() {
        let valid = StreamingOutcome {
            generation: 7,
            transcript: "ready text".to_string(),
            dropped_chunks: 0,
            error: None,
            metrics: StreamingMetrics::default(),
        };
        assert_eq!(
            valid.reusable_transcript(7).expect("reusable"),
            "ready text"
        );

        let stale = StreamingOutcome {
            generation: 8,
            ..valid
        };
        assert!(stale.reusable_transcript(7).is_err());

        let gap = StreamingOutcome {
            generation: 7,
            transcript: "partial".to_string(),
            dropped_chunks: 1,
            error: None,
            metrics: StreamingMetrics::default(),
        };
        assert!(gap.reusable_transcript(7).is_err());

        let failed = StreamingOutcome {
            generation: 7,
            transcript: "partial".to_string(),
            dropped_chunks: 0,
            error: Some("socket closed".to_string()),
            metrics: StreamingMetrics::default(),
        };
        assert!(failed.reusable_transcript(7).is_err());

        let empty = StreamingOutcome {
            generation: 7,
            transcript: "  ".to_string(),
            dropped_chunks: 0,
            error: None,
            metrics: StreamingMetrics::default(),
        };
        assert!(empty.reusable_transcript(7).is_err());

        let recovered_endpoint = StreamingOutcome {
            generation: 7,
            transcript: "recovered preview".to_string(),
            dropped_chunks: 0,
            error: None,
            metrics: StreamingMetrics {
                empty_endpoint_decodes: 1,
                recovered_endpoint_previews: 1,
                ..StreamingMetrics::default()
            },
        };
        assert!(recovered_endpoint.reusable_transcript(7).is_err());
    }

    #[test]
    fn ten_minute_segmented_session_stays_bounded() {
        let mut accumulator = SegmentAccumulator::new();
        let chunk = vec![0.0; SAMPLE_RATE / 4];
        let total_chunks = 10 * 60 * 4;
        let chunks_per_segment = 5 * 4;
        let speech_chunks_per_segment = 18;
        let mut decoded_samples = 0usize;
        let mut max_request_samples = 0usize;
        let mut completed_segments = 0usize;

        for chunk_index in 0..total_chunks {
            let segment_offset = chunk_index % chunks_per_segment;
            let events = if segment_offset < speech_chunks_per_segment {
                vec![event(chunk.len(), VadEventKind::Speech)]
            } else if segment_offset + 1 == chunks_per_segment {
                vec![event(chunk.len(), VadEventKind::Endpoint)]
            } else {
                Vec::new()
            };

            let requests = accumulator.push(&chunk, &events).expect("valid events");
            for request in requests {
                completed_segments += usize::from(request.reason == DecodeReason::Endpoint);
                decoded_samples = decoded_samples.saturating_add(request.samples.len());
                max_request_samples = max_request_samples.max(request.samples.len());
            }
            if let Some(request) = accumulator.take_preview_request(FAST_PREVIEW_STEP_SAMPLES) {
                decoded_samples = decoded_samples.saturating_add(request.samples.len());
                max_request_samples = max_request_samples.max(request.samples.len());
            }
            assert!(accumulator.active_sample_count() <= SAMPLE_RATE * 30);
        }

        let recorded_samples = total_chunks * chunk.len();
        assert_eq!(completed_segments, 120);
        assert!(max_request_samples <= SAMPLE_RATE * 5);
        assert!(decoded_samples < recorded_samples * 12);
        assert!(accumulator.finish().is_none());
    }
}
