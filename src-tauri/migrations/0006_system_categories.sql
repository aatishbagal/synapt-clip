-- System categories: built-in, auto-detected, and not deletable by the user.
ALTER TABLE categories ADD COLUMN is_system INTEGER NOT NULL DEFAULT 0;

-- The classifier shipped before this migration labelled URLs "Link". The system
-- category is named "Links" to match the rest of the set, so carry existing rows
-- and the clips pointing at them across before seeding.
-- Ordered so a user who already made their own "Links" category is merged into
-- rather than tripping the UNIQUE constraint and aborting the migration.
UPDATE clips SET category = 'Links' WHERE category = 'Link';
DELETE FROM categories
 WHERE name = 'Link'
   AND EXISTS (SELECT 1 FROM categories WHERE name = 'Links');
UPDATE categories SET name = 'Links' WHERE name = 'Link';

-- "Plain Text" was never a real category, only the classifier's way of saying
-- "no match". Earlier versions could leave a row behind for it.
UPDATE clips SET category = NULL WHERE category = 'Plain Text';
DELETE FROM categories WHERE name = 'Plain Text';

INSERT OR IGNORE INTO categories (name, is_system) VALUES
    ('Links',     1),
    ('Code',      1),
    ('Email',     1),
    ('Phone',     1),
    ('File Path', 1),
    ('Color',     1);

-- Rows the old classifier already created are left untouched by INSERT OR IGNORE,
-- so flag them explicitly. Without this they would stay user-owned and deletable.
UPDATE categories
   SET is_system = 1
 WHERE name IN ('Links', 'Code', 'Email', 'Phone', 'File Path', 'Color');

-- Auto-tagging is on by default. Keys are the category name lowercased with
-- non-alphanumeric characters folded to underscores.
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('category_enabled_links',     'true'),
    ('category_enabled_code',      'true'),
    ('category_enabled_email',     'true'),
    ('category_enabled_phone',     'true'),
    ('category_enabled_file_path', 'true'),
    ('category_enabled_color',     'true');
