<script lang="ts">
	import LinkOrValueEditor from "$lib/components/editor/link-or-value-editor.svelte";
	import type { InstanceDataWithoutType } from "../types";

    type Props = {
        instanceType: 'node' | 'filter',
        instanceId: string,
        config: InstanceDataWithoutType,
    }
	let {
        instanceType,
        instanceId,
        config = $bindable()
	}: Props = $props();
</script>

<div>
    <span class="badge">{config.class}</span>
    {instanceId}
</div>

{#snippet editor(value: unknown, onSave: (saveValue: unknown) => void) }

{/snippet}
<LinkOrValueEditor bind:value={config.config} valueDataFormat="object" editor = {editor} getDefaultInlineValue={() => ({})}/>

