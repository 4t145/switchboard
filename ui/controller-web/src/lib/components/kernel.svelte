<script lang="ts" module>
	import type { KernelInfo, KernelInfoAndState, KernelConnectionAndState } from '$lib/api/types';
	import { Button } from 'bits-ui';
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

<div class="kernel-item border-b p-4 last:border-0 hover:bg-gray-50">
	{#if kernel.connection === 'connected'}
		<div class="mb-2 flex items-center justify-between">
			<div class="text-lg font-medium">
				{kernel.state.info.name}
				<span class="text-sm text-gray-500">
					{addr}
				</span>
			</div>

			<div class="text-sm text-gray-500">
				<Button.Root><Info /></Button.Root>
			</div>
		</div>
		<div class="text-sm text-gray-500">
			<KernelStateLabel {...kernel.state.state} />
		</div>
	{:else}
		<div class="mb-2 flex items-center justify-between">
			<div class="text-lg font-medium">{addr}</div>
			<div class="text-sm text-red-500">未连接</div>
		</div>
	{/if}
</div>

<style>
</style>
