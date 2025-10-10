use serde::{Deserialize, Serialize};
use std::fmt;

/// Representa um operador de comparação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,          // =
    NotEqual,       // !=
    GreaterThan,    // >
    LessThan,       // <
    GreaterOrEqual, // >=
    LessOrEqual,    // <=
    Contains,       // ~ (contém)
    NotContains,    // !~ (não contém)
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOperator::Equal => write!(f, "="),
            ComparisonOperator::NotEqual => write!(f, "!="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::GreaterOrEqual => write!(f, ">="),
            ComparisonOperator::LessOrEqual => write!(f, "<="),
            ComparisonOperator::Contains => write!(f, "~"),
            ComparisonOperator::NotContains => write!(f, "!~"),
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
}

impl fmt::Display for QueryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryValue::String(s) => write!(f, "'{}'", s),
            QueryValue::Number(n) => write!(f, "{}", n),
            QueryValue::Boolean(b) => write!(f, "{}", b),
            QueryValue::Date(d) => write!(f, "{}", d.format("%Y-%m-%d")),
            QueryValue::DateTime(dt) => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
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
    Sum(String),    // field name
    Average(String), // field name
    Min(String),    // field name
    Max(String),    // field name
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

impl PaginationOptions {
    pub fn new(limit: Option<usize>, offset: Option<usize>) -> Self {
        Self { limit, offset }
    }
    
    pub fn default() -> Self {
        Self {
            limit: None,
            offset: None,
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
            pagination: PaginationOptions::default(),
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
            pagination: PaginationOptions::default(),
            sort: None,
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

        let value = self.parse_value()?;

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

        Ok(self.input[start..self.position].to_string())
    }

    fn parse_comparison_operator(&mut self) -> Result<ComparisonOperator, QueryParseError> {
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

        if self.peek() == Some('~') {
            let _ = self.consume('~');
            return Ok(ComparisonOperator::Contains);
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

        if self.peek() == Some('t') && self.starts_with("true") {
            self.advance_by(4);
            return Ok(QueryValue::Boolean(true));
        }

        if self.peek() == Some('f') && self.starts_with("false") {
            self.advance_by(5);
            return Ok(QueryValue::Boolean(false));
        }

        // Try to parse as date (YYYY-MM-DD) first
        if self.position < self.input.len() && self.input.len() - self.position >= 10 {
            let date_str = &self.input[self.position..self.position + 10];
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                self.advance_by(10);
                return Ok(QueryValue::Date(date));
            }
        }

        // Try to parse as number
        let start = self.position;
        let mut has_dot = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        if self.position > start {
            let num_str = &self.input[start..self.position];
            if let Ok(num) = num_str.parse::<f64>() {
                return Ok(QueryValue::Number(num));
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
}
