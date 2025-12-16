use crate::models::{Category, Item};
use color_eyre::eyre::{eyre, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ClaudeExporter {
    base_path: PathBuf,
}

impl ClaudeExporter {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        let path = base_path.as_ref();
        // Expand ~ to home directory
        let expanded = if path.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                home.join(path.strip_prefix("~").unwrap_or(path))
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        Self {
            base_path: expanded,
        }
    }

    pub fn export(&self, item: &Item) -> Result<PathBuf> {
        match item.category {
            Category::Agent => self.export_agent(item),
            Category::Command => self.export_command(item),
            Category::Skill => self.export_skill(item),
            Category::Prompt => Err(eyre!("Prompts cannot be exported (copy-only)")),
        }
    }

    fn export_agent(&self, item: &Item) -> Result<PathBuf> {
        let dir = self.base_path.join("agents");
        fs::create_dir_all(&dir)?;

        let file_path = dir.join(format!("{}.md", item.name));
        let content = self.format_agent(item);

        fs::write(&file_path, content)?;
        Ok(file_path)
    }

    fn export_command(&self, item: &Item) -> Result<PathBuf> {
        let dir = self.base_path.join("commands");
        fs::create_dir_all(&dir)?;

        let file_path = dir.join(format!("{}.md", item.name));
        let content = self.format_command(item);

        fs::write(&file_path, content)?;
        Ok(file_path)
    }

    fn export_skill(&self, item: &Item) -> Result<PathBuf> {
        let dir = self.base_path.join("skills").join(&item.name);
        fs::create_dir_all(&dir)?;

        let file_path = dir.join("SKILL.md");
        let content = self.format_skill(item);

        fs::write(&file_path, content)?;
        Ok(file_path)
    }

    fn format_agent(&self, item: &Item) -> String {
        let mut frontmatter = vec![format!("name: {}", item.name)];

        if let Some(ref desc) = item.description {
            frontmatter.push(format!("description: {}", desc));
        }

        if let Some(ref tools) = item.tools {
            frontmatter.push(format!("tools: {}", tools));
        }

        if let Some(ref model) = item.model {
            frontmatter.push(format!("model: {}", model));
        }

        if let Some(ref perm) = item.permission_mode {
            frontmatter.push(format!("permissionMode: {}", perm));
        }

        if let Some(ref skills) = item.skills {
            frontmatter.push(format!("skills: {}", skills));
        }

        format!("---\n{}\n---\n\n{}", frontmatter.join("\n"), item.content)
    }

    fn format_command(&self, item: &Item) -> String {
        let mut frontmatter = Vec::new();

        if let Some(ref desc) = item.description {
            frontmatter.push(format!("description: {}", desc));
        }

        if let Some(ref tools) = item.allowed_tools {
            frontmatter.push(format!("allowed-tools: {}", tools));
        }

        if let Some(ref hint) = item.argument_hint {
            frontmatter.push(format!("argument-hint: {}", hint));
        }

        if let Some(ref model) = item.model {
            frontmatter.push(format!("model: {}", model));
        }

        if frontmatter.is_empty() {
            item.content.clone()
        } else {
            format!("---\n{}\n---\n\n{}", frontmatter.join("\n"), item.content)
        }
    }

    fn format_skill(&self, item: &Item) -> String {
        let mut frontmatter = vec![format!("name: {}", item.name)];

        if let Some(ref desc) = item.description {
            frontmatter.push(format!("description: {}", desc));
        }

        if let Some(ref tools) = item.allowed_tools {
            frontmatter.push(format!("allowed-tools: {}", tools));
        }

        format!("---\n{}\n---\n\n{}", frontmatter.join("\n"), item.content)
    }
}

// Helper to get home directory
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}
