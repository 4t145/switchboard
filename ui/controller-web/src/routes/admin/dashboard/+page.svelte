<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import type { HumanReadableServiceConfig } from '$lib/api/types/human_readable';
	import { getCurrentConfig } from '$lib/api/config';
	import { RefreshCw, AlertCircle, Server, Lock, Loader2, Network } from '@lucide/svelte';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import ServicesTab from '$lib/components/config/tabs/services-tab.svelte';
	import BindingsTab from '$lib/components/config/tabs/bindings-tab.svelte';
	import TlsTab from '$lib/components/config/tabs/tls-tab.svelte';

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

	// Read URL parameters for navigation state
	let currentTab = $derived($page.url.searchParams.get('tab') || 'services');
	let highlightService = $derived($page.url.searchParams.get('service') || null);
	let highlightTls = $derived($page.url.searchParams.get('tls') || null);

	// Active tab state - sync with URL (use derived to avoid warning)
	let activeTab = $derived(currentTab);

	// Jump target state for cross-tab navigation (kept for backwards compatibility)
	let jumpTarget = $state<{ type: 'service' | 'tls' | null; id: string | null }>({
		type: null,
		id: null
	});

	// Update URL when tab changes
	function handleTabChange(newTab: string) {
		const url = new URL($page.url);
		url.searchParams.set('tab', newTab);
		// Clear highlight params when switching tabs manually
		url.searchParams.delete('service');
		url.searchParams.delete('tls');
		goto(url.toString(), { replaceState: false, noScroll: false });
	}

	// Jump functions - now update URL instead of internal state
	function jumpToService(serviceName: string) {
		const url = new URL($page.url);
		url.searchParams.set('tab', 'services');
		url.searchParams.set('service', serviceName);
		url.searchParams.delete('tls');
		goto(url.toString(), { replaceState: false, noScroll: false });
	}

	function jumpToTls(tlsName: string) {
		const url = new URL($page.url);
		url.searchParams.set('tab', 'tls');
		url.searchParams.set('tls', tlsName);
		url.searchParams.delete('service');
		goto(url.toString(), { replaceState: false, noScroll: false });
	}
</script>

<div class="container mx-auto space-y-6 p-6">
	<!-- Header -->
	<div class="flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
		<div>
			<h1 class="h1">Configuration Dashboard</h1>
			{#if lastUpdated}
				<p class="mt-1 text-sm text-surface-500 dark:text-surface-400">
					Last updated: {lastUpdated.toLocaleTimeString()}
				</p>
			{/if}
		</div>
		<button class="btn gap-2 preset-tonal-primary btn-sm" onclick={loadConfig} disabled={loading}>
			<RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
			Refresh
		</button>
	</div>

	{#if loading && !config}
		<!-- Initial loading state -->
		<div class="flex items-center justify-center py-12">
			<div class="flex items-center gap-3">
				<Loader2 class="h-6 w-6 animate-spin text-primary-500" />
				<span class="text-lg text-surface-600 dark:text-surface-400">Loading configuration...</span>
			</div>
		</div>
	{:else if error}
		<!-- Error state -->
		<div class="alert preset-filled-error-container">
			<AlertCircle class="h-5 w-5" />
			<div>
				<h3 class="font-semibold">Failed to load configuration</h3>
				<p class="mt-1 text-sm">{error}</p>
			</div>
		</div>
	{:else if !config}
		<!-- Empty state -->
		<div class="card p-8 text-center">
			<div class="flex flex-col items-center gap-4">
				<AlertCircle class="h-12 w-12 text-surface-400" />
				<div>
					<h3 class="h3 text-surface-700 dark:text-surface-300">No configuration loaded</h3>
					<p class="mt-2 text-surface-500 dark:text-surface-400">
						Start the controller with a configuration file to view it here.
					</p>
				</div>
			</div>
		</div>
	{:else}
		<!-- Statistics cards -->
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
			<div class="flex items-center gap-3 card p-4">
				<Server class="h-8 w-8 text-primary-500" />
				<div>
					<div class="text-2xl font-bold">{servicesCount}</div>
					<div class="text-sm text-surface-600 dark:text-surface-400">TCP Services</div>
				</div>
			</div>
			<div class="flex items-center gap-3 card p-4">
				<Network class="h-8 w-8 text-secondary-500" />
				<div>
					<div class="text-2xl font-bold">{bindsCount}</div>
					<div class="text-sm text-surface-600 dark:text-surface-400">Bindings</div>
				</div>
			</div>
			<div class="flex items-center gap-3 card p-4">
				<Lock class="h-8 w-8 text-tertiary-500" />
				<div>
					<div class="text-2xl font-bold">{tlsCount}</div>
					<div class="text-sm text-surface-600 dark:text-surface-400">TLS Configs</div>
				</div>
			</div>
		</div>

		<!-- Tabs navigation -->
		<Tabs value={activeTab} onValueChange={(details) => handleTabChange(details.value)}>
			<Tabs.List class="mb-6">
				<Tabs.Trigger value="services" class="flex items-center gap-2">
					<Server class="h-4 w-4" />
					Services ({servicesCount})
				</Tabs.Trigger>
				<Tabs.Trigger value="bindings" class="flex items-center gap-2">
					<Network class="h-4 w-4" />
					Bindings ({bindsCount})
				</Tabs.Trigger>
				<Tabs.Trigger value="tls" class="flex items-center gap-2">
					<Lock class="h-4 w-4" />
					TLS ({tlsCount})
				</Tabs.Trigger>
				<Tabs.Indicator />
			</Tabs.List>

			<Tabs.Content value="services">
				<ServicesTab services={config.tcp_services} {highlightService} {jumpTarget} {jumpToTls} />
			</Tabs.Content>

			<Tabs.Content value="bindings">
				<BindingsTab services={config.tcp_services} {jumpToService} {jumpToTls} />
			</Tabs.Content>

			<Tabs.Content value="tls">
				<TlsTab
					tlsConfigs={config.tls}
					services={config.tcp_services}
					{highlightTls}
					{jumpTarget}
					{jumpToService}
				/>
			</Tabs.Content>
		</Tabs>
	{/if}
</div>
