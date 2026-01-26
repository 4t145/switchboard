<script lang="ts">
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import { X } from 'lucide-svelte';
	import NodeTree from './node-tree.svelte';

	type Props = {
		open: boolean;
		nodes: Record<string, any>;
		filters: Record<string, any>;
		entrypoint: string;
		selectedId?: string;
		selectedType?: 'node' | 'filter';
		readonly?: boolean;
		onSelect: (id: string, type: 'node' | 'filter') => void;
		onAdd: (type: 'node' | 'filter') => void;
		onUpdateEntrypoint: (entrypoint: string) => void;
		onUpdateNodes?: (nodes: Record<string, any>) => void;
		onUpdateFilters?: (filters: Record<string, any>) => void;
		onClose?: () => void;
	};

	let {
		open,
		nodes,
		filters,
		entrypoint,
		selectedId,
		selectedType,
		readonly = false,
		onSelect,
		onAdd,
		onUpdateEntrypoint,
		onUpdateNodes,
		onUpdateFilters,
		onClose
	}: Props = $props();

	let internalOpen = $state(open);

	$effect(() => {
		internalOpen = open;
	});

	function handleOpenChange(details: { open: boolean }) {
		if (!details.open && onClose) {
			onClose();
		}
	}
</script>

	<Dialog open={internalOpen} onOpenChange={handleOpenChange}>
		<Portal>
			<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
			<Dialog.Positioner class="fixed inset-0 z-50 flex">
				<Dialog.Content class="h-screen w-full max-w-sm bg-surface-100-900 shadow-xl">
					<header class="flex items-center justify-between border-b border-surface-200-800 p-4">
						<h2 class="text-lg font-semibold">Flow Tree</h2>
						<Dialog.CloseTrigger class="btn-icon preset-tonal">
							<X class="size-5" />
						</Dialog.CloseTrigger>
					</header>

					<div class="flex-1 overflow-auto">
						<NodeTree
							nodes={nodes}
							filters={filters}
							entrypoint={entrypoint}
							onUpdateNodes={onUpdateNodes}
							onUpdateFilters={onUpdateFilters}
							{selectedId}
							{selectedType}
							onSelect={onSelect}
							onAdd={readonly ? () => {} : onAdd}
							onUpdateEntrypoint={onUpdateEntrypoint}
						/>
					</div>
				</Dialog.Content>
			</Dialog.Positioner>
		</Portal>
	</Dialog>
