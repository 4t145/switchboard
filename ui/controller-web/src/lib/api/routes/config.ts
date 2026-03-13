import type { HumanReadableServiceConfig } from '../types/human_readable';
import { fetchQuery } from './index';

export const configApi = {
	getCurrentConfig: () => fetchQuery<HumanReadableServiceConfig | null>('/api/state/current_config')
};
