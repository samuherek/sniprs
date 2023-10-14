use dirs;
use anyhow::anyhow;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::path::PathBuf;
use std::hash::{Hash, Hasher};

const DOTFOLDER_PATH: &str = ".sniprs";


fn config_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home dir");     
    let config_path = home_dir.join(DOTFOLDER_PATH);

    return config_path;
}

pub fn init_dotfolder() -> anyhow::Result<()> {
    let config_path = config_dir();

    if !config_path.exists() {
       fs::create_dir(&config_path)?;
    }

    return Ok(());
}

fn create_file_name(command: &str) -> String {
    let command_name = command.split_whitespace().next().unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    command.hash(&mut hasher);
    let command_hash = hasher.finish();
    
    return format!("{}-{:x}.md", command_name, command_hash);
}

pub fn save_command(command: &str) -> anyhow::Result<()> {
    let file_path = config_dir().join(create_file_name(&command));

    if !file_path.exists() {
        let data = format!("# {} \n\n", command);
        fs::write(&file_path, data);
    }

    return Ok(());
}
