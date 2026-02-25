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
