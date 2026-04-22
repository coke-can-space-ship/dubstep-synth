# Sound Recipes

This file documents parameter combinations that produce good results.

## Wobble Bass

### Classic Heavy Wobble (Drop 1)
```rust
// Root pattern
add_bass_to_buffer_stereo(..., 41.2, 0.75 * beat_duration, 6.0, ...);  // E1, 6Hz wobble
add_bass_to_buffer_stereo(..., 55.0, 0.5 * beat_duration, 6.0, ...);   // A1, 6Hz wobble

// Fast variation
add_bass_to_buffer_stereo(..., 41.2, 0.75 * beat_duration, 8.0, ...);  // E1, 8Hz wobble

// Slow variation
add_bass_to_buffer_stereo(..., 41.2, 2.0 * beat_duration, 4.0, ...);   // E1, 4Hz wobble
```

### Aggressive Bass (Drop 2)
```rust
// Faster wobble, melodic movement
add_bass_to_buffer_stereo(..., 41.2, 1.0 * beat_duration, 8.0, ...);   // E1
add_bass_to_buffer_stereo(..., 46.25, 1.0 * beat_duration, 8.0, ...);  // F#1
add_bass_to_buffer_stereo(..., 55.0, 1.0 * beat_duration, 8.0, ...);   // A1
```

### Minimal Bass (Buildup/Breakdown)
```rust
add_bass_to_buffer_stereo(..., 41.2, 0.5 * beat_duration, 4.0, ...);   // E1, slow wobble
add_bass_to_buffer_stereo(..., 41.2, 4.0 * beat_duration, 2.0, ...);   // E1, very slow
```

## Drum Patterns

### Syncopated Kick Pattern (Drop 1)
```
Beat:  1   1.75   2.5   3   3.75
       K    K     K    S/C   K
```

### Four-on-Floor (Drop 2)
```
Beat:  1   2   3   4
       K   K  S/C  K
```

### Minimal (Intro/Breakdown)
```
Beat:  1       3
       K       K
```

## Panning Values

- Kick/Bass/Sub: 0.0 (center)
- Hi-hats closed: -0.5 / +0.5 (alternating)
- Hi-hats open: 0.0 (center)
- Snare: +0.3 to +0.4 (slight right)
- Clap: -0.3 to -0.4 (slight left)
- Wide hi-hats: -0.6 / +0.6 (extra width)

## Notes

- BPM: 140
- Sample rate: 44100 Hz
- Normalization: 0.8 (80% of max to avoid clipping)
- All bass notes use E minor pentatonic: E1 (41.2), F#1 (46.25), A1 (55.0)
