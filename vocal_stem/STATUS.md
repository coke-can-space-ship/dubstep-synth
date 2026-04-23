# Vocal Stem Tools - Status Report

## Run: 2026-04-23 11:46 CT

### Tools Built ✅
All three Rust tools compiled successfully:
- **key_detect** - Chromagram-based key detection using Krumhansl-Schmuckler profiles
- **vocal_extract** - Center channel extraction + high-pass filter + spectral gating
- **pitch_shift** - Phase vocoder for pitch shifting vocals to match track key

### Key Analysis Results
- **Drum track** (`layered_v3_punchy_96bars.wav`): **F Major** (98.2% confidence)
- **Wobble bass** (`wobble.wav`): **C# Major** (96.5% confidence)

### Next Steps - USER INPUT NEEDED

**Option 1: MF DOOM**
- Provide a MF DOOM track (any key - we can pitch shift)
- Suggested: "All Caps", "Rapp Snitch Knishes", "Accordion"

**Option 2: Megan Thee Stallion**
- Provide a Megan track (any key - we can pitch shift)
- Suggested: "Savage", "Hot Girl Summer", "Body"

### Usage Examples
```bash
# Detect key of vocal source
./vocal_stem/target/release/key_detect input.wav

# Extract vocals from stereo track
./vocal_stem/target/release/vocal_extract input.wav vocals_only.wav

# Pitch shift to match our track (F Major)
./vocal_stem/target/release/pitch_shift vocals_only.wav vocals_shifted.wav -2
```

### Technical Approach
1. User provides audio file
2. Detect source key
3. Extract vocals using center channel + spectral filtering
4. Calculate semitone shift needed to reach F Major
5. Apply pitch shift
6. Deliver clean vocal stem ready for mixing

**Status: WAITING FOR AUDIO FILE**
