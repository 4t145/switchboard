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

import { fetchJson } from './index';

export const resolveApi = {
	service_config: (request: ResolveServiceConfigRequest) =>
		fetchJson<ResolveServiceConfigResponse>('/api/resolve/service_config', {
			method: 'POST',
			body: JSON.stringify(request)
		})
};
