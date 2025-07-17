use clap::{Parser, Subcommand};
use log::{info, error};
use state_manager::AppState;
use std::fs;
use std::path::Path;

mod pad_generator;
mod state_manager;
mod crypto;

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
    /// Encrypt a file using a specified pad
    Encrypt {
        /// Path to the input file to encrypt
        #[arg(short, long)]
        input: String,

        /// Path to the output file to save the encrypted content
        #[arg(short, long)]
        output: String,

        /// The ID of the pad to use for encryption
        #[arg(long)]
        pad_id: String,
    },
    /// Decrypt a file using a specified pad
    Decrypt {
        /// Path to the input file to decrypt
        #[arg(short, long)]
        input: String,

        /// Path to the output file to save the decrypted content
        #[arg(short, long)]
        output: String,

        /// The ID of the pad to use for encryption
        #[arg(long)]
        pad_id: String,
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
        Commands::Encrypt { input, output, pad_id } => {
            if let Some(pad) = state.pads.get_mut(pad_id) {
                let input_file = fs::File::open(input).expect("Failed to open input file");
                let input_metadata = input_file.metadata().expect("Failed to get input file metadata");
                let input_size = input_metadata.len() as usize;

                if pad.size - pad.used_bytes < input_size {
                    error!("Not enough unused pad data to encrypt this file.");
                    return;
                }

                let mut pad_file = fs::File::open(&pad.path).expect("Failed to open pad file");
                use std::io::Seek;
                pad_file.seek(std::io::SeekFrom::Start(pad.used_bytes as u64)).expect("Failed to seek in pad file");

                let mut pad_segment = vec![0u8; input_size];
                use std::io::Read;
                pad_file.read_exact(&mut pad_segment).expect("Failed to read pad segment");

                let output_file = fs::File::create(output).expect("Failed to create output file");

                match crypto::process_stream(input_file, output_file, &pad_segment) {
                    Ok(_) => {
                        pad.used_bytes += input_size;
                        save_state(&state);
                        info!("Successfully encrypted file '{}' to '{}'", input, output);
                    }
                    Err(e) => error!("Failed to encrypt file: {}", e),
                }

            } else {
                error!("Pad with ID '{}' not found.", pad_id);
            }
        }
        Commands::Decrypt { input, output, pad_id } => {
             if let Some(pad) = state.pads.get(pad_id) {
                let input_file = fs::File::open(input).expect("Failed to open input file");
                let input_metadata = input_file.metadata().expect("Failed to get input file metadata");
                let input_size = input_metadata.len() as usize;

                // For decryption, we need to know where the ciphertext's pad segment *started*.
                // This is a simplification; a real app would need to store this metadata.
                // We'll assume the user knows what they're doing and use the *current* `used_bytes`
                // as the end of the segment and calculate the start.
                if pad.used_bytes < input_size {
                    error!("Invalid state: used_bytes is less than the input file size for decryption.");
                    return;
                }
                
                let segment_start = pad.used_bytes - input_size;

                let mut pad_file = fs::File::open(&pad.path).expect("Failed to open pad file");
                use std::io::Seek;
                pad_file.seek(std::io::SeekFrom::Start(segment_start as u64)).expect("Failed to seek in pad file");

                let mut pad_segment = vec![0u8; input_size];
                use std::io::Read;
                pad_file.read_exact(&mut pad_segment).expect("Failed to read pad segment");

                let output_file = fs::File::create(output).expect("Failed to create output file");

                match crypto::process_stream(input_file, output_file, &pad_segment) {
                    Ok(_) => {
                        // We don't modify state on decrypt, as we are "re-using" the segment
                        info!("Successfully decrypted file '{}' to '{}'", input, output);
                    }
                    Err(e) => error!("Failed to decrypt file: {}", e),
                }

            } else {
                error!("Pad with ID '{}' not found.", pad_id);
            }
        }
    }
}
