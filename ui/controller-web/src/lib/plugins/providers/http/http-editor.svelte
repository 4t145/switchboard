<script lang="ts">
	import type { FlowConfig, HttpConfig } from './types';
	import FlowEditor from './flow-editor/flow-editor.svelte';
	import { untrack } from 'svelte';
	import { buildFlowGraph, FlowGraph } from './flow-editor/flow-view-builder';
	import FlowTree from './flow-editor/flow-tree-view.svelte';
	import FlowGraphView from './flow-editor/flow-graph-view.svelte';
	import { ListIcon, NetworkIcon, WorkflowIcon } from 'lucide-svelte';

	type Props = {
		value: HttpConfig;
	};

	let { value = $bindable() }: Props = $props();

	type ViewMode = 'list' | 'tree' | 'graph';
	type GraphState = {
		readonly type: 'ready';
		readonly graph: FlowGraph;
	} | {
		readonly type: 'error';
		readonly error: Error;
	} | {
		readonly type: 'building';
		readonly promise: Promise<FlowGraph>;
	} | {
		readonly type: 'uninitialized';
	}

	let graphState = $state<GraphState>({ type: 'uninitialized' });
	let viewMode = $state<ViewMode>('tree');
	let selectedValue = $state(undefined as string | undefined);

	async function updateGraphState(flow: FlowConfig) {
		if (graphState.type === 'building') {
			console.warn('Ignoring edits to flow while building graph');
			return;
		}
		try {
			const promise = buildFlowGraph(flow);
			graphState = {
				type: 'building',
				promise
			};
			const graph = await promise;
			graphState = {
				type: 'ready',
				graph
			};
		} catch (error) {
			graphState = {
				type: 'error',
				error: error as Error
			};
		}
	}

	$effect(() => {
		untrack(() => updateGraphState(value.flow));
	})
</script>

<div class="space-y-4">
	<!-- HTTP Version Selector -->
	<label class="label">
		<span class="label-text font-medium">HTTP Version</span>
		{#if value.server}
			<select class="select select-sm" bind:value={value.server.version}>
				<option value="auto">Auto (Negotiate)</option>
				<option value="http1">HTTP/1.1 Only</option>
				<option value="http2">HTTP/2 Only</option>
			</select>
		{/if}
	</label>

	<!-- Flow Editor (Visual Editor) -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<div class="label-text font-medium">Flow Configuration</div>
			<div class="flex gap-2">
				<button
					class="btn btn-sm {viewMode === 'list' ? 'btn-primary' : 'btn-ghost'}"
					onclick={() => viewMode = 'list'}
				>
					<ListIcon class="size-4" />
					List
				</button>
				<button
					class="btn btn-sm {viewMode === 'tree' ? 'btn-primary' : 'btn-ghost'}"
					onclick={() => viewMode = 'tree'}
				>
					<WorkflowIcon class="size-4" />
					Tree
				</button>
				<button
					class="btn btn-sm {viewMode === 'graph' ? 'btn-primary' : 'btn-ghost'}"
					onclick={() => viewMode = 'graph'}
				>
					<NetworkIcon class="size-4" />
					Graph
				</button>
			</div>
		</div>

		<div class="card border border-surface-200 dark:border-surface-700 h-[600px] overflow-hidden">
			{#if graphState.type === 'building'}
			<div>
				<div class="flex flex-col items-center justify-center h-full">
					<span class="loading loading-spinner loading-lg"></span>
					<p class="mt-4 text-sm text-center">Building flow graph...</p>
				</div>
			</div>
			{:else if graphState.type === 'error'}
			<div>
				<div class="flex flex-col items-center justify-center h-full p-4">
					<p class="text-sm text-center text-error">Error building flow graph: {graphState.error.message}</p>
				</div>
			</div>
			{:else if graphState.type === 'ready'}
				{#if viewMode === 'list'}
					<div class="flex flex-col items-center justify-center h-full">
						<p class="text-sm text-center text-muted-foreground">List view coming soon...</p>
					</div>
				{:else if viewMode === 'tree'}
					<FlowTree graph={graphState.graph} bind:selected={selectedValue} />
				{:else if viewMode === 'graph'}
					<FlowGraphView graph={graphState.graph} bind:selected={selectedValue} />
				{/if}
			{:else}
			<div>
				<div class="flex flex-col items-center justify-center h-full">
					<p class="mt-4 text-sm text-center">Initialize flow graph...</p>
				</div>
			</div>
			{/if}
		</div>

		<div class="label">
			<span class="label-text-alt opacity-75">
				Configure your HTTP request flow by adding nodes and filters. Select a node or filter from
				the tree to edit its configuration.
			</span>
		</div>
	</div>
</div>
