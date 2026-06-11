import { useState, useCallback } from "react";
import type { SynaptPeer } from "../types/synapt";

interface DevicesSectionProps {
  peers: SynaptPeer[];
  onSendLatest: (peerId: string) => Promise<void>;
}

type SendStatus = "idle" | "sending" | "sent" | "error";

interface PeerRowProps {
  peer: SynaptPeer;
  onSendLatest: (peerId: string) => Promise<void>;
}

function PeerRow({ peer, onSendLatest }: PeerRowProps) {
  const [status, setStatus] = useState<SendStatus>("idle");

  const handleSend = useCallback(async () => {
    setStatus("sending");
    try {
      await onSendLatest(peer.id);
      setStatus("sent");
      setTimeout(() => setStatus("idle"), 2000);
    } catch {
      setStatus("error");
      setTimeout(() => setStatus("idle"), 3000);
    }
  }, [peer.id, onSendLatest]);

  const buttonLabel =
    status === "sending"
      ? "Sending..."
      : status === "sent"
        ? "Sent"
        : status === "error"
          ? "Failed"
          : "Send clipboard";

  return (
    <div className="flex items-center justify-between px-3 py-1.5">
      <div className="flex items-center gap-2 min-w-0">
        <span
          className="shrink-0 rounded-full"
          style={{
            width: 7,
            height: 7,
            backgroundColor: peer.online ? "var(--accent)" : "var(--border)",
          }}
        />
        <span className="truncate" style={{ color: "var(--text)", fontSize: 13 }}>
          {peer.name}
        </span>
      </div>
      {peer.online && (
        <button
          type="button"
          onClick={handleSend}
          disabled={status === "sending"}
          className="shrink-0"
          style={{
            backgroundColor: "var(--surface)",
            border: "1px solid var(--border)",
            color: status === "error" ? "var(--danger)" : "var(--text)",
            borderRadius: 4,
            fontSize: 11,
            padding: "3px 8px",
            cursor: status === "sending" ? "default" : "pointer",
            opacity: status === "sending" ? 0.7 : 1,
          }}
        >
          {buttonLabel}
        </button>
      )}
    </div>
  );
}

export function DevicesSection({ peers, onSendLatest }: DevicesSectionProps) {
  return (
    <div
      className="shrink-0"
      style={{ borderBottom: "1px solid var(--border)" }}
    >
      <div
        className="px-3 pt-2 pb-1"
        style={{
          color: "var(--muted)",
          fontSize: 11,
          textTransform: "uppercase",
          letterSpacing: "0.06em",
        }}
      >
        Devices
      </div>
      {peers.length === 0 ? (
        <div
          className="px-3 pb-2"
          style={{ color: "var(--muted)", fontSize: 12 }}
        >
          No devices online. Ensure Synapt is running on the peer device.
        </div>
      ) : (
        <div className="pb-1">
          {peers.map((peer) => (
            <PeerRow key={peer.id} peer={peer} onSendLatest={onSendLatest} />
          ))}
        </div>
      )}
    </div>
  );
}
