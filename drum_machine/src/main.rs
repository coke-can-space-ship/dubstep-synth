use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

mod sample_loader;
mod synth;

use sample_loader::{Sample, SampleCache};
use synth::{Synth, SynthParams};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum VoiceType {
    Kick { frequency: f32, decay: f32, punch: f32 },
    Snare { frequency: f32, noise_amount: f32, decay: f32 },
    HiHat { frequency: f32, decay: f32 },
    Clap { num_claps: usize, spread: f32, decay: f32 },
    Tom { frequency: f32, decay: f32 },
    Cymbal { frequency: f32, decay: f32 },
    Rim { frequency: f32, decay: f32 },
    Sample { path: String, pitch_shift: f32 },
}

#[derive(Debug, Deserialize, Serialize)]
struct Voice {
    #[serde(flatten)]
    voice_type: VoiceType,
    gain: f32,
    pan: f32,
}

impl Voice {
    fn render(&self, sample_rate: u32, sample_cache: &SampleCache) -> Vec<f32> {
        match &self.voice_type {
            VoiceType::Kick { frequency, decay, punch } => {
                let params = SynthParams::Kick {
                    pitch: *frequency,
                    decay: *decay * 10.0,
                    punch: *punch,
                    sub: 0.3,
                };
                self.render_synth(sample_rate, &params, *decay)
            }
            VoiceType::Snare { frequency, noise_amount, decay } => {
                let params = SynthParams::Snare {
                    tone_freq: *frequency,
                    decay: *decay * 15.0,
                    noise_amount: *noise_amount,
                    snap: 0.5,
                };
                self.render_synth(sample_rate, &params, *decay)
            }
            VoiceType::HiHat { frequency, decay } => {
                let params = SynthParams::HiHat {
                    brightness: *frequency / 10000.0,
                    decay: *decay * 20.0,
                    amp: 0.5,
                };
                self.render_synth(sample_rate, &params, *decay)
            }
            VoiceType::Clap { num_claps, spread: _, decay } => {
                let params = SynthParams::Clap {
                    brightness: 0.7,
                    reverb: 0.3,
                    layers: *num_claps as u32,
                };
                self.render_synth(sample_rate, &params, *decay)
            }
            VoiceType::Tom { frequency, decay } => {
                let params = SynthParams::Tom {
                    pitch: *frequency,
                    decay: *decay * 8.0,
                    tone: 0.7,
                };
                self.render_synth(sample_rate, &params, *decay)
            }
            VoiceType::Cymbal { frequency, decay } => {
                let params = SynthParams::Cymbal {
                    brightness: *frequency / 5000.0,
                    decay: *decay * 2.0,
                    shimmer: 0.5,
                };
                self.render_synth(sample_rate, &params, *decay)
            }
            VoiceType::Rim { frequency, decay: _ } => {
                let params = SynthParams::Rim {
                    pitch: *frequency,
                    click: 0.7,
                };
                self.render_synth(sample_rate, &params, 0.05)
            }
            VoiceType::Sample { path, pitch_shift } => {
                if let Some(sample) = sample_cache.get(path) {
                    let duration = sample.data.len() as f32 / sample.sample_rate as f32;
                    let num_samples = (duration * sample_rate as f32) as usize;
                    (0..num_samples)
                        .map(|i| {
                            let t = i as f32 / sample_rate as f32;
                            sample.get_sample(t, *pitch_shift) * self.gain
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
        }
    }

    fn render_synth(&self, sample_rate: u32, params: &SynthParams, duration: f32) -> Vec<f32> {
        let synth = Synth::new(sample_rate);
        let num_samples = (sample_rate as f32 * duration) as usize;
        
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                synth.render(params, t) * self.gain
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Pattern {
    name: String,
    voice: Voice,
    steps: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Song {
    bpm: u32,
    bars: u32,
    patterns: Vec<Pattern>,
}

fn apply_reverb(samples: &[f32], sample_rate: u32, room_size: f32, decay: f32) -> Vec<f32> {
    let delay_samples = (sample_rate as f32 * room_size * 0.05) as usize;
    let mut output = samples.to_vec();
    
    for i in delay_samples..output.len() {
        output[i] += output[i - delay_samples] * decay;
    }
    
    output
}

fn apply_master_compression(samples: &[f32], threshold: f32, ratio: f32) -> Vec<f32> {
    samples.iter().map(|&sample| {
        let abs_sample = sample.abs();
        if abs_sample > threshold {
            let over = abs_sample - threshold;
            let compressed = threshold + (over / ratio);
            sample.signum() * compressed
        } else {
            sample
        }
    }).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: drum_machine <pattern.json> [reverb_room] [reverb_decay] [comp_threshold] [comp_ratio]");
        std::process::exit(1);
    }

    let pattern_file = &args[1];
    let reverb_room = args.get(2).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.3);
    let reverb_decay = args.get(3).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.15);
    let comp_threshold = args.get(4).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.7);
    let comp_ratio = args.get(5).and_then(|s| s.parse::<f32>().ok()).unwrap_or(4.0);
    let file = File::open(pattern_file).expect("Failed to open pattern file");
    let reader = BufReader::new(file);
    let song: Song = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let sample_rate = 44100;
    let beats_per_bar = 4;
    let steps_per_beat = 4;
    let steps_per_bar = beats_per_bar * steps_per_beat;
    let total_steps = song.bars * steps_per_bar;

    let seconds_per_beat = 60.0 / song.bpm as f32;
    let samples_per_step = (sample_rate as f32 * seconds_per_beat / steps_per_beat as f32) as usize;
    let total_samples = total_steps as usize * samples_per_step;

    println!("Rendering {} bars at {} BPM", song.bars, song.bpm);
    println!("Total steps: {}, samples per step: {}", total_steps, samples_per_step);
    println!("Duration: {:.2}s", total_samples as f32 / sample_rate as f32);

    // Pre-load all samples
    let mut sample_cache = SampleCache::new();
    for pattern in &song.patterns {
        if let VoiceType::Sample { path, pitch_shift: _ } = &pattern.voice.voice_type {
            println!("Loading sample: {}", path);
            sample_cache.load(path).expect("Failed to load sample");
        }
    }

    // Mix buffer (stereo)
    let mut mix_left = vec![0.0f32; total_samples];
    let mut mix_right = vec![0.0f32; total_samples];

    // Render each pattern
    for pattern in &song.patterns {
        println!("Rendering pattern: {}", pattern.name);

        for (step_idx, &trigger) in pattern.steps.iter().cycle().take(total_steps as usize).enumerate() {
            if trigger == 0 {
                continue;
            }

            let start_sample = step_idx * samples_per_step;
            let voice_samples = pattern.voice.render(sample_rate, &sample_cache);

            // Apply panning
            let pan = pattern.voice.pan;
            let left_gain = ((1.0 - pan) / 2.0_f32).sqrt();
            let right_gain = ((1.0 + pan) / 2.0_f32).sqrt();

            for (i, &sample) in voice_samples.iter().enumerate() {
                let mix_idx = start_sample + i;
                if mix_idx >= total_samples {
                    break;
                }
                mix_left[mix_idx] += sample * left_gain;
                mix_right[mix_idx] += sample * right_gain;
            }
        }
    }

    // Apply reverb to create space
    println!("Applying reverb (room: {}, decay: {})...", reverb_room, reverb_decay);
    mix_left = apply_reverb(&mix_left, sample_rate, reverb_room, reverb_decay);
    mix_right = apply_reverb(&mix_right, sample_rate, reverb_room * 1.15, reverb_decay);

    // Master compression to control peaks
    println!("Applying master compression (threshold: {}, ratio: {})...", comp_threshold, comp_ratio);
    mix_left = apply_master_compression(&mix_left, comp_threshold, comp_ratio);
    mix_right = apply_master_compression(&mix_right, comp_threshold, comp_ratio);

    // Normalize to -3dB to leave headroom
    let max_amplitude = mix_left
        .iter()
        .chain(mix_right.iter())
        .map(|&s| s.abs())
        .fold(0.0f32, f32::max);

    let target_peak = 0.707; // -3dB
    let normalize_gain = if max_amplitude > 0.0 {
        target_peak / max_amplitude
    } else {
        1.0
    };

    println!("Normalizing (peak: {:.3}, gain: {:.3})", max_amplitude, normalize_gain);

    // Interleave stereo and write
    let output_path = pattern_file.replace(".json", ".wav");
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(&output_path, spec).expect("Failed to create WAV file");

    for i in 0..total_samples {
        let left = (mix_left[i] * normalize_gain * i16::MAX as f32) as i16;
        let right = (mix_right[i] * normalize_gain * i16::MAX as f32) as i16;
        writer.write_sample(left).unwrap();
        writer.write_sample(right).unwrap();
    }

    writer.finalize().expect("Failed to finalize WAV file");
    println!("✓ Wrote {}", output_path);
}
