<script lang="ts">
	import LinkOrValueEditor from './link-or-value-editor.svelte';
	import { Code, Plus, Trash2 } from 'lucide-svelte';
	import type { FileTcpServiceConfig, FileBind } from '$lib/api/types';

	let { value = $bindable(), tlsKeys = [] } = $props<{
		value: FileTcpServiceConfig;
		tlsKeys: string[];
	}>();

	// Mock provider list for now
	const providers = ['Static', 'Kubernetes', 'Consul', 'Custom'];

	function addBind() {
		value.binds = [...value.binds, { bind: '', tls: undefined, description: '' }];
	}

	function removeBind(index: number) {
		value.binds = value.binds.filter((_: unknown, i: number) => i !== index);
	}
</script>

{#snippet jsonConfigSnippet()}
	<!-- Simple JSON editor for the generic 'config' field -->
	<div class="form-control">
		<label class="label">
			<span class="label-text">Configuration (JSON)</span>
		</label>
		<textarea
			class="textarea-bordered textarea h-48 font-mono text-xs"
			value={JSON.stringify(value.config, null, 2)}
			oninput={(e) => {
				try {
					value.config = JSON.parse(e.currentTarget.value);
				} catch (err) {
					// Ignore parse errors while typing
				}
			}}
		></textarea>
		<div class="label">
			<span class="label-text-alt opacity-75"
				>Enter valid JSON configuration for this provider.</span
			>
		</div>
	</div>
{/snippet}

<div class="space-y-6 p-4">
	<!-- Basic Info -->
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<label class="label">
			<span class="label-text">Service Name</span>
			<input class="input" type="text" bind:value={value.name} placeholder="my-service" />
		</label>

		<label class="label">
			<span class="label-text">Provider</span>
			<select class="select" bind:value={value.provider}>
				{#each providers as p}
					<option value={p}>{p}</option>
				{/each}
			</select>
		</label>
	</div>

	<label class="label">
		<span class="label-text">Description</span>
		<textarea class="textarea-bordered textarea h-20" bind:value={value.description}></textarea>
	</label>

	<!-- Binds & Routes -->
	<div
		class="card border border-surface-200 bg-surface-50 p-4 dark:border-surface-700 dark:bg-surface-900/50"
	>
		<div class="mb-4 flex items-center justify-between">
			<h4 class="h4 font-bold">Listeners & Routes</h4>
			<button class="variant-filled-secondary btn btn-sm" onclick={addBind}>
				<Plus size={14} /> Add Bind
			</button>
		</div>

		{#if value.binds.length === 0}
			<div class="py-4 text-center text-sm opacity-50">No listeners configured.</div>
		{/if}

		<div class="space-y-3">
			{#each value.binds as bind, i}
				<div
					class="flex items-start gap-4 card border border-surface-300 p-3 dark:border-surface-600"
				>
					<div class="grid flex-1 grid-cols-1 gap-4 md:grid-cols-2">
						<label class="label">
							<span class="label-text text-xs">Bind Address</span>
							<input
								class="input-sm input"
								type="text"
								bind:value={bind.bind}
								placeholder="0.0.0.0:8080"
							/>
						</label>

						<label class="label">
							<span class="label-text text-xs">TLS Config (Optional)</span>
							<select class="select-sm select" bind:value={bind.tls}>
								<option value={undefined}>None</option>
								{#each tlsKeys as key}
									<option value={key}>{key}</option>
								{/each}
							</select>
						</label>

						<label class="col-span-2 label">
							<span class="label-text text-xs">Description</span>
							<input
								class="input-sm input"
								type="text"
								bind:value={bind.description}
								placeholder="Description..."
							/>
						</label>
					</div>
					<button
						class="variant-soft-error mt-6 btn-icon btn-icon-sm"
						onclick={() => removeBind(i)}
					>
						<Trash2 size={16} />
					</button>
				</div>
			{/each}
		</div>
	</div>

	<!-- Provider Config -->
	<div
		class="card border border-surface-200 bg-surface-50 p-4 dark:border-surface-700 dark:bg-surface-900/50"
	>
		<h4 class="mb-2 flex items-center gap-2 h4 font-bold">
			<Code size={16} /> Provider Config
		</h4>
		<LinkOrValueEditor
			bind:value={value.config}
			dataType="TcpServiceConfig"
			renderValue={jsonConfigSnippet}
			defaultValue={() => ({})}
		/>
	</div>
</div>
