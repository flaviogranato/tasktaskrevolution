# TaskTaskRevolution Documentation

Welcome to the TaskTaskRevolution documentation. This directory contains comprehensive documentation for the TaskTaskRevolution project management system.

## 📚 Documentation Overview

### Core Documentation
- **[API Version Policy](./api-version-policy.md)** - Versioning policy and compatibility guidelines
- **[Migration Guide](./migration-guide.md)** - Step-by-step migration instructions
- **[Schema Documentation](./schemas/)** - Complete schema reference for all manifest types

### Schema Documentation
- **[Company Schema](./schemas/company-schema.md)** - Company manifest specification
- **[Project Schema](./schemas/project-schema.md)** - Project manifest specification  
- **[Resource Schema](./schemas/resource-schema.md)** - Resource manifest specification
- **[Task Schema](./schemas/task-schema.md)** - Task manifest specification
- **[Config Schema](./schemas/config-schema.md)** - Configuration manifest specification

## 🚀 Quick Start

### Understanding API Versions
TaskTaskRevolution follows a Kubernetes/Backstage-inspired versioning scheme:

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1  # Current version
kind: Company
metadata:
  name: "tech-corp"
  code: "TECH-001"
spec:
  # ... specification
```

### Current Version Status
- **v1alpha1**: ✅ Active (Current)
- **v1beta1**: 🔄 Planned (Q2 2024)
- **v1stable**: ⏳ Future (Q4 2024)

## 📖 Key Concepts

### Manifest Types
TaskTaskRevolution uses YAML manifests to define entities:

1. **Company** - Organizational units
2. **Project** - Work initiatives within companies
3. **Resource** - People and assets
4. **Task** - Work items within projects
5. **Config** - System configuration

### Versioning Policy
- **Breaking Changes**: Increment major version
- **New Features**: Increment minor version
- **Bug Fixes**: Increment patch version
- **Deprecation**: 6-month notice for alpha, 12-month for beta

### Migration Strategy
- **Automated Tools**: `ttr migrate` command
- **Manual Migration**: For complex changes
- **Validation**: Built-in validation and testing
- **Rollback**: Support for reverting changes

## 🛠️ Common Tasks

### Check Current Versions
```bash
# Check API versions in workspace
ttr validate system --check-versions

# List all manifests with versions
find . -name "*.yaml" -exec grep -l "apiVersion:" {} \; | xargs grep -H "apiVersion:"
```

### Migrate Manifests
```bash
# Migrate all manifests
ttr migrate --from v1alpha1 --to v1beta1

# Dry run to preview changes
ttr migrate --from v1alpha1 --to v1beta1 --dry-run

# Migrate specific type
ttr migrate --from v1alpha1 --to v1beta1 --kind Project
```

### Validate Manifests
```bash
# Validate entire system
ttr validate system

# Validate specific manifest
ttr validate manifest company.yaml

# Fix validation issues
ttr validate system --fix
```

## 📋 Migration Checklist

### Before Migration
- [ ] Backup all data
- [ ] Review breaking changes
- [ ] Update tools to target version
- [ ] Test migration in staging

### During Migration
- [ ] Run dry-run migration
- [ ] Execute actual migration
- [ ] Apply manual changes
- [ ] Validate each step

### After Migration
- [ ] Run full system validation
- [ ] Test all functionality
- [ ] Update documentation
- [ ] Train team members

## 🔧 Troubleshooting

### Common Issues
1. **Version Mismatch**: Ensure all manifests use same API version
2. **Validation Errors**: Run validation to identify schema issues
3. **Migration Failures**: Check for custom fields needing manual migration
4. **Tool Compatibility**: Update tools to support new API versions

### Getting Help
- **Documentation**: Check this guide and schema documentation
- **Migration Guide**: See specific migration guides for each version
- **Community**: Ask questions in community forums
- **Issues**: Report bugs via GitHub issues

## 📚 Additional Resources

### External References
- [Kubernetes API Versioning](https://kubernetes.io/docs/reference/using-api/api-concepts/#api-versioning)
- [Backstage API Versioning](https://backstage.io/docs/features/software-catalog/system-model)
- [Semantic Versioning](https://semver.org/)

### Internal Resources
- [Schema Documentation](./schemas/)
- [API Version Policy](./api-version-policy.md)
- [Migration Guide](./migration-guide.md)

## 🤝 Contributing

### Documentation Updates
1. Follow the established format and style
2. Include examples for new features
3. Update migration guides for breaking changes
4. Test all code examples

### Schema Changes
1. Update schema documentation
2. Add migration instructions
3. Update version compatibility matrix
4. Test with real data

## 📝 Changelog

### v1alpha1 (Current)
- Initial API version
- Basic manifest types
- Core functionality

### Planned Changes
- **v1beta1**: Enhanced validation, automated migrations
- **v1stable**: Production stability, long-term support
- **v2alpha1**: Next major version with new features

---

For more detailed information, see the individual documentation files in this directory.
