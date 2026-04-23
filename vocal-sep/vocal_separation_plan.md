# Vocal Separation Tool - Implementation Plan

## Contract-First Design

**Contract Document:** `vocal_separation_contract.md`

The contract defines a clean interface between the Rust wrapper CLI and the separation backend. This allows parallel development and eventual backend replacement without touching the wrapper.

## Three Beads (Issues)

### 1. **workspace-rm0**: Rust Wrapper CLI ✅ Ready
- **What:** User-facing `vocal-sep` command
- **Tech:** Rust (clap, serde, tokio, indicatif)
- **Interface:** High-level commands (`extract`, `list-models`)
- **Responsibility:** 
  - Discover `vocal-sep-backend` in PATH
  - Validate contract compliance
  - Pretty error messages
  - Progress bars
  - Model metadata caching

**Can start immediately** - just needs contract spec.

### 2. **workspace-qrq**: Python Backend (Demucs) ✅ Ready
- **What:** `vocal-sep-backend` script implementing contract
- **Tech:** Python + Demucs v4
- **Interface:** Contract-compliant CLI
- **Responsibility:**
  - Wrap Demucs library
  - Handle all contract parameters
  - Generate metadata JSON
  - Proper error codes/messages
  - Model listing

**Can start immediately** - just needs contract spec.

### 3. **workspace-4i9**: Pure Rust Backend 🔒 Blocked
- **What:** Drop-in replacement for Python backend
- **Tech:** Rust (tract/burn/candle + audio crates)
- **Interface:** Same contract as Python backend
- **Responsibility:**
  - Load Demucs ONNX weights
  - Implement hybrid Transformer architecture
  - Match quality/performance targets
  - GPU acceleration (Metal)

**Blocked on:** Both wrapper and Python backend complete.
**Priority:** P3 (future optimization)

## Dependency Graph

```
workspace-rm0 (Wrapper)  ←┐
                          ├─── workspace-4i9 (Rust Backend)
workspace-qrq (Python)   ←┘
```

The Rust backend is blocked until both the wrapper and Python backend are working. This ensures we have:
1. A working end-to-end system to validate against
2. Real usage patterns to inform the Rust implementation
3. Benchmarks for quality/performance comparison

## Next Steps

**For Subagents:**

1. **Agent A** (Rust wrapper):
   - Read `vocal_separation_contract.md`
   - Implement `vocal-sep` CLI
   - Test with mock backend first
   - Integrate with real backend once available

2. **Agent B** (Python backend):
   - Read `vocal_separation_contract.md`
   - Install Demucs from maintained fork
   - Implement contract-compliant wrapper
   - Pass all contract tests

3. **Agent C** (Rust backend - later):
   - Wait for A + B to complete
   - Research ONNX/ML inference in Rust
   - Port Demucs architecture
   - Benchmark against Python version

## Success Criteria

**Phase 1 Complete:**
- `vocal-sep extract --vocals input.wav` produces clean vocal stem
- Works with any common audio format
- Fast enough for interactive use
- Quality matches Demucs standard (9.0 dB SDR)

**Phase 2 Complete:**
- Pure Rust backend available
- Same quality as Python version
- Better performance (< 30s for 1 min audio)
- Lower memory usage (< 4GB)
- Native Metal GPU acceleration

## Integration Points

**Wrapper ↔ Backend:**
- CLI parameters (defined in contract)
- Exit codes (defined in contract)
- Output file naming (defined in contract)
- Metadata JSON (defined in contract)
- Error JSON on STDERR (defined in contract)

**Backend ↔ Models:**
- Model discovery via `--list-models`
- Model loading (htdemucs, htdemucs_ft, htdemucs_6s)
- Pre-trained weights from Demucs project

## File Structure

```
workspace/
├── vocal_separation_contract.md  ← Contract (already written)
├── vocal_separation_plan.md      ← This file
├── vocal-sep/                     ← Rust wrapper (workspace-rm0)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── backend.rs
│   │   ├── contract.rs
│   │   └── cli.rs
│   └── README.md
├── vocal-sep-backend-py/          ← Python backend (workspace-qrq)
│   ├── vocal-sep-backend
│   ├── requirements.txt
│   └── tests/
└── vocal-sep-backend-rs/          ← Rust backend (workspace-4i9, future)
    ├── Cargo.toml
    └── src/
```

## Timeline Estimate

- **Wrapper (workspace-rm0):** 1-2 days
- **Python Backend (workspace-qrq):** 1-2 days
- **Integration Testing:** 1 day
- **Rust Backend (workspace-4i9):** 2-4 weeks (research + implementation)

Total Phase 1: ~1 week
Total Phase 2: +2-4 weeks
