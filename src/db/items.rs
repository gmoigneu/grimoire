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
                   tags, created_at, updated_at
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
                   tags, created_at, updated_at
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
                   tags, created_at, updated_at
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

    #[allow(dead_code)]
    pub fn get(&self, id: i64) -> Result<Option<Item>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, category, description, content, model, tools,
                   allowed_tools, argument_hint, permission_mode, skills,
                   tags, created_at, updated_at
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
                              allowed_tools, argument_hint, permission_mode, skills, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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

    pub fn update(&self, item: &Item) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE items
            SET name = ?, category = ?, description = ?, content = ?, model = ?,
                tools = ?, allowed_tools = ?, argument_hint = ?, permission_mode = ?,
                skills = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
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
                item.id,
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
                   i.tags, i.created_at, i.updated_at
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
