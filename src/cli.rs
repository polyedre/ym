use std::collections::HashMap;

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
