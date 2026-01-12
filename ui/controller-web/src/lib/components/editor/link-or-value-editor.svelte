<script lang="ts">
    import { api } from "$lib/api/routes";
    import { Link, PenSquare, Save, X, ExternalLink, ArrowRightLeft, Upload } from 'lucide-svelte';
    import ObjectPages from '$lib/components/object-pages.svelte';
    import type { StorageObjectDescriptor } from '$lib/api/types';
    import { shortRev } from '$lib/utils';
    
    // Define Generic Type LinkOrValue<T> locally for props usage or import it
    // But Svelte components are not generic in script lang="ts" easily in Svelte 4/5 runes yet without generics attribute which is experimental/new.
    // We will treat value as `any` or cast it inside.
    
    type LinkType = { $link: StorageObjectDescriptor };
    type ValueType = any;
    type LinkOrValue = LinkType | ValueType;

    let { 
        value = $bindable(), 
        dataType,
        renderValue, // Snippet or Component for editing the value
        defaultValue // Factory function to create a default value
    } = $props<{
        value: LinkOrValue;
        dataType: string;
        renderValue: any; // RenderSnippet
        defaultValue: () => any;
    }>();

    // Determine mode
    let isLink = $derived(value && typeof value === 'object' && '$link' in value);
    
    // Editing State
    let isSelectingLink = $state(false);
    let isPromoting = $state(false); // Saving value as object
    let promoteId = $state('');
    let loading = $state(false);

    // Actions
    function switchToLinkSelection() {
        isSelectingLink = true;
    }

    function cancelLinkSelection() {
        isSelectingLink = false;
    }

    function selectLink(item: any) {
        value = { $link: item.descriptor };
        isSelectingLink = false;
    }

    async function switchToInline() {
        if (!isLink) return;
        loading = true;
        try {
            const descriptor = (value as LinkType).$link;
            const response = await api.storage.get(descriptor);
            // Assuming response is the object data
            value = response; 
        } catch (e) {
            console.error("Failed to fetch linked object", e);
            alert("Failed to load object content: " + e);
        } finally {
            loading = false;
        }
    }

    function startPromote() {
        isPromoting = true;
        promoteId = '';
    }

    async function confirmPromote() {
        if (!promoteId) return;
        loading = true;
        try {
            const req = {
                resolver: 'static', 
                config: value,
                save_as: promoteId ? promoteId : undefined
            };
            
            // Wait, storage.save returns StorageObjectDescriptor
            const descriptor = await api.storage.save(req);
            value = { $link: descriptor };
            isPromoting = false;
        } catch (e) {
            console.error("Failed to save object", e);
            alert("Failed to save object: " + e);
        } finally {
            loading = false;
        }
    }

</script>

<div class="card variant-soft-surface p-4 border border-surface-300 dark:border-surface-600 space-y-4">
    <div class="flex justify-between items-center border-b border-surface-300 dark:border-surface-600 pb-2 mb-2">
        <div class="flex items-center gap-2">
            {#if isLink}
                <Link size={16} class="text-tertiary-500" />
                <span class="font-bold text-tertiary-600 dark:text-tertiary-400">Reference (Link)</span>
            {:else}
                <PenSquare size={16} class="text-secondary-500" />
                <span class="font-bold text-secondary-600 dark:text-secondary-400">Inline Value</span>
            {/if}
        </div>
        
        <div class="flex gap-2">
            {#if isLink}
                <button class="btn btn-sm variant-filled-secondary" onclick={switchToInline} disabled={loading}>
                    <ArrowRightLeft size={14} class="mr-1" /> Convert to Inline
                </button>
            {:else}
                <button class="btn btn-sm variant-filled-tertiary" onclick={startPromote} disabled={loading}>
                    <Upload size={14} class="mr-1" /> Save as Link
                </button>
            {/if}
            {#if !isLink}
                 <button class="btn btn-sm variant-ghost-surface" onclick={switchToLinkSelection}>
                    Switch to Reference
                 </button>
            {/if}
        </div>
    </div>

    <!-- Content Area -->
    {#if isPromoting}
        <div class="alert variant-filled-surface flex flex-col gap-4 animate-fade-in">
            <div class="flex justify-between items-center">
                <span class="font-bold">Save current value as shared object</span>
                <button class="btn-icon btn-icon-sm" onclick={() => isPromoting = false}><X size={16} /></button>
            </div>
            <label class="label">
                <span>New Object ID</span>
                <input class="input" type="text" bind:value={promoteId} placeholder="e.g. my-shared-config" />
            </label>
            <div class="flex justify-end gap-2">
                <button class="btn btn-sm variant-ghost" onclick={() => isPromoting = false}>Cancel</button>
                <button class="btn btn-sm variant-filled-primary" onclick={confirmPromote} disabled={!promoteId || loading}>
                    Save & Link
                </button>
            </div>
        </div>
    {:else if isSelectingLink}
         <div class="flex flex-col gap-4 animate-fade-in">
            <div class="flex justify-between items-center">
                <span class="font-bold">Select {dataType} from storage</span>
                <button class="btn-icon btn-icon-sm" onclick={cancelLinkSelection}><X size={16} /></button>
            </div>
            <div class="h-64 overflow-y-auto border border-surface-300 rounded p-2 bg-surface-50 dark:bg-surface-900">
                <ObjectPages 
                    pageSize={10} 
                    filter={{ data_type: dataType, latest_only: true }} 
                    selectionMode="single"
                    onSelect={selectLink}
                />
            </div>
         </div>
    {:else if isLink}
        <!-- Link View -->
        <div class="flex items-center gap-4 p-4 bg-surface-200 dark:bg-surface-800 rounded-lg">
            <div class="flex-1">
                <div class="text-sm font-bold opacity-75">ID</div>
                <div class="text-lg font-mono">{(value as LinkType).$link.id}</div>
            </div>
            <div class="flex-1">
                <div class="text-sm font-bold opacity-75">Revision</div>
                <div class="text-lg font-mono">{shortRev((value as LinkType).$link.revision)}</div>
            </div>
            <button class="btn btn-sm variant-ghost-primary" onclick={switchToLinkSelection}>
                Change Link
            </button>
        </div>
    {:else}
        <!-- Value Editor -->
        <!-- Render the Snippet passed as prop -->
        {@render renderValue()}
    {/if}
</div>
