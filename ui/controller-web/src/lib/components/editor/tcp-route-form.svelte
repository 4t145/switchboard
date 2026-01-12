<script lang="ts">
    import type { TcpRoute } from '$lib/api/types';
    
    let { 
        value = $bindable(),
        key,
        onKeyChange,
        listenerKeys = [],
        serviceKeys = [],
        tlsKeys = []
    } = $props<{ 
        value: TcpRoute,
        key: string,
        onKeyChange: (newKey: string) => void,
        listenerKeys: string[],
        serviceKeys: string[],
        tlsKeys: string[]
    }>();
</script>

<div class="space-y-4 p-4">
    <label class="label">
        <span class="label-text">Bind Listener (Key)</span>
        <select 
            class="select" 
            value={key} 
            onchange={(e) => {
                value.bind = e.currentTarget.value;
                onKeyChange(e.currentTarget.value);
            }}
        >
            <option value="" disabled>Select a listener...</option>
            {#each listenerKeys as lKey}
                <option value={lKey}>{lKey}</option>
            {/each}
        </select>
        <p class="text-sm opacity-70 mt-1">For routes, the key is the listener bind address.</p>
    </label>

    <label class="label">
        <span class="label-text">Service</span>
        <select class="select" bind:value={value.service}>
            <option value="" disabled>Select a service...</option>
            {#each serviceKeys as key}
                <option value={key}>{key}</option>
            {/each}
        </select>
        <p class="text-sm opacity-70 mt-1">Select the backend service to route traffic to.</p>
    </label>

    <label class="label">
        <span class="label-text">TLS Configuration (Optional)</span>
        <select class="select" bind:value={value.tls}>
            <option value="">None</option>
            {#each tlsKeys as key}
                <option value={key}>{key}</option>
            {/each}
        </select>
        <p class="text-sm opacity-70 mt-1">Select a TLS configuration if encryption is required.</p>
    </label>
</div>
