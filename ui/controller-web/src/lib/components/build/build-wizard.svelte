<script lang="ts">
	import { Check, ChevronRight, Save } from 'lucide-svelte';
	import StepSourceSelection from './step-source-selection.svelte';
	import StepConfigEditor from './step-config-editor.svelte';
	import type { HumanReadableServiceConfig } from '$lib/api/types';

	let currentStep = $state<number>(1);
	let config = $state<HumanReadableServiceConfig | null>(null);
	let sourceSummary = $state<string>('');
	let saveAs = $state<string | undefined>(undefined);

	function onSourceSelected(
		loadedConfig: Record<string, any>,
		summary: string,
		saveAsVal?: string
	) {
		config = loadedConfig as HumanReadableServiceConfig;
		sourceSummary = summary;
		saveAs = saveAsVal;
		currentStep = 2;
	}

	function goToStep(step: number) {
		if (currentStep < step) return; // Prevent jumping forward
		currentStep = step;
	}

	function handleSave() {
		console.log('Saving config...', config);
		// TODO: Implement save logic
		alert('Save/Deploy logic not implemented yet.\nCheck console for config object.');
	}
</script>

<div class="flex h-full flex-col">
	<!-- Top Bar Stepper -->
	<div
		class="flex flex-none items-center justify-between border-b border-surface-200 bg-surface-50 p-4 dark:border-surface-700 dark:bg-surface-900"
	>
		<div class="flex items-center gap-2 overflow-x-auto">
			<!-- Step 1 Indicator -->
			<button
				class="flex items-center gap-2 rounded px-3 py-2 transition-colors
                {currentStep === 1
					? 'bg-primary-100 font-bold text-primary-700 dark:bg-primary-900/30 dark:text-primary-300'
					: currentStep > 1
						? 'text-surface-600 hover:bg-surface-200 dark:text-surface-300 dark:hover:bg-surface-800'
						: 'opacity-50'}"
				onclick={() => goToStep(1)}
			>
				<div
					class="flex h-6 w-6 items-center justify-center rounded-full border text-xs
                    {currentStep >= 1
						? 'border-primary-500 bg-primary-500 text-white'
						: 'border-surface-400'}"
				>
					{#if currentStep > 1}
						<Check size={14} />
					{:else}
						1
					{/if}
				</div>
				<span class="whitespace-nowrap">Select Source</span>
			</button>

			<ChevronRight size={16} class="text-surface-400" />

			<!-- Step 2 Indicator -->
			<button
				class="flex items-center gap-2 rounded px-3 py-2 transition-colors
                {currentStep === 2
					? 'bg-primary-100 font-bold text-primary-700 dark:bg-primary-900/30 dark:text-primary-300'
					: currentStep > 2
						? 'text-surface-600 hover:bg-surface-200 dark:text-surface-300 dark:hover:bg-surface-800'
						: 'cursor-not-allowed text-surface-400 dark:text-surface-500'}"
				onclick={() => goToStep(2)}
				disabled={currentStep < 2}
			>
				<div
					class="flex h-6 w-6 items-center justify-center rounded-full border text-xs
                    {currentStep >= 2
						? 'border-primary-500 bg-primary-500 text-white'
						: 'border-surface-400'}"
				>
					2
				</div>
				<span class="whitespace-nowrap">Edit Configuration</span>
			</button>

			<!-- Optional Summary in Header -->
			{#if currentStep > 1 && sourceSummary}
				<div
					class="ml-4 hidden items-center border-l border-surface-300 pl-4 md:flex dark:border-surface-600"
				>
					<span class="mr-2 text-xs text-surface-500">Source:</span>
					<span class="variant-soft-secondary badge text-xs">{sourceSummary}</span>
				</div>
			{/if}
		</div>

		<!-- Right Actions -->
		<div class="flex items-center gap-2">
			{#if currentStep === 2}
				<button class="variant-filled-primary btn btn-sm" onclick={handleSave}>
					<Save size={16} class="mr-2" />
					Save / Deploy
				</button>
			{/if}
		</div>
	</div>

	<!-- Main Content Area -->
	<div class="relative flex-1 overflow-hidden">
		<!-- Step 1 Content -->
		<!-- Using visibility/display toggling via absolute positioning to preserve state -->
		<div
			class="absolute inset-0 overflow-auto bg-surface-50 p-6 transition-opacity duration-300 dark:bg-surface-900"
			class:hidden={currentStep !== 1}
		>
			<div class="mx-auto flex h-full max-w-6xl flex-col justify-center">
				<div class="mb-8 text-center">
					<h2 class="mb-2 h2 font-bold">How would you like to start?</h2>
					<p class="text-surface-500">Select a source to load your initial configuration from.</p>
				</div>
				<StepSourceSelection onNext={onSourceSelected} />
			</div>
		</div>

		<!-- Step 2 Content -->
		<!-- Conditional rendering for Step 2 is fine as it depends on config loaded in Step 1 -->
		{#if currentStep === 2 && config}
			<div class="absolute inset-0 overflow-hidden bg-surface-50 p-4 dark:bg-surface-900">
				<StepConfigEditor bind:config />
			</div>
		{/if}
	</div>
</div>
