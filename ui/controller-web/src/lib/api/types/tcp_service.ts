import type { LinkOrValue } from "./controller";

export type TcpService = {
    provider: string;
    name: string;
    config?: LinkOrValue<unknown>;
    description?: unknown;
};
