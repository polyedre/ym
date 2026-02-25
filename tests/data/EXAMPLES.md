# YM Tool Examples

This document provides practical examples demonstrating the `ym` tool's capabilities using the test data files.

## Quick Start Examples

### Search for Passwords
Find all password fields across your configurations:

```bash
# In app-config
ym grep tests/data/app-config.yaml '.*\.password'

# Output:
# tests/data/app-config.yaml:database.primary.password=super_secret_123
# tests/data/app-config.yaml:database.replica.password=super_secret_123
# tests/data/app-config.yaml:cache.redis.password=redis_password_456
```

### Search for Host Addresses
Locate all database hosts:

```bash
ym grep tests/data/config-prod.yaml 'database\..*\.host'

# Output:
# tests/data/config-prod.yaml:database.primary.host=prod-db-primary.internal
# tests/data/config-prod.yaml:database.replica.host=prod-db-replica.internal
```

### Search for API Ports
Find all API port configurations:

```bash
ym grep tests/data/openstack-config.yaml '.*api\.port'

# Output:
# tests/data/openstack-config.yaml:nova_settings.api.port=8774
# tests/data/openstack-config.yaml:neutron_settings.api.port=9696
# tests/data/openstack-config.yaml:barbican_settings.api.port=9311
```

## Real-World Use Cases

### 1. DevOps Configuration Audit

**Scenario**: You need to find all database passwords in your configuration repository to ensure they're properly managed.

```bash
# Search across all OpenStack service configurations
ym grep tests/data/openstack-config.yaml '.*password'

# Verify all services have SSL enabled
ym grep tests/data/config-prod.yaml 'database\..*\.ssl'
```

### 2. Feature Flag Management

**Scenario**: You want to audit which payment providers are currently enabled in your application.

```bash
# Check payment processing configuration
ym grep tests/data/feature-flags.yaml 'features\.payment_processing\..*\.enabled'

# Output shows which payment methods are active
```

### 3. Infrastructure as Code Updates

**Scenario**: You need to bump the replica count in your Helm deployment.

```bash
# Update replica count
ym set tests/data/helm-values.yaml replicaCount=5

# Verify the change
ym grep tests/data/helm-values.yaml 'replicaCount'
```

### 4. Multi-Environment Configuration

**Scenario**: You have separate dev and prod configs and need to verify database settings.

```bash
# Check dev database settings
ym grep tests/data/config-dev.yaml 'database\.primary\..*'

# Check prod database settings  
ym grep tests/data/config-prod.yaml 'database\.primary\..*'

# Compare results for discrepancies
```

### 5. Kubernetes Deployment Configuration

**Scenario**: You need to verify and update container resource limits.

```bash
# View current resource requests
ym grep tests/data/k8s-deployment.yaml 'resources\.requests\..*'

# Update memory limit
ym set tests/data/k8s-deployment.yaml 'spec.template.spec.containers[0].resources.limits.memory'='1Gi'
```

## Advanced Patterns

### Finding All Configuration Keys at a Specific Level

**List all top-level sections:**
```bash
ym grep tests/data/app-config.yaml '^[a-z_]+$'
```

**List all database-related settings:**
```bash
ym grep tests/data/app-config.yaml 'database\..*'
```

### Security Audit Patterns

**Find all API keys and secrets:**
```bash
ym grep tests/data/config-prod.yaml '.*_(key|password|secret|token)$'
```

**Verify SSL is enabled on all database connections:**
```bash
ym grep tests/data/config-prod.yaml 'database\..*\.ssl'
```

### Configuration Validation

**Check all required service ports are configured:**
```bash
# Verify essential services have ports
ym grep tests/data/openstack-config.yaml '.*\.api\.port'

# Verify database connections exist
ym grep tests/data/openstack-config.yaml '.*_settings\.db\.connection_string'
```

## Batch Operations

### Update Multiple Related Values

**Scenario**: Increase resource limits across multiple services.

```bash
# Update both CPU and memory limits
ym set tests/data/helm-values.yaml \
  'resources.limits.cpu'='1000m' \
  'resources.limits.memory'='1Gi' \
  'resources.requests.cpu'='500m' \
  'resources.requests.memory'='512Mi'
```

### Rolling Configuration Changes

**Scenario**: Update all service API workers for improved throughput.

```bash
# Copy the config
cp tests/data/openstack-config.yaml config-updated.yaml

# Update all API worker counts
ym set config-updated.yaml nova_settings.api.workers=8
ym set config-updated.yaml neutron_settings.api.workers=8
ym set config-updated.yaml barbican_settings.api.workers=4
```

## Integration with Shell Scripts

### Find and Replace Pattern

**Script to update all password fields:**

```bash
#!/bin/bash
CONFIG_FILE=$1
NEW_PASSWORD=$2

# Find all passwords
ym grep "$CONFIG_FILE" '.*\.password' | while read line; do
    KEY=$(echo "$line" | cut -d'=' -f1 | cut -d':' -f2)
    ym set "$CONFIG_FILE" "$KEY=$NEW_PASSWORD"
done
```

### Configuration Validation Script

**Check required keys exist:**

```bash
#!/bin/bash
CONFIG_FILE=$1

# Check if critical keys are present
REQUIRED_KEYS=(
    "database.primary.host"
    "database.primary.password"
    "cache.redis.host"
)

for key in "${REQUIRED_KEYS[@]}"; do
    if ! ym grep "$CONFIG_FILE" "^$key$" > /dev/null; then
        echo "ERROR: Required key missing: $key"
        exit 1
    fi
done

echo "All required keys present!"
```

### Multi-File Search

**Search across all configs:**

```bash
#!/bin/bash
PATTERN=$1

for file in tests/data/*.yaml; do
    echo "=== $file ==="
    ym grep "$file" "$PATTERN"
done
```

## Testing Examples

### Unit Test Scenarios

**Test 1: Search returns correct values**
```bash
RESULT=$(ym grep tests/data/minimal-config.yaml 'app\.version')
[[ $RESULT == *"1.0.0"* ]] && echo "PASS" || echo "FAIL"
```

**Test 2: Update persists correctly**
```bash
cp tests/data/minimal-config.yaml /tmp/test.yaml
ym set /tmp/test.yaml app.version=2.0.0
RESULT=$(ym grep /tmp/test.yaml 'app\.version')
[[ $RESULT == *"2.0.0"* ]] && echo "PASS" || echo "FAIL"
```

**Test 3: Unset removes key**
```bash
cp tests/data/minimal-config.yaml /tmp/test.yaml
ym unset /tmp/test.yaml app.debug
! ym grep /tmp/test.yaml 'app\.debug' | grep -q 'true' && echo "PASS" || echo "FAIL"
```

## Performance Examples

### Large Configuration Files

**Test with large OpenStack config:**
```bash
# Measure grep performance
time ym grep tests/data/openstack-config.yaml '.*password'

# Measure set performance  
time ym set tests/data/openstack-config.yaml nova_settings.db.pool_size=30
```

## Troubleshooting

### Pattern Not Matching

**Debug: Check if key path is correct**
```bash
# Start with the root
ym grep tests/data/app-config.yaml 'database'

# Then be more specific
ym grep tests/data/app-config.yaml 'database\.primary'

# Finally use exact path
ym grep tests/data/app-config.yaml 'database\.primary\.host'
```

### File Not Found

**Ensure file exists:**
```bash
ls -la tests/data/config-prod.yaml

# Or use full path
ym grep /home/polyedre/code/ym/tests/data/config-prod.yaml 'database\..*'
```

### YAML Parse Errors

**Validate YAML syntax:**
```bash
# If ym fails to parse, check with standard tools
cat tests/data/minimal-config.yaml

# Ensure file has no duplicate document markers
head -5 tests/data/minimal-config.yaml
```
