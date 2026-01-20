<script lang="ts">
	import { settingsStore, PRESET_THEMES, type PresetThemeName } from '$lib/stores/settings.svelte';
	import { customThemesStore } from '$lib/stores/custom-themes.svelte';
	import { m } from '$lib/paraglide/messages';

	// Type cast to avoid TypeScript errors (Paraglide generates .js without types)
	const msg = m as any;

	// Get preset theme entries as array
	const presetThemeEntries = Object.entries(PRESET_THEMES) as [PresetThemeName, typeof PRESET_THEMES[PresetThemeName]][];
	
	// Get custom themes reactively
	let customThemes = $derived(customThemesStore.getAllThemes());
	
	// Map preset theme keys to translated names
	const getPresetThemeName = (key: PresetThemeName): string => {
		const map: Record<PresetThemeName, () => string> = {
			vintage: msg.settings_theme_vintage,
			modern: msg.settings_theme_modern,
			rocket: msg.settings_theme_rocket,
			seafoam: msg.settings_theme_seafoam,
			cerberus: msg.settings_theme_cerberus,
			sahara: msg.settings_theme_sahara,
			astro: msg.settings_theme_astro
		};
		return map[key]();
	};
	
	const getModeText = (): string => {
		return settingsStore.isDarkMode ? msg.settings_darkmode_dark() : msg.settings_darkmode_light();
	};

	// Get current preview data
	const getCurrentPreviewData = (): { name: string; colors: string[] } => {
		const currentTheme = settingsStore.theme;
		
		// Check if it's a custom theme
		if (currentTheme.startsWith('custom:')) {
			const themeId = currentTheme.slice(7);
			const customTheme = customThemesStore.getTheme(themeId);
			if (customTheme) {
				return { name: customTheme.name, colors: customTheme.colors };
			}
		}
		
		// Preset theme or fallback
		if (currentTheme in PRESET_THEMES) {
			const key = currentTheme as PresetThemeName;
			return { name: getPresetThemeName(key), colors: PRESET_THEMES[key].colors };
		}
		
		// Fallback to vintage
		return { name: getPresetThemeName('astro'), colors: PRESET_THEMES.astro.colors };
	};

	let previewData = $derived(getCurrentPreviewData());
</script>

<div class="space-y-4">
	<div>
		<label class="label mb-2">
			<span class="text-base font-semibold">{msg.settings_theme_label()}</span>
		</label>
		<select class="select" bind:value={settingsStore.theme}>
			<!-- Preset Themes -->
			<optgroup label={msg.settings_custom_themes_preset()}>
				{#each presetThemeEntries as [key, _]}
					<option value={key}>
						{getPresetThemeName(key)}
					</option>
				{/each}
			</optgroup>

			<!-- Custom Themes -->
			{#if customThemes.length > 0}
				<optgroup label={msg.settings_custom_themes_custom()}>
					{#each customThemes as theme}
						<option value="custom:{theme.id}">
							{theme.name}
						</option>
					{/each}
				</optgroup>
			{/if}
		</select>
	</div>

	<!-- Theme Preview -->
	<div class="card border border-surface-300 p-4 dark:border-surface-600">
		<div class="mb-2 text-sm font-medium opacity-75">{msg.settings_theme_preview()}</div>
		<div class="flex gap-2">
			{#each previewData.colors as color}
				<div
					class="h-12 flex-1 rounded-lg border border-surface-200 shadow-sm dark:border-surface-700"
					style="background-color: {color}"
					title={color}
				></div>
			{/each}
		</div>
		<div class="mt-3 text-center text-sm opacity-60">
			{previewData.name} - {getModeText()}
		</div>
	</div>
</div>
