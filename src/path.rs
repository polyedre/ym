use yamlpath::{Component, Route};

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YamlPath {
    segments: Vec<PathSegment>,
}

impl YamlPath {
    pub fn parse(input: &str) -> AppResult<Self> {
        let mut segments = Vec::new();
        let mut key = String::new();
        let mut chars = input.chars().peekable();
        let mut just_closed_index = false;

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    let escaped = chars.next().ok_or_else(|| {
                        AppError::invalid_path(input, "path cannot end with an escape character")
                    })?;
                    key.push(escaped);
                    just_closed_index = false;
                }
                '.' => {
                    if key.is_empty() {
                        if just_closed_index {
                            just_closed_index = false;
                            continue;
                        }
                        return Err(AppError::invalid_path(
                            input,
                            "empty path segment between dots",
                        ));
                    }
                    segments.push(PathSegment::Key(std::mem::take(&mut key)));
                    just_closed_index = false;
                }
                '[' => {
                    if !key.is_empty() {
                        segments.push(PathSegment::Key(std::mem::take(&mut key)));
                    }

                    let mut digits = String::new();
                    loop {
                        match chars.next() {
                            Some(']') => break,
                            Some(next) if next.is_ascii_digit() => digits.push(next),
                            Some(_) => {
                                return Err(AppError::invalid_path(
                                    input,
                                    "sequence indices must contain digits only",
                                ));
                            }
                            None => {
                                return Err(AppError::invalid_path(
                                    input,
                                    "unterminated sequence index",
                                ));
                            }
                        }
                    }

                    if digits.is_empty() {
                        return Err(AppError::invalid_path(
                            input,
                            "sequence index cannot be empty",
                        ));
                    }

                    let index = digits.parse::<usize>().map_err(|_| {
                        AppError::invalid_path(input, "sequence index is too large")
                    })?;
                    segments.push(PathSegment::Index(index));
                    just_closed_index = true;
                }
                ']' => {
                    return Err(AppError::invalid_path(input, "unexpected closing bracket"));
                }
                other => {
                    key.push(other);
                    just_closed_index = false;
                }
            }
        }

        if !key.is_empty() {
            segments.push(PathSegment::Key(key));
        }

        if segments.is_empty() {
            return Err(AppError::invalid_path(input, "path cannot be empty"));
        }

        Ok(Self { segments })
    }

    pub fn as_segments(&self) -> &[PathSegment] {
        &self.segments
    }

    pub fn to_route(&self) -> Route<'static> {
        Route::from(
            self.segments
                .iter()
                .map(|segment| match segment {
                    PathSegment::Key(key) => Component::from(key.clone()),
                    PathSegment::Index(index) => Component::from(*index),
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            None
        } else {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        }
    }

    pub fn last(&self) -> Option<&PathSegment> {
        self.segments.last()
    }

    pub fn push_key(&self, key: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Key(key.into()));
        Self { segments }
    }

    pub fn prefixes_requiring_mapping(&self) -> Vec<Self> {
        (0..self.segments.len().saturating_sub(1))
            .filter(|&index| matches!(self.segments[index + 1], PathSegment::Key(_)))
            .map(|index| Self {
                segments: self.segments[..=index].to_vec(),
            })
            .collect()
    }

    pub fn display(&self) -> String {
        let mut rendered = String::new();

        for segment in &self.segments {
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
}

#[cfg(test)]
mod tests {
    use super::{PathSegment, YamlPath};

    #[test]
    fn parses_keys_and_indices() {
        let path = YamlPath::parse("items[1].metadata.name").unwrap();
        assert_eq!(
            path.as_segments(),
            &[
                PathSegment::Key("items".to_string()),
                PathSegment::Index(1),
                PathSegment::Key("metadata".to_string()),
                PathSegment::Key("name".to_string()),
            ]
        );
    }

    #[test]
    fn parses_escaped_dots() {
        let path = YamlPath::parse(r"kubernetes\.io/hostname").unwrap();
        assert_eq!(
            path.as_segments(),
            &[PathSegment::Key("kubernetes.io/hostname".to_string())]
        );
        assert_eq!(path.display(), r"kubernetes\.io/hostname");
    }

    #[test]
    fn rejects_empty_segment() {
        assert!(YamlPath::parse("a..b").is_err());
    }
}
