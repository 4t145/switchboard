<script lang="ts" module>
	import type { Snippet } from 'svelte';
	export type ListApi<T> = {
		setFilter(filter: FilterFn<T> | null): void;
		removeItem(index: number): void;
		pushItem(item: T): void;
		readonly value: T[];
	};
	export type ItemApi<T> = {
		set selected(isSelected: boolean);
		get selected(): boolean;
		remove(): void;
		set value(val: T);
		get value(): T;
		readonly index: number;
	};

	type FilterFn<T> = (item: T) => boolean;
	type Props<T> = {
		id?: string;
		label?: string;
		value: T[];
		selectionMode?: 'single' | 'multiple';
		onSelectionChange?(value: T[]): void;
		onValueChange?(value: T[]): void;
		control: Snippet<[listApi: ListApi<T>]>;
		item: Snippet<[itemApi: ItemApi<T>]>;
	};
</script>

<!-- script/s -->
<script lang="ts" generics="T">
	import { ListManager } from './list-editor/list-manager';
	import { Listbox, useListCollection } from '@skeletonlabs/skeleton-svelte';
	let {
		id = undefined,
		label = undefined,
		value: rawValue,
		selectionMode = 'single',
		onSelectionChange = () => {},
		onValueChange = () => {},
		control,
		item
	}: Props<T> = $props();
	let filter = $state<FilterFn<T> | null>(null);
	// Version signal to trigger reactivity for non-reactive ListManager mutations
	let version = $state(0);

	let listManager = $derived(
		ListManager.new<{
			value: T;
			selected: boolean;
		}>(
			rawValue.map((option) => ({
				value: option,
				selected: false
			}))
		)
	);
	let itemViews = $derived.by(() => {
		version;
		return getItemsView();
	});
	export function getSelected(): T[] {
		return listManager
			.getItems()
			.filter((item) => item.value.selected)
			.map((item) => item.value.value);
	}
	export function getItemsView(): ItemApi<T>[] {
		// Subscribe to version changes
		let items = listManager.getItems();
		let filteredOptions = filter ? items.filter((item, index) => filter!(item.value.value)) : items;
		return filteredOptions.map((option, index) => ({
			set selected(isSelected: boolean) {
				option.value.selected = isSelected;
				if (selectionMode === 'single' && isSelected) {
					// Deselect other items
					items.forEach((otherItem) => {
						if (otherItem !== option) {
							otherItem.value.selected = false;
						}
					});
				}
				version += 1;
				// Notify selection change
				onSelectionChange(getSelected());
			},
			get selected() {
				return option.value.selected;
			},
			remove() {
				listManager.remove(option.index);
				version += 1;
				// Notify selection change
				onSelectionChange(getSelected());
			},
			set value(val: T) {
				version += 1;
				option.value.value = val;
			},
			get value() {
				return option.value.value;
			},
			index
		}));
	}
	const listApi = $derived(getListApi());
	const value = $derived(getValue());
	export function getItemApi(index: number): ItemApi<T> | undefined {
		return undefined;
	}
	export function getValue(): T[] {
		return listManager.getItems().map((item) => item.value.value);
	}
	function getListApi(): ListApi<T> {
		return {
			setFilter(newFilter: FilterFn<T> | null) {
				filter = newFilter;
				version += 1;
			},
			removeItem(index: number) {
				listManager.remove(index);
				onValueChange(value);
				onSelectionChange(getSelected());
				version += 1;
			},
			pushItem(item: T) {
				listManager.pushBack({
					value: item,
					selected: false
				});
				version += 1;
			},
			get value() {
				return getValue();
			}
		};
	}
</script>

<!-- expanded mode -->
<div class="flex flex-col gap-2">
	<div>
		{@render control(listApi)}
	</div>
	<Listbox
		class="w-full max-w-md flex-grow overflow-auto"
		selectionMode={selectionMode}
		onValueChange={
			(e) => {
				const selections = e.value;
				const items = listManager.getItems().filter((item) => selections.includes(item.index));
			}
		}
		collection={useListCollection({
			items: value
		})}
	>
		<Listbox.Content>
			{#each itemViews as itemApi (itemApi.index)}
				<Listbox.Item {item}>
					<Listbox.ItemText>{@render item(itemApi)}</Listbox.ItemText>
					<Listbox.ItemIndicator />
				</Listbox.Item>
			{/each}
		</Listbox.Content>
	</Listbox>
</div>
