import type { ProviderEditorPlugin } from './types';
import { writable } from 'svelte/store';

/**
 * Global provider editor plugin registry
 */
class ProviderEditorRegistry {
	private plugins = new Map<string, ProviderEditorPlugin>();
	private store = writable(this.plugins);

	/**`
	 * Register a provider editor plugin
	 */
	register<T>(plugin: ProviderEditorPlugin<T>) {
		const isUpdate = this.plugins.has(plugin.provider);
		this.plugins.set(plugin.provider, plugin as ProviderEditorPlugin);
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
// Export global singletons
export const providerEditorRegistry = new ProviderEditorRegistry();
/**
 * Helper functions for easier access
 */
export function getProviderEditorPlugin(provider: string): ProviderEditorPlugin | undefined {
	return providerEditorRegistry.get(provider);
}