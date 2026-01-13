<script lang="ts" module>
    import type { StorageObjectWithoutData } from '$lib/api/types';
    import type { ObjectFilter } from '$lib/api/routes/storage';

    interface Props {
        pageSize: number;
        filter: ObjectFilter & {
            lockedFields?: string[];
        };
        onSelect?: (item: StorageObjectWithoutData) => void;
        selectedId?: string | null;
        selectionMode?: 'none' | 'single';
        showFilters?: boolean; 
    } 
</script>

<script lang="ts">
	import { api } from "$lib/api/routes";
	import { shortRev } from '$lib/utils';
    import { Info, Loader2, Check, Search, Copy } from 'lucide-svelte';
    import ObjectFilterForm from '$lib/components/object-filter.svelte';

    // Props
    let { 
        pageSize, 
        filter: initialFilter,
        onSelect,
        selectedId = null,
        selectionMode = 'none',
        showFilters = true
    }: Props = $props();

    // Internal State
    let items = $state<StorageObjectWithoutData[]>([]);
    let cursor = $state<string | null>(null);
    let loading = $state(false);
    let error = $state<Error | null>(null);
    let hasMore = $state(true);
    
    // Filter State
    let currentFilter = $state<ObjectFilter>({ ...initialFilter });
    let filterKey = $state<string | null>(null);

    // --- Data Loading ---

    async function loadNextPage() {
        if (loading || (!hasMore && cursor === null) || error) return;
        
        loading = true;
        error = null;

        try {
            const response = await api.storage.list({
                cursor: { next: cursor },
                limit: pageSize
            }, currentFilter);
            
            const newItems = response.items.map(item => item.data);
            items = [...items, ...newItems];
            cursor = response.next_cursor?.next ?? null;
            
            hasMore = cursor !== null;
            
        } catch (e) {
            error = e instanceof Error ? e : new Error(String(e));
            console.error("Failed to load storage objects", e);
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
        if (selectionMode === 'single' && onSelect) {
            onSelect(item);
        }
    }

    function copyToClipboard(e: Event, text: string) {
        e.stopPropagation(); // Prevent card selection
        navigator.clipboard.writeText(text);
        // Ideally show toast
    }
</script>

<div class="space-y-4">
    <!-- Filter Controls (Reusing ObjectFilterForm) -->
    {#if showFilters}
        <div class="border-b border-surface-200 dark:border-surface-700 pb-4">
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

    <!-- Data Grid -->
    <div class="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        {#each items as item (item.descriptor.id + item.descriptor.revision)}
            <!-- 
                Clickable Card using div instead of button to avoid nesting issues
            -->
            <div 
                class="card p-4 text-left border rounded-lg transition-all duration-200 relative group
                       {selectionMode === 'single' ? 'cursor-pointer' : 'cursor-default'}
                       {selectedId === item.descriptor.id && selectionMode === 'single' 
                            ? 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-900/20 border-primary-500' 
                            : 'border-surface-200 dark:border-surface-700 hover:shadow-md hover:border-primary-400'}"
                onclick={() => handleSelect(item)}
                onkeydown={(e) => e.key === 'Enter' && handleSelect(item)}
                role={selectionMode === 'single' ? 'button' : undefined}
                tabindex={selectionMode === 'single' ? 0 : -1}
            >
                <div class="flex justify-between items-start mb-2 overflow-hidden">
                    <div class="flex-1 min-w-0 pr-2 flex items-center gap-2 group/title">
                        <h3 class="font-semibold text-lg truncate" title={item.descriptor.id}>
                            {item.descriptor.id}
                        </h3>
                        <button 
                            class="opacity-0 group-hover/title:opacity-100 transition-opacity p-1 hover:bg-surface-200 dark:hover:bg-surface-700 rounded text-surface-500"
                            onclick={(e) => copyToClipboard(e, item.descriptor.id)}
                            title="Copy ID"
                        >
                            <Copy size={14} />
                        </button>
                    </div>
                    <div class="flex items-center gap-1 group/rev flex-shrink-0">
                        <span class="badge variant-soft-surface text-xs whitespace-nowrap">
                            #{shortRev(item.descriptor.revision)}
                        </span>
                         <button 
                            class="opacity-0 group-hover/rev:opacity-100 transition-opacity p-1 hover:bg-surface-200 dark:hover:bg-surface-700 rounded text-surface-500"
                            onclick={(e) => copyToClipboard(e, item.descriptor.revision)}
                            title="Copy Revision"
                        >
                            <Copy size={14} />
                        </button>
                    </div>
                </div>
                
                <div class="flex flex-col gap-1 text-xs text-surface-500">
                    <div class="flex justify-between">
                        <span>Type:</span>
                        <span class="font-medium text-surface-700 dark:text-surface-300">{item.meta.data_type}</span>
                    </div>
                    <div class="flex justify-between">
                        <span>Created:</span>
                        <span>{new Date(item.meta.created_at).toLocaleString()}</span>
                    </div>
                </div>

                <!-- Selection Indicator -->
                {#if selectedId === item.descriptor.id && selectionMode === 'single'}
                    <div class="absolute -top-2 -right-2 bg-primary-500 text-white rounded-full p-1 shadow-sm animate-scale-in">
                        <Check size={12} strokeWidth={3} />
                    </div>
                {/if}
            </div>
        {/each}
    </div>

    <!-- Error State -->
    {#if error}
        <div class="alert variant-soft-error flex items-center gap-2">
            <Info size={16} />
            <span class="flex-1">Error: {error.message}</span>
            <button class="btn btn-sm variant-filled-error" onclick={reload}>Retry</button>
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
            <div class="flex flex-col items-center gap-2 text-surface-400 py-8">
                <Info size={32} />
                <p>No items found.</p>
            </div>
        {:else if hasMore}
             <button class="btn variant-soft-surface btn-sm" onclick={loadNextPage} disabled={loading}>
                {#if loading}
                    <Loader2 class="animate-spin mr-2" size={14} /> Loading...
                {:else}
                    Load More
                {/if}
             </button>
        {:else if items.length > 0}
            <span class="text-xs text-surface-400">End of list</span>
        {/if}
    </div>
</div>
