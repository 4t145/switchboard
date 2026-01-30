<script lang="ts">
	import { httpClassEditorRegistry } from '$lib/plugins/registry';
	import LinkOrValueEditor from './link-or-value-editor.svelte';

	type Props = {
		value: any;
		classId: string;
		instanceId: string;
		instanceType?: 'node' | 'filter';
		readonly?: boolean;
	};

	let { value = $bindable(), classId, instanceId, readonly = false }: Props = $props();

	let plugin = $derived(httpClassEditorRegistry.get(classId));
</script>

{#if plugin}
	{#snippet editor(value: unknown, onValueChange: (saveValue: unknown) => void)}
		{@const Editor = plugin.component}
		<Editor {value} {instanceId} {readonly} {onValueChange}></Editor>
	{/snippet}
	<LinkOrValueEditor
		bind:value
		valueDataFormat="object"
		getDefaultInlineValue={plugin.createDefaultConfig}
		{editor}
	/>
{:else}
	<div class="alert alert-warning">
		<div class="flex items-center space-x-2">
			<span>No editor available for HTTP Class "{classId}"</span>
		</div>
	</div>
{/if}
