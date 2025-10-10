use crate::application::query::{QueryBuilder, EntityType};
use crate::domain::shared::query_parser::{QueryParser, QueryValue};
use clap::Args;

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Query string to parse and execute
    #[arg(long)]
    pub query: Option<String>,

    /// Entity type to query (project, task, resource)
    #[arg(long, default_value = "project")]
    pub entity_type: String,

    /// Output format (json, table, csv)
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Field to filter by
    #[arg(long)]
    pub field: Option<String>,

    /// Filter operator (=, !=, >, <, >=, <=, ~, !~)
    #[arg(long)]
    pub operator: Option<String>,

    /// Filter value
    #[arg(long)]
    pub value: Option<String>,

    /// Aggregation type (count, sum, avg, min, max)
    #[arg(long)]
    pub aggregate: Option<String>,

    /// Field to aggregate (for sum, avg, min, max)
    #[arg(long)]
    pub aggregate_field: Option<String>,

    /// Sort field
    #[arg(long)]
    pub sort: Option<String>,

    /// Sort order (asc, desc)
    #[arg(long, default_value = "asc")]
    pub order: String,

    /// Limit number of results
    #[arg(long)]
    pub limit: Option<usize>,

    /// Offset for pagination
    #[arg(long)]
    pub offset: Option<usize>,

    /// Show available fields for entity type
    #[arg(long)]
    pub show_fields: bool,
}

pub fn execute_query(args: QueryArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Parse entity type
    let entity_type = EntityType::from_str(&args.entity_type)?;

    // Show available fields if requested
    if args.show_fields {
        show_available_fields(entity_type);
        return Ok(());
    }

    // Build query
    let query = build_query(&args)?;

    // For now, we'll show a demo since we don't have the full repository setup
    show_query_demo(&args, &query, entity_type)?;

    Ok(())
}

fn build_query(args: &QueryArgs) -> Result<crate::domain::shared::query_parser::Query, Box<dyn std::error::Error>> {
    let mut builder = QueryBuilder::new();

    // Build query from string if provided
    if let Some(query_str) = &args.query {
        let mut parser = QueryParser::new(query_str.clone());
        let parsed_query = parser.parse()?;
        return Ok(parsed_query);
    }

    // Build query from individual parameters
    if let (Some(field), Some(operator), Some(value)) = (&args.field, &args.operator, &args.value) {
        let query_value = parse_query_value(value)?;
        builder = builder.filter(field, operator, query_value);
    } else if args.field.is_some() || args.operator.is_some() || args.value.is_some() {
        return Err("Field, operator, and value must all be provided together".into());
    }

    // Add aggregation
    if let Some(aggregate) = &args.aggregate {
        match aggregate.to_lowercase().as_str() {
            "count" => builder = builder.count(),
            "sum" => {
                let field = args.aggregate_field.as_ref()
                    .ok_or("Aggregate field is required for sum")?;
                builder = builder.sum(field);
            }
            "avg" | "average" => {
                let field = args.aggregate_field.as_ref()
                    .ok_or("Aggregate field is required for average")?;
                builder = builder.average(field);
            }
            "min" => {
                let field = args.aggregate_field.as_ref()
                    .ok_or("Aggregate field is required for min")?;
                builder = builder.min(field);
            }
            "max" => {
                let field = args.aggregate_field.as_ref()
                    .ok_or("Aggregate field is required for max")?;
                builder = builder.max(field);
            }
            _ => return Err(format!("Invalid aggregation type: {}", aggregate).into()),
        }
    }

    // Add sorting
    if let Some(sort_field) = &args.sort {
        let ascending = args.order.to_lowercase() != "desc";
        builder = builder.sort_by(sort_field, ascending);
    }

    // Add pagination
    if args.limit.is_some() || args.offset.is_some() {
        builder = builder.paginate(args.limit, args.offset);
    }

    builder.build().map_err(|e| e.into())
}

fn parse_query_value(value: &str) -> Result<QueryValue, Box<dyn std::error::Error>> {
    // Try to parse as different types
    if let Ok(bool_val) = value.parse::<bool>() {
        return Ok(QueryValue::Boolean(bool_val));
    }

    if let Ok(num_val) = value.parse::<f64>() {
        return Ok(QueryValue::Number(num_val));
    }

    if let Ok(date_val) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(QueryValue::Date(date_val));
    }

    if let Ok(datetime_val) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(QueryValue::DateTime(datetime_val));
    }

    // Default to string
    Ok(QueryValue::String(value.to_string()))
}

fn show_available_fields(entity_type: EntityType) {
    // Mock implementation - in real implementation, this would come from QueryExecutor
    let fields = match entity_type {
        EntityType::Project => vec![
            "id", "code", "name", "description", "status", "priority",
            "start_date", "end_date", "actual_start_date", "actual_end_date",
            "company_code", "manager_id", "created_by", "created_at", "updated_at",
            "task_count", "resource_count", "is_active", "priority_weight"
        ],
        EntityType::Task => vec![
            "id", "project_code", "code", "name", "description", "status",
            "start_date", "due_date", "actual_end_date", "priority", "category",
            "dependency_count", "assigned_resource_count", "is_overdue",
            "days_until_due", "priority_weight"
        ],
        EntityType::Resource => vec![
            "id", "code", "name", "email", "resource_type", "scope", "project_id",
            "start_date", "end_date", "time_off_balance", "vacation_count",
            "time_off_history_count", "task_assignment_count", "active_task_count",
            "current_allocation_percentage", "is_wip_limits_exceeded", "wip_status", "is_available"
        ],
    };

    println!("Available fields for {} entities:", entity_type);
    for field in fields {
        println!("  - {}", field);
    }
}

fn show_query_demo(
    args: &QueryArgs,
    query: &crate::domain::shared::query_parser::Query,
    entity_type: EntityType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Query Engine Demo");
    println!("=================");
    println!("Entity type: {}", entity_type);
    println!("Output format: {}", args.format);
    println!("Query: {}", query);

    if let Some(aggregation) = &query.aggregation {
        println!("Aggregation: {}", aggregation);
    }

    if let Some(sort) = &query.sort {
        println!("Sort: {} {}", sort.field, if sort.ascending { "ASC" } else { "DESC" });
    }

    if query.pagination.limit.is_some() || query.pagination.offset.is_some() {
        println!("Pagination: limit={:?}, offset={:?}", query.pagination.limit, query.pagination.offset);
    }

    println!("\nQuery Structure:");
    println!("- Expression: {:?}", query.expression);

    println!("\nSupported Query Syntax:");
    println!("- Simple filters: status:active");
    println!("- Comparisons: priority > high");
    println!("- String contains: name ~ 'developer'");
    println!("- Logical operators: status:active AND priority:high");
    println!("- Negation: NOT status:cancelled");
    println!("- Parentheses: (status:active OR status:pending) AND priority:high");

    println!("\nExample Queries:");
    println!("- ttr query --query \"status:active\" --entity-type project");
    println!("- ttr query --field priority --operator '>' --value high --entity-type task");
    println!("- ttr query --field name --operator '~' --value developer --entity-type resource");
    println!("- ttr query --field status --operator '=' --value active --aggregate count");
    println!("- ttr query --field priority --operator '=' --value high --sort name --order desc");
    println!("- ttr query --field status --operator '=' --value active --limit 10 --offset 0");

    Ok(())
}
