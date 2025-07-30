<script module>
	export type ServiceNodeType = Node<
		{
			class: string;
			name: string;
			connection: {
				source: string | null;
				target: string | null;
			};
		},
		'service'
	>;
</script>

<script lang="ts">
	import { Position, useSvelteFlow, type NodeProps, type Node, Handle } from '@xyflow/svelte';
	import { Button } from 'bits-ui';
	import { Server } from 'lucide-svelte';
	let { id, data }: NodeProps<ServiceNodeType> = $props();
	const isConnectableEnd = $derived(data.connection.source === null);
	const isConnectableStart = $derived(data.connection.target === null);
	let { updateNodeData } = useSvelteFlow();
</script>

<div class="rounded-br-3xl rounded-bl-3xl rounded-tr rounded-tl border bg-white px-1 shadow-sm transition-shadow duration-200 hover:shadow-md pb-4">
    <div class="flex items-center justify-between">
        <span>
            <Server class="inline" size="1em"></Server>
            <span class="rounded bg-pink-500 px-1 text-sm font-semibold text-white">
                {data.class}
            </span>
            <span>
                {data.name}
            </span>
        </span>
    </div>
</div>
<Handle
	type="target"
	position={Position.Top}
	{isConnectableEnd}
	class="flex h-[10px] w-[18px] items-center justify-center !border-none !bg-transparent p-0 hover:!bg-gray-200"
>
	<svg width="18" height="10" viewBox="0 0 18 10" fill="none" xmlns="http://www.w3.org/2000/svg">
		<polygon points="9,10 0,0 18,0" fill="#000000" stroke="#9CA3AF" />
	</svg>
</Handle>