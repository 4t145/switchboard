<script lang="ts">
	type TimeoutDuration = number | string | 'Never';

	export type ReverseProxyConfig = {
		backend: string;
		scheme: string;
		timeout: TimeoutDuration;
		pool_idle_timeout?: TimeoutDuration;
		https_only: boolean;
	};

	type Props = {
		value: ReverseProxyConfig;
		instanceId?: string;
		readonly?: boolean;
	};

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
