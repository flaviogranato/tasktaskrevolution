#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the priority level of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Priority {
    /// Low priority - can be done when there's time
    Low,
    /// Medium priority - normal priority
    #[default]
    Medium,
    /// High priority - should be done soon
    High,
    /// Critical priority - must be done immediately
    Critical,
}

impl Priority {
    /// Returns the numeric value of the priority (higher = more important)
    pub fn value(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }

    /// Returns the display name of the priority
    pub fn display_name(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }

    /// Returns the color code for the priority (for CLI display)
    pub fn color_code(&self) -> &'static str {
        match self {
            Priority::Low => "32",      // Green
            Priority::Medium => "33",   // Yellow
            Priority::High => "35",     // Magenta
            Priority::Critical => "31", // Red
        }
    }

    /// Returns the icon for the priority
    pub fn icon(&self) -> &'static str {
        match self {
            Priority::Low => "🟢",
            Priority::Medium => "🟡",
            Priority::High => "🟣",
            Priority::Critical => "🔴",
        }
    }

    /// Creates a priority from a string
    pub fn parse_priority(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Ok(Priority::Low),
            "medium" | "med" | "m" => Ok(Priority::Medium),
            "high" | "h" => Ok(Priority::High),
            "critical" | "crit" | "c" => Ok(Priority::Critical),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }

    /// Returns all available priorities in order of importance
    pub fn all() -> Vec<Priority> {
        vec![Priority::Low, Priority::Medium, Priority::High, Priority::Critical]
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_values() {
        assert_eq!(Priority::Low.value(), 1);
        assert_eq!(Priority::Medium.value(), 2);
        assert_eq!(Priority::High.value(), 3);
        assert_eq!(Priority::Critical.value(), 4);
    }

    #[test]
    fn test_priority_display_names() {
        assert_eq!(Priority::Low.display_name(), "Low");
        assert_eq!(Priority::Medium.display_name(), "Medium");
        assert_eq!(Priority::High.display_name(), "High");
        assert_eq!(Priority::Critical.display_name(), "Critical");
    }

    #[test]
    fn test_priority_color_codes() {
        assert_eq!(Priority::Low.color_code(), "32");
        assert_eq!(Priority::Medium.color_code(), "33");
        assert_eq!(Priority::High.color_code(), "35");
        assert_eq!(Priority::Critical.color_code(), "31");
    }

    #[test]
    fn test_priority_icons() {
        assert_eq!(Priority::Low.icon(), "🟢");
        assert_eq!(Priority::Medium.icon(), "🟡");
        assert_eq!(Priority::High.icon(), "🟣");
        assert_eq!(Priority::Critical.icon(), "🔴");
    }

    #[test]
    fn test_priority_parse_priority() {
        assert_eq!(Priority::parse_priority("low").unwrap(), Priority::Low);
        assert_eq!(Priority::parse_priority("L").unwrap(), Priority::Low);
        assert_eq!(Priority::parse_priority("medium").unwrap(), Priority::Medium);
        assert_eq!(Priority::parse_priority("MED").unwrap(), Priority::Medium);
        assert_eq!(Priority::parse_priority("m").unwrap(), Priority::Medium);
        assert_eq!(Priority::parse_priority("high").unwrap(), Priority::High);
        assert_eq!(Priority::parse_priority("H").unwrap(), Priority::High);
        assert_eq!(Priority::parse_priority("critical").unwrap(), Priority::Critical);
        assert_eq!(Priority::parse_priority("CRIT").unwrap(), Priority::Critical);
        assert_eq!(Priority::parse_priority("c").unwrap(), Priority::Critical);
    }

    #[test]
    fn test_priority_parse_priority_invalid() {
        assert!(Priority::parse_priority("invalid").is_err());
        assert!(Priority::parse_priority("").is_err());
        assert!(Priority::parse_priority("x").is_err());
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", Priority::Low), "Low");
        assert_eq!(format!("{}", Priority::Medium), "Medium");
        assert_eq!(format!("{}", Priority::High), "High");
        assert_eq!(format!("{}", Priority::Critical), "Critical");
    }

    #[test]
    fn test_priority_default() {
        assert_eq!(Priority::default(), Priority::Medium);
    }

    #[test]
    fn test_priority_all() {
        let all = Priority::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], Priority::Low);
        assert_eq!(all[1], Priority::Medium);
        assert_eq!(all[2], Priority::High);
        assert_eq!(all[3], Priority::Critical);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low.value() < Priority::Medium.value());
        assert!(Priority::Medium.value() < Priority::High.value());
        assert!(Priority::High.value() < Priority::Critical.value());
    }

    #[test]
    fn test_priority_serialization() {
        let priority = Priority::High;
        let serialized = serde_yaml::to_string(&priority).unwrap();
        assert_eq!(serialized.trim(), "High");

        let deserialized: Priority = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Priority::High);
    }

    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::Low, Priority::Low);
        assert_ne!(Priority::Low, Priority::High);
    }

    #[test]
    fn test_priority_clone() {
        let priority = Priority::Critical;
        let cloned = priority;
        assert_eq!(priority, cloned);
    }
}
