<script module>
	export type RouterNodeType = Node<
		{
			class: string;
			name: string;
			connection: {
				output: Record<string, string | null>;
			};
		},
		'router'
	>;
</script>

<script lang="ts">
	import { Position, useSvelteFlow, type NodeProps, type Node, Handle } from '@xyflow/svelte';
	import { Button } from 'bits-ui';
	import { Split } from 'lucide-svelte';
	let { id, data }: NodeProps<RouterNodeType> = $props();
	let { updateNodeData } = useSvelteFlow();
</script>

<div class="rounded border bg-white px-1 shadow-sm transition-shadow duration-200 hover:shadow-md ">
    <div class="flex items-center justify-between">
        <span>
            <Split class="inline" size="1em"></Split>
            <span class="rounded bg-cyan-500 px-1 text-sm font-semibold text-white">
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
	class="flex h-[10px] w-[18px] items-center justify-center !border-none !bg-transparent p-0 hover:!bg-gray-200"
>
	<svg width="18" height="10" viewBox="0 0 18 10" fill="none" xmlns="http://www.w3.org/2000/svg">
		<polygon points="9,10 0,0 18,0" fill="#000000" stroke="#9CA3AF" />
	</svg>
</Handle>

<Handle
	type="source"
	position={Position.Bottom}
	class="flex h-[10px] w-[18px] items-center justify-center !border-none !bg-transparent p-0 hover:!bg-gray-200"
>
	<svg width="18" height="10" viewBox="0 0 18 10" fill="none" xmlns="http://www.w3.org/2000/svg">
		<polygon points="9,10 0,0 18,0" fill="#000000" stroke="#9CA3AF" />
	</svg>
</Handle>
