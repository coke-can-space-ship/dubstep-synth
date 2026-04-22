use hound;
use std::f32::consts::PI;
use std::i16;

fn load_sample(filename: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(filename).unwrap();
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();
    
    let spec = reader.spec();
    if spec.channels == 2 {
        samples
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        samples
    }
}

fn generate_wobble_bass(sample_rate: u32, duration: f32, root_freq: f32, wobble_rate: f32) -> Vec<f32> {
    let samples = (sample_rate as f32 * duration) as usize;
    let mut bass = Vec::with_capacity(samples);
    
    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        
        // Sawtooth wave
        let phase = (root_freq * t) % 1.0;
        let saw = 2.0 * phase - 1.0;
        
        // LFO for wobble (triangle wave)
        let lfo_phase = (wobble_rate * t) % 1.0;
        let lfo = if lfo_phase < 0.5 {
            4.0 * lfo_phase - 1.0
        } else {
            3.0 - 4.0 * lfo_phase
        };
        
        // Map LFO to cutoff frequency
        let cutoff = 200.0 + (lfo + 1.0) * 900.0;
        
        // Simple lowpass (resonant filter approximation)
        let q = 5.0;
        let omega = 2.0 * PI * cutoff / sample_rate as f32;
        let filter_mod = (omega * q).sin();
        
        // Apply envelope
        let envelope = if t < 0.01 {
            t / 0.01
        } else if t > duration - 0.05 {
            (duration - t) / 0.05
        } else {
            1.0
        };
        
        let sample = saw * filter_mod * envelope * 0.6;
        bass.push(sample);
    }
    bass
}

fn generate_hihat(sample_rate: u32, duration: f32, bright: bool) -> Vec<f32> {
    let samples = (sample_rate as f32 * duration) as usize;
    let mut hihat = Vec::with_capacity(samples);
    let mut noise_phase = if bright { 0.12345f32 } else { 0.54321f32 };
    
    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        
        // Brighter = faster decay, sharper attack
        let envelope = if bright {
            (-t * 60.0).exp()
        } else {
            (-t * 40.0).exp()
        };
        
        noise_phase = (noise_phase * 9876.5).fract();
        let noise = (noise_phase - 0.5) * 2.0;
        
        // Brighter = more aggressive highpass
        let highpass = if bright {
            noise - (noise * 0.15) // Less low-end
        } else {
            noise - (noise * 0.3)
        };
        
        let sample = highpass * envelope * 0.4;
        hihat.push(sample);
    }
    hihat
}

fn generate_clap(sample_rate: u32) -> Vec<f32> {
    let duration = 0.15;
    let samples = (sample_rate as f32 * duration) as usize;
    let mut clap = Vec::with_capacity(samples);
    let mut noise_phase = 0.98765f32;
    
    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        
        // Multi-hit envelope (3 quick hits)
        let hit1 = (-((t - 0.0) * 80.0).powi(2)).exp();
        let hit2 = (-((t - 0.02) * 80.0).powi(2)).exp();
        let hit3 = (-((t - 0.04) * 80.0).powi(2)).exp();
        let envelope = (hit1 + hit2 + hit3) * (-t * 10.0).exp();
        
        noise_phase = (noise_phase * 7654.3).fract();
        let noise = (noise_phase - 0.5) * 2.0;
        
        // Bandpass for body
        let sample = noise * envelope * 0.5;
        clap.push(sample);
    }
    clap
}

fn generate_sub_kick(sample_rate: u32) -> Vec<f32> {
    let duration = 0.4;
    let samples = (sample_rate as f32 * duration) as usize;
    let mut kick = Vec::with_capacity(samples);
    
    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        
        // Deep sine wave with pitch drop
        let freq = 50.0 * (-t * 8.0).exp();
        let phase = 2.0 * PI * freq * t;
        let sine = phase.sin();
        
        // Punchy envelope
        let envelope = (-t * 12.0).exp();
        
        kick.push(sine * envelope * 0.7);
    }
    kick
}

fn generate_snare_roll(sample_rate: u32, duration: f32, snare_sample: &[f32]) -> Vec<f32> {
    let total_samples = (sample_rate as f32 * duration) as usize;
    let mut buffer = vec![0.0f32; total_samples];
    
    // 16th note snare hits
    let sixteenth_duration = duration / 16.0;
    for i in 0..16 {
        let start = (i as f32 * sixteenth_duration * sample_rate as f32) as usize;
        for (j, &sample) in snare_sample.iter().enumerate() {
            if start + j < total_samples {
                buffer[start + j] += sample * 0.7; // Quieter for roll
            }
        }
    }
    buffer
}

fn main() {
    println!("Loading drum samples...");
    let kick = load_sample("kick.wav");
    let snare = load_sample("snare.wav");
    let hihat_closed = generate_hihat(44100, 0.05, true);  // Bright/crisp
    let hihat_open = generate_hihat(44100, 0.15, false);   // Darker/longer
    let clap = generate_clap(44100);
    let sub_kick = generate_sub_kick(44100);
    
    let sample_rate = 44100;
    let bpm = 140.0;
    let beat_duration = 60.0 / bpm;
    let bar_duration = beat_duration * 4.0;
    
    // 40 seconds ≈ 23 bars at 140 BPM
    let num_bars = 24;
    let total_duration = bar_duration * num_bars as f32;
    let total_samples = (sample_rate as f32 * total_duration) as usize;
    
    // STEREO buffer (2 channels)
    let mut buffer_left = vec![0.0f32; total_samples];
    let mut buffer_right = vec![0.0f32; total_samples];
    
    println!("Building 40-second track with structure...");
    
    // Helper functions with stereo panning
    // pan: -1.0 = hard left, 0.0 = center, 1.0 = hard right
    fn add_hit_to_buffer_stereo(buffer_left: &mut [f32], buffer_right: &mut [f32], time: f32, sample: &[f32], sample_rate: u32, pan: f32) {
        let start = (time * sample_rate as f32) as usize;
        // Constant power panning
        let pan_normalized = (pan + 1.0) / 2.0; // Convert -1..1 to 0..1
        let left_gain = ((1.0 - pan_normalized) * PI / 2.0).cos();
        let right_gain = (pan_normalized * PI / 2.0).cos();
        
        for (i, &s) in sample.iter().enumerate() {
            if start + i < buffer_left.len() {
                buffer_left[start + i] += s * left_gain;
                buffer_right[start + i] += s * right_gain;
            }
        }
    }
    
    fn add_bass_to_buffer_stereo(buffer_left: &mut [f32], buffer_right: &mut [f32], time: f32, freq: f32, duration: f32, wobble_rate: f32, sample_rate: u32) {
        let bass = generate_wobble_bass(sample_rate, duration, freq, wobble_rate);
        let start = (time * sample_rate as f32) as usize;
        // Bass stays centered
        for (i, &s) in bass.iter().enumerate() {
            if start + i < buffer_left.len() {
                buffer_left[start + i] += s;
                buffer_right[start + i] += s;
            }
        }
    }
    
    // INTRO: Bars 1-4 (minimal drums, no bass)
    println!("  Intro (bars 1-4)...");
    for bar in 0..4 {
        let offset = bar as f32 * bar_duration;
        // Kick + sub-kick layer (centered)
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        // Sparse closed hi-hats (slight left pan)
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &hihat_closed, sample_rate, -0.3);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &hihat_closed, sample_rate, -0.3);
    }
    
    // BUILDUP: Bars 5-8 (add snare, more hi-hats, introduce bass)
    println!("  Buildup (bars 5-8)...");
    for bar in 4..8 {
        let offset = bar as f32 * bar_duration;
        // Syncopated kicks with sub-kick layer (centered)
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.75 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.75 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.5 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.5 * beat_duration, &sub_kick, sample_rate, 0.0);
        // Snare (slight right) + clap (slight left) layer
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &snare, sample_rate, 0.2);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &clap, sample_rate, -0.2);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.75 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.75 * beat_duration, &sub_kick, sample_rate, 0.0);
        
        // Layered hi-hats (closed alternates L/R, open centered)
        for eighth in 0..8 {
            let pan = if eighth % 2 == 0 { -0.4 } else { 0.4 };
            add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + eighth as f32 * 0.5 * beat_duration, &hihat_closed, sample_rate, pan);
        }
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, &hihat_open, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, &hihat_open, sample_rate, 0.0);
        
        // Minimal bass (starting bar 6) - centered
        if bar >= 5 {
            add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, 41.2, 0.5 * beat_duration, 4.0, sample_rate);
            add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, 41.2, 0.5 * beat_duration, 4.0, sample_rate);
        }
    }
    
    // Snare roll into drop (bar 8, beat 4) - centered
    let roll_start = 7.0 * bar_duration + 3.0 * beat_duration;
    let roll = generate_snare_roll(sample_rate, beat_duration, &snare);
    for (i, &s) in roll.iter().enumerate() {
        let idx = (roll_start * sample_rate as f32) as usize + i;
        if idx < total_samples {
            buffer_left[idx] += s;
            buffer_right[idx] += s;
        }
    }
    
    // DROP 1: Bars 9-16 (full wobble bass, syncopated drums)
    println!("  Drop 1 (bars 9-16)...");
    for bar in 8..16 {
        let offset = bar as f32 * bar_duration;
        // Full syncopated pattern with layered kicks (centered)
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.75 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.75 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.5 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.5 * beat_duration, &sub_kick, sample_rate, 0.0);
        // Snare (right) + clap (left) layer
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &snare, sample_rate, 0.3);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &clap, sample_rate, -0.3);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.75 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.75 * beat_duration, &sub_kick, sample_rate, 0.0);
        
        // Layered hi-hats (closed alternates L/R, open centered)
        for eighth in 0..8 {
            let pan = if eighth % 2 == 0 { -0.5 } else { 0.5 };
            add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + eighth as f32 * 0.5 * beat_duration, &hihat_closed, sample_rate, pan);
        }
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, &hihat_open, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, &hihat_open, sample_rate, 0.0);
        
        // Heavy wobble bass (E1 root with variations) - centered
        let pattern = bar % 4;
        match pattern {
            0 | 1 => {
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, 41.2, 0.75 * beat_duration, 6.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.75 * beat_duration, 41.2, 0.75 * beat_duration, 6.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.5 * beat_duration, 55.0, 0.5 * beat_duration, 6.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.75 * beat_duration, 41.2, 1.25 * beat_duration, 6.0, sample_rate);
            }
            2 => {
                // Faster wobble
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, 41.2, 0.75 * beat_duration, 8.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.75 * beat_duration, 41.2, 0.75 * beat_duration, 8.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.5 * beat_duration, 46.25, 0.5 * beat_duration, 8.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.75 * beat_duration, 41.2, 1.25 * beat_duration, 8.0, sample_rate);
            }
            3 => {
                // Slower wobble
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, 41.2, 2.0 * beat_duration, 4.0, sample_rate);
                add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, 55.0, 2.0 * beat_duration, 4.0, sample_rate);
            }
            _ => {}
        }
    }
    
    // BREAKDOWN: Bars 17-20 (strip to minimal)
    println!("  Breakdown (bars 17-20)...");
    for bar in 16..20 {
        let offset = bar as f32 * bar_duration;
        // Just kick on 1 and 3 (no sub-kick, lighter) - centered
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &kick, sample_rate, 0.0);
        
        // Sparse closed hi-hats only - slight left
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &hihat_closed, sample_rate, -0.3);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, &hihat_closed, sample_rate, -0.3);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &hihat_closed, sample_rate, -0.3);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, &hihat_closed, sample_rate, -0.3);
        
        // Very minimal bass (last bar only) - centered
        if bar == 19 {
            add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, 41.2, 4.0 * beat_duration, 2.0, sample_rate);
        }
    }
    
    // DROP 2: Bars 21-24 (back to full energy with variation)
    println!("  Drop 2 (bars 21-24)...");
    for bar in 20..24 {
        let offset = bar as f32 * bar_duration;
        // Four-on-floor this time (different from drop 1) with full layers - centered
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        // Snare (right) + clap (left) on 3
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &snare, sample_rate, 0.4);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, &clap, sample_rate, -0.4);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, &kick, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, &sub_kick, sample_rate, 0.0);
        
        // Dense layered hi-hats (wide stereo field)
        for eighth in 0..8 {
            let pan = if eighth % 2 == 0 { -0.6 } else { 0.6 };
            add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + eighth as f32 * 0.5 * beat_duration, &hihat_closed, sample_rate, pan);
            // Add some 16th notes for extra density (opposite pan for width)
            if eighth % 2 == 0 {
                add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + (eighth as f32 + 0.5) * 0.5 * beat_duration, &hihat_closed, sample_rate, -pan);
            }
        }
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, &hihat_open, sample_rate, 0.0);
        add_hit_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, &hihat_open, sample_rate, 0.0);
        
        // Different bass pattern - more aggressive - centered
        add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 0.0 * beat_duration, 41.2, 1.0 * beat_duration, 8.0, sample_rate);
        add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 1.0 * beat_duration, 46.25, 1.0 * beat_duration, 8.0, sample_rate);
        add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 2.0 * beat_duration, 55.0, 1.0 * beat_duration, 8.0, sample_rate);
        add_bass_to_buffer_stereo(&mut buffer_left, &mut buffer_right, offset + 3.0 * beat_duration, 41.2, 1.0 * beat_duration, 8.0, sample_rate);
    }
    
    // Normalize stereo
    let max_left = buffer_left.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let max_right = buffer_right.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let max_val = max_left.max(max_right);
    
    if max_val > 0.0 {
        for sample in buffer_left.iter_mut() {
            *sample = (*sample / max_val) * 0.8;
        }
        for sample in buffer_right.iter_mut() {
            *sample = (*sample / max_val) * 0.8;
        }
    }
    
    // Write stereo file
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create("dubstep_full_track.wav", spec).unwrap();
    for i in 0..buffer_left.len() {
        let left_amplitude = (buffer_left[i] * i16::MAX as f32) as i16;
        let right_amplitude = (buffer_right[i] * i16::MAX as f32) as i16;
        writer.write_sample(left_amplitude).unwrap();
        writer.write_sample(right_amplitude).unwrap();
    }
    writer.finalize().unwrap();
    
    println!("Done! Created ~40-second STEREO track with full structure and panning.");
}
