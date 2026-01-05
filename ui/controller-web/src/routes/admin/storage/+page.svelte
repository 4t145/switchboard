<script lang="ts">
    import ObjectPages from '$lib/components/object-pages.svelte';
    import type { ObjectFilter } from '$lib/api/routes/storage';
    import { Button } from 'bits-ui';
    
    let filter = $state<ObjectFilter>({
        latest_only: true
    });
    
    let dataTypeFilter = $state('');
    let idFilter = $state('');
    let showAllVersions = $state(false);
    
    function applyFilters() {
        filter = {
            ...(dataTypeFilter ? { data_type: dataTypeFilter } : {}),
            ...(idFilter ? { id: idFilter } : {}),
            latest_only: !showAllVersions
        };
    }
    
    function resetFilters() {
        dataTypeFilter = '';
        idFilter = '';
        showAllVersions = false;
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
        
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- 数据类型筛选 -->
            <div class="flex flex-col gap-2">
                <label for="dataType" class="text-sm font-medium text-gray-700">数据类型</label>
                <input
                    id="dataType"
                    type="text"
                    bind:value={dataTypeFilter}
                    placeholder="例如: service, interface"
                    class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
            </div>
            
            <!-- ID 筛选 -->
            <div class="flex flex-col gap-2">
                <label for="id" class="text-sm font-medium text-gray-700">对象 ID</label>
                <input
                    id="id"
                    type="text"
                    bind:value={idFilter}
                    placeholder="输入对象 ID"
                    class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
            </div>
            
            <!-- 版本筛选 -->
            <div class="flex flex-col gap-2">
                <label for="versions" class="text-sm font-medium text-gray-700">版本</label>
                <label class="flex items-center gap-2 px-3 py-2 cursor-pointer">
                    <input
                        id="versions"
                        type="checkbox"
                        bind:checked={showAllVersions}
                        class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-2 focus:ring-blue-500"
                    />
                    <span class="text-sm text-gray-700">显示所有版本</span>
                </label>
            </div>
        </div>
        
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
