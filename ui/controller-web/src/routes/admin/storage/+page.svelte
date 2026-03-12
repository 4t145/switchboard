<script lang="ts">
	import ObjectSelector from '$lib/components/object-selector.svelte';
	import type { ObjectFilter } from '$lib/api/routes/storage';

	let filter = $state<ObjectFilter>({
		latest_only: true
	});
</script>

<div class="flex flex-col gap-4 card p-6">
	<h2 class="h2">Strorage</h2>
	<ObjectSelector pageSize={12} {filter} selectionMode="none" showFilters={true} showDelete={true} />
</div>
