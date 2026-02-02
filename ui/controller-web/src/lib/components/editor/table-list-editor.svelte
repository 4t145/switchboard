<script module lang="ts">
	export type RowParams<T> = {
		value: T;
	} & ItemOperations<T>;
	export type ListOperations<T> = {
		setFilter: (filter: ((item: T) => boolean) | null) => void;
		addNewItem: (item: T) => void;
		updateByIndex: (index: number, newValue: T) => void;
		deleteByIndex: (index: number) => void;
	};
	export type ItemOperations<T> = {
		updateItem: (newValue: T) => void;
		deleteItem: () => void;
	};
	type IndexedItem<T> = { index: number; item: T };
	function withIndex<T>(item: T, index: number): IndexedItem<T> {
		return {
			item,
			index
		};
	}
</script>

<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';

	type Props = {
		value: T[];
		caption?: string;
		onChange?: (value: T[]) => void;
		row: Snippet<[RowParams<T>]>;
		header?: Snippet<[ListOperations<T>]>;
		footer?: Snippet<[ListOperations<T>]>;
	};
	let {
		value = $bindable<T[]>(),
		row,
		header = undefined,
		footer = undefined,
		onChange = undefined
	}: Props = $props();

	let filter = $state<((item: T) => boolean) | null>(null);
	let view = $derived.by(() => {
		return value.map(withIndex).filter((x) => (filter ? filter(x.item) : true));
	});
	function deleteByIndex(index: number) {
		value = value.filter((_, i) => i !== index);
		onChange?.(value);
	}
	function updateByIndex(index: number, newValue: T) {
		value = value.map((item, i) => (i === index ? newValue : item));
		onChange?.(value);
	}
	function addNewItem(item: T) {
		// check for existing key?
		value = [...value, item];
		onChange?.(value);
	}
	const listOperation = {
		setFilter: (newFilter: ((item: T) => boolean) | null) => {
			filter = newFilter;
		},
		addNewItem,
		updateByIndex,
		deleteByIndex
	};
	$effect(() => {
		onChange?.(value);
	});
</script>

<div class="table-wrap">
	<table class="table caption-bottom">

		{#if header}
			<thead>
				{@render header(listOperation)}
			</thead>
		{/if}
		<tbody>
			{#each view as indexedItem (indexedItem.index)}
				{@render row({
					value: indexedItem.item,
					updateItem: (newValue: T) => updateByIndex(indexedItem.index, newValue),
					deleteItem: () => deleteByIndex(indexedItem.index)
				})}
			{/each}
		</tbody>
		{#if footer}
			<tfoot>
				{@render footer(listOperation)}
			</tfoot>
		{/if}
	</table>
</div>
