<script lang="ts">
	import ServiceConfigEditor from '../editor/service-config-editor.svelte';
	import type { HumanReadableServiceConfig } from '$lib/api/types';

	let { config = $bindable() } = $props<{ config: HumanReadableServiceConfig }>();
</script>

<div class="flex h-full w-full flex-col">
	<div class="flex-1 overflow-hidden">
		{#if config}
			<ServiceConfigEditor bind:config />
		{:else}
			<div class="flex h-full items-center justify-center">
				<div class="alert max-w-md preset-tonal-warning">
					No configuration loaded. Go back and select a source.
				</div>
			</div>
		{/if}
	</div>
</div>
