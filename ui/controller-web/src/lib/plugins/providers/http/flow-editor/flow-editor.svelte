<script lang="ts">
	import NodeTree from './node-tree.svelte';
	import ConfigPanel from './config-panel.svelte';

	type NodeData = {
		class?: string;
		config: any;
	};

	type FlowData = {
		entrypoint: string;
		nodes: Record<string, NodeData>;
		filters: Record<string, NodeData>;
	};

	type Props = {
		value: FlowData;
		readonly?: boolean;
	};

	let { value = $bindable(), readonly = false }: Props = $props();

	// Selection state
	let selectedId = $state<string | undefined>(undefined);
	let selectedType = $state<'node' | 'filter' | undefined>(undefined);

	// Ensure flow structure
	$effect(() => {
		if (!value || typeof value !== 'object') {
			value = {
				entrypoint: '',
				nodes: {},
				filters: {}
			};
		}

		if (!value.entrypoint) value.entrypoint = '';
		if (!value.nodes) value.nodes = {};
		if (!value.filters) value.filters = {};
	});

	// Get selected data
	const selectedData = $derived(() => {
		if (!selectedId || !selectedType) return null;

		if (selectedType === 'node') {
			return value.nodes[selectedId];
		} else {
			return value.filters[selectedId];
		}
	});

	// Handlers
	function handleSelect(id: string, type: 'node' | 'filter') {
		selectedId = id;
		selectedType = type;
	}

	function handleClosePanel() {
		selectedId = undefined;
		selectedType = undefined;
	}

	function handleAdd(type: 'node' | 'filter') {
		const idBase = type === 'node' ? 'node-' : 'filter-';
		let counter = 1;
		let newId = `${idBase}${counter}`;

		// Find unique ID
		const collection = type === 'node' ? value.nodes : value.filters;
		while (collection[newId]) {
			counter++;
			newId = `${idBase}${counter}`;
		}

		// Create new item
		const newItem: NodeData = {
			config: {}
		};

		if (type === 'node') {
			value.nodes[newId] = newItem;
		} else {
			value.filters[newId] = newItem;
		}

		// Select the new item
		selectedId = newId;
		selectedType = type;

		// Trigger reactivity
		value = value;
	}

	function handleUpdate(data: NodeData) {
		if (!selectedId || !selectedType) return;

		if (selectedType === 'node') {
			value.nodes[selectedId] = data;
		} else {
			value.filters[selectedId] = data;
		}

		// Trigger reactivity
		value = value;
	}

	function handleDelete() {
		if (!selectedId || !selectedType) return;

		const confirmMsg = `Delete ${selectedType} "${selectedId}"?`;
		if (confirm(confirmMsg)) {
			if (selectedType === 'node') {
				delete value.nodes[selectedId];
				// Clear entrypoint if it was deleted
				if (value.entrypoint === selectedId) {
					value.entrypoint = '';
				}
			} else {
				delete value.filters[selectedId];
			}

			// Close panel
			handleClosePanel();

			// Trigger reactivity
			value = value;
		}
	}

	function handleUpdateEntrypoint(entrypoint: string) {
		value.entrypoint = entrypoint;
		value = value;
	}
</script>

<div class="flow-editor flex h-full">
	<!-- Left: Tree View -->
	<div class="tree-container flex-1 border-r bg-surface-100 dark:bg-surface-900 overflow-y-auto">
		<NodeTree
			bind:nodes={value.nodes}
			bind:filters={value.filters}
			bind:entrypoint={value.entrypoint}
			{selectedId}
			{selectedType}
			onSelect={handleSelect}
			onAdd={readonly ? () => {} : handleAdd}
			onUpdateEntrypoint={handleUpdateEntrypoint}
		/>
	</div>

	<!-- Right: Config Panel (conditionally rendered) -->
	{#if selectedId && selectedType && selectedData()}
		<ConfigPanel
			nodeId={selectedId}
			nodeData={selectedData()!}
			instanceType={selectedType}
			onClose={handleClosePanel}
			onUpdate={handleUpdate}
			onDelete={readonly ? () => {} : handleDelete}
		/>
	{/if}
</div>

<style>
	.flow-editor {
		position: relative;
		overflow: hidden;
	}

	.tree-container {
		min-width: 300px;
		max-width: 600px;
	}
</style>
