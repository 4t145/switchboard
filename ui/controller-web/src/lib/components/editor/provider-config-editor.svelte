<script lang="ts">
	import { providerEditorRegistry } from '$lib/plugins/registry';
	import LinkOrValueEditor from './link-or-value-editor.svelte';

	type Props = {
		value: any;
		provider: string;
		readonly?: boolean;
	};

	let { value = $bindable(), provider = $bindable(), readonly = false }: Props = $props();

	let plugin = $derived(providerEditorRegistry.get(provider));
</script>

{#if plugin}
	{#snippet editor(value: unknown, onValueChange: (saveValue: unknown) => void)}
		{@const Editor = plugin.component}
		<Editor {value} {readonly} {onValueChange}></Editor>
	{/snippet}
	<LinkOrValueEditor
		bind:value
		valueDataFormat="object"
		getDefaultInlineValue={plugin.createDefaultConfig}
		{editor}
	></LinkOrValueEditor>
{:else}
	<div class="alert alert-warning">
		<div class="flex items-center space-x-2">
			<span>No editor available for provider "{provider}"</span>
		</div>
	</div>
{/if}
