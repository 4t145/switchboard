import type {
	ConfigRolloutReport,
	KernelConnectionAndState,
	ResultObject,
	HumanReadableServiceConfig
} from '../types';
import type { LinkOrValue } from '../types/controller';
import { fetchJson } from './index';

export type KernelSummary = Record<string, KernelConnectionAndState>;

export type UpdateConfigRequest =
	| {
			mode: 'new_config';
			new_config: LinkOrValue<unknown>;
	  }
	| {
			mode: 'resolve';
			resolver: string;
			config: unknown;
	  };

export const kernelManagerApi = {
	listKernels: () => fetchJson<KernelSummary>('/api/kernel_manager/kernels'),

	/**
	 * Update configuration for all kernels
	 * @returns Transactional rollout report
	 */
	updateConfig: (request: UpdateConfigRequest) =>
		fetchJson<ConfigRolloutReport>('/api/kernel_manager/kernels', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(request)
		}),

	/**
	 * Refresh kernel connections
	 */
	refreshKernels: () =>
		fetchJson<ResultObject<null>>('/api/kernel_manager/refresh', {
			method: 'POST'
		})
};
