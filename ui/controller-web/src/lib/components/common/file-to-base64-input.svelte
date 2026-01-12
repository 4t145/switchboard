<script lang="ts">
    import { Upload, X, FileText } from 'lucide-svelte';

    let { 
        value = $bindable(''),
        label = 'Upload File',
        accept = '*',
        helperText = ''
    } = $props<{
        value: string;
        label?: string;
        accept?: string;
        helperText?: string;
    }>();

    let fileName = $state<string | null>(null);

    // Simple heuristic to detect if current value is likely Base64 content vs empty
    $effect(() => {
        if (value && !fileName) {
            fileName = "Existing Content (Base64)";
        } else if (!value) {
            fileName = null;
        }
    });

    function handleFileSelect(event: Event) {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        if (!file) return;

        fileName = file.name;
        const reader = new FileReader();
        reader.onload = (e) => {
            const result = e.target?.result;
            if (typeof result === 'string') {
                // Extract base64 part if it's a data URL, or just take the content?
                // FileReader.readAsDataURL returns "data:application/octet-stream;base64,....."
                // We usually just want the base64 part.
                const base64 = result.split(',')[1];
                value = base64;
            }
        };
        reader.readAsDataURL(file);
    }

    function clear() {
        value = '';
        fileName = null;
    }
</script>

<div class="form-control w-full max-w-xs">
    <label class="label">
        <span class="label-text">{label}</span>
    </label>
    
    {#if !value}
        <input 
            type="file" 
            class="file-input file-input-bordered w-full" 
            {accept}
            onchange={handleFileSelect} 
        />
        {#if helperText}
            <div class="label">
                <span class="label-text-alt">{helperText}</span>
            </div>
        {/if}
    {:else}
        <div class="flex items-center gap-2 p-2 border border-surface-300 dark:border-surface-600 rounded-lg bg-surface-50 dark:bg-surface-800">
            <FileText size={20} class="text-primary-500" />
            <span class="flex-1 truncate text-sm font-medium">{fileName}</span>
            <button class="btn-icon btn-icon-sm hover:variant-soft-error" onclick={clear} title="Clear">
                <X size={16} />
            </button>
        </div>
    {/if}
</div>
