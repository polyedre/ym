use clap::{Parser, Subcommand};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Command {
    Grep {
        pattern: String,
        recursive: bool,
        files: Vec<String>,
    },
    Set {
        file: String,
        updates: HashMap<String, String>,
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
    /// Search YAML keys by regex pattern (reads stdin if no files provided)
    Grep {
        /// Pattern to search for
        pattern: String,

        /// Recursive search in directories
        #[arg(short = 'R')]
        recursive: bool,

        /// Files or directories to search (if empty, reads from stdin)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        files: Vec<String>,
    },
    /// Set YAML values at key paths
    Set {
        /// File to modify
        file: String,

        /// Key=value pairs to set (values can contain '=')
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        updates: Vec<String>,
    },
    /// Remove keys from YAML
    Unset {
        /// File to modify
        file: String,

        /// Keys to remove (support nested paths like database.password)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        keys: Vec<String>,
    },
    /// Copy a value from one key to another (same or different file)
    Cp {
        /// Source file and key in format: file.yaml:key.path
        source: String,

        /// Destination file and key in format: file.yaml:key.path (optional)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        destination: Vec<String>,
    },
    /// Move a value from one key to another (deletes source after copying)
    Mv {
        /// Source file and key in format: file.yaml:key.path
        source: String,

        /// Destination file and key in format: file.yaml:key.path (optional)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        destination: Vec<String>,
    },
}

pub fn parse_cli() -> Result<Command, String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Grep {
            pattern,
            recursive,
            files,
        } => Ok(Command::Grep {
            pattern,
            recursive,
            files,
        }),
        Commands::Set { file, updates } => {
            if updates.is_empty() {
                return Err("set requires at least one key=value pair".to_string());
            }

            let mut parsed_updates = HashMap::new();

            for update in updates {
                let parts: Vec<&str> = update.splitn(2, '=').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid key=value pair: {}", update));
                }
                parsed_updates.insert(parts[0].to_string(), parts[1].to_string());
            }

            Ok(Command::Set {
                file,
                updates: parsed_updates,
            })
        }
        Commands::Unset { file, keys } => {
            if keys.is_empty() {
                return Err("unset requires at least one key".to_string());
            }

            Ok(Command::Unset { file, keys })
        }
        Commands::Cp {
            source,
            destination,
        } => {
            // Parse source as file:key
            let (source_file, source_key) = parse_file_key_pair(&source)?;

            // Parse destination if provided
            let (dest_file, dest_key) = if destination.is_empty() {
                // No destination provided: error if neither file nor key changes
                (None, None)
            } else if destination.len() == 1 {
                // Single destination argument
                parse_optional_file_key_pair(&destination[0])?
            } else {
                return Err("cp accepts at most one destination argument".to_string());
            };

            // Validate that at least one of dest_file or dest_key is provided
            if dest_file.is_none() && dest_key.is_none() {
                return Err(
                    "destination file and destination key cannot both be omitted".to_string(),
                );
            }

            Ok(Command::Cp {
                source_file,
                source_key,
                dest_file,
                dest_key,
            })
        }
        Commands::Mv {
            source,
            destination,
        } => {
            // Parse source as file:key
            let (source_file, source_key) = parse_file_key_pair(&source)?;

            // Parse destination if provided
            let (dest_file, dest_key) = if destination.is_empty() {
                // No destination provided: error if neither file nor key changes
                (None, None)
            } else if destination.len() == 1 {
                // Single destination argument
                parse_optional_file_key_pair(&destination[0])?
            } else {
                return Err("mv accepts at most one destination argument".to_string());
            };

            // Validate that at least one of dest_file or dest_key is provided
            if dest_file.is_none() && dest_key.is_none() {
                return Err(
                    "destination file and destination key cannot both be omitted".to_string(),
                );
            }

            Ok(Command::Mv {
                source_file,
                source_key,
                dest_file,
                dest_key,
            })
        }
    }
}

/// Parse a required file:key pair
fn parse_file_key_pair(input: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(format!(
            "Invalid file:key pair: {} (expected format: file.yaml:key.path)",
            input
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Parse an optional file:key pair. Returns (None, None) if input is just a key (no colon),
/// or (Some(file), Some(key)) if both are provided.
/// If the input has a colon, file and key can be filled independently (at least one must be non-empty).
fn parse_optional_file_key_pair(input: &str) -> Result<(Option<String>, Option<String>), String> {
    if let Some(colon_pos) = input.find(':') {
        let file = &input[..colon_pos];
        let key = &input[colon_pos + 1..];

        // Ensure at least file or key is non-empty, and neither is exactly "invalid:invalid:too:many" pattern
        if file.is_empty() && key.is_empty() {
            return Err(format!(
                "Invalid file:key pair: {} (file and key cannot both be empty)",
                input
            ));
        }

        Ok((
            if file.is_empty() {
                None
            } else {
                Some(file.to_string())
            },
            if key.is_empty() {
                None
            } else {
                Some(key.to_string())
            },
        ))
    } else {
        // No colon: treat entire input as key
        if input.is_empty() {
            return Err("Key cannot be empty".to_string());
        }
        Ok((None, Some(input.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Testing the clap-based parser directly is less straightforward
    // than the old parse_args function since clap expects actual CLI arguments.
    // We'll test the parsing logic via the parse_cli function with simulated arguments.

    fn test_with_args(args: Vec<&str>) -> Result<Command, String> {
        let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
        match cli.command {
            Commands::Grep {
                pattern,
                recursive,
                files,
            } => Ok(Command::Grep {
                pattern,
                recursive,
                files,
            }),
            Commands::Set { file, updates } => {
                if updates.is_empty() {
                    return Err("set requires at least one key=value pair".to_string());
                }

                let mut parsed_updates = HashMap::new();

                for update in updates {
                    let parts: Vec<&str> = update.splitn(2, '=').collect();
                    if parts.len() != 2 {
                        return Err(format!("Invalid key=value pair: {}", update));
                    }
                    parsed_updates.insert(parts[0].to_string(), parts[1].to_string());
                }

                Ok(Command::Set {
                    file,
                    updates: parsed_updates,
                })
            }
            Commands::Unset { file, keys } => {
                if keys.is_empty() {
                    return Err("unset requires at least one key".to_string());
                }

                Ok(Command::Unset { file, keys })
            }
            Commands::Cp {
                source,
                destination,
            } => {
                let (source_file, source_key) = parse_file_key_pair(&source)?;
                let (dest_file, dest_key) = if destination.is_empty() {
                    (None, None)
                } else if destination.len() == 1 {
                    parse_optional_file_key_pair(&destination[0])?
                } else {
                    return Err("cp accepts at most one destination argument".to_string());
                };

                if dest_file.is_none() && dest_key.is_none() {
                    return Err(
                        "destination file and destination key cannot both be omitted".to_string(),
                    );
                }

                Ok(Command::Cp {
                    source_file,
                    source_key,
                    dest_file,
                    dest_key,
                })
            }
            Commands::Mv {
                source,
                destination,
            } => {
                let (source_file, source_key) = parse_file_key_pair(&source)?;
                let (dest_file, dest_key) = if destination.is_empty() {
                    (None, None)
                } else if destination.len() == 1 {
                    parse_optional_file_key_pair(&destination[0])?
                } else {
                    return Err("mv accepts at most one destination argument".to_string());
                };

                if dest_file.is_none() && dest_key.is_none() {
                    return Err(
                        "destination file and destination key cannot both be omitted".to_string(),
                    );
                }

                Ok(Command::Mv {
                    source_file,
                    source_key,
                    dest_file,
                    dest_key,
                })
            }
        }
    }

    // ==================== parse_args() Tests ====================

    #[test]
    fn test_parse_grep_simple() {
        let cmd = test_with_args(vec!["ym", "grep", "pattern", "file.yaml"]).unwrap();

        match cmd {
            Command::Grep {
                pattern,
                recursive,
                files,
            } => {
                assert_eq!(pattern, "pattern");
                assert!(!recursive);
                assert_eq!(files, vec!["file.yaml"]);
            }
            _ => panic!("Expected Grep command"),
        }
    }

    #[test]
    fn test_parse_grep_with_recursive_flag() {
        let cmd = test_with_args(vec!["ym", "grep", "-R", "pattern", "dir"]).unwrap();

        match cmd {
            Command::Grep {
                pattern,
                recursive,
                files,
            } => {
                assert_eq!(pattern, "pattern");
                assert!(recursive);
                assert_eq!(files, vec!["dir"]);
            }
            _ => panic!("Expected Grep command"),
        }
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

        match cmd {
            Command::Grep {
                pattern,
                recursive,
                files,
            } => {
                assert_eq!(pattern, "pattern");
                assert!(!recursive);
                assert_eq!(files, vec!["file1.yaml", "file2.yaml", "file3.yaml"]);
            }
            _ => panic!("Expected Grep command"),
        }
    }

    #[test]
    fn test_parse_grep_no_pattern_error() {
        let result = test_with_args(vec!["ym", "grep"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_grep_recursive_no_pattern_error() {
        let result = test_with_args(vec!["ym", "grep", "-R"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_grep_no_files_allowed() {
        // grep with pattern but no files should be valid (reads from stdin)
        let cmd = test_with_args(vec!["ym", "grep", "pattern"]).unwrap();

        match cmd {
            Command::Grep {
                pattern,
                recursive,
                files,
            } => {
                assert_eq!(pattern, "pattern");
                assert!(!recursive);
                assert_eq!(files, Vec::<String>::new());
            }
            _ => panic!("Expected Grep command"),
        }
    }

    #[test]
    fn test_parse_set_single_key_value() {
        let cmd = test_with_args(vec!["ym", "set", "file.yaml", "key=value"]).unwrap();

        match cmd {
            Command::Set { file, updates } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(updates.len(), 1);
                assert_eq!(updates.get("key"), Some(&"value".to_string()));
            }
            _ => panic!("Expected Set command"),
        }
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

        match cmd {
            Command::Set { file, updates } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(updates.len(), 3);
                assert_eq!(updates.get("key1"), Some(&"value1".to_string()));
                assert_eq!(updates.get("key2"), Some(&"value2".to_string()));
                assert_eq!(updates.get("key3"), Some(&"value3".to_string()));
            }
            _ => panic!("Expected Set command"),
        }
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

        match cmd {
            Command::Set { file, updates } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(updates.len(), 2);
                assert_eq!(updates.get("database.host"), Some(&"localhost".to_string()));
                assert_eq!(updates.get("database.port"), Some(&"5432".to_string()));
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_parse_set_value_with_equals() {
        // Values can contain '=' characters
        let cmd = test_with_args(vec![
            "ym",
            "set",
            "file.yaml",
            "url=http://example.com?param=value",
        ])
        .unwrap();

        match cmd {
            Command::Set { file, updates } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(updates.len(), 1);
                assert_eq!(
                    updates.get("url"),
                    Some(&"http://example.com?param=value".to_string())
                );
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_parse_set_no_file_error() {
        let result = test_with_args(vec!["ym", "set"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_set_no_key_value_error() {
        let result = test_with_args(vec!["ym", "set", "file.yaml"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_set_invalid_key_value_format() {
        let result = test_with_args(vec!["ym", "set", "file.yaml", "invalid_no_equals"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid key=value pair"));
    }

    #[test]
    fn test_parse_unset_single_key() {
        let cmd = test_with_args(vec!["ym", "unset", "file.yaml", "key"]).unwrap();

        match cmd {
            Command::Unset { file, keys } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(keys, vec!["key"]);
            }
            _ => panic!("Expected Unset command"),
        }
    }

    #[test]
    fn test_parse_unset_multiple_keys() {
        let cmd = test_with_args(vec!["ym", "unset", "file.yaml", "key1", "key2", "key3"]).unwrap();

        match cmd {
            Command::Unset { file, keys } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(keys, vec!["key1", "key2", "key3"]);
            }
            _ => panic!("Expected Unset command"),
        }
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

        match cmd {
            Command::Unset { file, keys } => {
                assert_eq!(file, "file.yaml");
                assert_eq!(keys, vec!["database.password", "database.username"]);
            }
            _ => panic!("Expected Unset command"),
        }
    }

    #[test]
    fn test_parse_unset_no_file_error() {
        let result = test_with_args(vec!["ym", "unset"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unset_no_keys_error() {
        let result = test_with_args(vec!["ym", "unset", "file.yaml"]);
        assert!(result.is_err());
    }

    // ==================== cp Tests ====================

    #[test]
    fn test_parse_cp_same_file_same_key() {
        // Error: both destination file and key omitted
        let result = test_with_args(vec!["ym", "cp", "file.yaml:source.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("destination file and destination key cannot both be omitted"));
    }

    #[test]
    fn test_parse_cp_same_file_different_key() {
        let cmd = test_with_args(vec!["ym", "cp", "file.yaml:source.key", "dest.key"]).unwrap();

        match cmd {
            Command::Cp {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "file.yaml");
                assert_eq!(source_key, "source.key");
                assert_eq!(dest_file, None);
                assert_eq!(dest_key, Some("dest.key".to_string()));
            }
            _ => panic!("Expected Cp command"),
        }
    }

    #[test]
    fn test_parse_cp_different_file_same_key() {
        let cmd = test_with_args(vec!["ym", "cp", "source.yaml:mykey", "dest.yaml:mykey"]).unwrap();

        match cmd {
            Command::Cp {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "source.yaml");
                assert_eq!(source_key, "mykey");
                assert_eq!(dest_file, Some("dest.yaml".to_string()));
                assert_eq!(dest_key, Some("mykey".to_string()));
            }
            _ => panic!("Expected Cp command"),
        }
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

        match cmd {
            Command::Cp {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "source.yaml");
                assert_eq!(source_key, "source.key");
                assert_eq!(dest_file, Some("dest.yaml".to_string()));
                assert_eq!(dest_key, Some("dest.key".to_string()));
            }
            _ => panic!("Expected Cp command"),
        }
    }

    #[test]
    fn test_parse_cp_only_destination_file() {
        let cmd = test_with_args(vec!["ym", "cp", "source.yaml:mykey", "dest.yaml:"]).unwrap();

        match cmd {
            Command::Cp {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "source.yaml");
                assert_eq!(source_key, "mykey");
                assert_eq!(dest_file, Some("dest.yaml".to_string()));
                assert_eq!(dest_key, None);
            }
            _ => panic!("Expected Cp command"),
        }
    }

    #[test]
    fn test_parse_cp_missing_source_key() {
        let result = test_with_args(vec!["ym", "cp", "source.yaml", "dest.key"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid file:key pair"));
    }

    #[test]
    fn test_parse_cp_invalid_source_format() {
        let result = test_with_args(vec!["ym", "cp", "invalid", "dest.key"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid file:key pair"));
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
            .contains("cp accepts at most one destination argument"));
    }

    // ==================== mv Tests ====================

    #[test]
    fn test_parse_mv_same_file_same_key() {
        // Error: both destination file and key omitted
        let result = test_with_args(vec!["ym", "mv", "file.yaml:source.key"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("destination file and destination key cannot both be omitted"));
    }

    #[test]
    fn test_parse_mv_same_file_different_key() {
        let cmd = test_with_args(vec!["ym", "mv", "file.yaml:source.key", "dest.key"]).unwrap();

        match cmd {
            Command::Mv {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "file.yaml");
                assert_eq!(source_key, "source.key");
                assert_eq!(dest_file, None);
                assert_eq!(dest_key, Some("dest.key".to_string()));
            }
            _ => panic!("Expected Mv command"),
        }
    }

    #[test]
    fn test_parse_mv_different_file_same_key() {
        let cmd = test_with_args(vec!["ym", "mv", "source.yaml:mykey", "dest.yaml:mykey"]).unwrap();

        match cmd {
            Command::Mv {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "source.yaml");
                assert_eq!(source_key, "mykey");
                assert_eq!(dest_file, Some("dest.yaml".to_string()));
                assert_eq!(dest_key, Some("mykey".to_string()));
            }
            _ => panic!("Expected Mv command"),
        }
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

        match cmd {
            Command::Mv {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "source.yaml");
                assert_eq!(source_key, "source.key");
                assert_eq!(dest_file, Some("dest.yaml".to_string()));
                assert_eq!(dest_key, Some("dest.key".to_string()));
            }
            _ => panic!("Expected Mv command"),
        }
    }

    #[test]
    fn test_parse_mv_only_destination_file() {
        let cmd = test_with_args(vec!["ym", "mv", "source.yaml:mykey", "dest.yaml:"]).unwrap();

        match cmd {
            Command::Mv {
                source_file,
                source_key,
                dest_file,
                dest_key,
            } => {
                assert_eq!(source_file, "source.yaml");
                assert_eq!(source_key, "mykey");
                assert_eq!(dest_file, Some("dest.yaml".to_string()));
                assert_eq!(dest_key, None);
            }
            _ => panic!("Expected Mv command"),
        }
    }

    #[test]
    fn test_parse_mv_missing_source_key() {
        let result = test_with_args(vec!["ym", "mv", "source.yaml", "dest.key"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid file:key pair"));
    }

    #[test]
    fn test_parse_mv_invalid_source_format() {
        let result = test_with_args(vec!["ym", "mv", "invalid", "dest.key"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid file:key pair"));
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
            .contains("mv accepts at most one destination argument"));
    }
}
