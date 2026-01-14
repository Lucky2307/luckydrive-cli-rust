use clap::{Parser, Subcommand};
use dotenv::dotenv;
mod commands;
mod config;
use crate::commands::{login::login, logout::logout};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Login { code: String },
    Logout,
    // Upload { file_path: String },
}
fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    let command_result = match cli.command {
        Commands::Login { code } => login(&code),
        Commands::Logout => logout(),
        // Commands::Upload {
        //     file_path: _file_path,
        // } => 
    };
    match command_result {
        Ok(message) => println!("{}", message),
        Err(message) => println!("Error: {}", message)
    }
}
