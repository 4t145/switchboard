<script lang="ts">
	import type { HttpClassEditorProps } from '$lib/plugins/types';

	type Props = HttpClassEditorProps;

	let { value = $bindable(), readonly = false }: Props = $props();

	// Initialize structure if needed
	$effect(() => {
		if (!value || typeof value !== 'object') {
			value = {
				hostname: {},
				path: {},
				output: {}
			};
		}

		if (!value.hostname) value.hostname = {};
		if (!value.path) value.path = {};
		if (!value.output) value.output = {};
	});

	// Helper to add new route
	function addHostnameRule() {
		const key = prompt('Enter hostname pattern (e.g., "api.example.com", "*.example.com"):');
		if (key && key.trim()) {
			value.hostname[key.trim()] = '';
		}
	}

	function addPathRule() {
		const key = prompt('Enter path pattern (e.g., "/api/*", "/v1/users"):');
		if (key && key.trim()) {
			value.path[key.trim()] = '';
		}
	}

	function addOutputPort() {
		const port = prompt('Enter output port name (e.g., "api", "frontend"):');
		if (port && port.trim()) {
			value.output[port.trim()] = {
				target: '',
				filters: []
			};
		}
	}

	function removeHostnameRule(key: string) {
		delete value.hostname[key];
		value = value; // Trigger reactivity
	}

	function removePathRule(key: string) {
		delete value.path[key];
		value = value;
	}

	function removeOutputPort(port: string) {
		delete value.output[port];
		value = value;
	}

	function addFilter(port: string) {
		const filterId = prompt('Enter filter ID:');
		if (filterId && filterId.trim()) {
			if (!value.output[port].filters) {
				value.output[port].filters = [];
			}
			value.output[port].filters.push(filterId.trim());
			value = value;
		}
	}

	function removeFilter(port: string, index: number) {
		value.output[port].filters.splice(index, 1);
		value = value;
	}
</script>

<div class="space-y-6">
	<!-- Hostname Routes -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<label class="label-text text-sm font-semibold">Hostname Routes</label>
			{#if !readonly}
				<button class="btn btn-sm preset-tonal-primary" onclick={addHostnameRule}>
					Add Rule
				</button>
			{/if}
		</div>

		<div class="space-y-1">
			{#if Object.keys(value.hostname || {}).length === 0}
				<p class="text-xs opacity-60">No hostname rules defined</p>
			{:else}
				{#each Object.entries(value.hostname) as [pattern, port]}
					<div class="flex items-center gap-2">
						<input
							type="text"
							class="input input-sm flex-1"
							value={pattern}
							disabled
							placeholder="Hostname pattern"
						/>
						<span class="text-xs opacity-60">→</span>
						<input
							type="text"
							class="input input-sm flex-1"
							bind:value={value.hostname[pattern]}
							{readonly}
							placeholder="Output port"
						/>
						{#if !readonly}
							<button
								class="btn-icon btn-icon-sm preset-filled-error"
								onclick={() => removeHostnameRule(pattern)}
							>
								×
							</button>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</div>

	<!-- Path Routes -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<label class="label-text text-sm font-semibold">Path Routes</label>
			{#if !readonly}
				<button class="btn btn-sm preset-tonal-primary" onclick={addPathRule}>
					Add Rule
				</button>
			{/if}
		</div>

		<div class="space-y-1">
			{#if Object.keys(value.path || {}).length === 0}
				<p class="text-xs opacity-60">No path rules defined</p>
			{:else}
				{#each Object.entries(value.path) as [pattern, port]}
					<div class="flex items-center gap-2">
						<input
							type="text"
							class="input input-sm flex-1"
							value={pattern}
							disabled
							placeholder="Path pattern"
						/>
						<span class="text-xs opacity-60">→</span>
						<input
							type="text"
							class="input input-sm flex-1"
							bind:value={value.path[pattern]}
							{readonly}
							placeholder="Output port"
						/>
						{#if !readonly}
							<button
								class="btn-icon btn-icon-sm preset-filled-error"
								onclick={() => removePathRule(pattern)}
							>
								×
							</button>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</div>

	<!-- Output Definitions -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<label class="label-text text-sm font-semibold">Output Ports</label>
			{#if !readonly}
				<button class="btn btn-sm preset-tonal-primary" onclick={addOutputPort}>
					Add Output
				</button>
			{/if}
		</div>

		<div class="space-y-3">
			{#if Object.keys(value.output || {}).length === 0}
				<p class="text-xs opacity-60">No output ports defined</p>
			{:else}
				{#each Object.entries(value.output) as [port, outputDef]: [string, any]}
					<div class="card preset-outlined p-3 space-y-2">
						<div class="flex items-center justify-between">
							<span class="text-sm font-medium">{port}</span>
							{#if !readonly}
								<button
									class="btn-icon btn-icon-sm preset-filled-error"
									onclick={() => removeOutputPort(port)}
								>
									×
								</button>
							{/if}
						</div>

						<!-- Target Node -->
						<label class="label">
							<span class="label-text text-xs">Target Node</span>
							<input
								type="text"
								class="input input-sm"
								bind:value={value.output[port].target}
								{readonly}
								placeholder="node-id"
							/>
						</label>

						<!-- Filters -->
						<div class="space-y-1">
							<div class="flex items-center justify-between">
								<span class="label-text text-xs">Filters</span>
								{#if !readonly}
									<button
										class="btn btn-xs preset-tonal-secondary"
										onclick={() => addFilter(port)}
									>
										Add Filter
									</button>
								{/if}
							</div>

							{#if !value.output[port].filters || value.output[port].filters.length === 0}
								<p class="text-xs opacity-60">No filters</p>
							{:else}
								<div class="space-y-1">
									{#each value.output[port].filters as filter, index}
										<div class="flex items-center gap-2">
											<input
												type="text"
												class="input input-xs flex-1"
												bind:value={value.output[port].filters[index]}
												{readonly}
												placeholder="filter-id"
											/>
											{#if !readonly}
												<button
													class="btn-icon btn-icon-sm preset-filled-error"
													onclick={() => removeFilter(port, index)}
												>
													×
												</button>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>
