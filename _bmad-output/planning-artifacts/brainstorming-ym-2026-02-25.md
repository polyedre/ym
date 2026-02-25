---
date: 2026-02-25
author: Polyedre
---

# Brainstorming: ym CLI Tool

## Core Vision
A CLI tool to manipulate YAML files with operations for searching, inserting, and deleting keys.

## Key Features

### 1. Grep Operation (Search)
- **Command**: `ym grep -R . .*_settings\..*\.password`
- **Purpose**: Search for keys matching regex patterns
- **Return Format**: `file:nova_settings.db.password` (one result per line)
- **Supports**: Recursive directory traversal with `-R` flag

### 2. Set Operation (Insert/Update)
- **Command**: `yq set file.yaml nova_settings.username=toto barbican_settings.username=myuser`
- **Purpose**: Insert or update keys in YAML mappings
- **Supports**: Multiple key-value pairs in single command
- **No Regex**: Direct key path specification

### 3. Unset Operation (Delete)
- **Command**: `yq unset file.yaml nova_settings.username`
- **Purpose**: Remove specific keys from YAML files
- **Supports**: Direct key path specification

## Technical Considerations
- YAML file manipulation
- Regex pattern matching for grep
- Nested key/mapping support (dot notation: `nova_settings.username`)
- Recursive directory traversal
- Bulk operations
