<script lang="ts">
	import type { FileStyleTls } from '$lib/api/types/human_readable';
	import { Collapsible } from 'bits-ui';
	import { Lock, Shield } from 'lucide-svelte';
	import LinkDisplay from './link-display.svelte';

	interface Props {
		tls: FileStyleTls;
	}

	let { tls }: Props = $props();

	// Check if this is SNI (multi-domain) TLS config
	const isSni = $derived('sni' in tls && Array.isArray(tls.sni));
	const sniDomains = $derived(isSni && 'sni' in tls ? tls.sni : []);

	// For single certificate
	const hasCerts = $derived('certs' in tls);
	const hasKey = $derived('key' in tls);

	let domainsOpen = $state(false);
</script>

<div class="card p-4 space-y-3">
	<!-- TLS header -->
	<div class="flex items-center gap-2">
		<Lock class="w-5 h-5 text-primary-500 flex-shrink-0" />
		<h3 class="font-semibold text-lg">{tls.name}</h3>
	</div>

	<!-- Type badge -->
	<div class="flex items-center gap-2">
		<Shield class="w-4 h-4 text-surface-500" />
		<span class="text-sm font-medium text-surface-700 dark:text-surface-300">
			{#if isSni}
				SNI Multi-Certificate ({sniDomains.length} domains)
			{:else}
				Single Certificate
			{/if}
		</span>
	</div>

	{#if isSni}
		<!-- SNI domains list -->
		<Collapsible.Root bind:open={domainsOpen}>
			<Collapsible.Trigger
				class="btn btn-sm preset-ghost-surface w-full text-left justify-start gap-2"
			>
				<span class="text-lg transition-transform duration-200" class:rotate-90={domainsOpen}
					>▶</span
				>
				{domainsOpen ? 'Hide domains' : 'View all domains'}
			</Collapsible.Trigger>
			<Collapsible.Content class="mt-2 space-y-3">
				{#each sniDomains as domain}
					<div class="pl-4 border-l-2 border-primary-300 dark:border-primary-700 space-y-2">
						<div class="font-medium text-sm">{domain.hostname}</div>
						{#if domain.certs}
							<div class="text-sm">
								<span class="text-surface-600 dark:text-surface-400">Certs:</span>
								<div class="mt-1">
									<LinkDisplay value={domain.certs} resolveContent={true} />
								</div>
							</div>
						{/if}
						{#if domain.key}
							<div class="text-sm">
								<span class="text-surface-600 dark:text-surface-400">Key:</span>
								<div class="mt-1">
									<LinkDisplay value={domain.key} resolveContent={true} />
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</Collapsible.Content>
		</Collapsible.Root>
	{:else}
		<!-- Single certificate -->
		<div class="space-y-2">
			{#if hasCerts && 'certs' in tls}
				<div class="text-sm">
					<span class="text-surface-600 dark:text-surface-400 font-medium">Certificate:</span>
					<div class="mt-1 pl-2">
						<LinkDisplay value={tls.certs} resolveContent={true} />
					</div>
				</div>
			{/if}
			{#if hasKey && 'key' in tls}
				<div class="text-sm">
					<span class="text-surface-600 dark:text-surface-400 font-medium">Private Key:</span>
					<div class="mt-1 pl-2">
						<LinkDisplay value={tls.key} resolveContent={true} />
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- TLS Options (if present) -->
	{#if tls.options}
		<div class="pt-2 border-t border-surface-300 dark:border-surface-700">
			<div class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
				TLS Options:
			</div>
			<code class="text-xs bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded block overflow-x-auto">
				{JSON.stringify(tls.options, null, 2)}
			</code>
		</div>
	{/if}
</div>
