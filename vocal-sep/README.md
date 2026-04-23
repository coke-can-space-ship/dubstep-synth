# vocal-sep

ML-powered vocal and stem separation CLI tool.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Extract vocals only (default)
vocal-sep extract --input song.wav

# Extract all stems
vocal-sep extract --input song.wav --all

# Extract specific stems
vocal-sep extract --input song.wav --vocals --drums

# Specify output directory and model
vocal-sep extract --input song.wav --output ./my-stems --model htdemucs_ft

# List available models
vocal-sep list-models

# Check version
vocal-sep version
```

## Requirements

- `vocal-sep-backend` must be installed and available in PATH
- See `vocal_separation_contract.md` for backend requirements

## Architecture

This is the user-facing Rust wrapper. It discovers and communicates with a backend that implements the separation contract. The backend can be:

1. **Python/Demucs** (current) - Uses pre-trained Demucs models
2. **Pure Rust** (future) - Native Rust implementation with ONNX models

The wrapper is backend-agnostic - as long as the backend implements the contract, it will work.

## Development

See `vocal_separation_plan.md` for the full development plan.
