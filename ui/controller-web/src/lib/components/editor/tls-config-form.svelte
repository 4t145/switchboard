<script lang="ts">
    import LinkOrValueEditor from './link-or-value-editor.svelte';
    import TlsResolverForm from './tls-resolver-form.svelte';
    import { Trash2 } from 'lucide-svelte';
    import type { Tls } from '$lib/api/types';

    // Tls Config Editor
    // Tls object has: { resolver: LinkOrValue<TlsResolver>, options: TlsOptions }
    
    let { 
        value = $bindable(),
        key,
        onKeyChange
    } = $props<{ 
        value: Tls,
        key: string,
        onKeyChange: (newKey: string) => void
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
    
    // Default Resolver
    if (!value.resolver) {
        value.resolver = { Single: { certs: [], key: '', ocsp: null } } as any;
    }

</script>

{#snippet resolverSnippet()}
    <!-- The inner form for TlsResolver -->
    <!-- We need to bind to the value passed by LinkOrValueEditor (which is the resolver object) -->
    <!-- Wait, LinkOrValueEditor renders this snippet passing nothing, 
         it expects the snippet to bind to the `value` prop passed to LinkOrValueEditor?
         No, LinkOrValueEditor has `value` prop which IS the LinkOrValue.
         When it is in 'Value' mode, `value` is the TlsResolver object.
         So we need to bind that object to TlsResolverForm.
    -->
    
    <!-- Issue: LinkOrValueEditor generic handling.
         In `LinkOrValueEditor`, when mode is Value, `value` is the raw object.
         We need to pass that raw object to `TlsResolverForm`.
         But `value` in `LinkOrValueEditor` is bound. 
         
         The `renderValue` snippet in `LinkOrValueEditor` is called without args currently.
         Ideally, `LinkOrValueEditor` should yield the value to the snippet?
         Let's update `LinkOrValueEditor` to yield value if possible, or we just rely on parent context?
         
         Actually, `value.resolver` in THIS component IS the value we are editing.
         So if `LinkOrValueEditor` binds to `value.resolver`, then `value.resolver` updates.
         If it is a Link, `value.resolver` is `{ $link: ... }`.
         If it is Value, `value.resolver` is `{ Single: ... }`.
         
         So `TlsResolverForm` needs to bind to `value.resolver`.
         BUT, `TlsResolverForm` expects `{ Single: ... }`, NOT `{ $link: ... }`.
         It only renders when `LinkOrValueEditor` determines it is NOT a link.
         
         So, we can simply bind `value={value.resolver}` to `TlsResolverForm`.
         Svelte 5 reactivity should handle the type change gracefully?
         Typescript will complain that `value.resolver` (LinkOrValue) is not assignable to `TlsResolver` (Value).
         
         We can cast it: `value={value.resolver as TlsResolver}`.
         Runtime safety is guaranteed by `LinkOrValueEditor` only rendering this when it IS a value.
    -->
    <TlsResolverForm bind:value={value.resolver as any} />
{/snippet}

<div class="space-y-6">
    <div class="card p-4 border border-surface-200 dark:border-surface-700">
        <label class="label mb-4">
            <span class="label-text font-bold">Key (Unique ID)</span>
            <input 
                class="input" 
                type="text" 
                value={key}
                onchange={(e) => onKeyChange(e.currentTarget.value)}
                placeholder="tls-1" 
            />
        </label>

        <h3 class="h3 mb-4 font-bold">Certificate Resolver</h3>
        <LinkOrValueEditor 
            bind:value={value.resolver} 
            dataType="TlsResolver"
            renderValue={resolverSnippet}
            defaultValue={() => ({ Single: { certs: [], key: '', ocsp: null } })}
        />
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
