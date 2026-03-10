<script lang="ts">
	import { Save, Loader2, CheckCircle2, AlertCircle } from '@lucide/svelte';
	import { Steps } from '@skeletonlabs/skeleton-svelte';
	import StepSourceSelection from './step-source-selection.svelte';
	import StepConfigEditor from './step-config-editor.svelte';
	import type {
		ConfigRolloutReport,
		HumanReadableServiceConfig,
		KernelRolloutResult,
		ResultObject
	} from '$lib/api/types';
	import { api } from '$lib/api/routes';

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

	// Deploy states
	let isDeploying = $state(false);
	let deploySuccess = $state(false);
	let deployError = $state<string | null>(null);
	let deployTransactionId = $state<string | null>(null);

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
		// Reset deploy states
		deploySuccess = false;
		deployError = null;
		deployTransactionId = null;
	}

	function getResultErrorMessage(result: ResultObject<null>): string | null {
		if ('error' in result) {
			const frames = result.error.frames || [];
			if (frames.length === 0) return 'Unknown error';
			return frames.map((f: { error: string }) => f.error).join(' -> ');
		}
		return null;
	}

	function collectStageFailures(stage: string, results: KernelRolloutResult[]): string[] {
		return results
			.map(([addr, result]) => {
				const message = getResultErrorMessage(result);
				return message ? `${stage} | ${addr}: ${message}` : null;
			})
			.filter((item): item is string => item !== null);
	}

	function buildRolloutFailureMessage(report: ConfigRolloutReport): string {
		const phase = report.status.status === 'failed' ? report.status.phase : 'unknown';
		const lines = [
			`transaction_id: ${report.transaction_id}`,
			`failed_phase: ${phase}`,
			`all_or_nothing: ${report.all_or_nothing}`
		];

		const failures = [
			...collectStageFailures('prepare', report.prepare_results),
			...collectStageFailures('commit', report.commit_results),
			...collectStageFailures('abort', report.abort_results),
			...collectStageFailures('rollback_prepare', report.rollback_prepare_results),
			...collectStageFailures('rollback_commit', report.rollback_commit_results),
			...collectStageFailures('rollback_abort', report.rollback_abort_results)
		];

		if (report.rollback_transaction_id) {
			lines.push(`rollback_transaction_id: ${report.rollback_transaction_id}`);
		}

		if (failures.length > 0) {
			lines.push('', 'errors:');
			lines.push(...failures);
		}

		return lines.join('\n');
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

	async function handleSave() {
		if (!config) {
			deployError = 'No configuration to deploy';
			return;
		}

		isDeploying = true;
		deploySuccess = false;
		deployError = null;
		deployTransactionId = null;

		try {
			const report = await api.kernelManager.updateConfig(config as HumanReadableServiceConfig);
			deployTransactionId = report.transaction_id;

			if (report.status.status === 'succeeded') {
				deploySuccess = true;
				deployError = null;
			} else {
				deploySuccess = false;
				deployError = buildRolloutFailureMessage(report);
			}
		} catch (err) {
			deployError = err instanceof Error ? err.message : 'Failed to deploy configuration';
			deploySuccess = false;
			deployTransactionId = null;
		} finally {
			isDeploying = false;
		}
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
						class="flex items-center gap-2 rounded px-3 py-2 transition-colors data-[complete]:text-surface-600 data-[complete]:hover:bg-surface-200 data-[current]:bg-primary-100 data-[current]:font-bold data-[current]:text-primary-700 data-[incomplete]:cursor-not-allowed data-[incomplete]:opacity-50 dark:data-[complete]:text-surface-300 dark:data-[complete]:hover:bg-surface-800 dark:data-[current]:bg-primary-900/30 dark:data-[current]:text-primary-300 dark:data-[incomplete]:text-surface-500"
					>
						<Steps.Indicator
							class="flex h-6 w-6 items-center justify-center rounded-full border text-xs data-[complete]:border-primary-500 data-[complete]:bg-primary-500 data-[complete]:text-white data-[current]:border-primary-500 data-[current]:bg-primary-500 data-[current]:text-white data-[incomplete]:border-surface-400"
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
				<div class="hidden items-center border-l pl-4 md:flex">
					<span class="mr-2 text-xs text-surface-500">Source:</span>
					<span class="badge preset-tonal-secondary text-xs">{sourceSummary}</span>
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
		<!-- Deploy Status Messages -->
		{#if deploySuccess}
			<div
				class="mb-3 flex items-center gap-2 rounded bg-success-100 p-3 text-success-700 dark:bg-success-900 dark:text-success-300"
			>
				<CheckCircle2 class="h-5 w-5 flex-shrink-0" />
				<div class="text-sm">
					<div>配置已成功部署到所有 kernels！</div>
					{#if deployTransactionId}
						<div class="mt-1 font-mono text-xs">transaction_id: {deployTransactionId}</div>
					{/if}
				</div>
			</div>
		{:else if deployError}
			<div
				class="mb-3 flex items-start gap-2 rounded bg-error-100 p-3 text-error-700 dark:bg-error-900 dark:text-error-300"
			>
				<AlertCircle class="mt-0.5 h-5 w-5 flex-shrink-0" />
				<div class="text-sm">
					<div class="font-semibold">部署失败</div>
					<pre class="mt-1 text-xs whitespace-pre-wrap">{deployError}</pre>
				</div>
			</div>
		{/if}

		<div class="flex items-center justify-between">
			<div>
				{#if currentStep === 0 && sourceMode !== 'select'}
					<!-- In a sub-step: show Cancel button -->
					<button
						class="preset-ghost-surface btn"
						onclick={handleCancelSubStep}
						disabled={isLoadingSource}
					>
						取消
					</button>
				{:else if currentStep > 0}
					<!-- In step 2: show Back to source button -->
					<button
						class="preset-ghost-surface btn"
						onclick={handleGoBackToSource}
						disabled={isDeploying}
					>
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
					<button class="preset-filled-primary btn" onclick={handleSave} disabled={isDeploying}>
						{#if isDeploying}
							<Loader2 size={16} class="mr-2 animate-spin" />
							部署中...
						{:else}
							<Save size={16} class="mr-2" />
							保存并部署
						{/if}
					</button>
				{/if}
			</div>
		</div>
	</div>
</Steps>
