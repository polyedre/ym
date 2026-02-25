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
}

pub fn parse_args(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Err("No command specified".to_string());
    }

    let cmd = &args[0];

    match cmd.as_str() {
        "grep" => {
            if args.len() < 2 {
                return Err("grep requires: ym grep [-R] <pattern> [FILE [FILE ...]]".to_string());
            }

            let mut recursive = false;
            let mut idx = 1;

            // Check for -R flag
            if args[1] == "-R" {
                recursive = true;
                idx = 2;
                if args.len() < 3 {
                    return Err(
                        "grep requires: ym grep [-R] <pattern> [FILE [FILE ...]]".to_string()
                    );
                }
            }

            let pattern = args[idx].clone();
            let files: Vec<String> = args[idx + 1..].iter().map(|s| s.clone()).collect();

            Ok(Command::Grep {
                pattern,
                recursive,
                files,
            })
        }
        "set" => {
            if args.len() < 3 {
                return Err("set requires: ym set <file> <key=value> [key=value]...".to_string());
            }
            let file = args[1].clone();
            let mut updates = HashMap::new();

            for arg in &args[2..] {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid key=value pair: {}", arg));
                }
                updates.insert(parts[0].to_string(), parts[1].to_string());
            }

            Ok(Command::Set { file, updates })
        }
        "unset" => {
            if args.len() < 3 {
                return Err("unset requires: ym unset <file> <key> [key]...".to_string());
            }
            let file = args[1].clone();
            let keys = args[2..].iter().map(|s| s.clone()).collect();

            Ok(Command::Unset { file, keys })
        }
        _ => Err(format!("Unknown command: {}", cmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_args() Tests ====================

    #[test]
    fn test_parse_grep_simple() {
        let args = vec![
            "grep".to_string(),
            "pattern".to_string(),
            "file.yaml".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "grep".to_string(),
            "-R".to_string(),
            "pattern".to_string(),
            "dir".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "grep".to_string(),
            "pattern".to_string(),
            "file1.yaml".to_string(),
            "file2.yaml".to_string(),
            "file3.yaml".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec!["grep".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("grep requires"));
    }

    #[test]
    fn test_parse_grep_recursive_no_pattern_error() {
        let args = vec!["grep".to_string(), "-R".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("grep requires"));
    }

    #[test]
    fn test_parse_grep_no_files_allowed() {
        // grep with pattern but no files should be valid (reads from stdin)
        let args = vec!["grep".to_string(), "pattern".to_string()];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "set".to_string(),
            "file.yaml".to_string(),
            "key=value".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "set".to_string(),
            "file.yaml".to_string(),
            "key1=value1".to_string(),
            "key2=value2".to_string(),
            "key3=value3".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "set".to_string(),
            "file.yaml".to_string(),
            "database.host=localhost".to_string(),
            "database.port=5432".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "set".to_string(),
            "file.yaml".to_string(),
            "url=http://example.com?param=value".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec!["set".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("set requires"));
    }

    #[test]
    fn test_parse_set_no_key_value_error() {
        let args = vec!["set".to_string(), "file.yaml".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("set requires"));
    }

    #[test]
    fn test_parse_set_invalid_key_value_format() {
        let args = vec![
            "set".to_string(),
            "file.yaml".to_string(),
            "invalid_no_equals".to_string(),
        ];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid key=value pair"));
    }

    #[test]
    fn test_parse_unset_single_key() {
        let args = vec![
            "unset".to_string(),
            "file.yaml".to_string(),
            "key".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "unset".to_string(),
            "file.yaml".to_string(),
            "key1".to_string(),
            "key2".to_string(),
            "key3".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec![
            "unset".to_string(),
            "file.yaml".to_string(),
            "database.password".to_string(),
            "database.username".to_string(),
        ];
        let cmd = parse_args(&args).unwrap();

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
        let args = vec!["unset".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unset requires"));
    }

    #[test]
    fn test_parse_unset_no_keys_error() {
        let args = vec!["unset".to_string(), "file.yaml".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unset requires"));
    }

    #[test]
    fn test_parse_unknown_command() {
        let args = vec!["unknown".to_string()];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }

    #[test]
    fn test_parse_empty_args() {
        let args = vec![];
        let result = parse_args(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No command specified"));
    }
}
