# vocal-sep-backend (Python/Demucs)

Python backend implementing the vocal separation contract using Demucs v4.

## Installation

```bash
# Install dependencies
pip install demucs torch

# Make executable
chmod +x vocal-sep-backend

# Add to PATH (or use absolute path)
export PATH="$PATH:$(pwd)"
```

## Usage

```bash
# List available models
vocal-sep-backend --list-models

# Extract vocals
vocal-sep-backend \
  --input song.wav \
  --output-dir ./stems \
  --model htdemucs_ft \
  --stems vocals

# Extract all stems
vocal-sep-backend \
  --input song.wav \
  --output-dir ./stems \
  --model htdemucs_ft \
  --stems vocals,drums,bass,other
```

## Contract Compliance

This backend implements `vocal_separation_contract.md`:

- ✅ CLI interface with required parameters
- ✅ Exit codes (0=success, 1=invalid args, 2=file error, 3=processing error, 4=model error)
- ✅ JSON metadata output
- ✅ JSON errors on STDERR
- ✅ Progress reporting on STDERR
- ✅ Model listing
- ✅ File naming convention

## Models

- **htdemucs** - Standard 4-stem (vocals, drums, bass, other)
- **htdemucs_ft** - Fine-tuned 4-stem (better quality)
- **htdemucs_6s** - 6-stem (vocals, drums, bass, other, guitar, piano)

Models are downloaded automatically on first use (~2.3-2.8GB each).

## Testing

```bash
# Test with sample audio
./test_backend.sh
```

## Development

See `vocal_separation_contract.md` for the full contract specification.
