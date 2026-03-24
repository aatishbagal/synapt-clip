import type { Clip } from "../types/clip";

interface ClipCardProps {
  clip: Clip;
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
}

export function ClipCard({ clip, onCopy, onDelete }: ClipCardProps) {
  void clip;
  void onCopy;
  void onDelete;
  return <div />;
}
