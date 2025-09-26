use crate::domain::shared::query_parser::QueryParser;
use clap::Args;

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Query string to parse and execute
    #[arg(long)]
    pub query: String,

    /// Entity type to query (project, task, resource, company)
    #[arg(long, default_value = "project")]
    pub entity_type: String,

    /// Output format (json, table)
    #[arg(long, default_value = "table")]
    pub format: String,
}

pub fn execute_query(args: QueryArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the query
    let mut parser = QueryParser::new(args.query.clone());
    let query = match parser.parse() {
        Ok(q) => q,
        Err(e) => {
            eprintln!("Error parsing query: {}", e);
            return Ok(());
        }
    };

    println!("✅ Query Parser Demo");
    println!("===================");
    println!("Query string: {}", args.query);
    println!("Parsed query: {}", query);
    println!("Entity type: {}", args.entity_type);
    println!("Output format: {}", args.format);

    println!("\n📋 Query Structure:");
    println!("- Expression: {:?}", query.expression);

    println!("\n🎯 Supported Query Syntax:");
    println!("- Simple filters: status:active");
    println!("- Comparisons: priority > high");
    println!("- String contains: name ~ 'developer'");
    println!("- Logical operators: status:active AND priority:high");
    println!("- Negation: NOT status:cancelled");
    println!("- Parentheses: (status:active OR status:pending) AND priority:high");

    println!("\n📊 Example Queries:");
    println!("- ttr query --query \"status:active\" --entity-type project");
    println!("- ttr query --query \"priority > medium\" --entity-type task");
    println!("- ttr query --query \"name ~ 'developer'\" --entity-type resource");
    println!("- ttr query --query \"status:active AND is_active:true\" --entity-type project");
    println!("- ttr query --query \"NOT status:cancelled\" --entity-type task");

    Ok(())
}
