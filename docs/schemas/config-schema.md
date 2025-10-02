# Config Schema

## Overview

The Config schema defines the structure for system configuration in the TaskTaskRevolution system, following the Backstage/Kubernetes manifest pattern.

## API Version

- **apiVersion**: `tasktaskrevolution.io/v1alpha1`
- **kind**: `Config`

## Schema Structure

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  id?: string
  code?: string
  name?: string
  description?: string
  createdAt: string (ISO 8601)
  updatedAt?: string (ISO 8601)
  createdBy?: string
spec:
  managerName: string
  managerEmail: string
  defaultTimezone: string
  companyName?: string
  workHoursStart?: string
  workHoursEnd?: string
  currency?: string
  workHoursPerDay?: number
  workDaysPerWeek?: string[]
  dateFormat?: string
  defaultTaskDuration?: number
  locale?: string
  vacationRules?: VacationRules
  resourceTypes?: string[]
```

## Field Descriptions

### Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | No | Unique identifier for the config |
| `code` | string | No | Configuration code |
| `name` | string | No | Configuration name |
| `description` | string | No | Configuration description |
| `createdAt` | string | Yes | Creation timestamp (ISO 8601) |
| `updatedAt` | string | No | Last update timestamp (ISO 8601) |
| `createdBy` | string | No | User who created the config |

### Spec

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `managerName` | string | Yes | Name of the project manager |
| `managerEmail` | string | Yes | Email of the project manager |
| `defaultTimezone` | string | Yes | Default timezone for the system |
| `companyName` | string | No | Company name |
| `workHoursStart` | string | No | Work hours start time (HH:MM format) |
| `workHoursEnd` | string | No | Work hours end time (HH:MM format) |
| `currency` | string | No | Default currency code |
| `workHoursPerDay` | number | No | Number of work hours per day |
| `workDaysPerWeek` | string[] | No | List of work days |
| `dateFormat` | string | No | Date format pattern |
| `defaultTaskDuration` | number | No | Default task duration in days |
| `locale` | string | No | System locale |
| `vacationRules` | VacationRules | No | Vacation rules configuration |
| `resourceTypes` | string[] | No | Available resource types |

## Complex Types

### VacationRules

```yaml
type: object
properties:
  maxConcurrentVacations?: number
  carryOverDays?: number
  allowLayoffVacations?: boolean
  requireLayoffVacationPeriod?: boolean
  layoffPeriods?: LayoffPeriod[]
```

### LayoffPeriod

```yaml
type: object
properties:
  startDate: string (YYYY-MM-DD)
  endDate: string (YYYY-MM-DD)
```

## Work Days

Valid work day values:
- `Monday`
- `Tuesday`
- `Wednesday`
- `Thursday`
- `Friday`
- `Saturday`
- `Sunday`

## Example

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  id: "0199598c-f574-7fdc-8887-354407d0e74b"
  code: "DEFAULT"
  name: "Default Configuration"
  description: "Default system configuration for TaskTaskRevolution"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-15T10:30:00Z"
  createdBy: "admin@techcorp.com"
spec:
  managerName: "John Smith"
  managerEmail: "john.smith@techcorp.com"
  defaultTimezone: "America/New_York"
  companyName: "Tech Corporation"
  workHoursStart: "09:00"
  workHoursEnd: "17:00"
  currency: "USD"
  workHoursPerDay: 8
  workDaysPerWeek:
    - "Monday"
    - "Tuesday"
    - "Wednesday"
    - "Thursday"
    - "Friday"
  dateFormat: "YYYY-MM-DD"
  defaultTaskDuration: 5
  locale: "en-US"
  vacationRules:
    maxConcurrentVacations: 20
    carryOverDays: 5
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
    layoffPeriods:
      - startDate: "2024-12-20"
        endDate: "2025-01-05"
  resourceTypes:
    - "Developer"
    - "Designer"
    - "Manager"
    - "QA Engineer"
    - "DevOps Engineer"
```

## Validation Rules

- `managerName` must not be empty
- `managerEmail` must be a valid email format
- `defaultTimezone` must be a valid timezone identifier
- `workHoursStart` and `workHoursEnd` must be in HH:MM format (if provided)
- `workHoursStart` must be before `workHoursEnd` (if both provided)
- `workHoursPerDay` must be between 1 and 24 (if provided)
- `workDaysPerWeek` must contain valid day names (if provided)
- `defaultTaskDuration` must be a positive number (if provided)
- `currency` must be a valid ISO 4217 currency code (if provided)
- `locale` must be a valid locale identifier (if provided)
- `vacationRules.maxConcurrentVacations` must be a positive integer (if provided)
- `vacationRules.carryOverDays` must be a non-negative integer (if provided)
