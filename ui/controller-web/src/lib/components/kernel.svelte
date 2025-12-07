<script lang="ts" module>
    import type { KernelInfo, KernelInfoAndState, KernelConnectionAndState } from '$lib/api/types';
    import  {} from 'bits-ui';
    interface Props {
        addr: string;
        kernel: KernelConnectionAndState;
    } 

</script>
<script lang="ts">
    const props: Props = $props();
    const { kernel, addr } = props;


  const fmtTime = (v?: string) =>
    v ? new Date(v).toLocaleString() : '—';
</script>

<div class="kernel-item p-4 border-b last:border-0 hover:bg-gray-50">
    {#if kernel.connection === 'Connected'}

    <div class="flex justify-between items-center mb-2">
        <div class="font-medium text-lg">{kernel.state.info.name}</div>
        <div class="text-sm text-gray-500">ID: {kernel.state.info.id}</div>
    </div>
    <div class="text-sm text-gray-500">
        最后更新: {fmtTime(kernel.state.state.since)}
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
