<script lang="ts">
    import ServiceConfigEditor from '../editor/service-config-editor.svelte';
    import type { HumanReadableServiceConfig } from '$lib/api/types';

    let { config = $bindable() } = $props<{ config: HumanReadableServiceConfig }>();
</script>

<div class="h-full w-full flex flex-col">
    <div class="flex-1 overflow-hidden">
        {#if config}
            <ServiceConfigEditor bind:config={config} />
        {:else}
            <div class="h-full flex items-center justify-center">
                <div class="alert variant-soft-warning max-w-md">
                    No configuration loaded. Go back and select a source.
                </div>
            </div>
        {/if}
    </div>
</div>
