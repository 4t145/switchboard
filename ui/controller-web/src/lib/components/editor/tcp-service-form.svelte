<script lang="ts">
	import ProviderConfigEditor from './provider-config-editor.svelte';
	import { Code, DeleteIcon, Plus, Trash2 } from '@lucide/svelte';
	import type { FileTcpServiceConfig, FileBind } from '$lib/api/types';
	import { onMount } from 'svelte';
	import SocketAddrEditor from './socket-addr-editor.svelte';
	import TableListEditor, { type RowParams, type ListOperations } from '../common/table-list-editor.svelte';
	type Props = {
		value: FileTcpServiceConfig;
		tlsKeys: string[];
	};
	let { value = $bindable(), tlsKeys = [] }: Props = $props();

	// Mock provider list for now
	const providers = ['http', 'pf'];
	onMount(() => {
		$inspect('TCP Service Form initialized with value:', value);
	});
	function addBind() {
		value.binds = [...value.binds, { bind: '', tls: undefined, description: '' }];
	}

	function removeBind(index: number) {
		value.binds = value.binds.filter((_: unknown, i: number) => i !== index);
	}
</script>

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
		<textarea
			class="textarea-bordered textarea h-20"
			bind:value={value.description}
			placeholder="Description..."
		></textarea>
	</label>

	<!-- Binds & Routes -->
	{#snippet header({}: ListOperations<FileBind>)}
		<tr>
			<th class="w-1/3">Bind Address</th>
			<th class="w-1/3">TLS Config</th>
			<th>Description</th>
			<th class="w-12"></th>
		</tr>
	{/snippet}
	{#snippet row({ value, deleteItem, updateItem }: RowParams<FileBind>)}
		<tr>
			<td class="w-1/3">
				<SocketAddrEditor
					value={value.bind}
					onchange={(e) => updateItem({ ...value, bind: e.currentTarget.value ?? undefined })}
					placeholder="0.0.0.0:80"
					required
				/>
			</td>
			<td class="w-1/3">
				<select
					class="select-sm select"
					value={value.tls}
					onchange={(e) => updateItem({ ...value, tls: e.currentTarget.value ?? undefined })}
				>
					<option value={undefined}></option>
					{#each tlsKeys as key}
						<option value={key}>{key}</option>
					{/each}
				</select>
			</td>
			<td>
				<input
					class="input-sm input"
					type="text"
					value={value.description}
					onchange={(e) => updateItem({ ...value, description: e.currentTarget.value })}
					placeholder="Description..."
				/>
			</td>
			<td class="w-12">
				<button class="btn-icon btn-icon-sm preset-tonal-error" onclick={() => deleteItem()}>
					<DeleteIcon class="size-4" />
				</button>
			</td>
		</tr>
	{/snippet}

	{#snippet footer({ addNewItem }: ListOperations<FileBind>)}
		<tr>
			<td colspan="3"></td>
			<td >
				<button
					type="button"
					class=" btn-icon btn-icon-sm preset-tonal-surface"
					onclick={() => addNewItem({ bind: '', tls: undefined, description: '' })}
				>
					<Plus class="size-4"></Plus></button
				>
			</td>
		</tr>
	{/snippet}
	<h4 class="mb-2 flex items-center gap-2 h4 font-bold">
		Binds
	</h4>
	<TableListEditor
		{row}
		{header}
		{footer}
		value={value.binds}
		onChange={(newBinds) => (value.binds = newBinds)}
	></TableListEditor>

	<h4 class="mb-2 flex items-center gap-2 h4 font-bold">
		<Code size={16} /> Config
	</h4>
	<ProviderConfigEditor bind:value={value.config} bind:provider={value.provider} />
</div>
