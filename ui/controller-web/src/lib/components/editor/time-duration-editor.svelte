<script lang="ts">
	import {
		formatTimeDuration,
		parseTimeDurationFromString,
		type TimeDuration,

		type TimeDurationKind

	} from '$lib/api/types';

	//  Time could be represented as a number of milliseconds, a string (H[n]M[n]S[n]ms), or the special value 'Never'.
	import type { HTMLInputAttributes } from 'svelte/elements';

	type Props = HTMLInputAttributes & {
		value: TimeDuration | undefined;
	};
	let {
		value = $bindable<TimeDuration | undefined>(),
		placeholder = 'e.g., 1h30m, 45s, 500ms, Never',
		...props    
	}: Props = $props();
	let inputElement: HTMLInputElement;
    let inputValue = $state(value ? String(value) : undefined);
	let validationError = $state<Error | null>(null);
	function onblur() {
        if (!inputValue) {
            value = undefined
            return
        };
		const result = parseTimeDurationFromString(inputValue);
		if (result instanceof Error) {
			validationError = result;
			inputElement.setCustomValidity(result.message);
		} else {
			validationError = null;
			inputElement.setCustomValidity('');
			value = formatTimeDuration(result);
		}
	}
</script>

<input
	type="text"
	class={`input-bordered input w-full font-mono ${validationError ? 'bg-error-100 dark:bg-error-900' : ''}`}
    bind:value={inputValue}
	{onblur}
	bind:this={inputElement}
	{placeholder}
	{...props}
/>
{#if validationError}
	<div class="mt-1 text-xs text-error-600 dark:text-error-400">
		{validationError.message}
	</div>
{/if}