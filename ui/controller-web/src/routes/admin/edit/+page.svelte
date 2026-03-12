<script lang="ts" module>
	type ConfigSource =
		| {
				kind: 'new';
		  }
		| {
				kind: 'resolved';
				source:
					| {
							from: 'k8s';
							namespace: string;
					  }
					| {
							from: 'fs';
							path: string;
					  };
		  }
		| {
				kind: 'link';
				link: string;
		  };
</script>

<script lang="ts">
	import { api } from '$lib/api/routes';
	import type { HumanReadableServiceConfig } from '$lib/api/types';
	import ServiceConfigEditor from '$lib/components/editor/service-config-editor.svelte';
	import { dialogQuery } from '$lib/components/dialog';
	import { parseLink } from '$lib/utils/link-parser';
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import FileTree from '$lib/components/file-tree.svelte';
	import { capabilitiesStore } from '$lib/stores/capabilities.svelte';
	import ObjectSelector from '$lib/components/object-selector.svelte';

	type SourceKind = 'new' | 'link' | 'resolved';
	type LinkKind = 'storage' | 'file';
	type ResolveKind = 'fs' | 'k8s';
	type SaveTargetKind = LinkKind;
	type SaveDialogOption = 'confirm' | 'cancel';

	const EMPTY_CONFIG: HumanReadableServiceConfig = {
		tcp_services: [],
		tls: []
	};

	const SERVICE_CONFIG_DATA_TYPE = 'ServiceConfig';

	let source: ConfigSource | null = $state(null);
	let config: HumanReadableServiceConfig | null = $state(null);

	let selectedSourceKind: SourceKind = $state('new');
	let selectedLinkKind: LinkKind = $state('file');
	let sourceLinkInput = $state('');
	let selectedLinkFilePath = $state('');
	let resolvedKind: ResolveKind = $state('fs');
	let resolvedFsPath = $state('');
	let resolvedNamespace = $state('default');

	let isLoadingSource = $state(false);
	let isSaving = $state(false);
	let isDeploying = $state(false);

	let errorMessage = $state<string | null>(null);
	let successMessage = $state<string | null>(null);

	let saveTargetKind: SaveTargetKind = $state('file');
	let saveTargetInput = $state('');
	let k8sCapability = $derived(capabilitiesStore.k8s);

	function resetMessages() {
		errorMessage = null;
		successMessage = null;
	}

	function toErrorMessage(error: unknown, fallback: string): string {
		if (error instanceof Error && error.message.trim().length > 0) {
			return error.message;
		}
		return fallback;
	}

	function cloneEmptyConfig(): HumanReadableServiceConfig {
		return {
			tcp_services: [...EMPTY_CONFIG.tcp_services],
			tls: [...EMPTY_CONFIG.tls]
		};
	}

	function buildSaveTargetLink(kind: SaveTargetKind, inputValue: string): string | null {
		const raw = inputValue.trim();
		if (!raw) {
			return null;
		}

		if (kind === 'file') {
			const filePath = raw.startsWith('file://') ? raw.slice(7).trim() : raw;
			if (!filePath) {
				return null;
			}
			return `file://${filePath}`;
		}

		const normalized = raw.startsWith('storage://') ? raw.slice(10) : raw;
		const id = normalized.split('#')[0]?.trim();
		if (!id) {
			return null;
		}
		return `storage://${id}#`;
	}

	async function requestSaveTargetLink(params: {
		title: string;
		confirmLabel: string;
		defaultKind?: SaveTargetKind;
		defaultInput?: string;
	}): Promise<string | null> {
		saveTargetKind = params.defaultKind ?? 'file';
		saveTargetInput = params.defaultInput ?? '';

		while (true) {
			const selected = await dialogQuery<SaveDialogOption>({
				title: params.title,
				message: saveTargetDialogMessage,
				options: {
					cancel: {
						label: 'Cancel',
						class: 'btn preset-tonal-secondary'
					},
					confirm: {
						label: params.confirmLabel,
						class: 'btn preset-tonal-primary'
					}
				},
				role: 'dialog'
			});

			if (selected !== 'confirm') {
				return null;
			}

			const link = buildSaveTargetLink(saveTargetKind, saveTargetInput);
			if (link) {
				return link;
			}

			errorMessage =
				saveTargetKind === 'file'
					? 'Please enter a valid file path for save target.'
					: 'Please enter a valid storage id for save target.';
		}
	}

	function describeSource(currentSource: ConfigSource): string {
		if (currentSource.kind === 'new') {
			return 'New';
		}
		if (currentSource.kind === 'link') {
			return `Link: ${currentSource.link}`;
		}
		if (currentSource.source.from === 'fs') {
			return `Resolved from fs: ${currentSource.source.path}`;
		}
		return `Resolved from k8s: ${currentSource.source.namespace}`;
	}

	async function handleLoadSource() {
		resetMessages();
		isLoadingSource = true;
		try {
			if (selectedSourceKind === 'new') {
				source = { kind: 'new' };
				config = cloneEmptyConfig();
				return;
			}

			if (selectedSourceKind === 'link') {
				const link = sourceLinkInput.trim();
				const parsedLink = parseLink(link);
				if (!parsedLink || (parsedLink.kind !== 'file' && parsedLink.kind !== 'storage')) {
					throw new Error(
						`Only file:// and storage:// links are supported for import, got ${link}`
					);
				}

				const loaded = await api.resolve.link_to_object(link);
				source = { kind: 'link', link };
				config = loaded as HumanReadableServiceConfig;
				return;
			}

			if (resolvedKind === 'fs') {
				const path = resolvedFsPath.trim();
				if (!path) {
					throw new Error('Filesystem path is required.');
				}

				const resolved = await api.resolve.service_config({
					resolver: 'fs',
					config: { path }
				});
				source = {
					kind: 'resolved',
					source: {
						from: 'fs',
						path
					}
				};
				config = resolved.config;
				return;
			}

			const namespace = resolvedNamespace.trim();
			if (!namespace) {
				throw new Error('Kubernetes namespace is required.');
			}
			if (!k8sCapability.available) {
				throw new Error('Kubernetes resolver is unavailable in current environment.');
			}

			const resolved = await api.resolve.service_config({
				resolver: 'k8s',
				config: {
					gateway_namespace: namespace
				}
			});
			source = {
				kind: 'resolved',
				source: {
					from: 'k8s',
					namespace
				}
			};
			config = resolved.config;
		} catch (error) {
			errorMessage = toErrorMessage(error, 'Failed to load configuration source.');
		} finally {
			isLoadingSource = false;
		}
	}

	async function saveToLink(link: string): Promise<void> {
		if (!config) {
			throw new Error('No configuration to save.');
		}
		await api.resolve.save_to_link({
			link,
			value: config,
			data_type: SERVICE_CONFIG_DATA_TYPE
		});
	}

	async function runSave(action: 'save' | 'save-as', silent = false): Promise<string | null> {
		if (!source || !config) {
			errorMessage = 'No configuration loaded.';
			return null;
		}

		resetMessages();
		isSaving = true;

		try {
			if (action === 'save' && source.kind === 'link') {
				await saveToLink(source.link);
				if (!silent) {
					successMessage = `Saved to ${source.link}`;
				}
				return source.link;
			}

			const defaultKind: SaveTargetKind =
				action === 'save-as' && source.kind === 'link' && source.link.startsWith('storage://')
					? 'storage'
					: 'file';

			const targetLink = await requestSaveTargetLink({
				title: action === 'save-as' ? 'Save Configuration As Link' : 'Save Configuration',
				confirmLabel: action === 'save-as' ? 'Save As' : 'Save',
				defaultKind
			});

			if (!targetLink) {
				return null;
			}

			await saveToLink(targetLink);

			if (action === 'save-as' && source.kind === 'link') {
				source = { kind: 'link', link: targetLink };
			}

			if (!silent) {
				successMessage = `Saved to ${targetLink}`;
			}

			return targetLink;
		} catch (error) {
			errorMessage = toErrorMessage(error, 'Failed to save configuration.');
			return null;
		} finally {
			isSaving = false;
		}
	}

	async function handleSave() {
		await runSave('save');
	}

	async function handleSaveAs() {
		await runSave('save-as');
	}

	async function handleSaveAndDeploy() {
		if (!config) {
			errorMessage = 'No configuration loaded.';
			return;
		}

		const savedLink = await runSave('save', true);
		if (!savedLink || !config) {
			return;
		}

		isDeploying = true;
		try {
			const report = await api.kernelManager.updateConfig({
				mode: 'new_config',
				new_config: config
			});

			if (report.status.status === 'succeeded') {
				resetMessages();
				successMessage = `Saved to ${savedLink} and deployed successfully.`;
				return;
			}

			errorMessage = `Saved to ${savedLink}, but deploy failed at phase: ${report.status.phase}`;
		} catch (error) {
			errorMessage = `Saved to ${savedLink}, but deploy request failed: ${toErrorMessage(error, 'Unknown error')}`;
		} finally {
			isDeploying = false;
		}
	}

	function handleCancel() {
		source = null;
		config = null;
		resetMessages();
	}
</script>

<div class="p-2">
	<h2 class="h2">Edit Config</h2>
	<p class="mt-1 text-sm opacity-70">
		Select a source, edit the configuration, then save or save and deploy.
	</p>

	{#if errorMessage}
		<div class="alert mt-3 preset-tonal-error">{errorMessage}</div>
	{/if}
	{#if successMessage}
		<div class="alert mt-3 preset-tonal-success">{successMessage}</div>
	{/if}

	{#if source !== null}
		<div class="mt-3 flex flex-col justify-between gap-2">
			<!-- top function bar -->
			<div class="flex flex-row justify-between gap-2">
				<!-- left -->
				<div class="flex items-center justify-start gap-2">
					<span class="badge preset-tonal-secondary">{describeSource(source)}</span>
				</div>
				<!-- middle -->
				<div class="flex justify-center gap-2"></div>
				<!-- right -->
				<div class="flex justify-end gap-2">
					<button
						class="btn preset-tonal-primary"
						onclick={handleSave}
						disabled={isSaving || isDeploying}
					>
						{isSaving ? 'Saving...' : 'Save'}
					</button>
					{#if source.kind === 'link'}
						<button
							class="btn preset-tonal-secondary"
							onclick={handleSaveAs}
							disabled={isSaving || isDeploying}
						>
							Save As
						</button>
					{/if}
					<button
						class="preset-filled-error-500 btn"
						onclick={handleSaveAndDeploy}
						disabled={isSaving || isDeploying}
					>
						{isDeploying ? 'Deploying...' : 'Save & Deploy'}
					</button>
					<button
						class="btn preset-tonal-warning"
						onclick={handleCancel}
						disabled={isSaving || isDeploying}
					>
						Cancel
					</button>
				</div>
			</div>
			<!-- editor region -->
			<div class="min-h-140 rounded border border-surface-200-800">
				{#if config}
					<ServiceConfigEditor bind:config />
				{/if}
			</div>
		</div>
	{:else}
		<div class="mt-3 space-y-4 card border border-surface-200-800 p-4">
			<div class="flex flex-row items-center justify-between gap-2">
				<h3 class="h3">Source</h3>
				<button
					class="btn preset-filled-primary-500"
					disabled={isLoadingSource}
					onclick={handleLoadSource}
				>
					{isLoadingSource
						? 'Loading...'
						: selectedSourceKind === 'new'
							? 'Create'
							: selectedSourceKind === 'link'
								? 'Open'
								: 'Resolve'}
				</button>
			</div>
			<SegmentedControl
				value={selectedSourceKind}
				onValueChange={(details) => {
					selectedSourceKind = details.value as SourceKind;
				}}
			>
				<SegmentedControl.Control>
					<SegmentedControl.Indicator />
					<SegmentedControl.Item value="new">
						<SegmentedControl.ItemText>New</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item value="link">
						<SegmentedControl.ItemText>From Link</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item value="resolved">
						<SegmentedControl.ItemText>From Resolver</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
				</SegmentedControl.Control>
			</SegmentedControl>

			{#if selectedSourceKind === 'link'}
				<SegmentedControl
					value={selectedLinkKind}
					onValueChange={(details) => {
						selectedLinkKind = details.value as LinkKind;
					}}
				>
					<SegmentedControl.Control>
						<SegmentedControl.Indicator />
						<SegmentedControl.Item value="file">
							<SegmentedControl.ItemText>File</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
						<SegmentedControl.Item value="storage">
							<SegmentedControl.ItemText>Storage</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
					</SegmentedControl.Control>
				</SegmentedControl>
				<label class="label">
					<span>Link (file:// or storage://)</span>
					<input
						class="input"
						type="text"
						value={sourceLinkInput}
						disabled
						placeholder="file://..."
					/>
					<div class="card border border-surface-900-100 p-4">
						{#if selectedLinkKind === 'file'}
							<FileTree
								onSelectionChange={(value) => {
									if (value) {
										sourceLinkInput = `file://${value}`;
									} else {
										sourceLinkInput = '';
									}
								}}
							></FileTree>
						{:else}
							<ObjectSelector
								selectionMode="single"
								lockedFileds={['data_type']}
								filter={{
									data_type: 'ServiceConfig'
								}}
								onSelectionChange={(value) => {
									if (value[0]) {
										sourceLinkInput = `storage://${value[0].descriptor.id}#${value[0].descriptor.revision}`;
									} else {
										sourceLinkInput = '';
									}
								}}
								showDelete={false}
								showEdit={false}
							></ObjectSelector>
						{/if}
					</div>
				</label>
			{:else if selectedSourceKind === 'resolved'}
				<div class="space-y-3">
					<label class="label">
						<span>Resolver Type</span>
						<select
							class="select"
							bind:value={resolvedKind}
							onchange={(event) => {
								if (
									!k8sCapability.available &&
									(event.currentTarget as HTMLSelectElement).value === 'k8s'
								) {
									resolvedKind = 'fs';
								}
							}}
						>
							<option value="fs">Filesystem</option>
							<option value="k8s" disabled={!k8sCapability.available}>Kubernetes</option>
						</select>
					</label>
					{#if !k8sCapability.loading && !k8sCapability.available}
						<div class="text-xs opacity-70">
							Kubernetes resolver is unavailable in current environment.
						</div>
					{/if}
					{#if k8sCapability.error}
						<div class="alert preset-tonal-error">{k8sCapability.error}</div>
					{/if}

					{#if resolvedKind === 'fs'}
						<label class="label">
							<span>Filesystem Path</span>
							<input
								class="input"
								type="text"
								bind:value={resolvedFsPath}
								placeholder="/path/to/config.yaml"
							/>
							<div class="card border border-surface-900-100 p-4">
								<FileTree bind:selectedFilePath={resolvedFsPath}></FileTree>
							</div>
						</label>
					{:else}
						<label class="label">
							<span>Kubernetes Namespace</span>
							<input
								class="input"
								type="text"
								bind:value={resolvedNamespace}
								placeholder="default"
							/>
						</label>
					{/if}
				</div>
			{/if}

		</div>
	{/if}
</div>

{#snippet saveTargetDialogMessage()}
	<div class="space-y-3">
		<label class="label">
			<span>Save Target Type</span>
			<select class="select" bind:value={saveTargetKind}>
				<option value="file">file://</option>
				<option value="storage">storage://</option>
			</select>
		</label>

		{#if saveTargetKind === 'file'}
			<label class="label">
				<span>File Path</span>
				<input
					class="input"
					type="text"
					bind:value={saveTargetInput}
					placeholder="/path/to/config.yaml"
				/>
			</label>
		{:else}
			<label class="label">
				<span>Storage ID</span>
				<input class="input" type="text" bind:value={saveTargetInput} placeholder="my-config" />
				<p class="mt-1 text-xs opacity-70">Revision is not required for Save As.</p>
			</label>
		{/if}
	</div>
{/snippet}
