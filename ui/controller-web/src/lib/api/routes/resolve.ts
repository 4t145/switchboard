import type { ServiceConfig, StorageObjectDescriptor } from '../types';

export type ResolveServiceConfigResponse = {
	descriptor?: StorageObjectDescriptor;
	config: ServiceConfig;
};

export type ResolveServiceConfigRequest = {
	resolver: string;
	config: unknown;
};

import { fetchJson } from './index';

export const resolveApi = {
	service_config: (request: ResolveServiceConfigRequest) =>
		fetchJson<ResolveServiceConfigResponse>('/resolve/service_config', {
			method: 'POST',
			body: JSON.stringify(request)
		})
};
