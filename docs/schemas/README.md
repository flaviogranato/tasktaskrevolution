# TaskTaskRevolution Schema Documentation

This directory contains the complete schema documentation for all TaskTaskRevolution manifests, following the Backstage/Kubernetes pattern.

## Overview

All manifests in TaskTaskRevolution follow the standard Kubernetes/Backstage manifest structure:

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: <EntityType>
metadata:
  # Entity identification and metadata
spec:
  # Entity-specific data
```

## Available Schemas

### Core Entities

- **[Company Schema](company-schema.md)** - Company entities and organizational structure
- **[Project Schema](project-schema.md)** - Project definitions and management
- **[Resource Schema](resource-schema.md)** - Human resources and team members
- **[Task Schema](task-schema.md)** - Individual tasks and work items

### System Configuration

- **[Config Schema](config-schema.md)** - System configuration and settings

## Common Patterns

### API Version

All manifests use the API version: `tasktaskrevolution.io/v1alpha1`

### Field Naming

All fields use `camelCase` naming convention, following Kubernetes standards.

### Optional Fields

Optional fields are marked with `?` in the schema documentation and use `skip_serializing_if` in the Rust code to avoid serializing empty values.

### Date Formats

- **ISO 8601 timestamps**: Used for `createdAt`, `updatedAt` fields
- **YYYY-MM-DD dates**: Used for date-only fields like `startDate`, `endDate`

### Validation

Each schema includes validation rules to ensure data integrity and consistency across the system.

## Usage Examples

### Creating a Company

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  code: "TECH-CORP"
  name: "Tech Corporation"
  createdBy: "admin@techcorp.com"
spec:
  size: Medium
  status: Active
```

### Creating a Project

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "WEB-APP-001"
  name: "E-commerce Platform"
  companyCode: "TECH-CORP"
spec:
  status: Planned
  timezone: "America/New_York"
```

### Creating a Resource

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  code: "DEV-001"
  name: "John Doe"
  email: "john@techcorp.com"
  resourceType: "Developer"
  status: "Available"
spec:
  timeOffBalance: 160
  scope: Company
```

### Creating a Task

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  code: "TASK-001"
  name: "Implement Authentication"
spec:
  projectCode: "WEB-APP-001"
  assignee: "DEV-001"
  status: Planned
  priority: High
  effort:
    estimatedHours: 40.0
```

## Implementation Details

### Rust Serialization

All manifests are implemented in Rust using `serde` with the following patterns:

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: EntityMetadata,
    pub spec: EntitySpec,
}
```

### Field Serialization

Optional fields use `skip_serializing_if` to avoid serializing empty values:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub field: Option<String>,
```

### Conversion

Each manifest implements conversion methods to/from domain entities:

- `From<&DomainEntity> for Manifest` - Convert domain entity to manifest
- `TryFrom<Manifest> for DomainEntity` - Convert manifest to domain entity

## Testing

Each schema includes comprehensive tests for:

- Serialization/deserialization
- Bidirectional conversion
- Validation rules
- Edge cases and error handling

## Versioning

The current API version is `v1alpha1`. Future versions will maintain backward compatibility where possible, with breaking changes clearly documented in migration guides.

## Contributing

When adding new fields or modifying existing schemas:

1. Update the relevant schema documentation
2. Add appropriate validation rules
3. Update the Rust implementation
4. Add comprehensive tests
5. Update this README if needed
