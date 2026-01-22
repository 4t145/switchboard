<script lang="ts">
	import type { DataTypeComponentProps } from '../registry';

	let {
		mode = 'edit',
		value = $bindable({}),
		disabled = false,
		readonly = false,
		placeholder = '{}'
	}: DataTypeComponentProps<object> = $props();

	let jsonString = $state('');
	let parseError = $state('');

	// 将 value 同步为格式化的 JSON 字符串
	$effect(() => {
		try {
			jsonString = JSON.stringify(value, null, 2);
		} catch (e) {
			jsonString = String(value);
		}
	});

	// 处理输入，尝试解析 JSON
	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		jsonString = target.value;
		try {
			value = JSON.parse(jsonString);
			parseError = '';
		} catch (e) {
			parseError = e instanceof Error ? e.message : 'Invalid JSON';
		}
	}
</script>

{#if mode === 'edit'}
	<textarea
		class="textarea font-mono text-xs"
		class:input-error={parseError}
		value={jsonString}
		oninput={handleInput}
		{disabled}
		{readonly}
		{placeholder}
		rows={8}
	></textarea>
	{#if parseError}
		<p class="text-error-500 text-sm mt-1">{parseError}</p>
	{/if}
{:else}
	<!-- View 模式：只读显示 -->
	<div class="card p-4 bg-surface-100 dark:bg-surface-800">
		<pre class="text-xs font-mono overflow-x-auto">{JSON.stringify(value, null, 2)}</pre>
	</div>
{/if}
