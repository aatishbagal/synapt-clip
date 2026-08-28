interface Props {
  /** 'active' animates; 'done'/'failed' show a static coloured underline. */
  state?: "active" | "done" | "failed";
  width?: number;
}

/**
 * Thin underline that fills from the left and then empties from the left in a
 * smooth, infinite cycle. Ported from Synapt's indexing banner so both apps
 * show activity the same way.
 */
export function UnderlineLoader({ state = "active", width = 20 }: Props) {
  const colour =
    state === "done"
      ? "var(--success, var(--accent))"
      : state === "failed"
        ? "var(--danger)"
        : "var(--text)";

  return (
    <span
      style={{
        position: "relative",
        display: "inline-block",
        width,
        height: 2,
        borderRadius: 1,
        flexShrink: 0,
        overflow: "hidden",
      }}
    >
      <span
        style={
          state === "active"
            ? {
                position: "absolute",
                top: 0,
                bottom: 0,
                background: colour,
                borderRadius: 1,
                animation: "underline-load 1.2s ease-in-out infinite",
              }
            : { position: "absolute", inset: 0, background: colour, borderRadius: 1 }
        }
      />
    </span>
  );
}
