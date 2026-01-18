<script lang="ts">
	import { Save, Loader2 } from 'lucide-svelte';
	import { Steps } from '@skeletonlabs/skeleton-svelte';
	import StepSourceSelection from './step-source-selection.svelte';
	import StepConfigEditor from './step-config-editor.svelte';
	import type { HumanReadableServiceConfig } from '$lib/api/types';

	const stepDefinitions = [
		{ title: 'Select Source', description: 'Choose configuration source' },
		{ title: 'Edit Configuration', description: 'Customize your settings' }
	];

	let currentStep = $state<number>(0);
	let config = $state<HumanReadableServiceConfig | null>(null);
	let sourceSummary = $state<string>('');
	let saveAs = $state<string | undefined>(undefined);

	// Source selection sub-step states
	let sourceMode = $state<'select' | 'load-saved' | 'from-source'>('select');
	let canProceed = $state(false);
	let isLoadingSource = $state(false);

	// Reference to child component
	let stepSourceSelectionRef: any;

	function onSourceSelected(
		loadedConfig: Record<string, any>,
		summary: string,
		saveAsVal?: string
	) {
		config = loadedConfig as HumanReadableServiceConfig;
		sourceSummary = summary;
		saveAs = saveAsVal;
		currentStep = 1;
		// Reset source mode when moving to next step
		sourceMode = 'select';
	}

	function handleGoBackToSource() {
		currentStep = 0;
		config = null;
		sourceSummary = '';
		saveAs = undefined;
		sourceMode = 'select';
	}

	function handleCancelSubStep() {
		sourceMode = 'select';
	}

	async function handleConfirmSubStep() {
		if (sourceMode === 'load-saved' && stepSourceSelectionRef) {
			await stepSourceSelectionRef.confirmLoadSaved();
		} else if (sourceMode === 'from-source' && stepSourceSelectionRef) {
			await stepSourceSelectionRef.confirmFromSource();
		}
	}

	function handleSave() {
		console.log('Saving config...', config);
		// TODO: Implement save logic
		alert('Save/Deploy logic not implemented yet.\nCheck console for config object.');
	}
</script>

<Steps
	step={currentStep}
	onStepChange={(details) => (currentStep = details.step)}
	count={stepDefinitions.length}
	linear
	class="flex h-full flex-col"
>
	<!-- Top Bar with Steps Navigation -->
	<div
		class="flex flex-none items-center justify-between border-b border-surface-200 bg-surface-50 p-4 dark:border-surface-700 dark:bg-surface-900"
	>
		<Steps.List class="flex items-center gap-2 overflow-x-auto">
			{#each stepDefinitions as step, index}
				<Steps.Item {index}>
					<Steps.Trigger
						class="flex items-center gap-2 rounded px-3 py-2 transition-colors data-[current]:bg-primary-100 data-[current]:font-bold data-[current]:text-primary-700 data-[complete]:text-surface-600 data-[complete]:hover:bg-surface-200 data-[incomplete]:cursor-not-allowed data-[incomplete]:opacity-50 dark:data-[current]:bg-primary-900/30 dark:data-[current]:text-primary-300 dark:data-[complete]:text-surface-300 dark:data-[complete]:hover:bg-surface-800 dark:data-[incomplete]:text-surface-500"
					>
						<Steps.Indicator
							class="flex h-6 w-6 items-center justify-center rounded-full border text-xs data-[current]:border-primary-500 data-[current]:bg-primary-500 data-[current]:text-white data-[complete]:border-primary-500 data-[complete]:bg-primary-500 data-[complete]:text-white data-[incomplete]:border-surface-400"
						>
							{index + 1}
						</Steps.Indicator>
						<span class="whitespace-nowrap">{step.title}</span>
					</Steps.Trigger>
					{#if index < stepDefinitions.length - 1}
						<Steps.Separator class="mx-2 h-px w-8 bg-surface-300 dark:bg-surface-600" />
					{/if}
				</Steps.Item>
			{/each}
		</Steps.List>

		<!-- Optional Summary -->
		<div class="flex items-center">
			{#if currentStep > 0 && sourceSummary}
				<div
					class="hidden items-center border-l border-surface-300 pl-4 md:flex dark:border-surface-600"
				>
					<span class="mr-2 text-xs text-surface-500">Source:</span>
					<span class="preset-tonal-secondary badge text-xs">{sourceSummary}</span>
				</div>
			{/if}
		</div>
	</div>

	<!-- Main Content Area -->
	<div class="relative flex-1 overflow-hidden">
		<!-- Step 0: Source Selection -->
		<Steps.Content index={0} class="absolute inset-0 overflow-auto">
			<div class="mx-auto flex min-h-full max-w-6xl flex-col justify-center p-6">
				<div class="mb-8 text-center">
					<h2 class="mb-2 h2 font-bold">How would you like to start?</h2>
					<p class="text-surface-500">Select a source to load your initial configuration from.</p>
				</div>
				<StepSourceSelection
					bind:this={stepSourceSelectionRef}
					bind:mode={sourceMode}
					bind:canProceed
					bind:isLoading={isLoadingSource}
					onNext={onSourceSelected}
				/>
			</div>
		</Steps.Content>

		<!-- Step 1: Config Editor -->
		<Steps.Content index={1} class="absolute inset-0 overflow-auto">
			{#if config}
				<StepConfigEditor bind:config />
			{:else}
				<div class="flex h-full items-center justify-center">
					<p class="text-surface-500">Loading configuration...</p>
				</div>
			{/if}
		</Steps.Content>
	</div>

	<!-- Bottom Navigation Bar -->
	<div
		class="flex-none border-t border-surface-200 bg-surface-50 p-4 dark:border-surface-700 dark:bg-surface-900"
	>
		<div class="flex items-center justify-between">
			<div>
				{#if currentStep === 0 && sourceMode !== 'select'}
					<!-- In a sub-step: show Cancel button -->
					<button class="preset-ghost-surface btn" onclick={handleCancelSubStep} disabled={isLoadingSource}>
						取消
					</button>
				{:else if currentStep > 0}
					<!-- In step 2: show Back to source button -->
					<button class="preset-ghost-surface btn" onclick={handleGoBackToSource}>
						← 返回选择源
					</button>
				{/if}
			</div>
			<div class="flex gap-2">
				{#if currentStep === 0 && sourceMode === 'load-saved'}
					<!-- Load Saved sub-step: show Load button -->
					<button
						class="preset-filled-secondary btn"
						disabled={!canProceed || isLoadingSource}
						onclick={handleConfirmSubStep}
					>
						{#if isLoadingSource}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						加载所选配置 →
					</button>
				{:else if currentStep === 0 && sourceMode === 'from-source'}
					<!-- From Source sub-step: show Resolve button -->
					<button
						class="preset-filled-tertiary btn"
						disabled={!canProceed || isLoadingSource}
						onclick={handleConfirmSubStep}
					>
						{#if isLoadingSource}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						解析并构建 →
					</button>
				{:else if currentStep === 1 && config}
					<!-- Step 2: show Save/Deploy button -->
					<button class="preset-filled-primary btn" onclick={handleSave}>
						<Save size={16} class="mr-2" />
						保存并部署
					</button>
				{/if}
			</div>
		</div>
	</div>
</Steps>
