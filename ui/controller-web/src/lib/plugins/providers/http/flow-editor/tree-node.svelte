<script lang="ts">
	import { Circle, Filter, ChevronRight, ChevronDown, AlertTriangle } from 'lucide-svelte';
	import { getHttpClassEditorPlugin } from '$lib/plugins/registry';
	import type { TreeNode, TreeConnection } from './tree-builder';
	import type { Snippet } from 'svelte';

	type Props = {
		node: TreeNode | TreeConnection;
		depth: number;
		isExpanded: boolean;
		hasChildren: boolean;
		onToggleExpand: () => void;
		onSelect: (id: string, type: 'node' | 'filter') => void;
		isSelected: boolean;
		children?: Snippet;
	};

	let {
		node,
		depth,
		isExpanded,
		hasChildren,
		onToggleExpand,
		onSelect,
		isSelected,
		children
	}: Props = $props();

	const isConnection = $derived('label' in node);
	const treeNode = $derived(!isConnection ? (node as TreeNode) : null);

	const plugin = $derived(
		treeNode?.data?.class ? getHttpClassEditorPlugin(treeNode.data.class, treeNode.type) : null
	);
	const displayName = $derived(plugin?.displayName || treeNode?.data?.class || 'Not configured');
</script>

{#if isConnection}
	<!-- Connection Node (极简) -->
	<div class="tree-connection" style:padding-left={`${depth * 16 + 16}px`}>
		<span class="connection-dot"></span>
		<span class="connection-label">{(node as TreeConnection).label}</span>
	</div>
{:else if treeNode}
	<!-- Node/Filter -->
	<div
		class="tree-node"
		class:selected={isSelected}
		style:padding-left={`${depth * 16}px`}
	>
		<!-- Expand/Collapse Button -->
		{#if hasChildren}
			<button
				class="expand-btn"
				onclick={(e) => {
					e.stopPropagation();
					onToggleExpand();
				}}
				title={isExpanded ? 'Collapse' : 'Expand'}
			>
				{#if isExpanded}
					<ChevronDown class="size-3" />
				{:else}
					<ChevronRight class="size-3" />
				{/if}
			</button>
		{:else}
			<span class="expand-spacer"></span>
		{/if}

		<!-- Node Content -->
		<button
			class="node-content"
			class:cycle={treeNode.isCycle}
			onclick={() => onSelect(treeNode.id, treeNode.type)}
			aria-label={`Select ${treeNode.type} ${treeNode.id}`}
		>
			<!-- Icon -->
			{#if treeNode.isCycle}
				<AlertTriangle class="icon warning" />
			{:else if treeNode.type === 'node'}
				<Circle class="icon" />
			{:else}
				<Filter class="icon" />
			{/if}

			<!-- Text -->
			<div class="text-group">
				<div class="primary-text">
					{treeNode.id}
					{#if treeNode.isEntry}
						<span class="entry-badge">Entry</span>
					{/if}
				</div>
				<div class="secondary-text" title={displayName}>
					{displayName}
				</div>
			</div>

			<!-- Cycle Badge -->
			{#if treeNode.isCycle}
				<span class="cycle-badge">Cycle</span>
			{/if}
		</button>
	</div>

	<!-- Render Children Slot if provided and expanded -->
	{#if treeNode && children && isExpanded}
		{@render children()}
	{/if}
{/if}

<style>
	/* Tree Node Styles */
	.tree-node {
		padding: 4px 0;
	}

	.tree-node.selected .node-content {
		background: var(--primary-500/10);
		border-radius: 4px;
	}

	.expand-btn {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		background: transparent;
		cursor: pointer;
		color: var(--surface-500-400);
		transition: color 0.15s ease;
	}

	.expand-btn:hover {
		color: var(--surface-900-50);
	}

	.expand-spacer {
		width: 20px;
	}

	.node-content {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px;
		width: 100%;
		border: none;
		background: transparent;
		cursor: pointer;
		border-radius: 4px;
		text-align: left;
		transition: background 0.15s ease;
	}

	.node-content:hover {
		background: var(--surface-100-900);
	}

	.node-content.cycle {
		color: var(--warning-500);
	}

	.icon {
		size: 16px;
		flex-shrink: 0;
		color: var(--surface-500-400);
	}

	.icon.warning {
		color: var(--warning-500);
	}

	.text-group {
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.primary-text {
		font-size: 13px;
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 6px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.secondary-text {
		font-size: 11px;
		color: var(--surface-400-600);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.entry-badge {
		font-size: 9px;
		padding: 1px 4px;
		background: var(--primary-600-400);
		color: var(--surface-50-950);
		border-radius: 2px;
		text-transform: uppercase;
		flex-shrink: 0;
	}

	.cycle-badge {
		font-size: 9px;
		padding: 1px 4px;
		background: var(--warning-600-400);
		color: var(--surface-50-950);
		border-radius: 2px;
		text-transform: uppercase;
		flex-shrink: 0;
	}

	/* Connection Styles */
	.tree-connection {
		padding: 2px 0;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.connection-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--surface-400-600);
		flex-shrink: 0;
	}

	.connection-label {
		font-size: 11px;
		font-family: monospace;
		color: var(--surface-500-400);
	}
</style>
