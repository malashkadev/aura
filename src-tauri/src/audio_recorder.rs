use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

const OUTPUT_SAMPLE_RATE: u32 = 16_000;
/// Bounded queue between the audio callback and the record worker. Sized so a
/// transient stall in downstream processing (WAV write, streaming publish, UI
/// volume emission) absorbs ~1.3 s of audio at 10 ms chunks instead of dropping
/// dictation. Overflow is still counted and surfaced via `dropped_chunks`.
const AUDIO_QUEUE_CAPACITY: usize = 128;
const SAMPLE_STREAM_QUEUE_CAPACITY: usize = 256;
const CONTROL_POLL_INTERVAL: Duration = Duration::from_millis(10);
const START_TIMEOUT: Duration = Duration::from_secs(10);
const FIR_TAPS: usize = 63;
const WAV_WRITE_BUFFER_BYTES: usize = 256 * 1024;
const WAV_HEADER_BYTES: u64 = 44;
const TOGGLE_TAIL_HOLD_MS: u64 = 90;
const HOLD_MIN_TAIL_MS: u64 = 100;
const HOLD_MAX_TAIL_MS: u64 = 250;
/// 180 ms at 16 kHz: when Hold mode has already drained >= 100 ms of WASAPI buffers
/// and at least the last 180 ms contain no speech, stop immediately instead of waiting 250 ms.
const HOLD_TRAILING_SILENCE_SAMPLES: u64 = 2_880;
const CALLBACK_POOL_BUFFERS: usize = 16;
const CALLBACK_POOL_INITIAL_CAPACITY: usize = 4_096;

pub struct AudioRecorder {
    state: Mutex<Option<ActiveRecording>>,
    /// True while a `start_recording` call has spawned its worker but not yet
    /// claimed `state`, and briefly after a startup timeout while the
    /// abandoned worker may still be opening the device. A second start in
    /// that window would open a concurrent stream on the same device; refuse
    /// it instead (C13).
    starting: AtomicBool,
    /// Single-pass VAD result produced by the last completed recording session:
    /// `Some(true)` = speech detected and trailing silence > 600 ms already trimmed,
    /// `Some(false)` = pure silence, `None` = streaming VAD unavailable (use file fallback).
    last_vad_has_speech: Mutex<Option<bool>>,
}

struct ActiveRecording {
    command_tx: mpsc::SyncSender<RecorderCommand>,
    worker: thread::JoinHandle<Result<Option<bool>, String>>,
    shared: Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<AudioSampleStream>,
}

pub struct AudioSampleStream {
    receiver: mpsc::Receiver<Vec<f32>>,
    dropped_chunks: Arc<AtomicU64>,
    queued_chunks: Arc<AtomicU64>,
    capacity: usize,
}

impl AudioSampleStream {
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Vec<f32>, mpsc::RecvTimeoutError> {
        let result = self.receiver.recv_timeout(timeout);
        if result.is_ok() {
            decrement_queued_chunks(&self.queued_chunks);
        }
        result
    }

    pub fn try_recv(&self) -> Result<Vec<f32>, mpsc::TryRecvError> {
        let result = self.receiver.try_recv();
        if result.is_ok() {
            decrement_queued_chunks(&self.queued_chunks);
        }
        result
    }

    pub fn dropped_chunks(&self) -> u64 {
        self.dropped_chunks.load(Ordering::Acquire)
    }

    pub fn queued_chunks(&self) -> u64 {
        self.queued_chunks.load(Ordering::Acquire)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for AudioSampleStream {
    fn drop(&mut self) {
        self.queued_chunks.store(0, Ordering::Release);
    }
}

struct SampleStreamPublisher {
    sender: mpsc::SyncSender<Vec<f32>>,
    dropped_chunks: Arc<AtomicU64>,
    queued_chunks: Arc<AtomicU64>,
}

impl SampleStreamPublisher {
    fn publish(&self, samples: Vec<f32>) {
        self.queued_chunks.fetch_add(1, Ordering::AcqRel);
        match self.sender.try_send(samples) {
            Ok(()) => {}
            Err(mpsc::TrySendError::Disconnected(_)) => {
                decrement_queued_chunks(&self.queued_chunks);
            }
            Err(mpsc::TrySendError::Full(_)) => {
                decrement_queued_chunks(&self.queued_chunks);
                self.dropped_chunks.fetch_add(1, Ordering::Release);
            }
        }
    }
}

fn sample_stream_channel(capacity: usize) -> (SampleStreamPublisher, AudioSampleStream) {
    let capacity = capacity.max(1);
    let (sender, receiver) = mpsc::sync_channel(capacity);
    let dropped_chunks = Arc::new(AtomicU64::new(0));
    let queued_chunks = Arc::new(AtomicU64::new(0));
    (
        SampleStreamPublisher {
            sender,
            dropped_chunks: Arc::clone(&dropped_chunks),
            queued_chunks: Arc::clone(&queued_chunks),
        },
        AudioSampleStream {
            receiver,
            dropped_chunks,
            queued_chunks,
            capacity,
        },
    )
}

fn decrement_queued_chunks(queued_chunks: &AtomicU64) {
    let mut current = queued_chunks.load(Ordering::Acquire);
    while current > 0 {
        match queued_chunks.compare_exchange_weak(
            current,
            current - 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => break,
            Err(observed) => current = observed,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecorderCommand {
    Stop { is_toggle: bool },
    Cancel,
}

struct AudioChunk {
    samples: Vec<f32>,
    rms: f32,
}

struct RecordingBuffer {
    samples_16k: Vec<f32>,
    runtime_error: Option<String>,
    retain_samples_for_preview: bool,
}

impl RecordingBuffer {
    fn new(retain_samples_for_preview: bool) -> Self {
        Self {
            samples_16k: Vec::new(),
            runtime_error: None,
            retain_samples_for_preview,
        }
    }

    fn append_samples(&mut self, samples: &[f32]) {
        if self.retain_samples_for_preview {
            self.samples_16k.extend_from_slice(samples);
        }
    }
}

impl Default for RecordingBuffer {
    fn default() -> Self {
        Self::new(true)
    }
}

fn pcm16_wav_header(data_bytes: u32) -> [u8; 44] {
    let riff_chunk_size = 36u32.saturating_add(data_bytes);
    let sample_rate = OUTPUT_SAMPLE_RATE;
    let channels = 1u16;
    let bits_per_sample = 16u16;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * u32::from(block_align);

    let mut header = [0u8; 44];
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&riff_chunk_size.to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16u32.to_le_bytes());
    header[20..22].copy_from_slice(&1u16.to_le_bytes()); // WAVE_FORMAT_PCM
    header[22..24].copy_from_slice(&channels.to_le_bytes());
    header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&block_align.to_le_bytes());
    header[34..36].copy_from_slice(&bits_per_sample.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&data_bytes.to_le_bytes());
    header
}

/// Writes into a side file and only publishes the final WAV path after optional
/// trailing-silence truncation, header finalization, flush, rename, and DACL
/// application. Dropping the guard removes the side file.
struct PendingWavFile {
    writer: Option<BufWriter<File>>,
    partial_path: PathBuf,
    final_path: PathBuf,
    sample_count: u64,
    committed: bool,
}

impl PendingWavFile {
    fn create(final_path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = final_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {e}"))?;
        }

        let partial_path = pending_wav_path(&final_path);
        let file = File::create(&partial_path)
            .map_err(|e| format!("Failed to create pending WAV file: {e}"))?;
        if let Err(error) = crate::secure_storage::apply_restrictive_acl(&partial_path) {
            crate::logger::log(
                "WARN",
                "Audio",
                None,
                &format!("Could not restrict ACL on pending recording: {error}"),
            );
        }
        let mut buffered = BufWriter::with_capacity(WAV_WRITE_BUFFER_BYTES, file);
        if let Err(error) = buffered.write_all(&pcm16_wav_header(0)) {
            drop(buffered);
            let _ = std::fs::remove_file(&partial_path);
            return Err(format!("Failed to write initial WAV header: {error}"));
        }
        Ok(Self {
            writer: Some(buffered),
            partial_path,
            final_path,
            sample_count: 0,
            committed: false,
        })
    }

    fn write_samples(&mut self, samples: &[f32]) -> Result<(), String> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| "Pending WAV writer is already closed".to_string())?;
        let mut pcm_bytes = [0u8; 2048];
        for chunk in samples.chunks(pcm_bytes.len() / 2) {
            let byte_len = chunk.len() * 2;
            for (idx, &sample) in chunk.iter().enumerate() {
                let bytes = sample_to_pcm_i16(sample).to_le_bytes();
                pcm_bytes[idx * 2] = bytes[0];
                pcm_bytes[idx * 2 + 1] = bytes[1];
            }
            writer
                .write_all(&pcm_bytes[..byte_len])
                .map_err(|e| format!("Failed to write WAV samples: {e}"))?;
        }
        self.sample_count = self.sample_count.saturating_add(samples.len() as u64);
        Ok(())
    }

    #[cfg(test)]
    fn commit(self) -> Result<(), String> {
        self.commit_with_keep_samples(None)
    }

    fn commit_with_keep_samples(mut self, max_samples_to_keep: Option<u64>) -> Result<(), String> {
        if self.sample_count == 0 {
            return Err("No audio samples were recorded".to_string());
        }
        let final_samples =
            max_samples_to_keep.map_or(self.sample_count, |keep| keep.clamp(1, self.sample_count));
        let data_bytes_u64 = final_samples
            .checked_mul(2)
            .ok_or_else(|| "WAV sample count overflow".to_string())?;
        let data_bytes = u32::try_from(data_bytes_u64)
            .map_err(|_| "WAV recording exceeds maximum 4 GB RIFF size".to_string())?;

        let buffered = self
            .writer
            .take()
            .ok_or_else(|| "Pending WAV writer is already closed".to_string())?;
        let mut file = buffered
            .into_inner()
            .map_err(|e| format!("Failed to flush pending WAV file: {}", e.error()))?;

        // Truncate trailing silence BEFORE finalizing the WAV header, renaming
        // `.wav.partial -> .wav`, and applying the restrictive DACL.
        if final_samples < self.sample_count {
            file.set_len(WAV_HEADER_BYTES + data_bytes_u64)
                .map_err(|e| format!("Failed to truncate trailing silence in WAV file: {e}"))?;
        }

        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("Failed to seek WAV header: {e}"))?;
        file.write_all(&pcm16_wav_header(data_bytes))
            .map_err(|e| format!("Failed to finalize WAV header: {e}"))?;
        file.flush()
            .map_err(|e| format!("Failed to flush finalized WAV file: {e}"))?;
        drop(file);

        match std::fs::remove_file(&self.final_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Failed to replace previous temporary WAV file: {error}"
                ));
            }
        }
        std::fs::rename(&self.partial_path, &self.final_path)
            .map_err(|e| format!("Failed to publish finalized WAV file: {e}"))?;
        if let Err(error) = crate::secure_storage::apply_restrictive_acl(&self.final_path) {
            crate::logger::log(
                "WARN",
                "Audio",
                None,
                &format!("Could not restrict ACL on finalized recording: {error}"),
            );
        }
        self.committed = true;
        Ok(())
    }
}

impl Drop for PendingWavFile {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        drop(self.writer.take());
        let _ = std::fs::remove_file(&self.partial_path);
    }
}

fn pending_wav_path(final_path: &Path) -> PathBuf {
    let mut path = final_path.as_os_str().to_os_string();
    path.push(".partial");
    PathBuf::from(path)
}

/// Removes leftover Aura recording files from a previous crash or power loss.
/// Only files created by `recording_wav_path` / `chunk_wav_path` are touched;
/// unrelated WAVs and directories are left alone.
pub(crate) fn sweep_stale_audio(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let is_aura_audio = (name.starts_with("aura-rec-") || name.starts_with("aura-chunk-"))
            && (name.ends_with(".wav") || name.ends_with(".wav.partial"));
        if is_aura_audio && std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

fn wav_spec() -> hound::WavSpec {
    hound::WavSpec {
        channels: 1,
        sample_rate: OUTPUT_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    }
}

/// Stateful downmixer/resampler used by the recorder worker and tests.
///
/// Uses a doubled contiguous ring buffer of length `taps + 1` so FIR convolution
/// runs ONLY at output 16 kHz sample positions (rather than on every 48 kHz/96 kHz
/// input frame) without modulo indexing in the inner loop.
struct StreamingResampler {
    channels: usize,
    input_rate: u32,
    partial_frame: Vec<f32>,
    coefficients_rev: Vec<f32>,
    ring_len: usize,
    history: Vec<f32>,
    write_pos: usize,
    input_index: u64,
    output_step: f64,
    next_output_position: f64,
    cached_filtered: Option<(u64, f32)>,
    #[cfg(test)]
    fir_evaluations: u64,
}

impl StreamingResampler {
    fn new(channels: u16, input_rate: u32) -> Result<Self, String> {
        if channels == 0 {
            return Err("Audio input reported zero channels".to_string());
        }
        if input_rate == 0 {
            return Err("Audio input reported a zero sample rate".to_string());
        }

        let coefficients = lowpass_coefficients(input_rate, OUTPUT_SAMPLE_RATE);
        let taps = coefficients.len();
        let ring_len = taps + 1;
        let coefficients_rev: Vec<f32> = coefficients.into_iter().rev().collect();
        Ok(Self {
            channels: channels as usize,
            input_rate,
            partial_frame: Vec::with_capacity(channels as usize),
            coefficients_rev,
            ring_len,
            history: vec![0.0; ring_len * 2],
            write_pos: 0,
            input_index: 0,
            output_step: input_rate as f64 / OUTPUT_SAMPLE_RATE as f64,
            next_output_position: 0.0,
            cached_filtered: None,
            #[cfg(test)]
            fir_evaluations: 0,
        })
    }

    fn process_interleaved(&mut self, input: &[f32], output: &mut Vec<f32>) {
        if self.channels == 1 {
            for &sample in input {
                self.push_mono(sanitize_sample(sample), output);
            }
            return;
        }

        let mut remaining = input;
        if !self.partial_frame.is_empty() {
            let needed = self.channels - self.partial_frame.len();
            if remaining.len() < needed {
                for &sample in remaining {
                    self.partial_frame.push(sanitize_sample(sample));
                }
                return;
            }
            for &sample in &remaining[..needed] {
                self.partial_frame.push(sanitize_sample(sample));
            }
            let mono = self.partial_frame.iter().copied().sum::<f32>() / self.channels as f32;
            self.partial_frame.clear();
            self.push_mono(mono, output);
            remaining = &remaining[needed..];
        }

        let inv_channels = 1.0 / self.channels as f32;
        let chunks = remaining.chunks_exact(self.channels);
        let tail = chunks.remainder();
        for frame in chunks {
            let mut sum = 0.0f32;
            for &sample in frame {
                sum += sanitize_sample(sample);
            }
            self.push_mono(sum * inv_channels, output);
        }
        for &sample in tail {
            self.partial_frame.push(sanitize_sample(sample));
        }
    }

    fn finish(&mut self, output: &mut Vec<f32>) {
        self.partial_frame.clear();

        if self.input_index == 0 {
            return;
        }
        let end = self.input_index as f64;
        if self.next_output_position < end - f64::EPSILON {
            let last_idx = self.input_index - 1;
            let last_pos = (self.write_pos + self.ring_len - 1) % self.ring_len;
            let last = match self.cached_filtered {
                Some((idx, val)) if idx == last_idx => val,
                _ => {
                    let val = self.convolve_at_slot(last_pos);
                    self.cached_filtered = Some((last_idx, val));
                    val
                }
            };
            while self.next_output_position < end - f64::EPSILON {
                output.push(last);
                self.next_output_position += self.output_step;
            }
        }
    }

    #[inline]
    fn convolve_at_slot(&mut self, slot: usize) -> f32 {
        #[cfg(test)]
        {
            self.fir_evaluations += 1;
        }
        let taps = self.coefficients_rev.len();
        if taps == 1 {
            return self.history[slot];
        }
        let end = slot + self.ring_len + 1;
        let start = end - taps;
        let window = &self.history[start..end];
        let mut sum = 0.0f32;
        for (&coeff, &sample) in self.coefficients_rev.iter().zip(window.iter()) {
            sum += coeff * sample;
        }
        sum
    }

    #[inline]
    fn push_mono(&mut self, sample: f32, output: &mut Vec<f32>) {
        let pos = self.write_pos;
        self.history[pos] = sample;
        self.history[pos + self.ring_len] = sample;
        self.write_pos = (pos + 1) % self.ring_len;

        let current_position = self.input_index as f64;
        if self.next_output_position <= current_position + f64::EPSILON {
            let current = self.convolve_at_slot(pos);
            if self.input_index == 0 {
                output.push(current);
                self.next_output_position += self.output_step;
            } else {
                let prev_idx = self.input_index - 1;
                let previous_position = prev_idx as f64;
                let mut previous_val: Option<f32> = None;
                while self.next_output_position <= current_position + f64::EPSILON {
                    let fraction =
                        (self.next_output_position - previous_position).clamp(0.0, 1.0) as f32;
                    if (1.0 - fraction).abs() <= f32::EPSILON {
                        output.push(current);
                    } else {
                        let prev = match previous_val {
                            Some(v) => v,
                            None => {
                                let v = match self.cached_filtered {
                                    Some((idx, cached)) if idx == prev_idx => cached,
                                    _ => {
                                        let prev_slot = (pos + self.ring_len - 1) % self.ring_len;
                                        self.convolve_at_slot(prev_slot)
                                    }
                                };
                                previous_val = Some(v);
                                v
                            }
                        };
                        output.push(prev + (current - prev) * fraction);
                    }
                    self.next_output_position += self.output_step;
                }
            }
            self.cached_filtered = Some((self.input_index, current));
        }
        self.input_index += 1;
    }
}

pub fn list_audio_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    let mut names = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if !name.trim().is_empty() && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    names
}

fn find_input_device(device_name: Option<&str>) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    if let Some(name) = device_name {
        let name_trimmed = name.trim();
        if !name_trimmed.is_empty() && name_trimmed != "default" {
            if let Ok(devices) = host.input_devices() {
                for dev in devices {
                    if let Ok(dev_name) = dev.name() {
                        if dev_name == name_trimmed {
                            return Ok(dev);
                        }
                    }
                }
            }
            crate::logger::log(
                "WARN",
                "Audio",
                None,
                &format!("Specified audio input device '{name_trimmed}' not found; falling back to default device"),
            );
        }
    }
    host.default_input_device()
        .ok_or_else(|| "No default audio input device found".to_string())
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(None),
            starting: AtomicBool::new(false),
            last_vad_has_speech: Mutex::new(None),
        }
    }

    fn set_last_vad_has_speech(&self, value: Option<bool>) {
        match self.last_vad_has_speech.lock() {
            Ok(mut guard) => *guard = value,
            Err(poisoned) => *poisoned.into_inner() = value,
        }
    }

    /// Takes and clears the single-pass `StreamingVad` result of the last finished recording:
    /// - `Some(true)`: speech was detected and trailing silence > 600 ms was already truncated.
    /// - `Some(false)`: no speech was detected anywhere in the recording.
    /// - `None`: streaming VAD did not run; callers should fall back to file-based VAD.
    pub fn take_last_vad_has_speech(&self) -> Option<bool> {
        match self.last_vad_has_speech.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }

    /// Starts the platform audio stream on a dedicated owner thread.
    ///
    /// cpal::Stream deliberately is not Send on all supported backends, so the
    /// handle never crosses a thread boundary. The callback uses a bounded queue
    /// and never waits for resampling, disk I/O, Tauri events, or a mutex.
    pub fn start_recording<F>(
        &self,
        output_path: &str,
        enable_sample_stream: bool,
        retain_samples_for_preview: bool,
        device_name: Option<&str>,
        on_volume: F,
    ) -> Result<(), String>
    where
        F: Fn(f32) + Send + 'static,
    {
        if self.starting.swap(true, Ordering::SeqCst) {
            return Err("Audio recorder start is already in progress".to_string());
        }
        let result = self.start_recording_impl(
            output_path,
            enable_sample_stream,
            retain_samples_for_preview,
            device_name,
            on_volume,
        );
        self.starting.store(false, Ordering::SeqCst);
        result
    }

    fn start_recording_impl<F>(
        &self,
        output_path: &str,
        enable_sample_stream: bool,
        retain_samples_for_preview: bool,
        device_name: Option<&str>,
        on_volume: F,
    ) -> Result<(), String>
    where
        F: Fn(f32) + Send + 'static,
    {
        {
            let state = self
                .state
                .lock()
                .map_err(|_| "Audio recorder state is unavailable".to_string())?;
            if state.is_some() {
                return Err("Already recording".to_string());
            }
        }
        self.set_last_vad_has_speech(None);

        let (command_tx, command_rx) = mpsc::sync_channel(1);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (sample_stream_publisher, sample_stream) = if enable_sample_stream {
            let (publisher, stream) = sample_stream_channel(SAMPLE_STREAM_QUEUE_CAPACITY);
            (Some(publisher), Some(stream))
        } else {
            (None, None)
        };
        let shared = Arc::new(Mutex::new(RecordingBuffer::new(retain_samples_for_preview)));
        let worker_shared = Arc::clone(&shared);
        let output_path = PathBuf::from(output_path);
        let dev_name = device_name.map(str::to_string);

        let worker = thread::Builder::new()
            .name("aura-audio-recorder".to_string())
            .spawn(move || {
                // A panic anywhere in the worker must become a session error
                // instead of silently killing the recording thread.
                let panic_shared = Arc::clone(&worker_shared);
                let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    recorder_worker(
                        output_path,
                        command_rx,
                        ready_tx,
                        worker_shared,
                        sample_stream_publisher,
                        dev_name,
                        Box::new(on_volume),
                    )
                }));
                match panic_result {
                    Ok(recorder_result) => recorder_result,
                    Err(payload) => {
                        let message = panic_payload_text(payload);
                        remember_runtime_error(&panic_shared, &message);
                        crate::logger::log(
                            "ERROR",
                            "Audio",
                            None,
                            &format!("Audio recorder worker panicked: {message}"),
                        );
                        Err(message)
                    }
                }
            })
            .map_err(|e| format!("Failed to start audio owner thread: {e}"))?;

        match ready_rx.recv_timeout(START_TIMEOUT) {
            Ok(Ok(())) => {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| "Audio recorder state is unavailable".to_string())?;
                if state.is_some() {
                    let _ = command_tx.try_send(RecorderCommand::Cancel);
                    drop(worker);
                    return Err("Already recording".to_string());
                }
                *state = Some(ActiveRecording {
                    command_tx,
                    worker,
                    shared,
                    sample_stream,
                });
                Ok(())
            }
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                let _ = command_tx.try_send(RecorderCommand::Cancel);
                drop(worker);
                Err("Timed out while starting the audio input device".to_string())
            }
        }
    }

    pub fn stop_recording(&self) -> Result<(), String> {
        self.stop_recording_adaptive(false)
    }

    pub fn stop_recording_adaptive(&self, is_toggle: bool) -> Result<(), String> {
        self.finish_recording(RecorderCommand::Stop { is_toggle })
    }

    pub fn cancel_recording(&self) -> Result<(), String> {
        self.finish_recording(RecorderCommand::Cancel)
    }

    pub fn take_sample_stream(&self) -> Result<AudioSampleStream, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Audio recorder state is unavailable".to_string())?;
        state
            .as_mut()
            .ok_or_else(|| "Not recording".to_string())?
            .sample_stream
            .take()
            .ok_or_else(|| "Audio sample stream is unavailable".to_string())
    }

    /// Returns a 16 kHz mono snapshot for legacy cloud-preview code.
    pub fn get_recorded_samples(&self) -> Result<(Vec<f32>, u32, u16), String> {
        let shared = {
            let state = self
                .state
                .lock()
                .map_err(|_| "Audio recorder state is unavailable".to_string())?;
            Arc::clone(
                &state
                    .as_ref()
                    .ok_or_else(|| "Not recording".to_string())?
                    .shared,
            )
        };
        let buffer = shared
            .lock()
            .map_err(|_| "Audio sample buffer is unavailable".to_string())?;
        if let Some(error) = &buffer.runtime_error {
            return Err(error.clone());
        }
        if !buffer.retain_samples_for_preview {
            return Err("Audio samples are not retained in this recording mode".to_string());
        }
        Ok((buffer.samples_16k.clone(), OUTPUT_SAMPLE_RATE, 1))
    }

    fn finish_recording(&self, command: RecorderCommand) -> Result<(), String> {
        let active = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| "Audio recorder state is unavailable".to_string())?;
            state.take().ok_or_else(|| "Not recording".to_string())?
        };

        let command_error = active.command_tx.send(command).err();
        match active.worker.join() {
            Ok(Ok(vad_has_speech)) => {
                if matches!(command, RecorderCommand::Stop { .. }) {
                    self.set_last_vad_has_speech(vad_has_speech);
                } else {
                    self.set_last_vad_has_speech(None);
                }
                Ok(())
            }
            Ok(Err(err)) => {
                self.set_last_vad_has_speech(None);
                Err(err)
            }
            Err(_) => {
                self.set_last_vad_has_speech(None);
                Err(command_error
                    .map(|e| format!("Audio worker stopped before receiving its command: {e}"))
                    .unwrap_or_else(|| "Audio worker panicked".to_string()))
            }
        }
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, serde::Serialize)]
pub struct MicMeterPayload {
    pub volume: f32,
    pub is_speech: bool,
}

struct ActiveMicMeter {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

static ACTIVE_MIC_METER: OnceLock<Mutex<Option<ActiveMicMeter>>> = OnceLock::new();

fn clear_active_meter_if_matches(thread_stop: &Arc<AtomicBool>) {
    let mutex = ACTIVE_MIC_METER.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = mutex.lock() {
        if let Some(current) = guard.as_ref() {
            if Arc::ptr_eq(&current.stop, thread_stop) {
                *guard = None;
            }
        }
    }
}

pub fn is_mic_meter_running() -> bool {
    let mutex = ACTIVE_MIC_METER.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = mutex.lock() {
        guard.is_some()
    } else {
        false
    }
}

pub fn stop_mic_meter() {
    let active = {
        let mutex = ACTIVE_MIC_METER.get_or_init(|| Mutex::new(None));
        match mutex.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    };
    if let Some(mut active) = active {
        active.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = active.handle.take() {
            if handle.thread().id() != thread::current().id() {
                let _ = handle.join();
            }
        }
    }
}

pub fn start_mic_meter<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    device_name: Option<&str>,
) -> Result<(), String> {
    stop_mic_meter();

    let stop_flag = Arc::new(AtomicBool::new(false));
    let dev_name = device_name.map(str::to_string);
    let thread_stop = Arc::clone(&stop_flag);

    let mutex = ACTIVE_MIC_METER.get_or_init(|| Mutex::new(None));
    let mut guard = mutex
        .lock()
        .map_err(|_| "Failed to lock mic meter state".to_string())?;

    let handle = std::thread::Builder::new()
        .name("aura-mic-meter".to_string())
        .spawn(move || {
            let device = match find_input_device(dev_name.as_deref()) {
                Ok(d) => d,
                Err(_) => {
                    clear_active_meter_if_matches(&thread_stop);
                    return;
                }
            };
            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(_) => {
                    clear_active_meter_if_matches(&thread_stop);
                    return;
                }
            };
            let sample_format = config.sample_format();
            let stream_config: cpal::StreamConfig = config.into();

            let (sample_tx, sample_rx) = mpsc::sync_channel::<Vec<f32>>(64);
            let (recycle_tx, recycle_rx) = mpsc::sync_channel::<Vec<f32>>(16);
            for _ in 0..8 {
                let _ = recycle_tx.try_send(Vec::with_capacity(CALLBACK_POOL_INITIAL_CAPACITY));
            }
            let cb_recycle_tx = recycle_tx.clone();
            let stream = match sample_format {
                cpal::SampleFormat::I16 => device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _| {
                        let mut samples = recycle_rx
                            .try_recv()
                            .unwrap_or_else(|_| Vec::with_capacity(data.len()));
                        samples.clear();
                        samples.extend(data.iter().map(|&s| sanitize_sample(s as f32 / 32768.0)));
                        if let Err(mpsc::TrySendError::Full(buf)) = sample_tx.try_send(samples) {
                            let _ = cb_recycle_tx.try_send(buf);
                        }
                    },
                    |_| {},
                    None,
                ),
                cpal::SampleFormat::U16 => device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _| {
                        let mut samples = recycle_rx
                            .try_recv()
                            .unwrap_or_else(|_| Vec::with_capacity(data.len()));
                        samples.clear();
                        samples.extend(
                            data.iter()
                                .map(|&s| sanitize_sample((s as f32 - 32768.0) / 32768.0)),
                        );
                        if let Err(mpsc::TrySendError::Full(buf)) = sample_tx.try_send(samples) {
                            let _ = cb_recycle_tx.try_send(buf);
                        }
                    },
                    |_| {},
                    None,
                ),
                cpal::SampleFormat::I32 => device.build_input_stream(
                    &stream_config,
                    move |data: &[i32], _| {
                        let mut samples = recycle_rx
                            .try_recv()
                            .unwrap_or_else(|_| Vec::with_capacity(data.len()));
                        samples.clear();
                        samples.extend(
                            data.iter()
                                .map(|&s| sanitize_sample(s as f32 / 2147483648.0)),
                        );
                        if let Err(mpsc::TrySendError::Full(buf)) = sample_tx.try_send(samples) {
                            let _ = cb_recycle_tx.try_send(buf);
                        }
                    },
                    |_| {},
                    None,
                ),
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        let mut samples = recycle_rx
                            .try_recv()
                            .unwrap_or_else(|_| Vec::with_capacity(data.len()));
                        samples.clear();
                        samples.extend(data.iter().map(|&s| sanitize_sample(s)));
                        if let Err(mpsc::TrySendError::Full(buf)) = sample_tx.try_send(samples) {
                            let _ = cb_recycle_tx.try_send(buf);
                        }
                    },
                    |_| {},
                    None,
                ),
                _ => {
                    clear_active_meter_if_matches(&thread_stop);
                    return;
                }
            };

            let stream = match stream {
                Ok(s) => s,
                Err(_) => {
                    clear_active_meter_if_matches(&thread_stop);
                    return;
                }
            };

            if stream.play().is_err() {
                clear_active_meter_if_matches(&thread_stop);
                return;
            }

            let mut last_emit = std::time::Instant::now();
            let mut acc_sum_sq = 0.0f32;
            let mut acc_count = 0usize;

            while !thread_stop.load(Ordering::Relaxed) {
                match sample_rx.recv_timeout(Duration::from_millis(15)) {
                    Ok(raw_chunk) => {
                        for &s in &raw_chunk {
                            acc_sum_sq += s * s;
                        }
                        acc_count += raw_chunk.len();
                        let _ = recycle_tx.try_send(raw_chunk);

                        if (acc_count >= 512 || last_emit.elapsed() >= Duration::from_millis(20))
                            && acc_count > 0
                        {
                            let rms = (acc_sum_sq / acc_count as f32).sqrt();
                            let volume = (rms * 3.5).clamp(0.0, 1.0);
                            let is_speech = volume > 0.05;

                            let _ = app_handle
                                .emit("mic-meter-level", MicMeterPayload { volume, is_speech });

                            acc_sum_sq = 0.0;
                            acc_count = 0;
                            last_emit = std::time::Instant::now();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }

            drop(stream);
            clear_active_meter_if_matches(&thread_stop);
        })
        .map_err(|e| format!("Failed to spawn mic meter thread: {e}"))?;

    *guard = Some(ActiveMicMeter {
        stop: stop_flag,
        handle: Some(handle),
    });

    Ok(())
}

fn should_continue_tail_hold(
    is_toggle: bool,
    elapsed: Duration,
    total_samples: u64,
    last_speech_sample: Option<u64>,
) -> bool {
    if is_toggle {
        return elapsed < Duration::from_millis(TOGGLE_TAIL_HOLD_MS);
    }
    if elapsed < Duration::from_millis(HOLD_MIN_TAIL_MS) {
        return true;
    }
    if elapsed >= Duration::from_millis(HOLD_MAX_TAIL_MS) {
        return false;
    }
    match last_speech_sample {
        Some(last_speech) => {
            total_samples.saturating_sub(last_speech) < HOLD_TRAILING_SILENCE_SAMPLES
        }
        None => false,
    }
}

fn finalize_session_wav_with_vad(
    wav_writer: PendingWavFile,
    streaming_vad: Option<&mut crate::vad::StreamingVad>,
) -> Result<Option<bool>, String> {
    let vad_outcome = streaming_vad.map(|vad| {
        let _ = vad.finish_pending_has_speech();
        (vad.has_speech(), vad.last_speech_end_sample())
    });

    match vad_outcome {
        Some((true, Some(speech_end))) => {
            let keep_samples =
                speech_end.saturating_add(crate::vad::TRAILING_SILENCE_PADDING_SAMPLES);
            wav_writer.commit_with_keep_samples(Some(keep_samples))?;
            Ok(Some(true))
        }
        Some((true, None)) => {
            wav_writer.commit_with_keep_samples(None)?;
            Ok(Some(true))
        }
        Some((false, _)) => {
            wav_writer.commit_with_keep_samples(None)?;
            Ok(Some(false))
        }
        None => {
            wav_writer.commit_with_keep_samples(None)?;
            Ok(None)
        }
    }
}

fn recorder_worker(
    output_path: PathBuf,
    command_rx: mpsc::Receiver<RecorderCommand>,
    ready_tx: mpsc::SyncSender<Result<(), String>>,
    shared: Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<SampleStreamPublisher>,
    device_name: Option<String>,
    on_volume: Box<dyn Fn(f32) + Send>,
) -> Result<Option<bool>, String> {
    let (stream, sample_rx, recycle_tx, error_rx, dropped_chunks, channels, sample_rate) =
        match setup_input_stream(device_name.as_deref()) {
            Ok(value) => value,
            Err(error) => {
                let _ = ready_tx.send(Err(error.clone()));
                return Err(error);
            }
        };

    let mut resampler = match StreamingResampler::new(channels, sample_rate) {
        Ok(value) => value,
        Err(error) => {
            let _ = ready_tx.send(Err(error.clone()));
            return Err(error);
        }
    };

    let mut wav_writer = match PendingWavFile::create(output_path) {
        Ok(writer) => writer,
        Err(error) => {
            let _ = ready_tx.send(Err(error.clone()));
            return Err(error);
        }
    };

    let mut streaming_vad = crate::vad::StreamingVad::new_16k().ok();
    let mut scratch_resampled = Vec::with_capacity(1024);
    let mut total_observed_samples = 0u64;

    if let Err(error) = stream.play() {
        let error = format!("Failed to start the input stream: {error}");
        let _ = ready_tx.send(Err(error.clone()));
        return Err(error);
    }
    ready_tx
        .send(Ok(()))
        .map_err(|_| "Recording startup was cancelled".to_string())?;

    let outcome = loop {
        if let Err(error) = drain_audio_queue(
            &sample_rx,
            &recycle_tx,
            &shared,
            sample_stream.as_ref(),
            &mut resampler,
            &mut wav_writer,
            streaming_vad.as_mut(),
            &mut scratch_resampled,
            &mut total_observed_samples,
            on_volume.as_ref(),
        ) {
            break Err(error);
        }

        if let Ok(error) = error_rx.try_recv() {
            break Err(error);
        }

        match command_rx.recv_timeout(CONTROL_POLL_INTERVAL) {
            Ok(command) => break Ok(command),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break Ok(RecorderCommand::Cancel),
        }
    };

    if let Ok(RecorderCommand::Stop { is_toggle }) = outcome {
        // Adaptive Post-Release Audio Grace Buffer (Tail Hold):
        // - Toggle mode: 90 ms WASAPI hardware drain.
        // - Hold mode: minimum 100 ms WASAPI drain, early exit once the last 180 ms
        //   contain no speech, capped at 250 ms.
        let tail_started = std::time::Instant::now();
        loop {
            let _ = drain_audio_queue(
                &sample_rx,
                &recycle_tx,
                &shared,
                sample_stream.as_ref(),
                &mut resampler,
                &mut wav_writer,
                streaming_vad.as_mut(),
                &mut scratch_resampled,
                &mut total_observed_samples,
                on_volume.as_ref(),
            );
            let last_speech = streaming_vad
                .as_ref()
                .and_then(|vad| vad.last_speech_end_sample());
            if !should_continue_tail_hold(
                is_toggle,
                tail_started.elapsed(),
                total_observed_samples,
                last_speech,
            ) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    drop(stream);

    let command = match outcome {
        Ok(command) => command,
        Err(error) => {
            remember_runtime_error(&shared, &error);
            return Err(error);
        }
    };

    // Drain up to 2 seconds of tail chunks. An unbounded `recv()` here would
    // hang forever if the audio device fails to release the callback on stop
    // (driver hang), leaving the session stuck on "processing" (C13).
    let drain_deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if std::time::Instant::now() >= drain_deadline {
            crate::logger::log(
                "WARN",
                "Audio",
                None,
                "Audio tail drain timed out; dropping remaining buffered chunks",
            );
            break;
        }
        match sample_rx.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(chunk) => {
                if let Err(error) = append_audio_chunk(
                    chunk,
                    &recycle_tx,
                    &shared,
                    sample_stream.as_ref(),
                    &mut resampler,
                    &mut wav_writer,
                    streaming_vad.as_mut(),
                    &mut scratch_resampled,
                    &mut total_observed_samples,
                    on_volume.as_ref(),
                ) {
                    remember_runtime_error(&shared, &error);
                    return Err(error);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let dropped = dropped_chunks.load(Ordering::Relaxed);
    if dropped > 0 {
        if let Some(publisher) = sample_stream.as_ref() {
            publisher
                .dropped_chunks
                .fetch_add(dropped, Ordering::Release);
        }
        crate::logger::log(
            "WARN",
            "Audio",
            None,
            &format!("Audio callback queue overflowed; {dropped} chunks were dropped"),
        );
    }

    if matches!(command, RecorderCommand::Stop { .. }) {
        let mut tail = Vec::new();
        resampler.finish(&mut tail);
        if let Err(error) = append_resampled_samples(
            tail,
            &shared,
            sample_stream.as_ref(),
            &mut wav_writer,
            streaming_vad.as_mut(),
            &mut total_observed_samples,
        ) {
            remember_runtime_error(&shared, &error);
            return Err(error);
        }
    }

    if let Some(publisher) = sample_stream.as_ref() {
        let dropped = publisher.dropped_chunks.load(Ordering::Acquire);
        if dropped > 0 {
            crate::logger::log(
                "WARN",
                "Audio",
                None,
                &format!("Streaming input observed {dropped} missing or overflowed chunks"),
            );
        }
    }

    // Closing the streaming sender after publishing the resampler tail lets the
    // ASR worker finish in parallel with the buffered WAV header flush/rename.
    drop(sample_stream);

    if matches!(command, RecorderCommand::Stop { .. }) {
        return match finalize_session_wav_with_vad(wav_writer, streaming_vad.as_mut()) {
            Ok(vad_has_speech) => Ok(vad_has_speech),
            Err(error) => {
                remember_runtime_error(&shared, &error);
                Err(error)
            }
        };
    }

    Ok(None)
}

fn remember_runtime_error(shared: &Arc<Mutex<RecordingBuffer>>, error: &str) {
    match shared.lock() {
        Ok(mut buffer) => buffer.runtime_error = Some(error.to_string()),
        Err(poisoned) => {
            // A panic while the guard was held must not discard this error nor
            // keep the buffer poisoned for the rest of the session.
            poisoned.into_inner().runtime_error = Some(error.to_string());
            shared.clear_poison();
        }
    }
}

fn panic_payload_text(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

type StreamSetup = (
    cpal::Stream,
    mpsc::Receiver<AudioChunk>,
    mpsc::SyncSender<Vec<f32>>,
    mpsc::Receiver<String>,
    Arc<AtomicU64>,
    u16,
    u32,
);

fn setup_input_stream(device_name: Option<&str>) -> Result<StreamSetup, String> {
    let device = find_input_device(device_name)?;
    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default input config: {e}"))?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();

    let (sample_tx, sample_rx) = mpsc::sync_channel(AUDIO_QUEUE_CAPACITY);
    let (recycle_tx, recycle_rx) = mpsc::sync_channel(AUDIO_QUEUE_CAPACITY);
    for _ in 0..CALLBACK_POOL_BUFFERS {
        let _ = recycle_tx.try_send(Vec::with_capacity(CALLBACK_POOL_INITIAL_CAPACITY));
    }
    let cb_recycle_tx = recycle_tx.clone();
    let (error_tx, error_rx) = mpsc::channel();
    let dropped = Arc::new(AtomicU64::new(0));
    let callback_dropped = Arc::clone(&dropped);

    let stream = match sample_format {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| {
                fill_and_publish_audio_chunk(
                    data.iter().map(|&s| s as f32 / 32768.0),
                    data.len(),
                    &recycle_rx,
                    &cb_recycle_tx,
                    &sample_tx,
                    &callback_dropped,
                );
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| {
                fill_and_publish_audio_chunk(
                    data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0),
                    data.len(),
                    &recycle_rx,
                    &cb_recycle_tx,
                    &sample_tx,
                    &callback_dropped,
                );
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        cpal::SampleFormat::I32 => device.build_input_stream(
            &stream_config,
            move |data: &[i32], _| {
                fill_and_publish_audio_chunk(
                    data.iter().map(|&s| s as f32 / 2147483648.0),
                    data.len(),
                    &recycle_rx,
                    &cb_recycle_tx,
                    &sample_tx,
                    &callback_dropped,
                );
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                fill_and_publish_audio_chunk(
                    data.iter().copied(),
                    data.len(),
                    &recycle_rx,
                    &cb_recycle_tx,
                    &sample_tx,
                    &callback_dropped,
                );
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        other => return Err(format!("Unsupported sample format: {other:?}")),
    }
    .map_err(|e| format!("Failed to build input stream: {e}"))?;

    Ok((
        stream,
        sample_rx,
        recycle_tx,
        error_rx,
        dropped,
        channels,
        sample_rate,
    ))
}

fn fill_and_publish_audio_chunk<I>(
    iter: I,
    len_hint: usize,
    recycle_rx: &mpsc::Receiver<Vec<f32>>,
    recycle_tx: &mpsc::SyncSender<Vec<f32>>,
    sender: &mpsc::SyncSender<AudioChunk>,
    dropped: &AtomicU64,
) where
    I: Iterator<Item = f32>,
{
    let mut samples = recycle_rx
        .try_recv()
        .unwrap_or_else(|_| Vec::with_capacity(len_hint));
    samples.clear();
    let mut sum_sq = 0.0f32;
    for raw in iter {
        let s = sanitize_sample(raw);
        sum_sq += s * s;
        samples.push(s);
    }
    let rms = if samples.is_empty() {
        0.0
    } else {
        (sum_sq / samples.len() as f32).sqrt()
    };
    if let Err(mpsc::TrySendError::Full(chunk)) = sender.try_send(AudioChunk { samples, rms }) {
        let _ = recycle_tx.try_send(chunk.samples);
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}

#[allow(clippy::too_many_arguments)]
fn drain_audio_queue(
    receiver: &mpsc::Receiver<AudioChunk>,
    recycle_tx: &mpsc::SyncSender<Vec<f32>>,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
    resampler: &mut StreamingResampler,
    wav_writer: &mut PendingWavFile,
    mut streaming_vad: Option<&mut crate::vad::StreamingVad>,
    scratch_resampled: &mut Vec<f32>,
    total_observed_samples: &mut u64,
    on_volume: &dyn Fn(f32),
) -> Result<(), String> {
    while let Ok(chunk) = receiver.try_recv() {
        append_audio_chunk(
            chunk,
            recycle_tx,
            shared,
            sample_stream,
            resampler,
            wav_writer,
            streaming_vad.as_deref_mut(),
            scratch_resampled,
            total_observed_samples,
            on_volume,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn append_audio_chunk(
    chunk: AudioChunk,
    recycle_tx: &mpsc::SyncSender<Vec<f32>>,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
    resampler: &mut StreamingResampler,
    wav_writer: &mut PendingWavFile,
    streaming_vad: Option<&mut crate::vad::StreamingVad>,
    scratch_resampled: &mut Vec<f32>,
    total_observed_samples: &mut u64,
    on_volume: &dyn Fn(f32),
) -> Result<(), String> {
    let rms = chunk.rms;
    if sample_stream.is_some() {
        let capacity = chunk.samples.len() * OUTPUT_SAMPLE_RATE as usize
            / (resampler.input_rate as usize * resampler.channels).max(1)
            + 2;
        let mut converted = Vec::with_capacity(capacity);
        resampler.process_interleaved(&chunk.samples, &mut converted);
        let _ = recycle_tx.try_send(chunk.samples);
        append_resampled_samples(
            converted,
            shared,
            sample_stream,
            wav_writer,
            streaming_vad,
            total_observed_samples,
        )?;
    } else {
        scratch_resampled.clear();
        resampler.process_interleaved(&chunk.samples, scratch_resampled);
        let _ = recycle_tx.try_send(chunk.samples);
        append_resampled_slice_no_stream(
            scratch_resampled,
            shared,
            wav_writer,
            streaming_vad,
            total_observed_samples,
        )?;
    }
    on_volume(rms);
    Ok(())
}

fn append_resampled_slice_no_stream(
    samples: &[f32],
    shared: &Arc<Mutex<RecordingBuffer>>,
    wav_writer: &mut PendingWavFile,
    streaming_vad: Option<&mut crate::vad::StreamingVad>,
    total_observed_samples: &mut u64,
) -> Result<(), String> {
    if samples.is_empty() {
        return Ok(());
    }
    *total_observed_samples = total_observed_samples.saturating_add(samples.len() as u64);
    if let Some(vad) = streaming_vad {
        vad.observe_samples(samples);
    }
    wav_writer.write_samples(samples)?;
    shared
        .lock()
        .map_err(|_| "Audio sample buffer is unavailable".to_string())?
        .append_samples(samples);
    Ok(())
}

fn append_resampled_samples(
    samples: Vec<f32>,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
    wav_writer: &mut PendingWavFile,
    streaming_vad: Option<&mut crate::vad::StreamingVad>,
    total_observed_samples: &mut u64,
) -> Result<(), String> {
    if samples.is_empty() {
        return Ok(());
    }
    *total_observed_samples = total_observed_samples.saturating_add(samples.len() as u64);
    if let Some(vad) = streaming_vad {
        vad.observe_samples(&samples);
    }
    wav_writer.write_samples(&samples)?;
    retain_and_publish_samples(samples, shared, sample_stream)
}

fn retain_and_publish_samples(
    samples: Vec<f32>,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
) -> Result<(), String> {
    shared
        .lock()
        .map_err(|_| "Audio sample buffer is unavailable".to_string())?
        .append_samples(&samples);

    if let Some(publisher) = sample_stream {
        publisher.publish(samples);
    }

    Ok(())
}

fn sanitize_sample(sample: f32) -> f32 {
    if sample.is_finite() {
        sample.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn sample_to_pcm_i16(sample: f32) -> i16 {
    let sample = sanitize_sample(sample);
    if sample >= 0.0 {
        (sample * i16::MAX as f32) as i16
    } else {
        (sample * 32768.0) as i16
    }
}

fn lowpass_coefficients(input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input_rate <= output_rate {
        return vec![1.0];
    }

    use std::f64::consts::PI;
    // Cycles per input sample; 0.94 leaves a transition band below 8 kHz.
    let cutoff = output_rate as f64 / (2.0 * input_rate as f64) * 0.94;
    let center = (FIR_TAPS - 1) as f64 / 2.0;
    let mut coefficients = Vec::with_capacity(FIR_TAPS);
    for index in 0..FIR_TAPS {
        let offset = index as f64 - center;
        let sinc = if offset.abs() < 1e-12 {
            2.0 * cutoff
        } else {
            (2.0 * PI * cutoff * offset).sin() / (PI * offset)
        };
        let window = 0.42 - 0.5 * (2.0 * PI * index as f64 / (FIR_TAPS - 1) as f64).cos()
            + 0.08 * (4.0 * PI * index as f64 / (FIR_TAPS - 1) as f64).cos();
        coefficients.push((sinc * window) as f32);
    }
    let gain: f32 = coefficients.iter().sum();
    if gain.abs() > f32::EPSILON {
        for coefficient in &mut coefficients {
            *coefficient /= gain;
        }
    }
    coefficients
}

pub fn resample_to_16k_mono(
    raw_samples: &[f32],
    channels: u16,
    input_sample_rate: u32,
) -> Result<Vec<f32>, String> {
    if raw_samples.is_empty() {
        return Ok(Vec::new());
    }
    let mut resampler = StreamingResampler::new(channels, input_sample_rate)?;
    let expected = raw_samples.len() * OUTPUT_SAMPLE_RATE as usize
        / (input_sample_rate as usize * channels as usize).max(1);
    let mut output = Vec::with_capacity(expected + 2);
    resampler.process_interleaved(raw_samples, &mut output);
    resampler.finish(&mut output);
    Ok(output)
}

pub fn process_and_write_wav(
    raw_samples: &[f32],
    channels: u16,
    input_sample_rate: u32,
    output_path: &str,
) -> Result<(), String> {
    if raw_samples.is_empty() {
        return Err("No audio samples were recorded".to_string());
    }
    let resampled = resample_to_16k_mono(raw_samples, channels, input_sample_rate)?;
    write_16k_mono_wav(&resampled, Path::new(output_path))
}

fn write_16k_mono_wav(samples: &[f32], output_path: &Path) -> Result<(), String> {
    if samples.is_empty() {
        return Err("No audio samples were recorded".to_string());
    }
    if let Some(parent) = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;
    }

    let mut writer = hound::WavWriter::create(output_path, wav_spec())
        .map_err(|e| format!("Failed to create WAV writer: {e}"))?;
    for &sample in samples {
        writer
            .write_sample(sample_to_pcm_i16(sample))
            .map_err(|e| format!("Failed to write WAV sample: {e}"))?;
    }
    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_wav_path(label: &str) -> PathBuf {
        let index = TEST_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "aura-audio-recorder-{label}-{}-{index}.wav",
            std::process::id()
        ))
    }

    fn remove_test_wav(path: &Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(pending_wav_path(path));
    }

    #[test]
    fn sweep_stale_audio_removes_only_aura_temp_files() {
        let dir = std::env::temp_dir().join(format!(
            "aura-sweep-test-{}-{}",
            std::process::id(),
            TEST_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let keep = dir.join("other.wav");
        let recording = dir.join("aura-rec-1-2.wav");
        let chunk = dir.join("aura-chunk-3-4.wav.partial");
        let subdir = dir.join("aura-rec-directory");
        std::fs::write(&keep, b"keep").unwrap();
        std::fs::write(&recording, b"recording").unwrap();
        std::fs::write(&chunk, b"chunk").unwrap();
        std::fs::create_dir_all(&subdir).unwrap();

        let removed = sweep_stale_audio(&dir);

        assert_eq!(removed, 2);
        assert!(keep.exists(), "unrelated WAV files must survive");
        assert!(!recording.exists());
        assert!(!chunk.exists());
        assert!(subdir.exists(), "directories must never be removed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resampler_preserves_phase_across_chunks() {
        let input_rate = 48_000;
        let raw: Vec<f32> = (0..48_000)
            .flat_map(|index| {
                let value =
                    (index as f32 * 440.0 * 2.0 * std::f32::consts::PI / input_rate as f32).sin();
                [value, value]
            })
            .collect();
        let one_shot = resample_to_16k_mono(&raw, 2, input_rate).expect("one-shot resample");

        let mut streaming = StreamingResampler::new(2, input_rate).expect("streaming resampler");
        let mut chunked = Vec::new();
        for chunk in raw.chunks(1_997) {
            streaming.process_interleaved(chunk, &mut chunked);
        }
        streaming.finish(&mut chunked);

        assert_eq!(one_shot.len(), 16_000);
        assert_eq!(chunked, one_shot);
    }

    #[test]
    fn resampler_handles_non_integer_ratio_44_1k() {
        // 44100 / 16000 is not an integer ratio; a second of audio must still
        // yield exactly 16k samples and the streaming path must match the
        // one-shot result regardless of chunk boundaries.
        let input_rate = 44_100;
        let raw: Vec<f32> = (0..input_rate)
            .map(|index| {
                (index as f32 * 440.0 * 2.0 * std::f32::consts::PI / input_rate as f32).sin()
            })
            .collect();
        let one_shot = resample_to_16k_mono(&raw, 1, input_rate).expect("one-shot resample");
        // A non-integer ratio may emit one trailing sample at the tail boundary
        // (44100/16000 = 2.75625), so the length is allowed to differ by one.
        assert!((one_shot.len() as i64 - 16_000).abs() <= 1);

        let mut streaming = StreamingResampler::new(1, input_rate).expect("streaming resampler");
        let mut chunked = Vec::new();
        for chunk in raw.chunks(997) {
            streaming.process_interleaved(chunk, &mut chunked);
        }
        streaming.finish(&mut chunked);

        assert!((chunked.len() as i64 - 16_000).abs() <= 1);
        assert_eq!(chunked, one_shot);
    }

    #[test]
    fn rejects_invalid_audio_format() {
        assert!(resample_to_16k_mono(&[0.1], 0, 48_000).is_err());
        assert!(resample_to_16k_mono(&[0.1], 1, 0).is_err());
    }

    #[test]
    fn sanitizes_non_finite_samples() {
        let output = resample_to_16k_mono(&[f32::NAN, f32::INFINITY, -f32::INFINITY], 1, 16_000)
            .expect("valid format");
        assert_eq!(output, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn poisoned_recording_buffer_still_records_runtime_error() {
        let shared = Arc::new(Mutex::new(RecordingBuffer::default()));
        // Simulate a worker panic that happened while the guard was held.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = shared.lock().expect("lock the buffer");
            panic!("simulated panic while holding the buffer lock");
        }));

        remember_runtime_error(
            &shared,
            "Audio recorder worker panicked: simulated panic while holding the buffer lock",
        );

        let buffer = shared.lock().expect("buffer is recoverable after poison");
        assert!(buffer
            .runtime_error
            .as_deref()
            .unwrap_or_default()
            .contains("simulated panic"));
    }

    #[test]
    fn sample_stream_is_bounded_without_losing_final_recording() {
        let (publisher, sample_stream) = sample_stream_channel(1);
        let shared = Arc::new(Mutex::new(RecordingBuffer::default()));

        retain_and_publish_samples(vec![0.1, 0.2], &shared, Some(&publisher))
            .expect("publish first chunk");
        retain_and_publish_samples(vec![0.3, 0.4], &shared, Some(&publisher))
            .expect("publish overflowing chunk");

        assert_eq!(
            sample_stream.try_recv().expect("queued chunk"),
            vec![0.1, 0.2]
        );
        assert_eq!(sample_stream.dropped_chunks(), 1);
        assert_eq!(sample_stream.queued_chunks(), 0);
        assert_eq!(
            shared.lock().expect("recording buffer").samples_16k,
            vec![0.1, 0.2, 0.3, 0.4]
        );
    }

    #[test]
    fn disconnected_sample_stream_does_not_fail_recording() {
        let (publisher, sample_stream) = sample_stream_channel(1);
        drop(sample_stream);
        let shared = Arc::new(Mutex::new(RecordingBuffer::default()));

        retain_and_publish_samples(vec![0.25], &shared, Some(&publisher))
            .expect("recording continues without stream consumer");

        assert_eq!(publisher.dropped_chunks.load(Ordering::Acquire), 0);
        assert_eq!(publisher.queued_chunks.load(Ordering::Acquire), 0);
        assert_eq!(
            shared.lock().expect("recording buffer").samples_16k,
            vec![0.25]
        );
    }

    #[test]
    fn sample_stream_queue_depth_tracks_enqueue_and_drain() {
        let (publisher, sample_stream) = sample_stream_channel(2);

        publisher.publish(vec![0.1]);
        publisher.publish(vec![0.2]);
        assert_eq!(sample_stream.queued_chunks(), 2);
        assert_eq!(sample_stream.capacity(), 2);

        assert_eq!(sample_stream.try_recv().expect("first chunk"), vec![0.1]);
        assert_eq!(sample_stream.queued_chunks(), 1);
        assert_eq!(sample_stream.try_recv().expect("second chunk"), vec![0.2]);
        assert_eq!(sample_stream.queued_chunks(), 0);
    }

    #[test]
    fn incremental_wav_is_published_only_after_commit() {
        let path = test_wav_path("incremental-commit");
        remove_test_wav(&path);
        let partial_path = pending_wav_path(&path);
        let mut pending = PendingWavFile::create(path.clone()).expect("create pending WAV");

        pending
            .write_samples(&[0.25, f32::NAN])
            .expect("write first chunk");
        pending
            .write_samples(&[-0.5, f32::INFINITY])
            .expect("write second chunk");
        assert!(partial_path.exists());
        assert!(!path.exists());

        pending.commit().expect("commit pending WAV");
        assert!(path.exists());
        assert!(!partial_path.exists());

        let mut reader = hound::WavReader::open(&path).expect("open committed WAV");
        assert_eq!(reader.spec(), wav_spec());
        assert_eq!(reader.duration(), 4);
        let samples = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .expect("read committed samples");
        assert_eq!(
            samples,
            vec![sample_to_pcm_i16(0.25), 0, sample_to_pcm_i16(-0.5), 0]
        );
        remove_test_wav(&path);
    }

    #[test]
    fn dropping_pending_wav_cleans_partial_file() {
        let path = test_wav_path("incremental-cancel");
        remove_test_wav(&path);
        let partial_path = pending_wav_path(&path);
        {
            let mut pending = PendingWavFile::create(path.clone()).expect("create pending WAV");
            pending.write_samples(&[0.1, 0.2]).expect("write samples");
            assert!(partial_path.exists());
        }

        assert!(!partial_path.exists());
        assert!(!path.exists());
    }

    #[test]
    fn empty_pending_wav_fails_commit_and_cleans_partial_file() {
        let path = test_wav_path("incremental-empty");
        remove_test_wav(&path);
        let partial_path = pending_wav_path(&path);
        let pending = PendingWavFile::create(path.clone()).expect("create pending WAV");

        assert!(pending.commit().is_err());
        assert!(!partial_path.exists());
        assert!(!path.exists());
    }

    #[test]
    fn parakeet_mode_does_not_retain_ten_minutes_of_samples() {
        let mut bounded = RecordingBuffer::new(false);
        let chunk = vec![0.0; OUTPUT_SAMPLE_RATE as usize / 4];
        for _ in 0..10 * 60 * 4 {
            bounded.append_samples(&chunk);
        }
        assert!(bounded.samples_16k.is_empty());
        assert_eq!(bounded.samples_16k.capacity(), 0);

        let mut legacy_preview = RecordingBuffer::new(true);
        legacy_preview.append_samples(&chunk);
        assert_eq!(legacy_preview.samples_16k, chunk);
    }

    #[test]
    fn writes_16k_mono_wav() {
        let path = test_wav_path("batch");
        remove_test_wav(&path);
        let raw = vec![0.25f32; 48_000];
        process_and_write_wav(&raw, 1, 48_000, &path.to_string_lossy()).expect("write test WAV");

        let reader = hound::WavReader::open(&path).expect("open test WAV");
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(reader.spec().sample_rate, OUTPUT_SAMPLE_RATE);
        assert_eq!(reader.duration(), OUTPUT_SAMPLE_RATE);
        remove_test_wav(&path);
    }

    #[test]
    fn test_i32_sample_conversion() {
        let i32_samples: [i32; 5] = [0, i32::MAX, i32::MIN, 1073741824, -1073741824];
        let f32_samples: Vec<f32> = i32_samples
            .iter()
            .map(|&s| s as f32 / 2147483648.0)
            .collect();
        assert_eq!(f32_samples[0], 0.0);
        assert!((f32_samples[1] - 1.0).abs() < 1e-6);
        assert_eq!(f32_samples[2], -1.0);
        assert!((f32_samples[3] - 0.5).abs() < 1e-6);
        assert!((f32_samples[4] - -0.5).abs() < 1e-6);
    }

    static MIC_METER_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(target_os = "windows")]
    fn file_dacl_ace_count(path: &Path) -> usize {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
        use windows_sys::Win32::Security::{
            AclSizeInformation, GetAclInformation, ACL_SIZE_INFORMATION, DACL_SECURITY_INFORMATION,
        };
        unsafe {
            let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            let mut dacl: *mut windows_sys::Win32::Security::ACL = std::ptr::null_mut();
            let mut descriptor: *mut std::ffi::c_void = std::ptr::null_mut();
            let result = GetNamedSecurityInfoW(
                path_wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut dacl,
                std::ptr::null_mut(),
                &mut descriptor,
            );
            if result != 0 || dacl.is_null() {
                if !descriptor.is_null() {
                    LocalFree(descriptor);
                }
                return usize::MAX;
            }
            let mut size_info: ACL_SIZE_INFORMATION = std::mem::zeroed();
            let ok = GetAclInformation(
                dacl,
                &mut size_info as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            );
            if !descriptor.is_null() {
                LocalFree(descriptor);
            }
            if ok == 0 {
                return usize::MAX;
            }
            size_info.AceCount as usize
        }
    }

    fn speech_chirp_16k() -> Vec<f32> {
        let sr = OUTPUT_SAMPLE_RATE as usize;
        (0..sr)
            .map(|i| {
                let f = 120.0 + 900.0 * (i as f32 / sr as f32);
                let t = i as f32 / sr as f32;
                let env = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * 4.0 * t).cos());
                (2.0 * std::f32::consts::PI * f * t).sin() * 0.8 * env
            })
            .collect()
    }

    #[test]
    fn adaptive_tail_hold_respects_toggle_and_hold_bounds() {
        // Toggle mode: 90ms drain regardless of trailing speech state.
        assert!(should_continue_tail_hold(
            true,
            Duration::from_millis(89),
            16_000,
            Some(15_900)
        ));
        assert!(!should_continue_tail_hold(
            true,
            Duration::from_millis(90),
            16_000,
            Some(15_900)
        ));

        // Hold mode: always drains at least 100ms, then exits immediately when >= 180ms (2,880 samples) are silent.
        assert!(should_continue_tail_hold(
            false,
            Duration::from_millis(99),
            16_000,
            Some(10_000)
        ));
        assert!(!should_continue_tail_hold(
            false,
            Duration::from_millis(100),
            16_000,
            Some(16_000 - HOLD_TRAILING_SILENCE_SAMPLES)
        ));

        // Hold mode with active speech right before key release keeps draining past 100ms up to 250ms ceiling.
        assert!(should_continue_tail_hold(
            false,
            Duration::from_millis(150),
            16_000,
            Some(15_000)
        ));
        assert!(!should_continue_tail_hold(
            false,
            Duration::from_millis(250),
            16_000,
            Some(15_999)
        ));

        // Hold mode with pure silence exits right at 100ms.
        assert!(!should_continue_tail_hold(
            false,
            Duration::from_millis(100),
            8_000,
            None
        ));
    }

    #[test]
    fn streaming_vad_commit_trims_trailing_silence_before_publish_and_preserves_dacl() {
        let speech_path = test_wav_path("streaming-vad-trim");
        remove_test_wav(&speech_path);

        let mut samples = speech_chirp_16k();
        let speech_len = samples.len() as u64;
        // Append 2.5 seconds of trailing silence (40,000 samples)
        samples.resize(samples.len() + 40_000, 0.0);

        let mut pending = PendingWavFile::create(speech_path.clone()).expect("create pending WAV");
        let mut vad = crate::vad::StreamingVad::new_16k().expect("build streaming VAD");
        for chunk in samples.chunks(320) {
            vad.observe_samples(chunk);
            pending.write_samples(chunk).expect("write chunk");
        }

        let outcome =
            finalize_session_wav_with_vad(pending, Some(&mut vad)).expect("finalize with VAD");
        assert_eq!(outcome, Some(true));
        assert!(speech_path.exists());

        #[cfg(target_os = "windows")]
        assert_eq!(
            file_dacl_ace_count(&speech_path),
            2,
            "committed trimmed WAV must have restrictive 2-ACE DACL"
        );

        let duration = {
            let reader = hound::WavReader::open(&speech_path).expect("open finalized WAV");
            assert_eq!(reader.spec(), wav_spec());
            u64::from(reader.duration())
        };
        remove_test_wav(&speech_path);

        let max_expected = speech_len + crate::vad::TRAILING_SILENCE_PADDING_SAMPLES + 512;
        assert!(
            duration <= max_expected,
            "trailing silence beyond 600ms padding must be truncated before commit (got {duration}, max {max_expected})"
        );
        assert!(duration >= crate::vad::TRAILING_SILENCE_PADDING_SAMPLES);

        // Pure silence session returns Some(false)
        let silent_path = test_wav_path("streaming-vad-silence");
        remove_test_wav(&silent_path);
        let mut silent_pending =
            PendingWavFile::create(silent_path.clone()).expect("create silent pending WAV");
        let mut silent_vad = crate::vad::StreamingVad::new_16k().expect("build silent VAD");
        let silence = vec![0.0f32; 16_000];
        silent_vad.observe_samples(&silence);
        silent_pending
            .write_samples(&silence)
            .expect("write silence");
        let silent_outcome = finalize_session_wav_with_vad(silent_pending, Some(&mut silent_vad))
            .expect("finalize silence");
        assert_eq!(silent_outcome, Some(false));
        remove_test_wav(&silent_path);
    }

    #[test]
    fn resampler_evaluates_fir_only_at_output_points() {
        let mut resampler = StreamingResampler::new(2, 48_000).expect("48k stereo resampler");
        let raw = vec![0.25f32; 48_000 * 2]; // 1 second of 48 kHz stereo (48,000 frames)
        let mut out = Vec::with_capacity(16_002);
        resampler.process_interleaved(&raw, &mut out);
        resampler.finish(&mut out);

        assert_eq!(out.len(), 16_000);
        assert_eq!(
            resampler.fir_evaluations, 16_000,
            "48 kHz -> 16 kHz resampler must evaluate FIR once per 16 kHz output sample, not 48,000 times"
        );
    }

    #[test]
    fn test_clear_active_meter_matches_only_own_handle() {
        let _lock = MIC_METER_TEST_LOCK.lock().expect("mic meter test lock");
        let stop_1 = Arc::new(AtomicBool::new(false));
        let stop_2 = Arc::new(AtomicBool::new(false));

        let mutex = ACTIVE_MIC_METER.get_or_init(|| Mutex::new(None));
        {
            let mut guard = mutex.lock().unwrap();
            *guard = Some(ActiveMicMeter {
                stop: Arc::clone(&stop_2),
                handle: None,
            });
        }

        // Thread 1 exits and tries to clear with stop_1 - should NOT clear stop_2!
        clear_active_meter_if_matches(&stop_1);
        {
            let guard = mutex.lock().unwrap();
            assert!(guard.is_some());
            assert!(Arc::ptr_eq(&guard.as_ref().unwrap().stop, &stop_2));
        }

        // Thread 2 exits and clears with stop_2 - should clear!
        clear_active_meter_if_matches(&stop_2);
        {
            let guard = mutex.lock().unwrap();
            assert!(guard.is_none());
        }
    }

    #[test]
    fn stop_mic_meter_joins_worker_thread_before_returning() {
        let _lock = MIC_METER_TEST_LOCK.lock().expect("mic meter test lock");
        let stop = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread_finished = Arc::clone(&finished);

        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(5));
            }
            thread::sleep(Duration::from_millis(15));
            thread_finished.store(true, Ordering::Release);
            clear_active_meter_if_matches(&thread_stop);
        });

        let mutex = ACTIVE_MIC_METER.get_or_init(|| Mutex::new(None));
        *mutex.lock().unwrap() = Some(ActiveMicMeter {
            stop,
            handle: Some(handle),
        });
        assert!(is_mic_meter_running());

        stop_mic_meter();

        assert!(
            finished.load(Ordering::Acquire),
            "stop_mic_meter must join the worker thread before returning"
        );
        assert!(!is_mic_meter_running());
    }
}
