import type { ProviderEditorPlugin } from '../../types';
import HttpEditor from './http-editor.svelte';
import type { HttpConfig } from './types';
export { httpClassEditorRegistry, getHttpClassEditorPlugin } from './classes';
/**
 * HTTP Provider Editor Plugin
 */
export const httpEditorPlugin: ProviderEditorPlugin<HttpConfig> = {
	provider: 'http',
	displayName: 'HTTP Service',
	component: HttpEditor,

	createDefaultConfig(): HttpConfig {
		return {
			flow: {
				entrypoint: 'router',
				nodes: {},
				filters: {},
				options: {}
			},
			server: {
				version: 'auto'
			}
		};
	},

	validate(config: HttpConfig) {
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
