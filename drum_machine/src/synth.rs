use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SynthParams {
    #[serde(rename = "kick")]
    Kick {
        pitch: f32,
        decay: f32,
        punch: f32,
        sub: f32,
    },
    #[serde(rename = "snare")]
    Snare {
        tone_freq: f32,
        decay: f32,
        noise_amount: f32,
        snap: f32,
    },
    #[serde(rename = "hihat")]
    HiHat {
        brightness: f32,
        decay: f32,
        amp: f32,
    },
    #[serde(rename = "clap")]
    Clap {
        brightness: f32,
        reverb: f32,
        layers: u32,
    },
    #[serde(rename = "tom")]
    Tom {
        pitch: f32,
        decay: f32,
        tone: f32,
    },
    #[serde(rename = "cymbal")]
    Cymbal {
        brightness: f32,
        decay: f32,
        shimmer: f32,
    },
    #[serde(rename = "rim")]
    Rim {
        pitch: f32,
        click: f32,
    },
    #[serde(rename = "sample")]
    Sample {
        path: String,
        amp: f32,
        pitch_shift: f32,
    },
}

pub struct Synth {
    sample_rate: u32,
}

impl Synth {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub fn render(&self, params: &SynthParams, time: f32) -> f32 {
        match params {
            SynthParams::Kick { pitch, decay, punch, sub } => {
                self.render_kick(time, *pitch, *decay, *punch, *sub)
            }
            SynthParams::Snare { tone_freq, decay, noise_amount, snap } => {
                self.render_snare(time, *tone_freq, *decay, *noise_amount, *snap)
            }
            SynthParams::HiHat { brightness, decay, amp } => {
                self.render_hihat(time, *brightness, *decay, *amp)
            }
            SynthParams::Clap { brightness, reverb, layers } => {
                self.render_clap(time, *brightness, *reverb, *layers)
            }
            SynthParams::Tom { pitch, decay, tone } => {
                self.render_tom(time, *pitch, *decay, *tone)
            }
            SynthParams::Cymbal { brightness, decay, shimmer } => {
                self.render_cymbal(time, *brightness, *decay, *shimmer)
            }
            SynthParams::Rim { pitch, click } => {
                self.render_rim(time, *pitch, *click)
            }
            SynthParams::Sample { .. } => {
                // Samples are handled separately in the main loop
                0.0
            }
        }
    }

    fn render_kick(&self, t: f32, pitch: f32, decay: f32, punch: f32, sub: f32) -> f32 {
        let env = (-t * decay).exp();
        let pitch_env = pitch * (1.0 + punch * (-t * 15.0).exp());
        let phase = t * pitch_env * 2.0 * PI;
        
        // Main body
        let body = phase.sin() * env;
        
        // Sub harmonic
        let sub_osc = (phase * 0.5).sin() * env * sub;
        
        // Click
        let click = (-t * 50.0).exp() * 0.3;
        
        (body + sub_osc + click) * 0.8
    }

    fn render_snare(&self, t: f32, tone_freq: f32, decay: f32, noise_amount: f32, snap: f32) -> f32 {
        let env = (-t * decay).exp();
        
        // Tonal component (shell resonance)
        let tone = (t * tone_freq * 2.0 * PI).sin() * (1.0 - noise_amount);
        
        // Noise component (snares)
        let noise = self.noise(t * 1000.0) * noise_amount;
        
        // Snap transient
        let snap_env = (-t * 80.0).exp();
        let snap_component = self.noise(t * 5000.0) * snap_env * snap;
        
        (tone + noise + snap_component) * env * 0.6
    }

    fn render_hihat(&self, t: f32, brightness: f32, decay: f32, amp: f32) -> f32 {
        let env = (-t * decay).exp();
        
        // Multiple noise bands for metallic sound
        let n1 = self.noise(t * 3000.0 * brightness);
        let n2 = self.noise(t * 7000.0 * brightness);
        let n3 = self.noise(t * 11000.0 * brightness);
        
        (n1 + n2 * 0.7 + n3 * 0.5) * env * amp * 0.3
    }

    fn render_clap(&self, t: f32, brightness: f32, reverb: f32, layers: u32) -> f32 {
        let mut signal = 0.0;
        
        // Multiple layers with slight timing offsets (hand clap effect)
        for i in 0..layers {
            let offset = i as f32 * 0.01;
            if t >= offset {
                let layer_t = t - offset;
                let env = (-layer_t * 40.0).exp();
                signal += self.noise(layer_t * 2000.0 * brightness) * env;
            }
        }
        
        // Add reverb tail
        let reverb_env = (-t * 5.0).exp() * reverb;
        signal += self.noise(t * 8000.0) * reverb_env * 0.3;
        
        signal * 0.4 / layers as f32
    }

    fn render_tom(&self, t: f32, pitch: f32, decay: f32, tone: f32) -> f32 {
        let env = (-t * decay).exp();
        
        // Pitch drops slightly
        let pitch_env = pitch * (1.0 + 0.3 * (-t * 20.0).exp());
        
        // Fundamental
        let fundamental = (t * pitch_env * 2.0 * PI).sin();
        
        // Harmonics
        let harmonic2 = (t * pitch_env * 2.0 * 2.0 * PI).sin() * 0.3;
        let harmonic3 = (t * pitch_env * 3.0 * 2.0 * PI).sin() * 0.15;
        
        // Body noise
        let noise = self.noise(t * 500.0) * (1.0 - tone) * 0.2;
        
        (fundamental + harmonic2 + harmonic3 + noise) * env * 0.7
    }

    fn render_cymbal(&self, t: f32, brightness: f32, decay: f32, shimmer: f32) -> f32 {
        let env = (-t * decay).exp();
        
        // Multiple inharmonic partials
        let mut signal = 0.0;
        let freqs = [3000.0, 5000.0, 7500.0, 9500.0, 12000.0];
        
        for (i, &freq) in freqs.iter().enumerate() {
            let amp = 1.0 / (i + 1) as f32;
            signal += (t * freq * brightness * 2.0 * PI).sin() * amp;
        }
        
        // Add shimmer modulation
        let mod_freq = 6.0;
        let modulation = 1.0 + shimmer * (t * mod_freq * 2.0 * PI).sin() * 0.3;
        
        signal * env * modulation * 0.25
    }

    fn render_rim(&self, t: f32, pitch: f32, click: f32) -> f32 {
        let env = (-t * 100.0).exp();
        
        // High pitched tone
        let tone = (t * pitch * 2.0 * PI).sin();
        
        // Sharp click
        let click_component = self.noise(t * 8000.0) * (-t * 200.0).exp() * click;
        
        (tone + click_component) * env * 0.5
    }

    fn noise(&self, seed: f32) -> f32 {
        // Simple pseudo-random noise
        (seed.sin() * 43758.5453).fract() * 2.0 - 1.0
    }
}
