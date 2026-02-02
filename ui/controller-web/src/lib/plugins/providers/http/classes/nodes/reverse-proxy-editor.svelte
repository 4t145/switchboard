<script lang="ts">
	import type { TimeDuration } from "$lib/api/types";
	import TimeDurationEditor from "$lib/components/editor/time-duration-editor.svelte";
	import type { HttpClassEditorProps } from "$lib/plugins/types";

	export type ReverseProxyConfig = {
		backend: string;
		scheme: string;
		timeout: TimeDuration;
		pool_idle_timeout?: TimeDuration;
		https_only: boolean;
	};
	
	type Props = HttpClassEditorProps<ReverseProxyConfig>;

	let { value = $bindable(), readonly = false }: Props = $props();

	// Initialize with defaults if needed
	if (!value) {
		value = {
			backend: '',
			scheme: 'http',
			timeout: '30s',
			https_only: false
		};
	}

	// Validation
	let backendError = $derived.by(() => {
		if (!value.backend) return 'Backend address is required';
		if (!value.backend.includes(':')) return 'Backend must include port (e.g., example.com:8080)';
		return null;
	});
</script>

<div class="space-y-4">
	<!-- Backend Address -->
	<label class="label">
		<span class="label-text">Backend Address</span>
		<input
			class="input-sm input"
			class:input-error={backendError}
			type="text"
			bind:value={value.backend}
			required
			placeholder="example.com:8080"
			disabled={readonly}
		/>
	</label>
	{#if backendError}
		<div class="text-xs text-error-600 dark:text-error-400">{backendError}</div>
	{/if}

	<!-- Scheme -->
	<label class="label">
		<span class="label-text">Scheme</span>
		<select class="select-sm select" bind:value={value.scheme} disabled={readonly}>
			<option value="http">HTTP</option>
			<option value="https">HTTPS</option>
		</select>
	</label>
	<label class="label">
		<span class="label-text">Timeout</span>
		<TimeDurationEditor bind:value={value.timeout} disabled={readonly} required />
	</label>
	<label class="label">
		<span class="label-text">Pool Idle Timeout (optional)</span>
		<TimeDurationEditor bind:value={value.pool_idle_timeout} disabled={readonly} />
	</label>
	<!-- HTTPS Only -->
	<label class="label flex items-center gap-2">
		<input
			type="checkbox"
			class="checkbox-sm checkbox"
			bind:checked={value.https_only}
			disabled={readonly}
		/>
		<span class="label-text">HTTPS Only (enforce secure connections)</span>
	</label>
</div>
