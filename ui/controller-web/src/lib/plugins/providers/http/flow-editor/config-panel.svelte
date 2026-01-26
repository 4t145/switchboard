<script lang="ts">
	import { X, Settings, Trash2 } from 'lucide-svelte';
	import { fade } from 'svelte/transition';
	import { getHttpClassEditorPlugin, listHttpClassEditorPlugins } from '$lib/plugins/registry';
	import HttpClassConfigEditor from '$lib/components/editor/http-class-config-editor.svelte';
	import ConfirmModal from './confirm-modal.svelte';

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

	let showDeleteConfirm = $state(false);
	let showChangeClassConfirm = $state(false);

	const availableClasses = $derived(listHttpClassEditorPlugins(instanceType));

	const currentPlugin = $derived(
		nodeData.class ? getHttpClassEditorPlugin(nodeData.class, instanceType) : null
	);

	const outputs = $derived.by<OutputInfo[]>(() => {
		if (!currentPlugin?.extractOutputs || !nodeData.config) {
			return [];
		}

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

	function handleDeleteConfirm() {
		showDeleteConfirm = true;
	}

	function executeDelete() {
		showDeleteConfirm = false;
		onDelete();
	}

	function handleChangeClass() {
		showChangeClassConfirm = true;
	}

	function executeChangeClass() {
		showChangeClassConfirm = false;
		nodeData.class = undefined;
		nodeData.config = {};
		onUpdate(nodeData);
	}
</script>

<div
	class="flex flex-col h-full w-full bg-surface-50-950 border-l border-surface-200-800"
	transition:fade={{ duration: 150 }}
>
	<header class="flex items-center justify-between gap-2 border-b border-surface-200-800 p-4">
		<div class="flex min-w-0 flex-1">
			<h3 class="truncate text-base font-semibold">{nodeId}</h3>
			<span class="badge badge-sm preset-tonal ml-2">
				{instanceType === 'node' ? 'Node' : 'Filter'}
			</span>
		</div>
		<div class="flex items-center gap-1">
			<button
				class="btn-icon btn-icon-sm preset-tonal-error"
				onclick={handleDeleteConfirm}
				title="Delete"
			>
				<Trash2 class="size-4" />
			</button>
			<button class="btn-icon btn-icon-sm preset-tonal" onclick={onClose} title="Close">
				<X class="size-4" />
			</button>
		</div>
	</header>

	<div class="flex-1 overflow-y-auto p-4">
		<div class="space-y-4">
			<div class="space-y-2">
				<label class="label">
					<span class="label-text text-sm font-semibold">Class Type</span>
					{#if !nodeData.class}
						<select class="select select-sm preset-outlined" value="" onchange={handleClassChange}>
							<option value="">-- Select Class --</option>
							{#each availableClasses as plugin}
								<option value={plugin.classId}>{plugin.displayName}</option>
							{/each}
						</select>
					{:else}
						<div class="flex items-center gap-2">
							<span class="code text-sm flex-1">{currentPlugin?.displayName || nodeData.class}</span>
							<button class="btn btn-xs preset-tonal" onclick={handleChangeClass}>
								Change
							</button>
						</div>
					{/if}
				</label>
			</div>

			{#if !nodeData.class}
				<div class="card preset-outlined p-4 space-y-2">
					<p class="text-sm font-medium">Please select a class type</p>
					<p class="text-sm text-surface-400-600">
						Choose a class from the dropdown above to configure this {instanceType}.
					</p>
				</div>
			{:else}
				<div class="space-y-4">
					<div class="divider">Configuration</div>

					{#key nodeId + '-' + nodeData.class}
						<HttpClassConfigEditor
							bind:value={nodeData.config}
							classId={nodeData.class}
							{instanceType}
						/>
					{/key}

					{#if outputs.length > 0}
						<div class="divider">Outputs</div>

						<div class="card preset-outlined p-3 space-y-2">
							<h4 class="flex items-center gap-1 text-xs font-semibold text-surface-400-600">
								<Settings class="size-3" />
								Connection Summary ({outputs.length})
							</h4>
							<div class="space-y-2 text-xs">
								{#each outputs as output}
									<div
										class="flex flex-col gap-0.5 rounded bg-surface-100-900 p-2"
									>
										<div class="flex items-center gap-2">
											<span class="font-medium">{output.label || output.port}</span>
											<span class="text-surface-400-600">→</span>
											<span class="flex-1 text-primary-600-400">
												{output.target || '(not set)'}
											</span>
										</div>
										{#if output.filters && output.filters.length > 0}
											<div class="ml-4 text-xs text-surface-400-600">
												via: {output.filters.join(', ')}
											</div>
										{/if}
									</div>
								{/each}
							</div>
							<p class="italic text-surface-400-600 text-xs">
								Edit outputs in the class configuration above
							</p>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>

<ConfirmModal
	open={showDeleteConfirm}
	title="Delete {instanceType}?"
	message="Are you sure you want to delete '{nodeId}'? This action cannot be undone."
	confirmLabel="Delete"
	cancelLabel="Cancel"
	type="danger"
	onConfirm={executeDelete}
	onCancel={() => (showDeleteConfirm = false)}
/>

<ConfirmModal
	open={showChangeClassConfirm}
	title="Change class type?"
	message="This will reset the configuration for '{nodeId}'. Continue?"
	confirmLabel="Change"
	cancelLabel="Cancel"
	type="warning"
	onConfirm={executeChangeClass}
	onCancel={() => (showChangeClassConfirm = false)}
/>
