<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { m } from '$lib/paraglide/messages';
	import { Moon, Sun, Monitor } from '@lucide/svelte';
	import type { ColorMode } from '$lib/stores/settings.svelte';

	const msg = m as any;

	const modes: { value: ColorMode; icon: any; label: () => string }[] = [
		{ value: 'light', icon: Sun, label: () => msg.settings_darkmode_light() },
		{ value: 'dark', icon: Moon, label: () => msg.settings_darkmode_dark() },
		{ value: 'auto', icon: Monitor, label: () => msg.settings_darkmode_auto() }
	];

	function selectMode(mode: ColorMode) {
		settingsStore.colorMode = mode;
	}
</script>

<div class="card border p-5">
	<div class="space-y-4">
		<div>
			<div class="mb-1 font-semibold">{msg.settings_darkmode_label()}</div>
			<div class="text-sm opacity-75">{msg.settings_darkmode_desc()}</div>
		</div>

		<!-- Mode Selection Buttons -->
		<div class="grid grid-cols-3 gap-2">
			{#each modes as mode}
				<button
					class="flex flex-col items-center gap-2 rounded-lg border-2 p-3 transition-all hover:bg-surface-100 dark:hover:bg-surface-800 {settingsStore.colorMode ===
					mode.value
						? 'border-primary-500 bg-primary-50 dark:bg-primary-950'
						: ''}"
					onclick={() => selectMode(mode.value)}
					type="button"
				>
					<svelte:component
						this={mode.icon}
						size={24}
						class={settingsStore.colorMode === mode.value
							? 'text-primary-600 dark:text-primary-400'
							: 'text-surface-600 dark:text-surface-400'}
					/>
					<span
						class="text-sm font-medium {settingsStore.colorMode === mode.value
							? 'text-primary-700 dark:text-primary-300'
							: 'text-surface-700 dark:text-surface-300'}"
					>
						{mode.label()}
					</span>
				</button>
			{/each}
		</div>
	</div>
</div>
