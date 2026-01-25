<script lang="ts">
	import { X, Settings, Trash2 } from 'lucide-svelte';
	import { fade } from 'svelte/transition';
	import { getHttpClassEditorPlugin, listHttpClassEditorPlugins } from '$lib/plugins/registry';
	import HttpClassConfigEditor from '$lib/components/editor/http-class-config-editor.svelte';

	type NodeData = {
		class?: string;
		config: any;
	};

	type OutputInfo = {
		port: string;
		target: string;
		filters?: string[];
		label?: string;
	};

	type Props = {
		nodeId: string;
		nodeData: NodeData;
		instanceType: 'node' | 'filter';
		onClose: () => void;
		onUpdate: (data: NodeData) => void;
		onDelete: () => void;
	};

	let { nodeId, nodeData, instanceType, onClose, onUpdate, onDelete }: Props = $props();

	// 可用的 class 列表
	const availableClasses = $derived(listHttpClassEditorPlugins(instanceType));

	// 当前选中的 plugin
	const currentPlugin = $derived(
		nodeData.class ? getHttpClassEditorPlugin(nodeData.class, instanceType) : null
	);

	// 提取 outputs 用于显示（只有当 config 是对象时才提取）
	const outputs = $derived.by<OutputInfo[]>(() => {
		// Skip if no plugin or no config
		if (!currentPlugin?.extractOutputs || !nodeData.config) {
			return [];
		}

		// Skip if config is a string (file:// reference)
		if (typeof nodeData.config === 'string') {
			return [];
		}

		try {
			return currentPlugin.extractOutputs(nodeData.config);
		} catch (e) {
			console.warn(`Failed to extract outputs for ${nodeData.class}:`, e);
			return [];
		}
	});

	// Handle class selection
	function handleClassChange(e: Event) {
		const target = e.currentTarget as HTMLSelectElement;
		const newClass = target.value;

		if (newClass) {
			const plugin = getHttpClassEditorPlugin(newClass, instanceType);
			if (plugin) {
				nodeData.class = newClass;
				nodeData.config = plugin.createDefaultConfig();
				onUpdate(nodeData);
			}
		}
	}
</script>

<div
	class="config-panel border-l bg-surface-50 dark:bg-surface-900 flex flex-col h-full w-full"
	transition:fade={{ duration: 150 }}
>
	<!-- Header -->
	<div class="header flex items-center justify-between border-b p-4">
		<div class="flex-1 min-w-0">
			<h3 class="text-base font-bold truncate">{nodeId}</h3>
			<p class="text-xs opacity-60">{instanceType === 'node' ? 'Node' : 'Filter'}</p>
		</div>
		<div class="flex items-center gap-1">
			<button class="btn-icon btn-icon-sm preset-filled-error" onclick={onDelete} title="Delete">
				<Trash2 size={16} />
			</button>
			<button class="btn-icon btn-icon-sm" onclick={onClose} title="Close">
				<X size={16} />
			</button>
		</div>
	</div>

	<div class="content flex-1 overflow-y-auto p-4 space-y-4">
		<!-- Class Selector -->
		<div class="space-y-2">
			<label class="label">
				<span class="label-text text-sm font-semibold">Class Type</span>
				{#if !nodeData.class}
					<select class="select select-sm" value="" onchange={handleClassChange}>
						<option value="">-- Select Class --</option>
						{#each availableClasses as plugin}
							<option value={plugin.classId}>{plugin.displayName}</option>
						{/each}
					</select>
				{:else}
					<div class="flex items-center gap-2">
						<span class="code text-sm flex-1">{currentPlugin?.displayName || nodeData.class}</span>
						<button
							class="btn btn-xs preset-tonal-error"
							onclick={() => {
								if (confirm('Change class type? This will reset the configuration.')) {
									nodeData.class = undefined;
									nodeData.config = {};
									onUpdate(nodeData);
								}
							}}
						>
							Change
						</button>
					</div>
				{/if}
			</label>
		</div>

		{#if !nodeData.class}
			<!-- Prompt to select class -->
			<div class="card preset-outlined-warning p-4 space-y-2">
				<p class="text-sm font-medium">Please select a class type</p>
				<p class="text-xs opacity-75">
					Choose a class from the dropdown above to configure this {instanceType}.
				</p>
			</div>
		{:else}
			<!-- Class Editor -->
			<div class="space-y-4">
				<div class="divider">Configuration</div>

				{#key nodeId + '-' + nodeData.class}
					<HttpClassConfigEditor
						bind:value={nodeData.config}
						classId={nodeData.class}
						{instanceType}
					/>
				{/key}

				<!-- Outputs Summary (Read-only) -->
				{#if outputs.length > 0}
					<div class="divider">Outputs</div>

					<div class="card preset-outlined p-3 space-y-2">
						<h4 class="text-xs font-semibold flex items-center gap-1 opacity-75">
							<Settings size={12} />
							Connection Summary ({outputs.length})
						</h4>
						<div class="space-y-2 text-xs">
							{#each outputs as output}
								<div class="flex flex-col gap-0.5 p-2 rounded bg-surface-100 dark:bg-surface-800">
									<div class="flex items-center gap-2">
										<span class="font-medium">{output.label || output.port}</span>
										<span class="opacity-60">→</span>
										<span class="text-primary-600 dark:text-primary-400 flex-1">
											{output.target || '(not set)'}
										</span>
									</div>
									{#if output.filters && output.filters.length > 0}
										<div class="opacity-60 text-xs ml-4">
											via: {output.filters.join(', ')}
										</div>
									{/if}
								</div>
							{/each}
						</div>
						<p class="text-xs opacity-60 italic">
							Edit outputs in the class configuration above
						</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>
