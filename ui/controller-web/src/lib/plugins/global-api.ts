import type { ProviderEditorPlugin, HttpClassEditorPlugin } from './types';
import { providerEditorRegistry, httpClassEditorRegistry } from './registry';

// Import Skeleton UI components to expose to plugins
import * as SkeletonComponents from '@skeletonlabs/skeleton-svelte';

// Import Lucide icons to expose to plugins
import * as LucideIcons from 'lucide-svelte';

/**
 * Validation utilities
 */
const validationUtils = {
	/**
	 * Validate URL format
	 */
	validateUrl(url: string): boolean {
		try {
			new URL(url);
			return true;
		} catch {
			return false;
		}
	},

	/**
	 * Validate port number
	 */
	validatePort(port: number): boolean {
		return Number.isInteger(port) && port >= 1 && port <= 65535;
	},

	/**
	 * Validate host:port format
	 */
	validateHostPort(hostPort: string): boolean {
		const parts = hostPort.split(':');
		if (parts.length !== 2) return false;
		const [host, portStr] = parts;
		const port = parseInt(portStr, 10);
		return host.length > 0 && this.validatePort(port);
	},

	/**
	 * Create a debounced function
	 */
	debounce<T extends (...args: any[]) => any>(fn: T, delay: number): T {
		let timeoutId: ReturnType<typeof setTimeout> | null = null;
		return ((...args: Parameters<T>) => {
			if (timeoutId) clearTimeout(timeoutId);
			timeoutId = setTimeout(() => fn(...args), delay);
		}) as T;
	}
};

/**
 * Global plugin API exposed to third-party plugins
 */
export const SwitchboardPluginAPI = {
	version: '1.0.0',

	/**
	 * Register a provider editor plugin
	 */
	registerProviderEditor(plugin: ProviderEditorPlugin) {
		providerEditorRegistry.register(plugin);
	},

	/**
	 * Register a HTTP class editor plugin
	 */
	registerHttpClassEditor(plugin: HttpClassEditorPlugin) {
		httpClassEditorRegistry.register(plugin);
	},

	/**
	 * Skeleton UI components (shared with plugins)
	 */
	skeleton: SkeletonComponents,

	/**
	 * Lucide icons (shared with plugins)
	 */
	icons: LucideIcons,

	/**
	 * Utility functions
	 */
	utils: validationUtils
};

/**
 * Initialize global API by attaching it to window
 * This allows third-party plugins loaded from server to register themselves
 */
export function initializeGlobalAPI() {
	if (typeof window !== 'undefined') {
		(window as any).SwitchboardPluginAPI = SwitchboardPluginAPI;
		console.log('✅ Switchboard Plugin API initialized (version 1.0.0)');
	}
}

// TypeScript augmentation for window object
declare global {
	interface Window {
		SwitchboardPluginAPI: typeof SwitchboardPluginAPI;
	}
}
