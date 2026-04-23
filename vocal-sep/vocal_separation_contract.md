# Vocal Separation Backend Contract

## Overview
This contract defines the interface between the Rust wrapper CLI and the separation backend (initially Python/Demucs, eventually pure Rust).

## Backend Interface

### Command-Line Interface
```bash
# Backend binary/script must accept:
vocal-sep-backend \
  --input <path> \
  --output-dir <path> \
  --model <model-name> \
  --stems <stem-types> \
  [--format <audio-format>] \
  [--sample-rate <rate>]

# Exit codes:
# 0 = success
# 1 = invalid arguments
# 2 = file not found
# 3 = processing error
# 4 = model not found
```

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `--input` | path | Yes | Input audio file (any common format) |
| `--output-dir` | path | Yes | Directory for output stems |
| `--model` | string | Yes | Model name (e.g., "htdemucs", "htdemucs_ft", "htdemucs_6s") |
| `--stems` | comma-separated | Yes | Stems to extract: "vocals", "drums", "bass", "other", "guitar", "piano" |
| `--format` | string | No | Output format: "wav", "flac", "mp3" (default: "wav") |
| `--sample-rate` | int | No | Output sample rate in Hz (default: 44100) |

### Output Specification

**File Naming Convention:**
```
<output-dir>/<input-basename>_<stem-name>.<format>

Example:
/path/to/output/rapp_snitch_knishes_vocals.wav
/path/to/output/rapp_snitch_knishes_drums.wav
/path/to/output/rapp_snitch_knishes_bass.wav
/path/to/output/rapp_snitch_knishes_other.wav
```

**Metadata JSON:**
```
<output-dir>/<input-basename>_metadata.json
```

```json
{
  "input_file": "/path/to/input.wav",
  "model": "htdemucs",
  "stems_requested": ["vocals", "drums", "bass", "other"],
  "stems_generated": ["vocals", "drums", "bass", "other"],
  "output_format": "wav",
  "sample_rate": 44100,
  "processing_time_ms": 12450,
  "backend_version": "python-demucs-4.0.0",
  "timestamp": "2026-04-23T14:46:10-05:00"
}
```

### Error Handling

**STDERR Output Format:**
```json
{
  "error": "error_code",
  "message": "Human-readable error message",
  "details": {
    "field": "additional context"
  }
}
```

**Error Codes:**
- `invalid_input_path`: Input file doesn't exist or isn't readable
- `invalid_output_dir`: Output directory doesn't exist or isn't writable
- `unsupported_format`: Input audio format not supported
- `model_not_found`: Requested model not available
- `invalid_stem`: Requested stem not supported by model
- `processing_failed`: Internal processing error
- `out_of_memory`: Insufficient memory for processing

### Progress Reporting (Optional)

Backend MAY write progress to STDERR in JSON lines:
```json
{"progress": 0.0, "stage": "loading_model"}
{"progress": 0.1, "stage": "loading_audio"}
{"progress": 0.3, "stage": "separating"}
{"progress": 0.9, "stage": "writing_outputs"}
{"progress": 1.0, "stage": "complete"}
```

## Model Specifications

### Required Models

| Model ID | Stems | Description |
|----------|-------|-------------|
| `htdemucs` | vocals, drums, bass, other | Standard 4-stem model |
| `htdemucs_ft` | vocals, drums, bass, other | Fine-tuned 4-stem (better quality) |
| `htdemucs_6s` | vocals, drums, bass, other, guitar, piano | 6-stem model |

### Model Discovery

Backend must support:
```bash
vocal-sep-backend --list-models
```

Output (JSON):
```json
{
  "models": [
    {
      "id": "htdemucs",
      "stems": ["vocals", "drums", "bass", "other"],
      "quality": "standard",
      "size_mb": 2300
    },
    {
      "id": "htdemucs_ft",
      "stems": ["vocals", "drums", "bass", "other"],
      "quality": "fine_tuned",
      "size_mb": 2300
    },
    {
      "id": "htdemucs_6s",
      "stems": ["vocals", "drums", "bass", "other", "guitar", "piano"],
      "quality": "standard",
      "size_mb": 2800
    }
  ]
}
```

## Audio Format Support

### Required Input Formats
- WAV (PCM, 16/24/32-bit)
- MP3
- FLAC
- M4A/AAC
- OGG

### Required Output Formats
- WAV (PCM 32-bit float, default)
- FLAC (lossless)
- MP3 (320kbps CBR)

## Performance Requirements

- **Throughput:** Process 1 minute of audio in < 30 seconds on M1 Mac
- **Memory:** Stay under 4GB RAM for 5-minute songs
- **Disk:** Temporary files cleaned up on exit (success or failure)

## Testing Contract

Backend implementation must pass:
```bash
# Test 1: Basic separation
vocal-sep-backend \
  --input test_audio.wav \
  --output-dir /tmp/test \
  --model htdemucs \
  --stems vocals,drums,bass,other

# Verify: 4 WAV files + metadata.json exist

# Test 2: Single stem
vocal-sep-backend \
  --input test_audio.wav \
  --output-dir /tmp/test \
  --model htdemucs \
  --stems vocals

# Verify: 1 WAV file + metadata.json exist

# Test 3: Error handling
vocal-sep-backend \
  --input nonexistent.wav \
  --output-dir /tmp/test \
  --model htdemucs \
  --stems vocals

# Verify: Exit code != 0, JSON error on STDERR

# Test 4: Model listing
vocal-sep-backend --list-models
# Verify: Valid JSON with model array
```

## Version Compatibility

- **Contract Version:** 1.0.0
- Backend MUST report version in metadata JSON
- Wrapper will validate contract version compatibility
- Breaking changes require contract version bump

## Future Extensions (Not Required for v1)

- GPU acceleration flag: `--device cuda` or `--device mps`
- Batch processing: `--input-list <file>`
- Streaming output for real-time processing
- Quality/speed tradeoff: `--quality high|medium|fast`

---

## Implementation Notes

### Python Backend (Phase 1)
- Thin Python script wrapping Demucs library
- Handles CLI parsing, calls Demucs, writes metadata
- ~200 lines of code

### Rust Backend (Phase 2)
- Pure Rust implementation using `tract` or `burn`
- Load ONNX weights from Demucs models
- Implement STFT/iSTFT, convolutions, attention
- Drop-in replacement - same CLI contract

### Wrapper CLI (Immediate)
- Rust binary: `vocal-sep`
- High-level commands: `vocal-sep extract --vocals <input>`
- Manages backend discovery, validation, caching
- User-friendly error messages
- Integration with our audio pipeline
