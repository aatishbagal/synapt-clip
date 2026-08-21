export interface SynaptPeer {
  id: string;
  name: string;
  ip: string;
  port: number;
  online: boolean;
  last_seen: string;
}

export interface BridgeState {
  active: boolean;
  peers: SynaptPeer[];
  api_version: string | null;
}

export interface SendResult {
  transfer_id: string;
  status: string;
}
