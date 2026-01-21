import type { HttpClassEditorPlugin } from '$lib/plugins/types';
import ReverseProxyEditor from './reverse-proxy-editor.svelte';

/**
 * Reverse Proxy Node Editor Plugin
 */
export const reverseProxyEditorPlugin: HttpClassEditorPlugin = {
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
