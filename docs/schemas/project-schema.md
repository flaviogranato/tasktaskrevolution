# Project Schema

## Overview

The Project schema defines the structure for project entities in the TaskTaskRevolution system, following the Backstage/Kubernetes manifest pattern.

## API Version

- **apiVersion**: `tasktaskrevolution.io/v1alpha1`
- **kind**: `Project`

## Schema Structure

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id?: string
  code?: string
  name: string
  description: string
  companyCode?: string
  createdAt?: string (ISO 8601)
  updatedAt?: string (ISO 8601)
  createdBy?: string
spec:
  timezone?: string
  startDate?: string (YYYY-MM-DD)
  endDate?: string (YYYY-MM-DD)
  status: ProjectStatus
  vacationRules?: VacationRules
```

## Field Descriptions

### Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | No | Unique identifier for the project |
| `code` | string | No | Human-readable project code |
| `name` | string | Yes | Project name |
| `description` | string | No | Project description |
| `companyCode` | string | No | Associated company code |
| `createdAt` | string | No | Creation timestamp (ISO 8601) |
| `updatedAt` | string | No | Last update timestamp (ISO 8601) |
| `createdBy` | string | No | User who created the project |

### Spec

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timezone` | string | No | Project timezone (e.g., "UTC", "America/New_York") |
| `startDate` | string | No | Project start date (YYYY-MM-DD format) |
| `endDate` | string | No | Project end date (YYYY-MM-DD format) |
| `status` | ProjectStatus | Yes | Current project status |
| `vacationRules` | VacationRules | No | Vacation rules for the project |

## Enums

### ProjectStatus

```yaml
enum:
  - Planned
  - InProgress
  - OnHold
  - Completed
  - Cancelled
```

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

## Example

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id: "0199598c-f574-7fdc-8887-354407d0e74b"
  code: "WEB-APP-001"
  name: "E-commerce Web Application"
  description: "A modern e-commerce platform built with React and Node.js"
  companyCode: "TECH-CORP"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-15T10:30:00Z"
  createdBy: "project.manager@techcorp.com"
spec:
  timezone: "America/New_York"
  startDate: "2024-01-15"
  endDate: "2024-06-30"
  status: InProgress
  vacationRules:
    maxConcurrentVacations: 20
    carryOverDays: 5
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
    layoffPeriods:
      - startDate: "2024-12-20"
        endDate: "2025-01-05"
```

## Validation Rules

- `name` must not be empty
- `code` must be unique within the company (if provided)
- `startDate` must be before `endDate` (if both provided)
- `startDate` and `endDate` must be valid dates in YYYY-MM-DD format
- `timezone` must be a valid timezone identifier (if provided)
- `companyCode` must reference an existing company (if provided)
- `vacationRules.maxConcurrentVacations` must be a positive integer (if provided)
- `vacationRules.carryOverDays` must be a non-negative integer (if provided)
