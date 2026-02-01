<script lang="ts">
	import type { HttpClassEditorProps } from "$lib/plugins/types";

	export type StaticResponseConfig = {
		headers: [string, string][];
        status_code: number;
        body: string | null;
	};
	
	type Props = HttpClassEditorProps<StaticResponseConfig>;

	let { value = $bindable(), readonly = false }: Props = $props();

</script>

<div class="space-y-4">
    <!-- Status Code -->
    <label class="label">
        <span class="label-text">Status Code</span>
        <input
            class="input-sm input"
            type="number"
            bind:value={value.status_code}
            placeholder="200"
            disabled={readonly}
        />
    </label>

    <!-- Headers -->
    <label class="label">
        <span class="label-text">Headers</span>
        <div class="space-y-2">
            {#each value.headers as header, index (index)}
                <div class="flex space-x-2">
                    <input
                        class="input-sm input flex-1"
                        type="text"
                        bind:value={header[0]}
                        placeholder="Header Name"
                        disabled={readonly}
                    />
                    <input
                        class="input-sm input flex-1"
                        type="text"
                        bind:value={header[1]}
                        placeholder="Header Value"
                        disabled={readonly}
                    />
                </div>
            {/each}
        </div>
    </label>

    <!-- Body -->
    <label class="label">
        <span class="label-text">Body</span>
        <textarea
            class="textarea-sm textarea w-full"
            bind:value={value.body}
            placeholder="Response Body"
            rows={5}
            disabled={readonly}
        ></textarea>
    </label>
</div>
