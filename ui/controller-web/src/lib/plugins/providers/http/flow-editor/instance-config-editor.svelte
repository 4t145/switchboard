<script lang="ts">
	import LinkOrValueEditor from '$lib/components/editor/link-or-value-editor.svelte';
	import { httpClassEditorRegistry } from '$lib/plugins/registry';
	import type { InstanceDataWithoutType } from '../types';

	type Props = {
		instanceType: 'node' | 'filter';
		instanceId: string;
		config: InstanceDataWithoutType;
	};
	let { instanceType, instanceId, config = $bindable() }: Props = $props();
	const plugin = $derived(httpClassEditorRegistry.get(config.class));
	let value = $state(config.config);
	$effect(() => {
		config.config = value;
	});
</script>

<div class="mb-4 flex items-center gap-2">
	<span class="text-lg font-bold"> {instanceId}</span>
	<span class="badge code">{config.class}</span>
</div>
{#if plugin}
	{@const Editor = plugin.component}
	{#snippet editor(value: unknown, onValueChange: (saveValue: unknown) => void)}
		<Editor {value} {onValueChange} />
	{/snippet}
	<LinkOrValueEditor
		bind:value
		valueDataFormat="object"
		{editor}
		getDefaultInlineValue={plugin.createDefaultConfig}
	/>
{:else}
	<div class="preset-outlined-warning card">unregistered class: {config.class}</div>
{/if}
