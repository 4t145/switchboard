<script lang="ts" module>
	import type { KernelState } from '$lib/api/types';
	import {} from 'bits-ui';
	import { AlertCircle, CheckCircle, Loader2, Loader } from 'lucide-svelte';
</script>

<script lang="ts">
	const props: KernelState = $props();
	const { kind, since } = props;
	const fmtTime = (v?: string) => (v ? new Date(v).toLocaleString() : '—');
</script>

<div
	class="inline-block rounded px-2 py-1 text-sm font-medium
    {kind === 'waiting_config' ? 'bg-red-100 text-red-800' : ''}
    {kind === 'running' ? 'bg-green-100 text-green-800' : ''}
    {kind === 'updating' ? 'bg-yellow-100 text-yellow-800' : ''}
    {kind === 'shutting_down' ? 'bg-red-100 text-red-800' : ''}
    {kind === 'stopped' ? 'bg-red-100 text-red-800' : ''}"
>
	<div class="flex items-center gap-1">
		{#if kind === 'waiting_config'}
			等待配置<Loader class="h-4 w-4" />
		{:else if kind === 'running'}
			<CheckCircle class="h-4 w-4" />
			运行中
		{:else if kind === 'updating'}
			<Loader class="h-4 w-4 animate-spin" />
			更新中
		{:else if kind === 'shutting_down'}
			<AlertCircle class="h-4 w-4" />
			关闭中
		{:else if kind === 'stopped'}
			<AlertCircle class="h-4 w-4" />
			已停止
		{/if}
		<div class="text-xs text-gray-500">自 {fmtTime(since)}</div>
	</div>
</div>

<style>
</style>
