<script lang="ts" module>
	export type IncomeNodeType = Node<
		{
			class: string;
			name: string;
			connection: {
				source: string | null;
				target: string | null;
			};
		},
		'income'
	>;
</script>

<script lang="ts">
	import { Position, useSvelteFlow, type NodeProps, type Node, Handle } from '@xyflow/svelte';
	import { Button } from 'bits-ui';
	import { ArrowBigDown } from 'lucide-svelte';
	let { id, data }: NodeProps<IncomeNodeType> = $props();
	const isConnectableEnd = $derived(data.connection.source === null);
	const isConnectableStart = $derived(data.connection.target === null);
	let { updateNodeData } = useSvelteFlow();
</script>

<div class="rounded-tr-3xl rounded-tl-3xl rounded-br rounded-bl border bg-white px-1 shadow-sm transition-shadow duration-200 hover:shadow-md pt-2 pb-1">
    <div class="flex items-center justify-between">
        <ArrowBigDown class="inline" size="1em"></ArrowBigDown>
    </div>
</div>
<Handle
	type="target"
	position={Position.Bottom}
	{isConnectableEnd}
	class="flex h-[10px] w-[18px] items-center justify-center !border-none !bg-transparent p-0 hover:!bg-gray-200"
>
	<svg width="18" height="10" viewBox="0 0 18 10" fill="none" xmlns="http://www.w3.org/2000/svg">
		<polygon points="9,10 0,0 18,0" fill="#000000" stroke="#9CA3AF" />
	</svg>
</Handle>