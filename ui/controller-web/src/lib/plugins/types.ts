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
export type ProviderEditorProps<T = unknown> ={
	/** Current configuration value (can be object or LinkOrValue string) */
	value: T;
	/** Whether the editor is read-only */
	readonly?: boolean;
}

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
}

/**
 * Props for HTTP class editor components
 */
export interface HttpClassEditorProps<T = unknown> {
	/** Current configuration value */
	value: T;
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

export interface HttpNodeClassPlugin<T = unknown> {
	classId: string;
	component: Component<HttpClassEditorProps<T>>;
	type: 'node';
	displayName: string;
	icon?: string;
	description?: string;
	createDefaultConfig: () => T;
	extractOutputs: (config: T) => OutputInfo[];
	validate?: (config: T) => ValidationResult;
}

export interface HttpFilterClassPlugin<T = unknown> {
	classId: string;
	component: Component<HttpClassEditorProps<T>>;
	type: 'filter';
	displayName: string;
	icon?: string;
	description?: string;
	createDefaultConfig: () => T;
	validate?: (config: T) => ValidationResult;
}

/**
 * HTTP class (node/filter) editor plugin interface
 */
export type HttpClassPlugin<T = unknown> = HttpNodeClassPlugin<T> | HttpFilterClassPlugin<T>;

export type HttpClassEditorPlugin<T = unknown> = HttpClassPlugin<T>;

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
