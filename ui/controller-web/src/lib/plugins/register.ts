/**
 * Central plugin registration
 * This file registers all built-in plugins
 */

import { providerEditorRegistry, httpClassEditorRegistry } from './registry';
import { httpEditorPlugin } from './providers/http';
import {
	reverseProxyEditorPlugin,
	routerEditorPlugin,
	directResponseEditorPlugin,
	staticResponseEditorPlugin
} from './providers/http/classes/nodes';
import {
	urlRewriteEditorPlugin,
	requestHeaderModifyEditorPlugin,
	responseHeaderModifyEditorPlugin,
	requestRedirectEditorPlugin
} from './providers/http/classes/filters';

/**
 * Register all built-in plugins
 * Called during application initialization
 */
export function registerAllPlugins() {
	console.log('[Plugins] 🚀 Starting built-in plugin registration...');
	const startTime = performance.now();

	// Register Provider Editors
	console.log('[Plugins] 📦 Registering provider editors...');
	providerEditorRegistry.register(httpEditorPlugin);

	// Register HTTP Class Editors - Nodes
	console.log('[Plugins] 🔌 Registering HTTP class editors (nodes)...');
	httpClassEditorRegistry.register(routerEditorPlugin);
	httpClassEditorRegistry.register(reverseProxyEditorPlugin);
	httpClassEditorRegistry.register(directResponseEditorPlugin);
	httpClassEditorRegistry.register(staticResponseEditorPlugin);

	// Register HTTP Class Editors - Filters
	console.log('[Plugins] 🔌 Registering HTTP class editors (filters)...');
	httpClassEditorRegistry.register(urlRewriteEditorPlugin);
	httpClassEditorRegistry.register(requestHeaderModifyEditorPlugin);
	httpClassEditorRegistry.register(responseHeaderModifyEditorPlugin);
	httpClassEditorRegistry.register(requestRedirectEditorPlugin);

	const elapsed = (performance.now() - startTime).toFixed(2);
	console.log(`[Plugins] ✅ Built-in plugin registration complete (${elapsed}ms)`);
	console.log('[Plugins] 📊 Summary:', {
		providers: providerEditorRegistry.getAll(),
		httpNodes: httpClassEditorRegistry.getAllNodes().map((p) => p.classId),
		httpFilters: httpClassEditorRegistry.getAllFilters().map((p) => p.classId)
	});
}
