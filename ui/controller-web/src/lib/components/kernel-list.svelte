<script lang="ts" module>
	import type { KernelInfo, KernelInfoAndState, KernelConnectionAndState } from '$lib/api/types';
	import { api } from '$lib/api/routes';
	interface Props {
		// kernels: Record<string, KernelConnectionAndState>;
	}
</script>

<script lang="ts">
	// let props: Props = $props();
</script>

<div class="kernel-list overflow-hidden rounded-md border bg-white shadow-sm">
	{#await api.kernelManager.listKernels()}
		<div>Loading...</div>
	{:then kernels}
		{#each Object.entries(kernels) as [addr, kernel]}{/each}
	{/await}
</div>

<style>
</style>
