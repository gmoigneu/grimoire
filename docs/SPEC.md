# GRIMOIRE Specification

A TUI application for managing Claude Code prompts, agents, skills, and commands.

## Overview

GRIMOIRE (Global Repository and Index for Model Operations, Instructions and Response Engineering) helps users manage their Claude Code configuration through an intuitive terminal interface with vim-style keybindings. Data is stored in SQLite at `~/.local/share/grimoire/grimoire.db`.

---

## Screen Layouts

### Main Screen
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ GRIMOIRE                                                        [?] Help    │
├───────────────────┬─────────────────────────────────────────────────────────┤
│ Categories        │ Recent Items                                            │
│                   │                                                         │
│ > Prompts    (12) │  NAME              CATEGORY   TAGS        UPDATED       │
│   Agents      (5) │  ────────────────────────────────────────────────────   │
│   Skills      (3) │  > code-reviewer   Agent      review,qa   2 hours ago   │
│   Commands    (8) │    pr-summary      Command    git,pr      5 hours ago   │
│                   │    rust-expert     Agent      rust,code   1 day ago     │
│ Tags              │    security-check  Skill      security    1 day ago     │
│                   │    api-template    Prompt     api,rest    2 days ago    │
│   #review     (4) │    debug-helper    Prompt     debug       3 days ago    │
│   #rust       (3) │    test-writer     Agent      testing     3 days ago    │
│   #git        (3) │    git-commit      Command    git         4 days ago    │
│   #api        (2) │    doc-generator   Skill      docs        5 days ago    │
│   #security   (2) │    refactor-aid    Prompt     refactor    1 week ago    │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
├───────────────────┴─────────────────────────────────────────────────────────┤
│ /search  n:new  e:edit  c:copy  d:delete  x:export  Enter:view  ?:help  q:quit │
└─────────────────────────────────────────────────────────────────────────────┘
```

### View Item Screen
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Agent: code-reviewer                                            [ESC] Back  │
├─────────────────────────────────────────────────────────────────────────────┤
│ Name:        code-reviewer                                                  │
│ Category:    Agent                                                          │
│ Tags:        review, qa, code                                               │
│ Model:       sonnet                                                         │
│ Tools:       Read, Grep, Glob                                               │
│ Created:     2024-01-15 10:30                                               │
│ Updated:     2024-01-20 14:22                                               │
├─────────────────────────────────────────────────────────────────────────────┤
│ Description:                                                                │
│ Expert code reviewer that analyzes code for bugs, style issues, and         │
│ potential improvements.                                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│ Content:                                                                    │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ You are a senior code reviewer with expertise in multiple languages.    │ │
│ │ When reviewing code:                                                    │ │
│ │                                                                         │ │
│ │ 1. Check for bugs and logic errors                                      │ │
│ │ 2. Evaluate code style and consistency                                  │ │
│ │ 3. Look for security vulnerabilities                                    │ │
│ │ 4. Suggest performance improvements                                     │ │
│ │ ...                                                                     │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ e:edit  c:copy  a:ai-assist  x:export  d:delete  ESC:back                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Edit/New Item Screen
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ New Agent                                                       [ESC] Cancel│
├─────────────────────────────────────────────────────────────────────────────┤
│ Name:        [_________________________]                                    │
│ Category:    [Agent     ▼]                                                  │
│ Tags:        [review, code_____________]                                    │
│ Model:       [sonnet    ▼]  (optional)                                      │
│ Tools:       [Read, Grep_______________]  (optional)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Description: (required)                                                     │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ _                                                                       │ │
│ │                                                                         │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Content: (required)                                                         │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ _                                                                       │ │
│ │                                                                         │ │
│ │                                                                         │ │
│ │                                                                         │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Tab:next field  S-Tab:prev  a:ai-assist  Ctrl+S:save  ESC:cancel            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Search Screen
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Search: rust review_                                            [ESC] Close │
├─────────────────────────────────────────────────────────────────────────────┤
│ Results (3)                                                                 │
│                                                                             │
│  NAME              CATEGORY   TAGS           MATCH                          │
│  ─────────────────────────────────────────────────────────────────────────  │
│  > rust-expert     Agent      rust,code      name: "rust"                   │
│    code-reviewer   Agent      review,qa      tag: "review"                  │
│    rust-template   Prompt     rust,template  name: "rust", content match    │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ j/k:navigate  Enter:select  c:copy  ESC:close                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Settings Screen
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Settings                                                        [ESC] Back  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│ LLM Configuration                                                           │
│ ────────────────────────────────────────────────────                        │
│ Provider:      [Anthropic  ▼]                                               │
│ Model:         [claude-sonnet-4-20250514________▼]                          │
│ API Key:       [sk-ant-••••••••••••••••••••]  [Test]                        │
│                                                                             │
│ OpenAI (optional)                                                           │
│ ────────────────────────────────────────────────────                        │
│ API Key:       [sk-••••••••••••••••••••••••]  [Test]                        │
│                                                                             │
│ Export Settings                                                             │
│ ────────────────────────────────────────────────────                        │
│ Default path:  [~/.claude_______________________]                           │
│                                                                             │
│ Data                                                                        │
│ ────────────────────────────────────────────────────                        │
│ Database:      ~/.local/share/grimoire/grimoire.db                          │
│ Items:         28 total (12 prompts, 5 agents, 3 skills, 8 commands)        │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Tab:next  S-Tab:prev  Ctrl+S:save  ESC:back                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### AI Assistant Popup (when pressing 'a' in edit mode)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Edit Agent: code-reviewer                                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Content:                                                                    │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ You are a senior code reviewer...                                       │ │
│ │                                                                         │ │
│ └───────────────────────────────────┬─────────────────────────────────────┘ │
│                                     │ AI Assistant                        │ │
│                                     ├─────────────────────────────────────┤ │
│                                     │ How can I help?                     │ │
│                                     │                                     │ │
│                                     │ > Improve this prompt               │ │
│                                     │   Make it more concise              │ │
│                                     │   Add examples                      │ │
│                                     │   Custom request...                 │ │
│                                     │                                     │ │
│                                     │ [____________________________]      │ │
│                                     │                                     │ │
│                                     │ Enter:select  ESC:close             │ │
│                                     └─────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Tab:next field  a:ai-assist  Ctrl+S:save  ESC:cancel                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Data Model

### SQLite Schema

```sql
-- ~/.local/share/grimoire/grimoire.db

CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL CHECK(category IN ('prompt', 'agent', 'skill', 'command')),
    description TEXT,
    content TEXT NOT NULL,

    -- Category-specific fields (nullable, validated by app)
    model TEXT,                    -- agents, commands: sonnet, opus, haiku, or full model ID
    tools TEXT,                    -- agents, skills: comma-separated tool list
    allowed_tools TEXT,            -- commands: tool permissions like "Bash(git:*)"
    argument_hint TEXT,            -- commands: e.g., "[pr-number]"
    permission_mode TEXT,          -- agents: default, acceptEdits, bypassPermissions, plan
    skills TEXT,                   -- agents: comma-separated skill names

    tags TEXT,                     -- comma-separated tags
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_items_category ON items(category);
CREATE INDEX idx_items_updated ON items(updated_at DESC);

-- Full-text search
CREATE VIRTUAL TABLE items_fts USING fts5(
    name, description, content, tags,
    content='items',
    content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER items_ai AFTER INSERT ON items BEGIN
    INSERT INTO items_fts(rowid, name, description, content, tags)
    VALUES (new.id, new.name, new.description, new.content, new.tags);
END;

CREATE TRIGGER items_ad AFTER DELETE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, name, description, content, tags)
    VALUES('delete', old.id, old.name, old.description, old.content, old.tags);
END;

CREATE TRIGGER items_au AFTER UPDATE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, name, description, content, tags)
    VALUES('delete', old.id, old.name, old.description, old.content, old.tags);
    INSERT INTO items_fts(rowid, name, description, content, tags)
    VALUES (new.id, new.name, new.description, new.content, new.tags);
END;

-- Settings table
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### Category Validation Rules

| Category | Required Fields | Optional Fields |
|----------|-----------------|-----------------|
| **Prompt** | name, content | description, tags |
| **Agent** | name, description, content | model, tools, permission_mode, skills, tags |
| **Skill** | name, description, content | allowed_tools, tags |
| **Command** | name, content | description, allowed_tools, argument_hint, model, tags |

---

## Keyboard Bindings

### Global (all screens)
| Key | Action |
|-----|--------|
| `q` | Quit / Back |
| `?` | Show help |
| `Esc` | Cancel / Back |
| `:` | Command mode (vim-style) |

### Main Screen - Navigation
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Focus sidebar |
| `l` / `→` | Focus main panel |
| `g` `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

### Main Screen - Actions
| Key | Action |
|-----|--------|
| `/` | Open search |
| `n` | New item |
| `Enter` | View selected item |
| `e` | Edit selected item |
| `c` | Copy item content to clipboard |
| `y` `y` | Copy item content (vim-style) |
| `d` `d` | Delete item (with confirmation) |
| `x` | Export item to .claude/ directory |
| `s` | Open settings |
| `1-4` | Jump to category (1:Prompts, 2:Agents, 3:Skills, 4:Commands) |
| `t` | Filter by tag (when tag selected in sidebar) |

### View Screen
| Key | Action |
|-----|--------|
| `e` | Edit item |
| `c` / `y` `y` | Copy content |
| `x` | Export to .claude/ |
| `d` `d` | Delete (with confirmation) |
| `a` | Open AI assistant |
| `Esc` / `q` | Back to list |

### Edit Screen
| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Ctrl+s` | Save |
| `a` | AI assistant (in content field) |
| `Esc` | Cancel (with unsaved changes warning) |

### Search
| Key | Action |
|-----|--------|
| `Enter` | Select result |
| `Esc` | Close search |
| `j` / `k` | Navigate results |
| `c` | Copy selected item |

---

## Export Format (Native Claude Code)

Export creates files in the `.claude/` directory structure:

### Agents -> `.claude/agents/{name}.md`
```markdown
---
name: code-reviewer
description: Expert code reviewer that analyzes code for bugs and style issues
tools: Read, Grep, Glob
model: sonnet
---

You are a senior code reviewer...
```

### Commands -> `.claude/commands/{name}.md`
```markdown
---
description: Summarize a pull request
allowed-tools: Bash(git:*), Bash(gh:*)
argument-hint: [pr-number]
---

Analyze PR #$1 and provide a summary...
```

### Skills -> `.claude/skills/{name}/SKILL.md`
```markdown
---
name: security-check
description: Analyzes code for security vulnerabilities
allowed-tools: Read, Grep
---

# Security Check

## Instructions
...
```

### Prompts
Prompts are **copy-only** (no file export). Use `c` or `yy` to copy content to clipboard.

---

## File Structure

```
src/
├── main.rs              # Entry point, terminal setup
├── app.rs               # App state, main event loop
├── ui/
│   ├── mod.rs
│   ├── main_screen.rs   # Main list view
│   ├── view_screen.rs   # Item detail view
│   ├── edit_screen.rs   # Edit/new item form
│   ├── search.rs        # Search overlay
│   ├── settings.rs      # Settings screen
│   ├── ai_popup.rs      # AI assistant popup
│   └── components/
│       ├── sidebar.rs   # Category/tag sidebar
│       ├── item_list.rs # Item list table
│       └── input.rs     # Text input widget
├── db/
│   ├── mod.rs
│   ├── schema.rs        # SQLite schema, migrations
│   ├── items.rs         # Item CRUD operations
│   └── settings.rs      # Settings storage
├── models/
│   ├── mod.rs
│   ├── item.rs          # Item struct, validation
│   └── category.rs      # Category enum, field requirements
├── export/
│   ├── mod.rs
│   └── claude.rs        # Export to .claude/ format
├── llm/
│   ├── mod.rs
│   ├── anthropic.rs     # Anthropic API client
│   └── openai.rs        # OpenAI API client
└── config.rs            # App configuration
```

---

## Dependencies

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
color-eyre = "0.6"
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
directories = "5.0"          # For ~/.local/share/grimoire path
arboard = "3.4"              # Clipboard
chrono = "0.4"
```

---

## Implementation Order

1. **Phase 1: Core Foundation**
   - Project setup with dependencies
   - SQLite database initialization
   - Basic App struct and event loop
   - Main screen layout (sidebar + list)

2. **Phase 2: CRUD Operations**
   - Item model with validation
   - Database CRUD operations
   - View screen
   - Edit/New screen
   - Delete with confirmation

3. **Phase 3: Navigation & Search**
   - Vim keybindings
   - Category filtering
   - Tag filtering
   - Full-text search

4. **Phase 4: Export**
   - Export single item
   - Export by category
   - Native .claude/ format

5. **Phase 5: AI Assistant**
   - Settings screen (API keys)
   - Anthropic client
   - OpenAI client
   - AI popup in edit mode

6. **Phase 6: Polish**
   - Clipboard support
   - Help screen
   - Error handling
   - Edge cases
