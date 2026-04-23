use hound::{WavReader, SampleFormat};
use rustfft::{FftPlanner, num_complex::Complex};
use std::env;
use std::f32::consts::PI;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: key_detect <audio_file.wav>");
        std::process::exit(1);
    }

    let audio_path = &args[1];
    
    match detect_key(audio_path) {
        Ok((key, confidence)) => {
            println!("Detected Key: {}", key);
            println!("Confidence: {:.2}%", confidence);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn detect_key(path: &str) -> Result<(String, f32), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();
    
    // Read samples (convert to mono if stereo)
    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => {
            reader.samples::<f32>()
                .step_by(spec.channels as usize)
                .collect::<Result<Vec<_>, _>>()?
        }
        SampleFormat::Int => {
            reader.samples::<i32>()
                .step_by(spec.channels as usize)
                .map(|s| s.map(|v| v as f32 / i32::MAX as f32))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    // Use first 30 seconds for analysis
    let analysis_samples = (spec.sample_rate * 30).min(samples.len() as u32) as usize;
    let samples = &samples[..analysis_samples];

    // Chromagram analysis
    let chroma = compute_chromagram(samples, spec.sample_rate);
    
    // Key profiles (Krumhansl-Schmuckler)
    let major_profile = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
    let minor_profile = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];
    
    let keys = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    
    let mut best_key = String::new();
    let mut best_correlation = -1.0;
    
    // Test all major and minor keys
    for shift in 0..12 {
        // Major
        let major_corr = correlate(&chroma, &major_profile, shift);
        if major_corr > best_correlation {
            best_correlation = major_corr;
            best_key = format!("{} Major", keys[shift]);
        }
        
        // Minor
        let minor_corr = correlate(&chroma, &minor_profile, shift);
        if minor_corr > best_correlation {
            best_correlation = minor_corr;
            best_key = format!("{} Minor", keys[shift]);
        }
    }
    
    let confidence = ((best_correlation + 1.0) / 2.0 * 100.0).min(100.0);
    
    Ok((best_key, confidence))
}

fn compute_chromagram(samples: &[f32], sample_rate: u32) -> [f32; 12] {
    let mut chroma = [0.0f32; 12];
    let window_size = 4096;
    let hop_size = 2048;
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    
    for window_start in (0..samples.len().saturating_sub(window_size)).step_by(hop_size) {
        let window = &samples[window_start..window_start + window_size];
        
        // Apply Hann window
        let mut windowed: Vec<Complex<f32>> = window
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let hann = 0.5 * (1.0 - (2.0 * PI * i as f32 / window_size as f32).cos());
                Complex::new(s * hann, 0.0)
            })
            .collect();
        
        fft.process(&mut windowed);
        
        // Map FFT bins to pitch classes
        for (bin, &magnitude) in windowed.iter().enumerate().take(window_size / 2) {
            let freq = bin as f32 * sample_rate as f32 / window_size as f32;
            if freq < 80.0 || freq > 1000.0 { continue; }
            
            // Convert frequency to MIDI note
            let midi = 12.0 * (freq / 440.0).log2() + 69.0;
            let pitch_class = (midi.round() as usize) % 12;
            
            let mag = magnitude.norm();
            chroma[pitch_class] += mag;
        }
    }
    
    // Normalize
    let max = chroma.iter().cloned().fold(0.0f32, f32::max);
    if max > 0.0 {
        for c in &mut chroma {
            *c /= max;
        }
    }
    
    chroma
}

fn correlate(chroma: &[f32; 12], profile: &[f32; 12], shift: usize) -> f32 {
    let mut sum = 0.0;
    let mut chroma_sum = 0.0;
    let mut profile_sum = 0.0;
    
    for i in 0..12 {
        let c = chroma[(i + shift) % 12];
        let p = profile[i];
        sum += c * p;
        chroma_sum += c * c;
        profile_sum += p * p;
    }
    
    if chroma_sum == 0.0 || profile_sum == 0.0 {
        return 0.0;
    }
    
    sum / (chroma_sum.sqrt() * profile_sum.sqrt())
}
