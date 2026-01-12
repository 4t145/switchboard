<script lang="ts">
    import type { Listener } from '$lib/api/types';

    let { 
        value = $bindable(),
        key,
        onKeyChange
    } = $props<{ 
        value: Listener,
        key: string,
        onKeyChange: (newKey: string) => void
    }>();
</script>

<div class="space-y-4 p-4">
    <label class="label">
        <span class="label-text">Bind Address (Key)</span>
        <input 
            class="input" 
            type="text"
            value={key}
            onchange={(e) => {
                value.bind = e.currentTarget.value;
                onKeyChange(e.currentTarget.value);
            }}
            placeholder="0.0.0.0:8080" 
        />
        <p class="text-sm opacity-70 mt-1">For listeners, the key is usually the bind address.</p>
    </label>

    <label class="label">
        <span class="label-text">Description</span>
        <textarea class="textarea textarea-bordered h-24" bind:value={value.description} placeholder="Optional description..."></textarea>
    </label>
</div>
