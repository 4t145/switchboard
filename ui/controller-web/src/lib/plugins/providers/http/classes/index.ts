import type { Component } from 'svelte';
import type { HttpEditorContext, NodeTargetObject } from '../types';

import { NodeTarget } from '../types';
import {
	ReverseProxyEditor,
	RouterEditor,
	StaticResponseEditor,
	type ReverseProxyConfig,
	type RouterConfig,
	type StaticResponseConfig
} from './nodes';
import { urlRewriteEditorPlugin } from './filters';

/**
 * Plugin validation result
 */
export interface ValidationResult {
	valid: boolean;
	errors?: string[];
	warnings?: string[];
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
	onValueChange: (value: T) => void;
	httpEditorContext: HttpEditorContext;
}

/**
 * Output information extracted from node config
 */
export interface OutputInfo {
	/** Output port name (e.g., "api", "frontend", "$default") */
	port: string;

	/** Target node ID */
	target: NodeTargetObject;

	/** Filters applied on this output (optional) */
	filters: string[];
}

export interface InputInfo {
	port: string;
	filters: string[];
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
	extractInputs: (config: T) => InputInfo[];
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
 * HTTP class editor registry (for HTTP nodes and filters)
 */
class HttpClassEditorRegistry {
	private nodes = new Map<string, HttpNodeClassPlugin>();
	private filters = new Map<string, HttpFilterClassPlugin>();
	prelude() {
		this.registerNode(routerEditorPlugin);
		this.registerNode(reverseProxyEditorPlugin);
		this.registerNode(staticResponseEditorPlugin);
		this.registerFilter(urlRewriteEditorPlugin);
	}
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

export const httpClassEditorRegistry = new HttpClassEditorRegistry();

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

const DefaultInputInfo: InputInfo = {
	port: '$default',
	filters: []
};

/**
 * Router Node Editor Plugin
 */
export const routerEditorPlugin: HttpNodeClassPlugin<RouterConfig> = {
	classId: 'router',
	type: 'node',
	displayName: 'Router',
	icon: 'GitBranch',
	description: 'Route requests based on hostname and path patterns',
	component: RouterEditor,

	createDefaultConfig() {
		return {
			hostname: {},
			path: {},
			output: {}
		};
	},

	extractOutputs(config: RouterConfig): OutputInfo[] {
		console.log('Extracting outputs from router config:', config);
		if (!config.output || typeof config.output !== 'object') {
			return [];
		}
		return Object.entries(config.output).map(
			([port, outputDef]: [
				string,
				{
					target: string;
					filters?: string[];
				}
			]) =>
				<OutputInfo>{
					port,
					target: NodeTarget.parse(outputDef.target),
					filters: outputDef.filters || []
				}
		);
	},
	extractInputs(): InputInfo[] {
		// Router has a single default input
		return [
			{
				...DefaultInputInfo
			}
		];
	},
	validate(config: RouterConfig) {
		const errors: string[] = [];
		const warnings: string[] = [];

		if (!config.output || Object.keys(config.output).length === 0) {
			errors.push('At least one output port must be defined');
		}

		// // Check if routes reference valid output ports
		// const outputPorts = new Set(Object.keys(config.output || {}));
		// const hostnameRoutes = Object.values(config.hostname || {});
		// const pathRoutes = Object.values(config.path || {});

		// [...hostnameRoutes, ...pathRoutes].forEach((port: RouterConfig) => {
		// 	if (port && !outputPorts.has(port) && port !== '$default') {
		// 		warnings.push(`Route references undefined output port: ${port}`);
		// 	}
		// });

		return {
			valid: errors.length === 0,
			errors,
			warnings
		};
	}
};

/**
 * Reverse Proxy Node Editor Plugin
 */
export const reverseProxyEditorPlugin: HttpNodeClassPlugin<ReverseProxyConfig> = {
	classId: 'reverse-proxy',
	type: 'node',
	displayName: 'Reverse Proxy',
	icon: 'ArrowRightLeft',
	description: 'Proxy requests to a backend server',
	component: ReverseProxyEditor,

	createDefaultConfig() {
		return {
			backend: '',
			scheme: 'http',
			timeout: '30s',
			https_only: false
		};
	},

	// Reverse proxy is a terminal node (no outputs)
	extractOutputs(): OutputInfo[] {
		return [];
	},

	extractInputs(): InputInfo[] {
		// Single default input
		return [
			{
				...DefaultInputInfo
			}
		];
	},

	validate(config: ReverseProxyConfig) {
		const errors: string[] = [];

		if (!config.backend) {
			errors.push('Backend address is required');
		} else if (!config.backend.includes(':')) {
			errors.push('Backend must include port (e.g., example.com:8080)');
		}

		if (!['http', 'https'].includes(config.scheme)) {
			errors.push('Scheme must be either "http" or "https"');
		}

		return {
			valid: errors.length === 0,
			errors
		};
	}
};

/**
 * Direct Response Node Editor Plugin
 */
export const staticResponseEditorPlugin: HttpNodeClassPlugin<StaticResponseConfig> = {
	classId: 'static-response',
	type: 'node',
	displayName: 'Static Response',
	icon: 'FileText',
	description: 'Return a static HTTP response',
	component: StaticResponseEditor, // TODO: Create dedicated editor

	createDefaultConfig() {
		return {
			status_code: 200,
			headers: [],
			body: ''
		};
	},

	// Direct response is a terminal node
	extractOutputs(): OutputInfo[] {
		return [];
	},

	extractInputs(): InputInfo[] {
		// Single default input
		return [
			{
				...DefaultInputInfo
			}
		];
	}
};
