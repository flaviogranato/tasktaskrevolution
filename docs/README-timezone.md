# Timezone System - TaskTaskRevolution

## Quick Start

The Timezone System provides comprehensive multi-timezone support for global projects. Get started quickly with these essential commands:

```bash
# Set your timezone
ttr timezone set --timezone "America/New_York"

# Convert time between timezones
ttr timezone convert --from "UTC" --to "America/New_York" --time "2024-01-15 14:00:00"

# Create a global schedule
ttr timezone sync --project "Global Project" --timezone "UTC"

# Generate a timezone report
ttr timezone report --format json
```

## Features

### 🌍 Multi-Timezone Coordination
- Automatic timezone detection and conversion
- Cross-timezone schedule coordination
- Conflict detection and resolution
- Working hours validation

### 📊 Advanced Analytics
- Performance metrics across timezones
- Collaboration insights and trends
- Efficiency analysis and optimization
- Risk assessment and recommendations

### ⚡ Global Scheduling
- Schedule optimization across timezones
- Resource leveling with timezone constraints
- Critical path calculation with timezone awareness
- Automatic conflict resolution

### 🔧 Developer-Friendly
- Comprehensive CLI commands
- Rich configuration options
- Extensive documentation and examples
- Full test coverage

## Installation

The timezone system is included with TaskTaskRevolution. No additional installation required.

```bash
# Verify timezone support
ttr timezone --help
```

## Configuration

### Basic Configuration

Create a `.ttr/timezone-preferences.yaml` file:

```yaml
timezone_preferences:
  default_timezone: "UTC"
  preferred_timezones:
    - "UTC"
    - "America/New_York"
    - "Europe/London"
  working_hours:
    start: "09:00"
    end: "17:00"
    days: ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
```

### Global Schedule Configuration

Create a `.ttr/global-schedules.yaml` file:

```yaml
global_schedules:
  - id: "project1"
    name: "Global Product Launch"
    timezone: "UTC"
    start_time: "2024-01-15T09:00:00Z"
    end_time: "2024-01-15T17:00:00Z"
    participants:
      - user_id: "alice@company.com"
        timezone: "America/New_York"
        name: "Alice Johnson"
        is_required: true
```

## CLI Commands

### Basic Commands

| Command | Description |
|---------|-------------|
| `ttr timezone set` | Set timezone for current context |
| `ttr timezone convert` | Convert time between timezones |
| `ttr timezone list` | List available timezones |
| `ttr timezone show` | Show current timezone information |

### Advanced Commands

| Command | Description |
|---------|-------------|
| `ttr timezone sync` | Synchronize schedules across timezones |
| `ttr timezone report` | Generate timezone reports and analytics |
| `ttr timezone validate` | Validate timezone configurations |
| `ttr timezone monitor` | Monitor timezone operations |

## Examples

### Convert Time Between Timezones

```bash
# Convert UTC to New York time
ttr timezone convert --from "UTC" --to "America/New_York" --time "2024-01-15 14:00:00"
# Output: 2024-01-15 09:00:00 EST

# Convert to multiple timezones
ttr timezone convert --from "UTC" --to "America/New_York" --to "Europe/London" --to "Asia/Tokyo" --time "2024-01-15 14:00:00"
```

### Create Global Schedules

```bash
# Create a global schedule
ttr timezone sync --project "Global Project" --timezone "UTC" --start "2024-01-15 09:00:00" --end "2024-01-15 17:00:00"

# Add participants from different timezones
ttr timezone sync --add-participant "alice@company.com" --timezone "America/New_York" --required
ttr timezone sync --add-participant "bob@company.com" --timezone "Europe/London" --required
```

### Generate Reports

```bash
# Generate comprehensive report
ttr timezone report --format json --output timezone-report.json

# Generate performance metrics
ttr timezone report --metrics performance --format csv --output performance.csv

# Generate collaboration insights
ttr timezone report --metrics collaboration --format table
```

## API Reference

### Core Components

- **TimezoneManager** - Core timezone information management
- **TimezoneConverter** - Timezone conversion utilities
- **GlobalScheduler** - Multi-timezone scheduling coordination
- **GlobalReporter** - Analytics and reporting
- **TimezoneValidator** - Timezone data validation

### Data Models

- **GlobalSchedule** - Represents a schedule with timezone information
- **GlobalParticipant** - Represents a participant in a global schedule
- **TimezonePreferences** - User timezone preferences
- **CoordinationResult** - Result of schedule coordination

## Testing

Run the timezone system tests:

```bash
# Run all timezone tests
cargo test --lib timezone

# Run specific component tests
cargo test --lib global_scheduler
cargo test --lib global_reporter
cargo test --lib timezone_converter
```

## Performance

The timezone system is optimized for performance:

- **Cached conversions** for frequently used timezones
- **Optimized algorithms** for large schedule coordination
- **Incremental metrics** aggregation
- **Memory-efficient** data structures

## Troubleshooting

### Common Issues

1. **Invalid timezone error**
   ```bash
   ttr timezone validate --timezone "America/New_York"
   ```

2. **Schedule conflicts**
   ```bash
   ttr timezone sync --detect-conflicts --schedule "project1"
   ttr timezone sync --resolve-conflicts --schedule "project1"
   ```

3. **Performance issues**
   ```bash
   ttr timezone sync --optimize --schedule "project1"
   ttr timezone report --performance --schedule "project1"
   ```

### Debug Mode

Enable debug mode for detailed information:

```bash
ttr timezone --debug sync --schedule "project1"
```

## Contributing

To contribute to the timezone system:

1. Read the [Timezone System Documentation](timezone-system.md)
2. Check the [Examples](timezone-examples.md) for usage patterns
3. Run the test suite: `cargo test --lib timezone`
4. Follow the coding standards and best practices

## License

The timezone system is part of TaskTaskRevolution and follows the same license terms.

## Support

For support with the timezone system:

1. Check the [documentation](timezone-system.md)
2. Review the [examples](timezone-examples.md)
3. Run diagnostics: `ttr timezone report --diagnostics`
4. Open an issue on GitHub

## Changelog

### Version 0.7.0
- Initial implementation of timezone system
- Multi-timezone coordination support
- Global scheduling capabilities
- Advanced analytics and reporting
- Comprehensive CLI commands
- Full test coverage

