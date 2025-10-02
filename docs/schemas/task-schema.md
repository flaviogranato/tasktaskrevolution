# Task Schema

## Overview

The Task schema defines the structure for task entities in the TaskTaskRevolution system, following the Backstage/Kubernetes manifest pattern.

## API Version

- **apiVersion**: `tasktaskrevolution.io/v1alpha1`
- **kind**: `Task`

## Schema Structure

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id?: string
  code: string
  name: string
  description?: string
  createdAt?: string (ISO 8601)
  updatedAt?: string (ISO 8601)
  createdBy?: string
spec:
  projectCode: string
  assignee: string
  status: TaskStatus
  priority: TaskPriority
  estimatedStartDate?: string (YYYY-MM-DD)
  estimatedEndDate?: string (YYYY-MM-DD)
  actualStartDate?: string (YYYY-MM-DD)
  actualEndDate?: string (YYYY-MM-DD)
  dependencies: string[]
  tags: string[]
  effort: Effort
  acceptanceCriteria: string[]
  comments: Comment[]
```

## Field Descriptions

### Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | No | Unique identifier for the task |
| `code` | string | Yes | Human-readable task code |
| `name` | string | Yes | Task name |
| `description` | string | No | Task description |
| `createdAt` | string | No | Creation timestamp (ISO 8601) |
| `updatedAt` | string | No | Last update timestamp (ISO 8601) |
| `createdBy` | string | No | User who created the task |

### Spec

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `projectCode` | string | Yes | Associated project code |
| `assignee` | string | Yes | Assigned resource code |
| `status` | TaskStatus | Yes | Current task status |
| `priority` | TaskPriority | Yes | Task priority level |
| `estimatedStartDate` | string | No | Estimated start date (YYYY-MM-DD) |
| `estimatedEndDate` | string | No | Estimated end date (YYYY-MM-DD) |
| `actualStartDate` | string | No | Actual start date (YYYY-MM-DD) |
| `actualEndDate` | string | No | Actual end date (YYYY-MM-DD) |
| `dependencies` | string[] | No | List of task codes this task depends on |
| `tags` | string[] | No | List of tags for categorization |
| `effort` | Effort | Yes | Effort estimation and tracking |
| `acceptanceCriteria` | string[] | No | List of acceptance criteria |
| `comments` | Comment[] | No | List of task comments |

## Enums

### TaskStatus

```yaml
enum:
  - Planned
  - ToDo
  - InProgress
  - Done
  - Blocked
  - Cancelled
```

### TaskPriority

```yaml
enum:
  - Low
  - Medium
  - High
  - Critical
```

## Complex Types

### Effort

```yaml
type: object
properties:
  estimatedHours: number
  actualHours?: number
```

### Comment

```yaml
type: object
properties:
  author: string
  message: string
  timestamp: string (YYYY-MM-DD)
```

## Example

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: "0199598c-f574-7fdc-8887-354407d0e74b"
  code: "TASK-001"
  name: "Implement User Authentication"
  description: "Implement secure user authentication system with JWT tokens"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-15T10:30:00Z"
  createdBy: "project.manager@techcorp.com"
spec:
  projectCode: "WEB-APP-001"
  assignee: "DEV-001"
  status: InProgress
  priority: High
  estimatedStartDate: "2024-01-15"
  estimatedEndDate: "2024-01-25"
  actualStartDate: "2024-01-15"
  actualEndDate: null
  dependencies:
    - "TASK-000"
  tags:
    - "backend"
    - "security"
    - "authentication"
  effort:
    estimatedHours: 40.0
    actualHours: 25.5
  acceptanceCriteria:
    - "Users can register with email and password"
    - "Users can login and receive JWT token"
    - "JWT tokens expire after 24 hours"
    - "Password must meet security requirements"
  comments:
    - author: "DEV-001"
      message: "Started implementation of login endpoint"
      timestamp: "2024-01-15"
    - author: "QA-001"
      message: "Please add input validation for email format"
      timestamp: "2024-01-16"
```

## Validation Rules

- `code` must be unique within the project
- `name` must not be empty
- `projectCode` must reference an existing project
- `assignee` must reference an existing resource (if not "unassigned")
- `estimatedStartDate` must be before `estimatedEndDate` (if both provided)
- `actualStartDate` must be before `actualEndDate` (if both provided)
- `estimatedStartDate`, `estimatedEndDate`, `actualStartDate`, `actualEndDate` must be valid dates in YYYY-MM-DD format
- `dependencies` must reference existing task codes within the same project
- `effort.estimatedHours` must be a positive number
- `effort.actualHours` must be a non-negative number (if provided)
- `priority` must be one of: Low, Medium, High, Critical
- `status` must be one of: Planned, ToDo, InProgress, Done, Blocked, Cancelled
