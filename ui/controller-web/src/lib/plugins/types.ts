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
export interface ProviderEditorProps {
	/** Current configuration value (can be object or LinkOrValue string) */
	value: any;
	/** Whether the editor is read-only */
	readonly?: boolean;
}

/**
 * Provider editor plugin interface
 */
export interface ProviderEditorPlugin {
	/** Provider name (e.g., "http", "socks5") */
	provider: string;

	/** Editor component */
	component: Component<ProviderEditorProps>;

	/** Display name */
	displayName: string;

	/** Description */
	description?: string;

	/** Default configuration generator */
	createDefaultConfig: () => any;

	/** Configuration validator (optional) */
	validate?: (config: any) => ValidationResult;
}

/**
 * Props for HTTP class editor components
 */
export interface HttpClassEditorProps {
	/** Current configuration value */
	value: any;
	/** Instance ID (for debugging/tracking) */
	instanceId?: string;
	/** Whether the editor is read-only */
	readonly?: boolean;
}

/**
 * Output information extracted from node config
 */
export interface OutputInfo {
	/** Output port name (e.g., "api", "frontend", "$default") */
	port: string;

	/** Target node ID */
	target: string;

	/** Filters applied on this output (optional) */
	filters?: string[];

	/** Display label (optional, defaults to port name) */
	label?: string;
}

/**
 * HTTP class (node/filter) editor plugin interface
 */
export interface HttpClassEditorPlugin {
	/** Class ID (e.g., "reverse-proxy", "balancer") */
	classId: string;

	/** Type (node or filter) */
	type: 'node' | 'filter';

	/** Editor component */
	component: Component<HttpClassEditorProps>;

	/** Display name */
	displayName: string;

	/** Icon name (lucide-svelte icon name) */
	icon?: string;

	/** Description */
	description?: string;

	/** Default configuration generator */
	createDefaultConfig: () => any;

	/** Configuration validator (optional) */
	validate?: (config: any) => ValidationResult;

	/**
	 * Extract output connections from node config
	 * Used for visualizing node connections in Flow Editor
	 * @param config - Node configuration object
	 * @returns Array of output information
	 */
	extractOutputs?: (config: any) => OutputInfo[];
}

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
