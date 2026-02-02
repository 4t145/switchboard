<script lang="ts">
	import type { HttpClassEditorProps } from '$lib/plugins/types';
	import { createTreeViewCollection, TreeView } from '@skeletonlabs/skeleton-svelte';
	import { GlobeIcon, FolderIcon, TargetIcon, FileJsonIcon, ArrowRightIcon } from '@lucide/svelte';

	export type PathTree = {
		[pattern: string]: RuleBucket;
	};
	export type RuleBucket =
		| {
				rules: string[][];
				target: string;
		  }
		| string;

	export type RouterConfig = {
		output: {
			[port: string]: {
				target: string;
				filters: string[];
			};
		};
		hostname: {
			[pattern: string]: PathTree;
		};
	};

	type Props = HttpClassEditorProps<RouterConfig>;

	let { value = $bindable(), readonly = false }: Props = $props();

	// Internal Node Types with ID
	type TreeNode = RootNode | HostNameNode | PathNode | RuleBucketNode;
	type RootNode = {
		id: string;
		kind: 'root';
		hostnames: HostNameNode[];
	};
	type HostNameNode = {
		id: string;
		kind: 'hostname';
		pattern: string;
		paths: PathNode[];
	};
	type PathNode = {
		id: string;
		kind: 'path';
		pattern: string;
		rules: RuleBucketNode[];
	};
	type RuleBucketNode = {
		id: string;
		kind: 'rule_bucket';
		rules: string[][];
		target: string;
	};

	function buildTree(config: RouterConfig): RootNode {
		const root: RootNode = { id: 'root', kind: 'root', hostnames: [] };
		// Sort hostnames for stable display
		const sortedHostnames = Object.keys(config.hostname || {}).sort();

		for (const hostnamePattern of sortedHostnames) {
			const pathTree = config.hostname[hostnamePattern];
			const hostId = `host:${hostnamePattern}`;
			const hostnameNode: HostNameNode = {
				id: hostId,
				kind: 'hostname',
				pattern: hostnamePattern,
				paths: []
			};

			// Sort paths
			const sortedPaths = Object.keys(pathTree || {}).sort();

			for (const pathPattern of sortedPaths) {
				const ruleBucket = pathTree[pathPattern];
				const pathId = `${hostId}|path:${pathPattern}`;
				const pathNode: PathNode = {
					id: pathId,
					kind: 'path',
					pattern: pathPattern,
					rules: []
				};

				if (typeof ruleBucket === 'string') {
					// Single target, no rules
					const ruleBucketNode: RuleBucketNode = {
						id: `${pathId}|bucket:0`,
						kind: 'rule_bucket',
						rules: [],
						target: ruleBucket
					};
					pathNode.rules.push(ruleBucketNode);
				} else {
					const ruleBucketNode: RuleBucketNode = {
						id: `${pathId}|bucket:0`,
						kind: 'rule_bucket',
						rules: ruleBucket.rules,
						target: ruleBucket.target
					};
					pathNode.rules.push(ruleBucketNode);
				}
				hostnameNode.paths.push(pathNode);
			}
			root.hostnames.push(hostnameNode);
		}
		return root;
	}

	// Helper functions for TreeView
	const nodeHelpers = {
		nodeToValue: (node: TreeNode) => node.id,
		nodeToString: (node: TreeNode) => {
			if (node.kind === 'hostname') return node.pattern;
			if (node.kind === 'path') return node.pattern;
			if (node.kind === 'rule_bucket') return node.target;
			return 'Root';
		},
		nodeToChildren: (node: TreeNode) => {
			if (node.kind === 'root') return node.hostnames;
			if (node.kind === 'hostname') return node.paths;
			if (node.kind === 'path') return node.rules;
			return [];
		}
	};

	let rawTree = $derived(buildTree(value));
	let treeCollection = $derived(
		createTreeViewCollection<TreeNode>({
			...nodeHelpers,
			rootNode: rawTree
		})
	);

	let selectedValue = $state<string[]>([]);
	let selectedNode = $derived.by(() => {
		if (selectedValue.length === 0) return null;
		// Simple lookup - in a real app with large trees, might want a map
		// But here we can just traverse or use the collection's internal map if exposed,
		// Recalculating by traversing the rawTree for the selected ID:
		const id = selectedValue[0];
		if (id === 'root') return rawTree;

		for (const h of rawTree.hostnames) {
			if (h.id === id) return h;
			for (const p of h.paths) {
				if (p.id === id) return p;
				for (const r of p.rules) {
					if (r.id === id) return r;
				}
			}
		}
		return null;
	});
</script>

{#snippet treeNode(node: TreeNode, indexPath: number[])}
	<TreeView.NodeProvider value={{ node, indexPath }}>
		{#if node.kind === 'root'}
			{#each node.hostnames as host, i}
				{@render treeNode(host, [...indexPath, i])}
			{/each}
		{:else if node.kind === 'hostname'}
			<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator />
					<TreeView.BranchText class="font-bold">
						<GlobeIcon class="mr-2 size-4" />
						{node.pattern}
					</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					<TreeView.BranchIndentGuide />
					{#each node.paths as path, i}
						{@render treeNode(path, [...indexPath, i])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else if node.kind === 'path'}
			<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator />
					<TreeView.BranchText>
						<FolderIcon class="mr-2 size-4 text-surface-500" />
						{node.pattern}
					</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					<TreeView.BranchIndentGuide />
					{#each node.rules as rule, i}
						{@render treeNode(rule, [...indexPath, i])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else if node.kind === 'rule_bucket'}
			<TreeView.Item>
				<div class="flex w-full items-center gap-2">
					<TargetIcon class="size-4 text-primary-500" />
					<span class="font-mono text-sm text-primary-600 dark:text-primary-400">{node.target}</span
					>

					{#if node.rules.length > 0}
						<span class="variant-soft-secondary ml-auto badge text-xs">
							{node.rules.length} rules
						</span>
					{/if}
				</div>
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<div class="flex h-[500px] gap-4">
	<!-- Left Side: Tree View -->
	<div class="w-1/3 overflow-y-auto border-r border-surface-500/30 pr-4">
		<TreeView 
			collection={treeCollection} 
			selectionMode="single"
			bind:selectedValue
		>
			<TreeView.Tree>
				<!-- Start rendering from root children -->
				{#each treeCollection.rootNode.hostnames as host, i}
					{@render treeNode(host, [i])}
				{/each}
			</TreeView.Tree>
		</TreeView>
	</div>

	<!-- Right Side: Details Panel -->
	<div class="flex-1 overflow-y-auto pl-4">
		{#if selectedNode}
			<div class="space-y-4">
				<h4 class="h4">Details</h4>
				{#if selectedNode.kind === 'hostname'}
					<div class="grid grid-cols-[100px_1fr] gap-2 text-sm">
						<div class="font-bold opacity-60">Type:</div>
						<div>Hostname</div>
						<div class="font-bold opacity-60">Pattern:</div>
						<div class="font-mono">{selectedNode.pattern}</div>
						<div class="font-bold opacity-60">Paths:</div>
						<div>{selectedNode.paths.length}</div>
					</div>
				{:else if selectedNode.kind === 'path'}
					<div class="grid grid-cols-[100px_1fr] gap-2 text-sm">
						<div class="font-bold opacity-60">Type:</div>
						<div>Path</div>
						<div class="font-bold opacity-60">Pattern:</div>
						<div class="font-mono">{selectedNode.pattern}</div>
					</div>
				{:else if selectedNode.kind === 'rule_bucket'}
					<div class="grid grid-cols-[100px_1fr] gap-2 text-sm">
						<div class="font-bold opacity-60">Type:</div>
						<div>Target Rule</div>
						<div class="font-bold opacity-60">Target:</div>
						<div class="font-mono text-primary-500">{selectedNode.target}</div>
					</div>
					
					{#if selectedNode.rules.length > 0}
						<div class="mt-3">
							<div class="font-bold opacity-60 text-xs uppercase mb-1">Rules applied</div>
							<div class="table-container">
								<table class="table table-compact table-hover w-full text-xs">
									<thead>
										<tr>
											<th>Type</th>
											<th>Key</th>
											<th>Value</th>
										</tr>
									</thead>
									<tbody>
										{#each selectedNode.rules as rule}
											<tr>
												{#each rule as part}
													<td>{part}</td>
												{/each}
												<!-- Fill remaining cells if rule doesn't have 3 parts -->
												{#if rule.length < 3}
													<td colspan={3 - rule.length}></td>
												{/if}
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						</div>
					{:else}
						<div class="text-xs opacity-50 mt-2 italic">No additional rules configured</div>
					{/if}
				{/if}
			</div>
		{:else}
			<div class="flex h-full items-center justify-center text-sm opacity-50">
				Select a node to view details
			</div>
		{/if}
	</div>
</div>
