<script lang="ts">
	import SocketAddrEditor from "$lib/components/editor/socket-addr-editor.svelte";
	import type { ProviderEditorProps } from "$lib/plugins/types";
	import type { PortForwardConfig } from ".";

    type Props = ProviderEditorProps<PortForwardConfig>;
    let { value = $bindable(), onValueChange, readonly = false }: Props = $props();
    $effect(() => {
        onValueChange(value);
    });
</script>

<label class="flex flex-col gap-2" aria-label="Forward To Address" >
    <span class="font-medium">
        Forward To Address
    </span> 
    <SocketAddrEditor
        bind:value={value.to}
        disabled={readonly}
        placeholder="e.g. 192.168.1.1:22"
    />
</label>
