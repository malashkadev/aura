use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fs::File;
use std::io::BufWriter;
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

pub struct AudioRecorder {
    state: Mutex<Option<ActiveRecording>>,
    /// True while a `start_recording` call has spawned its worker but not yet
    /// claimed `state`, and briefly after a startup timeout while the
    /// abandoned worker may still be opening the device. A second start in
    /// that window would open a concurrent stream on the same device; refuse
    /// it instead (C13).
    starting: AtomicBool,
}

struct ActiveRecording {
    command_tx: mpsc::SyncSender<RecorderCommand>,
    worker: thread::JoinHandle<Result<(), String>>,
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

#[derive(Clone, Copy)]
enum RecorderCommand {
    Stop,
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

type BufferedWavWriter = hound::WavWriter<BufWriter<File>>;

/// Writes into a side file and only publishes the final WAV path after a
/// successful header update and flush. Dropping the guard removes the side file.
struct PendingWavFile {
    writer: Option<BufferedWavWriter>,
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
        let buffered = BufWriter::with_capacity(WAV_WRITE_BUFFER_BYTES, file);
        let writer = match hound::WavWriter::new(buffered, wav_spec()) {
            Ok(writer) => writer,
            Err(error) => {
                let _ = std::fs::remove_file(&partial_path);
                return Err(format!("Failed to create WAV writer: {error}"));
            }
        };
        Ok(Self {
            writer: Some(writer),
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
        for &sample in samples {
            writer
                .write_sample(sample_to_pcm_i16(sample))
                .map_err(|e| format!("Failed to write WAV sample: {e}"))?;
        }
        self.sample_count = self.sample_count.saturating_add(samples.len() as u64);
        Ok(())
    }

    fn commit(mut self) -> Result<(), String> {
        if self.sample_count == 0 {
            return Err("No audio samples were recorded".to_string());
        }
        let writer = self
            .writer
            .take()
            .ok_or_else(|| "Pending WAV writer is already closed".to_string())?;
        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {e}"))?;

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
/// Keeping the FIR history and fractional output position across callback chunks
/// prevents discontinuities and timing drift at chunk boundaries.
struct StreamingResampler {
    channels: usize,
    input_rate: u32,
    partial_frame: Vec<f32>,
    coefficients: Vec<f32>,
    history: Vec<f32>,
    history_pos: usize,
    input_index: u64,
    output_step: f64,
    next_output_position: f64,
    previous_filtered: Option<(f64, f32)>,
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
        Ok(Self {
            channels: channels as usize,
            input_rate,
            partial_frame: Vec::with_capacity(channels as usize),
            history: vec![0.0; coefficients.len()],
            coefficients,
            history_pos: 0,
            input_index: 0,
            output_step: input_rate as f64 / OUTPUT_SAMPLE_RATE as f64,
            next_output_position: 0.0,
            previous_filtered: None,
        })
    }

    fn process_interleaved(&mut self, input: &[f32], output: &mut Vec<f32>) {
        for &sample in input {
            self.partial_frame.push(sanitize_sample(sample));
            if self.partial_frame.len() == self.channels {
                let mono = self.partial_frame.iter().copied().sum::<f32>() / self.channels as f32;
                self.partial_frame.clear();
                let filtered = self.filter_sample(mono);
                self.emit_filtered(filtered, output);
            }
        }
    }

    fn finish(&mut self, output: &mut Vec<f32>) {
        self.partial_frame.clear();

        if let Some((_, last)) = self.previous_filtered {
            let end = self.input_index as f64;
            while self.next_output_position < end - f64::EPSILON {
                output.push(last);
                self.next_output_position += self.output_step;
            }
        }
    }

    fn filter_sample(&mut self, sample: f32) -> f32 {
        if self.coefficients.len() == 1 {
            return sample;
        }

        self.history[self.history_pos] = sample;
        let len = self.history.len();
        let mut filtered = 0.0f32;
        for (lag, coefficient) in self.coefficients.iter().enumerate() {
            let index = (self.history_pos + len - (lag % len)) % len;
            filtered += coefficient * self.history[index];
        }
        self.history_pos = (self.history_pos + 1) % len;
        filtered
    }

    fn emit_filtered(&mut self, current: f32, output: &mut Vec<f32>) {
        let current_position = self.input_index as f64;
        match self.previous_filtered {
            None => {
                if self.next_output_position <= current_position + f64::EPSILON {
                    output.push(current);
                    self.next_output_position += self.output_step;
                }
            }
            Some((previous_position, previous)) => {
                while self.next_output_position <= current_position + f64::EPSILON {
                    let span = current_position - previous_position;
                    let fraction = if span > 0.0 {
                        ((self.next_output_position - previous_position) / span).clamp(0.0, 1.0)
                            as f32
                    } else {
                        1.0
                    };
                    output.push(previous + (current - previous) * fraction);
                    self.next_output_position += self.output_step;
                }
            }
        }
        self.previous_filtered = Some((current_position, current));
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
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Audio recorder state is unavailable".to_string())?;
        if state.is_some() {
            return Err("Already recording".to_string());
        }

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
        self.finish_recording(RecorderCommand::Stop)
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
            Ok(result) => result,
            Err(_) => Err(command_error
                .map(|e| format!("Audio worker stopped before receiving its command: {e}"))
                .unwrap_or_else(|| "Audio worker panicked".to_string())),
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

static ACTIVE_MIC_METER_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

fn clear_active_meter_if_matches(thread_stop: &Arc<AtomicBool>) {
    let mutex = ACTIVE_MIC_METER_STOP.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = mutex.lock() {
        if let Some(current) = guard.as_ref() {
            if Arc::ptr_eq(current, thread_stop) {
                *guard = None;
            }
        }
    }
}

pub fn is_mic_meter_running() -> bool {
    let mutex = ACTIVE_MIC_METER_STOP.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = mutex.lock() {
        guard.is_some()
    } else {
        false
    }
}

pub fn stop_mic_meter() {
    let mutex = ACTIVE_MIC_METER_STOP.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = mutex.lock() {
        if let Some(stop_flag) = guard.take() {
            stop_flag.store(true, Ordering::SeqCst);
        }
    }
}

pub fn start_mic_meter<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    device_name: Option<&str>,
) -> Result<(), String> {
    stop_mic_meter();

    let stop_flag = Arc::new(AtomicBool::new(false));
    {
        let mutex = ACTIVE_MIC_METER_STOP.get_or_init(|| Mutex::new(None));
        let mut guard = mutex
            .lock()
            .map_err(|_| "Failed to lock mic meter state".to_string())?;
        *guard = Some(Arc::clone(&stop_flag));
    }

    let dev_name = device_name.map(str::to_string);
    let thread_stop = Arc::clone(&stop_flag);
    std::thread::Builder::new()
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
            let sample_rate = config.sample_rate().0;
            let channels = config.channels();
            let sample_format = config.sample_format();
            let stream_config: cpal::StreamConfig = config.into();

            let (sample_tx, sample_rx) = mpsc::sync_channel::<Vec<f32>>(64);
            let stream = match sample_format {
                cpal::SampleFormat::I16 => device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _| {
                        let samples = data.iter().map(|&s| s as f32 / 32768.0).collect();
                        let _ = sample_tx.try_send(samples);
                    },
                    |_| {},
                    None,
                ),
                cpal::SampleFormat::U16 => device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _| {
                        let samples = data
                            .iter()
                            .map(|&s| (s as f32 - 32768.0) / 32768.0)
                            .collect();
                        let _ = sample_tx.try_send(samples);
                    },
                    |_| {},
                    None,
                ),
                cpal::SampleFormat::I32 => device.build_input_stream(
                    &stream_config,
                    move |data: &[i32], _| {
                        let samples = data.iter().map(|&s| s as f32 / 2147483648.0).collect();
                        let _ = sample_tx.try_send(samples);
                    },
                    |_| {},
                    None,
                ),
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        let samples = data.to_vec();
                        let _ = sample_tx.try_send(samples);
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

            let mut resampler = match StreamingResampler::new(channels, sample_rate) {
                Ok(r) => r,
                Err(_) => {
                    clear_active_meter_if_matches(&thread_stop);
                    return;
                }
            };
            let mut vad = crate::vad::StreamingVad::new_16k().ok();
            let mut last_emit = std::time::Instant::now();
            let mut accumulated_samples = Vec::with_capacity(1024);

            while !thread_stop.load(Ordering::Relaxed) {
                match sample_rx.recv_timeout(Duration::from_millis(15)) {
                    Ok(raw_chunk) => {
                        let mut resampled_chunk = Vec::new();
                        resampler.process_interleaved(&raw_chunk, &mut resampled_chunk);
                        accumulated_samples.extend_from_slice(&resampled_chunk);

                        if (accumulated_samples.len() >= 512
                            || last_emit.elapsed() >= Duration::from_millis(20))
                            && !accumulated_samples.is_empty()
                        {
                            let sum_sq: f32 = accumulated_samples.iter().map(|&s| s * s).sum();
                            let rms = (sum_sq / accumulated_samples.len() as f32).sqrt();
                            let volume = (rms * 3.5).clamp(0.0, 1.0);

                            let is_speech = if let Some(vad_ref) = vad.as_mut() {
                                vad_ref.push_with_threshold(&accumulated_samples, 0.25)
                            } else {
                                volume > 0.05
                            };

                            let _ = app_handle
                                .emit("mic-meter-level", MicMeterPayload { volume, is_speech });

                            accumulated_samples.clear();
                            last_emit = std::time::Instant::now();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }

            clear_active_meter_if_matches(&thread_stop);
        })
        .map_err(|e| format!("Failed to spawn mic meter thread: {e}"))?;

    Ok(())
}

fn recorder_worker(
    output_path: PathBuf,
    command_rx: mpsc::Receiver<RecorderCommand>,
    ready_tx: mpsc::SyncSender<Result<(), String>>,
    shared: Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<SampleStreamPublisher>,
    device_name: Option<String>,
    on_volume: Box<dyn Fn(f32) + Send>,
) -> Result<(), String> {
    let (stream, sample_rx, error_rx, dropped_chunks, channels, sample_rate) =
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
            &shared,
            sample_stream.as_ref(),
            &mut resampler,
            &mut wav_writer,
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

    if matches!(outcome, Ok(RecorderCommand::Stop)) {
        // Post-Release Audio Grace Buffer (Tail Hold):
        // Allow hardware/WASAPI buffers and human speech decay to capture trailing
        // phonemes and final words for 450ms before severing the input stream.
        let grace_deadline = std::time::Instant::now() + Duration::from_millis(450);
        while std::time::Instant::now() < grace_deadline {
            let _ = drain_audio_queue(
                &sample_rx,
                &shared,
                sample_stream.as_ref(),
                &mut resampler,
                &mut wav_writer,
                on_volume.as_ref(),
            );
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
                    &shared,
                    sample_stream.as_ref(),
                    &mut resampler,
                    &mut wav_writer,
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

    if matches!(command, RecorderCommand::Stop) {
        let mut tail = Vec::new();
        resampler.finish(&mut tail);
        if let Err(error) =
            append_resampled_samples(tail, &shared, sample_stream.as_ref(), &mut wav_writer)
        {
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

    if matches!(command, RecorderCommand::Stop) {
        if let Err(error) = wav_writer.commit() {
            remember_runtime_error(&shared, &error);
            return Err(error);
        }
    }

    Ok(())
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
    let (error_tx, error_rx) = mpsc::channel();
    let dropped = Arc::new(AtomicU64::new(0));
    let callback_dropped = Arc::clone(&dropped);

    let stream = match sample_format {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| {
                let samples = data.iter().map(|&s| s as f32 / 32768.0).collect();
                publish_audio_chunk(samples, &sample_tx, &callback_dropped);
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| {
                let samples = data
                    .iter()
                    .map(|&s| (s as f32 - 32768.0) / 32768.0)
                    .collect();
                publish_audio_chunk(samples, &sample_tx, &callback_dropped);
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        cpal::SampleFormat::I32 => device.build_input_stream(
            &stream_config,
            move |data: &[i32], _| {
                let samples = data.iter().map(|&s| s as f32 / 2147483648.0).collect();
                publish_audio_chunk(samples, &sample_tx, &callback_dropped);
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                publish_audio_chunk(data.to_vec(), &sample_tx, &callback_dropped);
            },
            move |error| {
                let _ = error_tx.send(format!("Audio input stream failed: {error}"));
            },
            None,
        ),
        other => return Err(format!("Unsupported sample format: {other:?}")),
    }
    .map_err(|e| format!("Failed to build input stream: {e}"))?;

    Ok((stream, sample_rx, error_rx, dropped, channels, sample_rate))
}

fn publish_audio_chunk(
    mut samples: Vec<f32>,
    sender: &mpsc::SyncSender<AudioChunk>,
    dropped: &AtomicU64,
) {
    let mut sum_sq = 0.0f32;
    for sample in &mut samples {
        *sample = sanitize_sample(*sample);
        sum_sq += *sample * *sample;
    }
    let rms = if samples.is_empty() {
        0.0
    } else {
        (sum_sq / samples.len() as f32).sqrt()
    };
    if let Err(mpsc::TrySendError::Full(_)) = sender.try_send(AudioChunk { samples, rms }) {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}

fn drain_audio_queue(
    receiver: &mpsc::Receiver<AudioChunk>,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
    resampler: &mut StreamingResampler,
    wav_writer: &mut PendingWavFile,
    on_volume: &dyn Fn(f32),
) -> Result<(), String> {
    while let Ok(chunk) = receiver.try_recv() {
        append_audio_chunk(
            chunk,
            shared,
            sample_stream,
            resampler,
            wav_writer,
            on_volume,
        )?;
    }
    Ok(())
}

fn append_audio_chunk(
    chunk: AudioChunk,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
    resampler: &mut StreamingResampler,
    wav_writer: &mut PendingWavFile,
    on_volume: &dyn Fn(f32),
) -> Result<(), String> {
    let capacity = chunk.samples.len() * OUTPUT_SAMPLE_RATE as usize
        / (resampler.input_rate as usize * resampler.channels).max(1)
        + 2;
    let mut converted = Vec::with_capacity(capacity);
    resampler.process_interleaved(&chunk.samples, &mut converted);
    append_resampled_samples(converted, shared, sample_stream, wav_writer)?;
    on_volume(chunk.rms);
    Ok(())
}

fn append_resampled_samples(
    samples: Vec<f32>,
    shared: &Arc<Mutex<RecordingBuffer>>,
    sample_stream: Option<&SampleStreamPublisher>,
    wav_writer: &mut PendingWavFile,
) -> Result<(), String> {
    if samples.is_empty() {
        return Ok(());
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

    #[test]
    fn test_clear_active_meter_matches_only_own_handle() {
        let stop_1 = Arc::new(AtomicBool::new(false));
        let stop_2 = Arc::new(AtomicBool::new(false));

        let mutex = ACTIVE_MIC_METER_STOP.get_or_init(|| Mutex::new(None));
        {
            let mut guard = mutex.lock().unwrap();
            *guard = Some(Arc::clone(&stop_2));
        }

        // Thread 1 exits and tries to clear with stop_1 - should NOT clear stop_2!
        clear_active_meter_if_matches(&stop_1);
        {
            let guard = mutex.lock().unwrap();
            assert!(guard.is_some());
            assert!(Arc::ptr_eq(guard.as_ref().unwrap(), &stop_2));
        }

        // Thread 2 exits and clears with stop_2 - should clear!
        clear_active_meter_if_matches(&stop_2);
        {
            let guard = mutex.lock().unwrap();
            assert!(guard.is_none());
        }
    }
}
