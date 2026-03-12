<script lang="ts">
	import type {
		HumanReadableServiceConfig,
		FileTcpServiceConfig,
		FileStyleTls
	} from '$lib/api/types';
	import { Listbox, Tabs, useListCollection } from '@skeletonlabs/skeleton-svelte';
	import TcpServiceForm from './tcp-service-form.svelte';
	import TlsConfigForm from './tls-config-form.svelte';
	import { PlusIcon, Trash2Icon } from '@lucide/svelte';

	let {
		config = $bindable(),
		readonly = false
	}: { config: HumanReadableServiceConfig; readonly?: boolean } = $props();

	let activeTab = $state<'services' | 'tls'>('services');
	let selectedService = $state<FileTcpServiceConfig | null>(null);
	let selectedTls = $state<FileStyleTls | null>(null);
	let serviceQuery = $state('');
	let tlsQuery = $state('');
	let availableTlsKeys = $derived.by(() =>
		config.tls.filter((t) => t && t.name).map((t: FileStyleTls) => t.name)
	);
	type ServiceListItem = {
		id: string;
		label: string;
		service: FileTcpServiceConfig;
	};

	type TlsListItem = {
		id: string;
		label: string;
		tls: FileStyleTls;
	};

	const serviceItems = $derived.by<ServiceListItem[]>(() =>
		config.tcp_services.map((service, index) => ({
			id: `service-${index}`,
			label: service.name || `Unnamed Service ${index + 1}`,
			service
		}))
	);

	const filteredServiceItems = $derived.by<ServiceListItem[]>(() => {
		const query = serviceQuery.trim().toLowerCase();
		if (query === '') {
			return serviceItems;
		}

		return serviceItems.filter((item) => item.label.toLowerCase().includes(query));
	});

	const serviceCollection = $derived.by(() =>
		useListCollection({
			items: filteredServiceItems,
			itemToString: (item) => item.label,
			itemToValue: (item) => item.id
		})
	);

	const selectedServiceValue = $derived.by<string[]>(() => {
		if (selectedService === null) {
			return [];
		}

		const selected = serviceItems.find((item) => item.service === selectedService);
		if (!selected || !filteredServiceItems.some((item) => item.id === selected.id)) {
			return [];
		}

		return [selected.id];
	});

	const tlsItems = $derived.by<TlsListItem[]>(() =>
		config.tls.map((tls, index) => ({
			id: `tls-${index}`,
			label: tls.name || `Unnamed TLS Config ${index + 1}`,
			tls
		}))
	);

	const filteredTlsItems = $derived.by<TlsListItem[]>(() => {
		const query = tlsQuery.trim().toLowerCase();
		if (query === '') {
			return tlsItems;
		}

		return tlsItems.filter((item) => item.label.toLowerCase().includes(query));
	});

	const tlsCollection = $derived.by(() =>
		useListCollection({
			items: filteredTlsItems,
			itemToString: (item) => item.label,
			itemToValue: (item) => item.id
		})
	);

	const selectedTlsValue = $derived.by<string[]>(() => {
		if (selectedTls === null) {
			return [];
		}

		const selected = tlsItems.find((item) => item.tls === selectedTls);
		if (!selected || !filteredTlsItems.some((item) => item.id === selected.id)) {
			return [];
		}

		return [selected.id];
	});
	// Service initializer
	function createService(): FileTcpServiceConfig {
		const counter = config.tcp_services.length + 1;
		let name = `service-${counter}`;

		// Ensure unique name
		let suffix = counter;
		while (config.tcp_services.some((s: FileTcpServiceConfig) => s.name === name)) {
			suffix++;
			name = `service-${suffix}`;
		}

		return {
			provider: 'Static',
			name,
			config: {},
			description: '',
			binds: []
		};
	}
	// let availableTlsKeys = $derived(config.tls.map((t: FileStyleTls) => t.name));
	// TLS initializer
	function createTls(): FileStyleTls {
		const counter = config.tls.length + 1;
		let name = `tls-${counter}`;

		// Ensure unique name
		let suffix = counter;
		while (config.tls.some((t: FileStyleTls) => t.name === name)) {
			suffix++;
			name = `tls-${suffix}`;
		}

		return {
			name,
			certs: '',
			key: '',
			ocsp: null,
			options: {
				ignore_client_order: false,
				max_fragment_size: null,
				alpn_protocols: ['h2', 'http/1.1'],
				enable_secret_extraction: false,
				max_early_data_size: 0,
				send_half_rtt_data: false,
				send_tls13_tickets: 0,
				require_ems: true
			}
		};
	}

	function deleteSelectedService(): void {
		if (readonly) {
			return;
		}
		if (selectedService === null) {
			return;
		}

		const index = config.tcp_services.indexOf(selectedService);
		if (index >= 0) {
			config.tcp_services.splice(index, 1);
		}
		selectedService = null;
	}

	function deleteSelectedTls(): void {
		if (readonly) {
			return;
		}
		if (selectedTls === null) {
			return;
		}

		const index = config.tls.indexOf(selectedTls);
		if (index >= 0) {
			config.tls.splice(index, 1);
		}
		selectedTls = null;
	}

	$effect(() => {
		if (selectedService !== null && !config.tcp_services.includes(selectedService)) {
			selectedService = null;
		}

		if (selectedTls !== null && !config.tls.includes(selectedTls)) {
			selectedTls = null;
		}
	});
</script>

<div class="flex h-full min-h-0 min-w-0 flex-col p-4">
	<Tabs
		value={activeTab}
		onValueChange={(e) => {
			activeTab = e.value as 'services' | 'tls';
		}}
		class="flex h-full min-h-0 min-w-0 flex-col"
	>
		<Tabs.List class="mb-4 flex-none">
			<Tabs.Trigger value="services">Services</Tabs.Trigger>
			<Tabs.Trigger value="tls">TLS</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>

		<Tabs.Content value="services" class="min-h-0 min-w-0 flex-1 overflow-hidden">
			<div class="flex h-full min-h-0 min-w-0 flex-row gap-4">
				<div class="flex w-80 flex-none flex-col gap-2">
					<div class="flex items-center gap-2">
						<input
							type="text"
							class="input flex-1"
							placeholder="Filter services..."
							value={serviceQuery}
							disabled={readonly}
							oninput={(event) => {
								serviceQuery = (event.currentTarget as HTMLInputElement).value;
							}}
						/>
						<button
							type="button"
							class="btn-icon preset-tonal-primary"
							disabled={readonly}
							onclick={() => {
								if (readonly) return;
								const created = createService();
								config.tcp_services.push(created);
								selectedService = created;
							}}
						>
							<PlusIcon />
						</button>
						<button
							type="button"
							class="btn-icon preset-tonal-error"
							onclick={deleteSelectedService}
							disabled={readonly || selectedService === null}
						>
							<Trash2Icon />
						</button>
					</div>
					<Listbox
						class="h-full"
						collection={serviceCollection}
						value={selectedServiceValue}
						onValueChange={(event) => {
							const selectedId = event.value[0];
							if (!selectedId) {
								selectedService = null;
								return;
							}

							const selected = filteredServiceItems.find((item) => item.id === selectedId);
							selectedService = selected ? selected.service : null;
						}}
					>
						<Listbox.Content
							class="rounded-base-container h-full overflow-auto border border-surface-200-800 p-2"
						>
							{#each serviceCollection.items as item (item.id)}
								<Listbox.Item {item}>
									<Listbox.ItemText>{item.label}</Listbox.ItemText>
									<Listbox.ItemIndicator />
								</Listbox.Item>
							{/each}
						</Listbox.Content>
					</Listbox>
				</div>
				<div class="min-h-0 min-w-0 flex-1 overflow-auto">
					{#if selectedService !== null}
						<TcpServiceForm value={selectedService} tlsKeys={availableTlsKeys} {readonly} />
					{:else}
						<div class="text-muted-foreground flex h-full items-center justify-center">
							No service selected
						</div>
					{/if}
				</div>
			</div>
		</Tabs.Content>

		<Tabs.Content value="tls" class="min-h-0 min-w-0 flex-1 overflow-hidden">
			<div class="flex h-full min-h-0 min-w-0 flex-row gap-4">
				<div class="flex w-80 flex-none flex-col gap-2">
					<div class="flex items-center gap-2">
						<input
							type="text"
							class="input flex-1"
							placeholder="Filter TLS configs..."
							value={tlsQuery}
							disabled={readonly}
							oninput={(event) => {
								tlsQuery = (event.currentTarget as HTMLInputElement).value;
							}}
						/>
						<button
							type="button"
							class="btn-icon preset-tonal-primary"
							disabled={readonly}
							onclick={() => {
								if (readonly) return;
								const created = createTls();
								config.tls.push(created);
								selectedTls = created;
							}}
						>
							<PlusIcon />
						</button>
						<button
							type="button"
							class="btn-icon preset-tonal-error"
							onclick={deleteSelectedTls}
							disabled={readonly || selectedTls === null}
						>
							<Trash2Icon />
						</button>
					</div>
					<Listbox
						class="h-full"
						collection={tlsCollection}
						value={selectedTlsValue}
						onValueChange={(event) => {
							const selectedId = event.value[0];
							if (!selectedId) {
								selectedTls = null;
								return;
							}

							const selected = filteredTlsItems.find((item) => item.id === selectedId);
							selectedTls = selected ? selected.tls : null;
						}}
					>
						<Listbox.Content
							class="rounded-base-container h-full overflow-auto border border-surface-200-800 p-2"
						>
							{#each tlsCollection.items as item (item.id)}
								<Listbox.Item {item}>
									<Listbox.ItemText>{item.label}</Listbox.ItemText>
									<Listbox.ItemIndicator />
								</Listbox.Item>
							{/each}
						</Listbox.Content>
					</Listbox>
				</div>
				<div class="min-h-0 min-w-0 flex-1 overflow-auto">
					{#if selectedTls !== null}
						<TlsConfigForm value={selectedTls} {readonly} />
					{:else}
						<div class="text-muted-foreground flex h-full items-center justify-center">
							No TLS config selected
						</div>
					{/if}
				</div>
			</div>
		</Tabs.Content>
	</Tabs>
</div>
