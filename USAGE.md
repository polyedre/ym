# YM Tool - Usage Guide

## Commands

### grep - Search YAML by Key Path

Search for keys matching a regex pattern in YAML files.

**Single File (no filename in output):**
```bash
ym grep <file> <pattern>
```

**Recursive Directory Search (with filename in output):**
```bash
ym grep -R <directory> <pattern>
```

**Examples:**

Single file search:
```bash
$ ym grep config.yaml '.*\.password'
database.password=secret123
cache.redis.password=redis_secret
```

Recursive directory search:
```bash
$ ym grep -R . '.*\.password'
config/app.yaml:database.password=secret123
config/cache.yaml:redis.password=redis_secret
```

### set - Update YAML Values

Set values at specified key paths. Creates nested structures if they don't exist.

**Syntax:**
```bash
ym set <file> <key=value> [key=value] ...
```

**Examples:**

Single value:
```bash
ym set config.yaml app.version=2.0.0
```

Multiple values:
```bash
ym set config.yaml app.version=2.0.0 database.port=5433 app.debug=false
```

Create new nested path:
```bash
ym set config.yaml new.section.key=value
```

### unset - Remove Keys

Remove keys from YAML files.

**Syntax:**
```bash
ym unset <file> <key> [key] ...
```

**Examples:**

Remove single key:
```bash
ym unset config.yaml app.debug
```

Remove multiple keys:
```bash
ym unset config.yaml app.debug database.password cache.redis.host
```

Remove entire section:
```bash
ym unset config.yaml features.experimental
```

## Regex Pattern Matching

The grep command supports full regex patterns:

- `.*` - Match any characters
- `\.` - Escape literal dots (required for key paths)
- `^pattern$` - Exact match
- `|` - OR operator
- `[a-z]` - Character classes

**Common patterns:**

Find all passwords:
```bash
ym grep config.yaml '.*\.password'
```

Find database settings:
```bash
ym grep config.yaml 'database\..*'
```

Find API ports:
```bash
ym grep config.yaml '.*\.api\.port'
```

Find enabled features:
```bash
ym grep config.yaml '.*\.enabled'
```

Find either password or secret:
```bash
ym grep config.yaml '.*(password|secret)'
```

## Output Format

### Single File (Non-Recursive)
```
key.path=value
key.path=value
```

### Recursive Directory
```
path/to/file.yaml:key.path=value
path/to/file.yaml:key.path=value
```

## Integration with Shell

### Count matches
```bash
ym grep config.yaml '.*\.password' | wc -l
```

### Filter results
```bash
ym grep -R . 'database\..*' | grep -i password
```

### Pipe to other tools
```bash
ym grep -R . 'database\..*' | cut -d= -f2
```

### Process results in loop
```bash
ym grep -R config/ 'app\..*' | while IFS=: read file key; do
  echo "File: $file, Key: $key"
done
```

## Exit Codes

- `0` - Success
- `1` - Error (file not found, parse error, etc.)

## Performance

The tool is optimized for:
- Large YAML files (tested on multi-KB files)
- Deep nesting (5+ levels)
- Many keys (500+ keys)
- Regex pattern matching

Search performance is comparable to standard grep on same datasets.

## Limitations

### Current MVP
- Single file per set/unset operation
- Regex patterns for grep only (set/unset use exact paths)
- No support for arrays in paths yet
- YAML 1.1 compatible

### Deferred Features (Future Versions)
- Batch stdin operations
- Multiple output formats (JSON, YAML)
- Config files
- Interactive mode
- Cross-platform support (Windows, macOS)

## Examples

### Security Audit

Find all database passwords in your configs:
```bash
ym grep -R . '.*_settings\.db\.password'
ym grep -R . 'database\..*\.password'
```

Find all API keys:
```bash
ym grep -R . '.*_key|.*_secret|.*_token'
```

### Configuration Management

Update replica counts across services:
```bash
ym set helm-values.yaml replicaCount=5
```

Change database port in dev environment:
```bash
ym set config-dev.yaml database.primary.port=5433
```

### Environment Validation

Verify production settings:
```bash
ym grep config-prod.yaml 'database\.replica\..*'
ym grep config-prod.yaml '.*ssl'
```

Compare dev and prod:
```bash
echo "=== Dev database ===" 
ym grep config-dev.yaml 'database\.primary\.host'
echo "=== Prod database ===" 
ym grep config-prod.yaml 'database\.primary\.host'
```

### CI/CD Integration

Validate required keys exist:
```bash
ym grep config.yaml 'app\.version' > /dev/null && echo "OK" || echo "MISSING"
```

Count configuration items:
```bash
ym grep -R config/ '.*' | wc -l
```

## Troubleshooting

### Pattern not matching

Debug by testing progressively:
```bash
# Start with root
ym grep config.yaml 'database'

# More specific
ym grep config.yaml 'database\.primary'

# Full path
ym grep config.yaml 'database\.primary\.host'
```

### File not found

Verify file exists:
```bash
ls -la config.yaml
ym grep ./config.yaml '.*'
```

### YAML parse error

Validate YAML syntax:
```bash
cat config.yaml
# Check for syntax errors
```

### No results from recursive

Check if directory exists and contains YAML files:
```bash
find . -name "*.yaml" -o -name "*.yml"
ym grep -R . 'test_pattern'
```

## Quick Reference

```bash
# Search
ym grep file.yaml 'pattern'           # Single file
ym grep -R dir/ 'pattern'             # Recursive

# Update
ym set file.yaml key=value            # Single
ym set file.yaml k1=v1 k2=v2          # Multiple

# Delete
ym unset file.yaml key                # Single
ym unset file.yaml k1 k2              # Multiple
```
