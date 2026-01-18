<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import ThemeSelector from '$lib/components/settings/theme-selector.svelte';
	import DarkModeToggle from '$lib/components/settings/dark-mode-toggle.svelte';
	import LanguageSelector from '$lib/components/settings/language-selector.svelte';
	import { RotateCcw, Settings as SettingsIcon } from 'lucide-svelte';

	function handleReset() {
		if (confirm('确定要恢复所有设置到默认值吗？此操作不可撤销。')) {
			settingsStore.reset();
		}
	}
</script>

<div class="container mx-auto max-w-3xl p-6">
	<!-- Header -->
	<div class="mb-8">
		<div class="mb-2 flex items-center gap-3">
			<SettingsIcon size={32} class="text-primary-600 dark:text-primary-400" />
			<h1 class="h1 font-bold">设置</h1>
		</div>
		<p class="text-surface-600 dark:text-surface-400">
			自定义您的偏好设置，包括主题、深色模式和语言选项。所有设置会自动保存。
		</p>
	</div>

	<!-- Appearance Section -->
	<section class="mb-6">
		<div class="mb-4 flex items-center gap-2 border-b border-surface-200 pb-2 dark:border-surface-700">
			<h2 class="h2 text-xl font-semibold">🎨 外观</h2>
		</div>

		<div class="space-y-4">
			<!-- Theme Selector -->
			<ThemeSelector />

			<!-- Dark Mode Toggle -->
			<DarkModeToggle />
		</div>
	</section>

	<!-- Language Section -->
	<section class="mb-6">
		<div class="mb-4 flex items-center gap-2 border-b border-surface-200 pb-2 dark:border-surface-700">
			<h2 class="h2 text-xl font-semibold">🌐 语言</h2>
		</div>

		<LanguageSelector />
	</section>

	<!-- Actions -->
	<div class="mt-8 flex items-center justify-between border-t border-surface-200 pt-6 dark:border-surface-700">
		<button class="btn preset-ghost-surface flex items-center gap-2" onclick={handleReset}>
			<RotateCcw size={18} />
			<span>恢复默认设置</span>
		</button>

		<div class="text-sm opacity-60">
			设置会自动保存
		</div>
	</div>

	<!-- Info Section -->
	<div class="mt-6 rounded-lg bg-primary-50 p-4 dark:bg-primary-900/10">
		<div class="text-sm opacity-75">
			<p class="mb-2 font-semibold">💡 提示</p>
			<ul class="ml-4 list-disc space-y-1">
				<li>主题和深色模式设置会立即生效</li>
				<li>所有设置保存在浏览器本地，不会在其他设备同步</li>
				<li>清除浏览器数据会重置所有设置</li>
			</ul>
		</div>
	</div>
</div>
