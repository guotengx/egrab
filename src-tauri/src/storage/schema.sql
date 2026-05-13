-- EGrab - SQLite Schema for Storage Engine
-- Derived from: PRD 3.3.1, ARCHITECTURE 4.4

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    platform TEXT NOT NULL,
    item_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    folder_path TEXT
);

CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    type TEXT NOT NULL,
    original_url TEXT NOT NULL,
    local_path TEXT,
    width INTEGER,
    height INTEGER,
    size_bytes INTEGER
);

CREATE INDEX IF NOT EXISTS idx_tasks_platform ON tasks(platform);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_item_id ON tasks(item_id);
CREATE INDEX IF NOT EXISTS idx_images_task_id ON images(task_id);
