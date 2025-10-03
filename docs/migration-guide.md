# Migration Guide

This guide provides step-by-step instructions for migrating TaskTaskRevolution manifests between API versions.

## Quick Start

### Check Current Versions
```bash
# Check API versions in your workspace
ttr validate system --check-versions

# List all manifests with their versions
find . -name "*.yaml" -exec grep -l "apiVersion:" {} \; | xargs grep -H "apiVersion:"
```

### Migrate All Manifests
```bash
# Migrate from v1alpha1 to v1beta1 (when available)
ttr migrate --from v1alpha1 --to v1beta1

# Dry run to see what would change
ttr migrate --from v1alpha1 --to v1beta1 --dry-run

# Migrate specific directory
ttr migrate --from v1alpha1 --to v1beta1 --path ./companies/
```

## Migration Commands

### Basic Migration
```bash
# Migrate all manifests
ttr migrate --from <old-version> --to <new-version>

# Migrate specific manifest type
ttr migrate --from v1alpha1 --to v1beta1 --kind Project

# Migrate specific file
ttr migrate --from v1alpha1 --to v1beta1 --file company.yaml

# Migrate with backup
ttr migrate --from v1alpha1 --to v1beta1 --backup ./backup/
```

### Advanced Options
```bash
# Force migration (skip validation)
ttr migrate --from v1alpha1 --to v1beta1 --force

# Verbose output
ttr migrate --from v1alpha1 --to v1beta1 --verbose

# Custom migration rules
ttr migrate --from v1alpha1 --to v1beta1 --rules ./custom-rules.yaml
```

## Version-Specific Migrations

### v1alpha1 → v1beta1 (Planned)

#### Breaking Changes
1. **Field Renames**:
   - `projectName` → `name` in Project manifests
   - `resourceType` → `type` in Resource manifests
   - `taskStatus` → `status` in Task manifests

2. **Field Type Changes**:
   - `startDate` and `endDate` now require timezone information
   - `priority` field now uses enum instead of string

3. **Validation Changes**:
   - Company codes must be uppercase
   - Project codes must follow pattern: `[A-Z]+-[0-9]+`
   - Resource codes must be unique within company

#### Migration Steps
```bash
# 1. Backup your data
cp -r companies/ backup-companies-$(date +%Y%m%d)/

# 2. Run migration
ttr migrate --from v1alpha1 --to v1beta1

# 3. Validate results
ttr validate system

# 4. Test functionality
ttr list companies
ttr list projects --company TECH-001
```

#### Manual Changes Required
Some changes require manual intervention:

1. **Update Company Codes**:
   ```yaml
   # Before (v1alpha1)
   metadata:
     code: "tech-001"  # lowercase
   
   # After (v1beta1)
   metadata:
     code: "TECH-001"  # uppercase
   ```

2. **Add Timezone Information**:
   ```yaml
   # Before (v1alpha1)
   spec:
     startDate: "2024-01-01"
     endDate: "2024-12-31"
   
   # After (v1beta1)
   spec:
     startDate: "2024-01-01T00:00:00Z"
     endDate: "2024-12-31T23:59:59Z"
     timezone: "UTC"
   ```

3. **Update Priority Enums**:
   ```yaml
   # Before (v1alpha1)
   spec:
     priority: "high"  # string
   
   # After (v1beta1)
   spec:
     priority: "High"  # enum value
   ```

### v1beta1 → v1stable (Future)

#### Planned Changes
1. **New Required Fields**:
   - All manifests will require `description` field
   - Projects will require `owner` field
   - Resources will require `email` field

2. **Enhanced Validation**:
   - Stricter date validation
   - Enhanced email validation
   - Required field validation

## Troubleshooting

### Common Migration Issues

#### 1. Validation Errors After Migration
```bash
# Check validation errors
ttr validate system --verbose

# Fix specific validation errors
ttr validate system --fix

# Validate specific manifest
ttr validate manifest company.yaml
```

#### 2. Missing Required Fields
```bash
# Add missing required fields
ttr migrate --from v1alpha1 --to v1beta1 --add-missing-fields

# Check what fields are missing
ttr validate manifest company.yaml --show-missing
```

#### 3. Field Type Mismatches
```bash
# Convert field types automatically
ttr migrate --from v1alpha1 --to v1beta1 --convert-types

# Manual field conversion
ttr migrate --from v1alpha1 --to v1beta1 --field-map priority:string:enum
```

### Rollback Procedures

#### Rollback Migration
```bash
# Rollback to previous version
ttr migrate --rollback --to v1alpha1

# Restore from backup
cp -r backup-companies-20240101/* companies/

# Validate rollback
ttr validate system
```

#### Partial Rollback
```bash
# Rollback specific manifest type
ttr migrate --rollback --to v1alpha1 --kind Project

# Rollback specific file
cp backup-companies-20240101/TECH-001/projects/WEB-APP/project.yaml \
   companies/TECH-001/projects/WEB-APP/project.yaml
```

## Migration Scripts

### Custom Migration Script
Create a custom migration script for complex scenarios:

```bash
#!/bin/bash
# custom-migration.sh

set -e

echo "Starting custom migration..."

# Backup data
echo "Creating backup..."
cp -r companies/ backup-$(date +%Y%m%d)/

# Run standard migration
echo "Running standard migration..."
ttr migrate --from v1alpha1 --to v1beta1

# Custom field mappings
echo "Applying custom field mappings..."
find companies/ -name "*.yaml" -exec sed -i 's/oldField:/newField:/g' {} \;

# Validate results
echo "Validating migration..."
ttr validate system

echo "Migration completed successfully!"
```

### Batch Migration Script
For large workspaces:

```bash
#!/bin/bash
# batch-migration.sh

COMPANIES_DIR="./companies"
BACKUP_DIR="./backup-$(date +%Y%m%d)"

echo "Starting batch migration for $(find $COMPANIES_DIR -name "*.yaml" | wc -l) manifests..."

# Create backup
mkdir -p "$BACKUP_DIR"
cp -r "$COMPANIES_DIR"/* "$BACKUP_DIR/"

# Process each company
for company in "$COMPANIES_DIR"/*; do
    if [ -d "$company" ]; then
        company_name=$(basename "$company")
        echo "Migrating company: $company_name"
        
        # Migrate company manifest
        ttr migrate --from v1alpha1 --to v1beta1 --file "$company/company.yaml"
        
        # Migrate projects
        for project in "$company/projects"/*; do
            if [ -d "$project" ]; then
                project_name=$(basename "$project")
                echo "  Migrating project: $project_name"
                ttr migrate --from v1alpha1 --to v1beta1 --file "$project/project.yaml"
            fi
        done
    fi
done

echo "Batch migration completed!"
```

## Best Practices

### Before Migration
1. **Backup Everything**: Always create a backup before migration
2. **Test in Staging**: Test migration in a staging environment first
3. **Review Breaking Changes**: Read the changelog for breaking changes
4. **Update Tools**: Ensure all tools support the target version

### During Migration
1. **Use Dry Run**: Always test with `--dry-run` first
2. **Monitor Progress**: Use `--verbose` to monitor migration progress
3. **Validate Frequently**: Run validation after each major step
4. **Document Changes**: Keep track of manual changes made

### After Migration
1. **Validate Everything**: Run full system validation
2. **Test Functionality**: Test all major functionality
3. **Update Documentation**: Update any documentation references
4. **Train Team**: Inform team about new features and changes

## Migration Checklist

### Pre-Migration
- [ ] Backup all data
- [ ] Review breaking changes
- [ ] Update tools to target version
- [ ] Test migration in staging
- [ ] Plan for manual changes

### During Migration
- [ ] Run dry-run migration
- [ ] Execute actual migration
- [ ] Apply manual changes
- [ ] Validate each step
- [ ] Document any issues

### Post-Migration
- [ ] Run full system validation
- [ ] Test all functionality
- [ ] Update documentation
- [ ] Train team members
- [ ] Monitor for issues

## Support

### Getting Help
- **Documentation**: Check this guide and API documentation
- **Community**: Ask questions in community forums
- **Issues**: Report bugs via GitHub issues
- **Migration Tools**: Use built-in migration tools and validation

### Reporting Issues
When reporting migration issues, include:
- Source and target versions
- Error messages and logs
- Sample manifests (anonymized)
- Steps to reproduce
- Expected vs actual behavior

## Examples

### Complete Migration Example
```bash
# 1. Check current state
ttr validate system --check-versions

# 2. Create backup
cp -r companies/ backup-$(date +%Y%m%d)/

# 3. Dry run migration
ttr migrate --from v1alpha1 --to v1beta1 --dry-run

# 4. Execute migration
ttr migrate --from v1alpha1 --to v1beta1 --verbose

# 5. Validate results
ttr validate system

# 6. Test functionality
ttr list companies
ttr list projects --company TECH-001
ttr report generate --type task --format csv
```

This completes the migration process. The system should now be running on the new API version with all manifests properly migrated.
