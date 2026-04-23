use hound::{WavReader, WavWriter, SampleFormat};
use rustfft::{FftPlanner, num_complex::Complex};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: vocal_extract <input.wav> <output.wav>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    
    match extract_vocals(input_path, output_path) {
        Ok(_) => println!("Vocals extracted to: {}", output_path),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn extract_vocals(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(input)?;
    let spec = reader.spec();
    
    if spec.channels != 2 {
        return Err("Input must be stereo (2 channels)".into());
    }
    
    // Read stereo samples
    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => {
            reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
        }
        SampleFormat::Int => {
            reader.samples::<i32>()
                .map(|s| s.map(|v| v as f32 / i32::MAX as f32))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    
    // Separate left and right channels
    let left: Vec<f32> = samples.iter().step_by(2).copied().collect();
    let right: Vec<f32> = samples.iter().skip(1).step_by(2).copied().collect();
    
    // Extract center channel (where vocals typically are)
    // Also apply high-pass filter to remove low-end rumble
    let vocals = extract_center_with_hpf(&left, &right, spec.sample_rate);
    
    // Write mono output
    let mut writer = WavWriter::create(
        output,
        hound::WavSpec {
            channels: 1,
            sample_rate: spec.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        },
    )?;
    
    for &sample in &vocals {
        writer.write_sample(sample)?;
    }
    
    writer.finalize()?;
    Ok(())
}

fn extract_center_with_hpf(left: &[f32], right: &[f32], sample_rate: u32) -> Vec<f32> {
    // Center extraction: (L + R) / 2 for center, (L - R) / 2 for sides
    // Vocals are typically centered in the mix
    let mut center: Vec<f32> = left
        .iter()
        .zip(right.iter())
        .map(|(&l, &r)| (l + r) / 2.0)
        .collect();
    
    // Apply high-pass filter at 80 Hz to remove bass/kick
    high_pass_filter(&mut center, sample_rate, 80.0);
    
    // Apply spectral gating to reduce non-vocal content
    spectral_gate(&mut center, sample_rate);
    
    center
}

fn high_pass_filter(samples: &mut [f32], sample_rate: u32, cutoff: f32) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
    let dt = 1.0 / sample_rate as f32;
    let alpha = rc / (rc + dt);
    
    let mut prev_input = 0.0;
    let mut prev_output = 0.0;
    
    for sample in samples.iter_mut() {
        let input = *sample;
        *sample = alpha * (prev_output + input - prev_input);
        prev_input = input;
        prev_output = *sample;
    }
}

fn spectral_gate(samples: &mut [f32], sample_rate: u32) {
    let window_size = 2048;
    let hop_size = 512;
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    let ifft = planner.plan_fft_inverse(window_size);
    
    let mut output = vec![0.0f32; samples.len()];
    
    for window_start in (0..samples.len().saturating_sub(window_size)).step_by(hop_size) {
        let window = &samples[window_start..window_start + window_size];
        
        // FFT
        let mut buffer: Vec<Complex<f32>> = window
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();
        
        fft.process(&mut buffer);
        
        // Gate: Keep frequencies typical for vocals (200 Hz - 5000 Hz)
        for (i, bin) in buffer.iter_mut().enumerate() {
            let freq = i as f32 * sample_rate as f32 / window_size as f32;
            
            if freq < 200.0 || freq > 5000.0 {
                *bin *= 0.1; // Reduce non-vocal frequencies
            }
        }
        
        // IFFT
        ifft.process(&mut buffer);
        
        // Overlap-add
        for (i, &Complex { re, .. }) in buffer.iter().enumerate() {
            output[window_start + i] += re / window_size as f32;
        }
    }
    
    samples.copy_from_slice(&output);
}
