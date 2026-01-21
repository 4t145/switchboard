/**
 * Central plugin registration
 * This file registers all built-in plugins
 */

import { providerEditorRegistry, httpClassEditorRegistry } from './registry';
import { httpEditorPlugin } from './providers/http';
import { reverseProxyEditorPlugin } from './providers/http/classes/nodes';

/**
 * Register all built-in plugins
 * Call this on app initialization
 */
export function registerAllPlugins() {
	console.log('🔌 Registering built-in plugins...');

	// Register provider plugins
	providerEditorRegistry.register(httpEditorPlugin);

	// Register HTTP class plugins (nodes/filters)
	httpClassEditorRegistry.register(reverseProxyEditorPlugin);

	console.log('✅ Built-in plugins registered');
}
