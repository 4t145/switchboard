<script lang="ts">
	import { dataTypeRegistry } from '../registry';

	let {
		// 数据类型标识符
		type,

		// 编辑/查看模式
		mode = 'edit',

		// 值（双向绑定）
		value = $bindable(),

		// 传递给子组件的其他 props
		...restProps
	}: {
		type: string;
		mode?: 'edit' | 'view';
		value?: any;
		[key: string]: any;
	} = $props();

	// 获取类型元信息和对应的组件
	const metadata = $derived(dataTypeRegistry.get(type));
	const component = $derived(metadata?.component);

	// 初始化默认值
	$effect(() => {
		if (value === undefined && metadata?.defaultValue) {
			value = metadata.defaultValue();
		}
	});
</script>

{#if component}
	<!-- 使用注册的数据类型组件 -->
	{@const Component = component}
	<Component {mode} bind:value {...restProps} />
{:else}
	<!-- 降级处理：未注册的类型 -->
	<div class="card p-4 preset-filled-warning">
		<p class="text-sm font-semibold mb-2">⚠️ 未注册的数据类型: {type}</p>
		{#if mode === 'edit'}
			<textarea class="textarea font-mono text-xs" bind:value rows={6}></textarea>
		{:else}
			<pre class="text-xs font-mono overflow-x-auto">{JSON.stringify(value, null, 2)}</pre>
		{/if}
	</div>
{/if}
