<script lang="ts">
	import { TreeView, createTreeViewCollection } from '@skeletonlabs/skeleton-svelte';
	import { Plus, AlertCircle, CircleDashed, Search, Filter as FilterIcon, Filter, Funnel } from 'lucide-svelte';
	import NodeItem from './node-item.svelte';
	import {
		buildFlowTree,
		toViewTree,
		type NodeData,
		type ViewNode
	} from './tree-builder';

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

	// Tab state
	let activeTab = $state<'flow' | 'filters'>('flow');
	
	// Filter search state
	let filterSearch = $state('');

	// Reactive tree generation
	let logicalTree = $derived(buildFlowTree(nodes, filters, entrypoint));
	let viewTree = $derived(toViewTree(logicalTree.tree, logicalTree.orphans));

	// Create TreeView Collection
	let treeCollection = $derived(
		createTreeViewCollection<ViewNode>({
			nodeToValue: (node) => node.id,
			nodeToString: (node) => node.label,
			rootNode: viewTree
		})
	);



	// Filtered filters based on search
	let filteredFilters = $derived.by(() => {
		$inspect(filters);
		const entries: [string, NodeData][] = Object.entries(filters);
		if (!filterSearch.trim()) {
			return entries;
		}
		const searchLower = filterSearch.toLowerCase();
		return entries.filter(([id, data]) => 
			id.toLowerCase().includes(searchLower) ||
			(data.class?.toLowerCase() || '').includes(searchLower)
		);
	});
</script>

{#snippet treeNode(node: ViewNode, _indexPath: number[])}
	<TreeView.NodeProvider value={{ node, indexPath: _indexPath }}>
		{#if node.children && node.children.length > 0}
			<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator />
					<TreeView.BranchText>
						<!-- Connection / Group Label -->
						{#if node.type === 'connection'}
							<div class="text-xs font-mono opacity-50 flex items-center gap-2">
								<span class="w-1.5 h-1.5 rounded-full bg-surface-400"></span>
								{node.label}
							</div>
						{:else if node.type === 'orphan-group'}
							<div class="text-xs font-bold uppercase tracking-widest opacity-40 py-1">
								{node.label}
							</div>
						{:else}
							<!-- Expandable Node/Filter -->
							<NodeItem
								id={node.originalId || node.id}
								classId={node.data?.class}
								config={node.data?.config}
								instanceType={node.type === 'filter' ? 'filter' : 'node'}
								isEntry={node.isEntry}
								isCycle={node.type === 'cycle'}
								selected={selectedId === node.originalId &&
									((node.type === 'node' && selectedType === 'node') ||
										(node.type === 'filter' && selectedType === 'filter'))}
								onclick={() => {
									if (
										node.originalId &&
										(node.type === 'node' || node.type === 'filter' || node.type === 'cycle')
									) {
										onSelect(node.originalId, node.type === 'filter' ? 'filter' : 'node');
									}
								}}
							/>
						{/if}
					</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					<TreeView.BranchIndentGuide />
					{#each node.children as child, index (child.id)}
						{@render treeNode(child, [..._indexPath, index])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else}
			<TreeView.Item>
				{#if node.type === 'missing'}
					<div class="flex items-center gap-2 text-error-500 px-2 py-1 text-sm">
						<AlertCircle size={14} />
						<span>{node.label}</span>
					</div>
				{:else if node.type === 'connection'}
					<!-- Empty connection (leaf) -->
					<div class="text-xs font-mono opacity-50 px-2 py-1 flex items-center gap-2">
						<span class="w-1.5 h-1.5 rounded-full bg-surface-400"></span>
						{node.label} (End)
					</div>
				{:else}
					<NodeItem
						id={node.originalId || node.id}
						classId={node.data?.class}
						config={node.data?.config}
						instanceType={node.type === 'filter' ? 'filter' : 'node'}
						isEntry={node.isEntry}
						isCycle={node.type === 'cycle'}
						selected={selectedId === node.originalId &&
							((node.type === 'node' && selectedType === 'node') ||
								(node.type === 'filter' && selectedType === 'filter'))}
						onclick={() => {
							if (
								node.originalId &&
								(node.type === 'node' || node.type === 'filter' || node.type === 'cycle')
							) {
								onSelect(node.originalId, node.type === 'filter' ? 'filter' : 'node');
							}
						}}
					/>
				{/if}
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<div class="h-full flex flex-col overflow-hidden bg-surface-50-950">
	<!-- Tabs -->
	<div class="flex border-b border-surface-200-800 shrink-0">
		<button
			class="flex-1 px-4 py-3 text-sm font-medium transition-colors
				{activeTab === 'flow'
					? 'text-primary-500 border-b-2 border-primary-500'
					: 'text-surface-500 hover:text-surface-700 dark:hover:text-surface-300'}"
			onclick={() => activeTab = 'flow'}
		>
			Flow Tree
		</button>
		<button
			class="flex-1 px-4 py-3 text-sm font-medium transition-colors
				{activeTab === 'filters'
					? 'text-primary-500 border-b-2 border-primary-500'
					: 'text-surface-500 hover:text-surface-700 dark:hover:text-surface-300'}"
			onclick={() => activeTab = 'filters'}
		>
			Filters ({Object.keys(filters).length})
		</button>
	</div>

	<!-- Content based on active tab -->
	{#if activeTab === 'flow'}
		<!-- Flow Tree Tab -->
		<div class="p-4 space-y-4 border-b border-surface-200-800 shrink-0">
			<div class="space-y-1">
				<label class="label">
					<span class="label-text text-xs font-semibold uppercase tracking-wider opacity-60"
						>Entry Point</span
					>
					<select
						class="select select-sm"
						value={entrypoint}
						onchange={(e) => onUpdateEntrypoint(e.currentTarget.value)}
					>
						<option value="">-- No Entry Point --</option>
						{#each Object.keys(nodes) as nodeId}
							<option value={nodeId}>{nodeId}</option>
						{/each}
					</select>
				</label>
			</div>

			<div class="flex gap-2">
				<button class="btn btn-sm preset-filled-primary flex-1" onclick={() => onAdd('node')}>
					<Plus size={14} /> Node
				</button>
				<button class="btn btn-sm preset-filled-secondary flex-1" onclick={() => onAdd('filter')}>
					<Plus size={14} /> Filter
				</button>
			</div>
		</div>

		<!-- Tree Content -->
		<div class="flex-1 overflow-y-auto p-2">
			{#if !entrypoint && Object.keys(nodes).length > 0}
				<div class="p-4 text-center opacity-60 text-sm italic">
					Select an Entry Point to view the flow structure.
				</div>
			{:else if Object.keys(nodes).length === 0}
				<div class="p-8 flex flex-col items-center justify-center text-center opacity-50 gap-2">
					<CircleDashed size={32} />
					<p class="text-sm">No nodes created yet.</p>
				</div>
			{/if}

			<TreeView collection={treeCollection}>
				<TreeView.Tree>
					<!-- Start rendering from root items -->
					{#each treeCollection.rootNode?.children || [] as node, index (node.id)}
						{@render treeNode(node, [index])}
					{/each}
				</TreeView.Tree>
			</TreeView>
		</div>
	{:else}
		<!-- Filters Tab -->
		<div class="flex-1 flex flex-col overflow-hidden">
			<!-- Search Bar -->
			<div class="p-4 border-b border-surface-200-800 shrink-0">
				<div class="relative">
					<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 text-surface-400" size={16} />
					<input
						type="text"
						class="input input-sm pl-9 w-full"
						placeholder="Search filters by name or type..."
						bind:value={filterSearch}
					/>
				</div>
			</div>

			<!-- Filter List -->
			<div class="flex-1 overflow-y-auto">
				{#if filteredFilters.length === 0}
					<div class="p-8 flex flex-col items-center justify-center text-center opacity-50 gap-2">
						<FilterIcon size={32} />
						{#if filterSearch.trim()}
							<p class="text-sm">No filters match your search.</p>
							<button
								class="btn btn-sm variant-soft-primary mt-2"
								onclick={() => filterSearch = ''}
							>
								Clear search
							</button>
						{:else}
							<p class="text-sm">No filters created yet.</p>
							<button
								class="btn btn-sm preset-filled-secondary mt-2"
								onclick={() => onAdd('filter')}
							>
								<Plus size={14} /> Create Filter
							</button>
						{/if}
					</div>
				{:else}
					<div class="divide-y divide-surface-200-800">
						{#each filteredFilters as [id, data], i (id)}
							<button
								type="button"
								class="w-full text-left p-3 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors
									{selectedId === id && selectedType === 'filter' ? 'bg-primary-500/10' : ''}"
								onclick={() => onSelect(id, 'filter')}
								onkeydown={(e) => {
									if (e.key === 'Enter' || e.key === ' ') {
										e.preventDefault();
										onSelect(id, 'filter');
									}
								}}
								aria-label={`Select filter ${id}`}
								tabindex="0"
							>
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-3">
										<Funnel size={16} class="text-surface-500 shrink-0" />
										<div class="min-w-0">
											<div class="font-medium truncate">{id}</div>
											<div class="text-xs text-surface-500 truncate">
												{data.class || 'Not configured'}
											</div>
										</div>
									</div>
									{#if data.class}
										<span class="badge badge-sm variant-soft-success">Configured</span>
									{:else}
										<span class="badge badge-sm variant-soft-warning">Not configured</span>
									{/if}
								</div>
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Add Filter Button (sticky) -->
			<div class="p-4 border-t border-surface-200-800 shrink-0">
				<button class="btn btn-sm preset-filled-secondary w-full" onclick={() => onAdd('filter')}>
					<Plus size={14} /> Add New Filter
				</button>
			</div>
		</div>
	{/if}
</div>
