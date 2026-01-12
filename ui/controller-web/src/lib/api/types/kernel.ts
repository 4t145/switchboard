import type { Bytes } from "./bytes";
export type KernelMeta = {
    version: string;
    build: string;
};

export type KernelInfo = {
    name: string;
    id: string;
    description: string | null;
    meta: KernelMeta;
};

export type KernelStateKind =
    | { kind: "waiting_config" }
    | { kind: "running"; data: { config_signature: Bytes } }
    | {
        kind: "updating";
        data: {
            original_config_signature: Bytes;
            new_config_signature: Bytes;
        };
    }
    | { kind: "shutting_down" }
    | { kind: "stopped" };

export type KernelState = {
    // Serialized as RFC3339 timestamp string
    since: string;
} & KernelStateKind;

export type KernelInfoAndState = {
    info: KernelInfo;
    state: KernelState;
};

export type KernelConnectionAndState =
    | { connection: "connected"; state: KernelInfoAndState }
    | { connection: "disconnected" };
