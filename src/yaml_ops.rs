use std::fs;
use std::path::Path;

use regex::Regex;
use serde_yaml::Value;
use yamlpatch::{Op, Patch};
use yamlpath::Document;

use crate::error::{AppError, AppResult};
use crate::path::{PathSegment, YamlPath};

const PLACEHOLDER_KEY: &str = "__ym_placeholder__";

pub fn grep(value: &Value, pattern: &str) -> AppResult<Vec<(String, Value)>> {
    let regex = Regex::new(pattern)?;
    let mut results = Vec::new();
    let mut path = Vec::new();
    collect_matching_keys(value, &regex, &mut path, &mut results);
    Ok(results)
}

fn collect_matching_keys(
    value: &Value,
    regex: &Regex,
    path: &mut Vec<PathSegment>,
    results: &mut Vec<(String, Value)>,
) {
    match value {
        Value::Mapping(map) => {
            for (key, value) in map {
                let Value::String(key) = key else {
                    continue;
                };

                path.push(PathSegment::Key(key.clone()));
                let rendered = render_path(path);
                if regex.is_match(&rendered) {
                    results.push((rendered, value.clone()));
                } else {
                    collect_matching_keys(value, regex, path, results);
                }
                path.pop();
            }
        }
        Value::Sequence(sequence) => {
            for (index, value) in sequence.iter().enumerate() {
                path.push(PathSegment::Index(index));
                collect_matching_keys(value, regex, path, results);
                path.pop();
            }
        }
        _ => {}
    }
}

fn render_path(path: &[PathSegment]) -> String {
    let mut rendered = String::new();

    for segment in path {
        match segment {
            PathSegment::Key(key) => {
                if !rendered.is_empty() {
                    rendered.push('.');
                }

                for ch in key.chars() {
                    match ch {
                        '.' | '[' | ']' | '\\' => {
                            rendered.push('\\');
                            rendered.push(ch);
                        }
                        other => rendered.push(other),
                    }
                }
            }
            PathSegment::Index(index) => {
                rendered.push('[');
                rendered.push_str(&index.to_string());
                rendered.push(']');
            }
        }
    }

    rendered
}

fn parse_user_value(input: &str) -> AppResult<Value> {
    serde_yaml::from_str(input)
        .map_err(|error| AppError::parse_yaml(format!("from value '{input}'"), error))
}

fn parse_yaml_document(yaml_content: &str, context: &str) -> AppResult<Value> {
    serde_yaml::from_str(yaml_content)
        .map_err(|error| AppError::parse_yaml(context.to_string(), error))
}

fn apply_patch(yaml_content: &str, patch: Patch<'static>) -> AppResult<String> {
    let document =
        Document::new(yaml_content).map_err(|error| AppError::patch(error.to_string()))?;
    let updated = yamlpatch::apply_yaml_patches(&document, &[patch])
        .map_err(|error| AppError::patch(error.to_string()))?;
    Ok(updated.source().to_string())
}

fn placeholder_mapping() -> Value {
    let mut placeholder = serde_yaml::Mapping::new();
    placeholder.insert(Value::String(PLACEHOLDER_KEY.to_string()), Value::Null);
    Value::Mapping(placeholder)
}

fn yaml_set(yaml_content: &str, key_path: &str, new_value: Value) -> AppResult<String> {
    let path = YamlPath::parse(key_path)?;
    let mut result = yaml_content.to_string();

    for prefix in path.prefixes_requiring_mapping() {
        result = ensure_mapping_at_path(&result, &prefix)?;
    }

    let current = parse_yaml_document(&result, "from document")?;
    let existing_value = get_value_at_path(&current, &path)?;
    let setting_mapping = matches!(new_value, Value::Mapping(_));

    let updated = match new_value {
        Value::Mapping(new_map) => set_mapping_at_path(&result, &path, existing_value, new_map)?,
        new_value => match existing_value {
            Some(_) => replace_value_at_path(&result, &path, new_value)?,
            None => add_value_at_path(&result, &path, new_value)?,
        },
    };

    cleanup_placeholders(&updated, &cleanup_paths_for_set(&path, setting_mapping))
}

fn ensure_mapping_at_path(yaml_content: &str, path: &YamlPath) -> AppResult<String> {
    let current = parse_yaml_document(yaml_content, "from document")?;

    match get_value_at_path(&current, path)? {
        Some(Value::Mapping(_)) => Ok(yaml_content.to_string()),
        Some(_) => replace_with_empty_mapping_at_path(yaml_content, path),
        None => add_empty_mapping_at_path(yaml_content, path),
    }
}

fn replace_with_empty_mapping_at_path(yaml_content: &str, path: &YamlPath) -> AppResult<String> {
    let removed = remove_at_path(yaml_content, path)?;
    add_empty_mapping_at_path(&removed, path)
}

fn add_empty_mapping_at_path(yaml_content: &str, path: &YamlPath) -> AppResult<String> {
    let parent = path
        .parent()
        .map(|parent| parent.to_route())
        .unwrap_or_default();

    let Some(PathSegment::Key(key)) = path.last() else {
        return Err(AppError::message(format!(
            "Cannot create a mapping at sequence path '{}'",
            path.display()
        )));
    };

    apply_patch(
        yaml_content,
        Patch {
            route: parent,
            operation: Op::Add {
                key: key.clone(),
                value: placeholder_mapping(),
            },
        },
    )
}

fn add_value_at_path(yaml_content: &str, path: &YamlPath, new_value: Value) -> AppResult<String> {
    match path.last() {
        Some(PathSegment::Key(key)) => {
            let parent = path
                .parent()
                .map(|parent| parent.to_route())
                .unwrap_or_default();
            apply_patch(
                yaml_content,
                Patch {
                    route: parent,
                    operation: Op::Add {
                        key: key.clone(),
                        value: new_value,
                    },
                },
            )
        }
        Some(PathSegment::Index(index)) => {
            append_value_at_path(yaml_content, path, *index, new_value)
        }
        None => Err(AppError::message("Empty key path")),
    }
}

fn append_value_at_path(
    yaml_content: &str,
    path: &YamlPath,
    index: usize,
    new_value: Value,
) -> AppResult<String> {
    let parent_path = path.parent().ok_or_else(|| {
        AppError::message(format!(
            "Cannot set root sequence index '{}'",
            path.display()
        ))
    })?;
    let current = parse_yaml_document(yaml_content, "from document")?;

    match get_value_at_path(&current, &parent_path)? {
        Some(Value::Sequence(sequence)) if index == sequence.len() => apply_patch(
            yaml_content,
            Patch {
                route: parent_path.to_route(),
                operation: Op::Append { value: new_value },
            },
        ),
        Some(Value::Sequence(sequence)) => Err(AppError::message(format!(
            "Cannot create sparse sequence entry at '{}'; next valid index is {}",
            path.display(),
            sequence.len()
        ))),
        Some(_) => Err(AppError::message(format!(
            "Parent of '{}' is not a sequence",
            path.display()
        ))),
        None => Err(AppError::message(format!(
            "Parent sequence '{}' does not exist",
            parent_path.display()
        ))),
    }
}

fn replace_value_at_path(
    yaml_content: &str,
    path: &YamlPath,
    new_value: Value,
) -> AppResult<String> {
    apply_patch(
        yaml_content,
        Patch {
            route: path.to_route(),
            operation: Op::Replace(new_value),
        },
    )
}

fn remove_at_path(yaml_content: &str, path: &YamlPath) -> AppResult<String> {
    apply_patch(
        yaml_content,
        Patch {
            route: path.to_route(),
            operation: Op::Remove,
        },
    )
}

fn set_mapping_at_path(
    yaml_content: &str,
    path: &YamlPath,
    existing_value: Option<Value>,
    new_map: serde_yaml::Mapping,
) -> AppResult<String> {
    let mut result = match existing_value {
        Some(Value::Mapping(current_map)) => {
            remove_missing_mapping_keys(yaml_content, path, &current_map, &new_map)?
        }
        Some(_) => replace_with_empty_mapping_at_path(yaml_content, path)?,
        None => add_empty_mapping_at_path(yaml_content, path)?,
    };

    for (key, value) in new_map {
        let Value::String(key) = key else {
            return Err(AppError::message(format!(
                "Unsupported non-string key under '{}'",
                path.display()
            )));
        };

        result = yaml_set(&result, &path.push_key(key).display(), value)?;
    }

    cleanup_placeholders(&result, std::slice::from_ref(path))
}

fn remove_missing_mapping_keys(
    yaml_content: &str,
    path: &YamlPath,
    current_map: &serde_yaml::Mapping,
    new_map: &serde_yaml::Mapping,
) -> AppResult<String> {
    let mut result = yaml_content.to_string();

    for key in current_map.keys().filter_map(|key| match key {
        Value::String(key) if key != PLACEHOLDER_KEY => Some(key.clone()),
        _ => None,
    }) {
        if !new_map.contains_key(Value::String(key.clone())) {
            result = remove_at_path(&result, &path.push_key(key))?;
        }
    }

    Ok(result)
}

fn cleanup_paths_for_set(path: &YamlPath, setting_mapping: bool) -> Vec<YamlPath> {
    let mut paths = path.prefixes_requiring_mapping();
    paths.reverse();
    if setting_mapping {
        paths.insert(0, path.clone());
    }
    paths
}

fn cleanup_placeholders(yaml_content: &str, paths: &[YamlPath]) -> AppResult<String> {
    let mut result = yaml_content.to_string();

    for path in paths {
        let current = parse_yaml_document(&result, "from document")?;
        let placeholder_path = path.push_key(PLACEHOLDER_KEY);
        if get_value_at_path(&current, &placeholder_path)?.is_some() {
            result = remove_at_path(&result, &placeholder_path)?;
        }
    }

    Ok(result)
}

pub fn set_values(yaml_content: &str, updates: &[(String, String)]) -> AppResult<String> {
    let mut result = yaml_content.to_string();

    for (key_path, new_value) in updates {
        result = yaml_set(&result, key_path, parse_user_value(new_value)?)?;
    }

    Ok(result)
}

pub fn unset_values(yaml_content: &str, keys: &[String]) -> AppResult<String> {
    let mut result = yaml_content.to_string();

    for key_path in keys {
        let path = YamlPath::parse(key_path)?;
        let current = parse_yaml_document(&result, "from document")?;
        if get_value_at_path(&current, &path)?.is_some() {
            result = remove_at_path(&result, &path)?;
        }
    }

    Ok(result)
}

pub fn get_value(value: &Value, path: &str) -> AppResult<Option<Value>> {
    let path = YamlPath::parse(path)?;
    get_value_at_path(value, &path)
}

fn get_value_at_path(value: &Value, path: &YamlPath) -> AppResult<Option<Value>> {
    let mut current = value;

    for segment in path.as_segments() {
        match segment {
            PathSegment::Key(key) => {
                let Value::Mapping(map) = current else {
                    return Ok(None);
                };
                match map.get(Value::String(key.clone())) {
                    Some(next) => current = next,
                    None => return Ok(None),
                }
            }
            PathSegment::Index(index) => {
                let Value::Sequence(sequence) = current else {
                    return Ok(None);
                };
                match sequence.get(*index) {
                    Some(next) => current = next,
                    None => return Ok(None),
                }
            }
        }
    }

    Ok(Some(current.clone()))
}

pub fn copy_in_document(yaml_content: &str, source_key: &str, dest_key: &str) -> AppResult<String> {
    let source_yaml = parse_yaml_document(yaml_content, "from source document")?;
    let value = get_value(&source_yaml, source_key)?.ok_or_else(|| {
        AppError::message(format!("Key '{source_key}' not found in source document"))
    })?;

    yaml_set(yaml_content, dest_key, value)
}

pub fn move_in_document(yaml_content: &str, source_key: &str, dest_key: &str) -> AppResult<String> {
    if source_key == dest_key {
        return Ok(yaml_content.to_string());
    }

    let copied = copy_in_document(yaml_content, source_key, dest_key)?;
    unset_values(&copied, &[source_key.to_string()])
}

pub fn copy_value(
    source_file: &str,
    source_key: &str,
    dest_file: &str,
    dest_key: &str,
) -> AppResult<()> {
    let source_contents =
        fs::read_to_string(source_file).map_err(|error| AppError::read_file(source_file, error))?;

    let dest_contents = if source_file == dest_file {
        source_contents.clone()
    } else if Path::new(dest_file).exists() {
        fs::read_to_string(dest_file).map_err(|error| AppError::read_file(dest_file, error))?
    } else {
        "{}".to_string()
    };

    let source_yaml = parse_yaml_document(&source_contents, &format!("from '{source_file}'"))?;
    let value = get_value(&source_yaml, source_key)?.ok_or_else(|| {
        AppError::message(format!("Key '{source_key}' not found in '{source_file}'"))
    })?;
    let updated = yaml_set(&dest_contents, dest_key, value)?;

    fs::write(dest_file, updated).map_err(|error| AppError::write_file(dest_file, error))?;
    Ok(())
}

pub fn move_value(
    source_file: &str,
    source_key: &str,
    dest_file: &str,
    dest_key: &str,
) -> AppResult<()> {
    let source_contents =
        fs::read_to_string(source_file).map_err(|error| AppError::read_file(source_file, error))?;

    if source_file == dest_file {
        let updated = move_in_document(&source_contents, source_key, dest_key)?;
        fs::write(source_file, updated)
            .map_err(|error| AppError::write_file(source_file, error))?;
        return Ok(());
    }

    let source_yaml = parse_yaml_document(&source_contents, &format!("from '{source_file}'"))?;
    let value = get_value(&source_yaml, source_key)?.ok_or_else(|| {
        AppError::message(format!("Key '{source_key}' not found in '{source_file}'"))
    })?;

    let dest_contents = if Path::new(dest_file).exists() {
        fs::read_to_string(dest_file).map_err(|error| AppError::read_file(dest_file, error))?
    } else {
        "{}".to_string()
    };

    let updated_dest = yaml_set(&dest_contents, dest_key, value)?;
    let updated_source = unset_values(&source_contents, &[source_key.to_string()])?;

    fs::write(dest_file, updated_dest).map_err(|error| AppError::write_file(dest_file, error))?;
    fs::write(source_file, updated_source)
        .map_err(|error| AppError::write_file(source_file, error))?;
    Ok(())
}

pub fn format_result(key: &str, value: &Value, terminal_width: usize) -> String {
    match value {
        Value::Mapping(_) => format_mapping_result(key, value),
        Value::String(s) => truncate_if_needed(&format!("{key}: {s}"), terminal_width),
        Value::Number(n) => truncate_if_needed(&format!("{key}: {n}"), terminal_width),
        Value::Bool(b) => truncate_if_needed(&format!("{key}: {b}"), terminal_width),
        Value::Null => format!("{key}: null"),
        _ => {
            let val_str = serde_yaml::to_string(value)
                .unwrap_or_else(|_| "<complex>".to_string())
                .trim()
                .to_string();
            truncate_if_needed(&format!("{key}: {val_str}"), terminal_width)
        }
    }
}

fn truncate_if_needed(text: &str, terminal_width: usize) -> String {
    if text.len() > terminal_width {
        format!("{}...", &text[..terminal_width.saturating_sub(3)])
    } else {
        text.to_string()
    }
}

fn format_mapping_result(key: &str, value: &Value) -> String {
    let yaml_str = match serde_yaml::to_string(value) {
        Ok(result) => result,
        Err(_) => return format!("{key}: <error>"),
    };

    let indented = yaml_str
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("  {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{key}:\n{indented}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_yaml(yaml_str: &str) -> Value {
        serde_yaml::from_str(yaml_str).expect("Failed to parse YAML")
    }

    fn temp_test_dir(name: &str) -> String {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = format!("{}_{}_{}", name, std::process::id(), unique);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn test_grep_simple_key() {
        let yaml = parse_yaml("name: Alice\nage: 30");
        let results = grep(&yaml, "name").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "name");
        assert_eq!(results[0].1.as_str(), Some("Alice"));
    }

    #[test]
    fn test_grep_compiles_regex_once_and_matches_nested_keys() {
        let yaml = parse_yaml("database:\n  host: localhost\n  port: 5432\n");
        let results = grep(&yaml, r"^database\.(host|port)$").unwrap();
        let keys: Vec<_> = results.into_iter().map(|result| result.0).collect();
        assert_eq!(keys, vec!["database.host", "database.port"]);
    }

    #[test]
    fn test_grep_escapes_dotted_keys() {
        let yaml = parse_yaml("metadata:\n  kubernetes.io/hostname: node-a\n");
        let results = grep(&yaml, r"metadata\.kubernetes\\\.io/hostname$").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, r"metadata.kubernetes\.io/hostname");
    }

    #[test]
    fn test_grep_sequence_paths() {
        let yaml = parse_yaml("items:\n  - name: first\n  - name: second\n");
        let results = grep(&yaml, r"items\[1\]\.name").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "items[1].name");
        assert_eq!(results[0].1.as_str(), Some("second"));
    }

    #[test]
    fn test_grep_invalid_regex() {
        let yaml = parse_yaml("name: Alice");
        assert!(grep(&yaml, "[invalid").is_err());
    }

    #[test]
    fn test_set_and_unset_values_update_yaml_semantics() {
        let yaml_str = "database:\n  host: localhost\n  port: 5432\nconfig:\n  level: info\n";
        let updates = vec![
            ("database.port".to_string(), "3306".to_string()),
            ("database.username".to_string(), "admin".to_string()),
            ("app.server.config.timeout".to_string(), "30".to_string()),
        ];

        let updated = set_values(yaml_str, &updates).unwrap();
        let parsed = parse_yaml(&updated);
        assert_eq!(parsed["database"]["host"].as_str(), Some("localhost"));
        assert_eq!(parsed["database"]["port"].as_i64(), Some(3306));
        assert_eq!(parsed["database"]["username"].as_str(), Some("admin"));
        assert_eq!(
            parsed["app"]["server"]["config"]["timeout"].as_i64(),
            Some(30)
        );

        let removed = unset_values(
            &updated,
            &[
                "database.port".to_string(),
                "app.server.config.timeout".to_string(),
            ],
        )
        .unwrap();
        let parsed = parse_yaml(&removed);
        assert!(get_value(&parsed, "database.port").unwrap().is_none());
        assert!(get_value(&parsed, "app.server.config.timeout")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_set_supports_escaped_dotted_keys() {
        let updated = set_values(
            "metadata: {}\n",
            &[(
                r"metadata.kubernetes\.io/hostname".to_string(),
                "node-a".to_string(),
            )],
        )
        .unwrap();
        let parsed = parse_yaml(&updated);
        assert_eq!(
            parsed["metadata"]["kubernetes.io/hostname"].as_str(),
            Some("node-a")
        );
    }

    #[test]
    fn test_set_supports_appending_to_sequences() {
        let updated = set_values(
            "items:\n  - first\n",
            &[("items[1]".to_string(), "second".to_string())],
        )
        .unwrap();
        let parsed = parse_yaml(&updated);
        assert_eq!(parsed["items"][1].as_str(), Some("second"));
    }

    #[test]
    fn test_get_value_supports_sequences_and_escaped_keys() {
        let yaml = parse_yaml("items:\n  - metadata:\n      kubernetes.io/hostname: node-a\n");
        let value = get_value(&yaml, r"items[0].metadata.kubernetes\.io/hostname").unwrap();
        assert_eq!(value.unwrap().as_str(), Some("node-a"));
    }

    #[test]
    fn test_set_mapping_may_rewrite_touched_key_but_preserves_untouched_layout() {
        let original = concat!(
            "# top comment\n",
            "before: keep\n",
            "\n",
            "app:\n",
            "  # touched comment\n",
            "  debug: true\n",
            "  logging:\n",
            "    level: warn\n",
            "\n",
            "# trailing comment\n",
            "after: stay\n",
        );

        let updated = set_values(
            original,
            &[(
                String::from("app"),
                String::from("debug: false\nlogging:\n  level: info\n  format: json"),
            )],
        )
        .unwrap();

        let parsed = parse_yaml(&updated);
        assert_eq!(parsed["app"]["debug"].as_bool(), Some(false));
        assert_eq!(parsed["app"]["logging"]["level"].as_str(), Some("info"));
        assert_eq!(parsed["app"]["logging"]["format"].as_str(), Some("json"));
        assert!(updated.contains("# top comment\nbefore: keep\n\n"));
        assert!(updated.contains("\n# trailing comment\nafter: stay\n"));
    }

    #[test]
    fn test_copy_in_document_and_move_in_document() {
        let original = "source:\n  nested:\n    key: value\nkeep: yes\n";
        let copied = copy_in_document(original, "source.nested", "dest.nested").unwrap();
        let copied_yaml = parse_yaml(&copied);
        assert_eq!(copied_yaml["dest"]["nested"]["key"].as_str(), Some("value"));
        assert_eq!(
            copied_yaml["source"]["nested"]["key"].as_str(),
            Some("value")
        );

        let moved = move_in_document(original, "source.nested", "dest.nested").unwrap();
        let moved_yaml = parse_yaml(&moved);
        assert!(get_value(&moved_yaml, "source.nested").unwrap().is_none());
        assert_eq!(moved_yaml["dest"]["nested"]["key"].as_str(), Some("value"));
    }

    #[test]
    fn test_copy_value_handles_scalars_and_mappings() {
        let test_dir = temp_test_dir("test_copy_value");
        let source_file = format!("{}/source.yaml", test_dir);
        let dest_file = format!("{}/dest.yaml", test_dir);

        fs::write(
            &source_file,
            "data:\n  value: test123\nconfig:\n  nested:\n    count: 42\n",
        )
        .unwrap();
        fs::write(&dest_file, "other: value\n").unwrap();

        copy_value(&source_file, "data.value", &dest_file, "copied.value").unwrap();
        copy_value(&source_file, "config.nested", &dest_file, "backup.config").unwrap();

        let yaml = serde_yaml::from_str::<Value>(&fs::read_to_string(&dest_file).unwrap()).unwrap();
        assert_eq!(yaml["other"].as_str(), Some("value"));
        assert_eq!(yaml["copied"]["value"].as_str(), Some("test123"));
        assert_eq!(yaml["backup"]["config"]["count"].as_i64(), Some(42));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_move_value_updates_destination_and_removes_source() {
        let test_dir = temp_test_dir("test_move_value");
        let source_file = format!("{}/source.yaml", test_dir);
        let dest_file = format!("{}/dest.yaml", test_dir);

        fs::write(
            &source_file,
            "source:\n  nested:\n    key: moved_value\nkeep: yes\n",
        )
        .unwrap();
        fs::write(&dest_file, "other: data\n").unwrap();

        move_value(&source_file, "source.nested", &dest_file, "dest.nested").unwrap();

        let dest_yaml =
            serde_yaml::from_str::<Value>(&fs::read_to_string(&dest_file).unwrap()).unwrap();
        let source_yaml =
            serde_yaml::from_str::<Value>(&fs::read_to_string(&source_file).unwrap()).unwrap();
        assert_eq!(
            dest_yaml["dest"]["nested"]["key"].as_str(),
            Some("moved_value")
        );
        assert_eq!(source_yaml["keep"].as_str(), Some("yes"));
        assert!(get_value(&source_yaml, "source.nested").unwrap().is_none());

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_format_string_value() {
        let value = Value::String("hello".to_string());
        assert_eq!(format_result("message", &value, 80), "message: hello");
    }

    #[test]
    fn test_format_mapping_value() {
        let value = parse_yaml("host: localhost\nport: 5432");
        let result = format_result("database", &value, 80);
        assert!(result.starts_with("database:\n"));
        assert!(result.contains("host"));
    }

    #[test]
    fn test_format_truncates_long_string() {
        let long_string = "a".repeat(100);
        let value = Value::String(long_string);
        let result = format_result("key", &value, 20);
        assert!(result.ends_with("..."));
    }
}
