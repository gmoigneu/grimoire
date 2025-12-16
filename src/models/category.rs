use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Prompt,
    Agent,
    Skill,
    Command,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Prompt => "prompt",
            Category::Agent => "agent",
            Category::Skill => "skill",
            Category::Command => "command",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::Prompt => "Prompts",
            Category::Agent => "Agents",
            Category::Skill => "Skills",
            Category::Command => "Commands",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "prompt" => Category::Prompt,
            "agent" => Category::Agent,
            "skill" => Category::Skill,
            "command" => Category::Command,
            _ => Category::Prompt, // Default fallback
        }
    }

    pub fn all() -> [Category; 4] {
        [
            Category::Prompt,
            Category::Agent,
            Category::Skill,
            Category::Command,
        ]
    }

    /// Returns the required fields for this category
    #[allow(dead_code)]
    pub fn required_fields(&self) -> &'static [&'static str] {
        match self {
            Category::Prompt => &["name", "content"],
            Category::Agent => &["name", "description", "content"],
            Category::Skill => &["name", "description", "content"],
            Category::Command => &["name", "content"],
        }
    }

    /// Returns the optional fields for this category
    #[allow(dead_code)]
    pub fn optional_fields(&self) -> &'static [&'static str] {
        match self {
            Category::Prompt => &["description", "tags"],
            Category::Agent => &["model", "tools", "permission_mode", "skills", "tags"],
            Category::Skill => &["allowed_tools", "tags"],
            Category::Command => &[
                "description",
                "allowed_tools",
                "argument_hint",
                "model",
                "tags",
            ],
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
