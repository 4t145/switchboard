<script module>
	export type NoteNodeType = Node<
		{
			content: string;
		},
		'note'
	>;
</script>

<script lang="ts">
	import { Position, useSvelteFlow, type NodeProps, type Node, Handle } from '@xyflow/svelte';
	import { Button } from 'bits-ui';
	import { Layers, Plus } from 'lucide-svelte';
	let { id, data }: NodeProps<NoteNodeType> = $props();

</script>

<div class="rounded border bg-white px-4 py-2 shadow-sm transition-shadow duration-200 hover:shadow-md bg-yellow-50">
    {data.content}
</div>
