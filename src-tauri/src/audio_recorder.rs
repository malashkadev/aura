use std::path::Path;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioRecorder {
    state: Mutex<Option<ActiveRecording>>,
}

struct ActiveRecording {
    stream: cpal::Stream,
    raw_samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    output_path: String,
}

unsafe impl Send for ActiveRecording {}
unsafe impl Sync for ActiveRecording {}

impl AudioRecorder {
    /// Creates a new `AudioRecorder` instance.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }

    /// Starts recording audio from the default input device to the specified output WAV path.
    pub fn start_recording(&self, output_path: &str) -> Result<(), String> {
        let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
        if state_guard.is_some() {
            return Err("Already recording".to_string());
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default input device found".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let sample_format = config.sample_format();

        let raw_samples = Arc::new(Mutex::new(Vec::new()));
        let raw_samples_clone = raw_samples.clone();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::I16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut guard) = raw_samples_clone.lock() {
                            for &sample in data {
                                guard.push(sample as f32 / 32768.0);
                            }
                        }
                    },
                    err_fn,
                    None
                )
            }
            cpal::SampleFormat::U16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut guard) = raw_samples_clone.lock() {
                            for &sample in data {
                                guard.push((sample as f32 - 32768.0) / 32768.0);
                            }
                        }
                    },
                    err_fn,
                    None
                )
            }
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut guard) = raw_samples_clone.lock() {
                            guard.extend_from_slice(data);
                        }
                    },
                    err_fn,
                    None
                )
            }
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        }.map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;

        *state_guard = Some(ActiveRecording {
            stream,
            raw_samples,
            sample_rate,
            channels,
            output_path: output_path.to_string(),
        });

        Ok(())
    }

    /// Stops the active recording, downmixes to mono, resamples to 16000Hz, and writes the WAV file.
    pub fn stop_recording(&self) -> Result<(), String> {
        let active_recording = {
            let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
            state_guard.take().ok_or_else(|| "Not recording".to_string())?
        };

        // Dropping the stream stops recording automatically.
        drop(active_recording.stream);

        let raw_samples = active_recording.raw_samples.lock()
            .map_err(|e| e.to_string())?;

        process_and_write_wav(
            &raw_samples,
            active_recording.channels,
            active_recording.sample_rate,
            &active_recording.output_path,
        )
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Downmixes raw samples to mono, resamples to 16kHz, converts to 16-bit PCM, and writes to a WAV file.
fn process_and_write_wav(
    raw_samples: &[f32],
    channels: u16,
    input_sample_rate: u32,
    output_path: &str,
) -> Result<(), String> {
    if raw_samples.is_empty() {
        return Err("No audio samples were recorded".to_string());
    }

    // 1. Downmix to mono (average all channels)
    let channels = channels as usize;
    let mut mono = Vec::with_capacity(raw_samples.len() / channels);
    for chunk in raw_samples.chunks_exact(channels) {
        let sum: f32 = chunk.iter().sum();
        mono.push(sum / channels as f32);
    }

    // 2. Resample from input_sample_rate to 16000Hz using linear interpolation
    let ratio = input_sample_rate as f64 / 16000.0;
    let output_len = (mono.len() as f64 / ratio).floor() as usize;
    let mut resampled = Vec::with_capacity(output_len);
    for j in 0..output_len {
        let t = j as f64 * ratio;
        let index = t as usize;
        let fract = (t - index as f64) as f32;
        if index + 1 < mono.len() {
            let sample = mono[index] * (1.0 - fract) + mono[index + 1] * fract;
            resampled.push(sample);
        } else if index < mono.len() {
            resampled.push(mono[index]);
        } else {
            resampled.push(0.0);
        }
    }

    // 3. Convert resampled f32 to i16 PCM, clamping to range
    let mut i16_samples = Vec::with_capacity(resampled.len());
    for &sample in &resampled {
        let clamped = sample.clamp(-1.0, 1.0);
        let s = if clamped >= 0.0 {
            (clamped * i16::MAX as f32) as i16
        } else {
            (clamped * 32768.0) as i16
        };
        i16_samples.push(s);
    }

    // 4. Ensure the parent directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories for output path: {}", e))?;
    }

    // 5. Write to WAV file using hound
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(output_path, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    for &sample in &i16_samples {
        writer.write_sample(sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer.finalize().map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_process_and_write_wav() {
        let mut temp_path = env::temp_dir();
        temp_path.push("test_audio_recorder_output.wav");
        let temp_path_str = temp_path.to_str().unwrap();

        // Remove file if left over from previous test
        let _ = fs::remove_file(temp_path_str);

        // 48000Hz stereo input, 0.5 second of 440Hz sine wave
        let sample_rate = 48000;
        let channels = 2;
        let duration_secs = 0.5;
        let num_frames = (sample_rate as f64 * duration_secs) as usize;
        let mut raw_samples = Vec::with_capacity(num_frames * channels);
        for i in 0..num_frames {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
            // Duplicate for stereo
            raw_samples.push(sample);
            raw_samples.push(sample);
        }

        let res = process_and_write_wav(&raw_samples, channels as u16, sample_rate, temp_path_str);
        assert!(res.is_ok());

        // Verify the output file exists and has correct parameters
        let reader = hound::WavReader::open(temp_path_str);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 16000);
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        // Verify duration: 16000 samples/sec * 0.5s = 8000 samples
        let samples: Vec<i16> = reader.into_samples::<i16>().map(|s| s.unwrap()).collect();
        assert!(samples.len() >= 7900 && samples.len() <= 8100);

        // Clean up
        let _ = fs::remove_file(temp_path_str);
    }
}
