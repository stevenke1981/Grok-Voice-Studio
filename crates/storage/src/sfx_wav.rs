use std::f32::consts::PI;

/// Generate a mono 16-bit PCM WAV at 24kHz with a simple tone + noise texture.
pub fn generate_placeholder_wav(duration_ms: u32, seed_freq: f32) -> Vec<u8> {
    let sample_rate = 24_000u32;
    let num_samples = (sample_rate as u64 * duration_ms as u64 / 1000) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let envelope = (1.0 - (t * 1000.0 / duration_ms as f32)).max(0.0);
        let tone = (t * seed_freq * 2.0 * PI).sin();
        let noise = ((i as u32).wrapping_mul(1103515245).wrapping_add(12345) % 1000) as f32 / 500.0 - 1.0;
        let mixed = (tone * 0.55 + noise * 0.45) * envelope * 0.35;
        let sample = (mixed.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        samples.push(sample);
    }

    encode_wav(&samples, sample_rate)
}

fn encode_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_size = samples.len() * 2;
    let mut wav = Vec::with_capacity(44 + data_size);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

pub fn freq_for_sfx_id(id: &str) -> f32 {
    match id {
        "rain" => 180.0,
        "thunder" => 55.0,
        "wind" => 120.0,
        "footsteps" => 320.0,
        "door_open" => 240.0,
        "door_close" => 200.0,
        "explosion" => 45.0,
        "birds" => 880.0,
        "car" => 150.0,
        "keyboard" => 1200.0,
        "notification" => 660.0,
        "heartbeat" => 72.0,
        "glass_break" => 1400.0,
        "laugh_track" => 400.0,
        "crowd" => 220.0,
        _ => 300.0,
    }
}