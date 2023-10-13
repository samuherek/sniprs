use anyhow::anyhow;
use std::path::PathBuf;
use std::{env, fs};


fn get_history_path() -> anyhow::Result<PathBuf> {
    let shell_path = env::var("SHELL")?;
    let shell_name = shell_path.rsplit('/').next().unwrap_or("");
    let home_dir = dirs::home_dir().expect("Could not determine home dir");

    return match shell_name {
        "bash" => Ok(home_dir.join(".bash_history")),
        "zsh" => Ok(home_dir.join(".zsh_history")),
        _ => Err(anyhow!("We could not find your shell.")),
    }
}

/// Read the .zsh_history and get the last x lines from it.
pub fn get_history(line_count: usize) -> anyhow::Result<Vec<String>> {
    let history_path = get_history_path()?;
    let data: Vec<String> = String::from_utf8_lossy(&fs::read(&history_path)?)
        .lines()
        .rev()
        .take(line_count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|l| l.to_string())
        .collect();

    return Ok(data);
}
