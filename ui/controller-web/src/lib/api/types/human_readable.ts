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

export type SniFileStyleTlsResolver = { sni: ({ hostname: string } & TlsCertParams)[] };
export type FileStyleTlsResolver =
	| TlsCertParams
	| SniFileStyleTlsResolver;

export function isSniTlsResolver(
	resolver: FileStyleTlsResolver
): resolver is SniFileStyleTlsResolver {
	return (resolver as SniFileStyleTlsResolver).sni !== undefined;
}
export function isSingleTlsResolver(
	resolver: FileStyleTlsResolver
): resolver is TlsCertParams {
	return (resolver as TlsCertParams).certs !== undefined;
}
export type FileStyleTls = {
	name: string;
	options?: TlsOptions;
} & FileStyleTlsResolver;

export type HumanReadableServiceConfig = {
	tcp_services: FileTcpServiceConfig[];
	tls: FileStyleTls[];
};
