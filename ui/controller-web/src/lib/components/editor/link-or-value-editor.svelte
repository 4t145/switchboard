<script lang="ts" generics="T=unknown">
	import { api } from '$lib/api/routes';
	import {
		Link as LinkIcon,
		ArrowRightLeft,
		FileText,
		Globe,
		ParenthesesIcon,
		SearchIcon,
		DatabaseIcon,
		EditIcon,
		ArrowRightIcon,
		XIcon
	} from '@lucide/svelte';
	import ObjectPages from '$lib/components/object-pages.svelte';
	import LinkOrValueDisplay from '$lib/components/config/link-or-value-display.svelte';
	import type { StorageObjectDescriptor } from '$lib/api/types';
	import {
		FloatingPanel,
		Portal,
		SegmentedControl,
		ToggleGroup
	} from '@skeletonlabs/skeleton-svelte';
	import DataTypeRenderer from '$lib/data-types/components/data-type-renderer.svelte';
	import { dataTypeRegistry } from '$lib/data-types/registry';
	import {
		parseLink,
		formatLink,
		isLinkValue,
		type LinkKind,
		getScheme,
		type ParsedLink
	} from '$lib/utils/link-parser';
	import { untrack, type Snippet } from 'svelte';

	type Props<T = unknown> = {
		value: T | string;
		valueDataFormat: 'string' | 'object';
		getDefaultLinkValue?: () => string;
		getDefaultInlineValue: () => T;
		editor: Snippet<[T, (saveValue: T) => void]>;
	};
	type EditorState =
		| {
				mode: 'reference';
				parsedLink: ParsedLink;
				remoteValue:
					| {
							status: 'loading';
					  }
					| {
							status: 'loaded';
							value: T;
					  }
					| {
							status: 'error';
							error: Error;
					  };
		  }
		| {
				mode: 'inline';
		  };
	type EditorMode = EditorState['mode'];
	type SelectedLinkScheme = 'storage' | 'file' | 'http' | 'https';
	const DEFAULT_LINK_VALUE = 'storage://#';
	let {
		value = $bindable(),
		valueDataFormat,
		getDefaultLinkValue = () => DEFAULT_LINK_VALUE,
		getDefaultInlineValue,
		editor
	}: Props<T> = $props();
	async function resolveLinkToValue(link: string) {
		try {
			let resolvedValue: T;
			if (valueDataFormat === 'object') {
				resolvedValue = (await api.resolve.link_to_object(link)) as T;
			} else {
				resolvedValue = (await api.resolve.link_to_string(link)) as T;
			}
			if (editorState.mode === 'reference' && editorState.remoteValue.status === 'loading') {
				editorState = {
					...editorState,
					remoteValue: {
						status: 'loaded',
						value: resolvedValue
					}
				};
			}
			return;
		} catch (e) {
			let error: Error;
			if (e instanceof Error) {
				error = e;
			} else {
				error = new Error(`Unknown error occurred while resolving link ${e}`);
			}
			if (editorState.mode === 'reference' && editorState.remoteValue.status === 'loading') {
				editorState = {
					...editorState,
					remoteValue: {
						status: 'error',
						error
					}
				};
			}
			return;
		}
	}
	let editorState: EditorState = $derived.by(() => {
		const parsedLink = parseLink(value);
		if (parsedLink !== null) {
			return { mode: 'reference', remoteValue: { status: 'loading' }, parsedLink };
		} else {
			return { mode: 'inline' };
		}
	});
	$effect(() => {
		if (editorState.mode === 'reference' && editorState.remoteValue.status === 'loading') {
			untrack(() => resolveLinkToValue(value as string));
		}
	});
	let cachedLinkValue = $state<string | undefined>(undefined);
	let parsedLink = $derived.by<ParsedLink | null>(() => {
		return parseLink(value);
	});
	let isEditingLink = $state(false);
	let selectedLinkScheme = $state<SelectedLinkScheme>();
	let cachedInlineValue = $state<T | undefined>(undefined);
	let linkLocationInput = $state<string>();
	let linkLocationInputRef: HTMLInputElement | undefined = $state();
	let editingReferenceValue = $state<T>();
	const defaultInlineValue = $derived(getDefaultInlineValue());
	function isInline(val: T | string): val is T {
		return !isLinkValue(val);
	}
	function switchEditorMode(mode: EditorMode) {
		if (mode === 'reference') {
			cachedInlineValue = value as T;
			if (cachedLinkValue) {
				value = cachedLinkValue;
			} else {
				value = getDefaultLinkValue();
			}
		} else {
			cachedLinkValue = value as string;
			if (cachedInlineValue) {
				value = cachedInlineValue;
			} else {
				value = getDefaultInlineValue();
			}
		}
	}
	function resetLinkScheme(scheme: SelectedLinkScheme) {
		if (isInline(value)) return;
	}
	function editLink() {
		if (editorState.mode !== 'reference') return;
		if (linkLocationInput === undefined) {
			linkLocationInput = parsedLink?.location ?? '';
		}
		if (selectedLinkScheme === undefined) {
			selectedLinkScheme = (parsedLink?.scheme as SelectedLinkScheme) ?? 'storage';
		}
		isEditingLink = true;
	}
	$effect(() => {
		if (isEditingLink) {
			setTimeout(() => {
				linkLocationInputRef?.focus();
				linkLocationInputRef?.select();
			}, 0);
		}
	});
	function quitEditLink() {
		isEditingLink = false;
	}
	let lastEscapePress: number = $state(0);
	function handleLinkInputKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			const timeNow = Date.now();
			console.debug('Escape key pressed in link input');
			if (lastEscapePress && timeNow - lastEscapePress < 500) {
				// Double escape detected, cancel editing
				lastEscapePress = 0;
				quitEditLink();
			} else {
				// Record the time of this escape press
				console.debug('First escape press detected');
				lastEscapePress = timeNow;
			}
		} else if (event.key === 'Enter') {
			event.preventDefault();
			// 触发你的保存逻辑
		}
	}
</script>

<div class="space-y-4 card">
	<!-- Header Controls -->
	<div class="card">
		<!-- Link Type Selector -->
		<div class="flex flex-row justify-between gap-2">
			<!-- Link Input Fields -->
			<ToggleGroup
				multiple={false}
				value={[editorState.mode]}
				onValueChange={(details) => {
					if (details.value[0]) switchEditorMode(details.value[0] as EditorMode);
				}}
			>
				<ToggleGroup.Item value="reference">
					<LinkIcon class="inline-block size-4" />
				</ToggleGroup.Item>
				<ToggleGroup.Item value="inline">
					<ParenthesesIcon class="inline-block size-4" />
				</ToggleGroup.Item>
			</ToggleGroup>
			{#if editorState.mode === 'reference'}
				<div class="input-group flex flex-grow">
					{#if isEditingLink}
						<select bind:value={selectedLinkScheme} class="ig-select max-w-[7.5rem] preset-tonal">
							<option value="storage">storage</option>
							<option value="file">file</option>
							<option value="http">http</option>
							<option value="https">https</option>
						</select>
						<div class="ig-cell">://</div>
						<input
							class="ig-input"
							type="text"
							bind:this={linkLocationInputRef}
							bind:value={linkLocationInput}
							placeholder="resource location"
							onkeydown={handleLinkInputKeydown}
						/>
						{#if selectedLinkScheme === 'storage'}
							<button type="button" class="ig-btn">
								<SearchIcon class="inline-block size-4" />
							</button>
						{/if}
						<button type="button" class="ig-btn" onclick={quitEditLink}>
							<XIcon class="inline-block size-4" />
						</button>
						<button type="button" class="ig-btn">
							<ArrowRightIcon class="inline-block size-4" />
						</button>
					{:else}
						<div class="ig-cell">
							{#if editorState.parsedLink.kind === 'storage'}
								<DatabaseIcon class="inline-block size-4" />
							{:else if editorState.parsedLink.kind === 'file'}
								<FileText class="inline-block size-4" />
							{:else if editorState.parsedLink.kind === 'http'}
								<Globe class="inline-block size-4" />
							{/if}
						</div>
						<input class="ig-input" type="text" readonly {value} onfocus={editLink} />
						<button type="button" class="ig-btn" onclick={editLink}>
							<EditIcon class="inline-block size-4" />
						</button>
						<button type="button" class="ig-btn">
							<ArrowRightLeft class="inline-block size-4" />
						</button>
					{/if}
				</div>
			{:else if editorState.mode === 'inline'}
				<div class="flex flex-grow items-center space-x-2">
					<span class="text-sm text-surface-500">Editing inline value</span>
				</div>
			{/if}
		</div>
	</div>
	<!-- Close Link Configuration Card -->
	<!-- Content Area -->
	<div class="card border p-2">
		{#if editorState.mode === 'reference'}
			{#if editorState.remoteValue.status === 'loading'}
				<div class="p-4">
					<span class="loading loading-spinner loading-lg"></span>
					<p class="mt-2 text-sm text-surface-500">Loading linked value...</p>
				</div>
			{:else if editorState.remoteValue.status === 'error'}
				<div class="alert alert-danger m-4">
					<div class="flex items-center space-x-2">
						<span>Error loading linked value: {editorState.remoteValue.error.message}</span>
					</div>
				</div>
			{:else if editorState.remoteValue.status === 'loaded'}
				{@render editor(editorState.remoteValue.value, (newValue: T) => {
					editingReferenceValue = newValue;
				})}
			{/if}
		{:else if editorState.mode === 'inline'}
			{@render editor(value as T, (newValue: T) => {
				value = newValue;
			})}
		{/if}
	</div>
</div>
<!-- <FloatingPanel
	open={isSelectingStorage}
	onOpenChange={(e) => (isSelectingStorage = e.open)}
	minSize={{ width: 900, height: 600 }}
	defaultSize={{ width: 900, height: 600 }}
>
	<FloatingPanel.Trigger
		class="preset-filled-surface btn rounded-none btn-sm"
		title="Open Selector"
	>
		<Search size={16} />
	</FloatingPanel.Trigger>
	<Portal>
		<FloatingPanel.Positioner class="z-50">
			<FloatingPanel.Content >
				<FloatingPanel.DragTrigger>
					<FloatingPanel.DragTrigger>
						<FloatingPanel.Header>
							<FloatingPanel.Title>
								<GripVerticalIcon class="size-4" />
								Select {dataType} from storage
							</FloatingPanel.Title>
							<FloatingPanel.Control>
								<FloatingPanel.StageTrigger stage="minimized">
									<MinusIcon class="size-4" />
								</FloatingPanel.StageTrigger>
								<FloatingPanel.StageTrigger stage="maximized">
									<MaximizeIcon class="size-4" />
								</FloatingPanel.StageTrigger>
								<FloatingPanel.StageTrigger stage="default">
									<MinimizeIcon class="size-4" />
								</FloatingPanel.StageTrigger>
								<FloatingPanel.CloseTrigger>
									<XIcon class="size-4" />
								</FloatingPanel.CloseTrigger>
							</FloatingPanel.Control>
						</FloatingPanel.Header>
					</FloatingPanel.DragTrigger>
				</FloatingPanel.DragTrigger>
				<FloatingPanel.Body class="overflow-y-auto">
					<ObjectPages
						pageSize={5}
						filter={{
							data_type: dataType,
							latest_only: true,
							lockedFields: ['dataType']
						}}
						selectionMode="single"
						onSelect={selectStorageLink}
						showViewDetails={false}
						showEdit={false}
						showDelete={false}
					/>
				</FloatingPanel.Body>
				<FloatingPanel.ResizeTrigger axis="se" />
			</FloatingPanel.Content>
		</FloatingPanel.Positioner>
	</Portal>
</FloatingPanel> -->
