use hound::{WavReader, WavWriter, WavSpec};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize, Serialize)]
struct Track {
    path: String,
    start_bar: f32,
    end_bar: Option<f32>,
    gain: f32,
    #[serde(default)]
    fade_in_bars: f32,
    #[serde(default)]
    fade_out_bars: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Composition {
    bpm: f32,
    total_bars: u32,
    sample_rate: u32,
    tracks: Vec<Track>,
}

fn bars_to_samples(bars: f32, bpm: f32, sample_rate: u32) -> usize {
    let beats_per_bar = 4.0;
    let seconds = (bars * beats_per_bar * 60.0) / bpm;
    (seconds * sample_rate as f32) as usize
}

fn apply_fade(sample: f32, position: usize, fade_in: usize, fade_out_start: usize, total: usize) -> f32 {
    let mut gain = 1.0;
    
    // Fade in
    if position < fade_in && fade_in > 0 {
        gain *= position as f32 / fade_in as f32;
    }
    
    // Fade out
    if position > fade_out_start && fade_out_start < total {
        let fade_out_length = total - fade_out_start;
        let fade_pos = position - fade_out_start;
        gain *= 1.0 - (fade_pos as f32 / fade_out_length as f32);
    }
    
    sample * gain
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: track_composer <composition.json> <output.wav>");
        std::process::exit(1);
    }

    let composition_path = &args[1];
    let output_path = &args[2];

    // Load composition
    let file = File::open(composition_path).expect("Failed to open composition file");
    let reader = BufReader::new(file);
    let comp: Composition = serde_json::from_reader(reader).expect("Failed to parse JSON");

    println!("Composing {} bars at {} BPM", comp.total_bars, comp.bpm);

    // Calculate total samples
    let total_samples = bars_to_samples(comp.total_bars as f32, comp.bpm, comp.sample_rate);
    let mut output_left = vec![0.0f32; total_samples];
    let mut output_right = vec![0.0f32; total_samples];

    // Process each track
    for (idx, track) in comp.tracks.iter().enumerate() {
        println!("  Track {}: {} (gain: {}, bars: {}-{})", 
            idx + 1, 
            track.path,
            track.gain,
            track.start_bar,
            track.end_bar.unwrap_or(comp.total_bars as f32)
        );

        let mut reader = match WavReader::open(&track.path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("    Warning: Failed to open {}: {}", track.path, e);
                continue;
            }
        };

        let spec = reader.spec();
        let start_sample = bars_to_samples(track.start_bar, comp.bpm, comp.sample_rate);
        let end_bar = track.end_bar.unwrap_or(comp.total_bars as f32);
        let end_sample = bars_to_samples(end_bar, comp.bpm, comp.sample_rate).min(total_samples);
        
        let fade_in_samples = bars_to_samples(track.fade_in_bars, comp.bpm, comp.sample_rate);
        let fade_out_samples = bars_to_samples(track.fade_out_bars, comp.bpm, comp.sample_rate);
        let fade_out_start = end_sample.saturating_sub(fade_out_samples);

        // Read all samples
        let samples: Vec<i16> = reader.samples::<i16>()
            .filter_map(|s| s.ok())
            .collect();

        let track_length = samples.len() / spec.channels as usize;
        let write_length = (end_sample - start_sample).min(track_length);

        // Mix into output
        for i in 0..write_length {
            let output_pos = start_sample + i;
            if output_pos >= total_samples {
                break;
            }

            let fade_gain = apply_fade(
                track.gain,
                i,
                fade_in_samples,
                fade_out_start.saturating_sub(start_sample),
                write_length
            );

            match spec.channels {
                1 => {
                    // Mono: duplicate to both channels
                    let sample = samples[i] as f32 / 32768.0 * fade_gain;
                    output_left[output_pos] += sample;
                    output_right[output_pos] += sample;
                }
                2 => {
                    // Stereo
                    let left = samples[i * 2] as f32 / 32768.0 * fade_gain;
                    let right = samples[i * 2 + 1] as f32 / 32768.0 * fade_gain;
                    output_left[output_pos] += left;
                    output_right[output_pos] += right;
                }
                _ => {
                    eprintln!("    Warning: Unsupported channel count: {}", spec.channels);
                }
            }
        }

        println!("    Mixed {} samples", write_length);
    }

    // Find peak for normalization
    let peak = output_left.iter()
        .chain(output_right.iter())
        .map(|&s| s.abs())
        .fold(0.0f32, f32::max);

    let normalize_gain = if peak > 0.95 {
        0.95 / peak
    } else {
        1.0
    };

    if normalize_gain < 1.0 {
        println!("Normalizing: peak={:.2}, gain={:.2}", peak, normalize_gain);
    }

    // Write output
    let spec = WavSpec {
        channels: 2,
        sample_rate: comp.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output_path, spec).expect("Failed to create output file");

    for i in 0..total_samples {
        let left = (output_left[i] * normalize_gain * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let right = (output_right[i] * normalize_gain * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(left).unwrap();
        writer.write_sample(right).unwrap();
    }

    writer.finalize().expect("Failed to finalize WAV file");
    println!("✓ Wrote {} samples to {}", total_samples, output_path);
}
