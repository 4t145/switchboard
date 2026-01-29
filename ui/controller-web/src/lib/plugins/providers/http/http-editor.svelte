<script lang="ts">
	import type { FlowConfig, HttpConfig } from './types';
	import FlowEditor from './flow-editor/flow-editor.svelte';
	import { untrack } from 'svelte';
	import { buildFlowGraph, FlowGraph, FlowTreeViewInstanceSelection } from './flow-editor/flow-view-builder';
	import FlowTree from './flow-editor/flow-tree-view.svelte';
	import FlowGraphView from './flow-editor/flow-graph-view.svelte';
	import { EditIcon, GripVerticalIcon, ListIcon, MaximizeIcon, MinimizeIcon, MinusIcon, NetworkIcon, View, WorkflowIcon, XIcon } from 'lucide-svelte';
	import { Dialog, FloatingPanel, Portal, SegmentedControl, ToggleGroup } from '@skeletonlabs/skeleton-svelte';
	import InstanceConfigEditor from './flow-editor/instance-config-editor.svelte';

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
	let viewModeSelection = $derived<string[]>([viewMode]);
	let selectedValue = $state(undefined as string | undefined);
	let selectedInstance = $derived.by(() => {
		if (graphState.type !== 'ready' || !selectedValue) {
			return undefined;
		}
		const selection = FlowTreeViewInstanceSelection.fromString(selectedValue);
		return selection;
	});
	let selectedInstanceData = $derived.by(() => {
		if (graphState.type !== 'ready' || !selectedInstance) {
			return undefined;
		}
		if (selectedInstance.type === 'node') {
			const node = value.flow.nodes[selectedInstance.id];
			if (node) {
				return node;
			}
		} else if (selectedInstance.type === 'filter') {
			const filter = value.flow.filters[selectedInstance.id];
			if (filter) {
				return filter;
			}
		}
		return undefined;
	})
	let editDialogOpen = $state(false);
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
			<div class="label-text font-medium flex-grow">Flow Configuration</div>
			<div class="btn-group">
				<button class="btn btn-sm btn-primary" onclick={() => { editDialogOpen = true; }} disabled={selectedInstance === undefined}>
					<EditIcon class="size-4 mr-2" />
				</button>
			</div>
			<ToggleGroup defaultValue={["tree"]} multiple={false} value={viewModeSelection} onValueChange={(details) => { viewModeSelection = details.value; viewMode = details.value[0] as ViewMode; }}>
				<ToggleGroup.Item value="list">
					<ListIcon class="size-4" />
				</ToggleGroup.Item>
				<ToggleGroup.Item value="tree">
					<WorkflowIcon class="size-4" />
				</ToggleGroup.Item>
			</ToggleGroup>
			<SegmentedControl value={viewMode} onValueChange={(details) => {  viewMode = details.value as ViewMode; }} defaultValue="tree" class="btn-group">
				<SegmentedControl.Control>
					<SegmentedControl.Indicator />
					<SegmentedControl.Item
						value="list"
						aria-label="List View"
						title="List View"
					>
						<SegmentedControl.ItemText>
							<ListIcon class="size-4" />
						</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item
						value="tree"
						aria-label="Tree View"
						title="Tree View"
					>

						<SegmentedControl.ItemText>
							<WorkflowIcon class="size-4" />
						</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
				</SegmentedControl.Control>
			</SegmentedControl>
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

<FloatingPanel open={editDialogOpen}>
	<Portal>
		<FloatingPanel.Positioner class="z-50">
			<FloatingPanel.Content>
				<FloatingPanel.DragTrigger>
					<FloatingPanel.Header>
						<FloatingPanel.Title>
							<GripVerticalIcon class="size-4" />
							Floating Panel
						</FloatingPanel.Title>
						<FloatingPanel.Control>
							<FloatingPanel.StageTrigger stage="minimized">
								<MinusIcon class="size-4" />
							</FloatingPanel.StageTrigger>
							<FloatingPanel.StageTrigger stage="maximized">
								<MaximizeIcon class="size-4" />
							</FloatingPanel.StageTrigger>
							<FloatingPanel.StageTrigger stage="default">
								<MinimizeIcon class="size-4" />
							</FloatingPanel.StageTrigger>
							<FloatingPanel.CloseTrigger>
								<XIcon className="size-4" />
							</FloatingPanel.CloseTrigger>
						</FloatingPanel.Control>
					</FloatingPanel.Header>
				</FloatingPanel.DragTrigger>
				<FloatingPanel.Body>
					{#if selectedInstanceData && selectedInstance}
						<InstanceConfigEditor instanceId={selectedInstance.id} bind:config={selectedInstanceData} instanceType={selectedInstance.type}>
						</InstanceConfigEditor>
					{/if}
				</FloatingPanel.Body>
				<FloatingPanel.ResizeTrigger axis="se" />
			</FloatingPanel.Content>
		</FloatingPanel.Positioner>
	</Portal>
</FloatingPanel>