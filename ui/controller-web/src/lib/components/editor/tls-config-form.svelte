<script lang="ts">
	import LinkOrValueEditor from './link-or-value-editor.svelte';
	import TlsResolverForm from './tls-resolver-form.svelte';
	import { TagsInput, Collapsible } from '@skeletonlabs/skeleton-svelte';
	import { ChevronDown } from 'lucide-svelte';
	import type { FileStyleTls, FileStyleTlsResolver, TlsCertParams } from '$lib/api/types';
	export type Props= {
		value: FileStyleTls;
	};
	let { value = $bindable() }: Props = $props<{
		value: FileStyleTls;
	}>();

	// Default TlsOptions - ensure value.options exists
	// Using $effect.pre to ensure this runs before rendering if possible, or just standard effect
	$effect(() => {
		if (!value) return; // Safety check
		
		if (!value.options) {
			value.options = {
				ignore_client_order: false,
				max_fragment_size: null,
				alpn_protocols: ['h2', 'http/1.1'],
				enable_secret_extraction: false,
				max_early_data_size: 0,
				send_half_rtt_data: false,
				send_tls13_tickets: 0,
				require_ems: true
			} as any;
		}
	});

	// Collapsible states
	let resolverOpen = $state(true);
	let optionsOpen = $state(false);
</script>

<div class="flex h-full flex-col space-y-3 p-3">
	<!-- Name Field -->
	<label class="label">
		<span class="label-text text-sm font-bold">Name (Unique ID)</span>
		<input class="input w-full" type="text" bind:value={value.name} placeholder="tls-1" />
	</label>

	<!-- Certificate Resolver Section -->
	<Collapsible 
		class="card border border-surface-200 dark:border-surface-700" 
		open={resolverOpen}
		onOpenChange={(details) => (resolverOpen = details.open)}
	>
		<Collapsible.Trigger class="flex w-full items-center justify-between border-b border-surface-200 px-4 py-3 hover:preset-tonal dark:border-surface-700 cursor-pointer">
			<h3 class="h3 font-bold">Certificate Resolver</h3>
			<Collapsible.Indicator>
				<ChevronDown size={18} class="transition-transform duration-200 [[data-state=open]_&]:rotate-180" />
			</Collapsible.Indicator>
		</Collapsible.Trigger>
		<Collapsible.Content class="p-3 w-full">
			<TlsResolverForm bind:value={value} />
		</Collapsible.Content>
	</Collapsible>

	<!-- TLS Options Section -->
	<Collapsible 
		class="card border border-surface-200 dark:border-surface-700"
		open={optionsOpen}
		onOpenChange={(details) => (optionsOpen = details.open)}
	>
		<Collapsible.Trigger class="flex w-full items-center justify-between border-b border-surface-200 px-4 py-3 hover:preset-tonal dark:border-surface-700 cursor-pointer">
			<h3 class="h3 font-bold">TLS Options</h3>
			<Collapsible.Indicator>
				<ChevronDown size={18} class="transition-transform duration-200 [[data-state=open]_&]:rotate-180" />
			</Collapsible.Indicator>
		</Collapsible.Trigger>
		<Collapsible.Content class="p-3 w-full">
			<div class="grid grid-cols-1 gap-3 md:grid-cols-2">
				{#if value.options}
					<label class="flex items-center space-x-2">
						<input class="checkbox" type="checkbox" bind:checked={value.options.ignore_client_order} />
						<span>Ignore Client Order</span>
					</label>
					<label class="flex items-center space-x-2">
						<input class="checkbox" type="checkbox" bind:checked={value.options.require_ems} />
						<span>Require EMS</span>
					</label>
					<label class="flex items-center space-x-2">
						<input
							class="checkbox"
							type="checkbox"
							bind:checked={value.options.enable_secret_extraction}
						/>
						<span>Enable Secret Extraction</span>
					</label>
					<label class="label">
						<span>Max Early Data Size</span>
						<input class="input w-full" type="number" bind:value={value.options.max_early_data_size} />
					</label>
					<div class="col-span-full">
						{#if value.options}
							<TagsInput 
								value={value.options.alpn_protocols}
								onValueChange={(details) => {
									if (value.options) {
										value.options.alpn_protocols = details.value;
									}
								}}
							>
								<TagsInput.Label>ALPN Protocols</TagsInput.Label>
								<TagsInput.Control>
									<TagsInput.Context>
										{#snippet children(tagsInput)}
											{#each tagsInput().value as protocol, index (index)}
												<TagsInput.Item value={protocol} {index}>
													<TagsInput.ItemPreview>
														<TagsInput.ItemText>{protocol}</TagsInput.ItemText>
														<TagsInput.ItemDeleteTrigger />
													</TagsInput.ItemPreview>
													<TagsInput.ItemInput />
												</TagsInput.Item>
											{/each}
										{/snippet}
									</TagsInput.Context>
									<TagsInput.Input placeholder="Add protocol (e.g. h2, http/1.1)..." />
								</TagsInput.Control>
								<TagsInput.HiddenInput />
							</TagsInput>
						{/if}
					</div>
				{/if}
			</div>
		</Collapsible.Content>
	</Collapsible>
</div>
