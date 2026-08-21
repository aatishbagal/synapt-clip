-- Columns for clips received from a remote Synapt peer.
-- source_app already exists (added in 0001_initial.sql), so only the sender
-- identity columns are added here.
ALTER TABLE clips ADD COLUMN sender_name TEXT;
ALTER TABLE clips ADD COLUMN sender_peer_id TEXT;
