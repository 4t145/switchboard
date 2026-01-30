<script lang="ts">
	import { customThemesStore } from '$lib/stores/custom-themes.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { Download, Trash2, Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	const msg = m as any;

	// Reactive state
	let themes = $derived(customThemesStore.getAllThemes());
	let currentThemeId = $derived(
		settingsStore.theme.startsWith('custom:') ? settingsStore.theme.slice(7) : null
	);

	function handleApplyTheme(themeId: string) {
		settingsStore.theme = `custom:${themeId}` as string;
	}

	function handleExportTheme(themeId: string) {
		customThemesStore.exportTheme(themeId);
	}

	function handleDeleteTheme(themeId: string, themeName: string) {
		if (confirm(msg.settings_custom_themes_delete_confirm().replace('{name}', themeName))) {
			const success = customThemesStore.removeTheme(themeId);

			if (success && currentThemeId === themeId) {
				// If deleted theme was active, switch to default
				settingsStore.theme = 'vintage';
			}
		}
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString();
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<h3 class="h3 text-lg font-semibold">{msg.settings_custom_themes_manage()}</h3>
		{#if themes.length > 0}
			<span class="text-sm opacity-60">
				{themes.length}
				{themes.length === 1 ? 'theme' : 'themes'}
			</span>
		{/if}
	</div>

	{#if themes.length === 0}
		<div class="card border p-8 text-center">
			<p class="opacity-60">{msg.settings_custom_themes_none()}</p>
		</div>
	{:else}
		<div class="space-y-3">
			{#each themes as theme (theme.id)}
				<div class="card border p-4">
					<div class="flex items-start gap-4">
						<!-- Color Preview -->
						<div class="flex flex-shrink-0 gap-1">
							{#each theme.colors.slice(0, 3) as color}
								<div
									class="h-8 w-8 rounded border border-surface-200 dark:border-surface-700"
									style="background-color: {color}"
									title={color}
								></div>
							{/each}
						</div>

						<!-- Theme Info -->
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2">
								<h4 class="truncate font-semibold">{theme.name}</h4>
								{#if currentThemeId === theme.id}
									<span class="badge flex items-center gap-1 preset-tonal-success">
										<Check size={12} />
										Active
									</span>
								{/if}
							</div>
							<p class="mt-1 text-xs opacity-60">
								{msg.settings_custom_themes_created()}: {formatDate(theme.createdAt)}
							</p>
						</div>

						<!-- Actions -->
						<div class="flex flex-shrink-0 gap-2">
							{#if currentThemeId !== theme.id}
								<button
									onclick={() => handleApplyTheme(theme.id)}
									class="btn preset-filled-primary-500 btn-sm"
									title={msg.settings_custom_themes_apply()}
								>
									{msg.settings_custom_themes_apply()}
								</button>
							{/if}

							<button
								onclick={() => handleExportTheme(theme.id)}
								class="preset-ghost-surface btn btn-sm"
								title={msg.settings_custom_themes_export()}
							>
								<Download size={16} />
							</button>

							<button
								onclick={() => handleDeleteTheme(theme.id, theme.name)}
								class="preset-ghost-error btn btn-sm"
								title={msg.settings_custom_themes_delete()}
							>
								<Trash2 size={16} />
							</button>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
