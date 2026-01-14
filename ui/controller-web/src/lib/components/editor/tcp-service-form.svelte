<script lang="ts">
    import LinkOrValueEditor from './link-or-value-editor.svelte';
    import { Code, Plus, Trash2 } from 'lucide-svelte';
    import type { FileTcpServiceConfig, FileBind } from '$lib/api/types';


    let { 
        value = $bindable(),
        tlsKeys = []
    } = $props<{ 
        value: FileTcpServiceConfig,
        tlsKeys: string[]
    }>();

    // Mock provider list for now
    const providers = ['Static', 'Kubernetes', 'Consul', 'Custom'];

    function addBind() {
        value.binds = [...value.binds, { bind: '', tls: undefined, description: '' }];
    }

    function removeBind(index: number) {
        value.binds = value.binds.filter((_: unknown, i: number) => i !== index);
    }
</script>

{#snippet jsonConfigSnippet()}
    <!-- Simple JSON editor for the generic 'config' field -->
    <div class="form-control">
        <label class="label">
            <span class="label-text">Configuration (JSON)</span>
        </label>
        <textarea 
            class="textarea textarea-bordered font-mono text-xs h-48" 
            value={JSON.stringify(value.config, null, 2)}
            oninput={(e) => {
                try {
                    value.config = JSON.parse(e.currentTarget.value);
                } catch (err) {
                    // Ignore parse errors while typing
                }
            }}
        ></textarea>
        <div class="label">
            <span class="label-text-alt opacity-75">Enter valid JSON configuration for this provider.</span>
        </div>
    </div>
{/snippet}

<div class="space-y-6 p-4">
    <!-- Basic Info -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <label class="label">
            <span class="label-text">Service Name</span>
            <input 
                class="input" 
                type="text" 
                bind:value={value.name}
                placeholder="my-service" 
            />
        </label>

        <label class="label">
            <span class="label-text">Provider</span>
            <select class="select" bind:value={value.provider}>
                {#each providers as p}
                    <option value={p}>{p}</option>
                {/each}
            </select>
        </label>
    </div>

    <label class="label">
        <span class="label-text">Description</span>
        <textarea class="textarea textarea-bordered h-20" bind:value={value.description}></textarea>
    </label>

    <!-- Binds & Routes -->
    <div class="card p-4 border border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900/50">
         <div class="flex justify-between items-center mb-4">
            <h4 class="h4 font-bold">Listeners & Routes</h4>
            <button class="btn btn-sm variant-filled-secondary" onclick={addBind}>
                <Plus size={14} /> Add Bind
            </button>
         </div>

         {#if value.binds.length === 0}
            <div class="text-center opacity-50 text-sm py-4">No listeners configured.</div>
         {/if}

         <div class="space-y-3">
             {#each value.binds as bind, i}
                <div class="card p-3 border border-surface-300 dark:border-surface-600 flex gap-4 items-start">
                    <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
                        <label class="label">
                            <span class="label-text text-xs">Bind Address</span>
                            <input class="input input-sm" type="text" bind:value={bind.bind} placeholder="0.0.0.0:8080" />
                        </label>
                        
                        <label class="label">
                            <span class="label-text text-xs">TLS Config (Optional)</span>
                            <select class="select select-sm" bind:value={bind.tls}>
                                <option value={undefined}>None</option>
                                {#each tlsKeys as key}
                                    <option value={key}>{key}</option>
                                {/each}
                            </select>
                        </label>
                        
                         <label class="label col-span-2">
                            <span class="label-text text-xs">Description</span>
                            <input class="input input-sm" type="text" bind:value={bind.description} placeholder="Description..." />
                        </label>
                    </div>
                    <button class="btn-icon btn-icon-sm variant-soft-error mt-6" onclick={() => removeBind(i)}>
                        <Trash2 size={16} />
                    </button>
                </div>
             {/each}
         </div>
    </div>

    <!-- Provider Config -->
    <div class="card p-4 border border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900/50">
        <h4 class="h4 font-bold mb-2 flex items-center gap-2">
            <Code size={16} /> Provider Config
        </h4>
        <LinkOrValueEditor 
            bind:value={value.config} 
            dataType="TcpServiceConfig" 
            renderValue={jsonConfigSnippet}
            defaultValue={() => ({})}
        />
    </div>
</div>
