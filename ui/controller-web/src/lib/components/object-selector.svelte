<script lang="ts" module>
	import type { StorageObjectWithoutData } from '$lib/api/types';
	import type { ObjectFilter } from '$lib/api/routes/storage';
	type ObjectField = keyof ObjectFilter;
	interface Props {
		pageSize?: number;
		filter: ObjectFilter;
		lockedFileds?: ObjectField[];
		onSelectionChange?: (selected: StorageObjectWithoutData[]) => void;
		selectionMode?: 'none' | 'single' | 'multiple';
		showFilters?: boolean;
		showViewDetails?: boolean;
		showEdit?: boolean;
		showDelete?: boolean;
	}
</script>

<script lang="ts">
	import { api } from '$lib/api/routes';
	import { shortRev } from '$lib/utils';
	import {
		Info,
		Loader2,
		Edit,
		Trash2,
		Eye,
		ListPlusIcon,
		FunnelIcon,
		FunnelPlusIcon,
		CopyIcon
	} from '@lucide/svelte';
	import { Popover, Portal } from '@skeletonlabs/skeleton-svelte';
	import TableListEditor, {
		type RowParams,
		type ListOperations
	} from './common/table-list-editor.svelte';
	import { type Snippet } from 'svelte';

	// Props
	let {
		pageSize = 20,
		filter: initialFilter,
		lockedFileds = [],
		onSelectionChange = () => {},
		selectionMode = 'none',
		showFilters = true,
		showViewDetails = true,
		showEdit = true,
		showDelete = true
	}: Props = $props();

	// Internal State
	let items = $state<StorageObjectWithoutData[]>([]);
	let cursor = $state<string | null>(null);
	let loading = $state(false);
	let error = $state<Error | null>(null);
	let hasMore = $state(true);
	let selectedItems: StorageObjectWithoutData[] = $state([]);
	// Delete confirmation dialog state
	// Filter State
	let currentFilter: ObjectFilter = $derived.by(() => ({
		...initialFilter,
		id: filterID ?? undefined,
		revision: filterRevision ?? undefined,
		key: filterKey ?? undefined,
		latest_only: filterLatestOnly ?? undefined,
		data_type: filterDataType ?? undefined
	}));
	let filterKey = $derived.by(() => JSON.stringify(initialFilter));
	
	let filterID = $state<string | null>(initialFilter.id ?? null);
	let filterRevision = $state<string | null>(initialFilter.revision ?? null);
	let filterLatestOnly = $state<boolean>(initialFilter.latest_only ?? true);
	let filterDataType = $state<string | null>(initialFilter.data_type ?? null);
	const dataTypeOptions = [
		{ label: 'Service Config', value: 'ServiceConfig' },
		{ label: 'Pem', value: 'Pem' }
	];
	// --- Data Loading ---

	async function loadNextPage() {
		if (loading || (!hasMore && cursor === null) || error) return;

		loading = true;
		error = null;

		try {
			const response = await api.storage.list(
				{
					cursor: { next: cursor },
					limit: pageSize
				},
				currentFilter
			);

			const newItems = response.items.map((item) => item.data);
			items = [...items, ...newItems];
			cursor = response.next_cursor?.next ?? null;

			hasMore = cursor !== null;
		} catch (e) {
			error = e instanceof Error ? e : new Error(String(e));
			console.error('Failed to load storage objects', e);
		} finally {
			loading = false;
		}
	}

	function reload() {
		items = [];
		cursor = null;
		hasMore = true;
		error = null;
		loadNextPage();
	}

	// React to initialFilter changes from parent
	$effect(() => {
		const nextKey = JSON.stringify(initialFilter);
		// Only reset if the PROP changes significantly
		// Note: this might conflict if we are manually filtering.
		// We assume if parent pushes new filter, we respect it.
		// To avoid loops, we check against currentFilter's base?
		// No, simple effect:
		if (nextKey === JSON.stringify(currentFilter)) return;

		// This is tricky: we want to allow local filtering (via form) AND parent updates.
		// If we just overwrite currentFilter here, we lose form state.
		// But if we don't, parent can't update.
		// Let's assume parent updates override everything.
		currentFilter = { ...initialFilter };
		filterKey = nextKey;
		reload();
	});

	// Also trigger reload when currentFilter changes (via form submit)
	$effect(() => {
		const nextKey = JSON.stringify(currentFilter);
		if (nextKey === filterKey) return;
		filterKey = nextKey;
		reload();
	});
	$effect(() => {
		const nextKey = JSON.stringify(currentFilter);
		if (nextKey === filterKey) return;
		filterKey = nextKey;
		reload();
	});
	$effect(() => {
		onSelectionChange(selectedItems);
	});
	function isSelected(item: StorageObjectWithoutData): boolean {
		return selectedItems.some((selected) => selected.descriptor.id === item.descriptor.id);
	}

	function copyToClipboard(e: Event, text: string) {
		e.stopPropagation(); // Prevent card selection
		navigator.clipboard.writeText(text);
		// Ideally show toast
	}

	function getIdRevString(item: StorageObjectWithoutData): string {
		return `${item.descriptor.id}#${item.descriptor.revision}`;
	}
</script>

{#snippet filterTableHeader(label: string, hasFilter: () => boolean, filter: Snippet)}
	<div class="align-center flex flex-row items-center gap-1">
		<span> {label} </span>
		{#if showFilters}
			<Popover>
				<Popover.Trigger class="btn-icon btn-icon-sm preset-filled">
					{#if !hasFilter()}
						<FunnelIcon></FunnelIcon>
					{:else}
						<FunnelPlusIcon></FunnelPlusIcon>
					{/if}
				</Popover.Trigger>
				<Portal>
					<Popover.Positioner>
						<Popover.Content class="z-100">
							<div class="flex flex-col gap-1 card bg-surface-100-900 p-2 shadow-xl">
								{@render filter()}
							</div>
						</Popover.Content>
						<Popover.Arrow
							class="[--arrow-background:var(--color-surface-100-900)] [--arrow-size:--spacing(2)]"
						>
							<Popover.ArrowTip />
						</Popover.Arrow>
					</Popover.Positioner>
				</Portal>
			</Popover>
		{/if}
	</div>
{/snippet}

<TableListEditor value={items}>
	{#snippet header()}
		<tr>
			{#if selectionMode !== 'none'}
				<th>
					{#if selectionMode === 'multiple'}{/if}
				</th>
			{/if}
			<th>
				{#snippet filter()}
					<input
						class="input"
						value={filterID ?? undefined}
						placeholder="ID"
						disabled={lockedFileds.includes('id')}
						onchange={(e) => {
							const input = e.currentTarget.value;
							if (input) {
								filterID = input;
							} else {
								filterID = null;
							}
						}}
					/>
				{/snippet}
				{@render filterTableHeader('ID', () => filterID !== null, filter)}
			</th>
			<th>
				{#snippet filter()}
					<input
						class="input"
						type="input"
						value={filterRevision ?? undefined}
						disabled={lockedFileds.includes('revision')}
						placeholder="revision"
						onchange={(e) => {
							const input = e.currentTarget.value;
							if (input) {
								filterRevision = input;
							} else {
								filterRevision = null;
							}
						}}
					/>
					<label id="latest-only" class="label">
						<input
							class="checkbox"
							type="checkbox"
							disabled={lockedFileds.includes('latest_only')}
							aria-label="latest-only"
							bind:checked={filterLatestOnly}
						/>
						Latest Only
					</label>
				{/snippet}
				{@render filterTableHeader(
					'Revision',
					() => filterRevision !== null || filterLatestOnly,
					filter
				)}
			</th>
			<th>
				{#snippet filter()}
					<select
						class="select"
						value={filterDataType}
						disabled={lockedFileds.includes('data_type')}
						onchange={(e) => {
							if (e.currentTarget.value === '') {
								filterDataType = null;
							} else {
								filterDataType = e.currentTarget.value;
							}
						}}
					>
						<option value="">All Types</option>
						{#each dataTypeOptions as option (option.value)}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				{/snippet}
				{@render filterTableHeader('Type', () => filterDataType !== null, filter)}
			</th>
			<th> Created At </th>
			<th> Operations </th>
		</tr>
	{/snippet}
	{#snippet row(api: RowParams<StorageObjectWithoutData>)}
		{@const item = api.value}
		<tr>
			{#if selectionMode !== 'none'}
				<td>
					<input
						class="checkbox"
						type="checkbox"
						checked={isSelected(item)}
						onchange={(e) => {
							let checked = e.currentTarget.checked;
							if (checked) {
								if (selectionMode === 'single') {
									selectedItems = [item];
								} else if (selectionMode === 'multiple') {
									selectedItems = [...selectedItems, item];
								}
							} else {
								if (selectionMode === 'single') {
									selectedItems = [];
								} else if (selectionMode === 'multiple') {
									selectedItems = selectedItems.filter(
										(selectedItem) => getIdRevString(selectedItem) !== getIdRevString(item)
									);
								}
							}
						}}
					/>
				</td>
			{/if}
			<td>
				<span class="font-mono text-primary-700-300">
					{item.descriptor.id}
				</span>
			</td>
			<td>
				<span class="font-mono text-surface-700-300">{shortRev(item.descriptor.revision)}</span>
			</td>
			<td>
				{item.meta.data_type}
			</td>
			<td>
				{new Date(item.meta.created_at).toLocaleDateString()}
			</td>
			<td>
				<button
					class="btn-icon btn-icon-sm preset-tonal-surface"
					onclick={(e) => {
						copyToClipboard(e, getIdRevString(item));
					}}
					title="Details"
				>
					<CopyIcon size={16} />
				</button>
				{#if showViewDetails}
					<button
						class="btn-icon btn-icon-sm preset-tonal-surface"
						onclick={(e) => {}}
						title="Details"
					>
						<Eye size={16} />
					</button>
				{/if}
				{#if showEdit}
					<button
						class="btn-icon btn-icon-sm preset-tonal-secondary"
						onclick={(e) => {}}
						title="Edit"
					>
						<Edit size={16} />
					</button>
				{/if}
				{#if showDelete}
					<button
						class="btn-icon btn-icon-sm preset-tonal-error"
						onclick={(e) => {}}
						title="Delete"
					>
						<Trash2 size={16} />
					</button>
				{/if}
			</td>
		</tr>
	{/snippet}
	{#snippet footer()}
		{@const colCount = selectionMode !== 'none' ? 6 : 5}
		<tr>
			<td colspan={colCount} class="text-center">
				{#if error}
					<div class="alert flex items-center gap-2 preset-tonal-error">
						<Info size={16} />
						<span class="flex-1">Error: {error.message}</span>
						<button class="btn preset-filled-error-500 btn-sm" onclick={reload}>Retry</button>
					</div>
				{/if}
				{#if loading && items.length === 0}
					<!-- Initial Load -->
					<div class="flex items-center gap-2 text-surface-500">
						<Loader2 class="animate-spin" size={20} />
						<span>Loading...</span>
					</div>
				{:else if !loading && items.length === 0 && !error}
					<div class="flex flex-col items-center gap-2 py-8 text-surface-400">
						<Info size={32} />
						<p>No items found.</p>
					</div>
				{:else if hasMore}
					<button class="btn preset-tonal-surface btn-sm" onclick={loadNextPage} disabled={loading}>
						{#if loading}
							<Loader2 class="mr-2 animate-spin" size={14} /> Loading...
						{:else}
							<ListPlusIcon class="size-4"></ListPlusIcon>
							Load More
						{/if}
					</button>
				{:else if items.length > 0}
					<span class="text-xs text-surface-400">End of list</span>
				{/if}
			</td>
		</tr>
	{/snippet}
</TableListEditor>
