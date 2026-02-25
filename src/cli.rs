use clap::{Parser, Subcommand};

use crate::error::{AppError, AppResult};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Grep {
        pattern: String,
        recursive: bool,
        files: Vec<String>,
    },
    Set {
        file: String,
        updates: Vec<(String, String)>,
    },
    Unset {
        file: String,
        keys: Vec<String>,
    },
    Cp {
        source_file: String,
        source_key: String,
        dest_file: Option<String>,
        dest_key: Option<String>,
    },
    Mv {
        source_file: String,
        source_key: String,
        dest_file: Option<String>,
        dest_key: Option<String>,
    },
}

#[derive(Parser, Debug)]
#[command(name = "ym")]
#[command(about = "A YAML search and patch tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Grep {
        pattern: String,

        #[arg(short = 'R')]
        recursive: bool,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        files: Vec<String>,
    },
    Set {
        file: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        updates: Vec<String>,
    },
    Unset {
        file: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        keys: Vec<String>,
    },
    Cp {
        source: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        destination: Vec<String>,
    },
    Mv {
        source: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        destination: Vec<String>,
    },
}

pub fn parse_cli() -> AppResult<Command> {
    command_from_cli(Cli::parse())
}

fn command_from_cli(cli: Cli) -> AppResult<Command> {
    command_from_parsed(cli.command)
}

fn command_from_parsed(command: Commands) -> AppResult<Command> {
    match command {
        Commands::Grep {
            pattern,
            recursive,
            files,
        } => Ok(Command::Grep {
            pattern,
            recursive,
            files,
        }),
        Commands::Set { file, updates } => Ok(Command::Set {
            file,
            updates: parse_updates(updates)?,
        }),
        Commands::Unset { file, keys } => {
            if keys.is_empty() {
                return Err(AppError::cli("unset requires at least one key"));
            }

            Ok(Command::Unset { file, keys })
        }
        Commands::Cp {
            source,
            destination,
        } => {
            let transfer = parse_transfer_command(source, destination, "cp")?;
            Ok(Command::Cp {
                source_file: transfer.source_file,
                source_key: transfer.source_key,
                dest_file: transfer.dest_file,
                dest_key: transfer.dest_key,
            })
        }
        Commands::Mv {
            source,
            destination,
        } => {
            let transfer = parse_transfer_command(source, destination, "mv")?;
            Ok(Command::Mv {
                source_file: transfer.source_file,
                source_key: transfer.source_key,
                dest_file: transfer.dest_file,
                dest_key: transfer.dest_key,
            })
        }
    }
}

fn parse_updates(updates: Vec<String>) -> AppResult<Vec<(String, String)>> {
    if updates.is_empty() {
        return Err(AppError::cli("set requires at least one key=value pair"));
    }

    updates
        .into_iter()
        .map(|update| {
            let (key, value) = update
                .split_once('=')
                .ok_or_else(|| AppError::cli(format!("Invalid key=value pair: {update}")))?;
            Ok((key.to_string(), value.to_string()))
        })
        .collect()
}

struct TransferCommand {
    source_file: String,
    source_key: String,
    dest_file: Option<String>,
    dest_key: Option<String>,
}

fn parse_transfer_command(
    source: String,
    destination: Vec<String>,
    name: &str,
) -> AppResult<TransferCommand> {
    let (source_file, source_key) = parse_file_key_pair(&source)?;

    let (dest_file, dest_key) = match destination.as_slice() {
        [] => (None, None),
        [single] => parse_optional_file_key_pair(single)?,
        _ => {
            return Err(AppError::cli(format!(
                "{name} accepts at most one destination argument"
            )));
        }
    };

    if dest_file.is_none() && dest_key.is_none() {
        return Err(AppError::cli(
            "destination file and destination key cannot both be omitted",
        ));
    }

    Ok(TransferCommand {
        source_file,
        source_key,
        dest_file,
        dest_key,
    })
}

fn parse_file_key_pair(input: &str) -> AppResult<(String, String)> {
    let Some((file, key)) = input.split_once(':') else {
        return Err(AppError::cli(format!(
            "Invalid file:key pair: {input} (expected format: file.yaml:key.path)"
        )));
    };

    if file.is_empty() || key.is_empty() {
        return Err(AppError::cli(format!(
            "Invalid file:key pair: {input} (expected format: file.yaml:key.path)"
        )));
    }

    Ok((file.to_string(), key.to_string()))
}

fn parse_optional_file_key_pair(input: &str) -> AppResult<(Option<String>, Option<String>)> {
    if let Some((file, key)) = input.split_once(':') {
        if file.is_empty() && key.is_empty() {
            return Err(AppError::cli(format!(
                "Invalid file:key pair: {input} (file and key cannot both be empty)"
            )));
        }

        return Ok((
            (!file.is_empty()).then(|| file.to_string()),
            (!key.is_empty()).then(|| key.to_string()),
        ));
    }

    if input.is_empty() {
        return Err(AppError::cli("Key cannot be empty"));
    }

    if looks_like_yaml_file_path(input) {
        Ok((Some(input.to_string()), None))
    } else {
        Ok((None, Some(input.to_string())))
    }
}

fn looks_like_yaml_file_path(input: &str) -> bool {
    input.ends_with(".yaml") || input.ends_with(".yml")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_with_args(args: Vec<&str>) -> AppResult<Command> {
        let cli = Cli::try_parse_from(args).map_err(|error| AppError::cli(error.to_string()))?;
        command_from_cli(cli)
    }

    #[test]
    fn test_parse_grep_simple() {
        let cmd = test_with_args(vec!["ym", "grep", "pattern", "file.yaml"]).unwrap();

        assert_eq!(
            cmd,
            Command::Grep {
                pattern: "pattern".to_string(),
                recursive: false,
                files: vec!["file.yaml".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_grep_with_recursive_flag() {
        let cmd = test_with_args(vec!["ym", "grep", "-R", "pattern", "dir"]).unwrap();

        assert_eq!(
            cmd,
            Command::Grep {
                pattern: "pattern".to_string(),
                recursive: true,
                files: vec!["dir".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_grep_multiple_files() {
        let cmd = test_with_args(vec![
            "ym",
            "grep",
            "pattern",
            "file1.yaml",
            "file2.yaml",
            "file3.yaml",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Grep {
                pattern: "pattern".to_string(),
                recursive: false,
                files: vec![
                    "file1.yaml".to_string(),
                    "file2.yaml".to_string(),
                    "file3.yaml".to_string(),
                ],
            }
        );
    }

    #[test]
    fn test_parse_grep_no_pattern_error() {
        assert!(test_with_args(vec!["ym", "grep"]).is_err());
    }

    #[test]
    fn test_parse_grep_recursive_no_pattern_error() {
        assert!(test_with_args(vec!["ym", "grep", "-R"]).is_err());
    }

    #[test]
    fn test_parse_grep_no_files_allowed() {
        let cmd = test_with_args(vec!["ym", "grep", "pattern"]).unwrap();

        assert_eq!(
            cmd,
            Command::Grep {
                pattern: "pattern".to_string(),
                recursive: false,
                files: Vec::new(),
            }
        );
    }

    #[test]
    fn test_parse_set_single_key_value() {
        let cmd = test_with_args(vec!["ym", "set", "file.yaml", "key=value"]).unwrap();

        assert_eq!(
            cmd,
            Command::Set {
                file: "file.yaml".to_string(),
                updates: vec![("key".to_string(), "value".to_string())],
            }
        );
    }

    #[test]
    fn test_parse_set_multiple_key_values() {
        let cmd = test_with_args(vec![
            "ym",
            "set",
            "file.yaml",
            "key1=value1",
            "key2=value2",
            "key3=value3",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Set {
                file: "file.yaml".to_string(),
                updates: vec![
                    ("key1".to_string(), "value1".to_string()),
                    ("key2".to_string(), "value2".to_string()),
                    ("key3".to_string(), "value3".to_string()),
                ],
            }
        );
    }

    #[test]
    fn test_parse_set_nested_key_path() {
        let cmd = test_with_args(vec![
            "ym",
            "set",
            "file.yaml",
            "database.host=localhost",
            "database.port=5432",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Set {
                file: "file.yaml".to_string(),
                updates: vec![
                    ("database.host".to_string(), "localhost".to_string()),
                    ("database.port".to_string(), "5432".to_string()),
                ],
            }
        );
    }

    #[test]
    fn test_parse_set_value_with_equals() {
        let cmd = test_with_args(vec![
            "ym",
            "set",
            "file.yaml",
            "url=http://example.com?param=value",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Set {
                file: "file.yaml".to_string(),
                updates: vec![(
                    "url".to_string(),
                    "http://example.com?param=value".to_string(),
                )],
            }
        );
    }

    #[test]
    fn test_parse_set_no_file_error() {
        assert!(test_with_args(vec!["ym", "set"]).is_err());
    }

    #[test]
    fn test_parse_set_no_key_value_error() {
        assert!(test_with_args(vec!["ym", "set", "file.yaml"]).is_err());
    }

    #[test]
    fn test_parse_set_invalid_key_value_format() {
        let result = test_with_args(vec!["ym", "set", "file.yaml", "invalid_no_equals"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid key=value pair"));
    }

    #[test]
    fn test_parse_unset_single_key() {
        let cmd = test_with_args(vec!["ym", "unset", "file.yaml", "key"]).unwrap();

        assert_eq!(
            cmd,
            Command::Unset {
                file: "file.yaml".to_string(),
                keys: vec!["key".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_unset_multiple_keys() {
        let cmd = test_with_args(vec!["ym", "unset", "file.yaml", "key1", "key2", "key3"]).unwrap();

        assert_eq!(
            cmd,
            Command::Unset {
                file: "file.yaml".to_string(),
                keys: vec!["key1".to_string(), "key2".to_string(), "key3".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_unset_nested_key_path() {
        let cmd = test_with_args(vec![
            "ym",
            "unset",
            "file.yaml",
            "database.password",
            "database.username",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Unset {
                file: "file.yaml".to_string(),
                keys: vec![
                    "database.password".to_string(),
                    "database.username".to_string(),
                ],
            }
        );
    }

    #[test]
    fn test_parse_unset_no_file_error() {
        assert!(test_with_args(vec!["ym", "unset"]).is_err());
    }

    #[test]
    fn test_parse_unset_no_keys_error() {
        assert!(test_with_args(vec!["ym", "unset", "file.yaml"]).is_err());
    }

    #[test]
    fn test_parse_cp_same_file_same_key() {
        let result = test_with_args(vec!["ym", "cp", "file.yaml:source.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("destination file and destination key cannot both be omitted"));
    }

    #[test]
    fn test_parse_cp_same_file_different_key() {
        let cmd = test_with_args(vec!["ym", "cp", "file.yaml:source.key", "dest.key"]).unwrap();

        assert_eq!(
            cmd,
            Command::Cp {
                source_file: "file.yaml".to_string(),
                source_key: "source.key".to_string(),
                dest_file: None,
                dest_key: Some("dest.key".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_cp_different_file_same_key() {
        let cmd = test_with_args(vec!["ym", "cp", "source.yaml:mykey", "dest.yaml:mykey"]).unwrap();

        assert_eq!(
            cmd,
            Command::Cp {
                source_file: "source.yaml".to_string(),
                source_key: "mykey".to_string(),
                dest_file: Some("dest.yaml".to_string()),
                dest_key: Some("mykey".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_cp_different_file_different_key() {
        let cmd = test_with_args(vec![
            "ym",
            "cp",
            "source.yaml:source.key",
            "dest.yaml:dest.key",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Cp {
                source_file: "source.yaml".to_string(),
                source_key: "source.key".to_string(),
                dest_file: Some("dest.yaml".to_string()),
                dest_key: Some("dest.key".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_cp_only_destination_file() {
        let cmd = test_with_args(vec!["ym", "cp", "source.yaml:mykey", "dest.yaml:"]).unwrap();

        assert_eq!(
            cmd,
            Command::Cp {
                source_file: "source.yaml".to_string(),
                source_key: "mykey".to_string(),
                dest_file: Some("dest.yaml".to_string()),
                dest_key: None,
            }
        );
    }

    #[test]
    fn test_parse_cp_bare_destination_file_defaults_to_source_key() {
        let cmd = test_with_args(vec![
            "ym",
            "cp",
            "tests/data/config-prod.yaml:environment",
            "tests/data/config-dev.yaml",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Cp {
                source_file: "tests/data/config-prod.yaml".to_string(),
                source_key: "environment".to_string(),
                dest_file: Some("tests/data/config-dev.yaml".to_string()),
                dest_key: None,
            }
        );
    }

    #[test]
    fn test_parse_cp_missing_source_key() {
        let result = test_with_args(vec!["ym", "cp", "source.yaml", "dest.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file:key pair"));
    }

    #[test]
    fn test_parse_cp_invalid_source_format() {
        let result = test_with_args(vec!["ym", "cp", "invalid", "dest.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file:key pair"));
    }

    #[test]
    fn test_parse_cp_too_many_arguments() {
        let result = test_with_args(vec![
            "ym",
            "cp",
            "source.yaml:key",
            "dest1.yaml:key",
            "dest2.yaml:key",
        ]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cp accepts at most one destination argument"));
    }

    #[test]
    fn test_parse_mv_same_file_same_key() {
        let result = test_with_args(vec!["ym", "mv", "file.yaml:source.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("destination file and destination key cannot both be omitted"));
    }

    #[test]
    fn test_parse_mv_same_file_different_key() {
        let cmd = test_with_args(vec!["ym", "mv", "file.yaml:source.key", "dest.key"]).unwrap();

        assert_eq!(
            cmd,
            Command::Mv {
                source_file: "file.yaml".to_string(),
                source_key: "source.key".to_string(),
                dest_file: None,
                dest_key: Some("dest.key".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_mv_different_file_same_key() {
        let cmd = test_with_args(vec!["ym", "mv", "source.yaml:mykey", "dest.yaml:mykey"]).unwrap();

        assert_eq!(
            cmd,
            Command::Mv {
                source_file: "source.yaml".to_string(),
                source_key: "mykey".to_string(),
                dest_file: Some("dest.yaml".to_string()),
                dest_key: Some("mykey".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_mv_different_file_different_key() {
        let cmd = test_with_args(vec![
            "ym",
            "mv",
            "source.yaml:source.key",
            "dest.yaml:dest.key",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            Command::Mv {
                source_file: "source.yaml".to_string(),
                source_key: "source.key".to_string(),
                dest_file: Some("dest.yaml".to_string()),
                dest_key: Some("dest.key".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_mv_only_destination_file() {
        let cmd = test_with_args(vec!["ym", "mv", "source.yaml:mykey", "dest.yaml:"]).unwrap();

        assert_eq!(
            cmd,
            Command::Mv {
                source_file: "source.yaml".to_string(),
                source_key: "mykey".to_string(),
                dest_file: Some("dest.yaml".to_string()),
                dest_key: None,
            }
        );
    }

    #[test]
    fn test_parse_mv_missing_source_key() {
        let result = test_with_args(vec!["ym", "mv", "source.yaml", "dest.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file:key pair"));
    }

    #[test]
    fn test_parse_mv_invalid_source_format() {
        let result = test_with_args(vec!["ym", "mv", "invalid", "dest.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file:key pair"));
    }

    #[test]
    fn test_parse_mv_too_many_arguments() {
        let result = test_with_args(vec![
            "ym",
            "mv",
            "source.yaml:key",
            "dest1.yaml:key",
            "dest2.yaml:key",
        ]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("mv accepts at most one destination argument"));
    }
}
