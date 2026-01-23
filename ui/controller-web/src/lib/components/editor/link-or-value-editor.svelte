<script lang="ts">
	import { api } from '$lib/api/routes';
	import {
		Link as LinkIcon,
		PenSquare,
		X,
		ArrowRightLeft,
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
	import LinkOrValueDisplay from '$lib/components/config/link-or-value-display.svelte';
	import type { StorageObjectDescriptor } from '$lib/api/types';
	import { shortRev } from '$lib/utils';
	import { FloatingPanel, Portal, SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import DataTypeRenderer from '$lib/data-types/components/data-type-renderer.svelte';
	import { dataTypeRegistry } from '$lib/data-types/registry';

	// Define types matching Rust Link variants
	type LinkKind = 'storage' | 'file' | 'http';

	type Props = {
		value: any;
		dataType: string;
		editorProps?: Record<string, any>;
	};

	let { value = $bindable(), dataType, editorProps = {} }: Props = $props();

	// 从注册表获取类型元信息
	const typeMetadata = $derived(dataTypeRegistry.get(dataType));
	const dataFormat = $derived(typeMetadata?.dataFormat || 'object');
	const defaultValue = $derived(() => typeMetadata?.defaultValue() ?? {});

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
	let editorMode = $state<'reference' | 'inline'>((() => {
        const linkState = parseLink(value);
        return linkState !== null ? 'reference' : 'inline';
    })());
	let linkMode = $state<LinkKind>('storage');

	// Input Cache
	let linkCache = $state<string>('');
	let inlineCache = $state<any>(null);

	// UI State
	let isSelectingStorage = $state(false);

	// Reference Tab state
	let referenceTab = $state<'edit' | 'convert'>('edit');
	
	// Edit content state
	let loadedValue = $state<any>(null);        // Value loaded from link
	let editedValue = $state<any>(null);        // User-edited value
	let isLoadingContent = $state(false);       // Loading state
	let loadContentError = $state<string | null>(null);  // Load error
	
	// Save state
	let isSaving = $state(false);               // Saving in progress
	let saveError = $state<string | null>(null); // Save error

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
	
	// Auto-load link content when in reference/edit mode (NEW)
	$effect(() => {
		if (editorMode === 'reference' && referenceTab === 'edit' && isLink && value) {
			loadLinkContent();
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

	function switchToDefaultValue() {
		if (defaultValue) {
			value = defaultValue();
		} else {
			value = {}; // Fallback
		}
	}
	
	// Load link content (NEW - replaces previewLink for edit mode)
	async function loadLinkContent() {
		if (!value || !isLink) return;
		
		isLoadingContent = true;
		loadContentError = null;
		loadedValue = null;
		editedValue = null;
		
		try {
			console.log('[LoadContent] Loading link:', value, 'dataFormat:', dataFormat);
			let resolved;
			if (dataFormat === 'object') {
				resolved = await api.resolve.link_to_object(value);
			} else if (dataFormat === 'string') {
				resolved = await api.resolve.link_to_string(value);
			}
			console.log('[LoadContent] Loaded successfully:', resolved);
			
			loadedValue = resolved;
			editedValue = resolved;  // Initialize edited value
		} catch (e) {
			loadContentError = e instanceof Error ? e.message : String(e);
			console.error('[LoadContent] Failed to load link content', e);
		} finally {
			isLoadingContent = false;
		}
	}
	
	// Save changes to link (NEW)
	async function saveChanges() {
		if (!isLink || !editedValue) return;
		
		isSaving = true;
		saveError = null;
		
		try {
			console.log('[Save] Saving to link:', value, 'dataType:', dataType);
			const response = await api.resolve.save_to_link({
				link: value,
				value: editedValue,
				data_type: dataType,
			});
			
			console.log('[Save] Saved successfully, new link:', response.link);
			// Update link (for storage, this will be a new revision)
			value = response.link;
			
			// Reload content to show saved data
			await loadLinkContent();
		} catch (e) {
			saveError = e instanceof Error ? e.message : String(e);
			console.error('[Save] Failed to save changes', e);
		} finally {
			isSaving = false;
		}
	}
	
	// Simplified convert to inline (NEW - uses loaded value from tab)
	function convertToInline() {
		if (loadedValue !== null) {
			linkCache = value;           // Save current link
			inlineCache = loadedValue;   // Update cache
			value = loadedValue;         // Set as current value
			editorMode = 'inline';       // Switch mode
		}
	}

</script>

<div
	class="@container space-y-4 card border border-surface-300 p-4 dark:border-surface-600"
>
<!-- Header Controls -->
	<div class="flex flex-wrap items-center gap-4 mb-4">
		<!-- Main Mode Selector -->
		<SegmentedControl 
			value={editorMode} 
			orientation="horizontal"
			onValueChange={(details) => {
				if (details.value) setEditorMode(details.value as 'reference' | 'inline');
			}}
		>
			<SegmentedControl.Control>
				<SegmentedControl.Indicator />
				<SegmentedControl.Item value="reference" title="Reference" aria-label="Reference">
					<SegmentedControl.ItemText>
						<LinkIcon class="inline-block size-4" />
						<span class="hidden @md:inline">Reference</span>
					</SegmentedControl.ItemText>
					<SegmentedControl.ItemHiddenInput />
				</SegmentedControl.Item>
				<SegmentedControl.Item value="inline" title="Inline Value" aria-label="Inline">
					<SegmentedControl.ItemText>
						<PenSquare class="inline-block size-4" />
						<span class="hidden @md:inline">Inline</span>
					</SegmentedControl.ItemText>
					<SegmentedControl.ItemHiddenInput />
				</SegmentedControl.Item>
			</SegmentedControl.Control>
		</SegmentedControl>

		<!-- Reference Action Selector (shown in same row when in reference mode) -->
		{#if editorMode === 'reference'}
			<div class="flex items-center gap-2 border-l pl-4 border-surface-300 dark:border-surface-600">
				<SegmentedControl 
					value={referenceTab} 
					orientation="horizontal"
					onValueChange={(details) => {
						if (details.value) referenceTab = details.value as 'edit' | 'convert';
					}}
				>
					<SegmentedControl.Control>
						<SegmentedControl.Indicator />
						<SegmentedControl.Item value="edit" title="Edit" aria-label="Edit">
							<SegmentedControl.ItemText>
								<PenSquare class="inline-block size-4" />
								<span class="hidden @md:inline">Edit</span>
							</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
						<SegmentedControl.Item value="convert" title="Convert to Inline" aria-label="Convert">
							<SegmentedControl.ItemText>
								<ArrowRightLeft class="inline-block size-4" />
								<span class="hidden @md:inline">Convert to Inline</span>
							</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
					</SegmentedControl.Control>
				</SegmentedControl>
			</div>
		{/if}
	</div>

	<!-- Content Area -->
	{#if editorMode === 'reference'}
		<!-- Link Configuration Card -->
		<div class="card border border-surface-300 dark:border-surface-600 p-4 space-y-3 mb-4">
			<!-- Link Type Selector -->
			<div class="flex items-center gap-2">
				<SegmentedControl 
					value={linkMode} 
					orientation="horizontal"
					onValueChange={(details) => {
						if (details.value) setLinkMode(details.value);
					}}
				>
					<SegmentedControl.Control>
						<SegmentedControl.Indicator />
						<SegmentedControl.Item value="storage" title="Storage" aria-label="Storage">
							<SegmentedControl.ItemText>
								<Database class="inline-block size-4" />
								<span class="hidden @md:inline">Storage</span>
							</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
						<SegmentedControl.Item value="file" title="File" aria-label="File">
							<SegmentedControl.ItemText>
								<FileText class="inline-block size-4" />
								<span class="hidden @md:inline">File</span>
							</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
						<SegmentedControl.Item value="http" title="HTTP" aria-label="HTTP">
							<SegmentedControl.ItemText>
								<Globe class="inline-block size-4" />
								<span class="hidden @md:inline">HTTP</span>
							</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
					</SegmentedControl.Control>
				</SegmentedControl>
			</div>
			
			<!-- Link Input Fields -->
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
		</div>  <!-- Close Link Configuration Card -->
		
		<!-- Content Area Card -->
		<div class="card border border-surface-300 dark:border-surface-600 p-4 min-h-[400px]">
		{#if referenceTab === 'edit'}
			<!-- Edit Tab Content -->
			<div class="space-y-4">
				{#if isLoadingContent}
					<div class="flex items-center justify-center h-[360px] text-surface-500">
						<div class="flex items-center gap-2">
							<div class="animate-spin">⏳</div>
							<span>Loading content...</span>
						</div>
					</div>
				{:else if loadContentError}
					<div class="p-4 bg-error-100 dark:bg-error-900 text-error-700 dark:text-error-300 rounded h-[360px] flex items-center justify-center">
						<div class="flex items-start gap-2">
							<AlertCircle size={16} class="mt-0.5 flex-shrink-0" />
							<div>
								<div class="font-semibold">Failed to load content</div>
								<div class="text-xs mt-1">{loadContentError}</div>
							</div>
						</div>
					</div>
				{:else if editedValue !== null}
					<!-- Editable content area -->
					<div class="space-y-4">
						{#if linkState?.kind === 'http'}
							<!-- HTTP links are read-only for now -->
							<div class="alert preset-filled-warning mb-4">
								<AlertCircle size={16} />
								<span>HTTP PUT is not yet supported. This content is read-only.</span>
							</div>
							<DataTypeRenderer
								type={dataType}
								mode="view"
								value={editedValue}
								{...editorProps}
							/>
						{:else}
							<DataTypeRenderer
								type={dataType}
								mode="edit"
								bind:value={editedValue}
								{...editorProps}
							/>
							
							<!-- Save button -->
							<div class="flex justify-end gap-2 items-center">
								{#if saveError}
									<span class="text-error-700 dark:text-error-300 text-sm">{saveError}</span>
								{/if}
								<button 
									class="btn btn-sm preset-filled-primary transition-all duration-200 cursor-pointer"
									onclick={saveChanges}
									disabled={isSaving}
								>
									{#if isSaving}
										<div class="animate-spin">⏳</div>
										<span>Saving...</span>
									{:else}
										<CheckCircle size={16} />
										<span>Save Changes</span>
									{/if}
								</button>
							</div>
						{/if}
					</div>
				{:else}
					<div class="flex items-center justify-center p-8 text-surface-500">
						No content to display
					</div>
				{/if}
			</div>
		{:else if referenceTab === 'convert'}
			<!-- Convert to Inline Tab Content -->
			<div class="space-y-4">
				{#if isLoadingContent}
					<div class="flex items-center justify-center h-[360px] text-surface-500">
						<div class="flex items-center gap-2">
							<div class="animate-spin">⏳</div>
							<span>Loading content...</span>
						</div>
					</div>
				{:else if loadContentError}
					<div class="p-4 bg-error-100 dark:bg-error-900 text-error-700 dark:text-error-300 rounded h-[360px] flex items-center justify-center">
						<div class="flex items-start gap-2">
							<AlertCircle size={16} class="mt-0.5 flex-shrink-0" />
							<div>
								<div class="font-semibold">Failed to load content</div>
								<div class="text-xs mt-1">{loadContentError}</div>
							</div>
						</div>
					</div>
				{:else if loadedValue !== null}
					<div class="space-y-4">
						<div class="text-sm text-surface-600 dark:text-surface-400">
							This will replace the reference <code class="bg-surface-200 dark:bg-surface-700 px-1 py-0.5 rounded text-xs">{value}</code> with its actual content.
						</div>
						
						<!-- Preview content -->
						<div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-300 dark:border-surface-600 p-4">
							<DataTypeRenderer
								type={dataType}
								mode="view"
								value={loadedValue}
								{...editorProps}
							/>
						</div>
						
						<!-- Confirm button -->
						<div class="flex justify-end">
							<button 
								class="btn btn-sm preset-filled-primary transition-all duration-200 cursor-pointer"
								onclick={convertToInline}
							>
								<CheckCircle size={16} />
								<span>Confirm Conversion</span>
							</button>
						</div>
					</div>
				{:else}
					<div class="flex items-center justify-center p-8 text-surface-500">
						No content to convert
					</div>
				{/if}
			</div>
		{/if}
		</div>  <!-- Close Content Area Card -->
	{:else}
		<!-- Inline Value Editor - 使用数据类型渲染器 -->
		<div class="card border border-surface-300 dark:border-surface-600 p-4">
		<DataTypeRenderer
			type={dataType}
			mode="edit"
			bind:value
			{...editorProps}
		/>
		</div>
	{/if}
</div>
