<script lang="ts">
    import ObjectPages from '$lib/components/object-pages.svelte';
    import ObjectFilterForm from '$lib/components/object-filter.svelte';
    import type { ObjectFilter } from '$lib/api/routes/storage';
    import { Button } from 'bits-ui';
    
    let filter = $state<ObjectFilter>({
        latest_only: true
    });
    
    let dataTypeFilter = $state('');
    let idFilter = $state('');
    let latestOnly = $state(false);
    
    function applyFilters() {
        filter = {
            ...(dataTypeFilter ? { data_type: dataTypeFilter } : {}),
            ...(idFilter ? { id: idFilter } : {}),
            latest_only: latestOnly
        };
    }
    
    function resetFilters() {
        dataTypeFilter = '';
        idFilter = '';
        latestOnly = true;
        filter = { latest_only: true };
    }
</script>

<div class="flex flex-col gap-6 p-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold text-gray-900">存储对象浏览</h1>
    </div>
    
    <!-- 筛选器面板 -->
    <div class="bg-white rounded-lg border border-gray-200 p-4">
        <h2 class="text-lg font-semibold mb-4">筛选条件</h2>
        
        <ObjectFilterForm
            bind:dataType={dataTypeFilter}
            bind:id={idFilter}
            bind:latestOnly={latestOnly}
        />
        
        <!-- 操作按钮 -->
        <div class="flex gap-2 mt-4">
            <Button.Root 
                onclick={applyFilters}
                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
                应用筛选
            </Button.Root>
            <Button.Root 
                onclick={resetFilters}
                class="px-4 py-2 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400"
            >
                重置
            </Button.Root>
        </div>
    </div>
    
    <!-- 对象列表 -->
    <div class="bg-white rounded-lg border border-gray-200 p-4">
        <h2 class="text-lg font-semibold mb-4">对象列表</h2>
        <ObjectPages pageSize={12} {filter} />
    </div>
</div>
