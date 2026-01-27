<script lang="ts">
	import { createTreeViewCollection, TreeView } from '@skeletonlabs/skeleton-svelte';
	import type { FlowGraph, FlowTreeView, FlowTreeViewNode } from './flow-view-builder';
	import { ExternalLinkIcon, FileIcon, LogInIcon, ServerIcon, SplitIcon } from 'lucide-svelte';

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
		return <FlowTreeViewNode>{
			id: '$view_root',
			children: [tree.root, ...tree.orphans],
			duplicate: false,
			graphReference: graph
		}
	}
	const treeView = $derived(buildTreeView(graph));
	const collection = $derived(createTreeViewCollection<FlowTreeViewNode>({
		nodeToValue: (node) => node.id,
		nodeToString: (node) => node.id,
		nodeToChildrenCount: (node) => node.children.length,
		nodeToChildren: (node) => node.children,
		rootNode: treeView,
	}));
	const selectedValue = $state(undefined as string[] | undefined);
	$effect(() => {
		selected = selectedValue?.at(0) ?? undefined;
	});
</script>
<TreeView {collection} selectedValue={selectedValue} selectionMode="single">
	<TreeView.Label>Flow Tree View</TreeView.Label>
	<TreeView.Tree>
		{#each collection.rootNode.children || [] as node, index (node)}
			{@render treeNode(node, [index])}
		{/each}
	</TreeView.Tree>
</TreeView>
{#snippet treeNode(node: FlowTreeViewNode, indexPath: number[])}
	<TreeView.NodeProvider value={{ node, indexPath }}>
		{#if node.children && node.children.length > 0}
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
					{#each node.children as childNode, childIndex (childNode)}
						{@render treeNode(childNode, [...indexPath, childIndex])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else}
			<TreeView.Item>
				{#if node.duplicate}
					<ExternalLinkIcon class="size-4" />
				{:else }
					<ServerIcon class="size-4" />
				{/if}
				{node.id}
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}
