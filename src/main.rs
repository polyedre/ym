use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

mod cli;
mod yaml_ops;

use cli::{parse_cli, Command};

fn get_terminal_width() -> usize {
    // Try to get terminal width from multiple sources

    // First, try using termsize crate which detects actual terminal size
    if let Some(size) = termsize::get() {
        return size.cols as usize;
    }

    // Fall back to COLUMNS environment variable
    if let Ok(cols) = env::var("COLUMNS") {
        if let Ok(width) = cols.parse::<usize>() {
            return width;
        }
    }

    // Default to 80 columns
    80
}

fn main() {
    let command = match parse_cli() {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = execute_command(command) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn execute_command(command: Command) -> Result<(), String> {
    match command {
        Command::Grep {
            pattern,
            recursive,
            files,
        } => {
            if files.is_empty() {
                // Read from stdin
                grep_stdin(&pattern)?;
            } else {
                // Determine if we should show filename
                // Show filename unless there's exactly 1 file (not directory) in args
                let show_filename = if files.len() == 1 {
                    // Only hide filename if the single arg is a file (not a directory)
                    let path = Path::new(&files[0]);
                    path.is_dir()
                } else {
                    true
                };

                // Search in provided files or directories
                for file in files {
                    grep_path(&file, &pattern, recursive, show_filename)?;
                }
            }
            Ok(())
        }
        Command::Set { file, updates } => {
            let contents = fs::read_to_string(&file)
                .map_err(|e| format!("Failed to read file '{}': {}", file, e))?;

            let mut value = serde_yaml::from_str(&contents)
                .map_err(|e| format!("Failed to parse YAML: {}", e))?;

            yaml_ops::set_values(&mut value, &updates)?;

            let updated_yaml = serde_yaml::to_string(&value)
                .map_err(|e| format!("Failed to serialize YAML: {}", e))?;

            fs::write(&file, updated_yaml)
                .map_err(|e| format!("Failed to write file '{}': {}", file, e))?;

            Ok(())
        }
        Command::Unset { file, keys } => {
            let contents = fs::read_to_string(&file)
                .map_err(|e| format!("Failed to read file '{}': {}", file, e))?;

            let mut value = serde_yaml::from_str(&contents)
                .map_err(|e| format!("Failed to parse YAML: {}", e))?;

            yaml_ops::unset_values(&mut value, &keys)?;

            let updated_yaml = serde_yaml::to_string(&value)
                .map_err(|e| format!("Failed to serialize YAML: {}", e))?;

            fs::write(&file, updated_yaml)
                .map_err(|e| format!("Failed to write file '{}': {}", file, e))?;

            Ok(())
        }
    }
}

fn grep_stdin(pattern: &str) -> Result<(), String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read from stdin: {}", e))?;

    let value = serde_yaml::from_str(&buffer)
        .map_err(|e| format!("Failed to parse YAML from stdin: {}", e))?;

    let results = yaml_ops::grep(&value, pattern)?;
    let width = get_terminal_width();
    for (key, val) in results {
        println!("{}", yaml_ops::format_result(&key, &val, width));
    }
    Ok(())
}

fn grep_path(
    file: &str,
    pattern: &str,
    _recursive: bool,
    show_filename: bool,
) -> Result<(), String> {
    let path = Path::new(file);

    if path.is_file() {
        // If it's a file, search that file
        grep_single(file, pattern, show_filename)
    } else if path.is_dir() {
        // If it's a directory, search it recursively regardless of -R flag
        search_dir(path, pattern, show_filename)
    } else {
        Err(format!("'{}' is not a file or directory", file))
    }
}

fn grep_single(file: &str, pattern: &str, show_filename: bool) -> Result<(), String> {
    let contents =
        fs::read_to_string(file).map_err(|e| format!("Failed to read file '{}': {}", file, e))?;

    let value =
        serde_yaml::from_str(&contents).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let results = yaml_ops::grep(&value, pattern)?;
    let width = get_terminal_width();
    for (key, val) in results {
        if show_filename {
            println!("{}:{}", file, yaml_ops::format_result(&key, &val, width));
        } else {
            println!("{}", yaml_ops::format_result(&key, &val, width));
        }
    }
    Ok(())
}

fn search_dir(dir: &Path, pattern: &str, show_filename: bool) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively search subdirectories
            search_dir(&path, pattern, show_filename)?;
        } else if path.is_file() {
            // Process YAML files
            if should_process_file(&path) {
                if let Err(e) = grep_file_with_name(&path, pattern, show_filename) {
                    eprintln!("Warning: {}", e);
                }
            }
        }
    }

    Ok(())
}

fn should_process_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        ext == "yaml" || ext == "yml"
    } else {
        false
    }
}

fn grep_file_with_name(file: &Path, pattern: &str, show_filename: bool) -> Result<(), String> {
    let file_str = file.to_string_lossy();
    let contents = fs::read_to_string(file)
        .map_err(|e| format!("Failed to read file '{}': {}", file_str, e))?;

    let value = serde_yaml::from_str(&contents)
        .map_err(|e| format!("Failed to parse YAML in '{}': {}", file_str, e))?;

    let results = yaml_ops::grep(&value, pattern)?;
    let width = get_terminal_width();
    for (key, val) in results {
        if show_filename {
            println!(
                "{}:{}",
                file_str,
                yaml_ops::format_result(&key, &val, width)
            );
        } else {
            println!("{}", yaml_ops::format_result(&key, &val, width));
        }
    }
    Ok(())
}
