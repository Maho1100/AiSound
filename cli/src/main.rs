use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sfx", about = "AiSound – Bfxr compatible sound effect generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a WAV file from a parameter JSON file
    Generate {
        /// Input parameter JSON file
        input: PathBuf,

        /// Output WAV file path
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { input, output } => {
            let json = match std::fs::read(&input) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error reading {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            let wav = match sfx_core::generate_wav(&json) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error generating sound: {}", e);
                    std::process::exit(1);
                }
            };

            match std::fs::write(&output, &wav) {
                Ok(_) => {
                    println!(
                        "Generated {} ({} bytes)",
                        output.display(),
                        wav.len()
                    );
                }
                Err(e) => {
                    eprintln!("Error writing {}: {}", output.display(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}
