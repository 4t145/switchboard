<script lang="ts">
    import LinkOrValueEditor from './link-or-value-editor.svelte';
    import TlsResolverForm from './tls-resolver-form.svelte';
    import type { FileStyleTls, FileStyleTlsResolver, TlsCertParams } from '$lib/api/types';

    let { 
        value = $bindable()
    } = $props<{ 
        value: FileStyleTls
    }>();

    // Default TlsOptions
    if (!value.options) {
        value.options = {
            ignore_client_order: false,
            max_fragment_size: null,
            alpn_protocols: ['h2', 'http/1.1'],
            enable_secret_extraction: false,
            max_early_data_size: 0,
            send_half_rtt_data: false,
            send_tls13_tickets: 0,
            require_ems: true
        } as any;
    }
</script>

{#snippet resolverSnippet()}
    <!-- In FileStyle, the resolver is part of the Tls object itself (flattened in Rust) -->
    <!-- So we bind directly to 'value' but treat it as the resolver part -->
    <!-- We need to ensure we only pass the resolver part which is typed correctly for the form -->
    <TlsResolverForm bind:value={value as unknown as { Single?: TlsCertParams; Sni?: Record<string, TlsCertParams> }} />
{/snippet}

<div class="space-y-6">
    <div class="card p-4 border border-surface-200 dark:border-surface-700">
        <label class="label mb-4">
            <span class="label-text font-bold">Key (Unique ID)</span>
            <input 
                class="input" 
                type="text" 
                bind:value={value.name}
                placeholder="tls-1" 
            />
        </label>

        <h3 class="h3 mb-4 font-bold">Certificate Resolver</h3>
        <!-- Since resolver is flattened into 'value', we pass 'value' as the resolver object to TlsResolverForm -->
        <!-- But wait, TlsResolverForm expects { Single: ... } or { Sni: ... } -->
        <!-- Our 'value' (FileStyleTls) has those keys because of the intersection type -->
        <TlsResolverForm bind:value={value as unknown as { Single?: TlsCertParams; Sni?: Record<string, TlsCertParams> }} />
    </div>

    <div class="card p-4 border border-surface-200 dark:border-surface-700">
        <h3 class="h3 mb-4 font-bold">TLS Options</h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
             <label class="flex items-center space-x-2">
                <input class="checkbox" type="checkbox" bind:checked={value.options.ignore_client_order} />
                <span>Ignore Client Order</span>
            </label>
            <label class="flex items-center space-x-2">
                <input class="checkbox" type="checkbox" bind:checked={value.options.require_ems} />
                <span>Require EMS</span>
            </label>
            <label class="flex items-center space-x-2">
                <input class="checkbox" type="checkbox" bind:checked={value.options.enable_secret_extraction} />
                <span>Enable Secret Extraction</span>
            </label>
            <label class="label">
                <span>Max Early Data Size</span>
                <input class="input" type="number" bind:value={value.options.max_early_data_size} />
            </label>
            <label class="label col-span-2">
                 <span>ALPN Protocols (comma separated)</span>
                 <input class="input" type="text" 
                        value={value.options.alpn_protocols.join(', ')} 
                        oninput={(e) => value.options.alpn_protocols = e.currentTarget.value.split(',').map(s => s.trim()).filter(Boolean)}
                 />
            </label>
        </div>
    </div>
</div>
