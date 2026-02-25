# Test Data Directory Index

## Overview

This directory contains **14 comprehensive YAML test files** covering real-world configuration management scenarios. Each file is designed to showcase different aspects of the `ym` tool's capabilities.

## Quick Navigation

### 📚 Documentation Files

| File | Purpose |
|------|---------|
| **INDEX.md** | This file - navigation guide |
| **README.md** | Detailed description of each test file |
| **EXAMPLES.md** | Practical usage examples and patterns |
| **TEST_SHOWCASE.md** | Live test outputs and use case coverage |

### 🧪 Test YAML Files

#### Basic/Simple Configs (Start Here)
- **minimal-config.yaml** (182B) - Simplest example, 3-level nesting
- **microservice-config.yaml** (681B) - Single service with features
- **k8s-service.yaml** (364B) - Basic Kubernetes Service

#### Multi-Service/Environment Configs
- **config-dev.yaml** (623B) - Development environment settings
- **config-prod.yaml** (1.2K) - Production environment with replicas
- **app-config.yaml** (1.1K) - Multi-service app (DB, cache, logging)
- **openstack-config.yaml** (1.3K) - 4 OpenStack services pattern

#### Infrastructure/Orchestration
- **docker-compose.yaml** (1.4K) - Multi-container Docker setup
- **k8s-deployment.yaml** (1.1K) - Full Kubernetes Deployment
- **helm-values.yaml** (1.7K) - Helm Chart configuration
- **terraform-values.yaml** (1.3K) - IaC configuration

#### DevOps/Modern Stack
- **ci-pipeline.yaml** (1.8K) - GitHub Actions workflow
- **argocd-app.yaml** (759B) - GitOps application manifest
- **feature-flags.yaml** (1.1K) - Feature toggles and flags

## File Features at a Glance

### By Complexity

**🟢 Beginner** (Good for learning)
- minimal-config.yaml
- k8s-service.yaml
- microservice-config.yaml
- argocd-app.yaml

**🟡 Intermediate** (Real-world scenarios)
- config-dev.yaml / config-prod.yaml
- docker-compose.yaml
- ci-pipeline.yaml
- feature-flags.yaml

**🔴 Advanced** (Complex patterns)
- openstack-config.yaml (multiple services)
- app-config.yaml (deep nesting)
- helm-values.yaml (resource management)
- k8s-deployment.yaml (advanced k8s features)
- terraform-values.yaml (IaC complexity)

### By Use Case

**Security & Audit**
- app-config.yaml (passwords)
- config-prod.yaml (credentials)
- openstack-config.yaml (multiple secrets)
- feature-flags.yaml (access controls)

**DevOps/SRE**
- openstack-config.yaml
- k8s-deployment.yaml
- k8s-service.yaml
- docker-compose.yaml

**Infrastructure as Code**
- terraform-values.yaml
- helm-values.yaml
- argocd-app.yaml
- ci-pipeline.yaml

**Application Configuration**
- microservice-config.yaml
- config-dev.yaml
- config-prod.yaml
- app-config.yaml
- feature-flags.yaml

## Common Patterns to Search

### Passwords & Secrets
```bash
# Find all passwords
ym grep <file> '.*\.password'

# Find API keys
ym grep <file> '.*_key|.*token|.*secret'

# Find credentials
ym grep config-prod.yaml '.*username|.*password'
```

**Test files:** app-config.yaml, config-prod.yaml, openstack-config.yaml

### Service Configuration
```bash
# Find all database hosts
ym grep <file> '.*\.db\.host'

# Find all API ports
ym grep <file> '.*\.api\.port'

# Find all service URLs
ym grep <file> '.*\.url|.*\.endpoint'
```

**Test files:** openstack-config.yaml, app-config.yaml, microservice-config.yaml

### Feature Management
```bash
# Find all toggles
ym grep <file> '.*\.enabled'

# Find providers
ym grep <file> '.*\.provider'

# Find experimental features
ym grep <file> '.*experimental.*'
```

**Test files:** feature-flags.yaml, ci-pipeline.yaml

### Kubernetes Configuration
```bash
# Find container images
ym grep <file> '.*\.image'

# Find resource limits
ym grep <file> '.*\.resources\..*'

# Find port configurations
ym grep <file> '.*\.ports\..*'
```

**Test files:** k8s-deployment.yaml, k8s-service.yaml, helm-values.yaml

### Environment-Specific
```bash
# Compare dev vs prod
ym grep config-dev.yaml 'database\.primary\.host'
ym grep config-prod.yaml 'database\.primary\.host'
```

**Test files:** config-dev.yaml, config-prod.yaml

## Usage Workflows

### 1. Audit Configuration
```bash
# Find all passwords
ym grep app-config.yaml '.*password'

# Find SSL settings
ym grep config-prod.yaml '.*ssl'

# Verify required settings exist
ym grep openstack-config.yaml '.*_settings\.db\.connection_string'
```

### 2. Update Configuration
```bash
# Update single value
ym set helm-values.yaml replicaCount=5

# Update multiple values
ym set config-prod.yaml \
  database.primary.port=5433 \
  cache.redis.port=6380

# Create new section
ym set minimal-config.yaml features.new_feature=enabled
```

### 3. Remove Sensitive Data
```bash
# Remove passwords
ym unset app-config.yaml database.primary.password

# Remove entire section
ym unset feature-flags.yaml features.experimental

# Redact secrets
ym unset config-prod.yaml external_services.payment.api_key
```

### 4. Compare Environments
```bash
# Check dev config
ym grep config-dev.yaml 'database\..*'

# Check prod config
ym grep config-prod.yaml 'database\..*'

# Look for differences
diff <(ym grep config-dev.yaml '.*') <(ym grep config-prod.yaml '.*')
```

## Statistics

| Metric | Count |
|--------|-------|
| Total YAML files | 14 |
| Documentation files | 4 |
| Total files | 18 |
| Total approximate keys | 500+ |
| Maximum nesting depth | 5 levels |
| Smallest file | 182 bytes (minimal-config.yaml) |
| Largest file | 1.8 KB (ci-pipeline.yaml) |
| Unique regex patterns covered | 20+ |
| Real-world scenarios | 10+ |

## Recommended Learning Path

### Day 1: Learn Basics
1. Read **README.md**
2. Start with **minimal-config.yaml**
3. Try simple grep: `ym grep minimal-config.yaml 'app\..*'`
4. Try set: `ym set minimal-config.yaml app.version=2.0.0`
5. Try unset: `ym unset minimal-config.yaml app.debug`

### Day 2: Explore Patterns
1. Read **EXAMPLES.md**
2. Try **microservice-config.yaml**
3. Try **app-config.yaml**
4. Practice regex patterns from EXAMPLES.md
5. Compare dev vs prod configs

### Day 3: Advanced Scenarios
1. Read **TEST_SHOWCASE.md**
2. Work with **openstack-config.yaml**
3. Try **helm-values.yaml**
4. Create shell scripts using ym
5. Try multi-file operations

### Day 4: Real-World Use
1. Use **config-dev.yaml** + **config-prod.yaml**
2. Work with **terraform-values.yaml**
3. Explore **feature-flags.yaml**
4. Create validation scripts
5. Build automation workflows

## Tips & Tricks

### Pattern Matching
- Use `.*` to match anything
- Use `\.` to escape dots
- Use `^pattern$` for exact matches
- Test simple patterns first, then add complexity

### File Selection
- Start with small files (minimal, k8s-service)
- Move to medium complexity (configs, microservice)
- Then tackle complex ones (openstack, helm)

### Common Operations
```bash
# Find all top-level keys
ym grep <file> '^[a-z_]+$'

# Find specific nesting level
ym grep <file> '^[a-z_]+\.[a-z_]+$'

# Find with values
ym grep <file> 'password=.*'
```

## Troubleshooting

### No Results
- Pattern might be too specific
- Try matching just the key name first
- Check file exists: `cat <file>`

### Pattern Not Matching
- Escape dots: `\.` not `.`
- Use `.*` for wildcards
- Test pattern incrementally

### File Not Found
- Check path is correct
- Verify file is valid YAML
- Check permissions

## Contributing

To add new test files:
1. Create a realistic YAML file in tests/data/
2. Update README.md with description
3. Add examples to EXAMPLES.md
4. Update TEST_SHOWCASE.md statistics
5. Update this INDEX.md

## Questions?

- See **README.md** for file descriptions
- See **EXAMPLES.md** for usage patterns
- See **TEST_SHOWCASE.md** for live examples
- Run `ym --help` for command help

---

**Total Test Coverage: 14 YAML files covering 500+ keys across 10+ real-world scenarios**
