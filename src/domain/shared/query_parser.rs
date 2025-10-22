use serde::{Deserialize, Serialize};
use std::fmt;

/// Representa um operador de comparação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    // Operadores básicos
    Equal,          // =
    NotEqual,       // !=
    GreaterThan,    // >
    LessThan,       // <
    GreaterOrEqual, // >=
    LessOrEqual,    // <=

    // Operadores de string
    Contains,    // ~ (contém)
    NotContains, // !~ (não contém)
    StartsWith,  // ^ (começa com)
    EndsWith,    // $ (termina com)
    Regex,       // ~* (regex case-insensitive)
    NotRegex,    // !~* (não regex case-insensitive)

    // Operadores de array
    In,    // IN (está em)
    NotIn, // NOT IN (não está em)

    // Operadores de range
    Between,    // BETWEEN (entre)
    NotBetween, // NOT BETWEEN (não entre)

    // Operadores de null
    IsNull,    // IS NULL (é nulo)
    IsNotNull, // IS NOT NULL (não é nulo)
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Operadores básicos
            ComparisonOperator::Equal => write!(f, "="),
            ComparisonOperator::NotEqual => write!(f, "!="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::GreaterOrEqual => write!(f, ">="),
            ComparisonOperator::LessOrEqual => write!(f, "<="),

            // Operadores de string
            ComparisonOperator::Contains => write!(f, "~"),
            ComparisonOperator::NotContains => write!(f, "!~"),
            ComparisonOperator::StartsWith => write!(f, "^"),
            ComparisonOperator::EndsWith => write!(f, "$"),
            ComparisonOperator::Regex => write!(f, "~*"),
            ComparisonOperator::NotRegex => write!(f, "!~*"),

            // Operadores de array
            ComparisonOperator::In => write!(f, "IN"),
            ComparisonOperator::NotIn => write!(f, "NOT IN"),

            // Operadores de range
            ComparisonOperator::Between => write!(f, "BETWEEN"),
            ComparisonOperator::NotBetween => write!(f, "NOT BETWEEN"),

            // Operadores de null
            ComparisonOperator::IsNull => write!(f, "IS NULL"),
            ComparisonOperator::IsNotNull => write!(f, "IS NOT NULL"),
        }
    }
}

/// Representa um operador lógico
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "AND"),
            LogicalOperator::Or => write!(f, "OR"),
            LogicalOperator::Not => write!(f, "NOT"),
        }
    }
}

/// Representa um valor em uma consulta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
    Array(Vec<QueryValue>),
    Range {
        start: Box<QueryValue>,
        end: Box<QueryValue>,
    },
    Null,
}

impl fmt::Display for QueryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryValue::String(s) => write!(f, "'{}'", s),
            QueryValue::Number(n) => write!(f, "{}", n),
            QueryValue::Boolean(b) => write!(f, "{}", b),
            QueryValue::Date(d) => write!(f, "{}", d.format("%Y-%m-%d")),
            QueryValue::DateTime(dt) => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
            QueryValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            QueryValue::Range { start, end } => write!(f, "{} AND {}", start, end),
            QueryValue::Null => write!(f, "NULL"),
        }
    }
}

/// Representa uma condição de filtro
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: QueryValue,
}

impl fmt::Display for FilterCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.field, self.operator, self.value)
    }
}

/// Representa uma expressão de consulta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryExpression {
    Condition(FilterCondition),
    Logical {
        operator: LogicalOperator,
        left: Box<QueryExpression>,
        right: Option<Box<QueryExpression>>,
    },
    Not(Box<QueryExpression>),
}

impl fmt::Display for QueryExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryExpression::Condition(condition) => write!(f, "{}", condition),
            QueryExpression::Logical { operator, left, right } => {
                write!(f, "({}", left)?;
                if let Some(right) = right {
                    write!(f, " {} {}", operator, right)?;
                }
                write!(f, ")")
            }
            QueryExpression::Not(expr) => write!(f, "NOT ({})", expr),
        }
    }
}

/// Representa um tipo de agregação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationType {
    Count,
    Sum(String),     // field name
    Average(String), // field name
    Min(String),     // field name
    Max(String),     // field name
}

impl fmt::Display for AggregationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregationType::Count => write!(f, "COUNT"),
            AggregationType::Sum(field) => write!(f, "SUM({})", field),
            AggregationType::Average(field) => write!(f, "AVG({})", field),
            AggregationType::Min(field) => write!(f, "MIN({})", field),
            AggregationType::Max(field) => write!(f, "MAX({})", field),
        }
    }
}

/// Representa uma ordenação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortOption {
    pub field: String,
    pub ascending: bool,
}

impl fmt::Display for SortOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction = if self.ascending { "ASC" } else { "DESC" };
        write!(f, "{} {}", self.field, direction)
    }
}

/// Representa opções de paginação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginationOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Representa uma projeção de campo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldProjection {
    pub field: String,
    pub alias: Option<String>,
}

/// Representa opções de projeção
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionOptions {
    pub fields: Vec<FieldProjection>,
    pub include_all: bool,
}

impl PaginationOptions {
    pub fn new(limit: Option<usize>, offset: Option<usize>) -> Self {
        Self { limit, offset }
    }

    pub fn new_default() -> Self {
        Self {
            limit: None,
            offset: None,
        }
    }
}

impl FieldProjection {
    pub fn new(field: String) -> Self {
        Self { field, alias: None }
    }

    pub fn with_alias(field: String, alias: String) -> Self {
        Self {
            field,
            alias: Some(alias),
        }
    }
}

impl ProjectionOptions {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            include_all: true,
        }
    }

    pub fn with_fields(fields: Vec<FieldProjection>) -> Self {
        Self {
            fields,
            include_all: false,
        }
    }

    pub fn add_field(mut self, field: FieldProjection) -> Self {
        self.fields.push(field);
        self.include_all = false;
        self
    }

    pub fn include_all_fields() -> Self {
        Self {
            fields: Vec::new(),
            include_all: true,
        }
    }
}

/// Representa uma consulta completa
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    pub expression: QueryExpression,
    pub aggregation: Option<AggregationType>,
    pub sort: Option<SortOption>,
    pub pagination: PaginationOptions,
    pub projection: ProjectionOptions,
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}

impl Query {
    pub fn new(expression: QueryExpression) -> Self {
        Self {
            expression,
            aggregation: None,
            sort: None,
            pagination: PaginationOptions::new_default(),
            projection: ProjectionOptions::include_all_fields(),
        }
    }

    pub fn with_aggregation(mut self, aggregation: AggregationType) -> Self {
        self.aggregation = Some(aggregation);
        self
    }

    pub fn with_sort(mut self, field: String, ascending: bool) -> Self {
        self.sort = Some(SortOption { field, ascending });
        self
    }

    pub fn with_pagination(mut self, limit: Option<usize>, offset: Option<usize>) -> Self {
        self.pagination = PaginationOptions::new(limit, offset);
        self
    }

    pub fn with_projection(mut self, projection: ProjectionOptions) -> Self {
        self.projection = projection;
        self
    }

    pub fn select_fields(mut self, fields: Vec<FieldProjection>) -> Self {
        self.projection = ProjectionOptions::with_fields(fields);
        self
    }

    pub fn select_field(mut self, field: String) -> Self {
        let field_projection = FieldProjection::new(field);
        self.projection = self.projection.add_field(field_projection);
        self
    }

    pub fn select_field_with_alias(mut self, field: String, alias: String) -> Self {
        let field_projection = FieldProjection::with_alias(field, alias);
        self.projection = self.projection.add_field(field_projection);
        self
    }
}

/// Erro de parsing de consulta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryParseError {
    InvalidSyntax(String),
    UnsupportedOperator(String),
    InvalidField(String),
    InvalidValue(String),
    UnexpectedToken(String),
    IncompleteExpression,
}

impl fmt::Display for QueryParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            QueryParseError::UnsupportedOperator(op) => write!(f, "Unsupported operator: {}", op),
            QueryParseError::InvalidField(field) => write!(f, "Invalid field: {}", field),
            QueryParseError::InvalidValue(value) => write!(f, "Invalid value: {}", value),
            QueryParseError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            QueryParseError::IncompleteExpression => write!(f, "Incomplete expression"),
        }
    }
}

impl std::error::Error for QueryParseError {}

/// Parser de consultas
pub struct QueryParser {
    input: String,
    position: usize,
}

impl QueryParser {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    /// Parse a query string into a Query AST
    pub fn parse(&mut self) -> Result<Query, QueryParseError> {
        let expression = self.parse_expression()?;
        Ok(Query {
            expression,
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
            projection: ProjectionOptions::include_all_fields(),
        })
    }

    fn parse_expression(&mut self) -> Result<QueryExpression, QueryParseError> {
        self.skip_whitespace();

        if self.peek() == Some('(') {
            let _ = self.consume('(');
            let expr = self.parse_expression()?;
            self.expect(')')?;
            return Ok(expr);
        }

        if self.peek() == Some('!') {
            let _ = self.consume('!');
            let expr = self.parse_expression()?;
            return Ok(QueryExpression::Not(Box::new(expr)));
        }

        // Handle NOT keyword
        if self.starts_with("NOT") {
            self.advance_by(3);
            self.skip_whitespace();
            let expr = self.parse_expression()?;
            return Ok(QueryExpression::Not(Box::new(expr)));
        }

        let condition = self.parse_condition()?;
        self.skip_whitespace();

        if let Some(op) = self.parse_logical_operator() {
            self.skip_whitespace();
            let right = Some(Box::new(self.parse_expression()?));
            return Ok(QueryExpression::Logical {
                operator: op,
                left: Box::new(QueryExpression::Condition(condition)),
                right,
            });
        }

        Ok(QueryExpression::Condition(condition))
    }

    fn parse_condition(&mut self) -> Result<FilterCondition, QueryParseError> {
        let field = self.parse_field()?;
        self.skip_whitespace();

        // Handle the colon operator (field:value syntax)
        if self.peek() == Some(':') {
            let _ = self.consume(':');
            self.skip_whitespace();
            let value = self.parse_value()?;
            return Ok(FilterCondition {
                field,
                operator: ComparisonOperator::Equal,
                value,
            });
        }

        let operator = self.parse_comparison_operator()?;
        self.skip_whitespace();

        // For NULL operators, we don't need to parse a value
        // For BETWEEN operators, we need to parse a range
        let value = match operator {
            ComparisonOperator::IsNull | ComparisonOperator::IsNotNull => QueryValue::Null,
            ComparisonOperator::Between | ComparisonOperator::NotBetween => {
                // Parse range: value1 AND value2
                let start_value = self.parse_value()?;
                self.skip_whitespace();
                if self.starts_with("AND") {
                    self.advance_by(3);
                    self.skip_whitespace();
                    let end_value = self.parse_value()?;
                    QueryValue::Range {
                        start: Box::new(start_value),
                        end: Box::new(end_value),
                    }
                } else {
                    return Err(QueryParseError::InvalidValue(
                        "Expected 'AND' after first value in BETWEEN".to_string(),
                    ));
                }
            }
            _ => self.parse_value()?,
        };

        Ok(FilterCondition { field, operator, value })
    }

    fn parse_field(&mut self) -> Result<String, QueryParseError> {
        let start = self.position;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                self.advance();
            } else {
                break;
            }
        }

        if self.position == start {
            return Err(QueryParseError::InvalidField("Empty field name".to_string()));
        }

        let field_name = self.input[start..self.position].to_string();

        // Consumir espaços em branco após o campo
        self.skip_whitespace();

        Ok(field_name)
    }

    fn parse_comparison_operator(&mut self) -> Result<ComparisonOperator, QueryParseError> {
        // Operadores de palavra-chave (devem ser verificados primeiro)
        if self.starts_with("BETWEEN") {
            self.advance_by(7);
            return Ok(ComparisonOperator::Between);
        }

        if self.starts_with("NOT BETWEEN") {
            self.advance_by(11);
            return Ok(ComparisonOperator::NotBetween);
        }

        if self.starts_with("IN") {
            self.advance_by(2);
            return Ok(ComparisonOperator::In);
        }

        if self.starts_with("NOT IN") {
            self.advance_by(6);
            return Ok(ComparisonOperator::NotIn);
        }

        if self.starts_with("IS NULL") {
            self.advance_by(7);
            return Ok(ComparisonOperator::IsNull);
        }

        if self.starts_with("IS NOT NULL") {
            self.advance_by(11);
            return Ok(ComparisonOperator::IsNotNull);
        }

        // Operadores básicos (caracteres individuais)
        if self.peek() == Some('=') {
            let _ = self.consume('=');
            return Ok(ComparisonOperator::Equal);
        }

        if self.peek() == Some('!') {
            let _ = self.consume('!');
            if self.peek() == Some('=') {
                let _ = self.consume('=');
                return Ok(ComparisonOperator::NotEqual);
            }
            if self.peek() == Some('~') {
                let _ = self.consume('~');
                if self.peek() == Some('*') {
                    let _ = self.consume('*');
                    return Ok(ComparisonOperator::NotRegex);
                }
                return Ok(ComparisonOperator::NotContains);
            }
            return Err(QueryParseError::UnsupportedOperator("!".to_string()));
        }

        if self.peek() == Some('>') {
            let _ = self.consume('>');
            if self.peek() == Some('=') {
                let _ = self.consume('=');
                return Ok(ComparisonOperator::GreaterOrEqual);
            }
            return Ok(ComparisonOperator::GreaterThan);
        }

        if self.peek() == Some('<') {
            let _ = self.consume('<');
            if self.peek() == Some('=') {
                let _ = self.consume('=');
                return Ok(ComparisonOperator::LessOrEqual);
            }
            return Ok(ComparisonOperator::LessThan);
        }

        // Operadores de string
        if self.peek() == Some('~') {
            let _ = self.consume('~');
            if self.peek() == Some('*') {
                let _ = self.consume('*');
                return Ok(ComparisonOperator::Regex);
            }
            return Ok(ComparisonOperator::Contains);
        }

        if self.peek() == Some('^') {
            let _ = self.consume('^');
            return Ok(ComparisonOperator::StartsWith);
        }

        if self.peek() == Some('$') {
            let _ = self.consume('$');
            return Ok(ComparisonOperator::EndsWith);
        }

        Err(QueryParseError::UnsupportedOperator(
            self.peek().unwrap_or(' ').to_string(),
        ))
    }

    fn parse_logical_operator(&mut self) -> Option<LogicalOperator> {
        if self.starts_with("AND") {
            self.advance_by(3);
            return Some(LogicalOperator::And);
        }

        if self.starts_with("OR") {
            self.advance_by(2);
            return Some(LogicalOperator::Or);
        }

        None
    }

    fn parse_value(&mut self) -> Result<QueryValue, QueryParseError> {
        self.skip_whitespace();

        // Parse arrays [value1, value2, ...]
        if self.peek() == Some('[') {
            let _ = self.consume('[');
            let mut values = Vec::new();

            self.skip_whitespace();
            while self.peek() != Some(']') {
                let value = self.parse_value()?;
                values.push(value);

                self.skip_whitespace();
                if self.peek() == Some(',') {
                    let _ = self.consume(',');
                    self.skip_whitespace();
                } else if self.peek() != Some(']') {
                    return Err(QueryParseError::InvalidValue(
                        "Expected ',' or ']' in array".to_string(),
                    ));
                }
            }
            self.expect(']')?;
            return Ok(QueryValue::Array(values));
        }

        // Parse boolean values
        if self.starts_with("true") {
            self.advance_by(4);
            return Ok(QueryValue::Boolean(true));
        }

        if self.starts_with("false") {
            self.advance_by(5);
            return Ok(QueryValue::Boolean(false));
        }

        // Parse NULL
        if self.starts_with("NULL") {
            self.advance_by(4);
            return Ok(QueryValue::Null);
        }

        // Try to parse as date (YYYY-MM-DD) first
        if self.position < self.input.len() && self.input.len() - self.position >= 10 {
            let date_str = &self.input[self.position..self.position + 10];
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                self.advance_by(10);
                return Ok(QueryValue::Date(date));
            }
        }

        // Parse quoted strings
        if self.peek() == Some('\'') {
            let _ = self.consume('\'');
            let start = self.position;
            while let Some(c) = self.peek() {
                if c == '\'' {
                    break;
                }
                self.advance();
            }
            self.expect('\'')?;
            return Ok(QueryValue::String(self.input[start..self.position - 1].to_string()));
        }

        // Parse numbers
        let start_position = self.position;
        let mut has_dot = false;
        let mut is_number = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
                is_number = true;
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.advance();
                is_number = true;
            } else {
                break;
            }
        }

        if is_number && self.position > start_position {
            let num_str = &self.input[start_position..self.position];
            if let Ok(num) = num_str.parse::<f64>() {
                return Ok(QueryValue::Number(num));
            }
        }

        // Reset position and parse as unquoted string
        self.position = start_position;
        let start = self.position;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                self.advance();
            } else {
                break;
            }
        }

        if self.position > start {
            Ok(QueryValue::String(self.input[start..self.position].to_string()))
        } else {
            Err(QueryParseError::InvalidValue("Empty value".to_string()))
        }
    }

    fn parse_value_legacy(&mut self) -> Result<QueryValue, QueryParseError> {
        if self.peek() == Some('t') && self.starts_with("true") {
            self.advance_by(4);
            return Ok(QueryValue::Boolean(true));
        }

        if self.peek() == Some('f') && self.starts_with("false") {
            self.advance_by(5);
            return Ok(QueryValue::Boolean(false));
        }

        // Parse NULL
        if self.starts_with("NULL") {
            self.advance_by(4);
            return Ok(QueryValue::Null);
        }

        // Try to parse as date (YYYY-MM-DD) first
        if self.position < self.input.len() && self.input.len() - self.position >= 10 {
            let date_str = &self.input[self.position..self.position + 10];
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                self.advance_by(10);
                return Ok(QueryValue::Date(date));
            }
        }

        // Default to string (unquoted)
        let start = self.position;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                self.advance();
            } else {
                break;
            }
        }

        if self.position > start {
            Ok(QueryValue::String(self.input[start..self.position].to_string()))
        } else {
            Err(QueryParseError::InvalidValue("Empty value".to_string()))
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }

    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn consume(&mut self, expected: char) -> Result<(), QueryParseError> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(QueryParseError::UnexpectedToken(self.peek().unwrap_or(' ').to_string()))
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), QueryParseError> {
        self.consume(expected)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_condition() {
        let mut parser = QueryParser::new("status:active".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Condition(condition) => {
                assert_eq!(condition.field, "status");
                assert_eq!(condition.operator, ComparisonOperator::Equal);
                assert_eq!(condition.value, QueryValue::String("active".to_string()));
            }
            _ => panic!("Expected condition"),
        }
    }

    #[test]
    fn test_parse_quoted_string() {
        let mut parser = QueryParser::new("name:'John Doe'".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Condition(condition) => {
                assert_eq!(condition.field, "name");
                assert_eq!(condition.operator, ComparisonOperator::Equal);
                assert_eq!(condition.value, QueryValue::String("John Doe".to_string()));
            }
            _ => panic!("Expected condition"),
        }
    }

    #[test]
    fn test_parse_logical_and() {
        let mut parser = QueryParser::new("status:active AND priority:high".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Logical {
                operator,
                left: _left,
                right,
            } => {
                assert_eq!(operator, LogicalOperator::And);
                assert!(right.is_some());
            }
            _ => panic!("Expected logical expression"),
        }
    }

    #[test]
    fn test_parse_boolean_value() {
        let mut parser = QueryParser::new("active:true".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Condition(condition) => {
                assert_eq!(condition.field, "active");
                assert_eq!(condition.value, QueryValue::Boolean(true));
            }
            _ => panic!("Expected condition"),
        }
    }

    #[test]
    fn test_parse_number_value() {
        let mut parser = QueryParser::new("progress:75.5".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Condition(condition) => {
                assert_eq!(condition.field, "progress");
                assert_eq!(condition.value, QueryValue::Number(75.5));
            }
            _ => panic!("Expected condition"),
        }
    }

    #[test]
    fn test_parse_date_value() {
        let mut parser = QueryParser::new("created:2024-01-15".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Condition(condition) => {
                assert_eq!(condition.field, "created");
                if let QueryValue::Date(date) = condition.value {
                    assert_eq!(date.format("%Y-%m-%d").to_string(), "2024-01-15");
                } else {
                    panic!("Expected date value");
                }
            }
            _ => panic!("Expected condition"),
        }
    }

    #[test]
    fn test_parse_comparison_operators() {
        let test_cases = vec![
            ("age > 18", ComparisonOperator::GreaterThan),
            ("age >= 18", ComparisonOperator::GreaterOrEqual),
            ("age < 65", ComparisonOperator::LessThan),
            ("age <= 65", ComparisonOperator::LessOrEqual),
            ("age != 0", ComparisonOperator::NotEqual),
            ("name ~ 'John'", ComparisonOperator::Contains),
        ];

        for (input, expected_op) in test_cases {
            let mut parser = QueryParser::new(input.to_string());
            let query = parser.parse().unwrap();

            match query.expression {
                QueryExpression::Condition(condition) => {
                    assert_eq!(condition.operator, expected_op);
                }
                _ => panic!("Expected condition for input: {}", input),
            }
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let mut parser = QueryParser::new("(status:active OR status:pending) AND priority:high".to_string());
        let query = parser.parse().unwrap();

        // Should parse without error
        assert!(!matches!(query.expression, QueryExpression::Condition(_)));
    }

    #[test]
    fn test_parse_not_expression() {
        let mut parser = QueryParser::new("NOT status:cancelled".to_string());
        let query = parser.parse().unwrap();

        match query.expression {
            QueryExpression::Not(_) => {}
            _ => panic!("Expected NOT expression"),
        }
    }

    #[test]
    fn test_between_operator_parsing_issue() {
        // Este teste demonstra a correção do problema com o operador BETWEEN
        // O problema estava na ordem de parsing dos operadores
        let mut parser = QueryParser::new("age BETWEEN 20 AND 30".to_string());

        // Agora o teste deve passar porque o parser consegue identificar corretamente
        // o operador BETWEEN quando usado diretamente na string de query
        let result = parser.parse();

        // Esperamos que seja bem-sucedido
        match result {
            Ok(query) => {
                // Verificar que a query foi parseada corretamente
                if let QueryExpression::Condition(condition) = &query.expression {
                    assert_eq!(condition.field, "age");
                    assert_eq!(condition.operator, ComparisonOperator::Between);
                    // Para BETWEEN, o valor deve ser um Range
                    if let QueryValue::Range { start, end } = &condition.value {
                        if let QueryValue::Number(start_val) = start.as_ref() {
                            assert_eq!(*start_val, 20.0);
                        } else {
                            panic!("Start value should be a number");
                        }
                        if let QueryValue::Number(end_val) = end.as_ref() {
                            assert_eq!(*end_val, 30.0);
                        } else {
                            panic!("End value should be a number");
                        }
                    } else {
                        panic!("BETWEEN should produce a Range value");
                    }
                } else {
                    panic!("Expected a condition expression");
                }
                println!("✅ Operador BETWEEN parseado com sucesso!");
            }
            Err(e) => {
                panic!("Erro inesperado ao parsear BETWEEN: {:?}", e);
            }
        }
    }

    #[test]
    fn test_between_operator_manual_construction() {
        // Este teste mostra como o BETWEEN deveria funcionar quando construído manualmente
        let condition = FilterCondition {
            field: "age".to_string(),
            operator: ComparisonOperator::Between,
            value: QueryValue::Range {
                start: Box::new(QueryValue::Number(20.0)),
                end: Box::new(QueryValue::Number(30.0)),
            },
        };

        let query = Query {
            expression: QueryExpression::Condition(condition),
            aggregation: None,
            sort: None,
            pagination: PaginationOptions::new_default(),
            projection: ProjectionOptions::include_all_fields(),
        };

        // Verificar que a condição foi construída corretamente
        match query.expression {
            QueryExpression::Condition(cond) => {
                assert_eq!(cond.field, "age");
                assert_eq!(cond.operator, ComparisonOperator::Between);
                match cond.value {
                    QueryValue::Range { start, end } => {
                        assert_eq!(*start, QueryValue::Number(20.0));
                        assert_eq!(*end, QueryValue::Number(30.0));
                    }
                    _ => panic!("Esperado Range value"),
                }
            }
            _ => panic!("Esperado condition"),
        }
    }
}
