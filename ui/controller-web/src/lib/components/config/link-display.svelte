<script lang="ts">
	import { resolveLink, resolveLinkAsString } from '$lib/api/resolve';
	import { Collapsible } from 'bits-ui';
	import { Link as LinkIcon, AlertCircle, Loader2 } from 'lucide-svelte';

	interface Props {
		value: unknown;
		/** Whether to attempt resolving the link content */
		resolveContent?: boolean;
	}

	let { value, resolveContent = false }: Props = $props();

	let open = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let resolvedContent = $state<string | null>(null);

	// Check if the value is a link
	const isLink = $derived(
		typeof value === 'string' &&
			(value.startsWith('file://') ||
				value.startsWith('storage://') ||
				value.startsWith('http://') ||
				value.startsWith('https://'))
	);

	const linkType = $derived.by(() => {
		if (!isLink || typeof value !== 'string') return 'unknown';
		if (value.startsWith('file://')) return 'file';
		if (value.startsWith('storage://')) return 'storage';
		if (value.startsWith('http://') || value.startsWith('https://')) return 'http';
		return 'unknown';
	});

	async function loadContent() {
		if (!isLink || typeof value !== 'string') return;

		loading = true;
		error = null;
		resolvedContent = null;

		try {
			// Try to resolve as string first (for TOML/text files)
			const content = await resolveLinkAsString(value);
			resolvedContent = content;
		} catch (err) {
			// If string resolution fails, try JSON
			try {
				const data = await resolveLink(value);
				resolvedContent = JSON.stringify(data, null, 2);
			} catch (jsonErr) {
				error = err instanceof Error ? err.message : 'Failed to resolve link';
			}
		} finally {
			loading = false;
		}
	}

	// Load content when opened
	$effect(() => {
		if (open && resolveContent && isLink && !resolvedContent && !loading && !error) {
			loadContent();
		}
	});
</script>

{#if isLink && typeof value === 'string'}
	<!-- Link display -->
	<div class="inline-flex items-center gap-1.5">
		<LinkIcon class="w-3.5 h-3.5 text-primary-500" />
		<code
			class="text-sm bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded font-mono break-all"
		>
			{value}
		</code>
		{#if linkType !== 'unknown'}
			<span class="text-xs text-surface-500 dark:text-surface-400">({linkType})</span>
		{/if}
	</div>

	{#if resolveContent}
		<!-- Collapsible content viewer -->
		<Collapsible.Root bind:open class="mt-2">
			<Collapsible.Trigger
				class="btn btn-sm preset-ghost-surface w-full text-left justify-start gap-2"
			>
				<span class="text-lg transition-transform duration-200" class:rotate-90={open}>▶</span>
				{open ? 'Hide content' : 'View content'}
			</Collapsible.Trigger>
			<Collapsible.Content class="mt-2">
				{#if loading}
					<div class="flex items-center gap-2 p-4 bg-surface-100 dark:bg-surface-800 rounded">
						<Loader2 class="w-4 h-4 animate-spin" />
						<span class="text-sm">Loading content...</span>
					</div>
				{:else if error}
					<div
						class="flex items-start gap-2 p-4 bg-error-100 dark:bg-error-900 text-error-700 dark:text-error-300 rounded"
					>
						<AlertCircle class="w-4 h-4 mt-0.5 flex-shrink-0" />
						<div class="text-sm">
							<div class="font-semibold">Failed to load content</div>
							<div class="text-xs mt-1">{error}</div>
						</div>
					</div>
				{:else if resolvedContent}
					<div class="bg-surface-100 dark:bg-surface-800 rounded p-4 overflow-x-auto">
						<pre class="text-xs font-mono"><code>{resolvedContent}</code></pre>
					</div>
				{/if}
			</Collapsible.Content>
		</Collapsible.Root>
	{/if}
{:else}
	<!-- Regular value display -->
	<code class="text-sm bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded font-mono">
		{JSON.stringify(value)}
	</code>
{/if}
