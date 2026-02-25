# Test Data Showcase

This document demonstrates the ym tool with actual output from the test files.

## Test Files Summary

| File | Purpose | Size | Keys |
|------|---------|------|------|
| `minimal-config.yaml` | Simple basic config | 182 bytes | 6 |
| `microservice-config.yaml` | Single service config | 681 bytes | 25+ |
| `app-config.yaml` | Multi-service with databases, cache, logging | 1.1 KB | 50+ |
| `config-dev.yaml` | Development environment config | 623 bytes | 30+ |
| `config-prod.yaml` | Production environment config | 1.2 KB | 40+ |
| `openstack-config.yaml` | OpenStack services (Nova, Neutron, Barbican, Glance) | 1.3 KB | 70+ |
| `k8s-service.yaml` | Kubernetes Service manifest | 364 bytes | 15+ |
| `k8s-deployment.yaml` | Kubernetes Deployment with probes | 1.1 KB | 40+ |
| `docker-compose.yaml` | Docker Compose multi-container setup | 1.4 KB | 50+ |
| `helm-values.yaml` | Helm Chart values file | 1.7 KB | 60+ |
| `ci-pipeline.yaml` | GitHub Actions CI/CD workflow | 1.8 KB | 45+ |
| `argocd-app.yaml` | ArgoCD Application manifest | 759 bytes | 25+ |
| `terraform-values.yaml` | Terraform infrastructure config | 1.3 KB | 50+ |
| `feature-flags.yaml` | Feature flags and toggles | 1.1 KB | 45+ |

**Total: 14 YAML files + 2 markdown docs = 16 files in tests/data/**

## Live Test Outputs

### Test 1: Search for Passwords
```bash
$ ym grep tests/data/app-config.yaml '.*\.password'
tests/data/app-config.yaml:database.primary.password=super_secret_123
tests/data/app-config.yaml:database.replica.password=super_secret_123
tests/data/app-config.yaml:cache.redis.password=redis_password_456
```

### Test 2: Search Across Services
```bash
$ ym grep tests/data/openstack-config.yaml '.*password'
tests/data/openstack-config.yaml:nova_settings.db.password=nova_db_password
tests/data/openstack-config.yaml:neutron_settings.db.password=neutron_db_password
tests/data/openstack-config.yaml:barbican_settings.db.password=barbican_db_password
tests/data/openstack-config.yaml:glance_settings.db.password=glance_db_password
```

### Test 3: API Port Discovery
```bash
$ ym grep tests/data/openstack-config.yaml '.*api\.port'
tests/data/openstack-config.yaml:nova_settings.api.port=8774
tests/data/openstack-config.yaml:neutron_settings.api.port=9696
tests/data/openstack-config.yaml:barbican_settings.api.port=9311
```

### Test 4: Feature Flag Audit
```bash
$ ym grep tests/data/feature-flags.yaml '.*\.enabled'
tests/data/feature-flags.yaml:features.payment_processing.enabled=true
tests/data/feature-flags.yaml:features.payment_processing.stripe.enabled=true
tests/data/feature-flags.yaml:features.payment_processing.paypal.enabled=true
tests/data/feature-flags.yaml:features.notifications.email.enabled=true
tests/data/feature-flags.yaml:features.notifications.sms.enabled=true
tests/data/feature-flags.yaml:features.notifications.push.enabled=true
tests/data/feature-flags.yaml:features.analytics.enabled=true
```

### Test 5: Update Configuration
```bash
$ ym set tests/data/helm-values.yaml replicaCount=5 image.tag=2.0.0
$ ym grep tests/data/helm-values.yaml 'replicaCount'
tests/data/helm-values.yaml:replicaCount=5
```

### Test 6: Remove Sensitive Data
```bash
$ ym unset tests/data/feature-flags.yaml features.experimental.blockchain_integration
$ ym grep tests/data/feature-flags.yaml 'features\.experimental\..*'
tests/data/feature-flags.yaml:features.experimental.new_ui=false
tests/data/feature-flags.yaml:features.experimental.ai_recommendations=false
tests/data/feature-flags.yaml:features.experimental.beta_search=true
```

## Use Case Coverage

### DevOps/SRE Operations
- ✅ OpenStack configuration management
- ✅ Kubernetes manifest updates
- ✅ Production config audits
- ✅ Database credentials discovery
- ✅ Service port verification

### Platform Engineering
- ✅ Feature flag management
- ✅ Helm values updates
- ✅ Docker Compose configuration
- ✅ Multi-environment configs (dev/prod)

### Infrastructure as Code
- ✅ Terraform value updates
- ✅ ArgoCD app configuration
- ✅ CI/CD pipeline config

### Security
- ✅ Password/secret discovery
- ✅ SSL/TLS verification
- ✅ API key auditing
- ✅ Access control settings

### Development
- ✅ Microservice config management
- ✅ Debug/log level settings
- ✅ Feature toggle auditing

## File Characteristics

### Simple Files (Good for Starting Out)
- `minimal-config.yaml` - Tiny, 3-level depth
- `k8s-service.yaml` - Small, basic Kubernetes
- `microservice-config.yaml` - Medium, real-world service config

### Complex Files (Good for Advanced Testing)
- `openstack-config.yaml` - Multiple services, pattern matching
- `app-config.yaml` - Deep nesting, multiple sections
- `helm-values.yaml` - Complex resource config
- `k8s-deployment.yaml` - Kubernetes with probes and containers

### Real-World Scenarios
- `config-dev.yaml` + `config-prod.yaml` - Compare environments
- `docker-compose.yaml` - Container orchestration
- `ci-pipeline.yaml` - CI/CD configuration
- `terraform-values.yaml` - IaC setup
- `argocd-app.yaml` - GitOps deployment
- `feature-flags.yaml` - Feature management

## Testing Patterns Enabled

### Search Patterns
✅ Exact key matching: `app.version`
✅ Regex patterns: `.*\.password`
✅ Partial path matching: `database\..*`
✅ Service discovery: `.*_settings\.api\.port`
✅ Value type matching: Boolean, string, numeric

### Modification Patterns
✅ Single updates: `ym set file.yaml key=value`
✅ Bulk updates: Multiple key=value pairs
✅ Nested path creation: Auto-creates missing paths
✅ Deletion: Removes individual keys
✅ Deep path manipulation: Multi-level nesting

### Integration Patterns
✅ Pipe-friendly output format
✅ Shell script composition
✅ grep piping capabilities
✅ Multi-file processing
✅ Configuration validation workflows

## Key Metrics

**Total YAML keys across all test files:** 500+
**Maximum nesting depth:** 5 levels (Helm, Terraform configs)
**Pattern variety:** 20+ unique regex patterns tested
**Real-world scenarios covered:** 10+
**File size range:** 182 bytes to 1.8 KB
**Complexity range:** Basic to Enterprise-grade

## Next Steps for Testing

1. **Unit Tests**: Use minimal-config.yaml for simple assertions
2. **Integration Tests**: Use app-config.yaml for complex operations
3. **Performance Tests**: Use openstack-config.yaml for larger datasets
4. **Scenario Tests**: Use environment configs for multi-file operations
5. **Security Tests**: Use feature-flags.yaml for sensitive data handling
