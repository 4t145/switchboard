<script lang="ts">
	import { Handle, Position, type NodeProps } from '@xyflow/svelte';
	import { ServerIcon, SplitIcon } from 'lucide-svelte';

	export let data: {
		id: string;
		type: string;
		inputs: string[];
		outputs: string[];
		isEntrypoint: boolean;
		config: unknown;
	};
	export let selected: NodeProps['selected'];

	function isFilterType(type: string): boolean {
		const filterTypes = ['Filter', 'Route', 'Switch', 'Conditional'];
		return filterTypes.some((filterType) => type.includes(filterType));
	}
</script>

<div class="custom-node {selected ? 'selected' : ''}">
	<div class="node-header">
		{#if isFilterType(data.type)}
			<SplitIcon class="node-icon" size={16} />
		{:else}
			<ServerIcon class="node-icon" size={16} />
		{/if}
		<span class="node-id">{data.id}</span>
		{#if data.isEntrypoint}
			<span class="entrypoint-badge">entrypoint</span>
		{/if}
	</div>
	<div class="node-body">
		<div class="node-type">{data.type}</div>
	</div>

	{#each data.inputs as inputPort}
		<Handle
			type="target"
			position={Position.Left}
			id={inputPort}
			class="handle handle-input"
		/>
		<div class="port-label port-input">{inputPort}</div>
	{/each}

	{#each data.outputs as outputPort}
		<Handle
			type="source"
			position={Position.Right}
			id={outputPort}
			class="handle handle-output"
		/>
		<div class="port-label port-output">{outputPort}</div>
	{/each}
</div>

<style>
	.custom-node {
		width: 200px;
		min-height: 120px;
		background-color: var(--md-sys-color-surface);
		border: 2px solid var(--md-sys-color-outline);
		border-radius: 8px;
		padding: 8px;
		font-family: var(--font-sans);
		transition: border-color 0.2s;
		position: relative;
	}

	.custom-node.selected {
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 0 0 2px var(--md-sys-color-primary-container);
	}

	.node-header {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 8px;
	}

	.node-icon {
		flex-shrink: 0;
	}

	.node-id {
		font-weight: 600;
		font-size: 13px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.entrypoint-badge {
		font-size: 10px;
		padding: 2px 6px;
		background-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		border-radius: 4px;
		margin-left: auto;
	}

	.node-body {
		margin-bottom: 8px;
	}

	.node-type {
		font-size: 12px;
		color: var(--md-sys-color-on-surface-variant);
	}

	.port-label {
		position: absolute;
		font-size: 10px;
		color: var(--md-sys-color-on-surface-variant);
		background: var(--md-sys-color-surface-container);
		padding: 2px 4px;
		border-radius: 4px;
		white-space: nowrap;
	}

	.port-input {
		left: 8px;
	}

	.port-output {
		right: 8px;
	}

	:global(.handle) {
		width: 8px;
		height: 8px;
		background-color: var(--md-sys-color-outline);
		border: 1px solid var(--md-sys-color-surface);
	}

	:global(.handle-input) {
		left: -4px;
	}

	:global(.handle-output) {
		right: -4px;
	}
</style>
