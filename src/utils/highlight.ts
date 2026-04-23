export interface Segment {
  text: string;
  highlighted: boolean;
}

const DISPLAY_LIMIT = 120;

export function buildHighlightSegments(
  content: string,
  positions: [number, number][],
): Segment[] {
  const chars = Array.from(content);
  const truncated = chars.slice(0, DISPLAY_LIMIT);
  const ellipsis = chars.length > DISPLAY_LIMIT ? "…" : "";

  if (positions.length === 0) {
    return [{ text: truncated.join("") + ellipsis, highlighted: false }];
  }

  const clipped: [number, number][] = positions
    .map(([s, e]) => [Math.max(0, s), Math.min(truncated.length, e)] as [number, number])
    .filter(([s, e]) => e > s)
    .sort((a, b) => a[0] - b[0]);

  const merged: [number, number][] = [];
  for (const range of clipped) {
    const last = merged[merged.length - 1];
    if (last && range[0] <= last[1]) {
      last[1] = Math.max(last[1], range[1]);
    } else {
      merged.push([...range]);
    }
  }

  const segments: Segment[] = [];
  let cursor = 0;
  for (const [start, end] of merged) {
    if (start > cursor) {
      segments.push({
        text: truncated.slice(cursor, start).join(""),
        highlighted: false,
      });
    }
    segments.push({
      text: truncated.slice(start, end).join(""),
      highlighted: true,
    });
    cursor = end;
  }
  if (cursor < truncated.length) {
    segments.push({
      text: truncated.slice(cursor).join(""),
      highlighted: false,
    });
  }
  if (ellipsis) {
    segments.push({ text: ellipsis, highlighted: false });
  }
  return segments;
}
