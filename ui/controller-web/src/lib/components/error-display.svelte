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

<div class="error-display card border-l-4 p-4 {errorStyle.borderClass} {className}">
	<!-- Error header -->
	<div class="flex items-start justify-between gap-3">
		<div class="flex flex-1 items-start gap-3">
			<!-- Icon -->
			<div class="mt-0.5 text-2xl">{errorStyle.icon}</div>

			<!-- Error content -->
			<div class="min-w-0 flex-1">
				<!-- Error type and status -->
				<div class="mb-1 flex flex-wrap items-center gap-2">
					<span class="badge {errorStyle.colorClass} text-xs font-semibold">
						{errorTypeLabel}
					</span>
					{#if statusText}
						<span class="text-surface-600-300-token font-mono text-xs">{statusText}</span>
					{/if}
				</div>

				<!-- Error message -->
				<p class="text-surface-900-50-token mb-1 text-sm font-medium">
					{error.message}
				</p>

				<!-- URL if available -->
				{#if error.url && showDetails}
					<p class="text-surface-600-300-token truncate font-mono text-xs" title={error.url}>
						{error.url}
					</p>
				{/if}

				<!-- Error stack (for internal errors) -->
				{#if error.errorStack && showDetails}
					<div class="mt-3">
						<button
							type="button"
							class="variant-ghost-surface btn btn-sm text-xs"
							onclick={() => (isStackExpanded = !isStackExpanded)}
						>
							{isStackExpanded ? '▼' : '▶'}
							{m.error_details_toggle()}
						</button>

						{#if isStackExpanded}
							<div class="bg-surface-900-50-token mt-2 rounded-md p-3">
								<!-- Error code -->
								{#if error.errorStack.code}
									<div class="mb-2">
										<span class="text-surface-400-600-token text-xs font-semibold">
											{m.error_code()}:
										</span>
										<span class="text-surface-300-700-token ml-1 font-mono text-xs">
											{error.errorStack.code}
										</span>
									</div>
								{/if}

								<!-- Stack trace -->
								{#if error.errorStack.stack}
									<div class="mb-2">
										<span class="text-surface-400-600-token mb-1 block text-xs font-semibold">
											{m.error_stack_trace()}:
										</span>
										<pre
											class="text-surface-300-700-token overflow-x-auto font-mono text-xs break-words whitespace-pre-wrap">{error
												.errorStack.stack}</pre>
									</div>
								{/if}

								<!-- Additional details -->
								{#if error.errorStack.details && Object.keys(error.errorStack.details).length > 0}
									<div>
										<span class="text-surface-400-600-token mb-1 block text-xs font-semibold">
											{m.error_additional_details()}:
										</span>
										<pre
											class="text-surface-300-700-token overflow-x-auto font-mono text-xs break-words whitespace-pre-wrap">{JSON.stringify(
												error.errorStack.details,
												null,
												2
											)}</pre>
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
				class="variant-ghost-surface btn-icon btn-icon-sm"
				onclick={onDismiss}
				aria-label={m.error_dismiss()}
			>
				<span class="text-lg">×</span>
			</button>
		{/if}
	</div>

	<!-- Action buttons -->
	{#if showRetry}
		<div class="border-surface-300-600-token mt-3 flex justify-end border-t pt-3">
			<button type="button" class="variant-filled-primary btn btn-sm" onclick={onRetry}>
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
