-- v0.6.0 replaces the first_run flag with onboarding_complete. Carry finished
-- onboarding over so existing installs do not see the new setup flow.
INSERT OR IGNORE INTO settings (key, value)
SELECT 'onboarding_complete', 'true' FROM settings WHERE key = 'first_run' AND value = 'done';
