<script lang="ts">
	import NodeTree from './node-tree.svelte';
	import ConfigPanel from './config-panel.svelte';
	import MobileTreeDrawer from './mobile-tree-drawer.svelte';
	import { Settings2, Plus, Menu, X, ArrowRight, ChevronLeft, Info, Zap } from 'lucide-svelte';
	import { UndoManager, createHistoryEntry } from './undo-manager';
	import { KeyboardShortcutManager, createShortcutManager } from './keyboard-shortcuts';
	import { FlowHintSystem, createHintSystem } from './flow-hints';

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

	let selectedId = $state<string | undefined>(undefined);
	let selectedType = $state<'node' | 'filter' | undefined>(undefined);
	let showTreeDrawer = $state(false);
	let showConfigDrawer = $state(false);
	let collapsedTree = $state(false);
	let showShortcutHelp = $state(false);
	let showHints = $state(true);

	let undoManager = $state(new UndoManager());
	let shortcutManager = $state(createShortcutManager());
	let hintSystem = $state(createHintSystem());

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

	$effect(() => {
		shortcutManager.registerHandler('delete', () => {
			if (selectedId && selectedType && !readonly) {
				handleDelete();
			}
		});

		shortcutManager.registerHandler('addNode', () => {
			if (!readonly) {
				handleAdd('node');
			}
		});

		shortcutManager.registerHandler('focusSearch', () => {
			showTreeDrawer = true;
		});

		shortcutManager.registerHandler('close', () => {
			handleClosePanel();
			showShortcutHelp = false;
		});

		shortcutManager.registerHandler('help', () => {
			showShortcutHelp = !showShortcutHelp;
		});

		shortcutManager.registerHandler('undo', () => {
			handleUndo();
		});

		shortcutManager.registerHandler('redo', () => {
			handleRedo();
		});

		return () => {
			shortcutManager.destroy();
		};
	});

	const selectedData = $derived(() => {
		if (!selectedId || !selectedType) return null;

		if (selectedType === 'node') {
			return value.nodes[selectedId];
		} else {
			return value.filters[selectedId];
		}
	});

	let hints = $derived(hintSystem.getAvailableHints(value));

	function handleSelect(id: string, type: 'node' | 'filter') {
		selectedId = id;
		selectedType = type;

		if (window.innerWidth < 768) {
			showConfigDrawer = true;
		}
	}

	function handleClosePanel() {
		selectedId = undefined;
		selectedType = undefined;
		showConfigDrawer = false;
	}

	function handleAdd(type: 'node' | 'filter') {
		const idBase = type === 'node' ? 'node-' : 'filter-';
		let counter = 1;
		let newId = `${idBase}${counter}`;

		const collection = type === 'node' ? value.nodes : value.filters;
		while (collection[newId]) {
			counter++;
			newId = `${idBase}${counter}`;
		}

		const newItem: NodeData = {
			config: {}
		};

		const entry = createHistoryEntry('add', newId, type, null, newItem);
		undoManager.recordAction(entry);

		if (type === 'node') {
			value.nodes[newId] = newItem;
		} else {
			value.filters[newId] = newItem;
		}

		selectedId = newId;
		selectedType = type;

		value = value;
	}

	function handleUpdate(data: NodeData) {
		if (!selectedId || !selectedType) return;

		const oldData = selectedData();
		if (!oldData) return;

		const entry = createHistoryEntry(
			'update',
			selectedId,
			selectedType,
			oldData,
			data
		);
		undoManager.recordAction(entry);

		if (selectedType === 'node') {
			value.nodes[selectedId] = data;
		} else {
			value.filters[selectedId] = data;
		}

		value = value;
	}

	function handleDelete() {
		if (!selectedId || !selectedType) return;

		if (selectedType === 'node') {
			delete value.nodes[selectedId];
			if (value.entrypoint === selectedId) {
				value.entrypoint = '';
			}
		} else {
			delete value.filters[selectedId];
		}

		handleClosePanel();
		value = value;
	}

	function handleUpdateEntrypoint(entrypoint: string) {
		const oldEntrypoint = value.entrypoint;
		const entry = createHistoryEntry(
			'update',
			'flow',
			'flow',
			{ entrypoint: oldEntrypoint },
			{ entrypoint }
		);
		undoManager.recordAction(entry);

		value.entrypoint = entrypoint;
		value = value;
	}

	function handleUndo() {
		try {
			const { flowData } = undoManager.undo(value);
			value = flowData;
		} catch (e) {
			console.warn('Nothing to undo');
		}
	}

	function handleRedo() {
		try {
			const { flowData } = undoManager.redo(value);
			value = flowData;
		} catch (e) {
			console.warn('Nothing to redo');
		}
	}

	function toggleTreeDrawer() {
		showTreeDrawer = !showTreeDrawer;
	}

	function dismissHint(hintId: string) {
		hintSystem.dismissHint(hintId);
	}
</script>

<div class="flex h-full w-full overflow-hidden bg-surface-50-950">
	<div class="flex h-full w-full overflow-hidden">
		<div class="flex h-full w-full overflow-hidden">
			<div
				class="hidden lg:flex flex-col bg-surface-100-900 border-r border-surface-200-800 transition-all duration-300 {collapsedTree
					? 'w-12'
					: 'w-80'}"
			>
				<!-- Fixed Top Header -->
				<header
					class="sidebar-header border-b border-surface-200-800 p-3 {collapsedTree
						? 'items-center justify-center'
						: 'items-center justify-between'}"
				>
					{#if !collapsedTree}
						<span class="text-sm font-semibold">Flow Tree</span>
					{/if}

					<button
						class="btn-icon btn-icon-sm preset-tonal"
						onclick={() => (collapsedTree = !collapsedTree)}
						title={collapsedTree ? 'Expand sidebar' : 'Collapse sidebar'}
					>
						{#if collapsedTree}
							<ArrowRight class="size-4" />
						{:else}
							<ChevronLeft class="size-4" />
						{/if}
					</button>
				</header>

				<!-- Tree Content (Only when expanded) -->
				{#if !collapsedTree}
					<NodeTree
						nodes={value.nodes}
						filters={value.filters}
						entrypoint={value.entrypoint}
						onUpdateNodes={(nodes) => (value.nodes = nodes)}
						onUpdateFilters={(filters) => (value.filters = filters)}
						onUpdateEntrypoint={handleUpdateEntrypoint}
						{selectedId}
						{selectedType}
						onSelect={handleSelect}
						onAdd={readonly ? () => {} : handleAdd}
					/>
				{/if}
			</div>

			<!-- Main Content Area -->
			<div class="flex-1 flex flex-col overflow-hidden min-w-0">
				<header
					class="flex items-center justify-between gap-2 border-b border-surface-200-800 bg-surface-50-950 p-3"
				>
					<div class="flex items-center gap-2">
						<button
							class="lg:hidden btn-icon preset-tonal"
							onclick={toggleTreeDrawer}
							title="Toggle tree"
						>
							{#if showTreeDrawer}
								<X class="size-5" />
							{:else}
								<Menu class="size-5" />
							{/if}
						</button>

						<h2 class="text-sm font-semibold text-surface-900-50">Flow Configuration</h2>

						{#if undoManager.canUndo() || undoManager.canRedo()}
							<div class="flex items-center gap-1 border-l border-surface-200-800 pl-2">
								<button
									class="btn-icon btn-icon-sm preset-tonal"
									onclick={handleUndo}
									disabled={!undoManager.canUndo()}
									title="Undo (Ctrl+Z)"
								>
									⟲
								</button>
								<button
									class="btn-icon btn-icon-sm preset-tonal"
									onclick={handleRedo}
									disabled={!undoManager.canRedo()}
									title="Redo (Ctrl+Y)"
								>
									⟳
								</button>
							</div>
						{/if}

						<button
							class="btn-icon btn-icon-sm preset-tonal"
							onclick={() => (showShortcutHelp = !showShortcutHelp)}
							title="Keyboard shortcuts"
						>
							<Zap class="size-4" />
						</button>
					</div>
				</header>

				<!-- Keyboard Shortcuts Panel -->
				{#if showShortcutHelp}
					<div class="border-b border-surface-200-800 bg-surface-100-900 p-4">
						<h3 class="mb-2 text-sm font-semibold">Keyboard Shortcuts</h3>
						<div class="grid grid-cols-2 gap-2 text-xs">
							<div class="flex justify-between">
								<span>Delete</span>
								<span class="code">Del/Backspace</span>
							</div>
							<div class="flex justify-between">
								<span>Undo</span>
								<span class="code">Ctrl+Z</span>
							</div>
							<div class="flex justify-between">
								<span>Redo</span>
								<span class="code">Ctrl+Y</span>
							</div>
							<div class="flex justify-between">
								<span>Add Node</span>
								<span class="code">Ctrl+N</span>
							</div>
							<div class="flex justify-between">
								<span>Close</span>
								<span class="code">Esc</span>
							</div>
							<div class="flex justify-between">
								<span>Help</span>
								<span class="code">/</span>
							</div>
						</div>
					</div>
				{/if}

				<!-- Hints Panel -->
				{#if showHints && hints.length > 0}
					<div class="border-b border-surface-200-800 bg-surface-50-950 p-2">
						{#each hints as hint (hint.id)}
							<div
								class="card preset-{hint.severity === 'error'
									? 'tonal-error'
									: hint.severity === 'warning'
										? 'tonal-warning'
										: 'tonal-info'} mb-2 p-2"
							>
								<div class="flex items-start gap-2">
									<Info class="size-4 shrink-0 mt-0.5" />
									<div class="flex-1">
										<div class="flex items-start justify-between gap-2">
											<h4 class="text-xs font-semibold">{hint.title}</h4>
											{#if hint.dismissible}
												<button
													class="btn-icon btn-icon-sm preset-tonal"
													onclick={() => dismissHint(hint.id)}
												>
													<X class="size-3" />
												</button>
											{/if}
										</div>
										<p class="text-xs text-surface-400-600">{hint.message}</p>
										{#if hint.actions}
											<div class="mt-1 flex gap-1">
												{#each hint.actions as action}
													<button class="btn btn-xs preset-filled-primary" onclick={action.handler}>
														{action.label}
													</button>
												{/each}
											</div>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Main Content Area -->
				<div class="flex-1 overflow-auto">
					{#if selectedId && selectedType && selectedData()}
						<ConfigPanel
							nodeId={selectedId}
							nodeData={selectedData()!}
							instanceType={selectedType}
							onClose={handleClosePanel}
							onUpdate={handleUpdate}
							onDelete={readonly ? () => {} : handleDelete}
						/>
					{:else}
						<!-- Empty State -->
						<div
							class="flex h-full flex-col items-center justify-center p-8 text-center text-surface-500-400"
						>
							<div class="mb-4 rounded-full preset-tonal p-4">
								<Settings2 class="size-8 text-surface-500-400" />
							</div>
							<h3 class="mb-2 text-lg font-semibold">Select a Node or Filter</h3>
							<p class="mb-6 max-w-md text-sm text-surface-400-600">
								Click on any node or filter in tree view to configure its properties. You can
								also add new nodes or filters using the buttons in the sidebar.
							</p>
							<div class="flex gap-2">
								<button
									class="btn preset-filled-primary"
									disabled={readonly}
									onclick={() => handleAdd('node')}
								>
									<Plus class="size-4" />
									Add Node
								</button>
								<button
									class="btn preset-filled-secondary"
									disabled={readonly}
									onclick={() => handleAdd('filter')}
								>
									<Plus class="size-4" />
									Add Filter
								</button>
							</div>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>

	<!-- Mobile Tree Drawer -->
	<MobileTreeDrawer
		open={showTreeDrawer}
		nodes={value.nodes}
		filters={value.filters}
		entrypoint={value.entrypoint}
		onClose={() => (showTreeDrawer = false)}
		{selectedId}
		{selectedType}
		onSelect={handleSelect}
		onAdd={readonly ? () => {} : handleAdd}
		onUpdateEntrypoint={handleUpdateEntrypoint}
		onUpdateNodes={(nodes) => (value.nodes = nodes)}
		onUpdateFilters={(filters) => (value.filters = filters)}
	/>
</div>
