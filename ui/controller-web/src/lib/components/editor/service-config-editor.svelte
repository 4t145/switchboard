<script lang="ts">
	import type {
		HumanReadableServiceConfig,
		FileTcpServiceConfig,
		FileStyleTls
	} from '$lib/api/types';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import TcpServiceForm from './tcp-service-form.svelte';
	import TlsConfigForm from './tls-config-form.svelte';
	import ListEditor from '../common/list-editor.svelte';
	import { PlusIcon } from '@lucide/svelte';

	let { config = $bindable() }: { config: HumanReadableServiceConfig } = $props();

	let activeTab = $state<'services' | 'tls'>('services');

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

	// Item name getters
	function getServiceName(item: FileTcpServiceConfig): string {
		return item.name || 'Unnamed Service';
	}

	function getTlsName(item: FileStyleTls): string {
		return item.name || 'Unnamed TLS Config';
	}
</script>

<div class="flex h-full flex-col p-4">
	<Tabs
		value={activeTab}
		onValueChange={(e) => {
			activeTab = e.value as 'services' | 'tls';
		}}
		class="flex h-full flex-col"
	>
		<Tabs.List class="mb-4 flex-none">
			<Tabs.Trigger value="services">Services</Tabs.Trigger>
			<Tabs.Trigger value="tls">TLS</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>

		<Tabs.Content value="services" class="flex-1 overflow-hidden">
			<div class="h-full">
				<ListEditor value={config.tcp_services}>
					{#snippet control(api)}
						<div>
							<input
								type="text"
								class="btn"
								onchange={(evt) => {
									const value = evt.currentTarget.value;
									if (value && value.length > 0) {
										api.setFilter((item) =>
											item.name.toLowerCase().includes(evt.currentTarget.value.toLowerCase())
										);
									} else {
										api.setFilter(null);
									}
								}}
								placeholder="Filter services..."
							/>
							<button type="button" class="btn"><PlusIcon /></button>
						</div>
					{/snippet}
					{#snippet item(itemApi)}
						<button
							onclick={() => {
								itemApi.selected = !itemApi.selected;
							}}
							class={`btn ${itemApi.selected ? 'preset-filled-surface-200-800 ' : 'preset-outlined-surface-200-800'}`}
						>
							{itemApi.value.name}
						</button>
					{/snippet}
				</ListEditor>
			</div>
		</Tabs.Content>

		<Tabs.Content value="tls" class="flex-1 overflow-hidden">
			<div class="h-full"></div>
		</Tabs.Content>
	</Tabs>
</div>
