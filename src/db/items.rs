use crate::models::{Category, Item};
use color_eyre::eyre::Result;
use rusqlite::{params, Connection};

pub struct ItemStore<'a> {
    conn: &'a Connection,
}

impl<'a> ItemStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, category, description, content, model, tools,
                   allowed_tools, argument_hint, permission_mode, skills,
                   tags, created_at, updated_at, version
            FROM items
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )?;

        let items = stmt
            .query_map([limit], Item::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn list_by_category(&self, category: Category) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, category, description, content, model, tools,
                   allowed_tools, argument_hint, permission_mode, skills,
                   tags, created_at, updated_at, version
            FROM items
            WHERE category = ?
            ORDER BY updated_at DESC
            "#,
        )?;

        let items = stmt
            .query_map([category.as_str()], Item::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn list_by_tag(&self, tag: &str) -> Result<Vec<Item>> {
        let pattern = format!("%{}%", tag);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, category, description, content, model, tools,
                   allowed_tools, argument_hint, permission_mode, skills,
                   tags, created_at, updated_at, version
            FROM items
            WHERE tags LIKE ?
            ORDER BY updated_at DESC
            "#,
        )?;

        let items = stmt
            .query_map([pattern], Item::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn get(&self, id: i64) -> Result<Option<Item>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, category, description, content, model, tools,
                   allowed_tools, argument_hint, permission_mode, skills,
                   tags, created_at, updated_at, version
            FROM items
            WHERE id = ?
            "#,
        )?;

        let item = stmt.query_row([id], Item::from_row).optional()?;
        Ok(item)
    }

    pub fn insert(&self, item: &Item) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO items (name, category, description, content, model, tools,
                              allowed_tools, argument_hint, permission_mode, skills, tags, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            "#,
            params![
                item.name,
                item.category.as_str(),
                item.description,
                item.content,
                item.model,
                item.tools,
                item.allowed_tools,
                item.argument_hint,
                item.permission_mode,
                item.skills,
                item.tags,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Update an item, creating a version snapshot of the current state first
    pub fn update(&self, item: &Item) -> Result<()> {
        let item_id = item.id.ok_or_else(|| color_eyre::eyre::eyre!("Item must have an id to update"))?;

        // Get current item to save as version
        if let Some(current) = self.get(item_id)? {
            // Save current state to item_versions
            self.conn.execute(
                r#"
                INSERT INTO item_versions (item_id, version, name, category, description, content,
                                          model, tools, allowed_tools, argument_hint,
                                          permission_mode, skills, tags)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    item_id,
                    current.version,
                    current.name,
                    current.category.as_str(),
                    current.description,
                    current.content,
                    current.model,
                    current.tools,
                    current.allowed_tools,
                    current.argument_hint,
                    current.permission_mode,
                    current.skills,
                    current.tags,
                ],
            )?;
        }

        // Update item with incremented version
        self.conn.execute(
            r#"
            UPDATE items
            SET name = ?, category = ?, description = ?, content = ?, model = ?,
                tools = ?, allowed_tools = ?, argument_hint = ?, permission_mode = ?,
                skills = ?, tags = ?, updated_at = CURRENT_TIMESTAMP,
                version = version + 1
            WHERE id = ?
            "#,
            params![
                item.name,
                item.category.as_str(),
                item.description,
                item.content,
                item.model,
                item.tools,
                item.allowed_tools,
                item.argument_hint,
                item.permission_mode,
                item.skills,
                item.tags,
                item_id,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM items WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT i.id, i.name, i.category, i.description, i.content, i.model, i.tools,
                   i.allowed_tools, i.argument_hint, i.permission_mode, i.skills,
                   i.tags, i.created_at, i.updated_at, i.version
            FROM items i
            JOIN items_fts fts ON i.id = fts.rowid
            WHERE items_fts MATCH ?
            ORDER BY rank
            "#,
        )?;

        let items = stmt
            .query_map([query], Item::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn count_by_category(&self) -> Result<Vec<(Category, usize)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT category, COUNT(*) as count
            FROM items
            GROUP BY category
            "#,
        )?;

        let counts = stmt
            .query_map([], |row| {
                let cat_str: String = row.get(0)?;
                let count: usize = row.get(1)?;
                Ok((Category::from_str(&cat_str), count))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(counts)
    }

    pub fn get_tags_with_counts(&self) -> Result<Vec<(String, usize)>> {
        // This is a simplified implementation - tags are comma-separated
        // A production version might use a separate tags table
        let mut stmt = self.conn.prepare("SELECT tags FROM items WHERE tags IS NOT NULL")?;

        let mut tag_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        let rows = stmt.query_map([], |row| {
            let tags: String = row.get(0)?;
            Ok(tags)
        })?;

        for tags in rows.flatten() {
            for tag in tags.split(',') {
                let tag = tag.trim().to_lowercase();
                if !tag.is_empty() {
                    *tag_counts.entry(tag).or_insert(0) += 1;
                }
            }
        }

        let mut tags: Vec<_> = tag_counts.into_iter().collect();
        // Sort by count descending, then by name ascending for stable ordering
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        Ok(tags)
    }

    /// List all versions of an item (version number and created_at)
    pub fn list_versions(&self, item_id: i64) -> Result<Vec<ItemVersion>> {
        // First get the current version from items table
        let current: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT version, updated_at FROM items WHERE id = ?",
                [item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let mut versions = Vec::new();

        // Add current version (latest)
        if let Some((version, updated_at)) = current {
            versions.push(ItemVersion {
                version,
                created_at: updated_at,
                is_current: true,
            });
        }

        // Get historical versions from item_versions table
        let mut stmt = self.conn.prepare(
            r#"
            SELECT version, created_at
            FROM item_versions
            WHERE item_id = ?
            ORDER BY version DESC
            "#,
        )?;

        let historical = stmt.query_map([item_id], |row| {
            Ok(ItemVersion {
                version: row.get(0)?,
                created_at: row.get(1)?,
                is_current: false,
            })
        })?;

        for v in historical.flatten() {
            versions.push(v);
        }

        Ok(versions)
    }

    /// Get a specific version of an item
    pub fn get_version(&self, item_id: i64, version: i64) -> Result<Option<Item>> {
        // First check if this is the current version
        let current = self.get(item_id)?;
        if let Some(ref item) = current {
            if item.version == version {
                return Ok(current);
            }
        }

        // Otherwise get from item_versions
        let mut stmt = self.conn.prepare(
            r#"
            SELECT item_id, name, category, description, content, model, tools,
                   allowed_tools, argument_hint, permission_mode, skills,
                   tags, created_at, created_at, version
            FROM item_versions
            WHERE item_id = ? AND version = ?
            "#,
        )?;

        let item = stmt
            .query_row(params![item_id, version], |row| {
                let category_str: String = row.get(2)?;
                let created_str: Option<String> = row.get(12)?;
                let version: i64 = row.get(14)?;

                Ok(Item {
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
                    created_at: created_str
                        .as_ref()
                        .and_then(|s| parse_sqlite_datetime(s)),
                    updated_at: created_str.and_then(|s| parse_sqlite_datetime(&s)),
                    version,
                })
            })
            .optional()?;

        Ok(item)
    }

    /// Restore an item to a specific version (creates a new version with the old content)
    pub fn restore_version(&self, item_id: i64, version: i64) -> Result<()> {
        // Get the version to restore
        let old_version = self
            .get_version(item_id, version)?
            .ok_or_else(|| color_eyre::eyre::eyre!("Version not found"))?;

        // Update the item with the old content (this will auto-increment version)
        self.update(&old_version)?;

        Ok(())
    }
}

/// Represents a version entry for the history list
#[derive(Debug, Clone)]
pub struct ItemVersion {
    pub version: i64,
    pub created_at: String,
    pub is_current: bool,
}

/// Parse SQLite datetime format (YYYY-MM-DD HH:MM:SS) to DateTime<Utc>
fn parse_sqlite_datetime(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

#[allow(dead_code)]
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
