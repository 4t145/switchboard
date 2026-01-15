<script lang="ts">
	import { api } from '$lib/api/routes';
	import {
		Link as LinkIcon,
		PenSquare,
		X,
		ArrowRightLeft,
		Upload,
		Database,
		FileText,
		Globe,
		Search,
		CheckCircle,
		AlertCircle,

		GripVerticalIcon,


		MinusIcon,

		MinimizeIcon,

		XIcon,

		MaximizeIcon






	} from 'lucide-svelte';
	import ObjectPages from '$lib/components/object-pages.svelte';
	import type { StorageObjectDescriptor } from '$lib/api/types';
	import { shortRev } from '$lib/utils';
	import { Tabs, FloatingPanel, Portal } from '@skeletonlabs/skeleton-svelte';

	// Define types matching Rust Link variants
	type LinkKind = 'storage' | 'file' | 'http';

	type Props = {
		value: any;
		dataType: string;
		renderValue: any; // RenderSnippet
		defaultValue: () => any;
	};

	let { value = $bindable(), dataType, renderValue, defaultValue }: Props = $props();

	// Parsing Helpers
	function parseLink(val: any): { kind: LinkKind; data: any } | null {
		if (typeof val === 'string') {
			if (val.startsWith('file://')) return { kind: 'file', data: val.slice(7) };
			if (val.startsWith('http://') || val.startsWith('https://'))
				return { kind: 'http', data: val };
			if (val.startsWith('storage://')) {
				const parts = val.slice(10).split('#');
				if (parts.length >= 2)
					return {
						kind: 'storage',
						data: { id: parts[0], revision: parts[1] } as StorageObjectDescriptor
					};
			}
		}
		// Legacy support for { $link: ... }
		if (val && typeof val === 'object' && '$link' in val) {
			return { kind: 'storage', data: val.$link };
		}
		return null;
	}

	function formatLink(kind: LinkKind, data: any): string {
		if (kind === 'file') {
			const path = data.toString();
			return path.startsWith('file://') ? path : `file://${path}`;
		}
		if (kind === 'http') return data.toString();
		if (kind === 'storage') return `storage://${data.id}#${data.revision}`;
		return '';
	}

	// Derived State
	let linkState = $derived(parseLink(value));
	let isLink = $derived(linkState !== null);

	// UI State
	let isSelectingStorage = $state(false);
	let isPromoting = $state(false);
	let promoteId = $state('');
	let loading = $state(false);

	// For File/Http inputs when in link mode - Local state to preserve input across switches
	let tempFileValue = $state('');
	let tempHttpValue = $state('');
	// For Storage mode
	let tempStorageId = $state('');
	let tempStorageRev = $state('');

	// Sync temp state from value when mounting or value changes externally
	$effect(() => {
		if (linkState) {
			if (linkState.kind === 'file') tempFileValue = linkState.data;
			if (linkState.kind === 'http') tempHttpValue = linkState.data;
			if (linkState.kind === 'storage') {
				tempStorageId = linkState.data.id;
				tempStorageRev = linkState.data.revision;
			}
		}
	});

	// Actions
	function setLinkMode(kind: string) {
		// When switching, use the stored temp value if available, otherwise default
		if (kind === 'storage') {
			// Try to restore storage link if we have temp values
			if (tempStorageId) {
				value = formatLink('storage', { id: tempStorageId, revision: tempStorageRev || 'latest' });
			} else if (linkState?.kind !== 'storage') {
				// No temp value, maybe trigger selector or just let them type
				// isSelectingStorage = true; // Removed auto-trigger for consistent input UI
				value = formatLink('storage', { id: '', revision: '' });
			}
		} else if (kind === 'file') {
			value = `file://${tempFileValue}`;
		} else if (kind === 'http') {
			// Default to http:// if empty, but respect existing temp value
			const prefix = tempHttpValue.startsWith('http') ? '' : 'http://';
			value = tempHttpValue ? tempHttpValue : 'http://';
		}
	}

	function updateLinkString(kind: 'file' | 'http' | 'storage', input: string, extra?: string) {
		if (kind === 'file') {
			tempFileValue = input;
			// Ensure file:// prefix is not duplicated if user types it
			if (input.startsWith('file://')) value = input;
			else value = `file://${input}`;
		} else if (kind === 'http') {
			// Enforce http/https
			let url = input;
			tempHttpValue = url;
			value = url;
		} else if (kind === 'storage') {
			// input is ID, extra is Revision
			tempStorageId = input;
			if (extra !== undefined) tempStorageRev = extra;
			value = formatLink('storage', { id: tempStorageId, revision: tempStorageRev });
		}
	}

	function selectStorageLink(item: any) {
		value = formatLink('storage', item.descriptor);
		isSelectingStorage = false;
	}

	async function switchToInline() {
		if (!linkState) return;
		loading = true;
		try {
			if (linkState.kind === 'storage') {
				const descriptor = linkState.data;
				const response = await api.storage.get(descriptor);
				value = response;
			} else {
				alert('Cannot convert File or HTTP links to inline value automatically yet.');
			}
		} catch (e) {
			console.error('Failed to fetch linked object', e);
			alert('Failed to load object content: ' + e);
		} finally {
			loading = false;
		}
	}

	// Promote (Save as Link)
	function startPromote() {
		isPromoting = true;
		promoteId = '';
	}

	async function confirmPromote() {
		if (!promoteId) return;
		loading = true;
		try {
			const req = {
				resolver: 'static',
				config: value,
				save_as: promoteId ? promoteId : undefined
			};

			const descriptor = await api.storage.save(req);
			value = formatLink('storage', descriptor);
			isPromoting = false;
		} catch (e) {
			console.error('Failed to save object', e);
			alert('Failed to save object: ' + e);
		} finally {
			loading = false;
		}
	}

	function switchToDefaultValue() {
		if (defaultValue) {
			value = defaultValue();
		} else {
			value = {}; // Fallback
		}
	}

	function switchToReferenceMode() {
		// Defaults to Storage selector if no link set
		isSelectingStorage = true;
		// We don't set value yet, user cancels -> stays inline?
		// Or we should set a dummy value?
		// Let's just open the selector. If they select, it becomes a link.
		// If they cancel, they stay in inline mode.
	}
</script>

<div
	class="variant-soft-surface space-y-4 card border border-surface-300 p-4 dark:border-surface-600"
>
	<!-- Header Controls -->
	<div
		class="mb-2 flex items-center justify-between border-b border-surface-300 pb-2 dark:border-surface-600"
	>
		<div class="flex items-center gap-2">
			{#if isLink || isSelectingStorage}
				<LinkIcon size={16} class="text-tertiary-500" />
				<span class="font-bold text-tertiary-600 dark:text-tertiary-400">Reference (Link)</span>
			{:else}
				<PenSquare size={16} class="text-secondary-500" />
				<span class="font-bold text-secondary-600 dark:text-secondary-400">Inline Value</span>
			{/if}
		</div>

		<div class="flex gap-2">
			{#if isLink}
				{#if linkState?.kind === 'storage'}
					<button
						class="variant-filled-secondary btn btn-sm"
						onclick={switchToInline}
						disabled={loading}
					>
						<ArrowRightLeft size={14} class="mr-1" /> Convert to Inline
					</button>
				{/if}
				<button class="variant-ghost-surface btn btn-sm" onclick={switchToDefaultValue}>
					Switch to Inline (Reset)
				</button>
			{:else if !isSelectingStorage && !isLink}
				<button
					class="variant-filled-tertiary btn btn-sm"
					onclick={startPromote}
					disabled={loading}
				>
					<Upload size={14} class="mr-1" /> Save as Link
				</button>
				<button class="variant-ghost-surface btn btn-sm" onclick={switchToReferenceMode}>
					Switch to Reference
				</button>
			{/if}
		</div>
	</div>

	<!-- Content Area -->
	{#if isPromoting}
		<!-- Promote Dialog -->
		<div class="alert variant-filled-surface animate-fade-in flex flex-col gap-4">
			<div class="flex items-center justify-between">
				<span class="font-bold">Save current value as shared object</span>
				<button class="btn-icon btn-icon-sm" onclick={() => (isPromoting = false)}
					><X size={16} /></button
				>
			</div>
			<label class="label">
				<span>New Object ID</span>
				<input
					class="input"
					type="text"
					bind:value={promoteId}
					placeholder="e.g. my-shared-config"
				/>
			</label>
			<div class="flex justify-end gap-2">
				<button class="variant-ghost btn btn-sm" onclick={() => (isPromoting = false)}
					>Cancel</button
				>
				<button
					class="variant-filled-primary btn btn-sm"
					onclick={confirmPromote}
					disabled={!promoteId || loading}
				>
					Save & Link
				</button>
			</div>
		</div>
		<!-- Storage Selector -->
		<!-- Removed inline storage selector since we use popover now -->
	{:else if isLink && linkState}
		<!-- Link Type Selector & Editor -->
		<Tabs value={linkState.kind} onValueChange={(e) => setLinkMode(e.value)}>
			<Tabs.List class="w-full">
				<Tabs.Trigger value="storage" class="flex-1">
					<Database size={14} class="mr-1 inline-block" /> Storage
				</Tabs.Trigger>
				<Tabs.Trigger value="file" class="flex-1">
					<FileText size={14} class="mr-1 inline-block" /> File
				</Tabs.Trigger>
				<Tabs.Trigger value="http" class="flex-1">
					<Globe size={14} class="mr-1 inline-block" /> HTTP
				</Tabs.Trigger>
				<Tabs.Indicator />
			</Tabs.List>
			<Tabs.Content value="storage">
				<div class="input-group-divider input-group grid-cols-[auto_1fr_auto_auto_auto]">
					<div class="ig-cell preset-tonal">storage://</div>
					<input
						class="ig-input"
						type="text"
						value={tempStorageId}
						oninput={(e) => updateLinkString('storage', e.currentTarget.value)}
						placeholder="object-id"
					/>
					<div class="ig-cell border-l border-surface-400/20">#</div>
					<input
						class="ig-input w-24 border-l-0"
						type="text"
						value={tempStorageRev}
						oninput={(e) => updateLinkString('storage', tempStorageId, e.currentTarget.value)}
						placeholder="rev"
					/>
					<FloatingPanel
						open={isSelectingStorage}
						onOpenChange={(e) => (isSelectingStorage = e.open)}
                        minSize={{ width: 900, height: 600 }}
                        defaultSize={{ width: 900, height: 600 }}
					>
						<FloatingPanel.Trigger
							class="variant-filled-surface btn rounded-none btn-sm"
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
					</FloatingPanel>
				</div>
				<!-- 
                {#if tempStorageId}
                     <div class="mt-2 flex items-center gap-2 text-xs text-success-500">
                        <CheckCircle size={12}/> Valid object reference
                     </div>
                {/if}
                -->
			</Tabs.Content>
			<Tabs.Content value="file">
				<div class="input-group-divider input-group grid-cols-[auto_1fr]">
					<div class="ig-cell preset-tonal">file://</div>
					<input
						class="ig-input"
						type="text"
						value={tempFileValue}
						oninput={(e) => updateLinkString('file', e.currentTarget.value)}
						placeholder="/path/to/config.json"
					/>
				</div>
			</Tabs.Content>
			<Tabs.Content value="http">
				<div class="input-group-divider input-group grid-cols-[auto_1fr]">
					<select
						class="ig-select preset-tonal"
						value={tempHttpValue.startsWith('https') ? 'https://' : 'http://'}
						onchange={(e) => {
							const scheme = e.currentTarget.value;
							const current = tempHttpValue.replace(/^https?:\/\//, '');
							updateLinkString('http', scheme + current);
						}}
					>
						<option value="http://">http://</option>
						<option value="https://">https://</option>
					</select>
					<input
						class="ig-input"
						type="text"
						value={tempHttpValue.replace(/^https?:\/\//, '')}
						oninput={(e) => {
							const scheme = tempHttpValue.startsWith('https') ? 'https://' : 'http://';
							updateLinkString('http', scheme + e.currentTarget.value);
						}}
						placeholder="example.com/config.json"
					/>
				</div>
			</Tabs.Content>
		</Tabs>
	{:else}
		<!-- Inline Value Editor -->
		{@render renderValue()}
	{/if}
</div>
