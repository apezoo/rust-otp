use clap::{Parser, Subcommand};
use log::{info, error};
use state_manager::AppState;
use std::fs;
use std::path::Path;

mod pad_generator;
mod state_manager;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new one-time pad file
    Generate {
        /// The path to save the pad file
        #[arg(short, long)]
        path: String,

        /// The size of the pad in megabytes (MB)
        #[arg(short, long, default_value_t = 1)]
        size: usize,
    },
}

const STATE_FILE: &str = ".otp_state.json";

fn load_state() -> AppState {
    if Path::new(STATE_FILE).exists() {
        let state_str = fs::read_to_string(STATE_FILE).expect("Failed to read state file");
        serde_json::from_str(&state_str).expect("Failed to parse state file")
    } else {
        AppState::default()
    }
}

fn save_state(state: &AppState) {
    let state_str = serde_json::to_string_pretty(state).expect("Failed to serialize state");
    fs::write(STATE_FILE, state_str).expect("Failed to write state file");
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let mut state = load_state();

    match &cli.command {
        Commands::Generate { path, size } => {
            let size_in_bytes = size * 1024 * 1024;
            info!("Generating a new pad file at '{path}' with size {size} MB.");
            match pad_generator::generate_pad(path, size_in_bytes) {
                Ok(_) => {
                    info!("Successfully generated pad file.");
                    state.add_pad(path.clone(), size_in_bytes);
                    save_state(&state);
                    info!("Pad has been registered to the application state.");
                }
                Err(e) => error!("Failed to generate pad file: {e}"),
            }
        }
    }
}
