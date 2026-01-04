import type { Bytes } from "./bytes";
import type { ServiceConfig } from "./index";
import type { ControllerInfo } from "./controller";
import type { KernelInfo, KernelState } from "./kernel";

export type TakeOver = {
    controllerInfo: ControllerInfo;
};

export type BeenTookOver = {
    newControllerInfo: ControllerInfo;
};

export type KernelAuth = {
    randomBytes: Bytes;
    kernelInfo: KernelInfo;
};

export type KernelAuthResponse = {
    signature: Bytes;
};

export type UpdateConfig = {
    config: ServiceConfig;
};

export type UpdateConfigBuilder = {
    config: ServiceConfig;
};

export type ControlCommandData =
    | { kind: "quit" }
    | { kind: "updateConfig"; data: UpdateConfig };

export type ControlCommand = {
    seq: number;
    ts: number;
    signerName: string;
    data: ControlCommandData;
    signature: Bytes;
};

export type ControlCommandAccepted = {
    seq: number;
};

export type ControllerMessage =
    | { kind: "heartBeat" }
    | { kind: "takeOver"; data: TakeOver }
    | { kind: "authResponse"; data: KernelAuthResponse }
    | { kind: "controlCommand"; data: ControlCommand }
    | { kind: "disconnect" };

export type KernelMessage =
    | { kind: "heartBeat"; data: KernelState }
    | { kind: "auth"; data: KernelAuth }
    | { kind: "controlCommandAccepted"; data: ControlCommandAccepted }
    | { kind: "beenTookOver"; data: BeenTookOver }
    | { kind: "disconnect" };
