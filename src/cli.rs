use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author = "Sam Uherek <samuherekbiz@gmail.com>")]
#[command(about = "Quick reference to commands", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add,
    List
}
