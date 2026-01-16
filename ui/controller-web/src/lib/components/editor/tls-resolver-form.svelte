<script lang="ts">
	import { isSingleTlsResolver, isSniTlsResolver, type FileStyleTls, type FileStyleTlsResolver, type SniFileStyleTlsResolver } from '$lib/api/types';
	import PemInput from '$lib/components/common/pem-input.svelte';
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import LinkOrValueEditor from './link-or-value-editor.svelte';
	import { Trash2, Plus, Shield, Network } from 'lucide-svelte';

	// The types now involve LinkOrValue wrapping the PemFile/PemsFile
	// We simplify for the frontend form, assuming generic params for LinkOrValue
	type TlsCertParams = {
		certs: any; // LinkOrValue<PemsFile>
		key: any; // LinkOrValue<PemFile>
		ocsp: any | null; // Base64 or Link? It's Base64Bytes in Rust struct usually, but let's check definition
	};
	export type Props = {
		value: FileStyleTlsResolver;
	};
	// TlsResolver can be { Single: ... } or { Sni: ... }
	// We bind to the whole object.
	let { value = $bindable() }: Props = $props<{
		value: FileStyleTlsResolver;
	}>();

	// Internal state to track mode
	let mode = $state<'Single' | 'Sni'>('Single');
	let singleModeCache = $state<TlsCertParams>(isSingleTlsResolver(value) ? value : {
		certs: '',
		key: '',
		ocsp: null
	});
	let sniModeCache = $state<SniFileStyleTlsResolver>(isSniTlsResolver(value) ? value : {
		sni: []
	});
	// Initialize and ensure data exists
	$effect(() => {
		const v = value as FileStyleTlsResolver;
		// Determine mode based on data presence
		if (isSniTlsResolver(v)) {
			mode = 'Sni';
		} else {
			// Default to Single
			mode = 'Single';
		}
	});

	function switchMode(newMode: 'Single' | 'Sni') {
		if (mode === newMode) return;
		mode = newMode;

		if (newMode === 'Single' && isSniTlsResolver(value)) {
			// Cache SNI mode data
			sniModeCache = { sni: value.sni };
			
			// Switch to Single mode, preserving name and options
			const { sni, ...rest } = value as any;
			Object.assign(value, rest, singleModeCache);
		} else if (newMode === 'Sni' && isSingleTlsResolver(value)) {
			// Cache Single mode data
			singleModeCache = { certs: value.certs, key: value.key, ocsp: value.ocsp };
			
			// Switch to SNI mode, preserving name and options
			const { certs, key, ocsp, ...rest } = value as any;
			Object.assign(value, rest, sniModeCache);
		}
	}
</script>

{#snippet certForm(params: TlsCertParams)}
	<div
		class="space-y-3 rounded border border-surface-200 bg-surface-50 p-3 dark:border-surface-700 dark:bg-surface-900/50"
	>
		<!-- Key: LinkOrValue<PemFile> -->
		<div class="space-y-1">
			<span class="label-text text-sm font-bold">Private Key</span>
			<LinkOrValueEditor
				bind:value={params.key}
				dataType="PemFile"
				dataFormat="string"
				defaultValue={() => ''}
			>
				{#snippet renderValue()}
					<PemInput
						label="Private Key Content (PEM)"
						bind:value={params.key}
						helperText="Private key file content."
					/>
				{/snippet}
			</LinkOrValueEditor>
		</div>

		<!-- Certs: LinkOrValue<PemsFile> -->
		<div class="space-y-1">
			<span class="label-text text-sm font-bold">Certificate Chain</span>
			<LinkOrValueEditor
				bind:value={params.certs}
				dataType="PemsFile"
				dataFormat="object"
				defaultValue={() => []}
			>
				{#snippet renderValue()}
					<PemInput
						label="Certificate Chain Content (PEM)"
						bind:value={params.certs}
						helperText="Certificate chain (server cert first, then intermediates)."
					/>
				{/snippet}
			</LinkOrValueEditor>
		</div>

		<!-- OCSP (Optional, usually just bytes, keeping commented out as per original) -->
		<!-- <FileToBase64Input 
            label="OCSP Stapling (Optional)" 
            bind:value={params.ocsp as string} 
            accept=".der,.ocsp"
            helperText="Upload OCSP response if needed."
        /> -->
	</div>
{/snippet}

<div class="flex flex-col space-y-3">
	<!-- Mode Switcher -->
	<SegmentedControl value={mode} onValueChange={(details) => {( details.value === "Sni" || details.value === "Single") ? switchMode(details.value) : null }} class="w-fit">
		<SegmentedControl.Label>Mode</SegmentedControl.Label>
		<SegmentedControl.Control>
			<SegmentedControl.Indicator />
			<SegmentedControl.Item value="Single">
				<SegmentedControl.ItemText>
					<Shield size={16} class="mr-1 inline-block" />
					Single Certificate
				</SegmentedControl.ItemText>
				<SegmentedControl.ItemHiddenInput />
			</SegmentedControl.Item>
			<SegmentedControl.Item value="Sni">
				<SegmentedControl.ItemText>
					<Network size={16} class="mr-1 inline-block" />
					SNI Mapping
				</SegmentedControl.ItemText>
				<SegmentedControl.ItemHiddenInput />
			</SegmentedControl.Item>
		</SegmentedControl.Control>
	</SegmentedControl>
	{#if mode === 'Single' && isSingleTlsResolver(value)}
		{@render certForm(value)}
	{:else if mode === 'Sni' && isSniTlsResolver(value)}
		<!-- Sni Editor -->
		{#each value.sni as { hostname, ...params }}
			<div class="card border border-surface-300 p-3 dark:border-surface-600">
				<div class="mb-3 flex items-center justify-between">
					<h4 class="text-base font-bold">{hostname}</h4>
					<button
						class="btn btn-sm preset-filled-error-500"
						onclick={() => {
							if (isSniTlsResolver(value)) {
								value.sni = value.sni.filter((entry) => entry.hostname !== hostname);
							}
						}}>Delete</button
					>
				</div>
				{@render certForm(params as TlsCertParams)}
			</div>
		{/each}
		<!-- Add Domain -->
		<div class="flex items-end gap-2">
			<label class="label flex-1">
				<span class="text-sm">New Domain</span>
				<input class="input" type="text" placeholder="example.com" id="new-sni-domain" />
			</label>
			<button
				class="btn preset-filled-primary-500"
				onclick={() => {
					const input = document.getElementById('new-sni-domain') as HTMLInputElement;
					const domain = input.value;
					if (domain && isSniTlsResolver(value)) {
						value.sni = [
							...value.sni,
							{
								hostname: domain,
								certs: '',
								key: '',
								ocsp: null
							}
						];
						input.value = '';
					}
				}}>
				<Plus size={16} class="mr-1 inline-block" />
				Add
				</button
			>
		</div>
	{/if}
</div>
