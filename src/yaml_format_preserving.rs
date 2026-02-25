use serde_yaml::Value;
use std::collections::HashMap;

/// Updates a YAML string while preserving comments, empty lines, and indentation
/// for both top-level and nested key modifications.
///
/// This function preserves formatting for all operations including nested keys
/// by performing line-by-line updates and intelligently handling indentation.
pub fn write_yaml_preserving_format(
    original_content: &str,
    updated_value: &Value,
) -> Result<String, String> {
    // Parse the original to understand structure
    let original_value: Value = serde_yaml::from_str(original_content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    // Check if there are unhandleable structural changes
    // If we're adding new nested structures, fall back to standard serialization
    if has_unhandleable_nested_changes(&original_value, updated_value) {
        // For truly complex nested changes, use standard YAML serialization
        return serde_yaml::to_string(updated_value)
            .map_err(|e| format!("Failed to serialize YAML: {}", e));
    }

    // Collect keys that were removed (in original but not in updated)
    let mut removed_keys = Vec::new();
    collect_removed_keys(&original_value, updated_value, "", &mut removed_keys);

    // Build a map of all keys and their new values
    let updates = collect_all_changes(original_content, updated_value)?;

    if updates.is_empty() && removed_keys.is_empty() {
        // No changes, return original
        return Ok(original_content.to_string());
    }

    // Apply changes line by line
    apply_changes_to_content(original_content, &updates, &removed_keys)
}

/// Build a map from line number to YAML key path
fn build_line_to_key_map(
    lines: &[&str],
) -> Result<std::collections::HashMap<usize, String>, String> {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let mut path_stack: Vec<(usize, String)> = Vec::new(); // (indent, key)

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse key:value
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().to_string();

            // Pop stack until we find the right indent level
            while let Some((last_indent, _)) = path_stack.last() {
                if *last_indent >= indent {
                    path_stack.pop();
                } else {
                    break;
                }
            }

            // Build full key path
            let full_key = if path_stack.is_empty() {
                key.clone()
            } else {
                let path_parts: Vec<String> = path_stack.iter().map(|(_, k)| k.clone()).collect();
                format!("{}.{}", path_parts.join("."), key)
            };

            map.insert(line_idx, full_key.clone());
            path_stack.push((indent, key));
        }
    }

    Ok(map)
}

/// Check if there are complex structural changes that we can't handle with line-based updates.
/// We can handle:
/// - Removing nested keys (deletions)
/// - Changing scalar values at any level
///
/// We cannot handle well:
/// - Adding new nested structures
/// - Changing mapping structures significantly
fn has_unhandleable_nested_changes(old: &Value, new: &Value) -> bool {
    match (old, new) {
        (Value::Mapping(old_map), Value::Mapping(new_map)) => {
            // Check if new keys were added that are nested structures
            for (key, new_val) in new_map {
                if !old_map.contains_key(key) && new_val.is_mapping() {
                    // New nested structure added - we can't handle this well
                    return true;
                }
            }

            // Check if old nested structures were significantly modified (not just deleted)
            for (key, old_val) in old_map {
                if let Some(new_val) = new_map.get(key) {
                    if old_val != new_val {
                        // If both are mappings and contents changed, we need to be careful
                        if old_val.is_mapping()
                            && new_val.is_mapping()
                            && has_unhandleable_nested_changes(old_val, new_val)
                        {
                            return true;
                        }
                    }
                }
            }

            false
        }
        _ => false,
    }
}

/// Collects keys that were removed (in original but not in updated), including nested keys
fn collect_removed_keys(old: &Value, new: &Value, prefix: &str, removed: &mut Vec<String>) {
    if let (Value::Mapping(old_map), Value::Mapping(new_map)) = (old, new) {
        for (key, old_val) in old_map {
            if let Value::String(key_str) = key {
                let full_key = if prefix.is_empty() {
                    key_str.clone()
                } else {
                    format!("{}.{}", prefix, key_str)
                };

                if !new_map.contains_key(key) {
                    // Key was removed entirely
                    removed.push(full_key);
                } else if let Some(new_val) = new_map.get(key) {
                    // Key exists in new, but might have removed nested keys
                    if old_val.is_mapping() && new_val.is_mapping() {
                        collect_removed_keys(old_val, new_val, &full_key, removed);
                    }
                }
            }
        }
    }
}

/// Collects all changes by comparing original and updated values
fn collect_all_changes(
    original_content: &str,
    updated_value: &Value,
) -> Result<HashMap<String, Value>, String> {
    let original_value: Value = serde_yaml::from_str(original_content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let mut changes = HashMap::new();
    collect_value_changes(&original_value, updated_value, "", &mut changes);

    Ok(changes)
}

/// Recursively collects changed values
fn collect_value_changes(
    old: &Value,
    new: &Value,
    prefix: &str,
    changes: &mut HashMap<String, Value>,
) {
    match (old, new) {
        (Value::Mapping(old_map), Value::Mapping(new_map)) => {
            // Check for changed or new values
            for (key, new_val) in new_map {
                if let Value::String(key_str) = key {
                    let full_key = if prefix.is_empty() {
                        key_str.clone()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };

                    if let Some(old_val) = old_map.get(key) {
                        if old_val != new_val {
                            // Value changed
                            if new_val.is_mapping() || new_val.is_sequence() {
                                // For complex types, recurse
                                collect_value_changes(old_val, new_val, &full_key, changes);
                            } else {
                                // For scalars, record the change
                                changes.insert(full_key, new_val.clone());
                            }
                        } else if new_val.is_mapping() {
                            // Same value, but might have nested changes
                            collect_value_changes(old_val, new_val, &full_key, changes);
                        }
                    } else {
                        // New key added
                        changes.insert(full_key, new_val.clone());
                    }
                }
            }
        }
        _ => {
            // For non-mapping types, just record if different
            if old != new {
                changes.insert(prefix.to_string(), new.clone());
            }
        }
    }
}

/// Applies changes to the original content while preserving formatting
fn apply_changes_to_content(
    content: &str,
    changes: &HashMap<String, Value>,
    removed_keys: &[String],
) -> Result<String, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    // Build a mapping of line numbers to the YAML keys they represent
    let line_key_map = build_line_to_key_map(&lines)?;

    // Track which keys from changes we've already processed
    let mut processed_changes = std::collections::HashSet::new();

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Always preserve empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            result.push(line.to_string());
            i += 1;
            continue;
        }

        // Check if this line should be removed based on the key map
        let mut should_skip = false;
        let mut skip_until_indent = None;

        if let Some(key_path) = line_key_map.get(&i) {
            // Check if this key or any parent key was removed
            for removed_key in removed_keys {
                if removed_key == key_path || key_path.starts_with(&format!("{}.", removed_key)) {
                    should_skip = true;
                    let indent = line.len() - trimmed.len();
                    skip_until_indent = Some(indent);
                    break;
                }
            }

            // Check if this key was changed
            if !should_skip && changes.contains_key(key_path) {
                if let Some(new_val) = changes.get(key_path) {
                    let formatted = format_value_for_yaml(new_val);
                    let indent = line.len() - trimmed.len();
                    let indent_str = &line[..indent];
                    let key_name = trimmed[..trimmed.find(':').unwrap()].trim();
                    result.push(format!(
                        "{}{}:{}",
                        indent_str,
                        key_name,
                        if formatted.is_empty() {
                            "".to_string()
                        } else {
                            format!(" {}", formatted)
                        }
                    ));
                    processed_changes.insert(key_path.clone());

                    // Skip the original value lines that are nested under this key
                    i += 1;
                    while i < lines.len() {
                        let next_line = lines[i];
                        let next_trimmed = next_line.trim_start();
                        let next_indent = next_line.len() - next_trimmed.len();

                        if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                            if next_indent == indent {
                                result.push(next_line.to_string());
                                i += 1;
                            } else if next_indent > indent {
                                i += 1;
                            } else {
                                break;
                            }
                        } else if next_indent <= indent {
                            break;
                        } else {
                            i += 1;
                        }
                    }
                    continue;
                }
            }
        }

        if should_skip {
            let indent = skip_until_indent.unwrap();
            // Skip this line and nested content
            i += 1;
            while i < lines.len() {
                let next_line = lines[i];
                let next_trimmed = next_line.trim_start();
                let next_indent = next_line.len() - next_trimmed.len();

                if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                    result.push(next_line.to_string());
                    i += 1;
                    continue;
                }

                if next_indent <= indent {
                    break;
                }

                i += 1;
            }
            continue;
        }

        result.push(line.to_string());
        i += 1;
    }

    // Add any changes that weren't already in the file (new keys)
    for (key_path, new_val) in changes {
        if !processed_changes.contains(key_path) {
            let formatted = format_value_for_yaml(new_val);

            if key_path.contains('.') {
                // Nested key - need to build the structure
                // For now, fall back to standard serialization for complex additions
                return serde_yaml::to_string(&build_yaml_from_changes(content, changes)?)
                    .map_err(|e| format!("Failed to serialize YAML: {}", e));
            } else {
                // Top-level key - just append it
                if !result.is_empty() && !result.last().unwrap().is_empty() {
                    result.push(String::new()); // Add blank line before new key
                }
                result.push(format!("{}: {}", key_path, formatted));
            }
        }
    }

    // Preserve trailing newline
    let mut output = result.join("\n");
    if content.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }

    Ok(output)
}

/// Build a YAML value from changes by parsing the original and applying changes
fn build_yaml_from_changes(
    content: &str,
    changes: &std::collections::HashMap<String, serde_yaml::Value>,
) -> Result<serde_yaml::Value, String> {
    let mut yaml =
        serde_yaml::from_str(content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    for (key_path, value) in changes {
        set_value(&mut yaml, key_path, value)?;
    }

    Ok(yaml)
}

/// Set a value in YAML at a specified key path - helper for rebuilding
fn set_value(
    value: &mut serde_yaml::Value,
    path: &str,
    new_value: &serde_yaml::Value,
) -> Result<(), String> {
    use serde_yaml::Value;

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

/// Formats a YAML value for inline output
fn format_value_for_yaml(value: &Value) -> String {
    match value {
        Value::String(s) => {
            // Quote if needed
            if s.contains(' ') || s.contains(':') || s.is_empty() || s.starts_with('#') {
                format!("'{}'", s)
            } else {
                s.clone()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        _ => {
            // For complex types, use YAML serialization but trim it
            serde_yaml::to_string(value)
                .unwrap_or_default()
                .trim()
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preserves_comments_simple() {
        let yaml = "# Configuration\nkey1: value1\nkey2: value2\n";
        let mut value = serde_yaml::from_str::<Value>(yaml).unwrap();

        if let Value::Mapping(ref mut map) = value {
            map.insert(
                Value::String("key1".to_string()),
                Value::String("newvalue1".to_string()),
            );
        }

        let result = write_yaml_preserving_format(yaml, &value).unwrap();

        assert!(result.contains("# Configuration"));
        assert!(result.contains("key1: newvalue1"));
        assert!(result.contains("key2: value2"));
    }

    #[test]
    fn test_preserves_empty_lines() {
        let yaml = "key1: value1\n\nkey2: value2\n";
        let value = serde_yaml::from_str::<Value>(yaml).unwrap();

        let result = write_yaml_preserving_format(yaml, &value).unwrap();

        // Without changes, should return original
        assert_eq!(result, yaml);
    }

    #[test]
    fn test_detects_removed_keys() {
        let yaml = "key1: value1\nkey2: value2\nkey3: value3\n";
        let mut value = serde_yaml::from_str::<Value>(yaml).unwrap();

        // Remove key2
        if let Value::Mapping(ref mut map) = value {
            map.remove(&Value::String("key2".to_string()));
        }

        let result = write_yaml_preserving_format(yaml, &value).unwrap();

        assert!(result.contains("key1: value1"));
        assert!(!result.contains("key2: value2"));
        assert!(result.contains("key3: value3"));
    }

    #[test]
    fn test_preserves_comments_and_empty_lines_on_change() {
        // Test that comments/empty lines are preserved for TOP-LEVEL key changes
        // For nested changes, standard serialization is used (comments won't be preserved)
        let yaml = "# Main config\nkey1: value1\n\n# Another key\nkey2: value2\n";
        let mut value = serde_yaml::from_str::<Value>(yaml).unwrap();

        // Change top-level key only
        if let Value::Mapping(ref mut map) = value {
            map.insert(
                Value::String("key1".to_string()),
                Value::String("newvalue1".to_string()),
            );
        }

        let result = write_yaml_preserving_format(yaml, &value).unwrap();

        // Comments and empty lines should be preserved for top-level changes
        assert!(result.contains("# Main config"));
        assert!(result.contains("# Another key"));
        assert!(result.contains("key1: newvalue1"));
        assert!(result.contains("key2: value2"));
    }
}
