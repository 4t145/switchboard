import type { Bytes } from "./bytes";

export type TlsCertParams = {
    certs: Bytes[];
    key: Bytes;
    ocsp: Bytes | null;
};

export type TlsOptions = {
    ignoreClientOrder: boolean;
    maxFragmentSize: number | null;
    alpnProtocols: string[];
    enableSecretExtraction: boolean;
    maxEarlyDataSize: number;
    sendHalfRttData: boolean;
    sendTls13Tickets: number;
    requireEms: boolean;
};

export type TlsResolver =
    | { sni: Record<string, TlsCertParams> }
    | { single: TlsCertParams };

export type Tls = {
    resolver: TlsResolver;
    options: TlsOptions;
};
