<script lang="ts">
	import { Circle, Filter, AlertTriangle, PlayCircle } from 'lucide-svelte';
	import { getHttpClassEditorPlugin } from '$lib/plugins/registry';

	let {
		id,
		classId,
		instanceType,
		selected = false,
		isEntry = false,
		isCycle = false,
		onclick
	}: {
		id: string;
		classId?: string;
		config?: any;
		instanceType: 'node' | 'filter';
		selected?: boolean;
		isEntry?: boolean;
		isCycle?: boolean;
		onclick?: () => void;
	} = $props();

	let plugin = $derived(classId ? getHttpClassEditorPlugin(classId, instanceType) : undefined);
	let displayName = $derived(plugin?.displayName || classId || 'Not configured');
</script>

<button
	class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left transition-colors text-sm
    {selected
		? 'bg-primary-500/20 text-primary-500'
		: 'hover:bg-surface-100 dark:hover:bg-surface-800'}"
	{onclick}
	type="button"
>
	<!-- Icon -->
	{#if isCycle}
		<AlertTriangle size={16} class="text-warning-500 shrink-0" />
	{:else if instanceType === 'node'}
		<Circle size={16} class="shrink-0 {isEntry ? 'fill-current' : ''}" />
	{:else}
		<Filter size={16} class="shrink-0" />
	{/if}

	<!-- Content -->
	<div class="flex-1 min-w-0 overflow-hidden">
		<div class="flex items-center gap-2 truncate">
			<span class="font-medium truncate">{id}</span>
			{#if isEntry}
				<span class="badge badge-sm variant-soft-primary px-1 py-0 text-[10px] uppercase font-bold"
					>Entry</span
				>
			{/if}
		</div>
		<div class="text-xs opacity-70 truncate" title={displayName}>
			{displayName}
		</div>
	</div>

	<!-- Cycle Indicator Text -->
	{#if isCycle}
		<span class="text-[10px] text-warning-500 uppercase font-bold tracking-wider">Cycle</span>
	{/if}
</button>
