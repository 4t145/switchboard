import type { ProviderEditorPlugin, HttpClassEditorPlugin } from './types';
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
		this.plugins.set(plugin.provider, plugin);
		this.store.set(this.plugins);
		console.log(`✅ Provider editor registered: ${plugin.displayName} (${plugin.provider})`);
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
		return this.plugins.get(provider);
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
		this.plugins.set(plugin.classId, plugin);
		console.log(
			`✅ HTTP ${plugin.type} editor registered: ${plugin.displayName} (${plugin.classId})`
		);
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
	get(classId: string): HttpClassEditorPlugin | undefined {
		return this.plugins.get(classId);
	}

	/**
	 * Get all registered node plugins
	 */
	getAllNodes(): HttpClassEditorPlugin[] {
		return Array.from(this.plugins.values()).filter((p) => p.type === 'node');
	}

	/**
	 * Get all registered filter plugins
	 */
	getAllFilters(): HttpClassEditorPlugin[] {
		return Array.from(this.plugins.values()).filter((p) => p.type === 'filter');
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
