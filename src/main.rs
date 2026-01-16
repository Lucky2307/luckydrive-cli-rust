use std::panic;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use ffmpeg_sidecar::{ffprobe::ffprobe_path, paths::ffmpeg_path};
mod commands;
mod config;
mod spinner;
mod token;
use crate::commands::{login::login, logout::logout, upload::upload};

fn get_ffmpeg_version() -> String {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap()
}

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long)]
    ffmpeg_version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Login { code: String },
    Logout,
    Upload { file_path: String },
}
fn main() {
    panic::set_hook(Box::new(|info| {
        eprintln!("Application panicked: {}", info.payload_as_str().unwrap());
    }));
    dotenv().ok();
    ffmpeg_path();
    let cli = Cli::parse();
    if cli.ffmpeg_version {
        println!("Ffmpeg Path: {}", ffmpeg_path().display());
        println!("Ffprobe Path: {}", ffprobe_path().display());
        println!("{}", get_ffmpeg_version());
        return;
    }
    let command_result = match cli.command {
        Some(Commands::Login { code }) => login(&code),
        Some(Commands::Logout) => logout(),
        Some(Commands::Upload { file_path }) => upload(&file_path),
        None => Ok("No command provided.".to_string()),
    };
    match command_result {
        Ok(message) => println!("{}", message),
        Err(message) => println!("Error: {}", message),
    }
}
