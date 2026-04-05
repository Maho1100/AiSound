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

        /// Output WAV file path (default: output/<category>/<name>.wav)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// JSON からカテゴリと名前を抽出して出力パスを決定する
fn resolve_output_path(json: &[u8], explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }

    // JSON を部分パースして meta.category と meta.name を取得
    let category;
    let name;
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(json) {
        category = v["meta"]["category"]
            .as_str()
            .unwrap_or("misc")
            .to_string();
        name = v["meta"]["name"]
            .as_str()
            .unwrap_or("output")
            .to_string();
    } else {
        category = "misc".to_string();
        name = "output".to_string();
    }

    PathBuf::from("output").join(&category).join(format!("{}.wav", name))
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

            let out_path = resolve_output_path(&json, output);

            // 出力先ディレクトリが無ければ作成
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("Error creating directory {}: {}", parent.display(), e);
                        std::process::exit(1);
                    }
                }
            }

            let wav = match sfx_core::generate_wav(&json) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error generating sound: {}", e);
                    std::process::exit(1);
                }
            };

            match std::fs::write(&out_path, &wav) {
                Ok(_) => {
                    println!(
                        "Generated {} ({} bytes)",
                        out_path.display(),
                        wav.len()
                    );
                }
                Err(e) => {
                    eprintln!("Error writing {}: {}", out_path.display(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}
