use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

mod cli;
mod error;
mod path;
mod yaml_ops;

use cli::{parse_cli, Command};
use error::{AppError, AppResult};
use yaml_ops::GrepOutputMode;

fn get_terminal_width() -> usize {
    if let Some(size) = termsize::get() {
        return size.cols as usize;
    }

    if let Ok(cols) = env::var("COLUMNS") {
        if let Ok(width) = cols.parse::<usize>() {
            return width;
        }
    }

    80
}

fn no_matches_error() -> AppError {
    AppError::message("No matches found")
}

fn is_no_matches_error(error: &AppError) -> bool {
    matches!(error, AppError::Message(message) if message == "No matches found")
}

fn main() {
    let command = match parse_cli() {
        Ok(command) => command,
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1);
        }
    };

    match execute_command(command) {
        Ok(()) => {}
        Err(error) if is_no_matches_error(&error) => {
            process::exit(2);
        }
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1);
        }
    }
}

fn execute_command(command: Command) -> AppResult<()> {
    match command {
        Command::Grep {
            pattern,
            recursive,
            full,
            files,
        } => run_grep(&pattern, recursive, full, &files),
        Command::Set { file, updates } => {
            apply_file_update(&file, |contents| yaml_ops::set_values(contents, &updates))
        }
        Command::Unset { file, keys } => {
            apply_file_update(&file, |contents| yaml_ops::unset_values(contents, &keys))
        }
        Command::Cp {
            source_file,
            source_key,
            dest_file,
            dest_key,
        } => {
            let final_dest_file = dest_file.unwrap_or_else(|| source_file.clone());
            let final_dest_key = dest_key.unwrap_or_else(|| source_key.clone());
            yaml_ops::copy_value(&source_file, &source_key, &final_dest_file, &final_dest_key)
        }
        Command::Mv {
            source_file,
            source_key,
            dest_file,
            dest_key,
        } => {
            let final_dest_file = dest_file.unwrap_or_else(|| source_file.clone());
            let final_dest_key = dest_key.unwrap_or_else(|| source_key.clone());
            yaml_ops::move_value(&source_file, &source_key, &final_dest_file, &final_dest_key)
        }
    }
}

fn apply_file_update<F>(file: &str, update: F) -> AppResult<()>
where
    F: FnOnce(&str) -> AppResult<String>,
{
    let contents = fs::read_to_string(file).map_err(|error| AppError::read_file(file, error))?;
    let updated = update(&contents)?;
    fs::write(file, updated).map_err(|error| AppError::write_file(file, error))?;
    Ok(())
}

fn run_grep(pattern: &str, recursive: bool, full: bool, files: &[String]) -> AppResult<()> {
    let output_mode = if full {
        GrepOutputMode::Full
    } else {
        GrepOutputMode::Inline
    };

    if files.is_empty() {
        return grep_stdin(pattern, output_mode);
    }

    let show_filename = should_show_filename(files, output_mode);
    let mut found_any = false;

    for file in files {
        match grep_path(
            Path::new(file),
            pattern,
            recursive,
            show_filename,
            output_mode,
        ) {
            Ok(()) => found_any = true,
            Err(error) if is_no_matches_error(&error) => {}
            Err(error) => return Err(error),
        }
    }

    if found_any {
        Ok(())
    } else {
        Err(no_matches_error())
    }
}

fn should_show_filename(files: &[String], output_mode: GrepOutputMode) -> bool {
    if matches!(output_mode, GrepOutputMode::Full) {
        return true;
    }

    if files.len() != 1 {
        return true;
    }

    Path::new(&files[0]).is_dir()
}

fn grep_stdin(pattern: &str, output_mode: GrepOutputMode) -> AppResult<()> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(AppError::ReadStdin)?;

    if buffer.trim().is_empty() {
        return Err(AppError::message("No input provided"));
    }

    let value =
        serde_yaml::from_str(&buffer).map_err(|error| AppError::parse_yaml("from stdin", error))?;
    print_grep_results(None, pattern, &value, output_mode)
}

fn grep_path(
    path: &Path,
    pattern: &str,
    recursive: bool,
    show_filename: bool,
    output_mode: GrepOutputMode,
) -> AppResult<()> {
    if path.is_file() {
        return grep_file(path, pattern, show_filename, output_mode);
    }

    if path.is_dir() {
        return if recursive {
            search_dir(path, pattern, show_filename, output_mode)
        } else {
            Err(AppError::message(format!(
                "'{}' is a directory (use -R to search recursively)",
                path.display()
            )))
        };
    }

    Err(AppError::message(format!(
        "'{}' is not a file or directory",
        path.display()
    )))
}

fn grep_file(
    path: &Path,
    pattern: &str,
    show_filename: bool,
    output_mode: GrepOutputMode,
) -> AppResult<()> {
    let display = path.to_string_lossy();
    let contents =
        fs::read_to_string(path).map_err(|error| AppError::read_file(display.as_ref(), error))?;
    let value = serde_yaml::from_str(&contents)
        .map_err(|error| AppError::parse_yaml(format!("in '{display}'"), error))?;

    print_grep_results(
        show_filename.then_some(display.as_ref()),
        pattern,
        &value,
        output_mode,
    )
}

fn print_grep_results(
    filename: Option<&str>,
    pattern: &str,
    value: &serde_yaml::Value,
    output_mode: GrepOutputMode,
) -> AppResult<()> {
    let results = yaml_ops::grep(value, pattern)?;
    if results.is_empty() {
        return Err(no_matches_error());
    }

    let width = get_terminal_width();

    for (key, value) in results {
        print_grep_result(filename, &key, &value, width, output_mode);
    }

    Ok(())
}

fn print_grep_result(
    filename: Option<&str>,
    key: &str,
    value: &serde_yaml::Value,
    width: usize,
    output_mode: GrepOutputMode,
) {
    let formatted = yaml_ops::format_result(key, value, width, output_mode);

    match (filename, output_mode) {
        (Some(filename), GrepOutputMode::Inline) => println!("{filename}:{formatted}"),
        (Some(filename), GrepOutputMode::Full) => println!("--- {filename} ---\n{formatted}"),
        (None, _) => println!("{formatted}"),
    }
}

fn search_dir(
    dir: &Path,
    pattern: &str,
    show_filename: bool,
    output_mode: GrepOutputMode,
) -> AppResult<()> {
    let entries =
        fs::read_dir(dir).map_err(|error| AppError::read_dir(dir.display().to_string(), error))?;

    let mut found_any = false;

    for entry in entries {
        let entry = entry.map_err(AppError::ReadDirEntry)?;
        let path = entry.path();

        if path.is_dir() {
            match search_dir(&path, pattern, show_filename, output_mode) {
                Ok(()) => found_any = true,
                Err(error) if is_no_matches_error(&error) => {}
                Err(error) => return Err(error),
            }
        } else if path.is_file() && should_process_file(&path) {
            match grep_file(&path, pattern, show_filename, output_mode) {
                Ok(()) => found_any = true,
                Err(error) if is_no_matches_error(&error) => {}
                Err(error) => return Err(error),
            }
        }
    }

    if found_any {
        Ok(())
    } else {
        Err(no_matches_error())
    }
}

fn should_process_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("yaml" | "yml")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ym_{}_{}_{}", name, process::id(), unique));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn read_yaml(path: &Path) -> Value {
        serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn test_set_and_unset_commands_update_yaml_semantics() {
        let dir = temp_dir("set_unset");
        let file = dir.join("config.yaml");
        fs::write(
            &file,
            "# config\nname  : \"Alice\"   # keep quote/comment\nstatus: enabled\ncount: 1\n",
        )
        .unwrap();

        execute_command(Command::Set {
            file: file.display().to_string(),
            updates: vec![
                ("name".to_string(), "Bob".to_string()),
                ("count".to_string(), "2".to_string()),
            ],
        })
        .unwrap();

        let updated = read_yaml(&file);
        assert_eq!(updated["name"].as_str(), Some("Bob"));
        assert_eq!(updated["status"].as_str(), Some("enabled"));
        assert_eq!(updated["count"].as_i64(), Some(2));

        execute_command(Command::Unset {
            file: file.display().to_string(),
            keys: vec!["count".to_string()],
        })
        .unwrap();

        let updated = read_yaml(&file);
        assert!(updated.get("count").is_none());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_cp_command_copies_value_between_files() {
        let dir = temp_dir("cp_semantics");
        let source = dir.join("source.yaml");
        let dest = dir.join("dest.yaml");

        fs::write(&source, "source:\n  value: copied\n  enabled: true\n").unwrap();
        fs::write(&dest, "existing: item\n").unwrap();

        execute_command(Command::Cp {
            source_file: source.display().to_string(),
            source_key: "source".to_string(),
            dest_file: Some(dest.display().to_string()),
            dest_key: Some("copied.settings".to_string()),
        })
        .unwrap();

        let dest_yaml = read_yaml(&dest);
        assert_eq!(
            dest_yaml["copied"]["settings"]["value"].as_str(),
            Some("copied")
        );
        assert_eq!(
            dest_yaml["copied"]["settings"]["enabled"].as_bool(),
            Some(true)
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_mv_command_moves_value_between_files() {
        let dir = temp_dir("mv_semantics");
        let source = dir.join("source.yaml");
        let dest = dir.join("dest.yaml");

        fs::write(&source, "move_me:\n  key: value\nkeep: still\n").unwrap();
        fs::write(&dest, "name: app\n").unwrap();

        execute_command(Command::Mv {
            source_file: source.display().to_string(),
            source_key: "move_me".to_string(),
            dest_file: Some(dest.display().to_string()),
            dest_key: Some("new_key".to_string()),
        })
        .unwrap();

        let source_yaml = read_yaml(&source);
        let dest_yaml = read_yaml(&dest);
        assert_eq!(source_yaml["keep"].as_str(), Some("still"));
        assert!(source_yaml.get("move_me").is_none());
        assert_eq!(dest_yaml["new_key"]["key"].as_str(), Some("value"));

        fs::remove_dir_all(dir).unwrap();
    }
}
