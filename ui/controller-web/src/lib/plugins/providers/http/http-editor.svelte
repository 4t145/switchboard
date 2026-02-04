<script lang="ts">
	import {
		flowConfigToHttpEditorContext,
		type FlowConfig,
		type HttpConfig,
		type HttpEditorContext,
		type InstanceDataWithoutType,
		type ServerConfig
	} from './types';
	import FlowEditor from './flow-editor/flow-editor.svelte';
	import { mount, onDestroy, setContext, unmount, untrack } from 'svelte';
	import {
		buildFlowGraph,
		FlowGraph,
		FlowTreeViewInstanceSelection
	} from './flow-editor/flow-view-builder';
	import FlowTree from './flow-editor/flow-tree-view.svelte';
	import FlowGraphView from './flow-editor/flow-graph-view.svelte';
	import { EditIcon, ListIcon, SquarePenIcon, WorkflowIcon } from '@lucide/svelte';
	import {
		Dialog,
		FloatingPanel,
		Portal,
		SegmentedControl,
		ToggleGroup
	} from '@skeletonlabs/skeleton-svelte';
	import InstanceConfigEditor from './flow-editor/instance-config-editor.svelte';
	import InstanceConfigEditPanel from './flow-editor/instance-config-edit-panel.svelte';
	import type { ProviderEditorProps } from '$lib/plugins/types';

	type Props = ProviderEditorProps<HttpConfig>;

	let { value = $bindable(), onValueChange, readonly }: Props = $props();
	$effect(() => {
		onValueChange(value);
	});
	$effect(() => {
		value.server = serverConfig;
	});
	$effect(() => {
		untrack(() => updateGraphState(value.flow));
	});
	
	let httpEditorContext: HttpEditorContext = $derived(
		flowConfigToHttpEditorContext(value.flow)
	);
	type ViewMode = 'list' | 'tree' | 'graph';
	type GraphState =
		| {
				readonly type: 'ready';
				readonly graph: FlowGraph;
		  }
		| {
				readonly type: 'error';
				readonly error: Error;
		  }
		| {
				readonly type: 'building';
				readonly promise: Promise<FlowGraph>;
		  }
		| {
				readonly type: 'uninitialized';
		  };

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
	function getSelectedInstanceData(selectedInstance: FlowTreeViewInstanceSelection) {
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
	}
	let serverConfig: ServerConfig = $state(
		value.server ?? {
			version: 'auto'
		}
	);


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

	let panels: Record<string, InstanceConfigEditPanel> = $state({});
	function createOrOpenEditPanel(selection: FlowTreeViewInstanceSelection) {
		let selectedInstance = getSelectedInstanceData(selection);
		if (!selectedInstance) {
			return null;
		}
		let instanceType = selection.type;
		let instanceId = selection.id;
		function onValueSave(newValue: InstanceDataWithoutType) {
			if (instanceType === 'node') {
				value.flow.nodes[instanceId] = newValue;
			} else if (instanceType === 'filter') {
				value.flow.filters[instanceId] = newValue;
			}
		}
		if (panels[instanceId]) {
			panels[instanceId].focus();
		} else {
			const panel = mount(InstanceConfigEditPanel, {
				target: container,
				props: {
					instanceType,
					instanceId,
					value: selectedInstance,
					httpEditorContext,
					onValueSave
				}
			});

			panels[instanceId] = panel;
		}
	}
	onDestroy(() => {
		Object.values(panels).forEach((panel) => {
			panel.closeWithoutSaving();
			unmount(panel);
		});
		panels = {};
	});
	let container: HTMLDivElement;
</script>

<div class="space-y-4" bind:this={container}>
	<!-- HTTP Version Selector -->
	<label class="label">
		<span class="label-text font-medium">HTTP Version</span>
		<select class="select-sm select" bind:value={serverConfig.version}>
			<option value="auto">Auto (Negotiate)</option>
			<option value="http1">HTTP/1.1 Only</option>
			<option value="http2">HTTP/2 Only</option>
		</select>
	</label>

	<!-- Flow Editor (Visual Editor) -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<div class="label-text flex-grow font-medium">Flow Configuration</div>
			<div class="btn-group">
				{#if !readonly}
					<button
						class="btn-icon preset-outlined-surface-200-800"
						onclick={() => {
							selectedInstance ? createOrOpenEditPanel(selectedInstance) : void 0;
						}}
						disabled={selectedInstance === undefined}
					>
						<SquarePenIcon class="size-4" />
					</button>
				{/if}
			</div>
			<ToggleGroup
				defaultValue={['tree']}
				multiple={false}
				value={viewModeSelection}
				onValueChange={(details) => {
					viewModeSelection = details.value;
					viewMode = details.value[0] as ViewMode;
				}}
			>
				<ToggleGroup.Item value="list">
					<ListIcon class="size-4" />
				</ToggleGroup.Item>
				<ToggleGroup.Item value="tree">
					<WorkflowIcon class="size-4" />
				</ToggleGroup.Item>
			</ToggleGroup>
		</div>

		<div class="h-[600px] overflow-hidden card border border-surface-200 dark:border-surface-700">
			{#if graphState.type === 'building'}
				<div>
					<div class="flex h-full flex-col items-center justify-center">
						<span class="loading loading-spinner loading-lg"></span>
						<p class="mt-4 text-center text-sm">Building flow graph...</p>
					</div>
				</div>
			{:else if graphState.type === 'error'}
				<div>
					<div class="flex h-full flex-col items-center justify-center p-4">
						<p class="text-error text-center text-sm">
							Error building flow graph: {graphState.error.message}
						</p>
					</div>
				</div>
			{:else if graphState.type === 'ready'}
				{#if viewMode === 'list'}
					<div class="flex h-full flex-col items-center justify-center">
						<p class="text-muted-foreground text-center text-sm">List view coming soon...</p>
					</div>
				{:else if viewMode === 'tree'}
					<FlowTree graph={graphState.graph} bind:selected={selectedValue} />
				{/if}
			{:else}
				<div>
					<div class="flex h-full flex-col items-center justify-center">
						<p class="mt-4 text-center text-sm">Initialize flow graph...</p>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>
<!-- {#if selectedInstance && selectedInstanceData}
	<InstanceConfigEditPanel
		value={selectedInstanceData}
		instanceType={selectedInstance ? selectedInstance.type : 'node'}
		instanceId={selectedInstance ? selectedInstance.id : ''}
		onValueSave={(newValue) => {
			if (selectedInstance?.type === 'node') {
				value.flow.nodes[selectedInstance.id] = newValue;
			} else if (selectedInstance?.type === 'filter') {
				value.flow.filters[selectedInstance.id] = newValue;
			}
		}}
	/>
{/if} -->
