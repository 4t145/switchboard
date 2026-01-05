<script lang="ts" module>
    import type { StorageObjectWithoutData, ServiceDescriptor, StorageObjectMeta, StorageObjectDescriptor } from '$lib/api/types';
    import type { ObjectFilter } from '$lib/api/routes/storage';

    import  {Button} from 'bits-ui';
    import KernelStateLabel from './kernel-state-label.svelte';
    import { Info } from 'lucide-svelte';
    interface Props {
        pageSize: number;
        filter: ObjectFilter;
    } 

</script>
<script lang="ts">
	import  { api } from "$lib/api/routes";

    const props: Props = $props();
    const { pageSize, filter } = props;
    let items = $state<StorageObjectWithoutData[]>([]);
    let cursor = $state<string | null>(null);
    let loading = $state(false);
    let error = $state<Error | null>(null);
    let hasMore = $state(true);
    async function loadNextPage() {
        if (loading || !hasMore || error) return;
        
        loading = true;
        error = null;

        try {
            const response = await api.storage.list({
                cursor: {
                    next: cursor
                },
                limit: pageSize
            }, filter);
            items = [...items, ...response.items.map(item => item.data)];
            cursor = response.next_cursor?.next ?? null;
            hasMore = cursor !== null;
        } catch (e) {
            error = e instanceof Error ? e : new Error(String(e));
            console.error("Failed to load storage objects", e);
        } finally {
            loading = false;
        }
    }
    function reload() {
        items = [];
        cursor = null;
        hasMore = true;
        loadNextPage();
    }
    $effect(() => {
        // 访问 filter 以建立依赖
        filter;
        reload();
    });
    
    // 初始加载
    $effect(() => {
        if (items.length === 0 && !loading && hasMore) {
            loadNextPage();
        }
    });
</script>


<div class="flex flex-col gap-4">
    <!-- 数据展示 -->
    <div class="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        {#each items as item (item.descriptor.id + item.descriptor.revision)}
            <div class="p-4 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                <div class="flex justify-between items-center mb-2">
                    <h3 class="font-semibold text-lg truncate">{item.descriptor.id}</h3>
                    <span class="text-sm text-gray-500 ml-2">v{item.descriptor.revision}</span>
                </div>
                <div class="flex gap-4 text-sm text-gray-600">
                    <span class="font-medium">{item.meta.data_type}</span>
                    <span>{new Date(item.meta.created_at).toLocaleString()}</span>
                </div>
            </div>
        {/each}
    </div>

    <!-- 错误提示 -->
    {#if error}
        <div class="flex items-center gap-2 p-4 bg-red-50 rounded border border-red-200 text-red-700">
            <Info size={16} />
            <span class="flex-1">Error: {error.message}</span>
            <Button.Root onclick={reload} class="ml-auto">重试</Button.Root>
        </div>
    {/if}

    <!-- 加载更多按钮 -->
    {#if hasMore}
        <div class="text-center p-4">
            <Button.Root 
                onclick={loadNextPage} 
                disabled={loading}
                class="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
                {loading ? '加载中...' : '加载更多'}
            </Button.Root>
        </div>
    {:else if items.length > 0}
        <div class="text-center p-4 text-gray-500">没有更多数据了</div>
    {/if}

    <!-- 空状态 -->
    {#if !loading && items.length === 0 && !error}
        <div class="flex flex-col items-center gap-2 p-8 text-gray-500">
            <Info size={24} />
            <p>暂无数据</p>
        </div>
    {/if}
</div>
<!-- {#await api.sto}
    
{:then } 
    
{/await} -->