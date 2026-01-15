<script lang="ts">
	import PemInput from '$lib/components/common/pem-input.svelte';
	import LinkOrValueEditor from './link-or-value-editor.svelte';
	import { Trash2, Plus } from 'lucide-svelte';

	// The types now involve LinkOrValue wrapping the PemFile/PemsFile
	// We simplify for the frontend form, assuming generic params for LinkOrValue
	type TlsCertParams = {
		certs: any; // LinkOrValue<PemsFile>
		key: any; // LinkOrValue<PemFile>
		ocsp: any | null; // Base64 or Link? It's Base64Bytes in Rust struct usually, but let's check definition
	};

	// TlsResolver can be { Single: ... } or { Sni: ... }
	// We bind to the whole object.
	let { value = $bindable() } = $props<{
		value: { Single?: TlsCertParams; Sni?: Record<string, TlsCertParams> };
	}>();

	// Internal state to track mode
	let mode = $state<'Single' | 'Sni'>('Single');

	// Initialize and ensure data exists
	$effect(() => {
		const v = value as any;
		// Determine mode based on data presence
		if (v.Sni) {
			mode = 'Sni';
		} else {
			// Default to Single
			mode = 'Single';
			// CRITICAL: Ensure Single data structure exists if missing
			// This fixes the issue where the form doesn't appear for new items
			if (!v.Single) {
				// Initialize with empty link/value structures if needed, 
				// but for now simple empty objects might suffice if LinkOrValueEditor handles undefined
				v.Single = { certs: undefined, key: undefined, ocsp: null };
			}
		}
	});

	function switchMode(newMode: 'Single' | 'Sni') {
		if (mode === newMode) return;
		mode = newMode;
		
		// CRITICAL: Modify the object properties in place using the reference.
		// DO NOT assign to 'value' (e.g. value = { ... }) as that replaces the entire object
		// and causes data loss of parent properties like 'name' and 'options'.
		const v = value as any;

		if (newMode === 'Single') {
			if (v.Sni) delete v.Sni;
			if (!v.Single) {
				v.Single = { certs: undefined, key: undefined, ocsp: null };
			}
		} else {
			if (v.Single) delete v.Single;
			if (!v.Sni) {
				v.Sni = {};
			}
		}
	}
</script>

{#snippet certForm(params: TlsCertParams)}
	<div
		class="space-y-4 rounded border border-surface-200 bg-surface-50 p-4 dark:border-surface-700 dark:bg-surface-900/50"
	>
		<!-- Key: LinkOrValue<PemFile> -->
		<div class="space-y-1">
			<span class="label-text font-bold">Private Key</span>
			<LinkOrValueEditor
				bind:value={params.key}
				dataType="PemFile"
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
			<span class="label-text font-bold">Certificate Chain</span>
			<LinkOrValueEditor
				bind:value={params.certs}
				dataType="PemsFile"
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

<div class="space-y-4">
	<!-- Mode Switcher -->
	<div class="flex gap-4 border-b border-surface-200 dark:border-surface-700">
		<button
			class="border-b-2 px-4 py-2 transition-colors {mode === 'Single'
				? 'border-primary-500 font-bold text-primary-500'
				: 'border-transparent opacity-50'}"
			onclick={() => switchMode('Single')}
		>
			Single Certificate
		</button>
		<button
			class="border-b-2 px-4 py-2 transition-colors {mode === 'Sni'
				? 'border-primary-500 font-bold text-primary-500'
				: 'border-transparent opacity-50'}"
			onclick={() => switchMode('Sni')}
		>
			SNI Mapping
		</button>
	</div>

	{#if mode === 'Single' && value.Single}
		{@render certForm(value.Single)}
	{:else if mode === 'Sni' && value.Sni}
		<!-- Sni Editor -->
		<div class="space-y-4">
			{#each Object.entries(value.Sni) as [domain, params]}
				<div class="card border border-surface-300 p-4 dark:border-surface-600">
					<div class="mb-4 flex items-center justify-between">
						<h4 class="h4 font-bold">{domain}</h4>
						<button
							class="variant-filled-error btn btn-sm"
							onclick={() => {
								const newSni = { ...value.Sni };
								delete newSni[domain];
								value.Sni = newSni;
							}}>Delete</button
						>
					</div>
					{@render certForm(params as TlsCertParams)}
				</div>
			{/each}

			<!-- Add Domain -->
			<div class="flex items-end gap-2">
				<label class="label">
					<span>New Domain (SNI)</span>
					<input class="input" type="text" placeholder="example.com" id="new-sni-domain" />
				</label>
				<button
					class="variant-filled-primary btn"
					onclick={() => {
						const input = document.getElementById('new-sni-domain') as HTMLInputElement;
						const domain = input.value;
						if (domain && value.Sni) {
							value.Sni = {
								...value.Sni,
								[domain]: { certs: undefined, key: undefined, ocsp: null }
							};
							input.value = '';
						}
					}}>Add Domain</button
				>
			</div>
		</div>
	{/if}
</div>
