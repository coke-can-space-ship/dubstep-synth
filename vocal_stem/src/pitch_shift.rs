use hound::{WavReader, WavWriter, SampleFormat};
use rustfft::{FftPlanner, num_complex::Complex};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: pitch_shift <input.wav> <output.wav> <semitones>");
        eprintln!("Example: pitch_shift vocals.wav vocals_shifted.wav -2");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let semitones: f32 = args[3].parse().unwrap_or_else(|_| {
        eprintln!("Invalid semitones value");
        std::process::exit(1);
    });
    
    match pitch_shift(input_path, output_path, semitones) {
        Ok(_) => println!("Pitch shifted by {} semitones: {}", semitones, output_path),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn pitch_shift(input: &str, output: &str, semitones: f32) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(input)?;
    let spec = reader.spec();
    
    // Read samples (convert to mono)
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
    
    // Pitch shift using phase vocoder
    let shifted = phase_vocoder_pitch_shift(&samples, semitones);
    
    // Write output
    let mut writer = WavWriter::create(
        output,
        hound::WavSpec {
            channels: 1,
            sample_rate: spec.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        },
    )?;
    
    for &sample in &shifted {
        writer.write_sample(sample)?;
    }
    
    writer.finalize()?;
    Ok(())
}

fn phase_vocoder_pitch_shift(samples: &[f32], semitones: f32) -> Vec<f32> {
    let pitch_ratio = 2.0f32.powf(semitones / 12.0);
    let window_size = 2048;
    let hop_analysis = 512;
    let hop_synthesis = (hop_analysis as f32 * pitch_ratio) as usize;
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    let ifft = planner.plan_fft_inverse(window_size);
    
    let mut output = vec![0.0f32; (samples.len() as f32 / pitch_ratio) as usize];
    let mut phase_accumulator = vec![0.0f32; window_size / 2 + 1];
    let mut prev_phase = vec![0.0f32; window_size / 2 + 1];
    
    let mut output_pos = 0;
    
    for window_start in (0..samples.len().saturating_sub(window_size)).step_by(hop_analysis) {
        let window = &samples[window_start..window_start + window_size];
        
        // Apply Hann window and FFT
        let mut buffer: Vec<Complex<f32>> = window
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let hann = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / window_size as f32).cos());
                Complex::new(s * hann, 0.0)
            })
            .collect();
        
        fft.process(&mut buffer);
        
        // Phase vocoder processing
        for (i, bin) in buffer.iter_mut().enumerate().take(window_size / 2 + 1) {
            let magnitude = bin.norm();
            let phase = bin.arg();
            
            // Calculate phase difference
            let phase_diff = phase - prev_phase[i];
            prev_phase[i] = phase;
            
            // Unwrap phase
            let phase_diff = phase_diff - (2.0 * std::f32::consts::PI * (phase_diff / (2.0 * std::f32::consts::PI)).round());
            
            // Calculate true frequency
            let bin_center_freq = 2.0 * std::f32::consts::PI * i as f32 / window_size as f32;
            let true_freq = bin_center_freq + phase_diff / hop_analysis as f32;
            
            // Update phase accumulator
            phase_accumulator[i] += true_freq * hop_synthesis as f32;
            
            // Reconstruct bin with new phase
            *bin = Complex::from_polar(magnitude, phase_accumulator[i]);
        }
        
        // Mirror for negative frequencies
        for i in (window_size / 2 + 1)..window_size {
            buffer[i] = buffer[window_size - i].conj();
        }
        
        // IFFT
        ifft.process(&mut buffer);
        
        // Overlap-add to output
        for (i, &Complex { re, .. }) in buffer.iter().enumerate() {
            let output_idx = output_pos + i;
            if output_idx < output.len() {
                output[output_idx] += re / window_size as f32;
            }
        }
        
        output_pos += hop_synthesis;
    }
    
    output
}
