<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import ThemeSelector from '$lib/components/settings/theme-selector.svelte';
	import DarkModeToggle from '$lib/components/settings/dark-mode-toggle.svelte';
	import LanguageSelector from '$lib/components/settings/language-selector.svelte';
	import CustomThemeUploader from '$lib/components/settings/custom-theme-uploader.svelte';
	import CustomThemeManager from '$lib/components/settings/custom-theme-manager.svelte';
	import { RotateCcw, Settings as SettingsIcon } from 'lucide-svelte';
	import { m } from '$lib/paraglide/messages';

	const msg = m as any;

	function handleReset() {
		if (confirm(msg.settings_reset_confirm())) {
			settingsStore.reset();
		}
	}
</script>

<div class="container mx-auto max-w-3xl p-6">
	<!-- Header -->
	<div class="mb-8">
		<div class="mb-2 flex items-center gap-3">
			<SettingsIcon size={32} class="text-primary-600 dark:text-primary-400" />
			<h1 class="h1 font-bold">{msg.settings_title()}</h1>
		</div>
		<p class="text-surface-600 dark:text-surface-400">
			{msg.settings_description()}
		</p>
	</div>

	<!-- Appearance Section -->
	<section class="mb-6">
		<div class="mb-4 flex items-center gap-2 border-b border-surface-200 pb-2 dark:border-surface-700">
			<h2 class="h2 text-xl font-semibold">🎨 {msg.settings_appearance()}</h2>
		</div>

		<div class="space-y-4">
			<!-- Theme Selector -->
			<ThemeSelector />

			<!-- Dark Mode Toggle -->
			<DarkModeToggle />
		</div>
	</section>

	<!-- Custom Themes Section -->
	<section class="mb-6">
		<div class="mb-4 flex items-center gap-2 border-b border-surface-200 pb-2 dark:border-surface-700">
			<h2 class="h2 text-xl font-semibold">✨ {msg.settings_custom_themes()}</h2>
		</div>

		<div class="space-y-6">
			<!-- Theme Uploader -->
			<CustomThemeUploader />

			<!-- Theme Manager -->
			<CustomThemeManager />
		</div>
	</section>

	<!-- Language Section -->
	<section class="mb-6">
		<div class="mb-4 flex items-center gap-2 border-b border-surface-200 pb-2 dark:border-surface-700">
			<h2 class="h2 text-xl font-semibold">🌐 {msg.settings_language()}</h2>
		</div>

		<LanguageSelector />
	</section>

	<!-- Actions -->
	<div class="mt-8 flex items-center justify-between border-t border-surface-200 pt-6 dark:border-surface-700">
		<button class="btn preset-ghost-surface flex items-center gap-2" onclick={handleReset}>
			<RotateCcw size={18} />
			<span>{msg.settings_reset()}</span>
		</button>

		<div class="text-sm opacity-60">
			{msg.settings_auto_save()}
		</div>
	</div>

	<!-- Info Section -->
	<div class="mt-6 rounded-lg bg-primary-50 p-4 dark:bg-primary-900/10">
		<div class="text-sm opacity-75">
			<p class="mb-2 font-semibold">💡 {msg.settings_tips_title()}</p>
			<ul class="ml-4 list-disc space-y-1">
				<li>{msg.settings_tips_instant()}</li>
				<li>{msg.settings_tips_local()}</li>
				<li>{msg.settings_tips_clear()}</li>
			</ul>
		</div>
	</div>
</div>
