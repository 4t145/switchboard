<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import { resolveLink, resolveLinkAsString } from '$lib/api/resolve';
	import { Accordion } from '@skeletonlabs/skeleton-svelte';
	import { Link as LinkIcon, AlertCircle, Loader2, ChevronDown } from 'lucide-svelte';
	import DataTypeRenderer from '$lib/data-types/components/data-type-renderer.svelte';

	interface Props<T> {
		value: T;
		/** Whether to attempt resolving the link content */
		resolveContent?: 'string' | 'value' | false;

		/** Custom snippet for displaying content (works for both direct values and resolved link content) */
		customDisplay?: Snippet<[{ content: T }]>;

		/** Data type for rendering with the data type system */
		dataType?: string;

		/** Additional props to pass to the data type renderer */
		editorProps?: Record<string, any>;
	}

	let { value, resolveContent = false, customDisplay, dataType, editorProps = {} }: Props<T> = $props();

	let loading = $state(false);
	let error = $state<string | null>(null);
	let resolvedContent = $state<T | null>(null);
	let accordionOpen = $state<string[]>([]);
		
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
			if (resolveContent === 'value') {
				const data = await resolveLink(value);
				resolvedContent = data as T;
				return;
			} else if (resolveContent === 'string') {
				const content = await resolveLinkAsString(value);
				resolvedContent = content as unknown as T;
				return;
			} 
		} catch (err) {
			// If string resolution fails, try JSON
			error = err instanceof Error ? err.message : 'Failed to resolve link';
		} finally {
			loading = false;
		}
	}

	// Load content when accordion is opened
	$effect(() => {
		const isOpen = accordionOpen.includes('content');
		if (isOpen && resolveContent && isLink && !resolvedContent && !loading && !error) {
			loadContent();
		}
	});
</script>

{#if isLink && typeof value === 'string'}
	<!-- Link display (fixed format) -->
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
		<!-- Accordion content viewer -->
		<Accordion
			collapsible
			class="mt-2"
			value={accordionOpen}
			onValueChange={(details) => (accordionOpen = details.value)}
		>
			<Accordion.Item value="content">
				<Accordion.ItemTrigger
					class="btn btn-sm preset-ghost-surface w-full text-left justify-between gap-2"
				>
					{accordionOpen.includes('content') ? 'Hide content' : 'View content'}
					<Accordion.ItemIndicator class="group">
						<ChevronDown class="h-4 w-4 transition group-data-[state=open]:rotate-180" />
					</Accordion.ItemIndicator>
				</Accordion.ItemTrigger>
				<Accordion.ItemContent class="mt-2">
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
						{#if customDisplay}
							<!-- Custom content display -->
							{@render customDisplay({ content: resolvedContent })}
						{:else if dataType}
							<!-- Use data type renderer for view mode -->
							<DataTypeRenderer
								type={dataType}
								mode="view"
								value={resolvedContent}
								{...editorProps}
							/>
						{:else}
							<!-- Default content display -->
							<div class="bg-surface-100 dark:bg-surface-800 rounded p-4 overflow-x-auto">
								<pre class="text-xs font-mono"><code>{resolvedContent}</code></pre>
							</div>
						{/if}
					{/if}
				</Accordion.ItemContent>
			</Accordion.Item>
		</Accordion>
	{/if}
{:else}
	<!-- Regular value display -->
	{#if customDisplay}
		{@render customDisplay({ content: value })}
	{:else if dataType}
		<!-- Use data type renderer for view mode -->
		<DataTypeRenderer
			type={dataType}
			mode="view"
			{value}
			{...editorProps}
		/>
	{:else}
		<!-- Default value display -->
		<code class="text-sm bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded font-mono">
			{JSON.stringify(value)}
		</code>
	{/if}
{/if}
		