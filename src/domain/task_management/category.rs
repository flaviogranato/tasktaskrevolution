use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents different categories for tasks with visual indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Category {
    /// Development tasks - Code, features, bugs
    #[default]
    Development,
    /// Testing tasks - Unit tests, integration tests, QA
    Testing,
    /// Documentation tasks - README, API docs, user guides
    Documentation,
    /// Design tasks - UI/UX, mockups, wireframes
    Design,
    /// Infrastructure tasks - DevOps, deployment, monitoring
    Infrastructure,
    /// Research tasks - Investigation, analysis, planning
    Research,
    /// Maintenance tasks - Refactoring, cleanup, optimization
    Maintenance,
    /// Meeting tasks - Standups, reviews, planning sessions
    Meeting,
    /// Review tasks - Code review, design review, testing
    Review,
    /// Other tasks - Miscellaneous, undefined
    Other,
}



impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Development => write!(f, "Development"),
            Category::Testing => write!(f, "Testing"),
            Category::Documentation => write!(f, "Documentation"),
            Category::Design => write!(f, "Design"),
            Category::Infrastructure => write!(f, "Infrastructure"),
            Category::Research => write!(f, "Research"),
            Category::Maintenance => write!(f, "Maintenance"),
            Category::Meeting => write!(f, "Meeting"),
            Category::Review => write!(f, "Review"),
            Category::Other => write!(f, "Other"),
        }
    }
}

impl Category {
    /// Returns a short display name for the category
    #[allow(dead_code)]
    pub fn short_name(&self) -> &'static str {
        match self {
            Category::Development => "DEV",
            Category::Testing => "TEST",
            Category::Documentation => "DOC",
            Category::Design => "DESIGN",
            Category::Infrastructure => "INFRA",
            Category::Research => "RESEARCH",
            Category::Maintenance => "MAINT",
            Category::Meeting => "MEET",
            Category::Review => "REVIEW",
            Category::Other => "OTHER",
        }
    }

    /// Returns an emoji icon for the category
    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            Category::Development => "💻",
            Category::Testing => "🧪",
            Category::Documentation => "📚",
            Category::Design => "🎨",
            Category::Infrastructure => "🏗️",
            Category::Research => "🔍",
            Category::Maintenance => "🔧",
            Category::Meeting => "👥",
            Category::Review => "👀",
            Category::Other => "📋",
        }
    }

    /// Returns a color code for the category (ANSI color codes)
    #[allow(dead_code)]
    pub fn color_code(&self) -> &'static str {
        match self {
            Category::Development => "\x1b[34m",    // Blue
            Category::Testing => "\x1b[32m",        // Green
            Category::Documentation => "\x1b[33m",  // Yellow
            Category::Design => "\x1b[35m",         // Magenta
            Category::Infrastructure => "\x1b[36m", // Cyan
            Category::Research => "\x1b[31m",       // Red
            Category::Maintenance => "\x1b[37m",    // White
            Category::Meeting => "\x1b[93m",        // Bright Yellow
            Category::Review => "\x1b[94m",         // Bright Blue
            Category::Other => "\x1b[90m",          // Dark Gray
        }
    }

    /// Returns the reset color code
    #[allow(dead_code)]
    pub fn reset_color() -> &'static str {
        "\x1b[0m"
    }

    /// Returns a colored display string
    #[allow(dead_code)]
    pub fn colored_display(&self) -> String {
        format!("{}{}{}", self.color_code(), self, Self::reset_color())
    }

    /// Returns a colored display with icon
    #[allow(dead_code)]
    pub fn colored_with_icon(&self) -> String {
        format!("{} {}{}{}", self.icon(), self.color_code(), self, Self::reset_color())
    }

    /// Returns all available categories
    #[allow(dead_code)]
    pub fn all() -> Vec<Category> {
        vec![
            Category::Development,
            Category::Testing,
            Category::Documentation,
            Category::Design,
            Category::Infrastructure,
            Category::Research,
            Category::Maintenance,
            Category::Meeting,
            Category::Review,
            Category::Other,
        ]
    }

    /// Returns categories grouped by type
    #[allow(dead_code)]
    pub fn by_type() -> std::collections::HashMap<&'static str, Vec<Category>> {
        let mut groups = std::collections::HashMap::new();
        groups.insert(
            "Technical",
            vec![
                Category::Development,
                Category::Testing,
                Category::Infrastructure,
                Category::Maintenance,
            ],
        );
        groups.insert("Creative", vec![Category::Design, Category::Documentation]);
        groups.insert("Process", vec![Category::Research, Category::Meeting, Category::Review]);
        groups.insert("General", vec![Category::Other]);
        groups
    }
}

impl std::str::FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Category::Development),
            "testing" | "test" => Ok(Category::Testing),
            "documentation" | "doc" => Ok(Category::Documentation),
            "design" => Ok(Category::Design),
            "infrastructure" | "infra" => Ok(Category::Infrastructure),
            "research" => Ok(Category::Research),
            "maintenance" | "maint" => Ok(Category::Maintenance),
            "meeting" | "meet" => Ok(Category::Meeting),
            "review" => Ok(Category::Review),
            "other" => Ok(Category::Other),
            _ => Err(format!("Invalid category: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_default() {
        assert_eq!(Category::default(), Category::Development);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", Category::Development), "Development");
        assert_eq!(format!("{}", Category::Testing), "Testing");
        assert_eq!(format!("{}", Category::Other), "Other");
    }

    #[test]
    fn test_category_short_name() {
        assert_eq!(Category::Development.short_name(), "DEV");
        assert_eq!(Category::Testing.short_name(), "TEST");
        assert_eq!(Category::Other.short_name(), "OTHER");
    }

    #[test]
    fn test_category_icon() {
        assert_eq!(Category::Development.icon(), "💻");
        assert_eq!(Category::Testing.icon(), "🧪");
        assert_eq!(Category::Other.icon(), "📋");
    }

    #[test]
    fn test_category_color_code() {
        assert_eq!(Category::Development.color_code(), "\x1b[34m");
        assert_eq!(Category::Testing.color_code(), "\x1b[32m");
        assert_eq!(Category::Other.color_code(), "\x1b[90m");
    }

    #[test]
    fn test_category_colored_display() {
        let dev = Category::Development.colored_display();
        assert!(dev.contains("Development"));
        assert!(dev.contains("\x1b[34m")); // Blue color
        assert!(dev.contains("\x1b[0m")); // Reset color
    }

    #[test]
    fn test_category_colored_with_icon() {
        let dev = Category::Development.colored_with_icon();
        assert!(dev.contains("💻"));
        assert!(dev.contains("Development"));
        assert!(dev.contains("\x1b[34m")); // Blue color
        assert!(dev.contains("\x1b[0m")); // Reset color
    }

    #[test]
    fn test_category_all() {
        let all = Category::all();
        assert_eq!(all.len(), 10);
        assert!(all.contains(&Category::Development));
        assert!(all.contains(&Category::Other));
    }

    #[test]
    fn test_category_by_type() {
        let by_type = Category::by_type();
        assert!(by_type.contains_key("Technical"));
        assert!(by_type.contains_key("Creative"));
        assert!(by_type.contains_key("Process"));
        assert!(by_type.contains_key("General"));

        let technical = by_type.get("Technical").unwrap();
        assert!(technical.contains(&Category::Development));
        assert!(technical.contains(&Category::Testing));
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!("development".parse::<Category>().unwrap(), Category::Development);
        assert_eq!("dev".parse::<Category>().unwrap(), Category::Development);
        assert_eq!("testing".parse::<Category>().unwrap(), Category::Testing);
        assert_eq!("test".parse::<Category>().unwrap(), Category::Testing);
        assert_eq!("other".parse::<Category>().unwrap(), Category::Other);

        assert!("invalid".parse::<Category>().is_err());
    }

    #[test]
    fn test_category_serialization() {
        let category = Category::Development;
        let serialized = serde_yaml::to_string(&category).unwrap();
        assert!(serialized.contains("Development"));

        let deserialized: Category = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, category);
    }

    #[test]
    fn test_category_equality() {
        assert_eq!(Category::Development, Category::Development);
        assert_ne!(Category::Development, Category::Testing);
    }

    #[test]
    fn test_category_clone() {
        let category = Category::Development;
        let cloned = category;
        assert_eq!(category, cloned);
    }
}
