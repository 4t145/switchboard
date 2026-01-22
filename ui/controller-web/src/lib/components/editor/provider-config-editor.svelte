<script lang="ts">
	import { providerEditorRegistry } from '$lib/plugins/registry';
	import { onMount } from 'svelte';
	import LinkOrValueEditor from './link-or-value-editor.svelte';

	type Props = {
		value: any;
		provider: string;
		dataType?: string;
	};

	let { value = $bindable(), provider, dataType = 'TcpServiceConfig' }: Props = $props();

	// Query the plugin registry for this provider
	let plugin = $derived(providerEditorRegistry.get(provider));

	// Track previous provider to detect changes
	let previousProvider = $state(provider);
	let isInitialized = $state(false);

	// Initialize value ONLY if it's truly undefined/null, or if provider changed
	$effect(() => {
		const providerChanged = previousProvider !== provider;
		// Only consider it "empty" if it's undefined or null, not an empty object
		const valueEmpty = value === undefined || value === null;
		// Initialize only when:
		// 1. First time loading (not initialized yet) AND value is truly empty
		// 2. Provider changed
		if ((valueEmpty && !isInitialized) || providerChanged) {
			if (plugin?.createDefaultConfig) {
				value = plugin.createDefaultConfig();
				$inspect(`Initialized config for provider "${provider}" using plugin default config.`, value);
			} else if (valueEmpty) {
				value = {};
			}
			isInitialized = true;
		}

		previousProvider = provider;
	});
</script>

<!-- Wrap the editor in LinkOrValueEditor to support file://, storage://, http:// references -->
<LinkOrValueEditor
	bind:value
	{dataType}
	editorProps={{ provider }}
/>
