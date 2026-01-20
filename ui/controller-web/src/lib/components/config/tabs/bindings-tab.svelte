<script lang="ts">
	import type { FileTcpServiceConfig } from '$lib/api/types/human_readable';
	import { Network } from 'lucide-svelte';

	interface Props {
		services: FileTcpServiceConfig[];
		jumpToService: (serviceName: string) => void;
		jumpToTls: (tlsName: string) => void;
	}

	let { services, jumpToService, jumpToTls }: Props = $props();

	// Flatten all bindings from all services with their parent service info
	interface BindingRow {
		bindAddress: string;
		serviceName: string;
		provider: string;
		tls: string | null;
		description: string | null;
	}

	const bindings = $derived<BindingRow[]>(
		services.flatMap((service) =>
			service.binds.map((bind) => ({
				bindAddress: bind.bind,
				serviceName: service.name,
				provider: service.provider,
				tls: bind.tls ?? null,
				description: bind.description ?? null
			}))
		)
	);
</script>

<div class="space-y-4">
	<h2 class="h2 flex items-center gap-2">
		<Network class="w-6 h-6" />
		Bindings
		{#if bindings.length > 0}
			<span class="text-sm font-normal text-surface-500 dark:text-surface-400">
				({bindings.length})
			</span>
		{/if}
	</h2>

	{#if bindings.length > 0}
		<div class="card overflow-x-auto">
			<table class="table table-hover">
				<thead>
					<tr>
						<th>Bind Address</th>
						<th>Service</th>
						<th>Provider</th>
						<th>TLS</th>
						<th>Description</th>
					</tr>
				</thead>
				<tbody>
					{#each bindings as binding}
						<tr>
							<td>
								<code class="text-sm bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded">
									{binding.bindAddress}
								</code>
							</td>
							<td>
								<button
									class="text-primary-500 hover:text-primary-600 dark:hover:text-primary-400 underline underline-offset-2 text-left"
									onclick={() => jumpToService(binding.serviceName)}
								>
									{binding.serviceName}
								</button>
							</td>
							<td>
								<code class="text-xs bg-surface-200 dark:bg-surface-700 px-2 py-0.5 rounded">
									{binding.provider}
								</code>
							</td>
							<td>
								{#if binding.tls}
									<button
										class="text-primary-500 hover:text-primary-600 dark:hover:text-primary-400 underline underline-offset-2 text-left"
										onclick={() => jumpToTls(binding.tls!)}
									>
										{binding.tls}
									</button>
								{:else}
									<span class="text-surface-400 dark:text-surface-500 text-sm">-</span>
								{/if}
							</td>
							<td>
								{#if binding.description}
									<span class="text-sm text-surface-600 dark:text-surface-400">
										{binding.description}
									</span>
								{:else}
									<span class="text-surface-400 dark:text-surface-500 text-sm">-</span>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="card p-6 text-center text-surface-500 dark:text-surface-400">
			No bindings configured
		</div>
	{/if}
</div>
