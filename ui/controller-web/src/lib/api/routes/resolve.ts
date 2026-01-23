import type { HumanReadableServiceConfig, StorageObjectDescriptor } from '../types';

export type ResolveServiceConfigResponse = {
	descriptor?: StorageObjectDescriptor;
	config: HumanReadableServiceConfig;
};

export type ResolveServiceConfigRequest = {
	resolver: string;
	config: unknown;
	save_as?: string;
};

export type SaveToLinkRequest = {
	link: string;
	value: unknown;
	data_type: string;
};

export type SaveToLinkResponse = {
	link: string;
};

import { fetchJson, fetchText } from './index';

export const resolveApi = {
	service_config: (request: ResolveServiceConfigRequest) =>
		fetchJson<ResolveServiceConfigResponse>('/api/resolve/service_config', {
			method: 'POST',
			body: JSON.stringify(request)
		}),
	link_to_string: (link: string) =>
		fetchText('/api/resolve/string', {}, new URLSearchParams({ link })),
	link_to_object: (link: string) =>
		fetchJson<unknown>('/api/resolve/value', {}, new URLSearchParams({ link })),
	save_to_link: (request: SaveToLinkRequest) =>
		fetchJson<SaveToLinkResponse>('/api/resolve/save_to_link', {
			method: 'POST',
			body: JSON.stringify(request)
		})
};
