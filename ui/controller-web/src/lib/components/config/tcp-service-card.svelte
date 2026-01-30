<script lang="ts">
	import type { FileTcpServiceConfig } from '$lib/api/types/human_readable';
	import { Server, Settings } from '@lucide/svelte';
	import BindItem from './bind-item.svelte';
	import LinkOrValueDisplay from './link-or-value-display.svelte';

	interface Props {
		service: FileTcpServiceConfig;
		jumpToTls?: ((tlsName: string) => void) | null;
	}

	let { service, jumpToTls = null }: Props = $props();
</script>

<div class="space-y-3 card p-4">
	<!-- Service header -->
	<div class="flex items-start justify-between gap-4">
		<div class="flex min-w-0 flex-1 items-center gap-2">
			<Server class="h-5 w-5 flex-shrink-0 text-primary-500" />
			<h3 class="truncate text-lg font-semibold">{service.name}</h3>
		</div>
	</div>

	<!-- Provider -->
	<div class="text-sm text-surface-600 dark:text-surface-400">
		Provider:
		<code class="ml-1 rounded bg-surface-200 px-2 py-0.5 dark:bg-surface-700">
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
		<div class="border-t pt-2 dark:border-surface-700">
			<div
				class="mb-2 flex items-center gap-2 text-sm font-medium text-surface-700 dark:text-surface-300"
			>
				<Settings class="h-4 w-4" />
				Configuration:
			</div>
			<div class="pl-2">
				<LinkOrValueDisplay
					value={service.config}
					resolveContent="value"
					dataType="TcpServiceConfig"
					editorProps={{ provider: service.provider }}
				/>
			</div>
		</div>
	{/if}
</div>
