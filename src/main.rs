use clap::{Parser, Subcommand};
use dotenv::dotenv;
mod commands;
use crate::commands::login::login;

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
    let cli = Cli::parse();
    match cli.command {
        Commands::Login { code } => login(&code),
        Commands::Logout => println!("Logout!"),
        Commands::Upload { file_path: _file_path } => println!("Upload"),
    }
}
