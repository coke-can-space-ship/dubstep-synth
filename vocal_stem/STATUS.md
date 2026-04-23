# Vocal Stem Tools - Status Report

## Run: 2026-04-23 13:12 CT

### ✅ COMPLETE - Ready for User Listening

**Three complete dubstep compositions with pitch-corrected rap vocals are ready.**

### Composition Details

All compositions built from:
- **Base track**: `drum_machine/layered_v3_punchy_96bars.wav` (96 bars, 140 BPM, F Major)
- **Vocals**: Pitch-shifted to F Major, mixed at 0.8 gain with 0.5 bar fades

#### Mix Variation 1: MF DOOM - "All Caps"
- **File**: `compositions/mix_variation_1_doom_allcaps_v2.wav`
- **Vocals placed**: Bars 16-32 (early breakdown section)
- **Original key**: A Minor → **Shifted to F Major (+5 semitones)**
- **Duration**: 96 bars (2:44 @ 140 BPM)

#### Mix Variation 2: Megan Thee Stallion - "Savage"
- **File**: `compositions/mix_variation_2_megan_savage_v2.wav`
- **Vocals placed**: Bars 32-48 (mid-track breakdown)
- **Original key**: A Minor → **Shifted to F Major (+5 semitones)**
- **Duration**: 96 bars (2:44 @ 140 BPM)

#### Mix Variation 3: MF DOOM - "Accordion"
- **File**: `compositions/mix_variation_3_doom_accordion_v2.wav`
- **Vocals placed**: Bars 48-64 (final breakdown section)
- **Original key**: C Major → **Shifted to F Major (+5 semitones)**
- **Duration**: 96 bars (2:44 @ 140 BPM)

### Technical Implementation

**Rust Tools Built:**
1. `key_detect` - Chromagram-based key detection
2. `vocal_extract` - Center channel isolation + spectral gating
3. `pitch_shift` - Phase vocoder pitch shifting
4. `track_composer` - Multi-track mixing with fade controls

**Processing Pipeline:**
1. Downloaded 3 rap tracks (MF DOOM × 2, Megan × 1)
2. Extracted vocals using center channel + spectral gate
3. Detected original keys via chromagram analysis
4. Pitch-shifted all vocals to F Major (drum track key)
5. Converted to 16-bit PCM for compatibility
6. Mixed vocals into breakdown sections with fades

### Files Committed to GitHub

**Compositions** (JSON metadata only, WAVs gitignored):
- `compositions/mix_variation_1_doom_allcaps_v2.json`
- `compositions/mix_variation_2_megan_savage_v2.json`
- `compositions/mix_variation_3_doom_accordion_v2.json`

**Latest commit**: `8b9ecee` - "feat(compositions): Add pitch-corrected vocal mixes v2"

### Key Compatibility Analysis

- **Drum track**: F Major (98.2% confidence) ← **TARGET**
- **Wobble bass**: C# Major (96.5% confidence) - intentional dissonance
- **All vocals**: Shifted to F Major for harmonic compatibility

### Next Steps

**User Action Required:**
1. Listen to all three compositions
2. Provide feedback on:
   - Vocal placement timing (too early/late in breakdown?)
   - Gain levels (vocals too loud/quiet vs drums?)
   - Key matching quality (any dissonance?)
   - Preference between the three tracks

**Potential Improvements:**
- BPM matching (vocals may need time-stretching to sync with 140 BPM)
- Additional processing (reverb, delay, compression)
- Multiple vocal placements per track
- Wobble bass integration during vocal sections

**Status**: ✅ **ALL WORK COMPLETE - AWAITING USER FEEDBACK**
