# API Version Policy and Compatibility

This document defines the versioning policy for TaskTaskRevolution manifests and how to manage breaking changes.

## Overview

TaskTaskRevolution follows a Kubernetes/Backstage-inspired versioning scheme for all manifest types. This ensures consistency across the platform and provides clear migration paths for breaking changes.

## Versioning Scheme

### Current Version
All manifests currently use:
```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
```

### Version Format
The version follows the pattern: `tasktaskrevolution.io/v{major}{stability}{minor}`

- **Major**: Incremented for breaking changes
- **Stability**: 
  - `alpha` - Experimental features, may have breaking changes
  - `beta` - Stable features, backward compatible within major version
  - `stable` - Production-ready, fully stable
- **Minor**: Incremented for backward-compatible additions

### Version Lifecycle

#### v1alpha1 (Current)
- **Status**: Active
- **Stability**: Alpha
- **Breaking Changes**: Allowed
- **Deprecation Notice**: 6 months before removal
- **Migration Path**: Manual migration required

#### v1beta1 (Planned)
- **Status**: Planned
- **Stability**: Beta
- **Breaking Changes**: Not allowed within major version
- **Deprecation Notice**: 12 months before removal
- **Migration Path**: Automated migration tools provided

#### v1stable (Future)
- **Status**: Future
- **Stability**: Stable
- **Breaking Changes**: Not allowed within major version
- **Deprecation Notice**: 24 months before removal
- **Migration Path**: Automated migration tools provided

## Manifest Types

All manifest types follow the same versioning policy:

### Company Manifest
```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  name: "tech-corp"
  code: "TECH-001"
spec:
  # ... company specification
```

### Project Manifest
```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  name: "web-application"
  code: "WEB-APP"
  companyCode: "TECH-001"
spec:
  # ... project specification
```

### Resource Manifest
```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  name: "john-doe"
  code: "DEV-001"
  resourceType: "Developer"
spec:
  # ... resource specification
```

### Task Manifest
```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  name: "login-implementation"
  code: "TASK-001"
  projectCode: "WEB-APP"
spec:
  # ... task specification
```

### Config Manifest
```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  name: "default"
spec:
  # ... configuration specification
```

## Breaking Changes Policy

### What Constitutes a Breaking Change

1. **Field Removal**: Removing any field from spec or metadata
2. **Field Type Changes**: Changing the type of an existing field
3. **Required Field Addition**: Making an optional field required
4. **Enum Value Removal**: Removing values from enum fields
5. **Default Value Changes**: Changing default values for fields
6. **Validation Rule Changes**: Making validation rules stricter

### What Does NOT Constitute a Breaking Change

1. **New Field Addition**: Adding new optional fields
2. **New Enum Values**: Adding new values to existing enums
3. **New Validation Rules**: Adding new optional validation rules
4. **Documentation Updates**: Updating field descriptions or examples
5. **Default Value Addition**: Adding default values for new fields

## Migration Strategy

### Automated Migration Tools

TaskTaskRevolution provides migration tools for version upgrades:

```bash
# Migrate all manifests to a new version
ttr migrate --from v1alpha1 --to v1beta1

# Migrate specific manifest type
ttr migrate --from v1alpha1 --to v1beta1 --kind Project

# Dry run to see what would be migrated
ttr migrate --from v1alpha1 --to v1beta1 --dry-run

# Migrate specific file
ttr migrate --from v1alpha1 --to v1beta1 --file company.yaml
```

### Manual Migration

For complex breaking changes, manual migration may be required:

1. **Review Breaking Changes**: Check the changelog for breaking changes
2. **Update Manifests**: Modify manifests according to new schema
3. **Validate Changes**: Run validation to ensure correctness
4. **Test Migration**: Test with a copy of your data first

### Migration Examples

#### Example 1: Field Rename (Breaking Change)
```yaml
# v1alpha1
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
spec:
  projectName: "Web App"  # Old field name

# v1beta1
apiVersion: tasktaskrevolution.io/v1beta1
kind: Project
spec:
  name: "Web App"  # New field name
```

#### Example 2: New Optional Field (Non-Breaking)
```yaml
# v1alpha1
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
spec:
  name: "Web App"

# v1beta1 (backward compatible)
apiVersion: tasktaskrevolution.io/v1beta1
kind: Project
spec:
  name: "Web App"
  description: "A web application project"  # New optional field
```

## Compatibility Matrix

| Manifest Type | v1alpha1 | v1beta1 | v1stable |
|---------------|----------|---------|----------|
| Company       | ✅       | 🔄      | ⏳       |
| Project       | ✅       | 🔄      | ⏳       |
| Resource      | ✅       | 🔄      | ⏳       |
| Task          | ✅       | 🔄      | ⏳       |
| Config        | ✅       | 🔄      | ⏳       |

Legend:
- ✅ Supported
- 🔄 Planned
- ⏳ Future

## Best Practices

### For Manifest Authors

1. **Use Latest Version**: Always use the latest stable version when creating new manifests
2. **Validate Before Deploy**: Run validation before committing manifests
3. **Test Migrations**: Test migration scripts with copies of production data
4. **Document Dependencies**: Document any dependencies on specific versions

### For Tool Developers

1. **Support Multiple Versions**: Tools should support multiple API versions
2. **Graceful Degradation**: Handle unknown fields gracefully
3. **Clear Error Messages**: Provide clear error messages for version mismatches
4. **Migration Hints**: Suggest migration paths when encountering old versions

### For CI/CD Pipelines

1. **Version Validation**: Validate API versions in CI pipelines
2. **Migration Checks**: Check for available migrations
3. **Compatibility Testing**: Test with multiple API versions
4. **Automated Migration**: Automate migration where possible

## Deprecation Timeline

### v1alpha1 Deprecation (Example)

| Phase | Timeline | Action Required |
|-------|----------|-----------------|
| Announcement | T-6 months | Plan migration to v1beta1 |
| Soft Deprecation | T-3 months | Start migration, warnings shown |
| Hard Deprecation | T-1 month | Migration required, errors shown |
| Removal | T-0 | v1alpha1 no longer supported |

## Troubleshooting

### Common Issues

1. **Version Mismatch**: Ensure all manifests use the same API version
2. **Migration Failures**: Check for custom fields that need manual migration
3. **Validation Errors**: Run validation to identify schema issues
4. **Tool Compatibility**: Update tools to support new API versions

### Getting Help

- **Documentation**: Check this document and schema documentation
- **Migration Guide**: See specific migration guides for each version
- **Community**: Ask questions in the community forums
- **Issues**: Report bugs and request features via GitHub issues

## Future Roadmap

### Planned Versions

- **v1beta1**: Q2 2024 - Beta stability, automated migrations
- **v1stable**: Q4 2024 - Production stability, long-term support
- **v2alpha1**: Q2 2025 - Next major version with new features

### Feature Roadmap

- **Automated Migration**: AI-powered migration suggestions
- **Version Compatibility**: Automatic version detection and conversion
- **Schema Evolution**: Backward-compatible schema evolution
- **Validation Rules**: Enhanced validation with better error messages

## References

- [Kubernetes API Versioning](https://kubernetes.io/docs/reference/using-api/api-concepts/#api-versioning)
- [Backstage API Versioning](https://backstage.io/docs/features/software-catalog/system-model)
- [Semantic Versioning](https://semver.org/)
- [TaskTaskRevolution Schema Documentation](./schemas/)
