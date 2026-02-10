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
	let {
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
	let itemViews = $derived(getItemsView());
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
				// Notify selection change
				onSelectionChange(getSelected());
			},
			get selected() {
				return option.value.selected;
			},
			remove() {
				listManager.remove(option.index);
				// Notify selection change
				onSelectionChange(getSelected());
			},
			set value(val: T) {
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
			},
			removeItem(index: number) {
				listManager.remove(index);
				onValueChange(value);
				onSelectionChange(getSelected());
			},
			pushItem(item: T) {
				listManager.pushBack({
					value: item,
					selected: false
				});
			},
			get value() {
				return getValue();
			}
		};
	}
</script>

<!-- expanded mode -->
{@render control(listApi)}
<div>
	{#each itemViews as itemApi (itemApi.index)}
		{@render item(itemApi)}
	{/each}
</div>
