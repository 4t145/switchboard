import type { Base64Bytes } from "./bytes";

export type TlsCertParams = {
    certs: Base64Bytes[];
    key: Base64Bytes;
    ocsp: Base64Bytes | null;
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
