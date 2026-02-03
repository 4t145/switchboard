<script lang="ts">
	import type { HttpClassEditorProps } from '$lib/plugins/types';
	import { createTreeViewCollection, TagsInput, TreeView } from '@skeletonlabs/skeleton-svelte';
	import {
		GlobeIcon,
		FolderIcon,
		TargetIcon,
		FileJsonIcon,
		ArrowRightIcon,
		SlashIcon,
		DeleteIcon,

		LogOutIcon,

		PlusIcon


	} from '@lucide/svelte';
	import TableListEditor, { type RowParams, type ListOperations } from '$lib/components/editor/table-list-editor.svelte';

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
				filters?: string[];
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
		},
		nodeToChildrenCount: (node: TreeNode): number => {
			if (node.kind === 'root') return node.hostnames.length;
			if (node.kind === 'hostname') return node.paths.length;
			if (node.kind === 'path') return node.rules.length;
			return 0;
		}
	};

	let rawTree = $derived.by(() => {
		$inspect(value, 'RouterConfig value changed');
		return buildTree(value);
	});
	let treeCollection = $derived(
		createTreeViewCollection<TreeNode>({
			...nodeHelpers,
			rootNode: rawTree
		})
	);
	let selectedNode: TreeNode | null = $state(null);
	let selectedValue = $state<string[]>([]);
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
					<LogOutIcon class="size-4 text-primary-500" />
					<span class="font-mono text-sm text-primary-500">{node.target}</span
					>

					{#if node.rules.length > 0}
						<span class="badge ml-auto text-xs preset-filled-secondary-500">
							{node.rules.length} rules
						</span>
					{/if}
				</div>
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<div>
	<!--  Output editor -->
	<label class="label">
		<span class="label-text font-medium">Output Ports</span>
	</label>
	<div class="space-y-2 mb-4">
		{#each Object.entries(value.output || {}) as [port, portConfig], index (port)}
			<div class="grid grid-cols-[1fr_auto] gap-2 items-center">
				<div class="grid grid-cols-[100px_1fr] gap-2 text-sm">
					<div class="font-bold opacity-60">Port:</div>
					<div class="font-mono">{port}</div>
					<div class="font-bold opacity-60">Target:</div>
					<div class="font-mono">{portConfig.target}</div>
					<div class="font-bold opacity-60">Filters:</div>
					<div>
						{#if !portConfig.filters || portConfig.filters.length === 0}
							<span class="text-sm opacity-50">No filters</span>
						{:else}
							<span class="font-mono text-sm">{portConfig.filters.join(', ')}</span>
						{/if}
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
<div class="flex h-[500px] gap-4">
	<!-- Router editor -->
	<!-- Left Side: Tree View -->
	<div class="w-1/3 overflow-y-auto border-r border-surface-500/30 pr-4">
		<TreeView
			collection={treeCollection}
			selectionMode="single"
			{selectedValue}
			onSelectionChange={(selection) => {
				selectedValue = selection.selectedValue;
				if (selection.selectedNodes.length === 0) {
					selectedNode = null;
				} else {
					selectedNode = selection.selectedNodes[0] as TreeNode;
				}
			}}
		>
			<TreeView.Tree>
				<!-- Start rendering from root children -->
				{@render treeNode(treeCollection.rootNode, [])}
			</TreeView.Tree>
		</TreeView>
	</div>

	<!-- Right Side: Details Panel -->
	<div class="flex-1 overflow-y-auto pl-4">
		{#if selectedNode}
			<div class="space-y-4">
				<h4 class="h4">{selectedNode.id}</h4>
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
						<div class="font-bold opacity-60">Output Port:</div>
						<div class="font-mono text-primary-500">{selectedNode.target}</div>
					</div>
					{#snippet row({ deleteItem, updateItem, value }: RowParams<string[]>)}
						<tr class="">
							<td class="w-1/1">
								<TagsInput
									{value}
									disabled={readonly}
									onValueChange={(details) => updateItem(details.value)}
								>
									<TagsInput.Control>
										<TagsInput.Context>
											{#snippet children(tagsInput)}
												{#each tagsInput().value as value, index (index)}
													<TagsInput.Item {value} {index}>
														<TagsInput.ItemPreview>
															<TagsInput.ItemText>{value}</TagsInput.ItemText>
															<TagsInput.ItemDeleteTrigger />
														</TagsInput.ItemPreview>
														<TagsInput.ItemInput />
													</TagsInput.Item>
												{/each}
											{/snippet}
										</TagsInput.Context>
										<TagsInput.Input placeholder="Add a rule" />
									</TagsInput.Control>
									<TagsInput.HiddenInput />
								</TagsInput>
							</td>
							<td >
								<button type="button" class="btn-icon preset-tonal-error" onclick={(e) => deleteItem()}>
									<DeleteIcon  />
								</button>
							</td>
						</tr>
					{/snippet}
					{#snippet footer({ addNewItem, value }:ListOperations<string[]> )}
						<tr>
							<td>
								
								{#if value.length === 0}
									<span class="text-sm opacity-50 mt-1">No rules defined. All requests match.</span>
								{:else}
									<span class="font-medium text-sm"> {value.length} rules</span>
								{/if}
							</td>
							<td >
								<button
									type="button"
									class="btn-icon preset-tonal-primary"
									onclick={() => addNewItem([])}
								>
									<PlusIcon class="size-4" />
								</button>
							</td>
						</tr>
					{/snippet}
					<TableListEditor value={selectedNode.rules} {row} {footer}></TableListEditor>
				{/if}
			</div>
		{:else}
			<div class="flex h-full items-center justify-center text-sm opacity-50">
				Select a node to view details
			</div>
		{/if}
	</div>
</div>
