mod gpt_query;

use clap::{Parser, Subcommand};
use std::io::Read;
use anyhow::{Result, Context};
use dirs;
use std::fs;
use gpt_query::query_gpt;

#[derive(Parser)]
#[command(name = "quickref")]
#[command(author = "Sam Uherek <samuherekbiz@gmail.com>")]
#[command(about = "Quick reference to commands", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add {},
    List
}


fn add_command() -> Result<String>{
    let history =  dirs::home_dir().expect("Can not find home dir").join(".zsh_history");
    println!("{:?}", history);

    // let mut file_content = Vec::new();
    // let mut file = File::open(&file_name).expect("Unable to open file");
    // file.read_to_end(&mut file_content).expect("Unable to read");
    // file_content

    let file = fs::read(&history).with_context(|| format!("Unable to read {:?}", &history))?;
    let content = String::from_utf8_lossy(&file);
    let command = content.lines().last().unwrap_or("");

    println!("{:?}", command);

    let res = query_gpt();
    println!("QUERY GPT:: {:?}", res);

    // let data = fs::read_to_string(&history).expect("can read to string");
    // let command = data.lines().last().unwrap_or("");
    // println!("{}", command);

    return Ok("".to_string());
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add {}) => {
            println!("add stuff");
            add_command().expect("Should exec the add command");
        },
        Some(Commands::List) => {
            println!("list stuff");
        },
        None => {
            println!("None");
        }
    }
}
