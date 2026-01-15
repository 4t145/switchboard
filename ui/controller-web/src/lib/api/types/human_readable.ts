import type { LinkOrValue } from './controller';
import type { TlsOptions, TlsCertParams } from './tls';

export type FileBind = {
	bind: string;
	tls?: string;
	description?: string;
};

export type FileTcpServiceConfig = {
	provider: string;
	name: string;
	config?: LinkOrValue<unknown>;
	description?: string;
	binds: FileBind[];
};

export type FileStyleTlsResolver =
	| { Single: TlsCertParams }
	| { Sni: { sni: { hostname: string; tls_in_file: TlsCertParams }[] } };

export type FileStyleTls = {
	name: string;
	options?: TlsOptions;
} & FileStyleTlsResolver;

export type HumanReadableServiceConfig = {
	tcp_services: FileTcpServiceConfig[];
	tls: FileStyleTls[];
};
