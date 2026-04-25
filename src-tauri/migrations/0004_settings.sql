CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('history_limit', '500'),
    ('expiry_days', '0'),
    ('theme', 'dark'),
    ('hotkey', 'Super+Shift+V'),
    ('excluded_apps', '[]');
