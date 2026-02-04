import type { Component } from 'svelte';
/**
 * Plugin validation result
 */
export interface ValidationResult {
	valid: boolean;
	errors?: string[];
	warnings?: string[];
}

/**
 * Props for provider editor components
 */
export type ProviderEditorProps<T = unknown> = {
	/** Current configuration value (can be object or LinkOrValue string) */
	value: T;
	onValueChange: (value: T) => void;
	/** Whether the editor is read-only */
	readonly?: boolean;
};

/**
 * Provider editor plugin interface
 */
export type ProviderEditorPlugin<T = unknown> = {
	/** Provider name (e.g., "http", "socks5") */
	provider: string;

	/** Editor component */
	component: Component<ProviderEditorProps<T>>;

	/** Display name */
	displayName: string;

	/** Description */
	description?: string;

	/** Default configuration generator */
	createDefaultConfig: () => T;

	/** Configuration validator (optional) */
	validate?: (config: T) => ValidationResult;
};


/**
 * Plugin manifest (loaded from server)
 */
export interface PluginManifest {
	id: string;
	name: string;
	version: string;
	description?: string;
	author?: string;

	/** Plugin type */
	plugin_type: 'provider' | 'http_class';

	/** For HTTP class plugins */
	http_class_type?: 'node' | 'filter';
	class_id?: string;

	/** For provider plugins */
	provider_type?: string;

	/** Entry point file */
	entry_point: string;

	/** Optional styles */
	styles?: string;

	/** Dependencies */
	dependencies?: Record<string, string>;
}
