<script lang="ts">
    import { api } from "$lib/api/routes";
    import { FileText, CloudDownload, PlusSquare, Loader2 } from 'lucide-svelte';
    import ObjectPages from '$lib/components/object-pages.svelte';
    import { shortRev } from "$lib/utils";
    
    let { 
        onNext,
        initialMode = 'select',
        initialResolver = 'fs',
        initialFsPath = ''
    } = $props<{ 
        onNext: (config: Record<string, any>, summary: string, saveAs?: string) => void;
        initialMode?: 'select' | 'load-saved' | 'from-source';
        initialResolver?: 'k8s' | 'fs';
        initialFsPath?: string;
    }>();

    let mode = $state(initialMode);
    let isLoading = $state(false);
    
    // --- Load Saved State ---
    let selectedConfigId = $state<string | null>(null);
    let selectedItemDescriptor = $state<any>(null); // To store full descriptor for loading

    // --- From Source State ---
    let resolver = $state(initialResolver);
    let fsPath = $state(initialFsPath);
    let saveAs = $state<string>(''); // For from-source save_as parameter

    // --- Actions ---

    function handleCreateNew() {
        // Initialize an empty structure for ServiceConfig
        onNext({
            tcp_services: [],
            tls: []
        }, "New Empty Configuration");
    }

    function startLoadSaved() {
        mode = 'load-saved';
        // Reset selection
        selectedConfigId = null;
        selectedItemDescriptor = null;
    }

    async function confirmLoadSaved() {
        if (!selectedItemDescriptor) return;

        isLoading = true;
        try {
            const response = await api.storage.get(selectedItemDescriptor);
            onNext(
                response as Record<string, any>, 
                `Loaded: ${selectedItemDescriptor.id} (rev ${shortRev(selectedItemDescriptor.revision)})`
            );
        } catch (e) {
            console.error("Failed to fetch config details", e);
        } finally {
            isLoading = false;
        }
    }

    function startFromSource() {
        mode = 'from-source';
    }

    async function confirmFromSource() {
        isLoading = true;
        try {
            const req = {
                resolver,
                config: resolver === 'fs' ? { path: fsPath } : {}, 
                save_as: saveAs || undefined
            };
            
            const response = await api.resolve.service_config(req);
            const summary = resolver === 'fs' ? `Source: File (${fsPath})` : `Source: Kubernetes`;
            onNext(response.config, summary, saveAs || undefined);
        } catch (e) {
            console.error("Failed to resolve config", e);
            alert("Failed to resolve configuration: " + e);
        } finally {
            isLoading = false;
        }
    }

    function goBack() {
        mode = 'select';
        selectedConfigId = null;
    }
</script>

{#if mode === 'select'}
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Create New -->
        <button class="card card-hover p-6 flex flex-col items-center text-center space-y-4 cursor-pointer hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors border border-surface-200 dark:border-surface-700" onclick={handleCreateNew}>
            <div class="p-4 rounded-full bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300">
                <PlusSquare size={48} />
            </div>
            <div>
                <h3 class="h3 font-bold">Create New</h3>
                <p class="opacity-75 mt-2">Start with a fresh, empty configuration.</p>
            </div>
        </button>

        <!-- Load Saved -->
        <button class="card card-hover p-6 flex flex-col items-center text-center space-y-4 cursor-pointer hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors border border-surface-200 dark:border-surface-700" onclick={startLoadSaved}>
            <div class="p-4 rounded-full bg-secondary-100 dark:bg-secondary-900 text-secondary-600 dark:text-secondary-300">
                <FileText size={48} />
            </div>
            <div>
                <h3 class="h3 font-bold">Load Saved</h3>
                <p class="opacity-75 mt-2">Load a previously saved configuration file.</p>
            </div>
        </button>

        <!-- From Source -->
        <button class="card card-hover p-6 flex flex-col items-center text-center space-y-4 cursor-pointer hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors border border-surface-200 dark:border-surface-700" onclick={startFromSource}>
            <div class="p-4 rounded-full bg-tertiary-100 dark:bg-tertiary-900 text-tertiary-600 dark:text-tertiary-300">
                <CloudDownload size={48} />
            </div>
            <div>
                <h3 class="h3 font-bold">From Source</h3>
                <p class="opacity-75 mt-2">Import configuration from an external source.</p>
            </div>
        </button>
    </div>

{:else if mode === 'load-saved'}
    <div class="space-y-4 animate-fade-in">
        <div class="flex items-center justify-between">
            <h4 class="h4">Select Saved Configuration</h4>
            <div class="flex gap-2">
                 <button class="btn variant-ghost-surface btn-sm" onclick={goBack}>Back</button>
                 <button class="btn variant-filled-secondary btn-sm" disabled={!selectedConfigId || isLoading} onclick={confirmLoadSaved}>
                    {#if isLoading} <Loader2 class="animate-spin mr-2" size={16} /> {/if}
                    Load Selected
                </button>
            </div>
        </div>
        
        <!-- Reusing ObjectPages Component -->
        <div class="h-[500px] overflow-y-auto border border-surface-200 dark:border-surface-700 rounded-lg p-2">
            <ObjectPages 
                pageSize={12}
                filter={{ data_type: 'ServiceConfig', latest_only: true, lockedFields: ['dataType'] }}
                selectionMode="single"
                selectedId={selectedConfigId}
                onSelect={(item) => {
                    selectedConfigId = item.descriptor.id;
                    selectedItemDescriptor = item.descriptor;
                }}
            />
        </div>
    </div>

{:else if mode === 'from-source'}
    <div class="space-y-4 animate-fade-in">
        <h4 class="h4">Configure Source</h4>
        
        <label class="label">
            <span>Resolver Type</span>
            <select class="select" bind:value={resolver}>
                <option value="fs">Filesystem (Server Side)</option>
                <option value="k8s">Kubernetes Cluster</option>
            </select>
        </label>

        {#if resolver === 'fs'}
            <label class="label">
                <span>Config File Path</span>
                <input class="input" type="text" bind:value={fsPath} placeholder="/path/to/config.yaml" />
                <p class="text-sm opacity-75">Absolute path on the server where the controller is running.</p>
            </label>
        {:else}
            <div class="alert variant-soft-info">
                Will attempt to load configuration from the Kubernetes cluster currently configured in the controller's environment.
            </div>
        {/if}

        <label class="label">
             <span>Save As (Optional)</span>
             <input class="input" type="text" bind:value={saveAs} placeholder="e.g. my-imported-config" />
             <p class="text-sm opacity-75">If provided, the resolved configuration will be saved to storage with this ID.</p>
        </label>

        <div class="flex justify-end gap-2 pt-4 border-t border-surface-200 dark:border-surface-700">
            <button class="btn variant-ghost-surface" onclick={goBack}>Back</button>
            <button class="btn variant-filled-tertiary" disabled={(resolver === 'fs' && !fsPath) || isLoading} onclick={confirmFromSource}>
                {#if isLoading} <Loader2 class="animate-spin mr-2" size={16} /> {/if}
                Resolve & Build
            </button>
        </div>
    </div>
{/if}
