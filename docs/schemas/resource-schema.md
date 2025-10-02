# Resource Schema

## Overview

The Resource schema defines the structure for human resources in the TaskTaskRevolution system, following the Backstage/Kubernetes manifest pattern.

## API Version

- **apiVersion**: `tasktaskrevolution.io/v1alpha1`
- **kind**: `Resource`

## Schema Structure

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  id?: string
  name: string
  email: string
  code: string
  resourceType: string
  status: string
  description?: string
  createdAt?: string (ISO 8601)
  updatedAt?: string (ISO 8601)
  createdBy?: string
spec:
  startDate?: string (YYYY-MM-DD)
  endDate?: string (YYYY-MM-DD)
  vacations?: Period[]
  projectAssignments?: ProjectAssignment[]
  timeOffBalance: number
  timeOffHistory?: TimeOffEntry[]
  scope: ResourceScope
  projectId?: string
```

## Field Descriptions

### Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | No | Unique identifier for the resource |
| `name` | string | Yes | Resource full name |
| `email` | string | No | Resource email address |
| `code` | string | Yes | Human-readable resource code |
| `resourceType` | string | Yes | Type of resource (e.g., "Developer", "Manager") |
| `status` | string | Yes | Resource status ("Available", "Assigned", "Inactive") |
| `description` | string | No | Resource description |
| `createdAt` | string | No | Creation timestamp (ISO 8601) |
| `updatedAt` | string | No | Last update timestamp (ISO 8601) |
| `createdBy` | string | No | User who created the resource |

### Spec

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `startDate` | string | No | Resource start date (YYYY-MM-DD format) |
| `endDate` | string | No | Resource end date (YYYY-MM-DD format) |
| `vacations` | Period[] | No | List of vacation periods |
| `projectAssignments` | ProjectAssignment[] | No | List of project assignments |
| `timeOffBalance` | number | Yes | Available time off balance in hours |
| `timeOffHistory` | TimeOffEntry[] | No | History of time off entries |
| `scope` | ResourceScope | Yes | Resource scope (Company, Project) |
| `projectId` | string | No | Associated project ID |

## Enums

### ResourceScope

```yaml
enum:
  - Company
  - Project
```

### PeriodType

```yaml
enum:
  - BirthdayBreak
  - DayOff
  - Vacation
  - SickLeave
  - PersonalLeave
  - TimeOffCompensation
  - TimeOff
```

## Complex Types

### Period

```yaml
type: object
properties:
  startDate: string (ISO 8601)
  endDate: string (ISO 8601)
  approved: boolean
  periodType: PeriodType
  isTimeOffCompensation: boolean
  compensatedHours?: number
  isLayoff: boolean
```

### ProjectAssignment

```yaml
type: object
properties:
  projectId: string
  startDate: string (ISO 8601)
  endDate: string (ISO 8601)
  allocationPercentage: number (0-100)
```

### TimeOffEntry

```yaml
type: object
properties:
  id: string
  type: string
  hours: number
  date: string (YYYY-MM-DD)
  description?: string
```

## Example

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  id: "0199598c-f574-7fdc-8887-354407d0e74b"
  name: "John Doe"
  email: "john.doe@techcorp.com"
  code: "DEV-001"
  resourceType: "Senior Developer"
  status: "Assigned"
  description: "Full-stack developer with 5 years experience"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-15T10:30:00Z"
  createdBy: "hr@techcorp.com"
spec:
  startDate: "2024-01-01"
  endDate: "2024-12-31"
  timeOffBalance: 160
  scope: Company
  projectAssignments:
    - projectId: "WEB-APP-001"
      startDate: "2024-01-15T00:00:00Z"
      endDate: "2024-06-30T23:59:59Z"
      allocationPercentage: 100
  vacations:
    - startDate: "2024-07-15T00:00:00Z"
      endDate: "2024-07-22T23:59:59Z"
      approved: true
      periodType: Vacation
      isTimeOffCompensation: false
      isLayoff: false
  timeOffHistory:
    - id: "timeoff-001"
      type: "Vacation"
      hours: 40
      date: "2024-07-15"
      description: "Summer vacation"
```

## Validation Rules

- `name` must not be empty
- `code` must be unique across all resources
- `email` must be a valid email format (if provided)
- `resourceType` must not be empty
- `status` must be one of: "Available", "Assigned", "Inactive"
- `startDate` must be before `endDate` (if both provided)
- `startDate` and `endDate` must be valid dates in YYYY-MM-DD format
- `timeOffBalance` must be a non-negative number
- `allocationPercentage` must be between 0 and 100
- `scope` must be either "Company" or "Project"
- `projectId` must reference an existing project (if provided)
