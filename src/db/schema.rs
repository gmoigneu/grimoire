use color_eyre::eyre::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn db_path() -> Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("", "", "grimoire")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine home directory"))?;

        Ok(proj_dirs.data_dir().join("grimoire.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Items table
            CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL CHECK(category IN ('prompt', 'agent', 'skill', 'command')),
                description TEXT,
                content TEXT NOT NULL,

                -- Category-specific fields
                model TEXT,
                tools TEXT,
                allowed_tools TEXT,
                argument_hint TEXT,
                permission_mode TEXT,
                skills TEXT,

                tags TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_items_category ON items(category);
            CREATE INDEX IF NOT EXISTS idx_items_updated ON items(updated_at DESC);

            -- Full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
                name, description, content, tags,
                content='items',
                content_rowid='id'
            );

            -- Triggers to keep FTS in sync
            CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
                INSERT INTO items_fts(rowid, name, description, content, tags)
                VALUES (new.id, new.name, new.description, new.content, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
                INSERT INTO items_fts(items_fts, rowid, name, description, content, tags)
                VALUES('delete', old.id, old.name, old.description, old.content, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
                INSERT INTO items_fts(items_fts, rowid, name, description, content, tags)
                VALUES('delete', old.id, old.name, old.description, old.content, old.tags);
                INSERT INTO items_fts(rowid, name, description, content, tags)
                VALUES (new.id, new.name, new.description, new.content, new.tags);
            END;

            -- Settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Item versions table for version history
            CREATE TABLE IF NOT EXISTS item_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                version INTEGER NOT NULL,

                -- Snapshot of all item fields at this version
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                content TEXT NOT NULL,
                model TEXT,
                tools TEXT,
                allowed_tools TEXT,
                argument_hint TEXT,
                permission_mode TEXT,
                skills TEXT,
                tags TEXT,

                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

                FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_versions_item ON item_versions(item_id, version DESC);
            "#,
        )?;

        // Run migrations for existing databases
        self.run_migrations()?;

        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        // Migration: Add version column to items table
        let has_version_column: bool = self
            .conn
            .prepare("SELECT version FROM items LIMIT 1")
            .is_ok();

        if !has_version_column {
            self.conn.execute(
                "ALTER TABLE items ADD COLUMN version INTEGER DEFAULT 1",
                [],
            )?;
        }

        Ok(())
    }
}
