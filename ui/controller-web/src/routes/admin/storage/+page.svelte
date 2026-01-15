<script lang="ts">
	import ObjectPages from '$lib/components/object-pages.svelte';
	import ObjectFilterForm from '$lib/components/object-filter.svelte';
	import type { ObjectFilter } from '$lib/api/routes/storage';
	import type { StorageObjectWithoutData } from '$lib/api/types';

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

	function handleEdit(item: StorageObjectWithoutData) {
		// 导航到编辑页面或打开编辑模态框
		console.log('编辑对象:', item.descriptor.id);
		// TODO: 实现编辑功能，可以导航到编辑页面
		// window.location.href = `/admin/storage/edit/${item.descriptor.id}`;
		alert('编辑功能待实现');
	}

	function handleViewDetails(item: StorageObjectWithoutData) {
		// 导航到详情页面或打开详情模态框
		console.log('查看详情:', item.descriptor.id);
		// TODO: 实现查看详情功能
		// window.location.href = `/admin/storage/view/${item.descriptor.id}`;
		alert('查看详情功能待实现');
	}
</script>

<div class="flex flex-col gap-4 card p-6">
	<ObjectFilterForm
		bind:dataType={dataTypeFilter}
		bind:id={idFilter}
		bind:latestOnly
		compact
		onSubmit={applyFilters}
	/>
	<hr class="hr" />
	<!-- 对象列表 -->
	<!-- <h2 class="text-lg font-semibold mb-4">对象列表</h2> -->
	<ObjectPages 
		pageSize={12} 
		{filter} 
		selectionMode="none" 
		showFilters={false}
		onEdit={handleEdit} 
		onViewDetails={handleViewDetails}
		showDelete={true}
	/>
</div>