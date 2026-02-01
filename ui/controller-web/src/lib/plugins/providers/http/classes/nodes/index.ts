import type { OutputInterface } from '$lib/api/types/http';
import type {
	HttpNodeClassPlugin,
	HttpClassEditorPlugin,
	OutputInfo,
	InputInfo
} from '$lib/plugins/types';
import { NodeTarget } from '../../types';
import ReverseProxyEditor, { type ReverseProxyConfig } from './reverse-proxy-editor.svelte';
import RouterEditor, { type RouterConfig } from './router-editor.svelte';
import StaticResponseEditor, { type StaticResponseConfig } from './static-response-editor.svelte';

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
			([port, outputDef]: [string, OutputInterface]) =>
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

	validate(config: any) {
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
		}
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