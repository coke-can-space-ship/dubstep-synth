use serde::{Deserialize, Serialize};

/// Metadata JSON structure from backend
#[derive(Debug, Serialize, Deserialize)]
pub struct SeparationMetadata {
    pub input_file: String,
    pub model: String,
    pub stems_requested: Vec<String>,
    pub stems_generated: Vec<String>,
    pub output_format: String,
    pub sample_rate: u32,
    pub processing_time_ms: u64,
    pub backend_version: String,
    pub timestamp: String,
}

/// Model information from backend
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub stems: Vec<String>,
    pub quality: String,
    pub size_mb: u32,
}

/// Models list response
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

/// Error JSON from backend
#[derive(Debug, Serialize, Deserialize)]
pub struct BackendError {
    pub error: String,
    pub message: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

/// Progress update from backend
#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub progress: f32,
    pub stage: String,
}
