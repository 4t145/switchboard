<script lang="ts">
	import { settingsStore, THEMES, type ThemeName } from '$lib/stores/settings.svelte';
	import { m } from '$lib/paraglide/messages';

	// Type cast to avoid TypeScript errors (Paraglide generates .js without types)
	const msg = m as any;

	// Get theme entries as array
	const themeEntries = Object.entries(THEMES) as [ThemeName, typeof THEMES[ThemeName]][];
	
	// Map theme keys to translated names
	const getThemeName = (key: ThemeName): string => {
		const map: Record<ThemeName, () => string> = {
			vintage: msg.settings_theme_vintage,
			modern: msg.settings_theme_modern,
			rocket: msg.settings_theme_rocket,
			seafoam: msg.settings_theme_seafoam,
			skeleton: msg.settings_theme_skeleton,
			sahara: msg.settings_theme_sahara
		};
		return map[key]();
	};
	
	const getModeText = (): string => {
		return settingsStore.darkMode ? msg.settings_darkmode_dark() : msg.settings_darkmode_light();
	};
</script>

<div class="space-y-4">
	<div>
		<label class="label mb-2">
			<span class="text-base font-semibold">{msg.settings_theme_label()}</span>
		</label>
		<select class="select" bind:value={settingsStore.theme}>
			{#each themeEntries as [key, theme]}
				<option value={key}>
					{getThemeName(key)}
				</option>
			{/each}
		</select>
	</div>

	<!-- Theme Preview -->
	<div class="card border border-surface-300 p-4 dark:border-surface-600">
		<div class="mb-2 text-sm font-medium opacity-75">{msg.settings_theme_preview()}</div>
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
			{getThemeName(settingsStore.theme)} - {getModeText()}
		</div>
	</div>
</div>
