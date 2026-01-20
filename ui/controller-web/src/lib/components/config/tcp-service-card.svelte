<script lang="ts">
	import type { FileTcpServiceConfig } from '$lib/api/types/human_readable';
	import { Server, Settings } from 'lucide-svelte';
	import BindItem from './bind-item.svelte';
	import LinkOrValueDisplay from './link-or-value-display.svelte';

	interface Props {
		service: FileTcpServiceConfig;
		jumpToTls?: ((tlsName: string) => void) | null;
	}

	let { service, jumpToTls = null }: Props = $props();
</script>

<div class="card p-4 space-y-3">
	<!-- Service header -->
	<div class="flex items-start justify-between gap-4">
		<div class="flex items-center gap-2 min-w-0 flex-1">
			<Server class="w-5 h-5 text-primary-500 flex-shrink-0" />
			<h3 class="font-semibold text-lg truncate">{service.name}</h3>
		</div>
	</div>

	<!-- Provider -->
	<div class="text-sm text-surface-600 dark:text-surface-400">
		Provider:
		<code class="bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded ml-1">
			{service.provider}
		</code>
	</div>

	<!-- Description -->
	{#if service.description}
		<p class="text-sm text-surface-600 dark:text-surface-400">
			{service.description}
		</p>
	{/if}

	<!-- Bindings -->
	{#if service.binds.length > 0}
		<div class="space-y-2">
			<div class="text-sm font-medium text-surface-700 dark:text-surface-300">
				Bindings ({service.binds.length}):
			</div>
			<div class="space-y-1.5 pl-2">
				{#each service.binds as bind}
					<BindItem {bind} {jumpToTls} />
				{/each}
			</div>
		</div>
	{/if}

	<!-- Config -->
	{#if service.config}
		<div class="pt-2 border-t border-surface-300 dark:border-surface-700">
			<div class="flex items-center gap-2 text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
				<Settings class="w-4 h-4" />
				Configuration:
			</div>
			<div class="pl-2">
				<LinkOrValueDisplay value={service.config} resolveContent={"string"} />
			</div>
		</div>
	{/if}
</div>
