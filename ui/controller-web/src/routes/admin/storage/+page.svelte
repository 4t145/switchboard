<script lang="ts">
    import ObjectPages from '$lib/components/object-pages.svelte';
    import ObjectFilterForm from '$lib/components/object-filter.svelte';
    import type { ObjectFilter } from '$lib/api/routes/storage';
    
    let filter = $state<ObjectFilter>({
        latest_only: true
    });
    
    let dataTypeFilter = $state('');
    let idFilter = $state('');
    let latestOnly = $state(true);
    
    function applyFilters() {
        filter = {
            ...(dataTypeFilter ? { data_type: dataTypeFilter } : {}),
            ...(idFilter ? { id: idFilter } : {}),
            latest_only: latestOnly
        };
    }
</script>

<div class="card flex flex-col gap-4 p-6">
    <ObjectFilterForm
        bind:dataType={dataTypeFilter}
        bind:id={idFilter}
        bind:latestOnly={latestOnly}
        compact
        onSubmit={applyFilters}
    />
    <hr class="hr" />
    <!-- 对象列表 -->
    <!-- <h2 class="text-lg font-semibold mb-4">对象列表</h2> -->
    <ObjectPages pageSize={12} {filter} selectionMode="none" showFilters={false} />
</div>
