<script lang="ts">
	import { Circle, Filter, ChevronDown, ChevronRight, Plus } from 'lucide-svelte';
	import NodeItem from './node-item.svelte';

	type NodeData = {
		class?: string;
		config: any;
	};

	type Props = {
		nodes: Record<string, NodeData>;
		filters: Record<string, NodeData>;
		entrypoint: string;
		selectedId?: string;
		selectedType?: 'node' | 'filter';
		onSelect: (id: string, type: 'node' | 'filter') => void;
		onAdd: (type: 'node' | 'filter') => void;
		onUpdateEntrypoint: (entrypoint: string) => void;
	};

	let {
		nodes = $bindable(),
		filters = $bindable(),
		entrypoint = $bindable(),
		selectedId,
		selectedType,
		onSelect,
		onAdd,
		onUpdateEntrypoint
	}: Props = $props();

	let nodesExpanded = $state(true);
	let filtersExpanded = $state(true);
</script>

<div class="node-tree space-y-4 p-4 overflow-y-auto">
	<!-- Entry Point -->
	<div class="space-y-1">
		<label class="label">
			<span class="label-text text-xs font-semibold">Entry Point</span>
			<select
				class="select select-sm"
				value={entrypoint}
				onchange={(e) => onUpdateEntrypoint(e.currentTarget.value)}
			>
				<option value="">-- Select Entry Point --</option>
				{#each Object.keys(nodes) as nodeId}
					<option value={nodeId}>{nodeId}</option>
				{/each}
			</select>
		</label>
	</div>

	<!-- Add Buttons -->
	<div class="flex gap-2">
		<button class="btn btn-sm preset-filled-primary flex-1" onclick={() => onAdd('node')}>
			<Plus size={14} /> Add Node
		</button>
		<button class="btn btn-sm preset-filled-secondary flex-1" onclick={() => onAdd('filter')}>
			<Plus size={14} /> Add Filter
		</button>
	</div>

	<!-- Nodes Section -->
	<div class="space-y-1">
		<button
			class="flex items-center gap-1 text-sm font-semibold w-full hover:bg-surface-100 dark:hover:bg-surface-800 rounded px-2 py-1 transition-colors"
			onclick={() => (nodesExpanded = !nodesExpanded)}
			type="button"
		>
			{#if nodesExpanded}
				<ChevronDown size={14} />
			{:else}
				<ChevronRight size={14} />
			{/if}
			<Circle size={14} />
			<span>Nodes ({Object.keys(nodes).length})</span>
		</button>

		{#if nodesExpanded}
			<div class="ml-2 space-y-1">
				{#if Object.keys(nodes).length === 0}
					<p class="text-xs opacity-60 px-3 py-2">No nodes defined</p>
				{:else}
					{#each Object.entries(nodes) as [id, node]}
						<NodeItem
							{id}
							classId={node.class}
							config={node.config}
							instanceType="node"
							isEntry={id === entrypoint}
							selected={selectedId === id && selectedType === 'node'}
							onclick={() => onSelect(id, 'node')}
						/>
					{/each}
				{/if}
			</div>
		{/if}
	</div>

	<!-- Filters Section -->
	<div class="space-y-1">
		<button
			class="flex items-center gap-1 text-sm font-semibold w-full hover:bg-surface-100 dark:hover:bg-surface-800 rounded px-2 py-1 transition-colors"
			onclick={() => (filtersExpanded = !filtersExpanded)}
			type="button"
		>
			{#if filtersExpanded}
				<ChevronDown size={14} />
			{:else}
				<ChevronRight size={14} />
			{/if}
			<Filter size={14} />
			<span>Filters ({Object.keys(filters).length})</span>
		</button>

		{#if filtersExpanded}
			<div class="ml-2 space-y-1">
				{#if Object.keys(filters).length === 0}
					<p class="text-xs opacity-60 px-3 py-2">No filters defined</p>
				{:else}
					{#each Object.entries(filters) as [id, filter]}
						<NodeItem
							{id}
							classId={filter.class}
							config={filter.config}
							instanceType="filter"
							selected={selectedId === id && selectedType === 'filter'}
							onclick={() => onSelect(id, 'filter')}
						/>
					{/each}
				{/if}
			</div>
		{/if}
	</div>
</div>
