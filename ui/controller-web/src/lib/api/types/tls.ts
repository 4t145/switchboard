import type { Base64Bytes } from './bytes';
import type { LinkOrValue } from './controller';

export type TlsCertParams = {
	certs: Base64Bytes[];
	key: Base64Bytes;
	ocsp: Base64Bytes | null;
};

export type TlsOptions = {
	ignore_client_order: boolean;
	max_fragment_size: number | null;
	alpn_protocols: string[];
	enable_secret_extraction: boolean;
	max_early_data_size: number;
	send_half_rtt_data: boolean;
	send_tls13_tickets: number;
	require_ems: boolean;
};

export type TlsResolver = { Sni: Record<string, TlsCertParams> } | { Single: TlsCertParams };

export type Tls = {
	resolver: LinkOrValue<TlsResolver>;
	options: TlsOptions;
};
