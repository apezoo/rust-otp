#![deny(missing_docs)]
//! A command-line interface for the OTP encryption tool.

use clap::{Parser, Subcommand};
use log::{error, info};
use otp_core::{pad_generator, state_manager};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(
    after_help = "EXAMPLES:\n  \n# Initialize a new vault\notp-cli --vault ./my_vault vault init\n\n# Generate a new pad\notp-cli --vault ./my_vault pad generate\n\n# Encrypt a file with automatic pad selection\notp-cli --vault ./my_vault encrypt ./my_file.txt\n\n# Encrypt a file with a specific pad\notp-cli --vault ./my_vault encrypt ./my_file.txt --pad-id <PAD_ID>\n\n# Decrypt using a metadata file\notp-cli --vault ./my_vault decrypt --metadata ./my_file.enc.metadata.json --input ./my_file.enc --output ./my_file.txt\n\n# Decrypt manually without a metadata file\notp-cli --vault ./my_vault decrypt --input ./my_file.enc --output ./my_file.txt --pad-id <PAD_ID> --length <FILE_SIZE>"
)]
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
        #[arg()]
        input: PathBuf,

        /// Path to the output file to save the encrypted content. If omitted, uses the input filename with a .enc extension.
        #[arg(short, long, value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>,

        /// The ID of the pad to use for encryption. If omitted, a suitable pad will be selected automatically.
        #[arg(long, value_name = "PAD_ID")]
        pad_id: Option<String>,

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

        /// Path to the ciphertext metadata file. If omitted, --pad-id and --length must be provided.
        #[arg(long, value_name = "METADATA_FILE")]
        metadata: Option<PathBuf>,

        /// The ID of the pad to use for decryption. Required if --metadata is not used.
        #[arg(long, value_name = "PAD_ID", required_if_eq("metadata", "None"))]
        pad_id: Option<String>,

        /// The length of the pad segment to use. Required if --metadata is not used.
        #[arg(long, value_name = "LENGTH", required_if_eq("metadata", "None"))]
        length: Option<usize>,

        /// The starting offset in bytes for the pad segment. Defaults to 0 if not provided.
        #[arg(long, value_name = "OFFSET", default_value_t = 0)]
        offset: usize,
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
#[command(
    after_help = "EXAMPLES:\n  \n# Generate a single 10MB pad\notp-cli --vault ./my_vault pad generate --size 10\n\n# Generate 5 pads of 1MB each\notp-cli --vault ./my_vault pad generate --count 5"
)]
enum PadCommands {
    /// Generate a new one-time pad file
    Generate {
        /// The size of the pad in megabytes (MB)
        #[arg(short, long, default_value_t = 1)]
        size: usize,
        /// The number of pads to generate
        #[arg(short, long, default_value_t = 1)]
        count: u32,
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
struct DecryptionInfo {
    pad_id: String,
    start_byte: usize,
    length: usize,
}
fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let vault_path = match &cli.command {
        Commands::Vault {
            command: VaultCommands::Init,
        } => cli.vault.unwrap_or_else(|| {
            error!("The --vault path is required for 'vault init'");
            std::process::exit(1);
        }),
        _ => {
            let path = cli.vault.unwrap_or_else(|| {
                error!("A --vault path is required for this command.");
                std::process::exit(1);
            });
            if !path.exists() {
                error!(
                    "Vault path '{}' does not exist. Please create it with 'vault init'.",
                    path.display()
                );
                std::process::exit(1);
            }
            path
        }
    };

    match &cli.command {
        Commands::Vault { command } => match command {
            VaultCommands::Init => {
                info!("Initializing new vault at '{}'", vault_path.display());
                if let Err(e) = fs::create_dir_all(&vault_path) {
                    error!("Failed to create vault directory: {e}");
                    std::process::exit(1);
                }
                if let Err(e) = fs::create_dir_all(vault_path.join("pads/available")) {
                    error!("Failed to create pads directory: {e}");
                    std::process::exit(1);
                }
                if let Err(e) = fs::create_dir_all(vault_path.join("pads/used")) {
                    error!("Failed to create used pads directory: {e}");
                    std::process::exit(1);
                }
                let initial_state = state_manager::VaultState::default();
                if let Err(e) = state_manager::save_state(&vault_path, &initial_state) {
                    error!("Failed to save initial state: {e}");
                    std::process::exit(1);
                }
                info!("Vault initialized successfully.");
            }
            VaultCommands::Status => {
                let state = state_manager::load_state(&vault_path).unwrap_or_else(|e| {
                    error!("Failed to load vault state: {e}");
                    std::process::exit(1);
                });
                let available_pads = state.pads.values().filter(|p| !p.is_fully_used).count();
                let used_pads = state.pads.len() - available_pads;
                let total_pads = state.pads.len();

                let total_storage_bytes: u64 = state.pads.values().map(|p| p.size as u64).sum();
                let total_storage_mb = total_storage_bytes as f64 / (1024.0 * 1024.0);

                let total_used_bytes: u64 = state
                    .pads
                    .values()
                    .map(|p| p.total_used_bytes() as u64)
                    .sum();
                let total_used_mb = total_used_bytes as f64 / (1024.0 * 1024.0);

                println!("Vault Status for: {}", vault_path.display());
                println!("{:-<40}", "");
                println!("Total Pads: {total_pads}");
                println!("  - Available: {available_pads}");
                println!("  - Fully Used: {used_pads}");
                println!();
                println!("Total Storage: {total_storage_mb:.2} MB");
                println!("  - Used: {total_used_mb:.2} MB");
                println!(
                    "  - Remaining: {:.2} MB",
                    total_storage_mb - total_used_mb
                );
            }
        },
        Commands::Pad { command } => {
            let mut state = state_manager::load_state(&vault_path).unwrap_or_else(|e| {
                error!("Failed to load vault state: {e}");
                std::process::exit(1);
            });
            match command {
                PadCommands::Generate { size, count } => {
                    info!("Generating {count} new pad(s) of {size} MB each...");
                    for _ in 0..*count {
                        let pad_id = Uuid::new_v4().to_string();
                        let file_name = format!("{pad_id}.pad");
                        let pad_path = vault_path.join("pads/available").join(&file_name);
                        let size_in_bytes = size * 1024 * 1024;

                        match pad_generator::generate_pad(
                            pad_path.to_str().unwrap_or_default(),
                            size_in_bytes,
                        ) {
                            Ok(()) => {
                                state.add_pad(pad_id.clone(), file_name, size_in_bytes);
                                println!("{pad_id}");
                            }
                            Err(e) => {
                                error!("Failed to generate pad file for ID {pad_id}: {e}");
                            }
                        }
                    }
                    if let Err(e) = state_manager::save_state(&vault_path, &state) {
                        error!("Failed to save state after generating pads: {e}");
                    } else {
                        info!("Successfully generated and registered {count} pad(s).");
                    }
                }
                PadCommands::List => {
                    if state.pads.is_empty() {
                        println!("No pads found in vault '{}'", vault_path.display());
                        return;
                    }

                    println!("Pads in vault '{}':", vault_path.display());
                    println!(
                        "{:<38} {:<10} {:<15} {:<15}",
                        "ID", "Size (MB)", "Used (Bytes)", "Remaining (Bytes)"
                    );
                    println!("{:-<80}", "");

                    for (id, pad) in &state.pads {
                        let total_used = pad.total_used_bytes();
                        let remaining = pad.size - total_used;
                        let size_mb = pad.size as f64 / (1024.0 * 1024.0);
                        println!("{id:<38} {size_mb:<10.2} {total_used:<15} {remaining:<15}");
                    }
                }
                PadCommands::Delete { pad_id } => {
                    if let Some(pad_to_delete) = state.pads.get(pad_id) {
                        let pad_dir = if pad_to_delete.is_fully_used {
                            "used"
                        } else {
                            "available"
                        };
                        let pad_path = vault_path
                            .join("pads")
                            .join(pad_dir)
                            .join(&pad_to_delete.file_name);

                        match fs::remove_file(&pad_path) {
                            Ok(()) => {
                                state.pads.remove(pad_id);
                                if let Err(e) = state_manager::save_state(&vault_path, &state) {
                                    error!("Failed to save state after deleting pad: {e}");
                                } else {
                                    println!(
                                        "Successfully deleted pad '{}' and its file '{}'",
                                        pad_id,
                                        pad_path.display()
                                    );
                                }
                            }
                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::NotFound {
                                    state.pads.remove(pad_id);
                                    if let Err(e) =
                                        state_manager::save_state(&vault_path, &state)
                                    {
                                        error!("Failed to save state after removing non-existent pad file: {e}");
                                    } else {
                                        println!("Pad file not found at '{}', but removed it from the state. The vault may be inconsistent.", pad_path.display());
                                    }
                                } else {
                                    error!(
                                        "Failed to delete pad file '{}': {e}",
                                        pad_path.display()
                                    );
                                }
                            }
                        }
                    } else {
                        println!("Pad with ID '{pad_id}' not found in the vault.");
                    }
                }
            }
        }
        Commands::Encrypt {
            input,
            output,
            pad_id,
            offset,
        } => {
            let mut state = state_manager::load_state(&vault_path).unwrap_or_else(|e| {
                error!("Failed to load vault state: {e}");
                std::process::exit(1);
            });
            let input_file_size = match fs::metadata(input) {
                Ok(m) => m.len() as usize,
                Err(e) => {
                    error!("Failed to get input file metadata: {e}");
                    std::process::exit(1);
                }
            };

            let output = output.clone().unwrap_or_else(|| {
                let mut new_path = input.as_os_str().to_owned();
                new_path.push(".enc");
                PathBuf::from(new_path)
            });

            let pad_id_to_use = pad_id.clone().map_or_else(
                || {
                    state
                        .pads
                        .values()
                        .find(|p| p.find_available_segment(input_file_size).is_some())
                        .map_or_else(
                            || {
                                error!("Could not find an available pad with enough contiguous space ({input_file_size} bytes).");
                                error!("Please generate a new pad with 'pad generate'.");
                                std::process::exit(1);
                            },
                            |pad| {
                                println!("Automatically selected pad '{}'", pad.id);
                                pad.id.clone()
                            },
                        )
                },
                |id| id,
            );

            if let Some(pad) = state.pads.get_mut(&pad_id_to_use) {
                if pad.is_fully_used {
                    error!(
                        "Cannot encrypt with pad '{}' because it is fully used.",
                        pad.id
                    );
                    return;
                }

                let start_byte = match offset {
                    Some(offset_val) => {
                        let requested_segment = state_manager::UsedSegment {
                            start: *offset_val,
                            end: *offset_val + input_file_size,
                        };
                        let is_overlapping = pad
                            .used_segments
                            .iter()
                            .any(|s| (requested_segment.start < s.end) && (requested_segment.end > s.start));

                        if is_overlapping {
                            error!("The requested pad segment overlaps with an already used segment. This is a critical security risk. Aborting.");
                            return;
                        }
                        if requested_segment.end > pad.size {
                            error!(
                                "The requested pad segment exceeds the pad's total size. Aborting."
                            );
                            return;
                        }
                        *offset_val
                    }
                    None => pad.find_available_segment(input_file_size).unwrap_or_else(|| {
                        error!("Not enough contiguous space left in pad '{pad_id_to_use}' to encrypt this file.");
                        std::process::exit(1);
                    }),
                };

                info!(
                    "Encrypting '{}' with pad '{}' starting at byte {}.",
                    input.display(),
                    pad_id_to_use,
                    start_byte
                );

                let pad_path = vault_path.join("pads/available").join(&pad.file_name);
                let mut pad_file = fs::File::open(&pad_path).unwrap_or_else(|e| {
                    error!("Failed to open pad file: {e}");
                    std::process::exit(1);
                });
                if let Err(e) = pad_file.seek(SeekFrom::Start(start_byte as u64)) {
                    error!("Failed to seek in pad file: {e}");
                    std::process::exit(1);
                }
                let mut pad_segment = vec![0u8; input_file_size];
                if let Err(e) = pad_file.read_exact(&mut pad_segment) {
                    error!("Failed to read pad segment: {e}");
                    std::process::exit(1);
                }

                let input_file = fs::File::open(input).unwrap_or_else(|e| {
                    error!("Failed to open input file: {e}");
                    std::process::exit(1);
                });
                let mut output_file = fs::File::create(&output).unwrap_or_else(|e| {
                    error!("Failed to create output file: {e}");
                    std::process::exit(1);
                });
                let mut hasher = Sha256::new();

                let mut reader = std::io::BufReader::new(input_file);
                let mut buffer = [0; 8192];
                let mut total_bytes_processed = 0;
                loop {
                    let bytes_read = reader.read(&mut buffer).unwrap_or_else(|e| {
                        error!("Failed to read from input: {e}");
                        std::process::exit(1);
                    });
                    if bytes_read == 0 {
                        break;
                    }

                    let input_chunk = &buffer[..bytes_read];
                    let pad_chunk =
                        &pad_segment[total_bytes_processed..total_bytes_processed + bytes_read];

                    let mut processed_chunk = Vec::with_capacity(bytes_read);
                    for (i, &byte) in input_chunk.iter().enumerate() {
                        processed_chunk.push(byte ^ pad_chunk[i]);
                    }

                    if let Err(e) = output_file.write_all(&processed_chunk) {
                        error!("Failed to write to output: {e}");
                        std::process::exit(1);
                    }
                    hasher.update(&processed_chunk);
                    total_bytes_processed += bytes_read;
                }

                let ciphertext_hash = format!("{:x}", hasher.finalize());

                let metadata = CiphertextMetadata {
                    pad_id: pad_id_to_use.clone(),
                    start_byte,
                    length: input_file_size,
                    ciphertext_hash,
                };
                let metadata_path = format!("{}.metadata.json", output.display());
                let metadata_str = serde_json::to_string_pretty(&metadata).unwrap_or_else(|e| {
                    error!("Failed to serialize metadata: {e}");
                    std::process::exit(1);
                });
                if let Err(e) = fs::write(&metadata_path, metadata_str) {
                    error!("Failed to write metadata file: {e}");
                    std::process::exit(1);
                }

                pad.used_segments.push(state_manager::UsedSegment {
                    start: start_byte,
                    end: start_byte + input_file_size,
                });

                let total_used_bytes = pad.total_used_bytes() as f64;
                let usage_percent = (total_used_bytes / pad.size as f64) * 100.0;

                pad.is_fully_used = pad.total_used_bytes() >= pad.size;
                let is_full = pad.is_fully_used;
                let file_name_clone = pad.file_name.clone();
                if let Err(e) = state_manager::save_state(&vault_path, &state) {
                    error!("Failed to save state after encryption: {e}");
                }

                println!("Pad '{pad_id_to_use}' is now {usage_percent:.2}% used.");

                if is_full {
                    println!(
                        "Pad '{pad_id_to_use}' is now fully consumed. Moving to 'used' directory."
                    );
                    let old_pad_path =
                        vault_path.join("pads/available").join(&file_name_clone);
                    let used_pad_path = vault_path.join("pads/used").join(&file_name_clone);
                    if old_pad_path.exists() {
                        if let Err(e) = fs::rename(old_pad_path, used_pad_path) {
                            error!("Failed to move used pad: {e}");
                        }
                    }
                }

                println!(
                    "Successfully encrypted file '{}' to '{}'",
                    input.display(),
                    output.display()
                );
                println!("Decryption metadata saved to '{metadata_path}'");
            } else {
                error!("Pad with ID '{pad_id_to_use}' not found.");
            }
        }
        Commands::Decrypt {
            input,
            output,
            metadata,
            pad_id,
            length,
            offset,
        } => {
            let mut state = state_manager::load_state(&vault_path).unwrap_or_else(|e| {
                error!("Failed to load vault state: {e}");
                std::process::exit(1);
            });

            let dec_info = if let Some(meta_path) = metadata {
                let metadata_str = fs::read_to_string(meta_path).unwrap_or_else(|e| {
                    error!("Failed to read metadata file: {e}");
                    std::process::exit(1);
                });
                let meta: CiphertextMetadata =
                    serde_json::from_str(&metadata_str).unwrap_or_else(|e| {
                        error!("Failed to parse metadata file: {e}");
                        std::process::exit(1);
                    });

                let mut hasher = Sha256::new();
                let mut ciphertext_file = fs::File::open(input).unwrap_or_else(|e| {
                    error!("Failed to open ciphertext file: {e}");
                    std::process::exit(1);
                });
                if let Err(e) = std::io::copy(&mut ciphertext_file, &mut hasher) {
                    error!("Failed to hash ciphertext: {e}");
                    std::process::exit(1);
                }
                let calculated_hash = format!("{:x}", hasher.finalize());

                if calculated_hash != meta.ciphertext_hash {
                    error!("Ciphertext hash does not match metadata hash. The file may be corrupt or tampered with. Aborting.");
                    return;
                }
                DecryptionInfo {
                    pad_id: meta.pad_id,
                    start_byte: meta.start_byte,
                    length: meta.length,
                }
            } else {
                DecryptionInfo {
                    pad_id: pad_id.clone().unwrap_or_default(),
                    start_byte: *offset,
                    length: length.unwrap_or_default(),
                }
            };

            if let Some(pad) = state.pads.get_mut(&dec_info.pad_id) {
                let pad_dir = if pad.is_fully_used {
                    "used"
                } else {
                    "available"
                };
                let pad_path = vault_path.join("pads").join(pad_dir).join(&pad.file_name);

                if !pad_path.exists() {
                    error!(
                        "Pad file '{}' not found in vault. It may have been moved or deleted.",
                        pad.file_name
                    );
                    return;
                }

                let mut pad_file = fs::File::open(&pad_path).unwrap_or_else(|e| {
                    error!("Failed to open pad file: {e}");
                    std::process::exit(1);
                });
                if let Err(e) = pad_file.seek(SeekFrom::Start(dec_info.start_byte as u64)) {
                    error!("Failed to seek in pad file: {e}");
                    std::process::exit(1);
                }
                let mut pad_segment = vec![0u8; dec_info.length];
                if let Err(e) = pad_file.read_exact(&mut pad_segment) {
                    error!("Failed to read pad segment: {e}");
                    std::process::exit(1);
                }

                let input_file = fs::File::open(input).unwrap_or_else(|e| {
                    error!("Failed to re-open input file: {e}");
                    std::process::exit(1);
                });
                let mut output_file = fs::File::create(output).unwrap_or_else(|e| {
                    error!("Failed to create output file: {e}");
                    std::process::exit(1);
                });

                let mut reader = std::io::BufReader::new(input_file);
                let mut buffer = [0; 8192];
                let mut total_bytes_processed = 0;
                loop {
                    let bytes_read = reader.read(&mut buffer).unwrap_or_else(|e| {
                        error!("Failed to read from input: {e}");
                        std::process::exit(1);
                    });
                    if bytes_read == 0 {
                        break;
                    }

                    let input_chunk = &buffer[..bytes_read];
                    let pad_chunk =
                        &pad_segment[total_bytes_processed..total_bytes_processed + bytes_read];

                    let mut processed_chunk = Vec::with_capacity(bytes_read);
                    for (i, &byte) in input_chunk.iter().enumerate() {
                        processed_chunk.push(byte ^ pad_chunk[i]);
                    }

                    if let Err(e) = output_file.write_all(&processed_chunk) {
                        error!("Failed to write to output: {e}");
                        std::process::exit(1);
                    }
                    total_bytes_processed += bytes_read;
                }

                let new_segment = state_manager::UsedSegment {
                    start: dec_info.start_byte,
                    end: dec_info.start_byte + dec_info.length,
                };
                if pad.used_segments.iter().all(|s| s.start != new_segment.start || s.end != new_segment.end) {
                    pad.used_segments.push(new_segment);

                    let file_name_clone = pad.file_name.clone();
                    let was_available = !pad.is_fully_used;
                    pad.is_fully_used = pad.total_used_bytes() >= pad.size;
                    let is_full_now = pad.is_fully_used;
                    if let Err(e) = state_manager::save_state(&vault_path, &state) {
                        error!("Failed to save state after decryption: {e}");
                    }

                    if is_full_now && was_available {
                        info!(
                            "Pad '{}' is now fully consumed on receiver side. Moving to 'used' directory.",
                            dec_info.pad_id
                        );
                        let old_pad_path =
                            vault_path.join("pads/available").join(&file_name_clone);
                        let used_pad_path =
                            vault_path.join("pads/used").join(&file_name_clone);
                        if old_pad_path.exists() {
                            if let Err(e) = fs::rename(old_pad_path, used_pad_path) {
                                error!("Failed to move used pad: {e}");
                            }
                        }
                    }
                }

                println!(
                    "Successfully decrypted file '{}' to '{}'",
                    input.display(),
                    output.display()
                );
            } else {
                error!("Pad with ID '{}' not found in vault.", dec_info.pad_id);
            }
        }
    }
}