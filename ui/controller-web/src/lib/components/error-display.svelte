<script lang="ts">
	import { HttpErrorType, type HttpError } from '$lib/types/http-error';
	import { isRetryableError } from '$lib/utils/http-error-parser';
	import * as m from '$lib/paraglide/messages';

	interface Props {
		error: HttpError;
		onRetry?: () => void;
		onDismiss?: () => void;
		dismissible?: boolean;
		showDetails?: boolean;
		class?: string;
	}

	let {
		error,
		onRetry,
		onDismiss,
		dismissible = false,
		showDetails = true,
		class: className = ''
	}: Props = $props();

	// State for collapsible error stack
	let isStackExpanded = $state(false);

	// Determine if retry button should be shown
	const showRetry = $derived(onRetry !== undefined && isRetryableError(error));

	// Get error icon and color based on error type
	const errorStyle = $derived.by(() => {
		switch (error.type) {
			case HttpErrorType.NETWORK_ERROR:
				return {
					icon: '🔌',
					colorClass: 'variant-soft-warning',
					borderClass: 'border-l-warning-500'
				};
			case HttpErrorType.INTERNAL_ERROR:
				return {
					icon: '⚠️',
					colorClass: 'variant-soft-error',
					borderClass: 'border-l-error-500'
				};
			case HttpErrorType.HTTP_ERROR:
				return {
					icon: '❌',
					colorClass: 'variant-soft-error',
					borderClass: 'border-l-error-500'
				};
			default:
				return {
					icon: '❌',
					colorClass: 'variant-soft-error',
					borderClass: 'border-l-error-500'
				};
		}
	});

	// Get error type label
	const errorTypeLabel = $derived.by(() => {
		switch (error.type) {
			case HttpErrorType.NETWORK_ERROR:
				return m.error_type_network();
			case HttpErrorType.INTERNAL_ERROR:
				return m.error_type_internal();
			case HttpErrorType.HTTP_ERROR:
				return m.error_type_http();
			default:
				return m.error_type_unknown();
		}
	});

	// Format error status
	const statusText = $derived(
		error.status ? `${error.status} ${error.statusText || ''}`.trim() : undefined
	);
</script>

<div class="error-display card p-4 border-l-4 {errorStyle.borderClass} {className}">
	<!-- Error header -->
	<div class="flex items-start justify-between gap-3">
		<div class="flex items-start gap-3 flex-1">
			<!-- Icon -->
			<div class="text-2xl mt-0.5">{errorStyle.icon}</div>

			<!-- Error content -->
			<div class="flex-1 min-w-0">
				<!-- Error type and status -->
				<div class="flex items-center gap-2 flex-wrap mb-1">
					<span class="badge {errorStyle.colorClass} text-xs font-semibold">
						{errorTypeLabel}
					</span>
					{#if statusText}
						<span class="text-xs text-surface-600-300-token font-mono">{statusText}</span>
					{/if}
				</div>

				<!-- Error message -->
				<p class="text-sm font-medium text-surface-900-50-token mb-1">
					{error.message}
				</p>

				<!-- URL if available -->
				{#if error.url && showDetails}
					<p class="text-xs text-surface-600-300-token font-mono truncate" title={error.url}>
						{error.url}
					</p>
				{/if}

				<!-- Error stack (for internal errors) -->
				{#if error.errorStack && showDetails}
					<div class="mt-3">
						<button
							type="button"
							class="btn btn-sm variant-ghost-surface text-xs"
							onclick={() => (isStackExpanded = !isStackExpanded)}
						>
							{isStackExpanded ? '▼' : '▶'}
							{m.error_details_toggle()}
						</button>

						{#if isStackExpanded}
							<div class="mt-2 p-3 bg-surface-900-50-token rounded-md">
								<!-- Error code -->
								{#if error.errorStack.code}
									<div class="mb-2">
										<span class="text-xs font-semibold text-surface-400-600-token">
											{m.error_code()}:
										</span>
										<span class="text-xs font-mono text-surface-300-700-token ml-1">
											{error.errorStack.code}
										</span>
									</div>
								{/if}

								<!-- Stack trace -->
								{#if error.errorStack.stack}
									<div class="mb-2">
										<span class="text-xs font-semibold text-surface-400-600-token block mb-1">
											{m.error_stack_trace()}:
										</span>
										<pre
											class="text-xs font-mono text-surface-300-700-token overflow-x-auto whitespace-pre-wrap break-words">{error.errorStack.stack}</pre>
									</div>
								{/if}

								<!-- Additional details -->
								{#if error.errorStack.details && Object.keys(error.errorStack.details).length > 0}
									<div>
										<span class="text-xs font-semibold text-surface-400-600-token block mb-1">
											{m.error_additional_details()}:
										</span>
										<pre
											class="text-xs font-mono text-surface-300-700-token overflow-x-auto whitespace-pre-wrap break-words">{JSON.stringify(error.errorStack.details, null, 2)}</pre>
									</div>
								{/if}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>

		<!-- Dismiss button -->
		{#if dismissible && onDismiss}
			<button
				type="button"
				class="btn-icon btn-icon-sm variant-ghost-surface"
				onclick={onDismiss}
				aria-label={m.error_dismiss()}
			>
				<span class="text-lg">×</span>
			</button>
		{/if}
	</div>

	<!-- Action buttons -->
	{#if showRetry}
		<div class="flex justify-end mt-3 pt-3 border-t border-surface-300-600-token">
			<button type="button" class="btn btn-sm variant-filled-primary" onclick={onRetry}>
				<span>🔄</span>
				<span>{m.error_retry()}</span>
			</button>
		</div>
	{/if}
</div>

<style>
	.error-display {
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}
</style>
