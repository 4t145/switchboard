<script lang="ts">
	import { onMount } from 'svelte';
	import { FileIcon, FolderIcon, LoaderCircleIcon } from '@lucide/svelte';
	import { createTreeViewCollection, TreeView } from '@skeletonlabs/skeleton-svelte';
	import { api } from '$lib/api/routes';
	import type { FsEntry } from '$lib/api/types/file_browser';

	type Props = {
		selectedFilePath?: string;
	};

	type FileTreeNode = {
		id: string;
		name: string;
		relativePath: string;
		isDirectory: boolean;
		children?: FileTreeNode[];
		childrenCount?: number;
	};

	let { selectedFilePath = $bindable(undefined) }: Props = $props();

	let allowedRoots = $state<string[]>([]);
	let selectedRoot = $state<string | undefined>(undefined);
	let selectedValue = $state<string[] | undefined>(undefined);
	let expandedValue = $state<string[] | undefined>(undefined);
	let loading = $state(false);
	let errorMessage = $state<string | undefined>(undefined);

	let collection = $state(
		createTreeViewCollection<FileTreeNode>({
			nodeToValue: (node) => node.id,
			nodeToString: (node) => node.name,
			rootNode: {
				id: 'virtual-root',
				name: '',
				relativePath: '',
				isDirectory: true,
				children: []
			}
		})
	);

	onMount(async () => {
		await loadRoots();
	});

	const loadChildren = async (details: { node: FileTreeNode }) => {
		if (!selectedRoot || !details.node.isDirectory) return [] as FileTreeNode[];
		const root = selectedRoot;
		const response = await api.fileBrowser.getEntry({
			root,
			relativePath: details.node.relativePath.length > 0 ? details.node.relativePath : undefined,
			listChildren: true
		});
		return (response.children ?? []).map((entry) => buildNodeFromEntry(root, entry));
	};

	const onLoadChildrenComplete = (details: { collection: typeof collection }) => {
		collection = details.collection;
	};

	const onLoadChildrenError = () => {
		errorMessage = 'Failed to load directory.';
	};

	function onSelectionChange(nextSelected: string[] | undefined) {
		selectedValue = nextSelected;
		if (!selectedRoot) {
			selectedFilePath = undefined;
			return;
		}
		const selectedNodeId = nextSelected?.at(0);
		if (!selectedNodeId) {
			selectedFilePath = undefined;
			return;
		}
		const selectedNode = findNodeById(collection.rootNode, selectedNodeId);
		if (!selectedNode || selectedNode.isDirectory) {
			selectedFilePath = undefined;
			return;
		}
		selectedFilePath = buildAbsolutePath(selectedRoot, selectedNode.relativePath);
	}

	async function loadRoots() {
		loading = true;
		errorMessage = undefined;
		try {
			const response = await api.fileBrowser.getRoots();
			allowedRoots = response.roots;
			selectedRoot = response.roots.at(0);
			if (selectedRoot) {
				await loadRootEntry(selectedRoot);
			}
		} catch (error) {
			console.error('Failed to load allowed roots', error);
			errorMessage = 'Failed to load allowed roots.';
		} finally {
			loading = false;
		}
	}

	async function loadRootEntry(root: string) {
		loading = true;
		errorMessage = undefined;
		selectedFilePath = undefined;
		selectedValue = undefined;
		expandedValue = undefined;
		try {
			const response = await api.fileBrowser.getEntry({ root, listChildren: true });
			const rootEntry = buildNodeFromEntry(root, response.entry, response.children ?? []);
			collection = createTreeViewCollection<FileTreeNode>({
				nodeToValue: (node) => node.id,
				nodeToString: (node) => node.name,
				rootNode: {
					id: 'virtual-root',
					name: '',
					relativePath: '',
					isDirectory: true,
					children: [rootEntry]
				}
			});
		} catch (error) {
			console.error('Failed to load root entry', error);
			errorMessage = 'Failed to load files under selected root.';
			collection = createTreeViewCollection<FileTreeNode>({
				nodeToValue: (node) => node.id,
				nodeToString: (node) => node.name,
				rootNode: {
					id: 'virtual-root',
					name: '',
					relativePath: '',
					isDirectory: true,
					children: []
				}
			});
		} finally {
			loading = false;
		}
	}

	function buildNodeFromEntry(root: string, entry: FsEntry, children: FsEntry[] = []): FileTreeNode {
		const nodeChildren = children.map((child) => buildNodeFromEntry(root, child));
		const hasChildren = entry.has_children ?? false;
		return {
			id: buildNodeId(root, toRelativePath(root, entry.path)),
			name: entry.name,
			relativePath: toRelativePath(root, entry.path),
			isDirectory: entry.entry_type === 'directory',
			children: nodeChildren.length > 0 ? nodeChildren : undefined,
			childrenCount: entry.entry_type === 'directory' && hasChildren ? (nodeChildren.length || 1) : undefined
		};
	}

	function findNodeById(node: FileTreeNode, nodeId: string): FileTreeNode | undefined {
		if (node.id === nodeId) return node;
		for (const child of node.children ?? []) {
			const found = findNodeById(child, nodeId);
			if (found) return found;
		}
		return undefined;
	}

	function toRelativePath(root: string, absolutePath: string): string {
		const normalizedRoot = root.replace(/\\/g, '/').replace(/\/$/, '');
		const normalizedAbsolute = absolutePath.replace(/\\/g, '/');
		if (normalizedAbsolute === normalizedRoot) return '';
		const rootPrefix = `${normalizedRoot}/`;
		if (normalizedAbsolute.startsWith(rootPrefix)) {
			return normalizedAbsolute.slice(rootPrefix.length);
		}
		return normalizedAbsolute;
	}

	function buildNodeId(root: string, relativePath: string): string {
		return `${root}::${relativePath.length > 0 ? relativePath : '.'}`;
	}

	function buildAbsolutePath(root: string, relativePath: string): string {
		if (!relativePath) return root;
		if (root.endsWith('/') || root.endsWith('\\')) return `${root}${relativePath}`;
		return `${root}/${relativePath}`;
	}
</script>

<div class="space-y-3">
	<label class="label" for="file-tree-root-select">
		<span>Allowed Root</span>
	</label>
	<select
		id="file-tree-root-select"
		class="select"
		bind:value={selectedRoot}
		onchange={async () => {
			if (selectedRoot) {
				await loadRootEntry(selectedRoot);
			}
		}}
		disabled={loading || allowedRoots.length === 0}
	>
		{#each allowedRoots as root (root)}
			<option value={root}>{root}</option>
		{/each}
	</select>

	{#if errorMessage}
		<div class="alert preset-tonal-error">{errorMessage}</div>
	{/if}

	<div class="rounded border border-surface-200 p-2 dark:border-surface-700">
		{#if loading}
			<div class="flex items-center gap-2 p-3 text-sm opacity-70">
				<LoaderCircleIcon class="size-4 animate-spin" />
				Loading files...
			</div>
		{:else if !collection.rootNode.children?.length}
			<div class="p-3 text-sm opacity-70">No root available.</div>
		{:else}
			<TreeView
				{collection}
				{selectedValue}
				{expandedValue}
				{loadChildren}
				{onLoadChildrenComplete}
				{onLoadChildrenError}
				onSelectionChange={(event) => onSelectionChange(event.selectedValue)}
				onExpandedChange={(event) => {
					expandedValue = event.expandedValue;
				}}
				selectionMode="single"
			>
				<TreeView.Tree class="w-full">
					{#each collection.rootNode.children ?? [] as node, index (node.id)}
						{@render treeNode(node, [index])}
					{/each}
				</TreeView.Tree>
			</TreeView>
		{/if}
	</div>
</div>

{#snippet treeNode(node: FileTreeNode, indexPath: number[])}
	<TreeView.NodeProvider value={{ node, indexPath }}>
		{#if node.children || node.childrenCount}
			<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator class="data-loading:hidden" />
					<TreeView.BranchIndicator class="hidden data-loading:inline animate-spin">
						<LoaderCircleIcon class="size-4" />
					</TreeView.BranchIndicator>
					<TreeView.BranchText>
						<FolderIcon class="size-4" />
						{node.name}
					</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					<TreeView.BranchIndentGuide />
					{#each node.children ?? [] as childNode, childIndex (childNode.id)}
						{@render treeNode(childNode, [...indexPath, childIndex])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else}
			<TreeView.Item>
				<FileIcon class="size-4" />
				{node.name}
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}
