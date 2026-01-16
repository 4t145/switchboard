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
	import { FloatingPanel, Portal, SegmentedControl } from '@skeletonlabs/skeleton-svelte';

	// Define types matching Rust Link variants
	type LinkKind = 'storage' | 'file' | 'http';

	type Props = {
		value: any;
		dataFormat: 'string' | 'object'
		dataType: string;
		renderValue: any; // RenderSnippet
		defaultValue: () => any;
	};

	let { value = $bindable(), dataType, renderValue, dataFormat, defaultValue }: Props = $props();

	// Helper function to check if a string is a valid URI
	function isValidURI(str: string): boolean {
		// Check for common URI schemes
		const uriSchemes = [
			'file://',
			'http://',
			'https://',
			'ftp://',
			'ftps://',
			'storage://',
		];
		
		// Check if string starts with any known URI scheme
		for (const scheme of uriSchemes) {
			if (str.startsWith(scheme)) {
				return true;
			}
		}
		
		return false;
	}
	// Parsing Helpers
	function parseLink(val: any): { kind: LinkKind; data: any } | null {
		if (typeof val === 'string') {
			// Only treat as link if it's actually a URI
			if (!isValidURI(val)) {
				return null;
			}
			
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
			
			// Handle other URI schemes as HTTP for now (could be extended later)
			if (val.startsWith('ftp://') || val.startsWith('ftps://')) {
				return { kind: 'http', data: val };
			}
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
	
	// Editor mode control
	let editorMode = $state<'reference' | 'inline'>('inline');
	let linkMode = $state<LinkKind>('storage');

	// Input Cache
	let linkCache = $state<string>('');
	let inlineCache = $state<any>(null);

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

	// Sync editor mode based on current value
	$effect(() => {
		editorMode = isLink ? 'reference' : 'inline';
		if (linkState) {
			linkMode = linkState.kind;
		}
	});

	// Actions
	function setEditorMode(mode: 'reference' | 'inline') {
		if (mode === 'reference' && !isLink) {
			// Switching to reference - try to use cache first, then default to storage
			if (linkCache) {
				inlineCache = value;  // Save current inline value
				value = linkCache;
			} else {
				inlineCache = value;  // Save current inline value
				value = formatLink('storage', { id: '', revision: '' });
				linkMode = 'storage';
			}
		} else if (mode === 'inline' && isLink) {
			// Switching to inline - only use cache, never fetch from server
			linkCache = value;  // Save current link
			if (inlineCache !== null) {
				value = inlineCache;
			} else {
				value = defaultValue();
			}
		}
		editorMode = mode;
	}

	function setLinkMode(kind: string) {
		const linkKind = kind as LinkKind;
		linkMode = linkKind;
		// When switching, use the stored temp value if available, otherwise default
		if (linkKind === 'storage') {
			// Try to restore storage link if we have temp values
			if (tempStorageId) {
				value = formatLink('storage', { id: tempStorageId, revision: tempStorageRev || 'latest' });
			} else if (linkState?.kind !== 'storage') {
				// No temp value, maybe trigger selector or just let them type
				// isSelectingStorage = true; // Removed auto-trigger for consistent input UI
				value = formatLink('storage', { id: '', revision: '' });
			}
		} else if (linkKind === 'file') {
			value = `file://${tempFileValue}`;
		} else if (linkKind === 'http') {
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
		if (isLink) {
			loading = true;
			linkCache = value;
			value = inlineCache;
			// try {
			// 	let resolvedValue;
			// 	if (dataFormat === 'object') {
			// 		resolvedValue = await api.resolve.link_to_object(value);
			// 	} else if (dataFormat === 'string'){
			// 		resolvedValue = await api.resolve.link_to_string(value);
			// 	}
			// 	linkCache = value;
			// 	value = resolvedValue;
			// } catch (e) {
			// 	console.error('Failed to resolve link to inline value', e);
			// 	alert('Failed to resolve link: ' + e);
			// } finally {
			// 	loading = false;
			// }
		}
	}

	async function convertToInlineValue() {
		if (!linkState) return;
		loading = true;
		try {
			let resolvedValue;
			if (dataFormat === 'object') {
				resolvedValue = await api.resolve.link_to_object(value);
			} else if (dataFormat === 'string'){
				resolvedValue = await api.resolve.link_to_string(value);
			}
			linkCache = value;  // Save current link
			inlineCache = resolvedValue;  // Update cache with server value
			value = resolvedValue;  // Set as current value
			editorMode = 'inline';
		} catch (e) {
			console.error('Failed to convert reference to inline value', e);
			alert('Failed to convert reference: ' + e);
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
		// Default to storage:// format to trigger link mode
		if (!isLink) {
			inlineCache = value;
			value = 'storage://#';
			editorMode = 'reference';
			linkMode = 'storage';
		}
		// Open the storage selector
		isSelectingStorage = true;
	}
</script>

<div
	class="space-y-4 card border border-surface-300 p-4 dark:border-surface-600"
>
<!-- Header Controls -->
	<div class="flex flex-row items-center gap-4 space-x-2">
		<!-- Main Mode Selector -->
		<SegmentedControl 
			value={editorMode} 
			orientation="horizontal"
			onValueChange={(details) => {
				if (details.value) setEditorMode(details.value as 'reference' | 'inline');
			}}
		>
			<SegmentedControl.Label class="text-sm font-medium mb-2 block">Editor Mode</SegmentedControl.Label>
			<SegmentedControl.Control>
				<SegmentedControl.Indicator />
				<SegmentedControl.Item value="reference">
					<SegmentedControl.ItemText>
						<LinkIcon size={16} class="inline-block" />
						Reference
					</SegmentedControl.ItemText>
					<SegmentedControl.ItemHiddenInput />
				</SegmentedControl.Item>
				<SegmentedControl.Item value="inline">
					<SegmentedControl.ItemText>
						<PenSquare size={16} class="inline-block" />
						Inline
					</SegmentedControl.ItemText>
					<SegmentedControl.ItemHiddenInput />
				</SegmentedControl.Item>
			</SegmentedControl.Control>
		</SegmentedControl>

		<!-- Link Type Selector (only shown in reference mode) -->
		{#if editorMode === 'reference'}
			<SegmentedControl 
				value={linkMode} 
				orientation="horizontal"
				onValueChange={(details) => {
					if (details.value) setLinkMode(details.value);
				}}
			>
				<SegmentedControl.Label class="text-sm font-medium mb-2 block">Link Type</SegmentedControl.Label>
				<SegmentedControl.Control>
					<SegmentedControl.Indicator />
					<SegmentedControl.Item value="storage">
						<SegmentedControl.ItemText>
							<Database size={14} class="mr-1 inline-block" />
							Storage
						</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item value="file">
						<SegmentedControl.ItemText>
							<FileText size={14} class="mr-1 inline-block" />
							File
						</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item value="http">
						<SegmentedControl.ItemText>
							<Globe size={14} class="mr-1 inline-block" />
							HTTP
						</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
				</SegmentedControl.Control>
			</SegmentedControl>
		{/if}

		<!-- Action Buttons -->
		<div class="flex justify-between items-center">

		</div>
	</div>

	<!-- Content Area -->
	{#if isPromoting}
		<!-- Promote Dialog -->
		<div class="alert preset-filled-surface animate-fade-in flex flex-col gap-4">
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
			<button class="preset-ghost btn btn-sm" onclick={() => (isPromoting = false)}
				>Cancel</button
			>
			<button
				class="preset-filled-primary btn btn-sm"
					onclick={confirmPromote}
					disabled={!promoteId || loading}
				>
					Save & Link
				</button>
			</div>
		</div>
	{:else if editorMode === 'reference'}
		<!-- Reference Mode Content -->
		{#if linkMode === 'storage'}
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
				</FloatingPanel>
			</div>
		{:else if linkMode === 'file'}
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
		{:else if linkMode === 'http'}
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
		{/if}
	{:else}
		<!-- Inline Value Editor -->
		{@render renderValue()}
	{/if}
</div>
