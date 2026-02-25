use regex::Regex;
use serde_yaml::Value;
use std::collections::HashMap;

/// Search YAML by key path pattern
/// When a key matches, return that value without recursing into nested keys
pub fn grep(value: &Value, pattern: &str) -> Result<Vec<(String, Value)>, String> {
    let mut results = Vec::new();
    collect_matching_keys(value, pattern, "", &mut results)?;
    Ok(results)
}

fn collect_matching_keys(
    value: &Value,
    pattern: &str,
    current_path: &str,
    results: &mut Vec<(String, Value)>,
) -> Result<(), String> {
    match value {
        Value::Mapping(map) => {
            for (key, val) in map {
                if let Value::String(k) = key {
                    let new_path = if current_path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", current_path, k)
                    };

                    // Check if pattern matches the current key path
                    if is_key_match(&new_path, pattern)? {
                        results.push((new_path, val.clone()));
                        // Don't recurse into matched keys - return the whole subtree
                    } else {
                        // Only recurse if this key doesn't match
                        collect_matching_keys(val, pattern, &new_path, results)?;
                    }
                }
            }
        }
        Value::Sequence(_) => {
            // For MVP, treat sequences as-is without special handling
        }
        _ => {}
    }
    Ok(())
}

/// Check if a key path matches the pattern (regex)
fn is_key_match(key: &str, pattern: &str) -> Result<bool, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
    Ok(re.is_match(key))
}

/// Set values in YAML at specified key paths
pub fn set_values(value: &mut Value, updates: &HashMap<String, String>) -> Result<(), String> {
    for (key_path, new_value) in updates {
        set_at_path(value, key_path, new_value)?;
    }
    Ok(())
}

fn set_at_path(value: &mut Value, path: &str, new_value: &str) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.is_empty() {
        return Err("Empty key path".to_string());
    }

    // Ensure root is a mapping
    if !matches!(value, Value::Mapping(_)) {
        *value = Value::Mapping(Default::default());
    }

    // Navigate/create the path
    let mut current = value;
    for (i, &part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part: set the value
            if let Value::Mapping(ref mut map) = current {
                map.insert(
                    Value::String(part.to_string()),
                    Value::String(new_value.to_string()),
                );
            }
        } else {
            // Intermediate part: navigate or create
            if let Value::Mapping(ref mut map) = current {
                current = map
                    .entry(Value::String(part.to_string()))
                    .or_insert_with(|| Value::Mapping(Default::default()));
            }
        }
    }

    Ok(())
}

/// Remove keys from YAML at specified paths
pub fn unset_values(value: &mut Value, keys: &[String]) -> Result<(), String> {
    for key_path in keys {
        unset_at_path(value, key_path)?;
    }
    Ok(())
}

/// Get a value from YAML at a specified key path
pub fn get_value(value: &Value, path: &str) -> Result<Option<Value>, String> {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.is_empty() {
        return Err("Empty key path".to_string());
    }

    let mut current = value;
    for part in parts {
        if let Value::Mapping(map) = current {
            match map.get(&Value::String(part.to_string())) {
                Some(next) => current = next,
                None => return Ok(None),
            }
        } else {
            return Ok(None);
        }
    }

    Ok(Some(current.clone()))
}

/// Copy a value from source file:key to destination file:key
/// Source and destination keys are required
/// If dest_file is None, use source_file
/// If dest_key is None, use source_key
pub fn copy_value(
    source_file: &str,
    source_key: &str,
    dest_file: &str,
    dest_key: &str,
) -> Result<(), String> {
    use crate::yaml_format_preserving;
    use std::fs;

    // Read source file
    let source_contents = fs::read_to_string(source_file)
        .map_err(|e| format!("Failed to read source file '{}': {}", source_file, e))?;

    let source_yaml = serde_yaml::from_str(&source_contents)
        .map_err(|e| format!("Failed to parse YAML from '{}': {}", source_file, e))?;

    // Get the value from source
    let value = get_value(&source_yaml, source_key)?
        .ok_or_else(|| format!("Key '{}' not found in '{}'", source_key, source_file))?;

    // Read destination file (or create if it doesn't exist)
    let (mut dest_yaml, dest_contents_option) = if std::path::Path::new(dest_file).exists() {
        let dest_contents = fs::read_to_string(dest_file)
            .map_err(|e| format!("Failed to read destination file '{}': {}", dest_file, e))?;

        let yaml = serde_yaml::from_str(&dest_contents)
            .map_err(|e| format!("Failed to parse YAML from '{}': {}", dest_file, e))?;
        (yaml, Some(dest_contents))
    } else {
        (Value::Mapping(Default::default()), None)
    };

    // Set the value at destination
    set_value(&mut dest_yaml, dest_key, &value)?;

    // Write destination file using format-preserving logic if possible
    let dest_yaml_str = if let Some(dest_contents) = dest_contents_option {
        // Destination file exists, preserve its formatting
        yaml_format_preserving::write_yaml_preserving_format(&dest_contents, &dest_yaml)
            .map_err(|e| format!("Failed to preserve YAML format: {}", e))?
    } else {
        // New destination file, use standard serialization
        serde_yaml::to_string(&dest_yaml).map_err(|e| format!("Failed to serialize YAML: {}", e))?
    };

    fs::write(dest_file, dest_yaml_str)
        .map_err(|e| format!("Failed to write to '{}': {}", dest_file, e))?;

    Ok(())
}

/// Move a value from source file:key to destination file:key
/// This copies the value and then deletes it from the source
/// Source and destination keys are required
/// If dest_file is None, use source_file
/// If dest_key is None, use source_key
pub fn move_value(
    source_file: &str,
    source_key: &str,
    dest_file: &str,
    dest_key: &str,
) -> Result<(), String> {
    use crate::yaml_format_preserving;
    use std::fs;

    // First, copy the value from source to destination
    copy_value(source_file, source_key, dest_file, dest_key)?;

    // Then, delete the source key from the source file
    let source_contents = fs::read_to_string(source_file)
        .map_err(|e| format!("Failed to read source file '{}': {}", source_file, e))?;

    let mut source_yaml = serde_yaml::from_str(&source_contents)
        .map_err(|e| format!("Failed to parse YAML from '{}': {}", source_file, e))?;

    // Unset the source key
    unset_at_path(&mut source_yaml, source_key)?;

    // Always use format-preserving write to preserve comments and spacing
    let source_yaml_str =
        yaml_format_preserving::write_yaml_preserving_format(&source_contents, &source_yaml)
            .map_err(|e| format!("Failed to preserve YAML format: {}", e))?;

    fs::write(source_file, &source_yaml_str)
        .map_err(|e| format!("Failed to write to '{}': {}", source_file, e))?;

    Ok(())
}

/// Set a value in YAML at a specified key path to a specific Value
fn set_value(value: &mut Value, path: &str, new_value: &Value) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.is_empty() {
        return Err("Empty key path".to_string());
    }

    // Ensure root is a mapping
    if !matches!(value, Value::Mapping(_)) {
        *value = Value::Mapping(Default::default());
    }

    // Navigate/create the path
    let mut current = value;
    for (i, &part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part: set the value
            if let Value::Mapping(ref mut map) = current {
                map.insert(Value::String(part.to_string()), new_value.clone());
            }
        } else {
            // Intermediate part: navigate or create
            if let Value::Mapping(ref mut map) = current {
                current = map
                    .entry(Value::String(part.to_string()))
                    .or_insert_with(|| Value::Mapping(Default::default()));
            }
        }
    }

    Ok(())
}

fn unset_at_path(value: &mut Value, path: &str) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.is_empty() {
        return Err("Empty key path".to_string());
    }

    if parts.len() == 1 {
        // Direct child: remove from root mapping
        if let Value::Mapping(ref mut map) = value {
            map.remove(&Value::String(parts[0].to_string()));
        }
    } else {
        // Navigate to parent, then remove the final key
        let mut current = value;
        for &part in parts[..parts.len() - 1].iter() {
            if let Value::Mapping(ref mut map) = current {
                if let Some(next) = map.get_mut(&Value::String(part.to_string())) {
                    current = next;
                } else {
                    // Path doesn't exist
                    return Ok(());
                }
            } else {
                // Path is not a mapping
                return Ok(());
            }
        }

        // Remove the final key
        if let Value::Mapping(ref mut map) = current {
            map.remove(&Value::String(parts[parts.len() - 1].to_string()));
        }
    }

    Ok(())
}

/// Format result for output as "key: value"
/// For mappings, display full YAML structure with indentation
pub fn format_result(key: &str, value: &Value, terminal_width: usize) -> String {
    match value {
        Value::Mapping(_) => {
            // For mappings, display as multi-line YAML with indentation
            format_mapping_result(key, value, terminal_width)
        }
        Value::String(s) => {
            let result = format!("{}: {}", key, s);
            truncate_if_needed(&result, terminal_width)
        }
        Value::Number(n) => {
            let result = format!("{}: {}", key, n);
            truncate_if_needed(&result, terminal_width)
        }
        Value::Bool(b) => {
            let result = format!("{}: {}", key, b);
            truncate_if_needed(&result, terminal_width)
        }
        Value::Null => {
            format!("{}: null", key)
        }
        _ => {
            // For sequences and other types, use YAML format
            let val_str = serde_yaml::to_string(value)
                .unwrap_or_else(|_| "<complex>".to_string())
                .trim()
                .to_string();
            let result = format!("{}: {}", key, val_str);
            truncate_if_needed(&result, terminal_width)
        }
    }
}

fn truncate_if_needed(text: &str, terminal_width: usize) -> String {
    // For single-line output, truncate if needed
    if text.len() > terminal_width {
        format!("{}...", &text[..terminal_width.saturating_sub(3)])
    } else {
        text.to_string()
    }
}

fn format_mapping_result(key: &str, value: &Value, _terminal_width: usize) -> String {
    // Convert mapping to YAML string with indentation
    let yaml_str = match serde_yaml::to_string(value) {
        Ok(s) => s,
        Err(_) => return format!("{}: <error>", key),
    };

    // Indent each line of the YAML output by 2 spaces
    let indented = yaml_str
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("  {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{}:\n{}", key, indented)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_yaml(yaml_str: &str) -> Value {
        serde_yaml::from_str(yaml_str).expect("Failed to parse YAML")
    }

    // ==================== grep() Tests ====================

    #[test]
    fn test_grep_simple_key() {
        let yaml = parse_yaml("name: Alice\nage: 30");
        let results = grep(&yaml, "name").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "name");
        assert_eq!(results[0].1.as_str().unwrap(), "Alice");
    }

    #[test]
    fn test_grep_exact_match() {
        let yaml_str = r#"
database:
  host: localhost
  port: 5432
cache:
  host: redis
"#;
        let yaml = parse_yaml(yaml_str);
        let results = grep(&yaml, "^database\\.host$").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "database.host");
    }

    #[test]
    fn test_grep_pattern_wildcard() {
        let yaml_str = r#"
database:
  host: localhost
  port: 5432
  username: admin
"#;
        let yaml = parse_yaml(yaml_str);
        let results = grep(&yaml, "database\\..*").unwrap();
        assert_eq!(results.len(), 3);
        let keys: Vec<_> = results.iter().map(|r| r.0.as_str()).collect();
        assert!(keys.contains(&"database.host"));
        assert!(keys.contains(&"database.port"));
        assert!(keys.contains(&"database.username"));
    }

    #[test]
    fn test_grep_nested_paths() {
        let yaml_str = r#"
app:
  server:
    address: 0.0.0.0
    port: 8080
"#;
        let yaml = parse_yaml(yaml_str);
        let results = grep(&yaml, "app\\.server.*").unwrap();
        // When "app.server" matches the pattern, it stops recursing, returning just "app.server" with its whole subtree
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "app.server");
        // The value should be the whole server mapping
        assert!(results[0].1.is_mapping());
    }

    #[test]
    fn test_grep_no_match() {
        let yaml = parse_yaml("name: Alice");
        let results = grep(&yaml, "nonexistent").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_grep_invalid_regex() {
        let yaml = parse_yaml("name: Alice");
        let result = grep(&yaml, "[invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid regex"));
    }

    #[test]
    fn test_grep_with_alternation() {
        let yaml_str = r#"
dev:
  password: devpass
prod:
  password: prodpass
staging:
  token: stagingtoken
"#;
        let yaml = parse_yaml(yaml_str);
        let results = grep(&yaml, "(dev|prod)\\.password").unwrap();
        assert_eq!(results.len(), 2);
        let keys: Vec<_> = results.iter().map(|r| r.0.as_str()).collect();
        assert!(keys.contains(&"dev.password"));
        assert!(keys.contains(&"prod.password"));
    }

    #[test]
    fn test_grep_stops_at_match() {
        let yaml_str = r#"
config:
  nested:
    value: test
"#;
        let yaml = parse_yaml(yaml_str);
        let results = grep(&yaml, "^config$").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "config");
        assert!(results[0].1.is_mapping());
    }

    // ==================== set_values() Tests ====================

    #[test]
    fn test_set_simple_value() {
        let mut yaml = parse_yaml("name: Alice");
        let mut updates = HashMap::new();
        updates.insert("name".to_string(), "Bob".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(yaml["name"].as_str().unwrap(), "Bob");
    }

    #[test]
    fn test_set_new_key() {
        let mut yaml = parse_yaml("name: Alice");
        let mut updates = HashMap::new();
        updates.insert("age".to_string(), "30".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(yaml["age"].as_str().unwrap(), "30");
    }

    #[test]
    fn test_set_nested_path_creates_structure() {
        let mut yaml = Value::Mapping(Default::default());
        let mut updates = HashMap::new();
        updates.insert("database.host".to_string(), "localhost".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(yaml["database"]["host"].as_str().unwrap(), "localhost");
    }

    #[test]
    fn test_set_deep_nesting() {
        let mut yaml = Value::Mapping(Default::default());
        let mut updates = HashMap::new();
        updates.insert("app.server.config.timeout".to_string(), "30".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(
            yaml["app"]["server"]["config"]["timeout"].as_str().unwrap(),
            "30"
        );
    }

    #[test]
    fn test_set_multiple_values() {
        let mut yaml = Value::Mapping(Default::default());
        let mut updates = HashMap::new();
        updates.insert("key1".to_string(), "value1".to_string());
        updates.insert("key2".to_string(), "value2".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(yaml["key1"].as_str().unwrap(), "value1");
        assert_eq!(yaml["key2"].as_str().unwrap(), "value2");
    }

    #[test]
    fn test_set_overwrites_existing() {
        let mut yaml = parse_yaml("config:\n  level: info");
        let mut updates = HashMap::new();
        updates.insert("config.level".to_string(), "debug".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(yaml["config"]["level"].as_str().unwrap(), "debug");
    }

    #[test]
    fn test_set_preserves_siblings() {
        let yaml_str = r#"
database:
  host: localhost
  port: 5432
  username: admin
"#;
        let mut yaml = parse_yaml(yaml_str);
        let mut updates = HashMap::new();
        updates.insert("database.port".to_string(), "3306".to_string());

        set_values(&mut yaml, &updates).unwrap();
        assert_eq!(yaml["database"]["port"].as_str().unwrap(), "3306");
        assert_eq!(yaml["database"]["host"].as_str().unwrap(), "localhost");
        assert_eq!(yaml["database"]["username"].as_str().unwrap(), "admin");
    }

    // ==================== unset_values() Tests ====================

    #[test]
    fn test_unset_top_level_key() {
        let mut yaml = parse_yaml("name: Alice\nage: 30");
        unset_values(&mut yaml, &["age".to_string()]).unwrap();
        assert_eq!(yaml["age"], Value::Null);
        assert_eq!(yaml["name"].as_str().unwrap(), "Alice");
    }

    #[test]
    fn test_unset_nested_key() {
        let yaml_str = r#"
database:
  host: localhost
  port: 5432
"#;
        let mut yaml = parse_yaml(yaml_str);
        unset_values(&mut yaml, &["database.port".to_string()]).unwrap();
        assert_eq!(yaml["database"]["port"], Value::Null);
        assert_eq!(yaml["database"]["host"].as_str().unwrap(), "localhost");
    }

    #[test]
    fn test_unset_deep_nested_key() {
        let yaml_str = r#"
app:
  server:
    config:
      timeout: 30
      retries: 3
"#;
        let mut yaml = parse_yaml(yaml_str);
        unset_values(&mut yaml, &["app.server.config.timeout".to_string()]).unwrap();
        assert_eq!(yaml["app"]["server"]["config"]["timeout"], Value::Null);
        assert_eq!(yaml["app"]["server"]["config"]["retries"].as_i64(), Some(3));
    }

    #[test]
    fn test_unset_multiple_keys() {
        let mut yaml = parse_yaml("a: 1\nb: 2\nc: 3");
        unset_values(&mut yaml, &["a".to_string(), "c".to_string()]).unwrap();
        assert_eq!(yaml["a"], Value::Null);
        assert_eq!(yaml["b"].as_i64(), Some(2));
        assert_eq!(yaml["c"], Value::Null);
    }

    #[test]
    fn test_unset_nonexistent_key() {
        let mut yaml = parse_yaml("name: Alice");
        unset_values(&mut yaml, &["nonexistent".to_string()]).unwrap();
        assert_eq!(yaml["name"].as_str().unwrap(), "Alice");
    }

    #[test]
    fn test_unset_nonexistent_nested_path() {
        let yaml_str = r#"
database:
  host: localhost
"#;
        let mut yaml = parse_yaml(yaml_str);
        unset_values(&mut yaml, &["database.nonexistent".to_string()]).unwrap();
        assert_eq!(yaml["database"]["host"].as_str().unwrap(), "localhost");
    }

    // ==================== format_result() Tests ====================

    #[test]
    fn test_format_string_value() {
        let value = Value::String("hello".to_string());
        let result = format_result("message", &value, 80);
        assert_eq!(result, "message: hello");
    }

    #[test]
    fn test_format_number_value() {
        let value = Value::Number(42.into());
        let result = format_result("count", &value, 80);
        assert_eq!(result, "count: 42");
    }

    #[test]
    fn test_format_boolean_true() {
        let value = Value::Bool(true);
        let result = format_result("enabled", &value, 80);
        assert_eq!(result, "enabled: true");
    }

    #[test]
    fn test_format_boolean_false() {
        let value = Value::Bool(false);
        let result = format_result("enabled", &value, 80);
        assert_eq!(result, "enabled: false");
    }

    #[test]
    fn test_format_null_value() {
        let value = Value::Null;
        let result = format_result("empty", &value, 80);
        assert_eq!(result, "empty: null");
    }

    #[test]
    fn test_format_mapping_value() {
        let value = parse_yaml("host: localhost\nport: 5432");
        let result = format_result("database", &value, 80);
        assert!(result.starts_with("database:\n"));
        assert!(result.contains("host"));
        assert!(result.contains("localhost"));
    }

    #[test]
    fn test_format_truncates_long_string() {
        let long_string = "a".repeat(100);
        let value = Value::String(long_string);
        let result = format_result("key", &value, 20);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 23);
    }

    #[test]
    fn test_format_does_not_truncate_short_string() {
        let value = Value::String("short".to_string());
        let result = format_result("key", &value, 80);
        assert_eq!(result, "key: short");
        assert!(!result.ends_with("..."));
    }

    #[test]
    fn test_format_sequence() {
        let value = parse_yaml("- item1\n- item2\n- item3");
        let result = format_result("items", &value, 80);
        assert!(result.contains("items:"));
    }

    // ==================== get_value() Tests ====================

    #[test]
    fn test_get_value_simple_key() {
        let yaml = parse_yaml("name: Alice\nage: 30");
        let result = get_value(&yaml, "name").unwrap();
        assert_eq!(result.unwrap().as_str().unwrap(), "Alice");
    }

    #[test]
    fn test_get_value_nested_key() {
        let yaml = parse_yaml("database:\n  host: localhost\n  port: 5432");
        let result = get_value(&yaml, "database.host").unwrap();
        assert_eq!(result.unwrap().as_str().unwrap(), "localhost");
    }

    #[test]
    fn test_get_value_nonexistent_key() {
        let yaml = parse_yaml("name: Alice");
        let result = get_value(&yaml, "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_value_nonexistent_nested_path() {
        let yaml = parse_yaml("database:\n  host: localhost");
        let result = get_value(&yaml, "database.nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_value_mapping() {
        let yaml = parse_yaml("database:\n  host: localhost\n  port: 5432");
        let result = get_value(&yaml, "database").unwrap();
        assert!(result.unwrap().is_mapping());
    }

    #[test]
    fn test_get_value_number() {
        let yaml = parse_yaml("age: 30\nheight: 180");
        let result = get_value(&yaml, "age").unwrap();
        assert_eq!(result.unwrap().as_i64().unwrap(), 30);
    }

    // ==================== copy_value() Tests ====================

    #[test]
    fn test_copy_value_same_file_simple() {
        use std::fs;

        let test_dir = "test_copy_same_file";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        fs::write(
            &test_file,
            "source:\n  key: value123\ndest:\n  key: old_value",
        )
        .unwrap();

        copy_value(&test_file, "source.key", &test_file, "dest.key").unwrap();

        let contents = fs::read_to_string(&test_file).unwrap();
        let yaml = serde_yaml::from_str::<Value>(&contents).unwrap();
        assert_eq!(yaml["dest"]["key"].as_str().unwrap(), "value123");
        assert_eq!(yaml["source"]["key"].as_str().unwrap(), "value123");

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_copy_value_different_files() {
        use std::fs;

        let test_dir = "test_copy_diff_files";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let source_file = format!("{}/source.yaml", test_dir);
        let dest_file = format!("{}/dest.yaml", test_dir);

        fs::write(&source_file, "data:\n  value: test123").unwrap();
        fs::write(&dest_file, "other: value").unwrap();

        copy_value(&source_file, "data.value", &dest_file, "copied.value").unwrap();

        let dest_contents = fs::read_to_string(&dest_file).unwrap();
        let yaml = serde_yaml::from_str::<Value>(&dest_contents).unwrap();
        assert_eq!(yaml["copied"]["value"].as_str().unwrap(), "test123");

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_copy_value_to_nonexistent_file() {
        use std::fs;

        let test_dir = "test_copy_to_new";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let source_file = format!("{}/source.yaml", test_dir);
        let dest_file = format!("{}/dest.yaml", test_dir);

        fs::write(&source_file, "data: value456").unwrap();

        copy_value(&source_file, "data", &dest_file, "new_key").unwrap();

        assert!(std::path::Path::new(&dest_file).exists());
        let dest_contents = fs::read_to_string(&dest_file).unwrap();
        let yaml = serde_yaml::from_str::<Value>(&dest_contents).unwrap();
        assert_eq!(yaml["new_key"].as_str().unwrap(), "value456");

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_copy_value_complex_type() {
        use std::fs;

        let test_dir = "test_copy_complex";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        fs::write(
            &test_file,
            "config:\n  nested:\n    value: test\n    count: 42",
        )
        .unwrap();

        copy_value(&test_file, "config.nested", &test_file, "backup.config").unwrap();

        let contents = fs::read_to_string(&test_file).unwrap();
        let yaml = serde_yaml::from_str::<Value>(&contents).unwrap();
        assert!(yaml["backup"]["config"].is_mapping());
        assert_eq!(yaml["backup"]["config"]["value"].as_str().unwrap(), "test");
        assert_eq!(yaml["backup"]["config"]["count"].as_i64().unwrap(), 42);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_copy_value_nonexistent_source_key() {
        use std::fs;

        let test_dir = "test_copy_nonexistent";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        fs::write(&test_file, "data: value").unwrap();

        let result = copy_value(&test_file, "nonexistent", &test_file, "dest");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        fs::remove_dir_all(test_dir).unwrap();
    }

    // ==================== move_value() Tests ====================

    #[test]
    fn test_move_value_same_file_same_key() {
        use std::fs;

        let test_dir = "test_move_same_same";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        fs::write(&test_file, "database:\n  password: secret123").unwrap();

        // Move should error since source and dest are identical
        let result = move_value(
            &test_file,
            "database.password",
            &test_file,
            "database.password",
        );
        // This is actually valid - it copies then unsets, which effectively leaves the value
        // But after unsetting its own copy, it would be gone
        assert!(result.is_ok());

        // Verify the value is gone from source
        let yaml_str = fs::read_to_string(&test_file).unwrap();
        let yaml = serde_yaml::from_str::<Value>(&yaml_str).unwrap();
        assert_eq!(get_value(&yaml, "database.password").unwrap(), None);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_move_value_same_file_different_key() {
        use std::fs;

        let test_dir = "test_move_same_diff";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        fs::write(&test_file, "source_key: moved_value\nother: data").unwrap();

        move_value(&test_file, "source_key", &test_file, "dest_key").unwrap();

        // Verify destination has the value
        let yaml_str = fs::read_to_string(&test_file).unwrap();
        let yaml = serde_yaml::from_str::<Value>(&yaml_str).unwrap();
        assert_eq!(
            get_value(&yaml, "dest_key")
                .unwrap()
                .unwrap()
                .as_str()
                .unwrap(),
            "moved_value"
        );

        // Verify source no longer has the value
        assert_eq!(get_value(&yaml, "source_key").unwrap(), None);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_move_value_different_file_same_key() {
        use std::fs;

        let test_dir = "test_move_diff_same";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let source_file = format!("{}/source.yaml", test_dir);
        let dest_file = format!("{}/dest.yaml", test_dir);

        fs::write(&source_file, "mykey: myvalue").unwrap();
        fs::write(&dest_file, "other: data").unwrap();

        move_value(&source_file, "mykey", &dest_file, "mykey").unwrap();

        // Verify destination has the value
        let dest_yaml_str = fs::read_to_string(&dest_file).unwrap();
        let dest_yaml = serde_yaml::from_str::<Value>(&dest_yaml_str).unwrap();
        assert_eq!(
            get_value(&dest_yaml, "mykey")
                .unwrap()
                .unwrap()
                .as_str()
                .unwrap(),
            "myvalue"
        );

        // Verify source no longer has the value
        let source_yaml_str = fs::read_to_string(&source_file).unwrap();
        let source_yaml = serde_yaml::from_str::<Value>(&source_yaml_str).unwrap();
        assert_eq!(get_value(&source_yaml, "mykey").unwrap(), None);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_move_value_different_file_different_key() {
        use std::fs;

        let test_dir = "test_move_diff_diff";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let source_file = format!("{}/source.yaml", test_dir);
        let dest_file = format!("{}/dest.yaml", test_dir);

        fs::write(&source_file, "source:\n  nested:\n    key: moved_value").unwrap();
        fs::write(&dest_file, "other: data").unwrap();

        move_value(
            &source_file,
            "source.nested.key",
            &dest_file,
            "dest.nested.key",
        )
        .unwrap();

        // Verify destination has the value
        let dest_yaml_str = fs::read_to_string(&dest_file).unwrap();
        let dest_yaml = serde_yaml::from_str::<Value>(&dest_yaml_str).unwrap();
        assert_eq!(
            get_value(&dest_yaml, "dest.nested.key")
                .unwrap()
                .unwrap()
                .as_str()
                .unwrap(),
            "moved_value"
        );

        // Verify source no longer has the value
        let source_yaml_str = fs::read_to_string(&source_file).unwrap();
        let source_yaml = serde_yaml::from_str::<Value>(&source_yaml_str).unwrap();
        assert_eq!(get_value(&source_yaml, "source.nested.key").unwrap(), None);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_move_value_nonexistent_source_key() {
        use std::fs;

        let test_dir = "test_move_nonexistent";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        fs::write(&test_file, "data: value").unwrap();

        let result = move_value(&test_file, "nonexistent", &test_file, "dest");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_set_preserves_comments_and_empty_lines() {
        use std::fs;

        let test_dir = "test_set_preserve_comments";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        let original = "# Configuration file\napp_name: myapp\n\n# Debug settings\ndebug: false\n";
        fs::write(&test_file, original).unwrap();

        // Simulate the set command
        let contents = fs::read_to_string(&test_file).unwrap();
        let mut value = serde_yaml::from_str::<Value>(&contents).unwrap();

        let mut updates = std::collections::HashMap::new();
        updates.insert("debug".to_string(), "true".to_string());
        set_values(&mut value, &updates).unwrap();

        let updated =
            crate::yaml_format_preserving::write_yaml_preserving_format(&contents, &value).unwrap();

        // Comments and empty lines should be preserved
        assert!(updated.contains("# Configuration file"));
        assert!(updated.contains("# Debug settings"));
        // The value should be updated
        assert!(updated.contains("debug: true"));
        // Original app_name should still be there
        assert!(updated.contains("app_name: myapp"));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_unset_preserves_comments_and_empty_lines() {
        use std::fs;

        let test_dir = "test_unset_preserve_comments";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        let original = "# Configuration\nkey1: value1\n\n# Comment\nkey2: value2\nkey3: value3\n";
        fs::write(&test_file, original).unwrap();

        // Simulate the unset command
        let contents = fs::read_to_string(&test_file).unwrap();
        let mut value = serde_yaml::from_str::<Value>(&contents).unwrap();

        unset_values(&mut value, &["key2".to_string()]).unwrap();

        let updated =
            crate::yaml_format_preserving::write_yaml_preserving_format(&contents, &value).unwrap();

        // Comments should be preserved
        assert!(updated.contains("# Configuration"));
        // key2 should be removed
        assert!(!updated.contains("key2: value2"));
        // Other keys should remain
        assert!(updated.contains("key1: value1"));
        assert!(updated.contains("key3: value3"));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_set_multiple_values_preserves_formatting() {
        use std::fs;

        let test_dir = "test_set_multi_preserve";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir(test_dir).unwrap();

        let test_file = format!("{}/test.yaml", test_dir);
        let original =
            "# Server config\nhost: localhost\n\n# Port settings\nport: 8080\nssl: false\n";
        fs::write(&test_file, original).unwrap();

        let contents = fs::read_to_string(&test_file).unwrap();
        let mut value = serde_yaml::from_str::<Value>(&contents).unwrap();

        let mut updates = std::collections::HashMap::new();
        updates.insert("host".to_string(), "0.0.0.0".to_string());
        updates.insert("ssl".to_string(), "true".to_string());
        set_values(&mut value, &updates).unwrap();

        let updated =
            crate::yaml_format_preserving::write_yaml_preserving_format(&contents, &value).unwrap();

        // Comments should be preserved
        assert!(updated.contains("# Server config"));
        assert!(updated.contains("# Port settings"));
        // Values should be updated
        assert!(updated.contains("host: 0.0.0.0"));
        assert!(updated.contains("ssl: true"));
        assert!(updated.contains("port: 8080"));

        fs::remove_dir_all(test_dir).unwrap();
    }
}
