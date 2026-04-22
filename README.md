# Dubstep Synth

Rust-based dubstep synthesis and sound design tool.

## Features

- Wobble bass synthesis with configurable LFO rates
- Layered drum synthesis (kick, sub-kick, snare, clap, hi-hats)
- Stereo panning for spatial width
- Full track arrangement with intro/buildup/drop/breakdown structure

## Usage

```bash
cargo run --release
```

Generates `dubstep_full_track.wav` - a ~40-second stereo dubstep track at 140 BPM.

## Sound Design

### Wobble Bass
- Sawtooth wave base
- Low-pass filter with LFO modulation
- Configurable wobble rate (4Hz, 6Hz, 8Hz for variety)
- Root note: E1 (41.2 Hz) with variations to A1 and F#1

### Drums
- **Kick**: Sine wave pitch envelope (150Hz → 50Hz)
- **Sub-kick**: Deep sine wave (40Hz) for sub-bass layer
- **Snare**: White noise with band-pass filter + pitch envelope
- **Clap**: Short noise burst
- **Hi-hats**: Filtered white noise (closed/open variants)

### Panning Strategy
- Kick/bass/sub: Centered (mono power)
- Hi-hats: Alternating L/R for width
- Snare: Slight right
- Clap: Slight left
- Constant-power panning for balanced energy

## Structure

```
Bars 1-4:   Intro (minimal drums, no bass)
Bars 5-8:   Buildup (add snare, hi-hats, introduce bass)
Bars 9-16:  Drop 1 (full wobble bass, syncopated drums)
Bars 17-20: Breakdown (strip to minimal)
Bars 21-24: Drop 2 (four-on-floor, dense hi-hats, aggressive bass)
```

## Dependencies

- `hound` - WAV file I/O
