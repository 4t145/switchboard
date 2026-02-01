import type { HttpFilterClassPlugin } from '$lib/plugins/types';
import UrlRewriteEditor, { type UrlRewriteFilterConfig } from './url-rewrite-editor.svelte';

/**
 * URL Rewrite Filter Editor Plugin
 */
export const urlRewriteEditorPlugin: HttpFilterClassPlugin<UrlRewriteFilterConfig> = {
	classId: 'url-rewrite',
	type: 'filter',
	displayName: 'URL Rewrite',
	icon: 'Link',
	description: 'Rewrite request URL path',
	component: UrlRewriteEditor,

	createDefaultConfig() {
		return {
			path: null,
			hostname: null
		};
	},

	validate(config: UrlRewriteFilterConfig) {
		const errors: string[] = [];

		if (!config.path) {
			errors.push('Rewrite path is required');
		}

		return {
			valid: errors.length === 0,
			errors
		};
	}
};

/**
 * Request Header Modify Filter Editor Plugin
 */
export const requestHeaderModifyEditorPlugin: HttpFilterClassPlugin = {
	classId: 'request-header-modify',
	type: 'filter',
	displayName: 'Request Headers',
	icon: 'FileEdit',
	description: 'Modify request headers',
	component: UrlRewriteEditor, // TODO: Create dedicated editor

	createDefaultConfig() {
		return {
			set: {},
			remove: []
		};
	}
};

/**
 * Response Header Modify Filter Editor Plugin
 */
export const responseHeaderModifyEditorPlugin: HttpFilterClassPlugin = {
	classId: 'response-header-modify',
	type: 'filter',
	displayName: 'Response Headers',
	icon: 'FileEdit',
	description: 'Modify response headers',
	component: UrlRewriteEditor, // TODO: Create dedicated editor

	createDefaultConfig() {
		return {
			set: {},
			remove: []
		};
	}
};

/**
 * Request Redirect Filter Editor Plugin
 */
export const requestRedirectEditorPlugin: HttpFilterClassPlugin = {
	classId: 'request-redirect',
	type: 'filter',
	displayName: 'Redirect',
	icon: 'CornerUpRight',
	description: 'Redirect requests to another URL',
	component: UrlRewriteEditor, // TODO: Create dedicated editor

	createDefaultConfig() {
		return {
			url: '',
			status: 302
		};
	}
};
