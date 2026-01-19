<script lang="ts">
	import { onMount } from 'svelte';
	import type { HumanReadableServiceConfig } from '$lib/api/types/human_readable';
	import { getCurrentConfig } from '$lib/api/config';
	import { RefreshCw, AlertCircle, Server, Lock, Loader2 } from 'lucide-svelte';
	import TcpServiceCard from '$lib/components/config/tcp-service-card.svelte';
	import TlsConfigCard from '$lib/components/config/tls-config-card.svelte';

	let config = $state<HumanReadableServiceConfig | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let lastUpdated = $state<Date | null>(null);

	async function loadConfig() {
		loading = true;
		error = null;
		try {
			config = await getCurrentConfig();
			lastUpdated = new Date();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load configuration';
			config = null;
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadConfig();
	});

	const servicesCount = $derived(config?.tcp_services.length ?? 0);
	const tlsCount = $derived(config?.tls.length ?? 0);
	const bindsCount = $derived(
		config?.tcp_services.reduce((sum, svc) => sum + svc.binds.length, 0) ?? 0
	);
</script>

<div class="container mx-auto p-6 space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
		<div>
			<h1 class="h1">Configuration Overview</h1>
			{#if lastUpdated}
				<p class="text-sm text-surface-500 dark:text-surface-400 mt-1">
					Last updated: {lastUpdated.toLocaleTimeString()}
				</p>
			{/if}
		</div>
		<button
			class="btn preset-tonal-primary btn-sm gap-2"
			onclick={loadConfig}
			disabled={loading}
		>
			<RefreshCw class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
			Refresh
		</button>
	</div>

	{#if loading && !config}
		<!-- Initial loading state -->
		<div class="flex items-center justify-center py-12">
			<div class="flex items-center gap-3">
				<Loader2 class="w-6 h-6 animate-spin text-primary-500" />
				<span class="text-lg text-surface-600 dark:text-surface-400">Loading configuration...</span>
			</div>
		</div>
	{:else if error}
		<!-- Error state -->
		<div class="alert preset-filled-error-container">
			<AlertCircle class="w-5 h-5" />
			<div>
				<h3 class="font-semibold">Failed to load configuration</h3>
				<p class="text-sm mt-1">{error}</p>
			</div>
		</div>
	{:else if !config}
		<!-- Empty state -->
		<div class="card p-8 text-center">
			<div class="flex flex-col items-center gap-4">
				<AlertCircle class="w-12 h-12 text-surface-400" />
				<div>
					<h3 class="h3 text-surface-700 dark:text-surface-300">No configuration loaded</h3>
					<p class="text-surface-500 dark:text-surface-400 mt-2">
						Start the controller with a configuration file to view it here.
					</p>
				</div>
			</div>
		</div>
	{:else}
		<!-- Statistics cards -->
		<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
			<div class="card p-4 flex items-center gap-3">
				<Server class="w-8 h-8 text-primary-500" />
				<div>
					<div class="text-2xl font-bold">{servicesCount}</div>
					<div class="text-sm text-surface-600 dark:text-surface-400">TCP Services</div>
				</div>
			</div>
			<div class="card p-4 flex items-center gap-3">
				<Server class="w-8 h-8 text-secondary-500" />
				<div>
					<div class="text-2xl font-bold">{bindsCount}</div>
					<div class="text-sm text-surface-600 dark:text-surface-400">Bindings</div>
				</div>
			</div>
			<div class="card p-4 flex items-center gap-3">
				<Lock class="w-8 h-8 text-tertiary-500" />
				<div>
					<div class="text-2xl font-bold">{tlsCount}</div>
					<div class="text-sm text-surface-600 dark:text-surface-400">TLS Configs</div>
				</div>
			</div>
		</div>

		<!-- Main content grid -->
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<!-- TCP Services column -->
			<div class="space-y-4">
				<h2 class="h2 flex items-center gap-2">
					<Server class="w-6 h-6" />
					TCP Services
					{#if servicesCount > 0}
						<span class="text-sm font-normal text-surface-500 dark:text-surface-400">
							({servicesCount})
						</span>
					{/if}
				</h2>
				{#if config.tcp_services.length > 0}
					{#each config.tcp_services as service}
						<TcpServiceCard {service} />
					{/each}
				{:else}
					<div class="card p-6 text-center text-surface-500 dark:text-surface-400">
						No TCP services configured
					</div>
				{/if}
			</div>

			<!-- TLS Configurations column -->
			<div class="space-y-4">
				<h2 class="h2 flex items-center gap-2">
					<Lock class="w-6 h-6" />
					TLS Configurations
					{#if tlsCount > 0}
						<span class="text-sm font-normal text-surface-500 dark:text-surface-400">
							({tlsCount})
						</span>
					{/if}
				</h2>
				{#if config.tls.length > 0}
					{#each config.tls as tls}
						<TlsConfigCard {tls} />
					{/each}
				{:else}
					<div class="card p-6 text-center text-surface-500 dark:text-surface-400">
						No TLS configurations
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
