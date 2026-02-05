<script lang="ts">
	import { GripVerticalIcon, PlusIcon, XIcon } from '@lucide/svelte';
	import { nodeTargetOptionsToValue, type HttpEditorContext } from './types';
	import {
		Combobox,
		Portal,
		type ComboboxRootProps,
		useListCollection,
		TagsInput
	} from '@skeletonlabs/skeleton-svelte';
	import { dialogQuery } from '$lib/components/dialog';
	type Props = {
		value: string[];
		httpEditorContext: HttpEditorContext;
		onChange: (newValue: string[]) => void;
	};
	let { value, httpEditorContext, onChange }: Props = $props<{
		value: string[];
		httpEditorContext: HttpEditorContext;
		onChange: (newValue: string[]) => void;
	}>();

	type InternalState = {};
	type ItemApi = {
		value: string;

		setValue: (newValue: string) => void;
		remove: () => void;
	};
	const dragging = $state<boolean>(false);
	// --- 拖拽状态 ---
	let hoveringDragHandler = $state<number | null>(null);
	let dragIndex = $state<number | null>(null);
	let dropIndex = $state<number | null>(null); // 视觉上：要在哪个索引“之前”显示占位符
	function addOne(filter: string) {
		const newValue = [...value, filter];
		onChange(newValue);
	}
	function onDragStart(event: DragEvent, index: number) {
		if (!event.dataTransfer) return;
		dragIndex = index;
		event.dataTransfer.effectAllowed = 'move';
		event.dataTransfer.setData('text/plain', index.toString());
		// 某些浏览器需要 setDragImage 防止默认虚影遮挡，这里使用默认行为即可
	}

	function onDragOver(event: DragEvent, targetIndex: number) {
		event.preventDefault(); // 允许 drop
		if (dragIndex === null || dragIndex === targetIndex) return;

		// 逻辑：放在此元素之前 => dropIndex = targetIndex
		dropIndex = targetIndex;
	}

	function onDrop(event: DragEvent, targetIndex: number) {
		event.preventDefault();
		if (dragIndex === null) return;

		const newValue = [...value];
		const [movedItem] = newValue.splice(dragIndex, 1);

		// 计算新的插入位置
		// 如果是从前往后拖 (drag < target)，移除源元素后，目标索引会自动减1，所以我们需要修正
		let insertAt = targetIndex;
		if (dragIndex < targetIndex) {
			insertAt = targetIndex - 1;
		}

		newValue.splice(insertAt, 0, movedItem);
		onChange(newValue);

		cleanupDragState();
	}

	// 处理放在最后的 "+" 按钮上的情况
	function onDropAtEnd(event: DragEvent) {
		event.preventDefault();
		if (dragIndex === null) return;
		const newValue = [...value];
		const [movedItem] = newValue.splice(dragIndex, 1);
		newValue.push(movedItem);
		onChange(newValue);
		cleanupDragState();
	}

	function cleanupDragState() {
		dragIndex = null;
		dropIndex = null;
	}

	async function queryAddOne() {
		const result = await dialogQuery({
			title: 'Select Filter',
			message: queryAddFilter,
			options: {
				confirm: {
					class: 'btn preset-tonal-primary'
				}
			},
			role: 'dialog'
		});
		if (result === 'confirm') {
			addOne(toInputValue!);
		}
	}
	let toInputValue = $state<string>();
	const filterCollection = $derived.by(() => {
		let options = httpEditorContext.filterOptions;
		if (toInputValue !== undefined && toInputValue.trim() !== '') {
			options = httpEditorContext.filterOptions.filter(
				(item) =>
					item.id.toLowerCase().includes(toInputValue!.toLowerCase()) ||
					item.filterClass.toLowerCase().includes(toInputValue!.toLowerCase())
			);
		}
		return useListCollection({
			items: options,
			itemToString: (item) => item.id,
			itemToValue: (item) => item.id,
			groupBy: (item) => item.filterClass
		});
	});
	const onInputValueChange: ComboboxRootProps['onInputValueChange'] = (event) => {
		toInputValue = event.inputValue;
	};
</script>

{#snippet queryAddFilter()}
	<Combobox
		class="max-w-md"
		placeholder="Select Node Target"
		collection={filterCollection}
		{onInputValueChange}
	>
		<!-- <Combobox.Label>Label</Combobox.Label> -->
		<Combobox.Control>
			<Combobox.Input />
			<Combobox.Trigger />
		</Combobox.Control>
		<Portal>
			<Combobox.Positioner>
				<Combobox.Content class="z-60">
					{#each filterCollection.group() as [type, items] (type)}
						<Combobox.ItemGroup>
							<Combobox.ItemGroupLabel>
								{#snippet element(props)}
									<div {...props} class="badge preset-tonal-primary font-mono">{type}</div>
								{/snippet}
							</Combobox.ItemGroupLabel>
							{#each items as item (item.id)}
								<Combobox.Item {item}>
									<Combobox.ItemText>{item.id}</Combobox.ItemText>
									<Combobox.ItemIndicator />
								</Combobox.Item>
							{/each}
						</Combobox.ItemGroup>
					{/each}
				</Combobox.Content>
			</Combobox.Positioner>
		</Portal>
	</Combobox>
{/snippet}
<div class="flex flex-wrap items-center gap-2">
	{#each value as filter, index}
		<!-- Drag Placeholder -->
		{#if dropIndex === index && dragIndex !== index}
			<div class="h-6 w-1 animate-pulse rounded-full bg-primary-500 transition-all"></div>
		{/if}

		<!-- item preview badge -->
		<div
			class="badge preset-filled-surface-900-100 px-0 py-0 transition-all {dragIndex === index
				? 'opacity-40'
				: ''}"
			draggable={hoveringDragHandler === index}
			ondragstart={(event) => onDragStart(event, index)}
			ondragover={(event) => onDragOver(event, index)}
			ondrop={(event) => onDrop(event, index)}
			ondragend={cleanupDragState}
			role="listitem"
		>
			<!-- drag handler -->
			<div
				class="btn-icon btn-icon-sm cursor-grab hover:text-primary-500"
				role="button"
				tabindex="-1"
				onmouseenter={() => (hoveringDragHandler = index)}
				onmouseleave={() => (hoveringDragHandler = null)}
			>
				<GripVerticalIcon />
			</div>
			<!-- filter -->
			<div>{filter}</div>
			<!-- remove button -->
			<button
				type="button"
				class="btn-icon btn-icon-sm hover:text-primary-500"
				onclick={() => {
					onChange(value.filter((_, i) => i !== index));
				}}><XIcon class="size-4"></XIcon></button
			>
		</div>
	{/each}
	<!-- Drag Placeholder -->
	{#if dropIndex === value.length}
		<div class="h-6 w-1 animate-pulse rounded-full bg-primary-500 transition-all"></div>
	{/if}
	<!-- the addon badge -->
	<button
		type="button"
		class="btn-icon btn-icon-sm preset-outlined-surface-500"
		ondragover={(e) => {
			e.preventDefault();
			dropIndex = value.length;
		}}
		ondrop={onDropAtEnd}
		onclick={() => queryAddOne()}><PlusIcon></PlusIcon></button
	>
</div>
