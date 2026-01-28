<script lang="ts">
	import LinkOrValueEditor from './link-or-value-editor.svelte';

	type Props = {
		value: any;
		classId: string;
		instanceType?: 'node' | 'filter';
		readonly?: boolean;
	};

	let {
		value = $bindable(),
		classId,
		instanceType = 'node',
		readonly = false
	}: Props = $props();
</script>

<LinkOrValueEditor
	bind:value
	dataType="HttpInstanceConfig"
	editorProps={{ classId, instanceType, readonly }}
/>
