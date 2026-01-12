<script lang="ts">
    import LinkOrValueEditor from './link-or-value-editor.svelte';
    import { Code } from 'lucide-svelte';
    import type { TcpService } from '$lib/api/types';

    let { 
        value = $bindable(),
        key,
        onKeyChange
    } = $props<{ 
        value: TcpService,
        key: string,
        onKeyChange: (newKey: string) => void
    }>();

    // Mock provider list for now
    const providers = ['Static', 'Kubernetes', 'Consul', 'Custom'];
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

<div class="space-y-4 p-4">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <label class="label">
            <span class="label-text">Service Name (Key)</span>
            <input 
                class="input" 
                type="text" 
                value={key} 
                onchange={(e) => {
                    value.name = e.currentTarget.value;
                    onKeyChange(e.currentTarget.value);
                }}
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
        <textarea class="textarea textarea-bordered h-20" bind:value={value.description as string}></textarea>
    </label>

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
