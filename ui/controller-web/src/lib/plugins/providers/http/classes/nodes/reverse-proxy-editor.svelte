<script lang="ts">
	type TimeoutDuration = {
		secs?: number;
		nanos?: number;
	} | string;

	export interface ReverseProxyConfig {
		backend: string;
		scheme: string;
		timeout: TimeoutDuration;
		pool_idle_timeout?: TimeoutDuration;
		https_only: boolean;
	}

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

	// Convert TimeoutDuration to seconds for display
	function durationToSeconds(duration: TimeoutDuration | undefined): number {
		if (!duration) return 30;
		if (typeof duration === 'string') {
			const match = duration.match(/^(\d+)s$/);
			return match ? parseInt(match[1]) : 30;
		}
		return duration.secs || 30;
	}

	// Convert seconds to TimeoutDuration string
	function secondsToDuration(seconds: number): string {
		return `${seconds}s`;
	}

	// Local state for timeout slider
	let timeoutSeconds = $state(durationToSeconds(value.timeout));
	let poolIdleTimeoutSeconds = $state(durationToSeconds(value.pool_idle_timeout));

	// Update value when timeout changes
	$effect(() => {
		value.timeout = secondsToDuration(timeoutSeconds);
	});

	$effect(() => {
		if (poolIdleTimeoutSeconds > 0) {
			value.pool_idle_timeout = secondsToDuration(poolIdleTimeoutSeconds);
		} else {
			value.pool_idle_timeout = undefined;
		}
	});

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
			class="input input-sm"
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
		<select class="select select-sm" bind:value={value.scheme} disabled={readonly}>
			<option value="http">HTTP</option>
			<option value="https">HTTPS</option>
		</select>
	</label>

	<!-- Request Timeout -->
	<div class="space-y-2">
		<label class="label">
			<span class="label-text">Request Timeout: {timeoutSeconds}s</span>
		</label>
		<input
			type="range"
			min="1"
			max="300"
			bind:value={timeoutSeconds}
			class="range range-sm"
			disabled={readonly}
		/>
		<div class="flex justify-between text-xs opacity-60">
			<span>1s</span>
			<span>30s</span>
			<span>60s</span>
			<span>300s</span>
		</div>
	</div>

	<!-- Pool Idle Timeout (Optional) -->
	<div class="space-y-2">
		<label class="label">
			<span class="label-text">
				Pool Idle Timeout: {poolIdleTimeoutSeconds > 0 ? `${poolIdleTimeoutSeconds}s` : 'Default (90s)'}
			</span>
		</label>
		<input
			type="range"
			min="0"
			max="300"
			bind:value={poolIdleTimeoutSeconds}
			class="range range-sm"
			disabled={readonly}
		/>
		<div class="text-xs opacity-60">
			Set to 0 to use default (90s)
		</div>
	</div>

	<!-- HTTPS Only -->
	<label class="label flex items-center gap-2">
		<input
			type="checkbox"
			class="checkbox checkbox-sm"
			bind:checked={value.https_only}
			disabled={readonly}
		/>
		<span class="label-text">HTTPS Only (enforce secure connections)</span>
	</label>
</div>
