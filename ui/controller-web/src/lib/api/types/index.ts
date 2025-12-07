export * from "./bind";
export * from "./bytes";
export * from "./control";
export * from "./controller";
export * from "./descriptor";
export * from "./error";
export * from "./kernel";
export * from "./named_service";
export * from "./protocol";
export * from "./tls";

import type { Bind } from "./bind";
import type { NamedService } from "./named_service";
import type { Tls } from "./tls";

export type Config = {
    namedServices: Record<string, NamedService>;
    binds: Record<string, Bind>;
    enabled: string[];
    tls: Record<string, Tls>;
};
