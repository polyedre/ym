# Test Data Manifest

## Directory Structure

```
tests/data/
├── YAML Files (14)
│   ├── Basic Configs
│   │   ├── minimal-config.yaml              182 bytes
│   │   ├── k8s-service.yaml                 364 bytes
│   │   └── microservice-config.yaml         681 bytes
│   ├── Multi-Service
│   │   ├── app-config.yaml                  1.1 KB
│   │   ├── config-dev.yaml                  623 bytes
│   │   ├── config-prod.yaml                 1.2 KB
│   │   └── openstack-config.yaml            1.3 KB
│   ├── Container & Orchestration
│   │   ├── docker-compose.yaml              1.4 KB
│   │   ├── k8s-deployment.yaml              1.1 KB
│   │   └── helm-values.yaml                 1.7 KB
│   ├── Infrastructure as Code
│   │   ├── terraform-values.yaml            1.3 KB
│   │   ├── argocd-app.yaml                  759 bytes
│   │   └── ci-pipeline.yaml                 1.8 KB
│   └── Feature Management
│       └── feature-flags.yaml               1.1 KB
│
├── Documentation (5)
│   ├── INDEX.md                             Complete guide
│   ├── README.md                            File descriptions
│   ├── EXAMPLES.md                          Usage patterns
│   ├── TEST_SHOWCASE.md                     Live outputs
│   ├── QUICK_START.txt                      Quick reference
│   └── MANIFEST.md                          This file
│
└── Total: 19 files, 92 KB
```

## File Manifest

### YAML Configuration Files

| File | Size | Keys | Purpose |
|------|------|------|---------|
| minimal-config.yaml | 182B | 6 | Simplest config for learning |
| k8s-service.yaml | 364B | 15+ | Kubernetes Service manifest |
| microservice-config.yaml | 681B | 25+ | Single service with auth/features |
| config-dev.yaml | 623B | 30+ | Development environment |
| config-prod.yaml | 1.2K | 40+ | Production environment |
| app-config.yaml | 1.1K | 50+ | Multi-service app config |
| openstack-config.yaml | 1.3K | 70+ | 4 OpenStack services |
| docker-compose.yaml | 1.4K | 50+ | Multi-container setup |
| k8s-deployment.yaml | 1.1K | 40+ | Full Kubernetes Deployment |
| helm-values.yaml | 1.7K | 60+ | Helm Chart values |
| terraform-values.yaml | 1.3K | 50+ | Infrastructure code |
| argocd-app.yaml | 759B | 25+ | GitOps application |
| ci-pipeline.yaml | 1.8K | 45+ | GitHub Actions workflow |
| feature-flags.yaml | 1.1K | 45+ | Feature toggles |

### Documentation Files

| File | Purpose | Content |
|------|---------|---------|
| **INDEX.md** | Navigation & Learning Path | Complete guide with 4-day learning path |
| **README.md** | File Descriptions | Detailed overview of each YAML file |
| **EXAMPLES.md** | Usage Patterns | 50+ practical examples |
| **TEST_SHOWCASE.md** | Live Examples | Actual command outputs |
| **QUICK_START.txt** | Quick Reference | One-page quick start |
| **MANIFEST.md** | This File | Complete file inventory |

## Content Breakdown

### YAML Files by Type

**Configuration Files:** 6
- minimal-config.yaml, microservice-config.yaml, app-config.yaml
- config-dev.yaml, config-prod.yaml, feature-flags.yaml

**Infrastructure Files:** 5
- k8s-service.yaml, k8s-deployment.yaml, docker-compose.yaml
- helm-values.yaml, terraform-values.yaml

**Service Files:** 3
- openstack-config.yaml, argocd-app.yaml, ci-pipeline.yaml

### Key Statistics

| Metric | Count |
|--------|-------|
| Total Files | 19 |
| YAML Files | 14 |
| Documentation | 5 |
| Total Size | 92 KB |
| Total Keys | 500+ |
| Max Nesting Depth | 5 levels |
| Nesting Levels | 3-5 |
| Examples Documented | 50+ |
| Regex Patterns | 20+ |
| Real-World Scenarios | 10+ |

### Nesting Depth Distribution

- 3 levels: minimal-config.yaml, k8s-service.yaml
- 4 levels: microservice-config.yaml, config-dev.yaml, config-prod.yaml, feature-flags.yaml, argocd-app.yaml
- 5 levels: app-config.yaml, openstack-config.yaml, docker-compose.yaml, k8s-deployment.yaml, helm-values.yaml, terraform-values.yaml, ci-pipeline.yaml

### Value Types Demonstrated

- **Strings**: Hostnames, URLs, passwords, paths
- **Numbers**: Ports, replica counts, timeouts, pool sizes
- **Booleans**: Flags, toggles, enable/disable settings
- **Lists**: Arrays of items, multiple values
- **Objects**: Nested structures, complex configurations
- **Null**: Optional/missing values

## Use Case Coverage

### DevOps/SRE
- OpenStack configuration management
- Kubernetes manifests (Service, Deployment)
- Production vs Development environments
- Multi-service orchestration

### Platform Engineering
- Feature flag management
- Helm chart deployment
- Docker Compose setups
- Service-level configuration

### Infrastructure as Code
- Terraform configurations
- ArgoCD deployments
- CI/CD pipelines
- GitOps workflows

### Security
- Password/secret discovery
- API key management
- SSL/TLS verification
- Access control settings

### Development
- Microservice configuration
- Debug/log settings
- Feature toggles
- Multi-environment setup

## Patterns Tested

### Search Patterns
- ✓ Exact key matching
- ✓ Wildcard patterns (`..*`)
- ✓ Partial path matching
- ✓ Service discovery patterns
- ✓ Value-based matching

### Update Patterns
- ✓ Single key updates
- ✓ Multi-key updates (bulk)
- ✓ Nested path creation
- ✓ Deep nesting (5 levels)

### Deletion Patterns
- ✓ Single key deletion
- ✓ Nested section deletion
- ✓ Sensitive data removal

## Documentation Quality

### Coverage
- 100% of files documented
- 50+ practical examples
- 20+ regex patterns shown
- 10+ real-world scenarios
- 4-day learning path

### Accessibility
- Quick start guide included
- Complete navigation index
- Beginner to advanced examples
- Troubleshooting tips
- Integration patterns

## Recommended Usage

### For Testing
1. Start with: minimal-config.yaml
2. Progress to: microservice-config.yaml, app-config.yaml
3. Advance to: openstack-config.yaml, helm-values.yaml

### For Learning
1. Read: INDEX.md (overview)
2. Review: README.md (file descriptions)
3. Study: EXAMPLES.md (patterns)
4. Reference: TEST_SHOWCASE.md (outputs)

### For Development
1. Use as templates for real configs
2. Adapt patterns to your needs
3. Create custom test files
4. Build automation scripts

## Version Information

- Created: 2026-02-25
- Format: YAML 1.1
- Encoding: UTF-8
- Total Lines: 1000+
- Tool: ym CLI

## File Dependencies

None - all files are independent and can be used standalone.

## Compatibility

- ✓ Compatible with standard YAML parsers
- ✓ No vendor-specific extensions
- ✓ Pure YAML syntax
- ✓ No embedded scripts or code

## Maintenance

All files are designed to be:
- Easy to understand
- Simple to modify
- Reusable as templates
- Extendable for custom scenarios

## License Information

Test data created for the ym tool project.
Free to use and modify for testing and learning purposes.

---

**Last Updated:** 2026-02-25
**Total Files:** 19
**Status:** Complete and Ready for Use
