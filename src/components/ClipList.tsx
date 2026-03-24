import type { Clip } from "../types/clip";

interface ClipListProps {
  clips: Clip[];
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
  loading: boolean;
}

export function ClipList({ clips, onCopy, onDelete, loading }: ClipListProps) {
  void clips;
  void onCopy;
  void onDelete;
  void loading;
  return <div />;
}
