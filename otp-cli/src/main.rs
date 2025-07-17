use clap::{Parser, Subcommand};
use log::{info, error};

mod pad_generator;

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

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { path, size } => {
            let size_in_bytes = size * 1024 * 1024;
            info!("Generating a new pad file at '{}' with size {} MB.", path, size);
            match pad_generator::generate_pad(path, size_in_bytes) {
                Ok(_) => info!("Successfully generated pad file."),
                Err(e) => error!("Failed to generate pad file: {}", e),
            }
        }
    }
}
