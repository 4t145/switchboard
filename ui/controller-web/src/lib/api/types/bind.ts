import type { ServiceDescriptor } from "./descriptor";

export type Bind = {
    addr: string;
    service: ServiceDescriptor;
    description: string | null;
};
