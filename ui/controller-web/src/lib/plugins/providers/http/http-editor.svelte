<script lang="ts">
	import type { HttpConfig } from './types';
	import FlowBuilder from './flow-builder.svelte';
	import { onMount } from 'svelte';

	type Props = {
		value: HttpConfig;
	};

	let { value = $bindable() }: Props = $props();
	onMount(() => {
		console.log('📝 HTTP Config Editor initialized with value:', value);
	});	
	// Initialize value if undefined/null or missing required fields
	if (!value || typeof value !== 'object') {
		value = {
			flow: {
				entrypoint: { node: 'main' },
				instances: {},
				nodes: {},
				filters: {},
				options: {}
			},
			server: {
				version: 'auto'
			}
		};
	} else {
		// Ensure server exists
		if (!value.server) {
			value.server = { version: 'auto' };
		}
		// Ensure flow exists
		if (!value.flow) {
			value.flow = {
				entrypoint: { node: 'main' },
				instances: {},
				nodes: {},
				filters: {},
				options: {}
			};
		}
	}
</script>

<div class="space-y-4">
	<!-- HTTP Version Selector -->
	<label class="label">
		<span class="label-text font-medium">HTTP Version</span>
		<select class="select select-sm" bind:value={value.server.version}>
			<option value="auto">Auto (Negotiate)</option>
			<option value="http1">HTTP/1.1 Only</option>
			<option value="http2">HTTP/2 Only</option>
		</select>
	</label>

	<!-- Flow Builder (Visual Editor) -->
	<div class="space-y-2">
		<label class="label-text font-medium">Flow Configuration</label>
		<div class="card border border-surface-200 overflow-hidden dark:border-surface-700">
			<FlowBuilder bind:value={value.flow} />
		</div>
		<div class="label">
			<span class="label-text-alt opacity-75">
				Use the visual editor to build your HTTP request flow. Add nodes and filters, connect them, and configure each component.
			</span>
		</div>
	</div>
</div>
