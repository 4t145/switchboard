/**
 * API client for configuration management
 */

import type { HumanReadableServiceConfig } from './types/human_readable';

/**
 * Fetch the current service configuration
 * @returns The current configuration or null if not loaded
 */
export async function getCurrentConfig(): Promise<HumanReadableServiceConfig | null> {
	const response = await fetch('/api/state/current_config');
	if (!response.ok) {
		throw new Error(`Failed to fetch config: ${response.statusText}`);
	}
	return await response.json();
}
