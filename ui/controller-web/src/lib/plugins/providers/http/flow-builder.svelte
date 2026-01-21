<script lang="ts">
	import { SvelteFlow, Background, Controls, Panel, type Node, type Edge } from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';
	import type { FlowConfig } from './types';
	import { flowToNodes, nodesToFlow, autoLayoutNodes } from './flow-utils';
	import { Plus } from 'lucide-svelte';

	type Props = {
		value: FlowConfig;
	};

	let { value = $bindable() }: Props = $props();

	// Convert FlowConfig to SvelteFlow format
	let { nodes: initialNodes, edges: initialEdges } = flowToNodes(value);
	initialNodes = autoLayoutNodes(initialNodes);

	// Use $state.raw for performance (don't make every property reactive)
	let nodes = $state.raw<Node[]>(initialNodes);
	let edges = $state.raw<Edge[]>(initialEdges);

	// Sync back to FlowConfig when nodes/edges change
	// Use event handlers instead of $effect to avoid infinite loops
	function syncToFlowConfig() {
		value = nodesToFlow(nodes, edges, value.entrypoint);
	}

	// Node palette - available node types
	const nodeTypes = [
		{ id: 'reverse-proxy', label: 'Reverse Proxy', icon: '🔀' },
		{ id: 'balancer', label: 'Load Balancer', icon: '⚖️' },
		{ id: 'router', label: 'Router', icon: '🔀' },
		{ id: 'static-response', label: 'Static Response', icon: '📄' }
	];

	const filterTypes = [
		{ id: 'request-header-modify', label: 'Request Headers', icon: '📝' },
		{ id: 'response-header-modify', label: 'Response Headers', icon: '📝' },
		{ id: 'url-rewrite', label: 'URL Rewrite', icon: '🔗' },
		{ id: 'request-redirect', label: 'Redirect', icon: '↩️' }
	];

	// Add new node
	function addNode(classId: string, instanceType: 'node' | 'filter') {
		const newNode: Node = {
			id: `${instanceType}-${Date.now()}`,
			type: 'custom',
			position: { x: 100, y: 100 },
			data: {
				label: classId,
				classId,
				instanceType,
				config: {}
			}
		};

		nodes = [...nodes, newNode];
		syncToFlowConfig();
	}
</script>

<div class="flow-builder h-full w-full">
	<SvelteFlow 
		bind:nodes 
		bind:edges 
		fitView
		onnodedragstop={syncToFlowConfig}
		onnodeschange={syncToFlowConfig}
		onedgeschange={syncToFlowConfig}
		onconnect={syncToFlowConfig}
		ondelete={syncToFlowConfig}
	>
		<Background />
		<Controls />

		<!-- Node Palette Panel -->
		<Panel position="top-left">
			<div class="card space-y-3 border border-surface-300 bg-white p-3 dark:border-surface-600 dark:bg-surface-800">
				<div>
					<h3 class="mb-2 text-xs font-bold uppercase text-surface-600 dark:text-surface-400">
						Nodes
					</h3>
					<div class="flex flex-wrap gap-2">
						{#each nodeTypes as nodeType}
							<button
								class="btn btn-sm preset-tonal-surface flex items-center gap-1"
								onclick={() => addNode(nodeType.id, 'node')}
								title="Add {nodeType.label}"
							>
								<Plus size={14} />
								<span class="text-xs">{nodeType.icon} {nodeType.label}</span>
							</button>
						{/each}
					</div>
				</div>

				<div>
					<h3 class="mb-2 text-xs font-bold uppercase text-surface-600 dark:text-surface-400">
						Filters
					</h3>
					<div class="flex flex-wrap gap-2">
						{#each filterTypes as filterType}
							<button
								class="btn btn-sm preset-tonal-surface flex items-center gap-1"
								onclick={() => addNode(filterType.id, 'filter')}
								title="Add {filterType.label}"
							>
								<Plus size={14} />
								<span class="text-xs">{filterType.icon} {filterType.label}</span>
							</button>
						{/each}
					</div>
				</div>
			</div>
		</Panel>

		<!-- Info Panel -->
		<Panel position="bottom-right">
			<div class="card border border-surface-300 bg-white/90 p-2 text-xs dark:border-surface-600 dark:bg-surface-800/90">
				<div class="opacity-75">
					Nodes: {nodes.length} | Edges: {edges.length}
				</div>
			</div>
		</Panel>
	</SvelteFlow>
</div>

<style>
	.flow-builder {
		min-height: 500px;
	}

	:global(.svelte-flow) {
		background-color: rgb(var(--color-surface-50));
	}

	:global(.dark .svelte-flow) {
		background-color: rgb(var(--color-surface-900));
	}
</style>
