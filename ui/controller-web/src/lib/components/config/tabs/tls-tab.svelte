<script lang="ts">
	import type { FileStyleTls, FileTcpServiceConfig } from '$lib/api/types/human_readable';
	import TlsConfigCard from '../tls-config-card.svelte';
	import { Lock } from 'lucide-svelte';

	interface Props {
		tlsConfigs: FileStyleTls[];
		services: FileTcpServiceConfig[];
		highlightTls?: string | null;
		jumpTarget?: { type: 'service' | 'tls' | null; id: string | null } | null;
		jumpToService: (serviceName: string) => void;
	}

	let { tlsConfigs, services, highlightTls = null, jumpTarget = null, jumpToService }: Props = $props();

	// Element references for jump functionality
	let tlsElements: Record<string, HTMLDivElement | undefined> = {};

	// Calculate which services use each TLS config
	const tlsUsage = $derived<Record<string, string[]>>(
		tlsConfigs.reduce(
			(acc, tls) => {
				const usedByServices = services
					.filter((service) => service.binds.some((bind) => bind.tls === tls.name))
					.map((service) => service.name);
				acc[tls.name] = usedByServices;
				return acc;
			},
			{} as Record<string, string[]>
		)
	);

	// Handle jump to TLS config (backwards compatibility)
	$effect(() => {
		if (jumpTarget?.type === 'tls' && jumpTarget.id) {
			const element = tlsElements[jumpTarget.id];
			if (element) {
				// Scroll to element
				setTimeout(() => {
					element.scrollIntoView({ behavior: 'smooth', block: 'center' });
					// Add highlight class
					element.classList.add('jump-highlight');
					// Remove after animation
					setTimeout(() => {
						element.classList.remove('jump-highlight');
					}, 2000);
				}, 100);
			}
		}
	});

	// Handle URL-based highlighting (new method)
	$effect(() => {
		if (highlightTls) {
			const element = tlsElements[highlightTls];
			if (element) {
				// Scroll to element
				setTimeout(() => {
					element.scrollIntoView({ behavior: 'smooth', block: 'center' });
					// Add highlight class
					element.classList.add('jump-highlight');
					// Remove after animation
					setTimeout(() => {
						element.classList.remove('jump-highlight');
					}, 2000);
				}, 100);
			}
		}
	});
</script>

<div class="space-y-4">
	<h2 class="h2 flex items-center gap-2">
		<Lock class="w-6 h-6" />
		TLS Configurations
		{#if tlsConfigs.length > 0}
			<span class="text-sm font-normal text-surface-500 dark:text-surface-400">
				({tlsConfigs.length})
			</span>
		{/if}
	</h2>

	{#if tlsConfigs.length > 0}
		{#each tlsConfigs as tls (tls.name)}
			<div bind:this={tlsElements[tls.name]} class="transition-all duration-300 space-y-2 rounded-lg">
				<TlsConfigCard {tls} />

				<!-- Show which services use this TLS config -->
				{#if tlsUsage[tls.name] && tlsUsage[tls.name].length > 0}
					<div class="card p-3 bg-surface-50 dark:bg-surface-800/50">
						<div class="text-sm text-surface-600 dark:text-surface-400">
							<span class="font-medium">Used by {tlsUsage[tls.name].length} service(s):</span>
							<div class="flex flex-wrap gap-2 mt-2">
								{#each tlsUsage[tls.name] as serviceName}
									<button
										class="btn btn-sm preset-ghost-primary gap-1"
										onclick={() => jumpToService(serviceName)}
									>
										{serviceName}
									</button>
								{/each}
							</div>
						</div>
					</div>
				{/if}
			</div>
		{/each}
	{:else}
		<div class="card p-6 text-center text-surface-500 dark:text-surface-400">
			No TLS configurations
		</div>
	{/if}
</div>

<style>
	.jump-highlight {
		box-shadow: 0 0 0 2px rgb(var(--color-primary-500) / 1);
		animation: pulse-border 1s ease-in-out 2;
	}

	@keyframes pulse-border {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}
</style>
