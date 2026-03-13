<script lang="ts">
	import { type HumanReadableServiceConfig, type FileTcpServiceConfig } from '$lib/api/types';
	import { FileCogIcon, LockIcon, LockOpenIcon } from '@lucide/svelte';
	import ProviderConfigEditor from './editor/provider-config-editor.svelte';
	type Props = {
		config: HumanReadableServiceConfig;
	};
	let { config }: Props = $props();
</script>

{#snippet serviceItem(serviceConfig: FileTcpServiceConfig)}
	<tr>
		<td>{serviceConfig.name}</td>
		<td>
			<span class="badge preset-filled-surface-500">{serviceConfig.provider}</span>
		</td>
		<td>
			{serviceConfig.description}
		</td>
		<td>
			<div class="flex gap-1">
				{#each serviceConfig.binds as bind (bind.bind)}
					{@const isTls = Boolean(bind.tls)}
					{@const Icon = isTls ? LockIcon : LockOpenIcon}
					<span
						class={`chip ${isTls ? 'preset-filled-success-300-700' : 'preset-filled-warning-300-700'}`}
					>
						<Icon class="size-4" />
						{bind.bind}
					</span>
				{/each}
			</div>
		</td>
	</tr>
	<tr>
		<td colspan="4">
		<details>
		    <summary>Config</summary>
			<div class="w-full">
				<ProviderConfigEditor
					value={serviceConfig.config}
					provider={serviceConfig.provider}
					readonly
				></ProviderConfigEditor>
			</div>
		</details>
		</td>
	</tr>
{/snippet}
<div>
	<table class="table">
		<thead>
			<tr>
				<th>Service Name</th>
				<th>Provider</th>
				<th>Description</th>
				<th>Bind Address</th>
			</tr>
		</thead>
		<tbody>
			{#each config.tcp_services as tcp_services (tcp_services.name)}
				{@render serviceItem(tcp_services)}
			{/each}
		</tbody>
	</table>
</div>
