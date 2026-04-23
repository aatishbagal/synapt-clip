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
}
