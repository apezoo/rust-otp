use clap::{Parser, Subcommand};
use log::{info, error};
use std::fs;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use std::io::{Read, Write, Seek, SeekFrom};
use uuid::Uuid;

use otp_cli::{pad_generator, state_manager};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the OTP vault.
    #[arg(long, global = true)]
    vault: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage OTP vaults
    Vault {
        #[command(subcommand)]
        command: VaultCommands,
    },
    /// Manage pads within a vault
    Pad {
        #[command(subcommand)]
        command: PadCommands,
    },
    /// Encrypt a file using a specified pad
    Encrypt {
        /// Path to the input file to encrypt
        #[arg(short, long)]
        input: PathBuf,

        /// Path to the output file to save the encrypted content
        #[arg(short, long)]
        output: PathBuf,

        /// The ID of the pad to use for encryption
        #[arg(long)]
        pad_id: String,

        /// [ADVANCED] Specify a starting offset in bytes for the pad segment.
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Decrypt a file using a specified pad
    Decrypt {
        /// Path to the input file to decrypt
        #[arg(short, long)]
        input: PathBuf,

        /// Path to the output file to save the decrypted content
        #[arg(short, long)]
        output: PathBuf,

        /// Path to the ciphertext metadata file
        #[arg(long)]
        metadata: PathBuf,
    },
}

#[derive(Subcommand)]
enum VaultCommands {
    /// Initialize a new vault at the specified path
    Init,
    /// Show the status of the vault
    Status,
}

#[derive(Subcommand)]
enum PadCommands {
    /// Generate a new one-time pad file
    Generate {
        /// The size of the pad in megabytes (MB)
        #[arg(short, long, default_value_t = 1)]
        size: usize,
    },
    /// List all pads in the vault
    List,
    /// Delete a pad from the vault
    Delete {
        /// The ID of the pad to delete
        #[arg(long)]
        pad_id: String,
    },
}

/// Metadata stored alongside the ciphertext to enable correct decryption.
#[derive(serde::Serialize, serde::Deserialize)]
struct CiphertextMetadata {
    pad_id: String,
    start_byte: usize,
    length: usize,
    ciphertext_hash: String,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    
    let vault_path = match &cli.command {
        // Init is special, it can create the vault path
        Commands::Vault { command: VaultCommands::Init } => {
             cli.vault.expect("The --vault path is required for 'vault init'")
        }
        // All other commands require an existing vault
        _ => {
            let path = cli.vault.expect("A --vault path is required for this command.");
            if !path.exists() {
                error!("Vault path '{}' does not exist. Please create it with 'vault init'.", path.display());
                std::process::exit(1);
            }
            path
        }
    };

    match &cli.command {
        Commands::Vault { command } => match command {
            VaultCommands::Init => {
                info!("Initializing new vault at '{}'", vault_path.display());
                fs::create_dir_all(&vault_path).expect("Failed to create vault directory");
                fs::create_dir_all(vault_path.join("pads/available")).expect("Failed to create pads directory");
                fs::create_dir_all(vault_path.join("pads/used")).expect("Failed to create used pads directory");
                let initial_state = state_manager::VaultState::default();
                state_manager::save_state(&vault_path, &initial_state);
                info!("Vault initialized successfully.");
            }
            VaultCommands::Status => {
                // TODO: Implement vault status logic
                 info!("Vault status for '{}'", vault_path.display());
            }
        },
        Commands::Pad { command } => {
             let mut state = state_manager::load_state(&vault_path);
             match command {
                PadCommands::Generate { size } => {
                    let pad_id = Uuid::new_v4().to_string();
                    let file_name = format!("{}.pad", pad_id);
                    let pad_path = vault_path.join("pads/available").join(&file_name);
                    let size_in_bytes = size * 1024 * 1024;

                    info!("Generating a new pad with ID '{}' ({} MB)", pad_id, size);

                    match pad_generator::generate_pad(pad_path.to_str().unwrap(), size_in_bytes) {
                        Ok(_) => {
                            state.add_pad(pad_id.clone(), file_name, size_in_bytes);
                            state_manager::save_state(&vault_path, &state);
                            info!("Successfully generated and registered pad.");
                            // Print the ID to stdout for scripting
                            println!("{}", pad_id);
                        }
                        Err(e) => error!("Failed to generate pad file: {}", e),
                    }
                }
                PadCommands::List => {
                    if state.pads.is_empty() {
                        info!("No pads found in vault '{}'", vault_path.display());
                        return;
                    }

                    info!("Pads in vault '{}':", vault_path.display());
                    println!("{:<38} {:<10} {:<15} {:<15}", "ID", "Size (MB)", "Used (Bytes)", "Remaining (Bytes)");
                    println!("{:-<80}", "");

                    for (id, pad) in &state.pads {
                        let total_used: usize = pad.used_segments.iter().map(|s| s.end - s.start).sum();
                        let remaining = pad.size - total_used;
                        let size_mb = pad.size as f64 / (1024.0 * 1024.0);
                        println!("{:<38} {:<10.2} {:<15} {:<15}", id, size_mb, total_used, remaining);
                    }
                }
                PadCommands::Delete { pad_id } => {
                    if let Some(pad_to_delete) = state.pads.get(pad_id) {
                        // Determine if the pad is in available or used directory
                        let used_amount: usize = pad_to_delete.used_segments.iter().map(|s| s.end - s.start).sum();
                        let is_fully_used = used_amount >= pad_to_delete.size;
                        
                        let pad_dir = if is_fully_used { "used" } else { "available" };
                        let pad_path = vault_path.join("pads").join(pad_dir).join(&pad_to_delete.file_name);

                        match fs::remove_file(&pad_path) {
                            Ok(_) => {
                                state.pads.remove(pad_id);
                                state_manager::save_state(&vault_path, &state);
                                info!("Successfully deleted pad '{}' and its file '{}'", pad_id, pad_path.display());
                            }
                            Err(e) => {
                                // If the file is not found, we might still want to remove it from the state
                                if e.kind() == std::io::ErrorKind::NotFound {
                                    state.pads.remove(pad_id);
                                    state_manager::save_state(&vault_path, &state);
                                    error!("Pad file not found at '{}', but removed it from the state. The vault may be inconsistent.", pad_path.display());
                                } else {
                                    error!("Failed to delete pad file '{}': {}", pad_path.display(), e);
                                }
                            }
                        }
                    } else {
                        error!("Pad with ID '{}' not found in the vault.", pad_id);
                    }
                }
            }
        },
        Commands::Encrypt { input, output, pad_id, offset } => {
            let mut state = state_manager::load_state(&vault_path);
            let input_file_size = fs::metadata(input).expect("Failed to get input file metadata").len() as usize;

            if let Some(pad) = state.pads.get_mut(pad_id) {
                let start_byte = match offset {
                Some(offset_val) => {
                    // Advanced mode: User specified an offset. Validate it.
                    let requested_segment = state_manager::UsedSegment { start: *offset_val, end: *offset_val + input_file_size };
                    let is_overlapping = pad.used_segments.iter().any(|s| {
                        (requested_segment.start < s.end) && (requested_segment.end > s.start)
                    });

                    if is_overlapping {
                        error!("The requested pad segment overlaps with an already used segment. This is a critical security risk. Aborting.");
                        return;
                    }
                    if requested_segment.end > pad.size {
                         error!("The requested pad segment exceeds the pad's total size. Aborting.");
                         return;
                    }
                    *offset_val
                }
                None => {
                    // Standard mode: Find the next available segment.
                    pad.find_available_segment(input_file_size).unwrap_or_else(|| {
                        error!("Not enough contiguous space left in pad '{}' to encrypt this file.", pad_id);
                        // TODO: Suggest creating a new pad or using a different one.
                        std::process::exit(1);
                    })
                }
            };

                info!("Encrypting '{}' with pad '{}' starting at byte {}.", input.display(), pad_id, start_byte);

                // Read the pad segment
                let pad_path = vault_path.join("pads/available").join(&pad.file_name);
                let mut pad_file = fs::File::open(&pad_path).expect("Failed to open pad file");
                pad_file.seek(SeekFrom::Start(start_byte as u64)).expect("Failed to seek in pad file");
                let mut pad_segment = vec![0u8; input_file_size];
                pad_file.read_exact(&mut pad_segment).expect("Failed to read pad segment");

                // Prepare for encryption and hashing
                let input_file = fs::File::open(input).expect("Failed to open input file");
                let mut output_file = fs::File::create(output).expect("Failed to create output file");
                let mut hasher = Sha256::new();

                // Encrypt stream and update hash simultaneously
                let mut reader = std::io::BufReader::new(input_file);
                let mut buffer = [0; 8192];
                let mut total_bytes_processed = 0;
                loop {
                    let bytes_read = reader.read(&mut buffer).expect("Failed to read from input");
                    if bytes_read == 0 { break; }

                    let input_chunk = &buffer[..bytes_read];
                    let pad_chunk = &pad_segment[total_bytes_processed..total_bytes_processed + bytes_read];
                    
                    let mut processed_chunk = Vec::with_capacity(bytes_read);
                    for (i, &byte) in input_chunk.iter().enumerate() {
                        processed_chunk.push(byte ^ pad_chunk[i]);
                    }

                    output_file.write_all(&processed_chunk).expect("Failed to write to output");
                    hasher.update(&processed_chunk);
                    total_bytes_processed += bytes_read;
                }
                
                let ciphertext_hash = format!("{:x}", hasher.finalize());

                // Create and save metadata
                let metadata = CiphertextMetadata {
                    pad_id: pad_id.clone(),
                    start_byte,
                    length: input_file_size,
                    ciphertext_hash,
                };
                let metadata_path = format!("{}.metadata.json", output.display());
                let metadata_str = serde_json::to_string_pretty(&metadata).expect("Failed to serialize metadata");
                fs::write(&metadata_path, metadata_str).expect("Failed to write metadata file");

                // Update state
                pad.used_segments.push(state_manager::UsedSegment { start: start_byte, end: start_byte + input_file_size });
                
                // End the mutable borrow of 'pad' here by getting what we need for later.
                let total_used: usize = pad.used_segments.iter().map(|s| s.end - s.start).sum();
                let pad_is_full = total_used >= pad.size;
                let file_name_clone = pad.file_name.clone();

                // Now that 'pad' is no longer borrowed, we can safely borrow 'state'.
                state_manager::save_state(&vault_path, &state);

                // Handle pad lifecycle (moving if fully used)
                if pad_is_full {
                    info!("Pad '{}' is now fully consumed. Moving to 'used' directory.", pad_id);
                    let old_pad_path = vault_path.join("pads/available").join(&file_name_clone);
                    let used_pad_path = vault_path.join("pads/used").join(&file_name_clone);
                    fs::rename(old_pad_path, used_pad_path).expect("Failed to move used pad");
                }

                info!("Successfully encrypted file '{}' to '{}'", input.display(), output.display());
                info!("Decryption metadata saved to '{}'", metadata_path);

            } else {
                error!("Pad with ID '{}' not found.", pad_id);
            }
        }
        Commands::Decrypt { input, output, metadata } => {
            let mut state = state_manager::load_state(&vault_path);
            
            // Read metadata
            let metadata_str = fs::read_to_string(metadata).expect("Failed to read metadata file");
            let metadata: CiphertextMetadata = serde_json::from_str(&metadata_str).expect("Failed to parse metadata file");

            // Hash the input ciphertext to verify integrity
            let mut hasher = Sha256::new();
            let mut ciphertext_file = fs::File::open(input).expect("Failed to open ciphertext file");
            std::io::copy(&mut ciphertext_file, &mut hasher).expect("Failed to hash ciphertext");
            let calculated_hash = format!("{:x}", hasher.finalize());

            if calculated_hash != metadata.ciphertext_hash {
                error!("Ciphertext hash does not match metadata hash. The file may be corrupt or tampered with. Aborting.");
                return;
            }

            if let Some(pad) = state.pads.get_mut(&metadata.pad_id) {
                // Find the pad file (could be in available or used)
                let used_amount: usize = pad.used_segments.iter().map(|s| s.end - s.start).sum();
                let is_fully_used = used_amount >= pad.size;
                let pad_dir = if is_fully_used { "used" } else { "available" };
                let pad_path = vault_path.join("pads").join(pad_dir).join(&pad.file_name);

                if !pad_path.exists() {
                     error!("Pad file '{}' not found in vault. It may have been moved or deleted.", pad.file_name);
                     return;
                }

                // Read pad segment
                let mut pad_file = fs::File::open(&pad_path).expect("Failed to open pad file");
                pad_file.seek(SeekFrom::Start(metadata.start_byte as u64)).expect("Failed to seek in pad file");
                let mut pad_segment = vec![0u8; metadata.length];
                pad_file.read_exact(&mut pad_segment).expect("Failed to read pad segment");

                // Decrypt
                let input_file = fs::File::open(input).expect("Failed to re-open input file");
                let mut output_file = fs::File::create(output).expect("Failed to create output file");
                
                let mut reader = std::io::BufReader::new(input_file);
                let mut buffer = [0; 8192];
                let mut total_bytes_processed = 0;
                loop {
                    let bytes_read = reader.read(&mut buffer).expect("Failed to read from input");
                    if bytes_read == 0 { break; }

                    let input_chunk = &buffer[..bytes_read];
                    let pad_chunk = &pad_segment[total_bytes_processed..total_bytes_processed + bytes_read];
                    
                    let mut processed_chunk = Vec::with_capacity(bytes_read);
                    for (i, &byte) in input_chunk.iter().enumerate() {
                        processed_chunk.push(byte ^ pad_chunk[i]);
                    }

                    output_file.write_all(&processed_chunk).expect("Failed to write to output");
                    total_bytes_processed += bytes_read;
                }

                // Update local state for synchronization
                let new_segment = state_manager::UsedSegment { start: metadata.start_byte, end: metadata.start_byte + metadata.length };
                let mut state_changed = false;
                if !pad.used_segments.iter().any(|s| s.start == new_segment.start && s.end == new_segment.end) {
                    pad.used_segments.push(new_segment);
                    state_changed = true;
                }

                if state_changed {
                    let total_used: usize = pad.used_segments.iter().map(|s| s.end - s.start).sum();
                    let pad_is_full = total_used >= pad.size;
                    let file_name_clone = pad.file_name.clone();
                    let was_in_available = pad_dir == "available";

                    state_manager::save_state(&vault_path, &state);

                    if pad_is_full && was_in_available {
                        info!("Pad '{}' is now fully consumed on receiver side. Moving to 'used' directory.", metadata.pad_id);
                        let old_pad_path = vault_path.join("pads/available").join(&file_name_clone);
                        let used_pad_path = vault_path.join("pads/used").join(&file_name_clone);
                        fs::rename(old_pad_path, used_pad_path).expect("Failed to move used pad");
                    }
                }
                
                info!("Successfully decrypted file '{}' to '{}'", input.display(), output.display());
            } else {
                error!("Pad with ID '{}' not found in vault.", metadata.pad_id);
            }
        }
    }
}
