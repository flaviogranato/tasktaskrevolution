# API Versioning Policy

## Overview

This document defines the API versioning policy for TaskTaskRevolution manifests and establishes guidelines for version management, compatibility, and migration strategies.

## Current API Version

**Current Version**: `tasktaskrevolution.io/v1alpha1`

## Version Format

API versions follow the format: `{domain}/{version}`

- **Domain**: `tasktaskrevolution.io` - Identifies the API domain
- **Version**: Semantic versioning with stability indicators

## Version Stability Levels

### Alpha (v1alpha1, v1alpha2, ...)
- **Purpose**: Experimental features and breaking changes
- **Stability**: Unstable, breaking changes allowed
- **Support**: Limited support, may be deprecated without notice
- **Use Case**: Development, testing, early adoption

### Beta (v1beta1, v1beta2, ...)
- **Purpose**: Feature-complete but not yet stable
- **Stability**: Mostly stable, minor breaking changes allowed
- **Support**: Good support, deprecation with advance notice
- **Use Case**: Pre-production, pilot programs

### Stable (v1, v2, ...)
- **Purpose**: Production-ready, stable API
- **Stability**: Stable, no breaking changes
- **Support**: Full support, long-term compatibility
- **Use Case**: Production environments

## Version Lifecycle

### 1. Alpha Phase
- New features introduced
- Breaking changes allowed
- Limited documentation
- Experimental status

### 2. Beta Phase
- Features stabilized
- Minor breaking changes allowed
- Comprehensive documentation
- Pre-production status

### 3. Stable Phase
- No breaking changes
- Full backward compatibility
- Production-ready
- Long-term support

## Breaking Changes Policy

### What Constitutes Breaking Changes

1. **Removing fields** from manifests
2. **Changing field types** (e.g., string to integer)
3. **Changing required fields** to optional or vice versa
4. **Removing enum values**
5. **Changing default values** that affect behavior
6. **Removing API endpoints** or commands

### What Does NOT Constitute Breaking Changes

1. **Adding new fields** (optional)
2. **Adding new enum values**
3. **Adding new API endpoints**
4. **Improving error messages**
5. **Adding new validation rules** (non-breaking)
6. **Performance improvements**

## Migration Strategy

### Alpha to Beta Migration

```yaml
# Before (Alpha)
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "123"
  code: "COMP-001"
  name: "Company Name"
spec:
  description: "Company description"

# After (Beta)
apiVersion: tasktaskrevolution.io/v1beta1
kind: Company
metadata:
  id: "123"
  code: "COMP-001"
  name: "Company Name"
  labels: {}  # New field
  annotations: {}  # New field
  namespace: "default"  # New field
spec:
  description: "Company description"
  # Additional fields may be added
```

### Beta to Stable Migration

```yaml
# Before (Beta)
apiVersion: tasktaskrevolution.io/v1beta1
kind: Company
metadata:
  id: "123"
  code: "COMP-001"
  name: "Company Name"
  labels:
    environment: "production"
  annotations:
    description: "Production company"
  namespace: "default"
spec:
  description: "Company description"

# After (Stable)
apiVersion: tasktaskrevolution.io/v1
kind: Company
metadata:
  id: "123"
  code: "COMP-001"
  name: "Company Name"
  labels:
    environment: "production"
  annotations:
    description: "Production company"
  namespace: "default"
spec:
  description: "Company description"
  # Fields are now stable and guaranteed
```

## Version Support Matrix

| Version | Status | Support Until | Breaking Changes |
|---------|--------|---------------|------------------|
| v1alpha1 | Current | TBD | Allowed |
| v1beta1 | Planned | TBD | Minor only |
| v1 | Planned | Long-term | None |

## Migration Tools

### Automatic Migration

The system provides migration tools to help users upgrade their manifests:

```bash
# Migrate all manifests to latest version
ttr migrate manifests

# Migrate specific manifest
ttr migrate manifests --file company.yaml

# Force migration (overwrite existing files)
ttr migrate manifests --force
```

### Manual Migration

For complex migrations, manual intervention may be required:

1. **Backup existing manifests**
2. **Review breaking changes**
3. **Update field mappings**
4. **Test migrated manifests**
5. **Deploy updated manifests**

## Best Practices

### For Users

1. **Pin to stable versions** in production
2. **Test alpha/beta versions** in development
3. **Monitor deprecation notices**
4. **Plan migration windows**
5. **Keep backups** before migrations

### For Developers

1. **Follow semantic versioning**
2. **Document breaking changes**
3. **Provide migration guides**
4. **Maintain backward compatibility** when possible
5. **Give advance notice** for deprecations

## Deprecation Policy

### Deprecation Timeline

1. **Announcement**: 6 months before deprecation
2. **Warning Period**: 3 months with warnings
3. **Deprecation**: Version marked as deprecated
4. **Removal**: Version removed after 12 months

### Deprecation Notices

```yaml
# Example deprecation notice
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  # ... existing fields ...
  annotations:
    deprecated: "true"
    deprecation-warning: "This version will be removed in v2.0.0"
    migration-guide: "https://docs.tasktaskrevolution.io/migration/v1alpha1-to-v1beta1"
```

## Version Validation

### Client-Side Validation

```rust
// Example validation logic
fn validate_api_version(version: &str) -> Result<(), ValidationError> {
    match version {
        "tasktaskrevolution.io/v1alpha1" => Ok(()),
        "tasktaskrevolution.io/v1beta1" => Ok(()),
        "tasktaskrevolution.io/v1" => Ok(()),
        _ => Err(ValidationError::UnsupportedVersion(version.to_string()))
    }
}
```

### Server-Side Validation

```yaml
# Example server validation
apiVersion: tasktaskrevolution.io/v1alpha1  # Must be supported version
kind: Company  # Must be valid kind
metadata:
  # Required fields must be present
  id: "required"
  code: "required"
  name: "required"
```

## Compatibility Matrix

| Client Version | Server Version | Compatibility |
|----------------|----------------|---------------|
| v1alpha1 | v1alpha1 | ✅ Full |
| v1alpha1 | v1beta1 | ⚠️ Limited |
| v1alpha1 | v1 | ❌ Incompatible |
| v1beta1 | v1alpha1 | ⚠️ Limited |
| v1beta1 | v1beta1 | ✅ Full |
| v1beta1 | v1 | ✅ Full |
| v1 | v1alpha1 | ❌ Incompatible |
| v1 | v1beta1 | ⚠️ Limited |
| v1 | v1 | ✅ Full |

## Future Roadmap

### v1beta1 (Planned)
- Enhanced validation rules
- Improved error messages
- Additional metadata fields
- Better performance

### v1 (Planned)
- Stable API surface
- Long-term support
- Production-ready features
- Full backward compatibility

### v2 (Future)
- Major architectural changes
- New features
- Breaking changes allowed
- Migration tools provided

## References

- [Kubernetes API Versioning](https://kubernetes.io/docs/reference/using-api/api-concepts/#api-versioning)
- [Semantic Versioning](https://semver.org/)
- [API Design Best Practices](https://docs.microsoft.com/en-us/azure/architecture/best-practices/api-design)

## Support

For questions about API versioning:

- **Documentation**: [docs.tasktaskrevolution.io](https://docs.tasktaskrevolution.io)
- **Issues**: [GitHub Issues](https://github.com/tasktaskrevolution/issues)
- **Discussions**: [GitHub Discussions](https://github.com/tasktaskrevolution/discussions)
- **Email**: support@tasktaskrevolution.io
