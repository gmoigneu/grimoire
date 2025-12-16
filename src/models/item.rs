use super::Category;
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Parse SQLite datetime format (YYYY-MM-DD HH:MM:SS) to DateTime<Utc>
fn parse_sqlite_datetime(s: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: Option<i64>,
    pub name: String,
    pub category: Category,
    pub description: Option<String>,
    pub content: String,

    // Category-specific fields
    pub model: Option<String>,
    pub tools: Option<String>,
    pub allowed_tools: Option<String>,
    pub argument_hint: Option<String>,
    pub permission_mode: Option<String>,
    pub skills: Option<String>,

    pub tags: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Item {
    pub fn new(name: String, category: Category, content: String) -> Self {
        Self {
            id: None,
            name,
            category,
            description: None,
            content,
            model: None,
            tools: None,
            allowed_tools: None,
            argument_hint: None,
            permission_mode: None,
            skills: None,
            tags: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let category_str: String = row.get(2)?;
        let created_str: Option<String> = row.get(12)?;
        let updated_str: Option<String> = row.get(13)?;

        Ok(Self {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            category: Category::from_str(&category_str),
            description: row.get(3)?,
            content: row.get(4)?,
            model: row.get(5)?,
            tools: row.get(6)?,
            allowed_tools: row.get(7)?,
            argument_hint: row.get(8)?,
            permission_mode: row.get(9)?,
            skills: row.get(10)?,
            tags: row.get(11)?,
            created_at: created_str.and_then(|s| parse_sqlite_datetime(&s)),
            updated_at: updated_str.and_then(|s| parse_sqlite_datetime(&s)),
        })
    }

    /// Validate the item based on its category requirements
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required fields based on category
        if self.name.trim().is_empty() {
            errors.push("Name is required".to_string());
        }

        if self.content.trim().is_empty() {
            errors.push("Content is required".to_string());
        }

        // Category-specific validation
        match self.category {
            Category::Agent | Category::Skill => {
                if self.description.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                    errors.push("Description is required for this category".to_string());
                }
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get tags as a vector
    #[allow(dead_code)]
    pub fn tags_vec(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Format the updated_at time as a relative string
    pub fn updated_ago(&self) -> String {
        match self.updated_at {
            Some(dt) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(dt);

                if duration.num_days() > 7 {
                    format!("{} weeks ago", duration.num_weeks())
                } else if duration.num_days() > 0 {
                    let days = duration.num_days();
                    if days == 1 {
                        "1 day ago".to_string()
                    } else {
                        format!("{} days ago", days)
                    }
                } else if duration.num_hours() > 0 {
                    let hours = duration.num_hours();
                    if hours == 1 {
                        "1 hour ago".to_string()
                    } else {
                        format!("{} hours ago", hours)
                    }
                } else if duration.num_minutes() > 0 {
                    let mins = duration.num_minutes();
                    if mins == 1 {
                        "1 minute ago".to_string()
                    } else {
                        format!("{} minutes ago", mins)
                    }
                } else {
                    "just now".to_string()
                }
            }
            None => "unknown".to_string(),
        }
    }
}

impl Default for Item {
    fn default() -> Self {
        Self::new(String::new(), Category::Prompt, String::new())
    }
}
