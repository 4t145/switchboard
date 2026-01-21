import type { PluginManifest } from './types';

/**
 * Load all plugins from server
 */
export async function loadPluginsFromServer(): Promise<void> {
	if (typeof window === 'undefined') {
		// Skip during SSR
		return;
	}

	try {
		console.log('🔌 Loading plugins from server...');

		// 1. Fetch plugin list
		const response = await fetch('/api/plugins');

		if (!response.ok) {
			// If endpoint doesn't exist yet (backend not implemented), silently skip
			if (response.status === 404) {
				console.log('ℹ️ Plugin API not available (endpoint not found)');
				return;
			}
			throw new Error(`Failed to fetch plugin list: ${response.statusText}`);
		}

		const plugins: PluginManifest[] = await response.json();

		if (plugins.length === 0) {
			console.log('ℹ️ No third-party plugins found');
			return;
		}

		console.log(`📦 Found ${plugins.length} plugin(s)`);

		// 2. Load each plugin
		for (const plugin of plugins) {
			await loadPlugin(plugin);
		}

		console.log('✅ All plugins loaded successfully');
	} catch (error) {
		console.error('❌ Failed to load plugins from server:', error);
		// Don't throw - allow app to continue without third-party plugins
	}
}

/**
 * Load a single plugin
 */
async function loadPlugin(manifest: PluginManifest): Promise<void> {
	try {
		// 1. Load styles if present
		if (manifest.styles) {
			await loadPluginStyles(manifest.id, manifest.styles);
		}

		// 2. Load script
		await loadPluginScript(manifest.id, manifest.entry_point);

		console.log(`  ✅ ${manifest.name} v${manifest.version}`);
	} catch (error) {
		console.error(`  ❌ Failed to load plugin ${manifest.id}:`, error);
		throw error;
	}
}

/**
 * Load plugin stylesheet
 */
function loadPluginStyles(pluginId: string, stylesFile: string): Promise<void> {
	return new Promise((resolve, reject) => {
		const link = document.createElement('link');
		link.rel = 'stylesheet';
		link.href = `/api/plugins/${pluginId}/${stylesFile}`;

		link.onload = () => resolve();
		link.onerror = () => reject(new Error(`Failed to load styles: ${stylesFile}`));

		document.head.appendChild(link);
	});
}

/**
 * Load plugin script
 */
function loadPluginScript(pluginId: string, scriptFile: string): Promise<void> {
	return new Promise((resolve, reject) => {
		const script = document.createElement('script');
		script.src = `/api/plugins/${pluginId}/${scriptFile}`;
		script.async = true;

		// Allow modern ES modules if the file ends with .mjs or has type="module"
		if (scriptFile.endsWith('.mjs')) {
			script.type = 'module';
		}

		script.onload = () => resolve();
		script.onerror = () => reject(new Error(`Failed to load script: ${scriptFile}`));

		document.head.appendChild(script);
	});
}
