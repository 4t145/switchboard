import type { ProviderEditorPlugin } from '../../types';
import HttpEditor from './http-editor.svelte';
import type { HttpConfig } from './types';

/**
 * HTTP Provider Editor Plugin
 */
export const httpEditorPlugin: ProviderEditorPlugin = {
	provider: 'http',
	displayName: 'HTTP Service',
	component: HttpEditor,
	
	createDefaultConfig(): HttpConfig {
		return {
			flow: {
				entrypoint: { node: 'main' },
				instances: {},
				nodes: {},
				filters: {},
				options: {}
			},
			server: {
				version: 'auto'
			}
		};
	},

	validate(config: any) {
		const errors: string[] = [];

		if (!config.flow) {
			errors.push('Missing "flow" configuration');
		} else {
			if (!config.flow.entrypoint) {
				errors.push('Missing flow entrypoint');
			}
		}

		if (!config.server) {
			errors.push('Missing "server" configuration');
		}

		return {
			valid: errors.length === 0,
			errors
		};
	}
};
