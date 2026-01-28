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
<LinkOrValueEditor bind:value={config.config} dataType="HttpInstanceConfig" editorProps = {
    {
        classId: config.class,
        instanceType: instanceType,
        readonly: false
    }
}/>

