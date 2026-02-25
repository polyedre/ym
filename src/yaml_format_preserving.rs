use serde_yaml::Value;
use std::collections::HashMap;

/// Updates a YAML string while preserving comments, empty lines, and indentation
/// for top-level key modifications.
///
/// NOTE: This only preserves formatting for top-level key operations.
/// For nested key operations (e.g., database.primary.password), standard YAML
/// serialization is used as serde_yaml does not preserve comments in nested structures.
/// To fully preserve comments for all operations, a comment-aware YAML library
/// would be needed (e.g., using a library like `yaml-prs`).
pub fn write_yaml_preserving_format(
    original_content: &str,
    updated_value: &Value,
) -> Result<String, String> {
    // Parse the original to understand structure
    let original_value: Value = serde_yaml::from_str(original_content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    // Check if this is a simple top-level operation
    // If we're adding/removing/modifying nested structures, fall back to standard serialization
    if has_nested_changes(&original_value, updated_value) {
        // For nested changes, use standard YAML serialization
        // This is simpler and more reliable than trying to do line-based updates
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

/// Check if there are nested structure changes that we can't handle with line-based updates
fn has_nested_changes(old: &Value, new: &Value) -> bool {
    match (old, new) {
        (Value::Mapping(old_map), Value::Mapping(new_map)) => {
            // Check if all old keys still exist in new
            for (key, old_val) in old_map {
                if let Some(new_val) = new_map.get(key) {
                    // Check if the value changed and involves nested structures
                    if old_val != new_val && (old_val.is_mapping() || new_val.is_mapping()) {
                        // Nested structure change
                        return true;
                    }
                } else {
                    // Key was removed - this is okay for top-level operations
                }
            }

            // Check if all new keys existed in old (to detect if new nested structures were added)
            for (key, new_val) in new_map {
                if !old_map.contains_key(key) && new_val.is_mapping() {
                    // New nested structure added
                    return true;
                }
            }

            false
        }
        _ => false,
    }
}

/// Collects keys that were removed (in original but not in updated)
fn collect_removed_keys(old: &Value, new: &Value, prefix: &str, removed: &mut Vec<String>) {
    match (old, new) {
        (Value::Mapping(old_map), Value::Mapping(new_map)) => {
            for (key, _) in old_map {
                if let Value::String(key_str) = key {
                    if !new_map.contains_key(key) {
                        let full_key = if prefix.is_empty() {
                            key_str.clone()
                        } else {
                            format!("{}.{}", prefix, key_str)
                        };
                        removed.push(full_key);
                    }
                }
            }
        }
        _ => {}
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

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Always preserve empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            result.push(line.to_string());
            i += 1;
            continue;
        }

        // Try to parse key: value from this line
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let indent = line.len() - trimmed.len();

            // Check if this is a top-level key
            if indent == 0 {
                // Check if this key was removed
                if removed_keys.contains(&key.to_string()) {
                    // Skip this line and any associated nested content
                    let mut next_i = i + 1;
                    while next_i < lines.len() {
                        let next_line = lines[next_i];
                        let next_trimmed = next_line.trim_start();
                        let next_indent = next_line.len() - next_trimmed.len();

                        // Preserve comments and empty lines ONLY
                        if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                            result.push(next_line.to_string());
                            next_i += 1;
                            continue;
                        }

                        // Stop if we hit same-level or lower indent content (non-comment, non-empty)
                        if next_indent <= indent {
                            break;
                        }

                        // Skip intermediate indented lines (nested content)
                        next_i += 1;
                    }

                    i = next_i;
                    continue;
                }

                // Check if this key has been updated
                if changes.contains_key(key) {
                    if let Some(new_val) = changes.get(key) {
                        // Replace the line with updated value
                        let formatted = format_value_for_yaml(new_val);
                        result.push(format!("{}: {}", key, formatted));

                        // Skip original value lines (handle multi-line values)
                        // But preserve comments and empty lines at the same indentation level
                        let mut next_i = i + 1;
                        while next_i < lines.len() {
                            let next_line = lines[next_i];
                            let next_trimmed = next_line.trim_start();
                            let next_indent = next_line.len() - next_trimmed.len();

                            // Preserve comments and empty lines at top level
                            if (next_trimmed.is_empty() || next_trimmed.starts_with('#'))
                                && next_indent == 0
                            {
                                result.push(next_line.to_string());
                                next_i += 1;
                                continue;
                            }

                            // Stop if we hit same-level or lower indent content
                            if !next_trimmed.is_empty()
                                && !next_trimmed.starts_with('#')
                                && next_indent <= indent
                            {
                                break;
                            }

                            // Skip intermediate lines (indented nested content)
                            if next_indent > indent {
                                next_i += 1;
                                continue;
                            }

                            break;
                        }

                        i = next_i;
                        continue;
                    }
                }
            }
        }

        result.push(line.to_string());
        i += 1;
    }

    // Preserve trailing newline
    let mut output = result.join("\n");
    if content.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }

    Ok(output)
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
