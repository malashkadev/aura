use voice_activity_detector::VoiceActivityDetector;

/// Returns true if any 16 kHz mono frame in `samples` exceeds the speech probability threshold.
pub fn has_speech(samples: &[f32], sample_rate: i64) -> bool {
    let mut vad = match VoiceActivityDetector::builder()
        .sample_rate(sample_rate)
        .chunk_size(512usize)
        .build()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Aura Dev Log ERROR: Failed to build VAD in has_speech: {:?}", e);
            return true; // Fallback to true so we don't discard audio on error
        }
    };

    let chunk_size = if sample_rate == 8000 { 256 } else { 512 };
    for chunk in samples.chunks(chunk_size) {
        let prob = vad.predict(chunk.iter().copied());
        if prob > 0.4 { // Using 0.4 for a slightly higher sensitivity to capture soft starts
            return true;
        }
    }
    false
}

/// Returns samples with leading and trailing silence (non-speech) trimmed, preserving a small margin.
pub fn trim_silence(samples: &[f32], sample_rate: i64) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mut vad = match VoiceActivityDetector::builder()
        .sample_rate(sample_rate)
        .chunk_size(512usize)
        .build()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Aura Dev Log ERROR: Failed to build VAD in trim_silence: {:?}", e);
            return samples.to_vec(); // Fallback to returning original samples on error
        }
    };

    let chunk_size = if sample_rate == 8000 { 256 } else { 512 };
    let mut speech_indices = Vec::new();

    for (i, chunk) in samples.chunks(chunk_size).enumerate() {
        let prob = vad.predict(chunk.iter().copied());
        if prob > 0.4 {
            speech_indices.push(i);
        }
    }

    if speech_indices.is_empty() {
        // If no speech was detected, return the original audio to be safe and avoid erasing dictation
        return samples.to_vec();
    }

    let first_speech_chunk = speech_indices[0];
    let last_speech_chunk = speech_indices[speech_indices.len() - 1];

    // Margin: 200ms = 0.2 * sample_rate
    let margin_samples = (0.2 * sample_rate as f32) as usize;

    let start_sample = (first_speech_chunk * chunk_size).saturating_sub(margin_samples);
    let end_sample = ((last_speech_chunk + 1) * chunk_size + margin_samples).min(samples.len());

    samples[start_sample..end_sample].to_vec()
}

/// Reads a WAV file, trims the leading and trailing silence using Silero VAD, and overwrites the file.
pub fn trim_wav_file(path: &str) -> Result<(), String> {
    // 1. Read i16 samples from the WAV file
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV file for VAD trimming: {}", e))?;
    let spec = reader.spec();
    
    // Ensure the file is indeed 16kHz mono as expected
    if spec.sample_rate != 16000 || spec.channels != 1 {
        return Err(format!(
            "Unsupported WAV format for VAD: channels={}, sample_rate={}",
            spec.channels, spec.sample_rate
        ));
    }
    
    let samples_i16: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<Vec<i16>, hound::Error>>()
        .map_err(|e| format!("Failed to read WAV samples: {}", e))?;
    
    // 2. Convert to f32
    let samples_f32: Vec<f32> = samples_i16
        .iter()
        .map(|&s| s as f32 / 32768.0)
        .collect();
        
    // 3. Trim silence
    let trimmed_f32 = trim_silence(&samples_f32, 16000);
    
    // 4. Convert back to i16
    let mut trimmed_i16 = Vec::with_capacity(trimmed_f32.len());
    for &sample in &trimmed_f32 {
        let clamped = sample.clamp(-1.0, 1.0);
        let s = if clamped >= 0.0 {
            (clamped * i16::MAX as f32) as i16
        } else {
            (clamped * 32768.0) as i16
        };
        trimmed_i16.push(s);
    }
    
    // 5. Overwrite the WAV file
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| format!("Failed to create WAV writer for VAD trimming: {}", e))?;
        
    for &sample in &trimmed_i16 {
        writer
            .write_sample(sample)
            .map_err(|e| format!("Failed to write trimmed sample: {}", e))?;
    }
    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize trimmed WAV file: {}", e))?;
        
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_speech_silence() {
        let silence = vec![0.0f32; 16000]; // 1 second of silence
        assert!(!has_speech(&silence, 16000));
    }

    #[test]
    fn test_trim_silence_no_speech() {
        let silence = vec![0.0f32; 16000];
        let trimmed = trim_silence(&silence, 16000);
        assert_eq!(trimmed.len(), silence.len());
    }
}
