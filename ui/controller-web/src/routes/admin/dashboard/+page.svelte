<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import type { HumanReadableServiceConfig, KernelConnectionAndState } from '$lib/api/types';
	import type { KernelSummary } from '$lib/api/routes/kernel_manager';
	import {
		RefreshCw,
		ServerIcon,
		Lock,
		Loader2Icon,
		FileCogIcon,
		AlertCircleIcon
	} from '@lucide/svelte';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { api } from '$lib/api/routes';
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import ServiceConfigPreview from '$lib/components/service-config-preview.svelte';

	type DashboardTab = '#instances' | '#config';
	let activeTab: DashboardTab = $state.raw('#instances');

	let config = $state<HumanReadableServiceConfig | null>(null);
	let instances = $state<KernelSummary>({});

	let configLoading = $state(true);
	let instancesLoading = $state(true);

	let configError = $state<string | null>(null);
	let instancesError = $state<string | null>(null);

	let lastUpdated = $state<Date | null>(null);
	let deploySuccessMessage = $state<string | null>(null);

	const instancesCount = $derived(Object.keys(instances).length);

	const refreshing = $derived(configLoading || instancesLoading);
	let currentHashTag = $state('#instances');
	function syncHashTag() {
		currentHashTag = (window.location.hash || '#instances') as DashboardTab;
	}
	function normalizeError(error: unknown, fallback: string): string {
		if (error instanceof Error && error.message.trim().length > 0) {
			return error.message;
		}
		return fallback;
	}

	function toRows(): [string, KernelConnectionAndState][] {
		return Object.entries(instances).sort(([a], [b]) => a.localeCompare(b));
	}

	function getKernelName(kernel: KernelConnectionAndState): string {
		if (kernel.connection === 'Disconnected') {
			return '-';
		}

		return kernel.state.info.name;
	}

	function getKernelId(kernel: KernelConnectionAndState): string {
		if (kernel.connection === 'Disconnected') {
			return '-';
		}

		return kernel.state.info.id;
	}

	function getKernelStateLabel(kernel: KernelConnectionAndState): string {
		if (kernel.connection === 'Disconnected') {
			return 'Disconnected';
		}

		return kernel.state.state.kind;
	}

	function getKernelStateClass(kernel: KernelConnectionAndState): string {
		if (kernel.connection === 'Disconnected') {
			return 'preset-tonal-surface';
		}

		switch (kernel.state.state.kind) {
			case 'Running':
				return 'preset-tonal-success';
			case 'Updating':
			case 'Committing':
			case 'Preparing':
			case 'Prepared':
				return 'preset-tonal-warning';
			case 'WaitingConfig':
				return 'preset-tonal-surface';
			case 'ShuttingDown':
			case 'Stopped':
			default:
				return 'preset-tonal-error';
		}
	}

	function getKernelConfigVersion(kernel: KernelConnectionAndState): string {
		if (kernel.connection === 'Disconnected') {
			return '-';
		}

		const inner = kernel.state.state;
		switch (inner.kind) {
			case 'Running':
				return inner.data.config_version;
			case 'Updating':
				return inner.data.new_config_version;
			default:
				return '-';
		}
	}

	function formatSince(kernel: KernelConnectionAndState): string {
		if (kernel.connection === 'Disconnected') {
			return '-';
		}

		return new Date(kernel.state.state.since).toLocaleString();
	}

	function handleMainTabChange(value: string) {
		activeTab = value as DashboardTab;
	}

	async function loadInstances() {
		instancesLoading = true;
		instancesError = null;
		try {
			instances = await api.kernelManager.listKernels();
		} catch (error) {
			instancesError = normalizeError(error, 'Failed to load instances.');
			instances = {};
		} finally {
			instancesLoading = false;
		}
	}

	async function loadConfig() {
		configLoading = true;
		configError = null;
		try {
			config = await api.config.getCurrentConfig();
		} catch (error) {
			configError = normalizeError(error, 'Failed to load current configuration.');
			config = null;
		} finally {
			configLoading = false;
		}
	}

	async function refreshAll() {
		await Promise.all([loadInstances(), loadConfig()]);
		lastUpdated = new Date();
	}

	async function handleRefresh() {
		deploySuccessMessage = null;
		await refreshAll();
	}

	async function handleDeployedQueryState() {
		if ($page.url.searchParams.get('deployed') !== '1') {
			return;
		}

		const tx = $page.url.searchParams.get('tx');
		deploySuccessMessage = tx ? `Deploy succeeded. Transaction: ${tx}` : 'Deploy succeeded.';
		const url = new URL($page.url);
		url.searchParams.delete('deployed');
		url.searchParams.delete('tx');
		await goto(url.toString(), { replaceState: true, noScroll: true });
	}

	onMount(() => {
		syncHashTag();
		window.addEventListener('hashchange', syncHashTag);
		refreshAll();
		handleDeployedQueryState();
		return () => {
			window.removeEventListener('hashchange', syncHashTag);
		};
	});
</script>

<div class="space-y-4 p-4">
	<div class="flex items-center justify-between gap-3">
		<div>
			<h2 class="h2">Dashboard</h2>
			{#if lastUpdated}
				<p class="text-xs opacity-70">Last updated: {lastUpdated.toLocaleString()}</p>
			{/if}
		</div>
		<button class="btn preset-tonal-primary" onclick={handleRefresh} disabled={refreshing}>
			<RefreshCw class={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
			{refreshing ? 'Refreshing...' : 'Refresh'}
		</button>
	</div>

	{#if deploySuccessMessage}
		<div class="alert preset-tonal-success">{deploySuccessMessage}</div>
	{/if}

	<Tabs
		value={currentHashTag}
		navigate={(details) => {
			let pageUrl = `/admin/dashboard${details.value}` as `/admin/dashboard#${string}`;
			goto(resolve(pageUrl));
		}}
	>
		<Tabs.List class="mb-4">
			<Tabs.Trigger value="#instances" class="flex items-center gap-2">
				{#snippet element(attributes)}
					<a {...attributes as HTMLAnchorAttributes} href="#instances">
						<ServerIcon class="h-4 w-4" />
						<span>
							Instances ({instancesCount})
						</span>
					</a>
				{/snippet}
			</Tabs.Trigger>
			<Tabs.Indicator />
			<Tabs.Trigger value="#config" class="flex items-center gap-2">
				{#snippet element(attributes)}
					<a {...attributes as HTMLAnchorAttributes} href="#config">
						<FileCogIcon class="h-4 w-4" />
						<span> Service Config </span>
					</a>
				{/snippet}
				<FileCogIcon class="h-4 w-4" />
			</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>

		<Tabs.Content value="#instances" class="space-y-3">
			{#if instancesError}
				<div class="alert preset-tonal-error">
					<AlertCircleIcon class="h-4 w-4" />
					{instancesError}
				</div>
			{:else if instancesLoading}
				<div class="flex items-center justify-center gap-2 card p-8 text-sm opacity-70">
					<Loader2Icon class="h-4 w-4 animate-spin" />
					Loading instances...
				</div>
			{:else if toRows().length === 0}
				<div class="card p-8 text-center text-sm opacity-70">No instances found.</div>
			{:else}
				<div class="overflow-x-auto card">
					<table class="table">
						<thead>
							<tr>
								<th>Address</th>
								<th>Connection</th>
								<th>State</th>
								<th>Name</th>
								<th>ID</th>
								<th>Config Version</th>
								<th>Since</th>
							</tr>
						</thead>
						<tbody>
							{#each toRows() as [address, kernel] (address)}
								<tr>
									<td><code class="text-xs code">{address}</code></td>
									<td>
										<span
											class={`badge ${kernel.connection === 'Connected' ? 'preset-tonal-success' : 'preset-tonal-error'}`}
										>
											{kernel.connection}
										</span>
									</td>
									<td
										><span class={`badge ${getKernelStateClass(kernel)}`}
											>{getKernelStateLabel(kernel)}</span
										></td
									>
									<td>{getKernelName(kernel)}</td>
									<td><code class="text-xs code">{getKernelId(kernel)}</code></td>
									<td><code class="text-xs code">{getKernelConfigVersion(kernel)}</code></td>
									<td>{formatSince(kernel)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</Tabs.Content>

		<Tabs.Content value="#config" class="space-y-3">
			{#if configError}
				<div class="alert preset-tonal-error">
					<AlertCircleIcon class="h-4 w-4" />
					{configError}
				</div>
			{:else if configLoading}
				<div class="flex items-center justify-center gap-2 card p-8 text-sm opacity-70">
					<Loader2Icon class="h-4 w-4 animate-spin" />
					Loading current configuration...
				</div>
			{:else if !config}
				<div class="card p-8 text-center text-sm opacity-70">No current configuration loaded.</div>
			{:else}
				<ServiceConfigPreview {config} />
			{/if}
		</Tabs.Content>
	</Tabs>
</div>
