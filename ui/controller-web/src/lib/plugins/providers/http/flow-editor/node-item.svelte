<script lang="ts">
	import {
		ArrowRight,
		CornerDownRight,
		Circle,
		Filter as FilterIcon,
		AlertCircle,
		ChevronDown,
		ChevronRight
	} from 'lucide-svelte';
	import { getHttpClassEditorPlugin } from '$lib/plugins/registry';

	type OutputInfo = {
		port: string;
		target: string;
		filters?: string[];
		label?: string;
	};

	type Props = {
		id: string;
		classId?: string;
		config: any;
		instanceType: 'node' | 'filter';
		isEntry?: boolean;
		selected?: boolean;
		onclick?: () => void;
	};

	let {
		id,
		classId,
		config,
		instanceType,
		isEntry = false,
		selected = false,
		onclick
	}: Props = $props();

	// 获取 plugin 并提取 outputs
	const plugin = $derived(classId ? getHttpClassEditorPlugin(classId, instanceType) : null);
	const outputs = $derived<OutputInfo[]>(
		plugin?.extractOutputs && config ? plugin.extractOutputs(config) : []
	);

	let expanded = $state(true);
</script>

<div class="node-item">
	<!-- Node Header -->
	<div
		class="node-header flex items-center gap-2 w-full px-3 py-2 rounded transition-colors cursor-pointer"
		class:bg-primary-500={selected}
		class:text-white={selected}
		class:hover:bg-surface-200={!selected}
		class:dark:hover:bg-surface-700={!selected}
		onclick={onclick}
		role="button"
		tabindex="0"
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				onclick?.();
			}
		}}
	>
		<!-- Icon -->
		{#if instanceType === 'node'}
			<Circle size={16} class={(isEntry ? 'text-primary-500' : '') + (selected ? ' text-white' : '')} />
		{:else}
			<FilterIcon size={16} class={selected ? 'text-white' : ''} />
		{/if}

		<!-- ID -->
		<span class="flex-1 text-sm font-medium">{id}</span>

		<!-- Warning if not configured -->
		{#if !classId}
			<div title="Class not configured">
				<AlertCircle size={14} class="text-warning-500" />
			</div>
		{/if}

		<!-- Class name -->
		{#if classId}
			<span class="text-xs opacity-60" class:opacity-100={selected}>{plugin?.displayName || classId}</span>
		{:else}
			<span class="text-xs opacity-60" class:opacity-100={selected}>Not configured</span>
		{/if}

		<!-- Expand/collapse button -->
		{#if outputs.length > 0}
			<div
				class="expand-btn p-1 hover:bg-surface-300 dark:hover:bg-surface-600 rounded"
				onclick={(e) => {
					e.stopPropagation();
					expanded = !expanded;
				}}
				role="button"
				tabindex="0"
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.stopPropagation();
						expanded = !expanded;
					}
				}}
			>
				{#if expanded}
					<ChevronDown size={14} />
				{:else}
					<ChevronRight size={14} />
				{/if}
			</div>
		{/if}
	</div>

	<!-- Outputs (展开显示) -->
	{#if expanded && outputs.length > 0}
		<div class="outputs ml-8 mt-1 space-y-1 text-xs opacity-75">
			{#each outputs as output}
				<div class="output-item space-y-0.5">
					<!-- Port name -->
					<div class="flex items-center gap-1">
						<ArrowRight size={12} />
						<span class="font-medium">{output.label || output.port}</span>
					</div>

					<!-- Target node -->
					{#if output.target}
						<div class="ml-4 flex items-center gap-1">
							<CornerDownRight size={12} />
							<span class="text-primary-600 dark:text-primary-400">{output.target}</span>
						</div>
					{/if}

					<!-- Filters -->
					{#if output.filters && output.filters.length > 0}
						<div class="ml-4 flex items-center gap-1 opacity-60">
							<CornerDownRight size={12} />
							<span>via: {output.filters.join(', ')}</span>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.node-item {
		transition: all 0.15s ease;
	}

	.node-header {
		cursor: pointer;
	}

	.output-item {
		padding-left: 0.25rem;
		border-left: 2px solid rgb(var(--color-surface-300));
	}

	:global(.dark) .output-item {
		border-left-color: rgb(var(--color-surface-600));
	}
</style>
