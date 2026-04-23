use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod backend;
mod contract;

#[derive(Parser)]
#[command(name = "vocal-sep")]
#[command(about = "Extract vocal stems from audio using ML-powered separation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract stems from audio file
    Extract {
        /// Input audio file
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory (default: ./stems)
        #[arg(short, long, default_value = "./stems")]
        output: PathBuf,

        /// Model to use (htdemucs, htdemucs_ft, htdemucs_6s)
        #[arg(short, long, default_value = "htdemucs_ft")]
        model: String,

        /// Extract only vocals
        #[arg(long)]
        vocals: bool,

        /// Extract only drums
        #[arg(long)]
        drums: bool,

        /// Extract only bass
        #[arg(long)]
        bass: bool,

        /// Extract all stems
        #[arg(long)]
        all: bool,

        /// Output format (wav, flac, mp3)
        #[arg(short, long, default_value = "wav")]
        format: String,

        /// Sample rate in Hz
        #[arg(long)]
        sample_rate: Option<u32>,
    },

    /// List available models
    ListModels,

    /// Show version
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Extract {
            input,
            output,
            model,
            vocals,
            drums,
            bass,
            all,
            format,
            sample_rate,
        } => {
            // Determine which stems to extract
            let stems = if all {
                vec!["vocals", "drums", "bass", "other"]
            } else if vocals || drums || bass {
                let mut s = Vec::new();
                if vocals {
                    s.push("vocals");
                }
                if drums {
                    s.push("drums");
                }
                if bass {
                    s.push("bass");
                }
                s
            } else {
                // Default: extract vocals
                vec!["vocals"]
            };

            println!("🎵 Extracting stems from: {}", input.display());
            println!("   Model: {}", model);
            println!("   Stems: {}", stems.join(", "));
            println!("   Output: {}", output.display());

            backend::extract_stems(
                &input,
                &output,
                &model,
                &stems,
                &format,
                sample_rate,
            )
            .await
            .context("Failed to extract stems")?;

            println!("✅ Done! Stems saved to: {}", output.display());
        }

        Commands::ListModels => {
            println!("📋 Available models:");
            let models = backend::list_models().await?;
            for model in models {
                println!("  • {} - {} stems", model.id, model.stems.join(", "));
            }
        }

        Commands::Version => {
            println!("vocal-sep v{}", env!("CARGO_PKG_VERSION"));
            let backend_version = backend::get_version().await?;
            println!("backend: {}", backend_version);
        }
    }

    Ok(())
}
