<script lang="ts">
	import { settingsStore, type LanguageCode } from '$lib/stores/settings.svelte';

	const languages: { code: LanguageCode; name: string; flag: string; nativeName: string }[] = [
		{ code: 'zh', name: 'Chinese', flag: '🇨🇳', nativeName: '中文' },
		{ code: 'en', name: 'English', flag: '🇺🇸', nativeName: 'English' }
	];
</script>

<div class="space-y-3">
	<label class="label mb-2">
		<span class="text-base font-semibold">界面语言</span>
	</label>

	<div class="space-y-2">
		{#each languages as lang}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="card cursor-pointer border p-4 transition-all {settingsStore.language === lang.code
					? 'border-primary-500 bg-primary-50 ring-2 ring-primary-500 dark:bg-primary-900/20'
					: 'border-surface-300 hover:border-primary-300 dark:border-surface-600 dark:hover:border-primary-700'}"
				onclick={() => (settingsStore.language = lang.code)}
			>
				<div class="flex items-center gap-3">
					<!-- Radio indicator -->
					<div
						class="flex h-5 w-5 flex-none items-center justify-center rounded-full border-2 {settingsStore.language ===
						lang.code
							? 'border-primary-500'
							: 'border-surface-400'}"
					>
						{#if settingsStore.language === lang.code}
							<div class="h-2.5 w-2.5 rounded-full bg-primary-500"></div>
						{/if}
					</div>

					<!-- Flag and language name -->
					<div class="flex flex-1 items-center gap-3">
						<span class="text-3xl">{lang.flag}</span>
						<div>
							<div class="font-semibold">{lang.nativeName}</div>
							<div class="text-sm opacity-60">{lang.name}</div>
						</div>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
