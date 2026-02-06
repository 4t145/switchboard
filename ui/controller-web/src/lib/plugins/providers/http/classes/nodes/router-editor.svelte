<script lang="ts">
	import type { HttpClassEditorProps } from '../';
	import { createTreeViewCollection, TagsInput, TreeView } from '@skeletonlabs/skeleton-svelte';
	import {
		GlobeIcon,
		FolderIcon,
		TargetIcon,
		ArrowRightIcon,
		SlashIcon,
		DeleteIcon,
		LogOutIcon,
		PlusIcon,
		ExternalLinkIcon,
		SquarePlusIcon,
		TrashIcon,
		PlusCircleIcon,
		CirclePlusIcon,
		RouteIcon
	} from '@lucide/svelte';
	import TableListEditor, {
		type RowParams,
		type ListOperations,
		type ItemOperations
	} from '$lib/components/common/table-list-editor.svelte';
	import TargetSelector from '../../target-selector.svelte';
	import FilterSelector from '../../filter-selector.svelte';
	import BadgedIcon from '$lib/components/common/badged-icon.svelte';

	export type PathTree = {
		[pattern: string]: RuleBucket;
	};
	export type RuleBucket =
		| {
				rules: string[][];
				target: string;
		  }
		| string;
	export type OutputItem = {
		target: string;
		filters?: string[];
	};
	export type RouterConfig = {
		output: {
			[port: string]: OutputItem;
		};
		hostname: {
			[pattern: string]: PathTree;
		};
	};

	type Props = HttpClassEditorProps<RouterConfig>;

	let { value, readonly = false, httpEditorContext, onValueChange }: Props = $props();

	// Internal Node Types with ID
	type TreeNode = RootNode | HostNameNode | PathNode;
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
		hostname: string;
		pattern: string;
		rules_count: number;
		port: string;
	};
	function formatId(nodePath?: { host: string; path?: string; bucket?: number }): string {
		if (!nodePath) {
			return 'root';
		} else if (nodePath.host !== undefined && nodePath.path === undefined) {
			return `${nodePath.host}`;
		} else if (
			nodePath.host !== undefined &&
			nodePath.path !== undefined &&
			nodePath.bucket === undefined
		) {
			return `${nodePath.host}${nodePath.path}`;
		} else if (
			nodePath.host !== undefined &&
			nodePath.path !== undefined &&
			nodePath.bucket !== undefined
		) {
			return `${nodePath.host}${nodePath.path}#${nodePath.bucket}`;
		} else {
			return 'root';
		}
	}
	function buildTree(config: RouterConfig): RootNode {
		const root: RootNode = { id: 'root', kind: 'root', hostnames: [] };
		// Sort hostnames for stable display
		const sortedHostnames = Object.keys(config.hostname || {}).sort();

		for (const hostnamePattern of sortedHostnames) {
			const pathTree = config.hostname[hostnamePattern];
			const hostId = formatId({ host: hostnamePattern, path: undefined });
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
				const pathId = formatId({ host: hostnamePattern, path: pathPattern });
				const pathNode: PathNode = {
					id: pathId,
					kind: 'path',
					hostname: hostnamePattern,
					pattern: pathPattern,
					rules_count: rulesOfRuleBucket(ruleBucket).length,
					port: targetOfRuleBucket(ruleBucket)
				};

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
			return 'Root';
		},
		nodeToChildren: (node: TreeNode) => {
			if (node.kind === 'root') return node.hostnames;
			if (node.kind === 'hostname') return node.paths;
			return [];
		},
		nodeToChildrenCount: (node: TreeNode): number => {
			if (node.kind === 'root') return node.hostnames.length;
			if (node.kind === 'hostname') return node.paths.length;
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

	let outputAsTable = $derived.by(() => {
		const output = value.output || {};
		return Object.entries(output).map(([port, config]) => ({
			port,
			...config
		}));
	});

	function outputTableAsOutput(
		table: {
			port: string;
			target: string;
			filters?: string[];
		}[]
	) {
		return table.reduce(
			(acc, item) => {
				acc[item.port] = {
					target: item.target,
					filters: item.filters
				};
				return acc;
			},
			{} as { [port: string]: OutputItem }
		);
	}

	function focusOnOutput(port: string) {
		const index = outputAsTable.findIndex((p) => p.port === port);
		if (index !== -1 && outputPortInputList[index]) {
			outputPortInputList[index].focus();
			outputPortInputList[index].scrollIntoView({ behavior: 'smooth', block: 'center' });
		}
	}
	let outputPortInputList = $state<HTMLInputElement[]>([]);

	// --- CRUD Operations ---

	function addHostname() {
		let pattern = 'new-host.com';
		let counter = 1;
		const hosts = value.hostname || {};
		while (hosts[pattern]) {
			pattern = `new-host-${counter}.com`;
			counter++;
		}

		value = {
			...value,
			hostname: {
				...hosts,
				[pattern]: {}
			}
		};
		// Select the new host
		selectedValue = [formatId({ host: pattern })];
	}

	function deleteHostname(pattern: string) {
		// eslint-disable-next-line
		if (!confirm(`Delete hostname ${pattern}?`)) return;
		const newHosts = { ...value.hostname };
		delete newHosts[pattern];
		value = { ...value, hostname: newHosts };
		selectedNode = null;
		selectedValue = [];
	}

	function renameHostname(oldPattern: string, newPattern: string) {
		if (oldPattern === newPattern) return;
		const hosts = value.hostname || {};
		if (hosts[newPattern]) {
			// eslint-disable-next-line
			alert('Hostname already exists');
			// Revert input? The UI will re-render with old value if this function is called and 'value' isn't updated, 
			// assuming the input value is derived from selectedNode. 
			// But if using bind:value to local state, it might stick. 
			// Best to rely on re-render.
			return;
		}

		// Preserve order if possible, or key order
		const entries = Object.entries(hosts);
		const index = entries.findIndex(([k]) => k === oldPattern);
		if (index === -1) return; // Should not happen

		const newEntries = [...entries];
		newEntries[index] = [newPattern, entries[index][1]];

		value = {
			...value,
			hostname: Object.fromEntries(newEntries)
		};
		selectedValue = [formatId({ host: newPattern })];
	}

	function addPath(hostPattern: string) {
		let pattern = '/new-path';
		let counter = 1;
		const paths = value.hostname[hostPattern] || {};
		while (paths[pattern]) {
			pattern = `/new-path-${counter}`;
			counter++;
		}

		const newPaths = {
			...paths,
			[pattern]: { target: '', rules: [] }
		};

		value = {
			...value,
			hostname: {
				...value.hostname,
				[hostPattern]: newPaths
			}
		};
		selectedValue = [formatId({ host: hostPattern, path: pattern })];
	}

	function deletePath(hostPattern: string, pathPattern: string) {
		// eslint-disable-next-line
		if (!confirm(`Delete path ${pathPattern}?`)) return;
		const paths = { ...value.hostname[hostPattern] };
		delete paths[pathPattern];

		value = {
			...value,
			hostname: {
				...value.hostname,
				[hostPattern]: paths
			}
		};
		selectedValue = [formatId({ host: hostPattern })];
	}

	function renamePath(hostPattern: string, oldPath: string, newPath: string) {
		if (oldPath === newPath) return;
		const paths = value.hostname[hostPattern] || {};
		if (paths[newPath]) {
			// eslint-disable-next-line
			alert('Path already exists');
			return;
		}

		const entries = Object.entries(paths);
		const index = entries.findIndex(([k]) => k === oldPath);
		if (index === -1) return;

		const newEntries = [...entries];
		newEntries[index] = [newPath, entries[index][1]];

		value = {
			...value,
			hostname: {
				...value.hostname,
				[hostPattern]: Object.fromEntries(newEntries)
			}
		};
		selectedValue = [formatId({ host: hostPattern, path: newPath })];
	}

	function parseNodeId(node_id: string):
		| undefined
		| {
				host: string;
				path?: string;
		  } {
		if (node_id === 'root') {
			return undefined;
		}
		let parts = node_id.split('re:', 2);
		if (parts[1]) {
			return {
				host: parts[0],
				path: parts[1]
			};
		}
		parts = node_id.split('/', 2);
		if (parts[1]) {
			return {
				host: parts[0],
				path: `/${parts[1]}`
			};
		}
		return {
			host: parts[0]
		};
	}
	function updateRuleBucket(hostname: string, path: string, ruleBucket: RuleBucket) {
		if (value.hostname[hostname] && value.hostname[hostname][path]) {
			value.hostname[hostname][path] = ruleBucket;
			onValueChange(value);
		}
	}
	function updatePath(hostname: string, path: string, newPath: string) {
		if (
			value.hostname[hostname] &&
			value.hostname[hostname][path] &&
			!value.hostname[hostname][newPath]
		) {
			let pathValue = value.hostname[hostname][path];
			value.hostname[hostname][newPath] = pathValue;
			delete value.hostname[hostname][path];
			onValueChange(value);
		}
	}
	function getRuleBucket(hostname: string, path: string): RuleBucket | undefined {
		if (value.hostname[hostname] && value.hostname[hostname][path]) {
			return value.hostname[hostname][path];
		}
		return undefined;
	}
	function targetOfRuleBucket(ruleBucket: RuleBucket): string {
		if (typeof ruleBucket === 'string') {
			return ruleBucket;
		} else {
			return ruleBucket.target;
		}
	}
	function rulesOfRuleBucket(ruleBucket: RuleBucket): string[][] {
		if (typeof ruleBucket === 'string') {
			return [];
		} else {
			return ruleBucket.rules;
		}
	}
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
			<TreeView.Item>
				<div class="flex w-full items-center gap-2">
					<RouteIcon class="mr-2 size-4"></RouteIcon>
					{#if node.pattern.startsWith('re:')}
						<span class="badge preset-tonal font-mono">regex:{node.pattern.slice(3)}</span>
					{:else if node.pattern === 'fallback'}
						<span class="badge preset-tonal font-mono">fallback</span>
					{:else}
						<span class="font-bold">{node.pattern}</span>
					{/if}
					<div class="badge text-sm preset-filled-primary-200-800 font-mono">
						<LogOutIcon class="size-4 " />
						<span >{node.port}</span>
						{#if node.rules_count > 0}
							({node.rules_count}) rules
						{/if}
					</div>
				</div>
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<!--  Output editor -->
<div class="label">
	<span class="label-text">Output Ports</span>
</div>
<div class="mb-4 space-y-2">
	<TableListEditor
		value={outputAsTable}
		onChange={(newTable) => {
			value = {
				...value,
				output: outputTableAsOutput(newTable)
			};
		}}
	>
		{#snippet row({
			deleteItem,
			updateItem,
			value,
			index
		}: RowParams<
			OutputItem & {
				port: string;
			}
		>)}
			<tr>
				<td>
					<input
						type="text"
						class="input-sm input w-full font-mono"
						{readonly}
						bind:this={outputPortInputList[index]}
						value={value.port}
						onchange={({ currentTarget: { value: port } }) => updateItem({ ...value, port })}
					/>
				</td>
				<td>
					<TargetSelector
						{httpEditorContext}
						value={value.target}
						onChange={(target) => {
							if (target !== undefined) {
								updateItem({ ...value, target });
							}
						}}
					></TargetSelector>
				</td>
				<td>
					<FilterSelector
						{httpEditorContext}
						value={value.filters || []}
						onChange={(filters) => {
							updateItem({ ...value, filters });
						}}
					></FilterSelector>
				</td>
				<td>
					<button
						type="button"
						class="btn-icon preset-tonal-error"
						onclick={(e) => deleteItem()}
						disabled={readonly}
					>
						<DeleteIcon />
					</button>
				</td>
			</tr>
		{/snippet}
		{#snippet header()}
			<tr>
				<th>Port</th>
				<th>Target</th>
				<th>Filters</th>
				<th></th>
			</tr>
		{/snippet}
		{#snippet footer({ value, addNewItem }: ListOperations<OutputItem & { port: string }>)}
			<tr>
				<td colspan="3"> </td>
				<td>
					<button
						type="button"
						class="btn-icon preset-tonal-surface"
						onclick={() =>
							addNewItem({
								port: `port${value.length + 1}`,
								target: '',
								filters: []
							})}
					>
						<PlusIcon class="size-4" />
					</button>
				</td>
			</tr>
		{/snippet}
	</TableListEditor>
</div>
<hr />
<div class="flex h-[500px]">
	<!-- Router editor -->
	<!-- Left Side: Tree View -->
	<div class="w-1/3 overflow-y-auto border-r border-surface-500/30">
		<!-- control bar -->
		<div class="flex items-center justify-between p-1">
			<div class="label">
				<span class="label-text">Router Tree</span>
			</div>
			<div class="flex gap-1">
				<button type="button" class="btn-icon btn-icon-sm preset-tonal-surface" onclick={addHostname} title="Add Hostname">
					<BadgedIcon>
						<GlobeIcon class="size-4" />
						{#snippet badge()}
							<CirclePlusIcon strokeWidth={3} />
						{/snippet}
					</BadgedIcon>
				</button>
				{#if selectedNode?.kind == 'hostname'}
					<button type="button" class="btn-icon btn-icon-sm preset-tonal-surface" onclick={() => {
						selectedNode?.kind == 'hostname' ? addPath(selectedNode.pattern) : void 0
					}} title="Add Path">
						<BadgedIcon>
							<RouteIcon class="size-4" />
							{#snippet badge()}
								<CirclePlusIcon strokeWidth={3} />
							{/snippet}
						</BadgedIcon>
					</button>
				{/if}
				{#if selectedNode?.kind == 'hostname'}
					<button type="button" class="btn-icon btn-icon-sm preset-tonal-error" onclick={() => {
						selectedNode?.kind == 'hostname' ? deleteHostname(selectedNode.pattern) : void 0
					}} title="Delete Hostname">
						<BadgedIcon>
							<GlobeIcon class="size-4" />
							{#snippet badge()}
								<TrashIcon strokeWidth={3} />
							{/snippet}
						</BadgedIcon>
					</button>
				{:else if selectedNode?.kind == 'path'}

					<button type="button" class="btn-icon btn-icon-sm preset-tonal-error" onclick={() => {
						if (selectedNode && selectedNode.kind === 'path') {
							deletePath(selectedNode.hostname, selectedNode.pattern);
						}
                    }} title="Delete Path">
						<BadgedIcon>
							<RouteIcon class="size-4" />
							{#snippet badge()}
								<TrashIcon strokeWidth={3} />
							{/snippet}
						</BadgedIcon>
					</button>
				{/if}
			</div>
		</div>
		<hr />
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
			<TreeView.Tree class="w-full">
				<!-- Start rendering from root children -->
				{@render treeNode(treeCollection.rootNode, [])}
			</TreeView.Tree>
		</TreeView>
	</div>

	<!-- Right Side: Details Panel -->
	<div class="flex-1 overflow-y-auto p-2">
		{#if selectedNode}
			<div class="space-y-4">
				{#if selectedNode.kind === 'hostname'}
					<h4 class="h4">{selectedNode.id}</h4>
					<div class="grid grid-cols-[100px_1fr] gap-2 text-sm items-center">
						<div class="font-bold opacity-60">Type:</div>
						<div>Hostname</div>
						<div class="font-bold opacity-60">Pattern:</div>
						<input
							type="text"
							class="input input-sm font-mono"
							value={selectedNode.pattern}
							onchange={(e) => {
								if (selectedNode && selectedNode.kind === 'hostname') {
									renameHostname(selectedNode.pattern, e.currentTarget.value)
								}
							}}
						/>
						<div class="font-bold opacity-60">Paths:</div>
						<div>{selectedNode.paths.length}</div>
					</div>
				{:else if selectedNode.kind === 'path'}
					{@const ruleBucket = getRuleBucket(selectedNode.hostname, selectedNode.pattern)}

					{#if ruleBucket}
						{@const targetPort = targetOfRuleBucket(ruleBucket)}
						{@const rules = rulesOfRuleBucket(ruleBucket)}
						<div class="block-inline space-x-1">
							{#if selectedNode.pattern === 'fallback'}
								<span class="h4">
									{selectedNode.hostname}
								</span>
								<span class="badge preset-tonal-secondary py-0 font-mono">fallback</span>
							{:else if selectedNode.pattern.startsWith('re:')}
								<span class="h4">
									{selectedNode.hostname}
								</span>
								<span class="badge preset-tonal-secondary py-0 font-mono"
									>regex:{selectedNode.pattern.slice(3)}</span
								>
							{:else}
								<span class="h4">
									{selectedNode.hostname}{selectedNode.pattern}
								</span>
							{/if}
						</div>
						<div class="flex items-center gap-2 mt-2">
							<span class="text-sm font-bold opacity-60">Pattern:</span>
							<input
								type="text"
								class="input input-sm font-mono flex-1"
								value={selectedNode.pattern}
								onchange={(e) => {
										if (selectedNode && selectedNode.kind === 'path') {
											renamePath(selectedNode.hostname, selectedNode.pattern, e.currentTarget.value)
										}
								}}
							/>
						</div>
						<div class="label label-text">Output Port:</div>
						<div class="flex items-center">
							<div class="input-group flex flex-grow">
								<select
									class="ig-select"
									value={targetPort}
									onchange={(e) => {
										// Update logic using updateRuleBucket or similar
										if (ruleBucket && selectedNode && selectedNode.kind === 'path') {
											const currentRules = rulesOfRuleBucket(ruleBucket);
											const newBucket = {
												rules: currentRules,
												target: e.currentTarget.value
											};
											updateRuleBucket(selectedNode.hostname, selectedNode.pattern, newBucket);
										}
									}}
								>
									{#each outputAsTable as output, index}
										<option value={output.port}>{output.port}</option>
									{/each}
								</select>
								<button
									class="ig-btn anchor font-mono"
									onclick={() => selectedNode?.kind === 'path' && focusOnOutput(selectedNode.port)}
								>
									<ExternalLinkIcon class="size-4" />
								</button>
							</div>
						</div>
						<div class="label label-text">Rules:</div>
						<TableListEditor value={rules}>
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
									<td>
										<button
											type="button"
											class="btn-icon preset-tonal-error"
											onclick={(e) => deleteItem()}
										>
											<DeleteIcon />
										</button>
									</td>
								</tr>
							{/snippet}
							{#snippet footer({ addNewItem, value }: ListOperations<string[]>)}
								<tr>
									<td>
										{#if value.length === 0}
											<span class="mt-1 text-sm opacity-50"
												>No rules defined. All requests match.</span
											>
										{:else}
											<span class="text-sm font-medium"> {value.length} rules</span>
										{/if}
									</td>
									<td>
										<button
											type="button"
											class="btn-icon preset-tonal"
											onclick={() => addNewItem([])}
										>
											<PlusIcon class="size-4" />
										</button>
									</td>
								</tr>
							{/snippet}
						</TableListEditor>
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
