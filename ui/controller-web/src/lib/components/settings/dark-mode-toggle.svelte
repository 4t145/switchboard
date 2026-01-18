<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { m } from '$lib/paraglide/messages';
	import { Moon, Sun } from 'lucide-svelte';
	
	const msg = m as any;
	
	function toggle() {
		settingsStore.darkMode = !settingsStore.darkMode;
	}
</script>

<div class="card border border-surface-300 p-5 dark:border-surface-600">
	<div class="flex items-center justify-between gap-4">
		<div class="flex-1">
			<div class="mb-1 font-semibold">{msg.settings_darkmode_label()}</div>
			<div class="text-sm opacity-75">{msg.settings_darkmode_desc()}</div>
		</div>

		<!-- Custom Toggle Switch -->
		<button
			class="relative inline-flex h-8 w-14 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 {settingsStore.darkMode
				? 'bg-primary-600'
				: 'bg-surface-300 dark:bg-surface-600'}"
			onclick={toggle}
			role="switch"
			aria-checked={settingsStore.darkMode}
		>
			<span
				class="inline-flex h-6 w-6 transform items-center justify-center rounded-full bg-white shadow-lg transition-transform {settingsStore.darkMode
					? 'translate-x-7'
					: 'translate-x-1'}"
			>
				{#if settingsStore.darkMode}
					<Moon size={14} class="text-primary-600" />
				{:else}
					<Sun size={14} class="text-amber-500" />
				{/if}
			</span>
		</button>
	</div>
</div>
