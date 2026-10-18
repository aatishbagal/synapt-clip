import { useState, useCallback } from "react";
import { motion } from "motion/react";
import type { SynaptPeer } from "../types/synapt";
import { FADE_IN } from "../animations";
import { describeError } from "../utils/errors";

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
  const [reason, setReason] = useState<string | undefined>(undefined);

  const handleSend = useCallback(async () => {
    setStatus("sending");
    try {
      await onSendLatest(peer.id);
      setStatus("sent");
      setTimeout(() => setStatus("idle"), 2000);
    } catch (err) {
      setReason(describeError(err, `Could not send to ${peer.name}. Try again.`));
      setStatus("error");
      setTimeout(() => setStatus("idle"), 3000);
    }
  }, [peer.id, peer.name, onSendLatest]);

  const buttonLabel =
    status === "sending"
      ? "Sending..."
      : status === "sent"
        ? "Sent"
        : status === "error"
          ? "Could not send"
          : "Send clipboard";

  return (
    <div className="flex items-center justify-between gap-2 px-3 py-1">
      <span className="truncate" style={{ color: "var(--text)", fontSize: 13 }}>
        {peer.name}
      </span>
      {peer.online ? (
        <button
          type="button"
          onClick={handleSend}
          disabled={status === "sending"}
          title={status === "error" ? reason : undefined}
          aria-live="polite"
          className="btn btn-secondary shrink-0"
          style={status === "error" ? { color: "var(--danger)" } : undefined}
        >
          {buttonLabel}
        </button>
      ) : (
        // Text label rather than a status dot.
        <span className="shrink-0 text-xs" style={{ color: "var(--muted)" }}>
          Offline
        </span>
      )}
    </div>
  );
}

export function DevicesSection({ peers, onSendLatest }: DevicesSectionProps) {
  return (
    <motion.div
      {...FADE_IN}
      className="shrink-0"
      style={{ borderBottom: "1px solid var(--border)" }}
    >
      <h2 className="section-heading px-3 pt-2">Devices</h2>
      {peers.length === 0 ? (
        <div
          className="px-3 pb-2"
          style={{ color: "var(--muted)", fontSize: 12 }}
        >
          No devices online. Open Synapt on your other device to send clips to it.
        </div>
      ) : (
        <div className="pb-1">
          {peers.map((peer) => (
            <PeerRow key={peer.id} peer={peer} onSendLatest={onSendLatest} />
          ))}
        </div>
      )}
    </motion.div>
  );
}
