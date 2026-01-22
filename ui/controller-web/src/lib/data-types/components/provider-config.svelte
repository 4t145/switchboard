<script lang="ts">
	import type { DataTypeComponentProps } from '../registry';
	import { providerEditorRegistry } from '$lib/plugins/registry';
	import { AlertCircle } from 'lucide-svelte';

	let {
		mode = 'edit',
		value = $bindable({}),
		provider = ''
	}: DataTypeComponentProps<any> & { provider: string } = $props();

	// 从插件注册表获取对应的编辑器
	const plugin = $derived(providerEditorRegistry.get(provider));

	// 初始化默认值
	$effect(() => {
		const valueEmpty = value === undefined || value === null;
		if (valueEmpty && plugin?.createDefaultConfig) {
			value = plugin.createDefaultConfig();
		}
	});
</script>

{#if mode === 'edit'}
	{#if plugin}
		{@const EditorComponent = plugin.component}
		<!-- 渲染插件提供的编辑器组件 -->
		<div class="plugin-editor">
			<EditorComponent bind:value />
		</div>
	{:else}
		<!-- 降级方案：没有对应插件时使用 JSON 编辑器 -->
		<div class="space-y-3">
			<div class="alert preset-tonal-warning">
				<AlertCircle size={16} class="inline-block" />
				<span class="font-medium">No editor available for provider "{provider}"</span>
			</div>
			<textarea
				class="textarea font-mono text-xs"
				value={JSON.stringify(value, null, 2)}
				oninput={(e) => {
					try {
						value = JSON.parse(e.currentTarget.value);
					} catch (err) {
						// 忽略输入过程中的解析错误
					}
				}}
				rows={8}
			></textarea>
		</div>
	{/if}
{:else}
	<!-- View 模式：只读显示 -->
	<div class="card p-4 bg-surface-100 dark:bg-surface-800">
		<pre class="text-xs font-mono overflow-x-auto">{JSON.stringify(value, null, 2)}</pre>
	</div>
{/if}
