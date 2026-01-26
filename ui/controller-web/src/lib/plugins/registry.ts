import type {
	ProviderEditorPlugin,
	HttpClassEditorPlugin,
	HttpNodeClassPlugin,
	HttpFilterClassPlugin
} from './types';
import { writable } from 'svelte/store';

/**
 * Global provider editor plugin registry
 */
class ProviderEditorRegistry {
	private plugins = new Map<string, ProviderEditorPlugin>();
	private store = writable(this.plugins);

	/**
	 * Register a provider editor plugin
	 */
	register(plugin: ProviderEditorPlugin) {
		const isUpdate = this.plugins.has(plugin.provider);
		this.plugins.set(plugin.provider, plugin);
		this.store.set(this.plugins);
		
		if (isUpdate) {
			console.warn(`🔄 Provider editor updated: ${plugin.displayName} (${plugin.provider})`);
		} else {
			console.log(`✅ Provider editor registered: ${plugin.displayName} (${plugin.provider})`);
		}
		console.debug('[ProviderEditorRegistry] Current plugins:', Array.from(this.plugins.keys()));
	}

	/**
	 * Unregister a plugin
	 */
	unregister(provider: string) {
		const plugin = this.plugins.get(provider);
		if (plugin) {
			this.plugins.delete(provider);
			this.store.set(this.plugins);
			console.log(`❌ Provider editor unregistered: ${plugin.displayName} (${provider})`);
		}
	}

	/**
	 * Get a plugin by provider name
	 */
	get(provider: string): ProviderEditorPlugin | undefined {
		const plugin = this.plugins.get(provider);
		if (!plugin) {
			console.debug(`[ProviderEditorRegistry] Plugin not found for provider: ${provider}`);
		}
		return plugin;
	}

	/**
	 * Get all registered provider names
	 */
	getAll(): string[] {
		return Array.from(this.plugins.keys());
	}

	/**
	 * Subscribe to registry changes
	 */
	subscribe = this.store.subscribe;
}

/**
 * HTTP class editor registry (for HTTP nodes and filters)
 */
class HttpClassEditorRegistry {
	private plugins = new Map<string, HttpClassEditorPlugin>();

	/**
	 * Register a HTTP class editor plugin
	 */
	register(plugin: HttpClassEditorPlugin) {
		const isUpdate = this.plugins.has(plugin.classId);
		this.plugins.set(plugin.classId, plugin);
		
		if (isUpdate) {
			console.warn(
				`🔄 HTTP ${plugin.type} editor updated: ${plugin.displayName} (${plugin.classId})`
			);
		} else {
			console.log(
				`✅ HTTP ${plugin.type} editor registered: ${plugin.displayName} (${plugin.classId})`
			);
		}
		console.debug('[HttpClassEditorRegistry] Current plugins:', {
			nodes: this.getAllNodes().map(p => p.classId),
			filters: this.getAllFilters().map(p => p.classId)
		});
	}

	/**
	 * Unregister a plugin
	 */
	unregister(classId: string) {
		const plugin = this.plugins.get(classId);
		if (plugin) {
			this.plugins.delete(classId);
			console.log(
				`❌ HTTP ${plugin.type} editor unregistered: ${plugin.displayName} (${classId})`
			);
		}
	}

/**
 * Get a plugin by class ID
 */
get<T extends HttpClassEditorPlugin = HttpClassEditorPlugin>(
	classId: string
): T | undefined {
	const plugin = this.plugins.get(classId) as T | undefined;
	if (!plugin) {
		console.debug(`[HttpClassEditorRegistry] Plugin not found for class: ${classId}`);
	}
	return plugin;
}

/**
 * Get all registered node plugins
 */
getAllNodes(): HttpNodeClassPlugin[] {
	return Array.from(this.plugins.values()).filter((p): p is HttpNodeClassPlugin => p.type === 'node');
}

/**
 * Get all registered filter plugins
 */
getAllFilters(): HttpFilterClassPlugin[] {
	return Array.from(this.plugins.values()).filter((p): p is HttpFilterClassPlugin => p.type === 'filter');
}

	/**
	 * Get all class IDs
	 */
	getAll(): string[] {
		return Array.from(this.plugins.keys());
	}
}

// Export global singletons
export const providerEditorRegistry = new ProviderEditorRegistry();
export const httpClassEditorRegistry = new HttpClassEditorRegistry();

/**
 * Helper functions for easier access
 */
export function getProviderEditorPlugin(provider: string): ProviderEditorPlugin | undefined {
	return providerEditorRegistry.get(provider);
}

export function getHttpClassEditorPlugin<T extends HttpClassEditorPlugin = HttpClassEditorPlugin>(
	classId: string,
	type?: 'node' | 'filter'
): T | undefined {
	const plugin = httpClassEditorRegistry.get<T>(classId);
	// Optionally validate type
	if (plugin && type && plugin.type !== type) {
		console.warn(
			`[getHttpClassEditorPlugin] Type mismatch: ${classId} is a ${plugin.type}, not ${type}`
		);
		return undefined;
	}
	return plugin;
}

export function listHttpClassEditorPlugins(type?: 'node' | 'filter'): HttpClassEditorPlugin[] {
	if (type === 'node') {
		return httpClassEditorRegistry.getAllNodes();
	} else if (type === 'filter') {
		return httpClassEditorRegistry.getAllFilters();
	} else {
		return [...httpClassEditorRegistry.getAllNodes(), ...httpClassEditorRegistry.getAllFilters()];
	}
}
