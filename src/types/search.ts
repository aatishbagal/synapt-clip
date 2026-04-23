export interface SearchResult {
  clip_id: number;
  score: number;
  match_type: "prefix" | "substring" | "fuzzy";
  match_positions: [number, number][];
}
