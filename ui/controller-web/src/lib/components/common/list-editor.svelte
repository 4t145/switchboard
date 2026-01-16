<script lang="ts" generics="T extends Record<string, any>">
	import { Trash2, Plus, Edit, Search, X } from 'lucide-svelte';

	type Props = {
		items: T[];
		title: string;
		getItemName: (item: T) => string;
		createItem: () => T;
		renderEditor: any; // Snippet
		emptyMessage?: string;
	};

	let {
		items = $bindable(),
		title,
		getItemName,
		createItem,
		renderEditor,
		emptyMessage = 'No items found. Click "Add" to create one.'
	}: Props = $props();

	let editingIndex = $state<number | null>(null);
	let searchQuery = $state<string>('');

	// Filtered items based on search query
	let filteredItems = $derived(
		searchQuery.trim() === ''
			? items.map((item, index) => ({ item, originalIndex: index }))
			: items
					.map((item, index) => ({ item, originalIndex: index }))
					.filter(({ item }) =>
						getItemName(item).toLowerCase().includes(searchQuery.toLowerCase())
					)
	);

	function addItem() {
		const newItem = createItem();
		items = [...items, newItem];
		editingIndex = items.length - 1;
	}

	function deleteItem(index: number) {
		items = items.filter((_: T, i: number) => i !== index);
		
		if (editingIndex === index) {
			editingIndex = null;
		} else if (editingIndex !== null && editingIndex > index) {
			editingIndex--;
		}
	}

	// Auto-select first item and handle list changes
	$effect(() => {
		// Access items to track it
		const _ = items;
		
		// If no item is selected and list is not empty, select first item
		if (editingIndex === null && items.length > 0) {
			editingIndex = 0;
		}
		// Reset editing index when current index is out of bounds
		else if (editingIndex !== null && editingIndex >= items.length) {
			editingIndex = items.length > 0 ? 0 : null;
		}
	});
</script>

<div class="flex h-full gap-6">
	<!-- Sidebar: List -->
	<div
		class="bg-surface-100-800-token border-surface-200-700-token flex w-80 flex-none flex-col card border shadow-sm"
	>
		<div
			class="border-surface-200-700-token flex flex-none items-center justify-between border-b p-4"
		>
			<h3 class="text-lg font-bold">{title}</h3>
			<button class="btn btn-sm preset-filled-primary-500" onclick={addItem}>
				<Plus size={16} /> Add
			</button>
		</div>
		
		<!-- Search Box -->
		<div class="border-surface-200-700-token border-b p-2">
			<div class="relative">
				<Search size={16} class="absolute left-3 top-1/2 -translate-y-1/2 opacity-50" />
				<input
					type="text"
					class="input pl-9 pr-8"
					placeholder="Search..."
					bind:value={searchQuery}
				/>
				{#if searchQuery}
					<button
						class="btn-icon btn-icon-sm absolute right-1 top-1/2 -translate-y-1/2"
						onclick={() => (searchQuery = '')}
					>
						<X size={14} />
					</button>
				{/if}
			</div>
		</div>
		
		<div class="flex-1 space-y-2 overflow-y-auto p-2">
			{#each filteredItems as { item, originalIndex } (originalIndex)}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="rounded-container-token hover:preset-tonal flex cursor-pointer items-center justify-between p-3 transition-all {editingIndex ===
					originalIndex
						? 'preset-tonal-primary'
						: ''}"
					onclick={() => (editingIndex = originalIndex)}
				>
					<span class="flex-1 truncate font-medium">{getItemName(item)}</span>
					<div class="flex gap-1">
						<button
							class="btn-icon btn-icon-sm"
							onclick={(e) => {
								e.stopPropagation();
								deleteItem(originalIndex);
							}}
						>
							<Trash2 size={14} class="text-error-500" />
						</button>
					</div>
				</div>
			{/each}
			{#if filteredItems.length === 0 && items.length > 0}
				<div class="p-8 text-center text-sm opacity-50">
					No items match "{searchQuery}"
				</div>
			{:else if items.length === 0}
				<div class="p-8 text-center text-sm opacity-50">
					{emptyMessage}
				</div>
			{/if}
		</div>
	</div>

	<!-- Editor: Detail -->
	<div
		class="bg-surface-100-800-token border-surface-200-700-token flex flex-1 flex-col overflow-hidden card border shadow-sm"
	>
		{#if editingIndex !== null && items[editingIndex]}
			<div
				class="border-surface-200-700-token bg-surface-200-700-token/50 flex flex-none items-center gap-2 border-b px-4 py-3"
			>
				<Edit size={16} class="opacity-70" />
				<span class="font-bold">Editing: {getItemName(items[editingIndex])}</span>
			</div>

			<div class="flex-1 overflow-y-auto">
				{@render renderEditor(items[editingIndex], editingIndex)}
			</div>
		{:else}
			<div class="flex h-full flex-col items-center justify-center opacity-40">
				<Edit size={48} class="mb-4" />
				<p class="text-lg">Select an item to edit details</p>
			</div>
		{/if}
	</div>
</div>
