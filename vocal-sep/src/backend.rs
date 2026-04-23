use anyhow::{anyhow, bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::process::Command as TokioCommand;
use which::which;

use crate::contract::{BackendError, ModelInfo, ModelsResponse, ProgressUpdate, SeparationMetadata};

/// Find the vocal-sep-backend binary in PATH
fn find_backend() -> Result<PathBuf> {
    which("vocal-sep-backend")
        .context("vocal-sep-backend not found in PATH. Please install the backend first.")
}

/// Get backend version
pub async fn get_version() -> Result<String> {
    let backend = find_backend()?;
    
    let output = TokioCommand::new(&backend)
        .arg("--version")
        .output()
        .await
        .context("Failed to execute backend")?;

    if !output.status.success() {
        bail!("Backend version check failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// List available models
pub async fn list_models() -> Result<Vec<ModelInfo>> {
    let backend = find_backend()?;

    let output = TokioCommand::new(&backend)
        .arg("--list-models")
        .output()
        .await
        .context("Failed to execute backend")?;

    if !output.status.success() {
        let err: BackendError = serde_json::from_slice(&output.stderr)
            .context("Failed to parse backend error")?;
        bail!("Backend error: {}", err.message);
    }

    let response: ModelsResponse = serde_json::from_slice(&output.stdout)
        .context("Failed to parse models response")?;

    Ok(response.models)
}

/// Extract stems from audio file
pub async fn extract_stems(
    input: &Path,
    output_dir: &Path,
    model: &str,
    stems: &[&str],
    format: &str,
    sample_rate: Option<u32>,
) -> Result<SeparationMetadata> {
    let backend = find_backend()?;

    // Validate input exists
    if !input.exists() {
        bail!("Input file not found: {}", input.display());
    }

    // Create output directory
    std::fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    // Build command
    let mut cmd = Command::new(&backend);
    cmd.arg("--input")
        .arg(input)
        .arg("--output-dir")
        .arg(output_dir)
        .arg("--model")
        .arg(model)
        .arg("--stems")
        .arg(stems.join(","))
        .arg("--format")
        .arg(format);

    if let Some(sr) = sample_rate {
        cmd.arg("--sample-rate").arg(sr.to_string());
    }

    // Set up progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Execute with streaming stderr for progress
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn backend process")?;

    // Read stderr for progress updates
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line?;
            
            // Try to parse as progress JSON
            if let Ok(progress) = serde_json::from_str::<ProgressUpdate>(&line) {
                pb.set_position((progress.progress * 100.0) as u64);
                pb.set_message(progress.stage.clone());
            }
            // Otherwise it might be an error
            else if line.contains("\"error\"") {
                if let Ok(err) = serde_json::from_str::<BackendError>(&line) {
                    pb.finish_and_clear();
                    bail!("Backend error: {}", err.message);
                }
            }
        }
    }

    let status = child.wait().context("Failed to wait for backend")?;

    pb.finish_and_clear();

    if !status.success() {
        bail!("Backend exited with code: {:?}", status.code());
    }

    // Read metadata JSON
    let input_basename = input
        .file_stem()
        .ok_or_else(|| anyhow!("Invalid input filename"))?
        .to_string_lossy();
    
    let metadata_path = output_dir.join(format!("{}_metadata.json", input_basename));
    
    let metadata_content = std::fs::read_to_string(&metadata_path)
        .context("Failed to read metadata JSON")?;
    
    let metadata: SeparationMetadata = serde_json::from_str(&metadata_content)
        .context("Failed to parse metadata JSON")?;

    Ok(metadata)
}
