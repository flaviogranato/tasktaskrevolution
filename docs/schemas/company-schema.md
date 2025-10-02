# Company Schema

## Overview

The Company schema defines the structure for company entities in the TaskTaskRevolution system, following the Backstage/Kubernetes manifest pattern.

## API Version

- **apiVersion**: `tasktaskrevolution.io/v1alpha1`
- **kind**: `Company`

## Schema Structure

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: string
  code: string
  name: string
  createdAt: string (ISO 8601)
  updatedAt: string (ISO 8601)
  createdBy: string
spec:
  description?: string
  taxId?: string
  address?: string
  email?: string
  phone?: string
  website?: string
  industry?: string
  size: CompanySize
  status: CompanyStatus
```

## Field Descriptions

### Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier for the company |
| `code` | string | Yes | Human-readable company code |
| `name` | string | Yes | Company name |
| `createdAt` | string | Yes | Creation timestamp (ISO 8601) |
| `updatedAt` | string | Yes | Last update timestamp (ISO 8601) |
| `createdBy` | string | Yes | User who created the company |

### Spec

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | string | No | Company description |
| `taxId` | string | No | Tax identification number |
| `address` | string | No | Company address |
| `email` | string | No | Company email |
| `phone` | string | No | Company phone number |
| `website` | string | No | Company website URL |
| `industry` | string | No | Industry sector |
| `size` | CompanySize | Yes | Company size classification |
| `status` | CompanyStatus | Yes | Company status |

## Enums

### CompanySize

```yaml
enum:
  - Small
  - Medium
  - Large
```

### CompanyStatus

```yaml
enum:
  - Active
  - Inactive
  - Suspended
```

## Example

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "0199598c-f574-7fdc-8887-354407d0e74b"
  code: "TECH-CORP"
  name: "Tech Corporation"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "admin@techcorp.com"
spec:
  description: "A technology company focused on innovation"
  taxId: "12.345.678/0001-90"
  address: "123 Tech Street, Silicon Valley, CA 94000"
  email: "contact@techcorp.com"
  phone: "+1-555-0123"
  website: "https://techcorp.com"
  industry: "Technology"
  size: Medium
  status: Active
```

## Validation Rules

- `code` must be unique across all companies
- `name` must not be empty
- `email` must be a valid email format (if provided)
- `website` must be a valid URL (if provided)
- `taxId` must follow the specified format (if provided)
- `createdAt` and `updatedAt` must be valid ISO 8601 timestamps
