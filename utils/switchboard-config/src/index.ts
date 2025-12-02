// pub struct NamedService {
//     pub provider: String,
//     pub name: String,
//     pub config: Option<String>,
//     pub description: Option<String>,
//     pub tls: Option<String>,
// }

export type NamedService = {
    provider: string;
    name: string;
    config?: string;
    description?: string;
    tls?: string;
}

export type Bind = {
    addr: string,
    service: ServiceDescriptor,
    description?: string,
}

export type ServiceDescriptor = string

export type Config = {
    services: Record<string, NamedService>,
    binds: Record<string, Bind>,
    enabled: string[],
}