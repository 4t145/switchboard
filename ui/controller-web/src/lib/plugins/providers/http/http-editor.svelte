<script lang="ts">
	import type { HttpConfig } from './types';
	import FlowEditor from './flow-editor/flow-editor.svelte';
	import { untrack } from 'svelte';

	type Props = {
		value: HttpConfig;
	};

	let { value = $bindable() }: Props = $props();

	// FlowEditor 内部使用的简化数据结构
	type FlowEditorData = {
		entrypoint: string;
		nodes: Record<string, { class?: string; config: any }>;
		filters: Record<string, { class?: string; config: any }>;
	};

	// Safe deep clone using JSON (works with Svelte proxies)
	function deepClone<T>(obj: T): T {
		try {
			return JSON.parse(JSON.stringify(obj));
		} catch (e) {
			console.warn('Failed to deep clone object:', e);
			return obj;
		}
	}

	// 将 FlowConfig 转换为 FlowEditorData
	function toEditorFormat(flowConfig: any): FlowEditorData {
		const result: FlowEditorData = {
			entrypoint: '',
			nodes: {},
			filters: {}
		};

		// Convert entrypoint (NodeTarget -> string)
		if (flowConfig?.entrypoint) {
			if (typeof flowConfig.entrypoint === 'string') {
				result.entrypoint = flowConfig.entrypoint;
			} else if (flowConfig.entrypoint?.node) {
				result.entrypoint = flowConfig.entrypoint.node;
			}
		}

		// Convert nodes - handle both object and string (LinkOrValue) formats
		if (flowConfig?.nodes) {
			for (const [nodeId, nodeValue] of Object.entries(flowConfig.nodes)) {
				if (typeof nodeValue === 'string') {
					// This is a LinkOrValue reference (e.g., "file://...")
					result.nodes[nodeId] = {
						config: nodeValue
					};
				} else if (nodeValue && typeof nodeValue === 'object') {
					// This is a full node definition - ensure it has a config field
					const cloned = deepClone(nodeValue) as any;
					result.nodes[nodeId] = {
						class: cloned.class,
						config: cloned.config || {}
					};
				} else {
					// Fallback for undefined/null
					result.nodes[nodeId] = {
						config: {}
					};
				}
			}
		}

		// Convert filters - handle both object and string (LinkOrValue) formats
		if (flowConfig?.filters) {
			for (const [filterId, filterValue] of Object.entries(flowConfig.filters)) {
				if (typeof filterValue === 'string') {
					// This is a LinkOrValue reference
					result.filters[filterId] = {
						config: filterValue
					};
				} else if (filterValue && typeof filterValue === 'object') {
					// This is a full filter definition - ensure it has a config field
					const cloned = deepClone(filterValue) as any;
					result.filters[filterId] = {
						class: cloned.class,
						config: cloned.config || {}
					};
				} else {
					// Fallback for undefined/null
					result.filters[filterId] = {
						config: {}
					};
				}
			}
		}

		return result;
	}

	// 将 FlowEditorData 转换回 FlowConfig
	function fromEditorFormat(editorData: FlowEditorData): any {
		return {
			entrypoint: editorData.entrypoint ? { node: editorData.entrypoint } : { node: '' },
			nodes: deepClone(editorData.nodes),
			filters: deepClone(editorData.filters)
		};
	}

	// Reactive flow editor data
	let flowEditorData = $state<FlowEditorData>(toEditorFormat(value.flow || {}));

	// Use a flag to prevent circular updates
	let isUpdatingFromEditor = false;
	let isUpdatingFromValue = false;

	// Initialize and sync from value.flow to flowEditorData when value changes externally
	$effect(() => {
		// Ensure basic structure
		if (!value.server) {
			value.server = {
				version: 'auto'
			};
		}

		if (!value.flow) {
			value.flow = {
				entrypoint: { node: '' },
				nodes: {},
				filters: {}
			};
		}

		// Skip if we're in the middle of updating from editor
		if (isUpdatingFromEditor) return;

		// Read value.flow and convert to editor format
		// Use untrack to prevent this effect from re-running when we update flowEditorData
		isUpdatingFromValue = true;
		flowEditorData = toEditorFormat(untrack(() => value.flow));
		isUpdatingFromValue = false;
	});

	// Sync changes back from flowEditorData to value.flow
	$effect(() => {
		// Skip if we're in the middle of updating from value
		if (isUpdatingFromValue) return;

		// Convert editor data back to flow config
		isUpdatingFromEditor = true;
		const newFlowConfig = fromEditorFormat(flowEditorData);
		
		// Use untrack to read current value without creating dependency
		const currentFlow = untrack(() => value.flow);
		
		// Only update if there's an actual change
		// Compare serialized versions to avoid reference comparison issues
		if (JSON.stringify(newFlowConfig) !== JSON.stringify(currentFlow)) {
			value.flow = newFlowConfig;
		}
		
		isUpdatingFromEditor = false;
	});
</script>

<div class="space-y-4">
	<!-- HTTP Version Selector -->
	<label class="label">
		<span class="label-text font-medium">HTTP Version</span>
		{#if value.server}
			<select class="select select-sm" bind:value={value.server.version}>
				<option value="auto">Auto (Negotiate)</option>
				<option value="http1">HTTP/1.1 Only</option>
				<option value="http2">HTTP/2 Only</option>
			</select>
		{/if}
	</label>

	<!-- Flow Editor (Tree-based Visual Editor) -->
	<div class="space-y-2">
		<div class="label-text font-medium">Flow Configuration</div>
		<div class="card border border-surface-200 dark:border-surface-700 h-[600px] overflow-hidden">
			<FlowEditor bind:value={flowEditorData} />
		</div>
		<div class="label">
			<span class="label-text-alt opacity-75">
				Configure your HTTP request flow by adding nodes and filters. Select a node or filter from
				the tree to edit its configuration.
			</span>
		</div>
	</div>
</div>
