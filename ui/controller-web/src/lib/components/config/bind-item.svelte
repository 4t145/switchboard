<script lang="ts">
	import type { FileBind } from '$lib/api/types/human_readable';
	import { Lock } from 'lucide-svelte';

	interface Props {
		bind: FileBind;
		jumpToTls?: ((tlsName: string) => void) | null;
	}

	let { bind, jumpToTls = null }: Props = $props();
</script>

<div class="flex items-center gap-2 text-sm">
	{#if bind.tls}
		<Lock class="w-4 h-4 text-primary-500 flex-shrink-0" />
	{/if}
	<code class="font-mono bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded"
		>{bind.bind}</code
	>
	{#if bind.tls}
		{#if jumpToTls}
			<button
				class="text-xs bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300 hover:bg-primary-200 dark:hover:bg-primary-800 px-2 py-0.5 rounded transition-colors underline underline-offset-2"
				onclick={() => jumpToTls(bind.tls!)}
			>
				{bind.tls}
			</button>
		{:else}
			<span
				class="text-xs bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300 px-2 py-0.5 rounded"
			>
				{bind.tls}
			</span>
		{/if}
	{/if}
	{#if bind.description}
		<span class="text-xs text-surface-500 dark:text-surface-400">- {bind.description}</span>
	{/if}
</div>
