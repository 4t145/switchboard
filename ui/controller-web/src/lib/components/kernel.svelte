<script lang="ts" module>
    import type { KernelInfo, KernelInfoAndState, KernelConnectionAndState } from '$lib/api/types';
    import  {Button} from 'bits-ui';
    import KernelStateLabel from './kernel-state-label.svelte';
    import { Info } from 'lucide-svelte';
    interface Props {
        addr: string;
        kernel: KernelConnectionAndState;
    } 

</script>
<script lang="ts">
    const props: Props = $props();
    const { kernel, addr } = props;



</script>

<div class="kernel-item p-4 border-b last:border-0 hover:bg-gray-50">
    {#if kernel.connection === 'connected'}

    <div class="flex justify-between items-center mb-2">
        <div class="font-medium text-lg">
            {kernel.state.info.name}
            <span class ="text-sm text-gray-500">
                {addr}
            </span>
        </div>
        
        <div class="text-sm text-gray-500">

            <Button.Root ><Info/></Button.Root>
        </div>
    </div>
    <div class="text-sm text-gray-500">
        <KernelStateLabel {...kernel.state.state} />
    </div>
    {:else}
    <div class="flex justify-between items-center mb-2">
        <div class="font-medium text-lg">{addr}</div>
        <div class="text-sm text-red-500">未连接</div>
    </div>
    {/if}
</div>

<style>

</style>
