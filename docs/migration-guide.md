# Migration Guide

## Overview

This guide provides step-by-step instructions for migrating TaskTaskRevolution manifests between API versions.

## Quick Start

### Automatic Migration

The easiest way to migrate your manifests is using the built-in migration tool:

```bash
# Migrate all manifests in current directory
ttr migrate manifests

# Migrate specific file
ttr migrate manifests --file company.yaml

# Force migration (overwrite existing files)
ttr migrate manifests --force

# Dry run (show what would be migrated)
ttr migrate manifests --dry-run
```

### Manual Migration

For complex scenarios or when you need more control, follow the manual migration steps below.

## Migration Scenarios

### Scenario 1: Alpha to Beta Migration

#### Step 1: Backup Your Manifests

```bash
# Create backup directory
mkdir -p backup/$(date +%Y%m%d)

# Copy all manifests
cp -r companies/ backup/$(date +%Y%m%d)/
cp -r projects/ backup/$(date +%Y%m%d)/
cp -r resources/ backup/$(date +%Y%m%d)/
cp -r tasks/ backup/$(date +%Y%m%d)/
```

#### Step 2: Update API Version

```bash
# Update all YAML files
find . -name "*.yaml" -exec sed -i 's/tasktaskrevolution.io\/v1alpha1/tasktaskrevolution.io\/v1beta1/g' {} \;
```

#### Step 3: Add New Fields

For each manifest type, add the new fields:

**Company Manifest:**
```yaml
# Before
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "123"
  code: "COMP-001"
  name: "Company Name"
spec:
  description: "Company description"

# After
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
```

**Project Manifest:**
```yaml
# Before
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id: "456"
  code: "PROJ-001"
  name: "Project Name"
spec:
  status: "Planned"

# After
apiVersion: tasktaskrevolution.io/v1beta1
kind: Project
metadata:
  id: "456"
  code: "PROJ-001"
  name: "Project Name"
  labels: {}  # New field
  annotations: {}  # New field
  namespace: "default"  # New field
spec:
  status: "Planned"
```

**Resource Manifest:**
```yaml
# Before
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  id: "789"
  code: "RES-001"
  name: "Resource Name"
  email: "resource@example.com"
  resourceType: "Developer"
spec:
  scope: "Company"

# After
apiVersion: tasktaskrevolution.io/v1beta1
kind: Resource
metadata:
  id: "789"
  code: "RES-001"
  name: "Resource Name"
  email: "resource@example.com"
  resourceType: "Developer"
  labels: {}  # New field
  annotations: {}  # New field
  namespace: "default"  # New field
spec:
  scope: "Company"
```

**Task Manifest:**
```yaml
# Before
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: "101"
  code: "TASK-001"
  name: "Task Name"
spec:
  projectCode: "PROJ-001"
  status: "Planned"

# After
apiVersion: tasktaskrevolution.io/v1beta1
kind: Task
metadata:
  id: "101"
  code: "TASK-001"
  name: "Task Name"
  labels: {}  # New field
  annotations: {}  # New field
  namespace: "default"  # New field
spec:
  projectCode: "PROJ-001"
  status: "Planned"
```

#### Step 4: Validate Migration

```bash
# Validate all manifests
ttr validate

# Check specific manifest
ttr validate --file company.yaml
```

### Scenario 2: Beta to Stable Migration

#### Step 1: Update API Version

```bash
# Update all YAML files
find . -name "*.yaml" -exec sed -i 's/tasktaskrevolution.io\/v1beta1/tasktaskrevolution.io\/v1/g' {} \;
```

#### Step 2: Remove Deprecated Fields

Check for any deprecated fields and remove them:

```yaml
# Remove deprecated fields
metadata:
  # Remove any fields marked as deprecated
  # Keep only stable fields
```

#### Step 3: Validate Migration

```bash
# Validate all manifests
ttr validate

# Run comprehensive tests
ttr test
```

## Common Migration Issues

### Issue 1: Missing Required Fields

**Error**: `Missing required field 'labels'`

**Solution**: Add the missing field:

```yaml
metadata:
  labels: {}  # Add empty labels
  annotations: {}  # Add empty annotations
  namespace: "default"  # Add default namespace
```

### Issue 2: Invalid Field Types

**Error**: `Invalid field type for 'labels'`

**Solution**: Ensure correct field types:

```yaml
metadata:
  labels: {}  # Must be object, not string
  annotations: {}  # Must be object, not string
  namespace: "default"  # Must be string, not object
```

### Issue 3: Deprecated Fields

**Error**: `Field 'oldField' is deprecated`

**Solution**: Replace with new field:

```yaml
# Before
spec:
  oldField: "value"

# After
spec:
  newField: "value"  # Use new field name
```

## Migration Scripts

### Bash Script for Alpha to Beta

```bash
#!/bin/bash
# migrate-alpha-to-beta.sh

echo "Starting migration from v1alpha1 to v1beta1..."

# Backup
echo "Creating backup..."
mkdir -p backup/$(date +%Y%m%d)
cp -r companies/ backup/$(date +%Y%m%d)/
cp -r projects/ backup/$(date +%Y%m%d)/
cp -r resources/ backup/$(date +%Y%m%d)/
cp -r tasks/ backup/$(date +%Y%m%d)/

# Update API version
echo "Updating API version..."
find . -name "*.yaml" -exec sed -i 's/tasktaskrevolution.io\/v1alpha1/tasktaskrevolution.io\/v1beta1/g' {} \;

# Add new fields to Company manifests
echo "Adding new fields to Company manifests..."
find companies/ -name "*.yaml" -exec sed -i '/^  name:.*/a\  labels: {}\n  annotations: {}\n  namespace: "default"' {} \;

# Add new fields to Project manifests
echo "Adding new fields to Project manifests..."
find projects/ -name "*.yaml" -exec sed -i '/^  name:.*/a\  labels: {}\n  annotations: {}\n  namespace: "default"' {} \;

# Add new fields to Resource manifests
echo "Adding new fields to Resource manifests..."
find resources/ -name "*.yaml" -exec sed -i '/^  resourceType:.*/a\  labels: {}\n  annotations: {}\n  namespace: "default"' {} \;

# Add new fields to Task manifests
echo "Adding new fields to Task manifests..."
find tasks/ -name "*.yaml" -exec sed -i '/^  name:.*/a\  labels: {}\n  annotations: {}\n  namespace: "default"' {} \;

# Validate
echo "Validating migration..."
ttr validate

echo "Migration completed!"
```

### Python Script for Complex Migrations

```python
#!/usr/bin/env python3
# migrate_manifests.py

import os
import yaml
import shutil
from datetime import datetime

def backup_manifests():
    """Create backup of all manifests"""
    backup_dir = f"backup/{datetime.now().strftime('%Y%m%d')}"
    os.makedirs(backup_dir, exist_ok=True)
    
    for dir_name in ['companies', 'projects', 'resources', 'tasks']:
        if os.path.exists(dir_name):
            shutil.copytree(dir_name, f"{backup_dir}/{dir_name}")

def migrate_company_manifest(file_path):
    """Migrate company manifest from alpha to beta"""
    with open(file_path, 'r') as f:
        manifest = yaml.safe_load(f)
    
    # Update API version
    manifest['apiVersion'] = 'tasktaskrevolution.io/v1beta1'
    
    # Add new fields to metadata
    if 'metadata' not in manifest:
        manifest['metadata'] = {}
    
    if 'labels' not in manifest['metadata']:
        manifest['metadata']['labels'] = {}
    if 'annotations' not in manifest['metadata']:
        manifest['metadata']['annotations'] = {}
    if 'namespace' not in manifest['metadata']:
        manifest['metadata']['namespace'] = 'default'
    
    # Write back
    with open(file_path, 'w') as f:
        yaml.dump(manifest, f, default_flow_style=False)

def migrate_project_manifest(file_path):
    """Migrate project manifest from alpha to beta"""
    with open(file_path, 'r') as f:
        manifest = yaml.safe_load(f)
    
    # Update API version
    manifest['apiVersion'] = 'tasktaskrevolution.io/v1beta1'
    
    # Add new fields to metadata
    if 'metadata' not in manifest:
        manifest['metadata'] = {}
    
    if 'labels' not in manifest['metadata']:
        manifest['metadata']['labels'] = {}
    if 'annotations' not in manifest['metadata']:
        manifest['metadata']['annotations'] = {}
    if 'namespace' not in manifest['metadata']:
        manifest['metadata']['namespace'] = 'default'
    
    # Write back
    with open(file_path, 'w') as f:
        yaml.dump(manifest, f, default_flow_style=False)

def migrate_resource_manifest(file_path):
    """Migrate resource manifest from alpha to beta"""
    with open(file_path, 'r') as f:
        manifest = yaml.safe_load(f)
    
    # Update API version
    manifest['apiVersion'] = 'tasktaskrevolution.io/v1beta1'
    
    # Add new fields to metadata
    if 'metadata' not in manifest:
        manifest['metadata'] = {}
    
    if 'labels' not in manifest['metadata']:
        manifest['metadata']['labels'] = {}
    if 'annotations' not in manifest['metadata']:
        manifest['metadata']['annotations'] = {}
    if 'namespace' not in manifest['metadata']:
        manifest['metadata']['namespace'] = 'default'
    
    # Write back
    with open(file_path, 'w') as f:
        yaml.dump(manifest, f, default_flow_style=False)

def migrate_task_manifest(file_path):
    """Migrate task manifest from alpha to beta"""
    with open(file_path, 'r') as f:
        manifest = yaml.safe_load(f)
    
    # Update API version
    manifest['apiVersion'] = 'tasktaskrevolution.io/v1beta1'
    
    # Add new fields to metadata
    if 'metadata' not in manifest:
        manifest['metadata'] = {}
    
    if 'labels' not in manifest['metadata']:
        manifest['metadata']['labels'] = {}
    if 'annotations' not in manifest['metadata']:
        manifest['metadata']['annotations'] = {}
    if 'namespace' not in manifest['metadata']:
        manifest['metadata']['namespace'] = 'default'
    
    # Write back
    with open(file_path, 'w') as f:
        yaml.dump(manifest, f, default_flow_style=False)

def main():
    """Main migration function"""
    print("Starting migration from v1alpha1 to v1beta1...")
    
    # Create backup
    backup_manifests()
    print("Backup created")
    
    # Migrate company manifests
    for root, dirs, files in os.walk('companies'):
        for file in files:
            if file.endswith('.yaml'):
                file_path = os.path.join(root, file)
                migrate_company_manifest(file_path)
                print(f"Migrated: {file_path}")
    
    # Migrate project manifests
    for root, dirs, files in os.walk('projects'):
        for file in files:
            if file.endswith('.yaml'):
                file_path = os.path.join(root, file)
                migrate_project_manifest(file_path)
                print(f"Migrated: {file_path}")
    
    # Migrate resource manifests
    for root, dirs, files in os.walk('resources'):
        for file in files:
            if file.endswith('.yaml'):
                file_path = os.path.join(root, file)
                migrate_resource_manifest(file_path)
                print(f"Migrated: {file_path}")
    
    # Migrate task manifests
    for root, dirs, files in os.walk('tasks'):
        for file in files:
            if file.endswith('.yaml'):
                file_path = os.path.join(root, file)
                migrate_task_manifest(file_path)
                print(f"Migrated: {file_path}")
    
    print("Migration completed!")

if __name__ == "__main__":
    main()
```

## Testing Migration

### Pre-Migration Testing

```bash
# Validate current state
ttr validate

# Run tests
ttr test

# Check for deprecated fields
ttr validate --check-deprecated
```

### Post-Migration Testing

```bash
# Validate migrated manifests
ttr validate

# Run comprehensive tests
ttr test

# Check for any issues
ttr validate --strict
```

## Rollback Procedures

### Automatic Rollback

```bash
# Restore from backup
cp -r backup/20240101/* .

# Validate rollback
ttr validate
```

### Manual Rollback

1. **Stop all operations**
2. **Restore from backup**
3. **Validate restored state**
4. **Resume operations**

## Best Practices

### Before Migration

1. **Always backup** your manifests
2. **Test in development** environment first
3. **Review breaking changes**
4. **Plan migration window**
5. **Notify stakeholders**

### During Migration

1. **Use dry-run mode** first
2. **Migrate in batches** if large dataset
3. **Monitor for errors**
4. **Validate each step**
5. **Keep backups accessible**

### After Migration

1. **Validate all manifests**
2. **Run comprehensive tests**
3. **Monitor system behavior**
4. **Document any issues**
5. **Update documentation**

## Troubleshooting

### Common Issues

1. **Missing fields**: Add required fields
2. **Invalid types**: Fix field types
3. **Deprecated fields**: Replace with new fields
4. **Validation errors**: Check field requirements
5. **Performance issues**: Optimize large datasets

### Getting Help

- **Documentation**: [docs.tasktaskrevolution.io](https://docs.tasktaskrevolution.io)
- **Issues**: [GitHub Issues](https://github.com/tasktaskrevolution/issues)
- **Discussions**: [GitHub Discussions](https://github.com/tasktaskrevolution/discussions)
- **Email**: support@tasktaskrevolution.io

## Conclusion

Migration between API versions is a critical process that requires careful planning and execution. Follow the steps in this guide to ensure a smooth migration experience.

Remember to always backup your data and test thoroughly before applying changes to production environments.