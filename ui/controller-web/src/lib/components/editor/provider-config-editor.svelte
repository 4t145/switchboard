<script lang="ts">
	import { providerEditorRegistry } from '$lib/plugins/registry';
	import LinkOrValueEditor from './link-or-value-editor.svelte';
	import { AlertCircle } from 'lucide-svelte';

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

	// Initialize value if it's undefined/null/empty, or if provider changed
	$effect(() => {
		const providerChanged = previousProvider !== provider;
		const valueEmpty = !value || (typeof value === 'object' && Object.keys(value).length === 0);

		if (valueEmpty || providerChanged) {
			if (plugin?.createDefaultConfig) {
				value = plugin.createDefaultConfig();
				console.log(`📝 Initialized config for provider "${provider}":`, value);
			} else if (valueEmpty) {
				value = {};
			}
		}

		previousProvider = provider;
	});

	// Default config when switching to inline mode in LinkOrValueEditor
	function getDefaultValue() {
		if (plugin?.createDefaultConfig) {
			return plugin.createDefaultConfig();
		}
		return {};
	}
</script>

{#snippet inlineEditorSnippet()}
	{#if plugin}
		{@const EditorComponent = plugin.component}
		<!-- Render the plugin's component -->
		<div class="plugin-editor">
			<EditorComponent bind:value />
		</div>
	{:else}
		<!-- Fallback: JSON editor with warning -->
		<div class="space-y-3">
			<div class="alert preset-tonal-warning">
				<AlertCircle size={16} class="inline-block" />
				<span class="font-medium">No editor available for provider "{provider}"</span>
			</div>
			<div class="form-control">
				<label class="label">
					<span class="label-text">Configuration (JSON)</span>
				</label>
				<textarea
					class="textarea-bordered textarea h-48 font-mono text-xs"
					value={JSON.stringify(value, null, 2)}
					oninput={(e) => {
						try {
							value = JSON.parse(e.currentTarget.value);
						} catch (err) {
							// Ignore parse errors while typing
						}
					}}
				></textarea>
				<div class="label">
					<span class="label-text-alt opacity-75"
						>Enter valid JSON configuration for this provider.</span
					>
				</div>
			</div>
		</div>
	{/if}
{/snippet}

<!-- Wrap the editor in LinkOrValueEditor to support file://, storage://, http:// references -->
<LinkOrValueEditor
	bind:value
	{dataType}
	dataFormat="object"
	renderValue={inlineEditorSnippet}
	defaultValue={getDefaultValue}
/>
