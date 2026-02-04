<script lang="ts">
	import { nodeTargetOptionsToValue, type HttpEditorContext } from './types';
	import {
		Combobox,
		Portal,
		type ComboboxRootProps,
		useListCollection
	} from '@skeletonlabs/skeleton-svelte';
	type Props = {
		value: string;
		httpEditorContext: HttpEditorContext;
		onChange: (newValue: string | undefined) => void;
	};
	let { value, httpEditorContext, onChange }: Props = $props<{
		value: string;
		httpEditorContext: HttpEditorContext;
		onChange: (newValue: string | undefined) => void;
	}>();
	const collection = $derived.by(() => {
		$inspect('httpEditorContext.nodeTargetOptions', httpEditorContext.nodeTargetOptions);
		return useListCollection({
			items: httpEditorContext.nodeTargetOptions,
			itemToString: (item) => nodeTargetOptionsToValue(item),
			itemToValue: (item) => nodeTargetOptionsToValue(item)
		});
	});
	const onInputValueChange: ComboboxRootProps['onInputValueChange'] = ({ inputValue }) => {
		if (inputValue === undefined || inputValue === '') {
            onChange(undefined);
		} else {
            onChange(inputValue);
		}
	};
    const onOpenChange: ComboboxRootProps['onOpenChange'] = ({ open }) => {
        console.debug('Combobox open changed:', open);
    };
</script>

<Combobox class="max-w-md" placeholder="Select Node Target" {collection} {onInputValueChange} {onOpenChange} value={[value]}>
	<!-- <Combobox.Label>Label</Combobox.Label> -->
	<Combobox.Control>
		<Combobox.Input />
		<Combobox.Trigger />
	</Combobox.Control>
	<Portal>
		<Combobox.Positioner>
			<Combobox.Content class="z-60">
				{#each httpEditorContext.nodeTargetOptions as item (nodeTargetOptionsToValue(item))}
					<Combobox.Item item={item}>
						<Combobox.ItemText>
							<span>{item.node}</span>
							{#if item.interface && item.interface !== '$default'}
								<span>:</span>
								<span>{item.interface}</span>
							{/if}
						</Combobox.ItemText>
						<Combobox.ItemIndicator />
					</Combobox.Item>
				{/each}
			</Combobox.Content>
		</Combobox.Positioner>
	</Portal>
</Combobox>
