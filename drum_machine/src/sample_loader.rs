use hound;
use std::path::Path;

pub struct Sample {
    pub data: Vec<f32>,
    pub sample_rate: u32,
}

impl Sample {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        
        let data: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.samples::<f32>().map(|s| s.unwrap()).collect()
            }
            hound::SampleFormat::Int => {
                match spec.bits_per_sample {
                    16 => reader.samples::<i16>()
                        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
                        .collect(),
                    24 => reader.samples::<i32>()
                        .map(|s| s.unwrap() as f32 / 8388608.0)
                        .collect(),
                    32 => reader.samples::<i32>()
                        .map(|s| s.unwrap() as f32 / i32::MAX as f32)
                        .collect(),
                    _ => return Err("Unsupported bit depth".into()),
                }
            }
        };
        
        Ok(Sample {
            data,
            sample_rate: spec.sample_rate,
        })
    }
    
    pub fn get_sample(&self, position: f32, pitch_shift: f32) -> f32 {
        let adjusted_pos = position * pitch_shift;
        let index = (adjusted_pos * self.sample_rate as f32) as usize;
        
        if index >= self.data.len() {
            return 0.0;
        }
        
        // Linear interpolation for smoother playback
        let frac = adjusted_pos * self.sample_rate as f32 - index as f32;
        let sample1 = self.data[index];
        let sample2 = if index + 1 < self.data.len() {
            self.data[index + 1]
        } else {
            0.0
        };
        
        sample1 * (1.0 - frac) + sample2 * frac
    }
}

pub struct SampleCache {
    samples: std::collections::HashMap<String, Sample>,
}

impl SampleCache {
    pub fn new() -> Self {
        Self {
            samples: std::collections::HashMap::new(),
        }
    }
    
    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.samples.contains_key(path) {
            let sample = Sample::load(path)?;
            self.samples.insert(path.to_string(), sample);
        }
        Ok(())
    }
    
    pub fn get(&self, path: &str) -> Option<&Sample> {
        self.samples.get(path)
    }
}
