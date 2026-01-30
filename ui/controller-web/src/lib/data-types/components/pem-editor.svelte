<script lang="ts">
	import type { DataTypeComponentProps } from '../registry';
	import PemInput from '$lib/components/common/pem-input.svelte';

	let {
		mode = 'edit',
		value = $bindable(''),
		disabled = false,
		label = 'PEM Certificate/Key'
	}: DataTypeComponentProps<string, { label?: string }> = $props();
</script>

{#if mode === 'edit'}
	<!-- 复用现有的 PEM 输入组件 -->
	<PemInput bind:value {label} required={false} />
{:else}
	<!-- View 模式：只读显示 -->
	<div class="card bg-surface-100 p-4 dark:bg-surface-800">
		<pre class="overflow-x-auto font-mono text-xs">{value || '(Empty)'}</pre>
	</div>
{/if}
