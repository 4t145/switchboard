import type {
	ServiceConfig,
	KernelConnectionAndState,
	ResultObject,
	HumanReadableServiceConfig
} from '../types';
import type { LinkOrValue } from '../types/controller';
import { fetchJson } from './index';

export type KernelSummary = Record<string, KernelConnectionAndState>;
export type ConfigUpdateResults = Array<[string, ResultObject<null>]>;

export interface UpdateConfigRequest {
	new_config: LinkOrValue<unknown>;
}

export const kernelManagerApi = {
	listKernels: () => fetchJson<KernelSummary>('/api/kernel_manager/kernels'),

	/**
	 * Update configuration for all kernels
	 * @param config - The configuration (can be HumanReadableServiceConfig or ServiceConfig)
	 * @returns Results for each kernel update
	 */
	updateConfig: (config: HumanReadableServiceConfig | string) =>
		fetchJson<ConfigUpdateResults>('/api/kernel_manager/kernels', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				new_config: config
			} as UpdateConfigRequest)
		}),

	/**
	 * Refresh kernel connections
	 */
	refreshKernels: () =>
		fetchJson<ResultObject<null>>('/api/kernel_manager/refresh', {
			method: 'POST'
		})
};
