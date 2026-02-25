# Test Data Examples

This directory contains comprehensive example YAML files showcasing various real-world use cases for the `ym` tool.

## Files Overview

### 1. **k8s-service.yaml**
Kubernetes Service manifest. Demonstrates:
- Service metadata and labels
- Port configuration with protocols
- Selector matching

**Example grep queries:**
```bash
ym grep tests/data/k8s-service.yaml 'metadata\..*'
ym grep tests/data/k8s-service.yaml 'spec\.ports\[0\]\..*'
```

### 2. **k8s-deployment.yaml**
Kubernetes Deployment with containers, resources, and probes. Demonstrates:
- Multi-level nested structure
- Container configuration
- Resource limits and requests
- Environment variables
- Liveness and readiness probes

**Example queries:**
```bash
ym grep tests/data/k8s-deployment.yaml 'spec\.template\.spec\.containers\[0\]\.image'
ym grep tests/data/k8s-deployment.yaml '.*\.resources\..*'
```

### 3. **app-config.yaml**
Comprehensive application configuration. Demonstrates:
- Multiple service sections (database, cache, logging)
- Nested configuration with multiple levels
- Passwords and secrets
- Boolean and numeric values
- List configurations

**Example queries:**
```bash
ym grep tests/data/app-config.yaml '.*\.password'
ym grep tests/data/app-config.yaml 'database\..*\.host'
ym grep tests/data/app-config.yaml 'logging\..*'
```

### 4. **openstack-config.yaml**
OpenStack/Hiera-style configuration with multiple services. Demonstrates:
- Pattern matching across similar service configurations
- Database, API, and messaging sections
- Connection strings and credentials

**Example queries:**
```bash
ym grep tests/data/openstack-config.yaml '.*_settings\.db\..*'
ym grep tests/data/openstack-config.yaml '.*_settings\.api\.port'
ym grep tests/data/openstack-config.yaml 'neutron_settings\.ml2\..*'
```

### 5. **docker-compose.yaml**
Docker Compose configuration for containerized applications. Demonstrates:
- Service definitions
- Environment variables
- Volume configuration
- Network definitions
- Healthcheck configuration

**Example queries:**
```bash
ym grep tests/data/docker-compose.yaml 'services\..*\.environment\.DB_.*'
ym grep tests/data/docker-compose.yaml 'services\..*\.image'
```

### 6. **ci-pipeline.yaml**
GitHub Actions CI/CD workflow. Demonstrates:
- Job definitions
- Environment variables and secrets
- Step configurations
- Conditional execution

**Example queries:**
```bash
ym grep tests/data/ci-pipeline.yaml 'jobs\..*\.steps\[0\]\..*'
ym grep tests/data/ci-pipeline.yaml 'env\..*'
```

### 7. **microservice-config.yaml**
Microservice configuration with auth and features. Demonstrates:
- Authentication settings
- Feature toggles
- Rate limiting
- CORS configuration

**Example queries:**
```bash
ym grep tests/data/microservice-config.yaml 'api\..*\.enabled'
ym grep tests/data/microservice-config.yaml 'features\..*'
```

### 8. **minimal-config.yaml**
Simple configuration file with essential settings. Good for basic testing.

**Example queries:**
```bash
ym grep tests/data/minimal-config.yaml '.*\..*'
ym grep tests/data/minimal-config.yaml 'database\..*'
```

### 9. **helm-values.yaml**
Helm Chart values file. Demonstrates:
- Kubernetes-specific configurations
- Image settings
- Replica counts
- Resource limits
- Autoscaling configuration

**Example queries:**
```bash
ym grep tests/data/helm-values.yaml 'image\..*'
ym grep tests/data/helm-values.yaml 'autoscaling\..*'
ym grep tests/data/helm-values.yaml 'resources\.limits\..*'
```

### 10. **config-dev.yaml**
Development environment configuration. Demonstrates:
- Environment-specific settings
- Localhost services
- Debug enabled
- Test/sandbox APIs

**Example queries:**
```bash
ym grep tests/data/config-dev.yaml 'database\..*\.host'
ym grep tests/data/config-dev.yaml 'external_services\..*\..*'
```

### 11. **config-prod.yaml**
Production environment configuration. Demonstrates:
- Production databases and credentials
- SSL/TLS enabled
- Monitoring integrations
- Backup settings
- Multiple replicas

**Example queries:**
```bash
ym grep tests/data/config-prod.yaml '.*\.password'
ym grep tests/data/config-prod.yaml 'monitoring\..*\..*'
ym grep tests/data/config-prod.yaml 'database\.replica\..*'
```

### 12. **argocd-app.yaml**
ArgoCD Application manifest for GitOps. Demonstrates:
- ArgoCD-specific configuration
- Helm chart deployment settings
- Sync policies
- Retry configuration

**Example queries:**
```bash
ym grep tests/data/argocd-app.yaml 'spec\.source\..*'
ym grep tests/data/argocd-app.yaml 'spec\.syncPolicy\..*'
```

### 13. **terraform-values.yaml**
Terraform configuration values. Demonstrates:
- Infrastructure as Code settings
- AWS configuration
- Kubernetes cluster settings
- Instance and resource sizing

**Example queries:**
```bash
ym grep tests/data/terraform-values.yaml 'aws\..*\..*'
ym grep tests/data/terraform-values.yaml 'kubernetes\.node_groups\..*'
```

### 14. **feature-flags.yaml**
Feature flag configuration. Demonstrates:
- Multiple feature toggle sections
- Boolean values
- Provider selections
- Experimental features

**Example queries:**
```bash
ym grep tests/data/feature-flags.yaml 'features\..*\.enabled'
ym grep tests/data/feature-flags.yaml 'features\..*\.provider'
ym grep tests/data/feature-flags.yaml 'features\.experimental\..*'
```

## Usage Examples

### Search for all passwords
```bash
ym grep tests/data/app-config.yaml '.*\.password'
ym grep tests/data/config-prod.yaml '.*_settings\.db\.password'
```

### Search for host configurations
```bash
ym grep tests/data/app-config.yaml '.*\.host'
ym grep tests/data/config-prod.yaml '.*\.host'
```

### Search for port configurations
```bash
ym grep tests/data/app-config.yaml '.*\.port'
```

### Update values
```bash
ym set tests/data/app-config.yaml database.primary.port=5433
ym set tests/data/config-dev.yaml app.log_level=info
```

### Remove sensitive data
```bash
ym unset tests/data/app-config.yaml database.primary.password
ym unset tests/data/config-prod.yaml external_services.payment.api_key
```

## Recommended Test Scenarios

1. **Multi-level path matching**: Use `openstack-config.yaml` to test regex patterns across similar keys
2. **Complex nesting**: Use `app-config.yaml` to test deep key paths
3. **Kubernetes configs**: Use `k8s-deployment.yaml` and `k8s-service.yaml` for Kubernetes-specific patterns
4. **Bulk operations**: Update multiple values in `config-prod.yaml`
5. **Sensitive data handling**: Search for and redact passwords and API keys
