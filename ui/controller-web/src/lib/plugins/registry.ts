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

/**
 * HTTP class editor registry (for HTTP nodes and filters)
 */
class HttpClassEditorRegistry {
	private nodes = new Map<string, HttpNodeClassPlugin>();
	private filters = new Map<string, HttpFilterClassPlugin>();

	register<T>(plugin: HttpClassEditorPlugin<T>) {
		if (plugin.type === 'node') {
			this.registerNode(plugin as HttpNodeClassPlugin<T>);
		} else if (plugin.type === 'filter') {
			this.registerFilter(plugin as HttpFilterClassPlugin<T>);
		} else {
			console.error(`[HttpClassEditorRegistry] Failed to register plugin: Unknown type`);
		}
	}

	/**
	 * Register a HTTP class editor plugin
	 */
	registerNode<T>(plugin: HttpNodeClassPlugin<T>) {
		const isUpdate = this.nodes.has(plugin.classId);
		this.nodes.set(plugin.classId, plugin as HttpNodeClassPlugin);

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
			nodes: this.getAllNodes().map((p) => p.classId),
			filters: this.getAllFilters().map((p) => p.classId)
		});
	}

	registerFilter<T>(plugin: HttpFilterClassPlugin<T>) {
		const isUpdate = this.filters.has(plugin.classId);
		this.filters.set(plugin.classId, plugin as HttpFilterClassPlugin);

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
			nodes: this.getAllNodes().map((p) => p.classId),
			filters: this.getAllFilters().map((p) => p.classId)
		});
	}

	unregister(classId: string) {
		if (this.nodes.has(classId)) {
			this.unregisterNode(classId);
		} else if (this.filters.has(classId)) {
			this.unregisterFilter(classId);
		}
	}

	unregisterFilter(classId: string) {
		const plugin = this.filters.get(classId);
		if (plugin) {
			this.filters.delete(classId);
			console.log(`❌ HTTP ${plugin.type} editor unregistered: ${plugin.displayName} (${classId})`);
		}
	}
	/**
	 * Unregister a plugin
	 */
	unregisterNode(classId: string) {
		const plugin = this.nodes.get(classId);
		if (plugin) {
			this.nodes.delete(classId);
			console.log(`❌ HTTP ${plugin.type} editor unregistered: ${plugin.displayName} (${classId})`);
		}
	}

	/**
	 * Get a plugin by class ID
	 */
	get<T extends HttpClassEditorPlugin = HttpClassEditorPlugin>(classId: string): T | undefined {
		const plugin =
			(this.nodes.get(classId) as T | undefined) ??
			(this.filters.get(classId) as T | undefined) ??
			undefined;
		if (!plugin) {
			console.debug(`[HttpClassEditorRegistry] Plugin not found for class: ${classId}`);
		}
		return plugin;
	}

	getFilter(classId: string): HttpFilterClassPlugin | undefined {
		const plugin = this.filters.get(classId);
		if (!plugin) {
			console.debug(`[HttpClassEditorRegistry] Filter plugin not found for class: ${classId}`);
		}
		return plugin;
	}

	getNode(classId: string): HttpNodeClassPlugin | undefined {
		const plugin = this.nodes.get(classId);
		if (!plugin) {
			console.debug(`[HttpClassEditorRegistry] Node plugin not found for class: ${classId}`);
		}
		return plugin;
	}

	/**
	 * Get all registered node plugins
	 */
	getAllNodes(): HttpNodeClassPlugin[] {
		return Array.from(this.nodes.values()).filter(
			(p): p is HttpNodeClassPlugin => p.type === 'node'
		);
	}

	/**
	 * Get all registered filter plugins
	 */
	getAllFilters(): HttpFilterClassPlugin[] {
		return Array.from(this.filters.values()).filter(
			(p): p is HttpFilterClassPlugin => p.type === 'filter'
		);
	}

	/**
	 * Get all class IDs
	 */
	getAll(): string[] {
		return [...this.nodes.keys(), ...this.filters.keys()];
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

export const getHttpClassEditorPlugin: {
	(classId: string): HttpClassEditorPlugin | undefined;
} = (classId: string) => {
	const plugin = httpClassEditorRegistry.get(classId);
	return plugin;
};

export function listHttpClassEditorPlugins(type?: 'node' | 'filter'): HttpClassEditorPlugin[] {
	if (type === 'node') {
		return httpClassEditorRegistry.getAllNodes();
	} else if (type === 'filter') {
		return httpClassEditorRegistry.getAllFilters();
	} else {
		return [...httpClassEditorRegistry.getAllNodes(), ...httpClassEditorRegistry.getAllFilters()];
	}
}
