<script lang="ts">
	import {
		type Node,
		type Edge,
		type NodeTypes,
		SvelteFlow,
		Background,
		Controls,
		MiniMap,
		MarkerType
	} from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';
	import type { FlowGraph } from './flow-view-builder';
	import FlowGraphNode from './flow-graph-node.svelte';
	import { convertFlowGraphToSvelteFlow } from './graph-layout';
	import { untrack } from 'svelte';

	type Props = {
		graph: FlowGraph;
		selected: string | undefined;
	};

	let { graph, selected = $bindable(undefined) }: Props = $props();

	const nodeTypes: NodeTypes = {
		custom: FlowGraphNode
	};

	let nodes = $state<Node[]>([]);
	let edges = $state<Edge[]>([]);

	$effect(() => {
		const layout = untrack(() => convertFlowGraphToSvelteFlow(graph));
		nodes = layout.nodes.map((node) => ({
			...node,
			selected: node.id === selected
		}));
		edges = layout.edges;
	});

	function handleSelectionChange(event: { nodes: Node[] }) {
		const selectedNode = event.nodes[0];
		selected = selectedNode ? selectedNode.id : undefined;
	}
</script>

<div class="flow-graph-container">
	<SvelteFlow
		{nodes}
		{edges}
		{nodeTypes}
		fitView
		nodesDraggable={false}
		nodesConnectable={false}
		elementsSelectable={true}
		selectNodesOnDrag={false}
		onselectionchange={handleSelectionChange}
	>
		<Background />
		<Controls />
		<MiniMap />
	</SvelteFlow>
</div>

<style>
	.flow-graph-container {
		width: 100%;
		height: 100%;
	}

	:global(.svelte-flow) {
		background-color: var(--md-sys-color-surface-container-low);
	}

	:global(.svelte-flow__minimap) {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline);
	}

	:global(.svelte-flow__controls) {
		button {
			background-color: var(--md-sys-color-surface);
			border: 1px solid var(--md-sys-color-outline);
			color: var(--md-sys-color-on-surface);
		}

		button:hover {
			background-color: var(--md-sys-color-surface-container-high);
		}
	}

	:global(.svelte-flow__edge-path) {
		stroke: var(--md-sys-color-outline);
		stroke-width: 2;
	}

	:global(.svelte-flow__edge.selected .svelte-flow__edge-path) {
		stroke: var(--md-sys-color-primary);
		stroke-width: 3;
	}
</style>
