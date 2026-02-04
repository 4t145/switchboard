<script lang="ts">
	import { Handle, Position, useSvelteFlow, type Node, type NodeProps } from '@xyflow/svelte';
	import {
		ArrowUpDownIcon,
		FilterIcon,
		FunnelIcon,
		ServerIcon,
		SplitIcon,
		SquareIcon
	} from '@lucide/svelte';
	import '@xyflow/svelte/dist/style.css';
	import type { FlowGraph, FlowGraphNode } from './flow-view-builder';
	import { Collapsible, Listbox } from '@skeletonlabs/skeleton-svelte';
	type NodeData = {
		graphReference: FlowGraph;
		node: FlowGraphNode;
	};
	let {
		id,
		data,
		selected,
		selectable,
		sourcePosition,
		targetPosition,
		type
	}: NodeProps<Node<NodeData>> = $props();
	let { updateNodeData } = useSvelteFlow();
	let isEntrypoint = $derived(data.node.id === data.graphReference.entrypoint);
	let outputFilterTree;
</script>

{#if !isEntrypoint}
	<Handle type="target" position={Position.Left} id="input" class="mb-2"></Handle>
{/if}
<div class="flex flex-col card preset-filled-surface-100-900 {selected ? 'selected' : ''}">
	<div class="flex items-center preset-tonal-primary p-1">
		<ServerIcon size={16} />
		<span>{data.node.data.class}</span>

		{#if isEntrypoint}
			<span>entrypoint</span>
		{/if}
	</div>
	<div class="flex items-center preset-tonal-surface p-1">
		<span>{data.node.id}</span>
	</div>
	<Collapsible class="-4 mx-auto w-56 items-start card">
		<div class="flex w-full items-center justify-between">
			<Collapsible.Trigger class="btn-icon hover:preset-tonal">
				<FunnelIcon class="size-4" />
			</Collapsible.Trigger>
		</div>
		<Collapsible.Content class="flex flex-col gap-2"></Collapsible.Content>
	</Collapsible>
</div>
<Handle type="target" position={Position.Right} id="output" class="mb-2"></Handle>
