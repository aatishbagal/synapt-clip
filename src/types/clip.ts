export interface Clip {
  id: number;
  content: string;
  content_type: string;
  created_at: string;
  source_app: string | null;
  pinned: boolean;
  deleted_at: string | null;
  was_compressed: boolean;
  original_size: number;
  compressed_size: number;
  category: string | null;
  sender_name?: string | null;
  sender_peer_id?: string | null;
}

/// A category row. `is_system` is 1 for the built-in, auto-detected categories
/// and 0 for ones the user created; only the latter can be deleted.
export interface Category {
  id: number;
  name: string;
  is_system: number;
}
