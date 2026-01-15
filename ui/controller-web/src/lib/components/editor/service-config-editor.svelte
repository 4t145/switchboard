<script lang="ts">
	import { Trash2, Plus, Edit } from 'lucide-svelte';
	import type {
		HumanReadableServiceConfig,
		FileTcpServiceConfig,
		FileStyleTls,
		FileBind
	} from '$lib/api/types';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import TcpServiceForm from './tcp-service-form.svelte';
	import TlsConfigForm from './tls-config-form.svelte';
	// Removed ListenerForm and TcpRouteForm imports as they are merged into service config

	let { config = $bindable() } = $props<{ config: HumanReadableServiceConfig }>();

	let activeTab = $state('services');
	let editingItem: { type: string; index: number } | null = $state(null);

	// Initializers for new items
	const initializers = {
		services: (): FileTcpServiceConfig => ({
			provider: 'Static',
			name: '',
			config: {},
			description: '',
			binds: []
		}),
		tls: (): FileStyleTls => ({
			name: '',
			Single: { certs: [], key: '', ocsp: null },
			options: {
				ignore_client_order: false,
				max_fragment_size: null,
				alpn_protocols: [],
				enable_secret_extraction: false,
				max_early_data_size: 0,
				send_half_rtt_data: false,
				send_tls13_tickets: 0,
				require_ems: false
			} as any
		})
	};

	function addItem(type: 'services' | 'tls') {
		const newItem = initializers[type]();
		const collection = type === 'services' ? config.tcp_services : config.tls;

		// Auto-generate name/ID
		const prefix = type === 'services' ? 'service' : 'tls';
		let counter = collection.length + 1;
		let name = `${prefix}-${counter}`;

		// Simple unique name check
		while (collection.some((item: any) => item.name === name)) {
			counter++;
			name = `${prefix}-${counter}`;
		}
		newItem.name = name;

		// Add to collection
		if (type === 'services') {
			config.tcp_services = [...config.tcp_services, newItem as FileTcpServiceConfig];
		} else {
			config.tls = [...config.tls, newItem as FileStyleTls];
		}

		editingItem = { type, index: config[type === 'services' ? 'tcp_services' : 'tls'].length - 1 };
	}

	function deleteItem(type: 'services' | 'tls', index: number) {
		if (type === 'services') {
			config.tcp_services = config.tcp_services.filter((_: unknown, i: number) => i !== index);
		} else {
			config.tls = config.tls.filter((_: unknown, i: number) => i !== index);
		}

		if (editingItem?.type === type && editingItem?.index === index) {
			editingItem = null;
		} else if (editingItem?.type === type && editingItem.index > index) {
			editingItem.index--;
		}
	}

	// Helper to get collection for template
	function getCollection(type: string) {
		if (type === 'services') return config.tcp_services;
		if (type === 'tls') return config.tls;
		return [];
	}

	// Helper to get name of item
	function getItemName(item: any) {
		return item.name || 'Unnamed';
	}
</script>

<div class="flex h-full flex-col p-4">
	<!-- Tabs Header -->
	<Tabs
		value={activeTab}
		onValueChange={(e) => {
			activeTab = e.value;
			editingItem = null;
		}}
	>
		<Tabs.List class="mb-4">
			<Tabs.Trigger value="services">Services</Tabs.Trigger>
			<Tabs.Trigger value="tls">TLS</Tabs.Trigger>
		</Tabs.List>

		<!-- Tab Content Wrapper -->
		<div class="flex h-[calc(100vh-12rem)] gap-6">
			<!-- Sidebar: List -->
			<div
				class="bg-surface-100-800-token border-surface-200-700-token flex w-80 flex-none flex-col card border shadow-sm"
			>
				<div
					class="border-surface-200-700-token flex flex-none items-center justify-between border-b p-4"
				>
					<h3 class="text-lg font-bold capitalize">{activeTab}</h3>
					<button
						class="variant-filled-primary btn btn-sm"
						onclick={() => addItem(activeTab as any)}
					>
						<Plus size={16} /> Add
					</button>
				</div>
				<div class="flex-1 space-y-2 overflow-y-auto p-2">
					{#each getCollection(activeTab) as item, index}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="rounded-container-token hover:bg-surface-200-700-token flex cursor-pointer items-center justify-between border border-transparent p-3 transition-colors {editingItem?.index ===
								index && editingItem?.type === activeTab
								? 'variant-soft-primary'
								: ''}"
							onclick={() => (editingItem = { type: activeTab, index })}
						>
							<span class="flex-1 truncate font-medium">{getItemName(item)}</span>
							<div class="flex gap-1">
								<button
									class="btn-icon btn-icon-sm"
									onclick={(e) => {
										e.stopPropagation();
										deleteItem(activeTab as any, index);
									}}
								>
									<Trash2 size={14} class="text-error-500" />
								</button>
							</div>
						</div>
					{/each}
					{#if getCollection(activeTab).length === 0}
						<div class="p-8 text-center text-sm opacity-50">
							No items found.<br />Click "Add" to create one.
						</div>
					{/if}
				</div>
			</div>

			<!-- Editor: Detail -->
			<div
				class="bg-surface-100-800-token border-surface-200-700-token flex flex-1 flex-col overflow-hidden card border shadow-sm"
			>
				{#if editingItem && editingItem.type === activeTab}
					<div
						class="border-surface-200-700-token bg-surface-200-700-token/50 flex flex-none items-center justify-between border-b p-4"
					>
						<div class="flex items-center gap-2">
							<Edit size={16} class="opacity-70" />
							<span class="font-bold"
								>Editing: {getItemName(getCollection(activeTab)[editingItem.index])}</span
							>
						</div>
					</div>

					<div class="flex-1 overflow-y-auto">
						{#if activeTab === 'services'}
							<TcpServiceForm
								bind:value={config.tcp_services[editingItem.index]}
								tlsKeys={config.tls.map((t: any) => t.name)}
							/>
						{:else if activeTab === 'tls'}
							<TlsConfigForm bind:value={config.tls[editingItem.index]} />
						{/if}
					</div>
				{:else}
					<div class="flex h-full flex-col items-center justify-center opacity-40">
						<Edit size={48} class="mb-4" />
						<p class="text-lg">Select an item to edit details</p>
					</div>
				{/if}
			</div>
		</div>
	</Tabs>
</div>
