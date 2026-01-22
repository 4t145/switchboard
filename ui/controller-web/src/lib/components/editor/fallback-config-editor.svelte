<script lang="ts">
	import { AlertCircle, Copy, ChevronDown } from 'lucide-svelte';

	type Props = {
		value: any;
		classId: string;
		instanceType?: 'node' | 'filter';
		readonly?: boolean;
	};

	let {
		value = $bindable(),
		classId,
		instanceType = 'node',
		readonly = false
	}: Props = $props();

	// JSON 编辑状态
	let jsonString = $state('');
	let parseError = $state<string | null>(null);
	let showPluginGuide = $state(false);

	// 初始化和同步 value 到 jsonString
	$effect(() => {
		try {
			if (typeof value === 'object' && value !== null) {
				jsonString = JSON.stringify(value, null, 2);
			} else if (typeof value === 'string') {
				jsonString = value;
			} else {
				jsonString = '{}';
			}
			parseError = null;
		} catch (e) {
			jsonString = '{}';
			parseError = 'Invalid value';
			console.error('[FallbackConfigEditor] Failed to serialize value:', e);
		}
	});

	// 更新 value
	function handleJsonChange() {
		if (readonly) return;

		try {
			const parsed = JSON.parse(jsonString);
			value = parsed;
			parseError = null;
			console.debug('[FallbackConfigEditor] Config updated:', parsed);
		} catch (e) {
			parseError = (e as Error).message;
			console.warn('[FallbackConfigEditor] JSON parse error:', e);
		}
	}

	// 复制示例代码
	async function copyPluginCode() {
		const code = `window.SwitchboardPluginAPI.registerHttpClassEditor({
  classId: '${classId}',
  type: '${instanceType}',
  component: YourEditorComponent,
  displayName: 'Your Plugin Name',
  description: 'Plugin description',
  createDefaultConfig: () => ({
    // Your default config here
  })
});`;

		try {
			await navigator.clipboard.writeText(code);
			console.log('[FallbackConfigEditor] Plugin registration code copied to clipboard');
		} catch (e) {
			console.error('[FallbackConfigEditor] Failed to copy to clipboard:', e);
		}
	}

	// 格式化 JSON
	function formatJson() {
		try {
			const parsed = JSON.parse(jsonString);
			jsonString = JSON.stringify(parsed, null, 2);
			parseError = null;
		} catch (e) {
			parseError = 'Cannot format invalid JSON';
		}
	}
</script>

<div class="fallback-editor space-y-4">
	<!-- 警告提示 -->
	<div class="alert preset-filled-warning">
		<div class="alert-icon">
			<AlertCircle size={20} />
		</div>
		<div class="alert-message flex-1">
			<h4 class="font-semibold">插件编辑器未找到</h4>
			<p class="text-sm mt-1">
				未找到 <code class="code text-xs">{classId}</code> 类型的编辑器插件（{instanceType === 'node' ? '节点' : '过滤器'}）。
			</p>
			<p class="text-xs mt-2 text-warning-700 dark:text-warning-300">
				你可以使用下方的 JSON 编辑器手动编辑配置，或切换到 Reference 模式引用外部配置文件。
			</p>
		</div>
	</div>

	<!-- JSON 编辑器 -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<label class="label-text font-semibold">配置 JSON</label>
			<button
				type="button"
				class="btn btn-sm preset-tonal-surface"
				onclick={formatJson}
				disabled={readonly || !!parseError}
			>
				格式化
			</button>
		</div>

		<textarea
			class="textarea font-mono text-xs leading-relaxed"
			class:input-error={parseError}
			rows="16"
			bind:value={jsonString}
			onblur={handleJsonChange}
			placeholder="{JSON.stringify({}, null, 2)}"
			disabled={readonly}
		></textarea>

		{#if parseError}
			<div class="text-xs text-error-600 dark:text-error-400 flex items-start gap-1">
				<AlertCircle size={14} class="mt-0.5 flex-shrink-0" />
				<span>JSON 解析错误: {parseError}</span>
			</div>
		{/if}
	</div>

	<!-- 插件开发指南（可折叠） -->
	<div class="card preset-outlined">
		<button
			type="button"
			class="w-full p-3 flex items-center justify-between text-left hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
			onclick={() => (showPluginGuide = !showPluginGuide)}
		>
			<span class="text-sm font-semibold">需要更好的编辑体验？</span>
			<ChevronDown
				size={16}
				class="transition-transform duration-200"
				style={showPluginGuide ? 'transform: rotate(180deg)' : ''}
			/>
		</button>

		{#if showPluginGuide}
			<div class="px-3 pb-3 space-y-3 text-sm">
				<p class="text-xs text-surface-600 dark:text-surface-400">
					如果你是插件开发者，可以通过以下方式注册专用编辑器：
				</p>

				<div class="relative">
					<pre
						class="code-block text-xs !pr-12 overflow-x-auto"><code>{`window.SwitchboardPluginAPI.registerHttpClassEditor({
  classId: '${classId}',
  type: '${instanceType}',
  component: YourEditorComponent,
  displayName: 'Your Plugin Name',
  description: 'Plugin description',
  createDefaultConfig: () => ({
    // Your default config here
  })
});`}</code></pre>

					<button
						type="button"
						class="btn-icon btn-icon-sm absolute top-2 right-2"
						onclick={copyPluginCode}
						title="复制代码"
					>
						<Copy size={14} />
					</button>
				</div>

				<div class="text-xs text-surface-600 dark:text-surface-400 space-y-1">
					<p class="font-semibold">插件开发资源：</p>
					<ul class="list-disc list-inside ml-2 space-y-1">
						<li>参考现有插件实现：<code class="code">lib/plugins/providers/http/classes/</code></li>
						<li>使用 Skeleton UI 组件库构建界面</li>
						<li>通过 <code class="code">window.SwitchboardPluginAPI</code> 访问共享依赖</li>
					</ul>
				</div>
			</div>
		{/if}
	</div>

	<!-- 快速操作提示 -->
	<div class="text-xs text-surface-600 dark:text-surface-400 bg-surface-100 dark:bg-surface-800 p-3 rounded">
		<p class="font-semibold mb-1">💡 提示</p>
		<ul class="list-disc list-inside space-y-1 ml-2">
			<li>编辑完成后点击其他区域以保存更改</li>
			<li>可以使用 "格式化" 按钮美化 JSON 代码</li>
			<li>切换到 <strong>Reference 模式</strong>可引用外部配置文件</li>
		</ul>
	</div>
</div>

<style>
	.code-block {
		padding: 0.75rem;
		border-radius: 0.25rem;
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		background-color: rgb(var(--color-surface-900));
		color: rgb(var(--color-surface-50));
	}

	:global(.dark) .code-block {
		background-color: rgb(var(--color-surface-950));
	}
</style>
