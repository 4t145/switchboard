<script lang="ts">
	import type { DataTypeComponentProps } from '../registry';
	import { httpClassEditorRegistry } from '$lib/plugins/registry';
	import FallbackConfigEditor from '$lib/components/editor/fallback-config-editor.svelte';

	let {
		mode = 'edit',
		value = $bindable({}),
		classId = '',
		instanceType = 'node',
		readonly = false
	}: DataTypeComponentProps<any> & {
		classId: string;
		instanceType?: 'node' | 'filter';
		readonly?: boolean;
	} = $props();

	// 从插件注册表获取对应的编辑器
	const plugin = $derived(httpClassEditorRegistry.get(classId));

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
		<!-- 插件已注册：使用插件编辑器组件 -->
		<div class="space-y-4">
			<!-- 插件信息头部 -->
			<div class="mb-3 flex items-center gap-2">
				<span class="badge preset-filled-surface text-xs">
					{plugin.displayName || classId}
				</span>
				{#if plugin.description}
					<span class="text-xs text-surface-600 dark:text-surface-400">
						{plugin.description}
					</span>
				{/if}
			</div>

			<!-- 动态加载插件编辑器组件 -->
			<EditorComponent bind:value {readonly} />
		</div>
	{:else}
		<!-- 插件缺失：使用 Fallback 编辑器 -->
		<FallbackConfigEditor bind:value {classId} {instanceType} {readonly} />
	{/if}
{:else}
	<!-- View 模式：只读显示 -->
	<div class="card p-4 bg-surface-100 dark:bg-surface-800">
		<pre class="text-xs font-mono overflow-x-auto">{JSON.stringify(value, null, 2)}</pre>
	</div>
{/if}
