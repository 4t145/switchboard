<script lang="ts">
	import type { FileTcpServiceConfig } from '$lib/api/types/human_readable';
	import TcpServiceCard from '../tcp-service-card.svelte';
	import { Server } from 'lucide-svelte';

	interface Props {
		services: FileTcpServiceConfig[];
		highlightService?: string | null;
		jumpTarget?: { type: 'service' | 'tls' | null; id: string | null } | null;
		jumpToTls?: ((tlsName: string) => void) | null;
	}

	let { services, highlightService = null, jumpTarget = null, jumpToTls = null }: Props = $props();

	// Element references for jump functionality
	let serviceElements: Record<string, HTMLDivElement | undefined> = {};

	// Handle jump to service (backwards compatibility)
	$effect(() => {
		if (jumpTarget?.type === 'service' && jumpTarget.id) {
			const element = serviceElements[jumpTarget.id];
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
		if (highlightService) {
			const element = serviceElements[highlightService];
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
		<Server class="w-6 h-6" />
		TCP Services
		{#if services.length > 0}
			<span class="text-sm font-normal text-surface-500 dark:text-surface-400">
				({services.length})
			</span>
		{/if}
	</h2>

	{#if services.length > 0}
		{#each services as service (service.name)}
			<div bind:this={serviceElements[service.name]} class="transition-all duration-300 rounded-lg">
				<TcpServiceCard {service} {jumpToTls} />
			</div>
		{/each}
	{:else}
		<div class="card p-6 text-center text-surface-500 dark:text-surface-400">
			No TCP services configured
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
