<script lang="ts">
	import type { FileBind } from '$lib/api/types/human_readable';
	import { Lock, Unlock } from '@lucide/svelte';

	interface Props {
		bind: FileBind;
		jumpToTls?: ((tlsName: string) => void) | null;
	}

	let { bind, jumpToTls = null }: Props = $props();
</script>

<div class="flex items-center gap-2 text-sm">
	{#if bind.tls}
		<Lock class="h-4 w-4 flex-shrink-0 text-success-500" />
	{:else}
		<Unlock class="h-4 w-4 flex-shrink-0 text-warning-500" />
	{/if}
	<code class="rounded bg-surface-200 px-2 py-0.5 font-mono dark:bg-surface-700">{bind.bind}</code>
	{#if bind.tls}
		{#if jumpToTls}
			<button
				class="rounded bg-primary-100 px-2 py-0.5 text-xs text-primary-700 underline underline-offset-2 transition-colors hover:bg-primary-200 dark:bg-primary-900 dark:text-primary-300 dark:hover:bg-primary-800"
				onclick={() => jumpToTls(bind.tls!)}
			>
				{bind.tls}
			</button>
		{:else}
			<span
				class="rounded bg-primary-100 px-2 py-0.5 text-xs text-primary-700 dark:bg-primary-900 dark:text-primary-300"
			>
				{bind.tls}
			</span>
		{/if}
	{/if}
	{#if bind.description}
		<span class="text-xs text-surface-500 dark:text-surface-400">- {bind.description}</span>
	{/if}
</div>
