<script lang="ts">
	import type { TlsOptions } from '$lib/api/types/tls';
	import { Check, X, ChevronDown } from '@lucide/svelte';

	interface Props {
		options: TlsOptions;
	}

	let { options }: Props = $props();

	function formatBytes(bytes: number | undefined): string {
		if (bytes === undefined || bytes === 0) return '0 Bytes';
		const k = 1024;
		const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}
</script>

<details class="group">
	<summary
		class="rounded-token flex cursor-pointer list-none items-center justify-between p-2 text-sm font-medium text-surface-700 hover:bg-surface-100 dark:text-surface-300 dark:hover:bg-surface-800"
	>
		<span>Advanced Options</span>
		<ChevronDown class="h-4 w-4 transition-transform duration-200 group-open:rotate-180" />
	</summary>
	<div
		class="grid grid-cols-1 gap-x-6 gap-y-4 border-t border-surface-200 p-4 text-sm md:grid-cols-2 lg:grid-cols-3 dark:border-surface-700"
	>
		<!-- ALPN Protocols -->
		{#if options.alpn_protocols && options.alpn_protocols.length > 0}
			<div class="space-y-1 md:col-span-2 lg:col-span-3">
				<span class="font-medium text-surface-600 dark:text-surface-400">ALPN Protocols</span>
				<div class="flex flex-wrap gap-2">
					{#each options.alpn_protocols as protocol}
						<span class="badge preset-tonal-primary">{protocol}</span>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Max Fragment Size -->
		<div class="space-y-1">
			<span class="font-medium text-surface-600 dark:text-surface-400">Max Fragment Size</span>
			<p class="text-surface-800 dark:text-surface-200">
				{options.max_fragment_size ? formatBytes(options.max_fragment_size) : 'Default'}
			</p>
		</div>

		<!-- Max Early Data Size -->
		<div class="space-y-1">
			<span class="font-medium text-surface-600 dark:text-surface-400">Max Early Data Size</span>
			<p class="text-surface-800 dark:text-surface-200">
				{formatBytes(options.max_early_data_size)}
			</p>
		</div>

		<!-- TLS 1.3 Tickets -->
		<div class="space-y-1">
			<span class="font-medium text-surface-600 dark:text-surface-400">TLS 1.3 Tickets</span>
			<p class="text-surface-800 dark:text-surface-200">{options.send_tls13_tickets}</p>
		</div>

		<!-- Ignore Client Order -->
		<div class="flex items-center gap-2">
			{#if options.ignore_client_order}
				<Check class="h-4 w-4 text-success-500" />
				<span class="text-surface-800 dark:text-surface-200">Ignore Client Cipher Order</span>
			{:else}
				<X class="h-4 w-4 text-surface-500" />
				<span class="text-surface-800 dark:text-surface-200">Respect Client Cipher Order</span>
			{/if}
		</div>

		<!-- Enable Secret Extraction -->
		<div class="flex items-center gap-2">
			{#if options.enable_secret_extraction}
				<Check class="h-4 w-4 text-success-500" />
				<span class="text-surface-800 dark:text-surface-200">Secret Extraction Enabled</span>
			{:else}
				<X class="h-4 w-4 text-surface-500" />
				<span class="text-surface-800 dark:text-surface-200">Secret Extraction Disabled</span>
			{/if}
		</div>

		<!-- Send Half RTT Data -->
		<div class="flex items-center gap-2">
			{#if options.send_half_rtt_data}
				<Check class="h-4 w-4 text-success-500" />
				<span class="text-surface-800 dark:text-surface-200">0.5-RTT Data Enabled</span>
			{:else}
				<X class="h-4 w-4 text-surface-500" />
				<span class="text-surface-800 dark:text-surface-200">0.5-RTT Data Disabled</span>
			{/if}
		</div>

		<!-- Require Extended Master Secret -->
		<div class="flex items-center gap-2">
			{#if options.require_ems}
				<Check class="h-4 w-4 text-success-500" />
				<span class="text-surface-800 dark:text-surface-200"
					>Extended Master Secret (EMS) Required</span
				>
			{:else}
				<X class="h-4 w-4 text-surface-500" />
				<span class="text-surface-800 dark:text-surface-200">EMS Not Required</span>
			{/if}
		</div>
	</div>
</details>
