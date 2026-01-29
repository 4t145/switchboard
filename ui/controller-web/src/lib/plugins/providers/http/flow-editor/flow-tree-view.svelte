<script lang="ts">
	import { createTreeViewCollection, TreeView } from '@skeletonlabs/skeleton-svelte';
	import { FlowTreeViewNode, type FlowGraph } from './flow-view-builder';
	import { ExternalLinkIcon, FileIcon, FunnelIcon, LogInIcon, ServerIcon, SplitIcon } from 'lucide-svelte';

	type Props = {
		graph: FlowGraph;
		selected: string | undefined;
	};

	let {
		graph,
		selected = $bindable(undefined)
	}: Props = $props();

	function buildTreeView(graph: FlowGraph) {
		const tree = graph.asTree();
		return tree
	}
	
	const treeView = $derived.by(() => {
		try {
			return buildTreeView(graph)
		} catch (e) {
			console.error(`Failed to build tree view`, e);
			return Error(`Failed to build tree view: ${e}`);
		}
	});
	const componentState: {
		type: 'ready';
		collection: ReturnType<typeof createTreeViewCollection<FlowTreeViewNode>>;
	} | {
		type: 'error';
		error: Error;
	} = $derived.by(() => {
		try {
			return {
				type: 'ready',
				collection: createTreeViewCollection<FlowTreeViewNode>({
					nodeToValue: FlowTreeViewNode.nodeToValue,
					nodeToString: FlowTreeViewNode.nodeToString,
					nodeToChildrenCount: FlowTreeViewNode.nodeToChildrenCount,
					nodeToChildren: FlowTreeViewNode.nodeToChildren,
					rootNode: buildTreeView(graph),
				})
			};
		} catch (e) {
			console.error(`Failed to build tree view`, e);
			return {
				type: 'error',
				error: Error(`Failed to build tree view: ${e}`)
			}
		}
		
	});
	let selectedValue = $state(undefined as string[] | undefined);
	$effect(() => {
		selected = selectedValue?.at(0) ?? undefined;
	});
</script>

{#if componentState.type === 'error'}
	<div class="alert alert-danger">
		<div class="flex items-center space-x-2">
			<span>Error building flow tree view: {componentState.error.message}</span>
		</div>
	</div>
{:else if componentState.type === 'ready'}
	{@const collection = componentState.collection}
	<TreeView {collection} selectedValue={selectedValue} onSelectionChange={(event) => { selectedValue = event.selectedValue; }} selectionMode="single">
		<TreeView.Tree>
			{#if collection.rootNode.type === 'root'}
				{@render treeNode(collection.rootNode.node_entrypoint, [0])}
				{#if collection.rootNode.orphans.nodes.length > 0}
					{@render treeNode(collection.rootNode.orphans, [1])}
				{/if}
				{@render treeNode(collection.rootNode.filters, [2])}
			{/if}
		</TreeView.Tree>
	</TreeView>
{/if}
{#snippet treeNode(node: FlowTreeViewNode, indexPath: number[])}
	<TreeView.NodeProvider value={{ node, indexPath }}>
		{#if node.type === 'dispatcher'}
		<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator />
					<TreeView.BranchText>
						<SplitIcon  class="size-4 rotate-90"></SplitIcon>
						{node.id}
						{#if node.id === graph.entrypoint}
							<span  class="text-xs preset-outlined px-1">entrypoint</span>
						{/if}
					</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					<TreeView.BranchIndentGuide />
					{@const isInputEmpty = node.inputs.filters.reduce((acc, port) => acc + port.filters.length, 0) > 0}
					{@const isOutputEmpty = node.outputs.filters.reduce((acc, port) => acc + port.filters.length, 0) > 0}
					{#each node.children as childNode, childIndex (childNode)}
						{@render treeNode(childNode, [...indexPath, childIndex])}
					{/each}
					{#if isInputEmpty}
						{@render treeNode(node.inputs, [...indexPath, node.children.length + 1])}
					{/if}
					{#if isOutputEmpty}
						{@render treeNode(node.outputs, [...indexPath, node.children.length])}
					{/if}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else if node.type === 'node-interface'}
		<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator />
					<TreeView.BranchText>
						{#if node.kind === 'input'}
							<LogInIcon  class="size-4 rotate-180"></LogInIcon>
							Inputs
						{:else}
							<LogInIcon  class="size-4"></LogInIcon>
							Outputs
						{/if}
					</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					<TreeView.BranchIndentGuide />
					{#each node.filters as childNode, childIndex (childNode)}
						{@render treeNode(childNode, [...indexPath, childIndex])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else if node.type === 'dispatcher-reference'}
		<TreeView.Item>
			<ExternalLinkIcon class="size-4" />
			{node.ref} (dispatcher reference)
		</TreeView.Item>
		{:else if node.type === 'service'}
			{@const isEmpty = node.inputs.filters.reduce((acc, port) => acc + port.filters.length, 0) > 0}
			{#if isEmpty}
				<TreeView.Branch>
					<TreeView.BranchControl>
						<TreeView.BranchIndicator />
						<TreeView.BranchText>
							<ServerIcon class="size-4" />
							{node.id}
						</TreeView.BranchText>
					</TreeView.BranchControl>
					<TreeView.BranchContent>
						<TreeView.BranchIndentGuide />
						{@render treeNode(node.inputs, [...indexPath, 0])}
					</TreeView.BranchContent>
				</TreeView.Branch>
			{:else}
				<TreeView.Item>
					<ServerIcon class="size-4" />
					{node.id}
				</TreeView.Item>
			{/if}
		{:else if node.type === 'filter'}
		<TreeView.Item>
			<FunnelIcon class="size-4" />
			{node.id}
		</TreeView.Item>
		{:else if node.type === 'filter-dir'}
			{#if node.filters.length > 0}
				<TreeView.Branch>
					<TreeView.BranchControl>
						<TreeView.BranchIndicator />
						<TreeView.BranchText>
							{#if node.location.type === 'global'}
								<FunnelIcon  class="size-4"></FunnelIcon>
								Filters
							{:else }	
								<ExternalLinkIcon  class="size-4"></ExternalLinkIcon>
								{node.location.node} {node.location.port}
							{/if}
						</TreeView.BranchText>
					</TreeView.BranchControl>
					<TreeView.BranchContent>
						<TreeView.BranchIndentGuide />
						{#each node.filters as childNode, childIndex (childNode)}
							{@render treeNode(childNode, [...indexPath, childIndex])}
						{/each}
					</TreeView.BranchContent>
				</TreeView.Branch>
			{/if}
		{:else if node.type === 'orphans'}
		<TreeView.Branch>
			<TreeView.BranchControl>
				<TreeView.BranchIndicator />
				<TreeView.BranchText>
					<FileIcon  class="size-4"></FileIcon>
					Orphan Nodes
				</TreeView.BranchText>
			</TreeView.BranchControl>
			<TreeView.BranchContent>
				<TreeView.BranchIndentGuide />
				{#each node.nodes as childNode, childIndex (childNode)}
					{@render treeNode(childNode, [...indexPath, childIndex])}
				{/each}
			</TreeView.BranchContent>
		</TreeView.Branch>
		{/if}
	</TreeView.NodeProvider>
{/snippet}
