CREATE TABLE IF NOT EXISTS zenmux_config (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  ctoken TEXT NOT NULL DEFAULT '',
  session_id TEXT NOT NULL DEFAULT '',
  session_id_sig TEXT NOT NULL DEFAULT '',
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
