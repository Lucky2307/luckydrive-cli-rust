use clap::{Parser, Subcommand};
use dotenv::dotenv;
use ffmpeg_sidecar::paths::ffmpeg_path;
mod commands;
mod config;
mod token;
use crate::commands::{login::login, logout::logout, upload::upload};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Login { code: String },
    Logout,
    Upload { file_path: String },
}
fn main() {
    dotenv().ok();
    ffmpeg_path();
    let cli = Cli::parse();
    let command_result = match cli.command {
        Commands::Login { code } => login(&code),
        Commands::Logout => logout(),
        Commands::Upload { file_path } => upload(&file_path),
    };
    match command_result {
        Ok(message) => println!("{}", message),
        Err(message) => println!("Error: {}", message),
    }
}
