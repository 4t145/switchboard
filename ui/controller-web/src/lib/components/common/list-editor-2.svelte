<script lang="ts" module>
	export type ListApi<T> = {
		addItem(item: T): void;
		deleteItem(index: number): void;
		value: T[];
	};
	export type ItemApi<T> = {
		delete(): void;
		onValueChange(value: T): void;
		index: number;
		value: T;
	};
	type Props<T> = {
		value: T[];
		onValueChange(value: T[]): void;

		editor: import('svelte').Snippet<[ItemApi<T>]>;
		empty?: import('svelte').Snippet<[]>;
	};
</script>

<script lang="ts" generics="T">
	import { Listbox, useListCollection } from '@skeletonlabs/skeleton-svelte';
	let { value, editor, onValueChange, empty = defaultEmpty }: Props<T> = $props();
	let query = $state('');

	const collection = $derived(
		useListCollection({
			items: data.filter((item) => item.label.toLowerCase().includes(query.toLowerCase())),
			itemToString: (item) => item.label,
			itemToValue: (item) => item.value
		})
	);

	function getItemApi(index: number): ItemApi<T> | undefined {
        return undefined
    }

	let selectedIndex: number | null = $state(null);
</script>

{#snippet defaultEmpty()}
	...
{/snippet}
<!-- expanded mode -->
<div>
	<Listbox class="w-full max-w-md" {collection}>
		<Listbox.Label>Search for Food</Listbox.Label>
		<Listbox.Input
			placeholder="Type to search..."
			value={query}
			oninput={(e) => (query = e.currentTarget.value)}
		/>
		<Listbox.Content>
			{#each collection.items as item (item.value)}
				<Listbox.Item {item}>
					<Listbox.ItemText>{item.label}</Listbox.ItemText>
					<Listbox.ItemIndicator />
				</Listbox.Item>
			{/each}
		</Listbox.Content>
	</Listbox>
	<!-- editor -->
	<div>
		{#if selectedIndex !== null}
			{@const itemApi = getItemApi(selectedIndex)}
			{#if itemApi}
				{@render editor(itemApi)}
			{/if}
		{:else}
			{@render empty()}
		{/if}
	</div>
</div>
