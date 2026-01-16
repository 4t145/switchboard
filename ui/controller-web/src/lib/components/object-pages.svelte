<script lang="ts" module>
	import type { StorageObjectWithoutData } from '$lib/api/types';
	import type { ObjectFilter } from '$lib/api/routes/storage';

	interface Props {
		pageSize: number;
		filter: ObjectFilter & {
			lockedFields?: string[];
		};
		onSelect?: (item: StorageObjectWithoutData) => void;
		onRemove?: (item: StorageObjectWithoutData) => void;
		onEdit?: (item: StorageObjectWithoutData) => void;
		onViewDetails?: (item: StorageObjectWithoutData) => void;
		selectedId?: string | null;
		selectedItems?: StorageObjectWithoutData[];
		selectionMode?: 'none' | 'single' | 'multiple';
		showFilters?: boolean;
		// Action button visibility controls
		showViewDetails?: boolean;
		showEdit?: boolean;
		showDelete?: boolean;
	}
</script>

<script lang="ts">
	import { api } from '$lib/api/routes';
	import { shortRev } from '$lib/utils';
	import { Info, Loader2, Check, Search, Copy, Edit, Trash2, Eye, X } from 'lucide-svelte';
	import ObjectFilterForm from '$lib/components/object-filter.svelte';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';

	// Props
	let {
		pageSize,
		filter: initialFilter,
		onSelect,
		onRemove,
		onEdit,
		onViewDetails,
		selectedId = null,
		selectedItems = [],
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

	// Delete confirmation dialog state
	let showDeleteDialog = $state(false);
	let itemToDelete = $state<StorageObjectWithoutData | null>(null);
	let deleteLoading = $state(false);

	// Filter State
	let currentFilter = $state<ObjectFilter>({ ...initialFilter });
	let filterKey = $state<string | null>(null);

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

	// React to filter form updates
	function handleFilterSubmit(newFilter: ObjectFilter) {
		// Merge with initialFilter to keep any fixed constraints (e.g. data_type) if they weren't in the form
		// But here we trust the form output mostly.
		// We should probably merge: base constraints + form values.
		// Assuming the form returns a full filter object.
		currentFilter = { ...initialFilter, ...newFilter };
		// If initialFilter had specific constraints not exposed in form, make sure they aren't lost if the form returns undefined for them.
		// For now, simple merge.
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

	// --- Interaction ---
	function handleSelect(item: StorageObjectWithoutData) {
		if (onSelect) {
			onSelect(item);
		}
	}

	function handleRemove(item: StorageObjectWithoutData) {
		// If onRemove callback is provided, use it (custom behavior)
		// Otherwise, use built-in delete functionality
		if (onRemove) {
			onRemove(item);
		} else {
			// Built-in delete functionality
			itemToDelete = item;
			showDeleteDialog = true;
		}
	}

	async function handleDeleteConfirm() {
		if (!itemToDelete) return;

		deleteLoading = true;
		try {
			await api.storage.delete(itemToDelete.descriptor);
			// Remove item from local state immediately for better UX
			items = items.filter(item => 
				!(item.descriptor.id === itemToDelete!.descriptor.id && 
				  item.descriptor.revision === itemToDelete!.descriptor.revision)
			);
			// TODO: Show success toast notification
			console.log(`Object deleted: ${itemToDelete.descriptor.id}#${itemToDelete.descriptor.revision}`);
		} catch (deleteError) {
			// TODO: Show error toast notification
			console.error('Failed to delete object:', deleteError);
			error = deleteError instanceof Error ? deleteError : new Error(String(deleteError));
		} finally {
			deleteLoading = false;
			itemToDelete = null;
			showDeleteDialog = false;
		}
	}

	function handleDeleteCancel() {
		itemToDelete = null;
		showDeleteDialog = false;
	}

	function handleEdit(item: StorageObjectWithoutData) {
		// Use custom callback if provided, otherwise use built-in edit functionality
		if (onEdit) {
			onEdit(item);
		} else {
			// Built-in edit functionality (placeholder for now)
			console.log('Edit functionality not implemented. Override with onEdit callback.');
			alert('编辑功能需要通过 onEdit 回调实现');
		}
	}

	function handleViewDetails(item: StorageObjectWithoutData) {
		// Use custom callback if provided, otherwise use built-in view functionality
		if (onViewDetails) {
			onViewDetails(item);
		} else {
			// Built-in view functionality (placeholder for now)
			console.log('View details functionality not implemented. Override with onViewDetails callback.');
			alert('查看详情功能需要通过 onViewDetails 回调实现');
		}
	}

	function isSelected(item: StorageObjectWithoutData): boolean {
		if (selectionMode === 'single') {
			return selectedId === item.descriptor.id;
		} else if (selectionMode === 'multiple') {
			return selectedItems.some(selected => selected.descriptor.id === item.descriptor.id);
		}
		return false;
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

<div class="space-y-4">
	<!-- Filter Controls (Reusing ObjectFilterForm) -->
	{#if showFilters}
		<div class="border-b border-surface-200 pb-4 dark:border-surface-700">
			<ObjectFilterForm
				dataType={currentFilter.data_type}
				id={currentFilter.id}
				revision={currentFilter.revision}
				latestOnly={currentFilter.latest_only}
				createdAfter={currentFilter.created_after}
				createdBefore={currentFilter.created_before}
				lockedFields={initialFilter.lockedFields}
				compact
				onSubmit={handleFilterSubmit}
			/>
		</div>
	{/if}

	<!-- Selected Items Section -->
	{#if selectionMode === 'multiple' && selectedItems.length > 0}
		<div class="bg-primary-50 border border-primary-200 rounded-lg p-4 dark:bg-primary-900/10 dark:border-primary-800">
			<h3 class="text-lg font-semibold mb-3 text-primary-900 dark:text-primary-100">
				已选择项目 ({selectedItems.length})
			</h3>
			<div class="space-y-2">
				{#each selectedItems as item (item.descriptor.id + item.descriptor.revision)}
					<div class="bg-white border border-surface-200 rounded-lg p-3 flex items-center justify-between dark:bg-surface-800 dark:border-surface-700">
						<div class="flex items-center gap-3 flex-1 min-w-0">
							<div class="flex items-center gap-2 min-w-0 flex-1">
								<span class="font-mono text-sm font-medium text-primary-700 dark:text-primary-300 truncate" title={getIdRevString(item)}>
									{item.descriptor.id}<span class="text-surface-500 dark:text-surface-400">#{shortRev(item.descriptor.revision)}</span>
								</span>
								<button
									class="rounded p-1 text-surface-500 hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors"
									onclick={(e) => copyToClipboard(e, getIdRevString(item))}
									title="Copy ID#Rev"
								>
									<Copy size={14} />
								</button>
							</div>
							<span class="text-xs text-surface-500 whitespace-nowrap">
								{item.meta.data_type}
							</span>
						</div>
						<div class="flex items-center gap-1 ml-3">
							{#if showViewDetails}
								<button
									class="btn-icon btn-icon-sm hover:preset-tonal-primary transition-colors"
									onclick={(e) => { e.stopPropagation(); handleViewDetails(item); }}
									title="Details"
								>
									<Eye size={16} />
								</button>
							{/if}
							{#if showEdit}
								<button
									class="btn-icon btn-icon-sm hover:preset-tonal-secondary transition-colors"
									onclick={(e) => { e.stopPropagation(); handleEdit(item); }}
									title="Edit"
								>
									<Edit size={16} />
								</button>
							{/if}
							{#if showDelete}
								<button
									class="btn-icon btn-icon-sm hover:preset-tonal-error transition-colors"
									onclick={(e) => { e.stopPropagation(); handleRemove(item); }}
									title="Delete"
								>
									<Trash2 size={16} />
								</button>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Candidates List -->
	<div class="space-y-2">
		{#if items.length > 0}
			<h3 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-3">
				{selectionMode === 'multiple' ? '候选项目' : '项目列表'}
			</h3>
		{/if}
		
		{#each items as item (item.descriptor.id + item.descriptor.revision)}
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="group card border border-surface-200 p-4 transition-all duration-200 hover:border-primary-300 hover:shadow-sm
				       {isSelected(item) ? 'border-primary-500 bg-primary-50 ring-1 ring-primary-500 dark:bg-primary-900/20' : 'dark:bg-surface-800 dark:border-surface-700'}
				       {selectionMode !== 'none' ? 'cursor-pointer' : ''}"
				onclick={() => selectionMode !== 'none' && handleSelect(item)}
				role={selectionMode !== 'none' ? 'button' : undefined}
				tabindex={selectionMode !== 'none' ? 0 : undefined}
				onkeydown={(e) => selectionMode !== 'none' && e.key === 'Enter' && handleSelect(item)}
			>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3 flex-1 min-w-0">
						<div class="flex items-center gap-2 min-w-0 flex-1">
							<span class="font-mono text-sm font-semibold text-surface-900 dark:text-surface-100 truncate" title={getIdRevString(item)}>
								<span class="text-primary-700 dark:text-primary-300">{item.descriptor.id}</span><span class="text-surface-500 dark:text-surface-400">#{shortRev(item.descriptor.revision)}</span>
							</span>
							<button
								class="btn-icon btn-icon-sm opacity-0 group-hover:opacity-100 hover:preset-tonal-surface transition-opacity"
								onclick={(e) => copyToClipboard(e, getIdRevString(item))}
								title="Copy ID#Rev"
							>
								<Copy size={14} />
							</button>
						</div>
						<div class="flex items-center gap-4 text-xs text-surface-500">
							<span>Type: <span class="font-medium text-surface-700 dark:text-surface-300">{item.meta.data_type}</span></span>
							<span>Created: {new Date(item.meta.created_at).toLocaleDateString()}</span>
						</div>
					</div>
					
					<div class="flex items-center gap-1 ml-3">
						{#if showViewDetails}
							<button
								class="btn-icon btn-icon-sm opacity-0 group-hover:opacity-100 hover:preset-tonal-primary transition-opacity"
								onclick={(e) => { e.stopPropagation(); handleViewDetails(item); }}
								title="Details"
							>
								<Eye size={16} />
							</button>
						{/if}
						{#if showEdit}
							<button
								class="btn-icon btn-icon-sm opacity-0 group-hover:opacity-100 hover:preset-tonal-secondary transition-opacity"
								onclick={(e) => { e.stopPropagation(); handleEdit(item); }}
								title="Edit"
							>
								<Edit size={16} />
							</button>
						{/if}
						{#if showDelete}
							<button
								class="btn-icon btn-icon-sm opacity-0 group-hover:opacity-100 hover:preset-tonal-error transition-opacity"
								onclick={(e) => { e.stopPropagation(); handleRemove(item); }}
								title="Delete"
							>
								<Trash2 size={16} />
							</button>
						{/if}
						{#if selectionMode === 'multiple' && !isSelected(item)}
							<button
								class="btn-icon btn-icon-sm opacity-0 group-hover:opacity-100 hover:preset-tonal-success transition-opacity"
								onclick={(e) => { e.stopPropagation(); handleSelect(item); }}
								title="Select"
							>
								<Check size={16} />
							</button>
						{/if}
					</div>

					<!-- Selection Indicator -->
					{#if isSelected(item)}
						<div class="ml-3 flex items-center justify-center w-6 h-6 rounded-full bg-primary-500 text-white">
							<Check size={14} strokeWidth={3} />
						</div>
					{/if}
				</div>
			</div>
		{/each}
	</div>

	<!-- Error State -->
	{#if error}
		<div class="alert preset-tonal-error flex items-center gap-2">
			<Info size={16} />
			<span class="flex-1">Error: {error.message}</span>
			<button class="preset-filled-error btn btn-sm" onclick={reload}>Retry</button>
		</div>
	{/if}

	<!-- Loading / Empty States -->
	<div class="flex justify-center py-4">
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
			<button class="preset-tonal-surface btn btn-sm" onclick={loadNextPage} disabled={loading}>
				{#if loading}
					<Loader2 class="mr-2 animate-spin" size={14} /> Loading...
				{:else}
					Load More
				{/if}
			</button>
		{:else if items.length > 0}
			<span class="text-xs text-surface-400">End of list</span>
		{/if}
	</div>
</div>

<!-- Delete Confirmation Dialog -->
<Dialog open={showDeleteDialog} onOpenChange={(details) => (showDeleteDialog = details.open)} closeOnInteractOutside={false}>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex justify-center items-center p-4">
			<Dialog.Content class="card bg-surface-100-900 w-full max-w-md p-6 space-y-4 shadow-xl 
			                       transition transition-discrete opacity-0 translate-y-[100px] 
			                       starting:data-[state=open]:opacity-0 starting:data-[state=open]:translate-y-[100px] 
			                       data-[state=open]:opacity-100 data-[state=open]:translate-y-0">
				<header class="flex items-start justify-between gap-4">
					<div class="flex items-center gap-3">
						<div class="flex items-center justify-center w-12 h-12 rounded-full bg-error-100 dark:bg-error-900">
							<Trash2 size={24} class="text-error-600 dark:text-error-400" />
						</div>
						<div>
							<Dialog.Title class="text-lg font-semibold text-surface-900 dark:text-surface-100">
								确认删除
							</Dialog.Title>
						</div>
					</div>
					<Dialog.CloseTrigger class="btn-icon btn-icon-sm hover:preset-tonal-surface">
						<X size={16} />
					</Dialog.CloseTrigger>
				</header>
				
				<Dialog.Description class="text-surface-600 dark:text-surface-400 leading-relaxed">
					{#if itemToDelete}
						<p>您确定要删除 <strong class="font-mono text-primary-700 dark:text-primary-300">{itemToDelete.descriptor.id}#{itemToDelete.descriptor.revision.slice(0, 8)}</strong> 吗？</p>
						<p class="text-sm text-surface-500 dark:text-surface-400 mt-2">此操作不可撤销。删除后无法恢复。</p>
					{:else}
						<p>没有选择要删除的项目。</p>
					{/if}
				</Dialog.Description>
				
				<footer class="flex justify-end gap-3 pt-2">
					<button
						type="button"
						class="btn preset-tonal-surface"
						onclick={handleDeleteCancel}
						disabled={deleteLoading}
					>
						取消
					</button>
					<button
						type="button"
						class="btn preset-tonal-error"
						onclick={handleDeleteConfirm}
						disabled={deleteLoading}
					>
						{#if deleteLoading}
							<Loader2 class="mr-2 animate-spin" size={14} />
						{/if}
						确认删除
					</button>
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>