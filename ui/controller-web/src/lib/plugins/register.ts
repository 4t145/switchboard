/**
 * Central plugin registration
 * This file registers all built-in plugins
 */

import { providerEditorRegistry } from './registry';
import { portForwardEditorPlugin } from './providers/pf';
import { httpClassEditorRegistry, httpEditorPlugin } from './providers/http';

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
	providerEditorRegistry.register(portForwardEditorPlugin); 

	// Register HTTP Class Editors - Nodes
	console.log('[Plugins] 🔌 Registering HTTP class editors...');
	httpClassEditorRegistry.prelude();

	const elapsed = (performance.now() - startTime).toFixed(2);
	console.log(`[Plugins] ✅ Built-in plugin registration complete (${elapsed}ms)`);
	console.log('[Plugins] 📊 Summary:', {
		providers: providerEditorRegistry.getAll(),
		httpNodes: httpClassEditorRegistry.getAllNodes().map((p) => p.classId),
		httpFilters: httpClassEditorRegistry.getAllFilters().map((p) => p.classId)
	});
}
