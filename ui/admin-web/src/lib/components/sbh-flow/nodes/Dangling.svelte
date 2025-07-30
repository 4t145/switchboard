<script module>
	export type DanglingNodeType = Node<
		{
			connection: {
				source: string | null;
			};
		},
		'dangling'
	>;
</script>

<script lang="ts">
	import { Position, useSvelteFlow, type NodeProps, type Node, Handle } from '@xyflow/svelte';
	import { Button } from 'bits-ui';
	import { Layers, Plus } from 'lucide-svelte';
	let { id, data }: NodeProps<DanglingNodeType> = $props();
	const isConnectableEnd = $derived(data.connection.source === null);
	let { updateNodeData } = useSvelteFlow();
</script>

<button
	class="flex items-center justify-center rounded border bg-white p-0 hover:!border-orange-500"
>
	<Plus size="1em" class="text-orange-500" />
</button>
<Handle
	type="target"
	position={Position.Top}
	{isConnectableEnd}
	class="!border-none !bg-transparent"
></Handle>
