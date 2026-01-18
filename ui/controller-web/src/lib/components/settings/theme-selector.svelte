<script lang="ts">
	import { settingsStore, THEMES, type ThemeName } from '$lib/stores/settings.svelte';

	// Get theme entries as array
	const themeEntries = Object.entries(THEMES) as [ThemeName, typeof THEMES[ThemeName]][];
</script>

<div class="space-y-4">
	<div>
		<label class="label mb-2">
			<span class="text-base font-semibold">主题</span>
		</label>
		<select class="select" bind:value={settingsStore.theme}>
			{#each themeEntries as [key, theme]}
				<option value={key}>
					{theme.name}
				</option>
			{/each}
		</select>
	</div>

	<!-- Theme Preview -->
	<div class="card border border-surface-300 p-4 dark:border-surface-600">
		<div class="mb-2 text-sm font-medium opacity-75">主题预览</div>
		<div class="flex gap-2">
			{#each THEMES[settingsStore.theme].colors as color}
				<div
					class="h-12 flex-1 rounded-lg border border-surface-200 shadow-sm dark:border-surface-700"
					style="background-color: {color}"
					title={color}
				></div>
			{/each}
		</div>
		<div class="mt-3 text-center text-sm opacity-60">
			{THEMES[settingsStore.theme].name} - {settingsStore.darkMode ? '深色模式' : '浅色模式'}
		</div>
	</div>
</div>
