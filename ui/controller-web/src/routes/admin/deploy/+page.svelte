<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		CloudUploadIcon,
		EyeIcon,
		GripVerticalIcon,
		MaximizeIcon,
		MinimizeIcon,
		MinusIcon,
		XIcon
	} from '@lucide/svelte';
	import { FloatingPanel, Portal, SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import { api } from '$lib/api/routes';
	import type { K8sEnvResponse, K8sNamespacesResponse, HumanReadableServiceConfig } from '$lib/api/types';
	import type { UpdateConfigRequest } from '$lib/api/routes/kernel_manager';
	import FileTree from '$lib/components/file-tree.svelte';
	import ServiceConfigEditor from '$lib/components/editor/service-config-editor.svelte';

	type DeploySource = 'file' | 'storage' | 'k8s';

	const FILE: DeploySource = 'file';
	const STORAGE: DeploySource = 'storage';
	const K8S: DeploySource = 'k8s';

	let deploySource: DeploySource = $state('file');
	let selectedFilePath = $state<string | undefined>(undefined);
	let k8sEnv = $state<K8sEnvResponse | undefined>(undefined);
	let k8sNamespaces = $state<string[]>([]);
	let selectedK8sNamespace = $state<string | undefined>(undefined);
	let k8sEnvLoading = $state(false);
	let k8sNamespacesLoading = $state(false);
	let k8sErrorMessage = $state<string | undefined>(undefined);

	let deployLoading = $state(false);
	let deployErrorMessage = $state<string | undefined>(undefined);

	let previewLoading = $state(false);
	let previewErrorMessage = $state<string | undefined>(undefined);
	let previewPanelOpen = $state(false);
	let previewConfig = $state<HumanReadableServiceConfig | undefined>(undefined);

	onMount(async () => {
		await loadK8sEnv();
	});

	$effect(() => {
		if (deploySource !== K8S) return;
		if (!k8sEnvLoading && !k8sEnv) {
			void loadK8sEnv();
			return;
		}
		if (k8sEnv?.in_cluster && !k8sNamespacesLoading && k8sNamespaces.length === 0) {
			void loadK8sNamespaces();
		}
	});

	let selected = $derived.by(() => {
		if (selectedFilePath && deploySource === FILE) {
			return `file://${selectedFilePath}`;
		}
		if (selectedK8sNamespace && deploySource === K8S) {
			return `k8s://namespace/${selectedK8sNamespace}`;
		}
		return undefined;
	});

	async function loadK8sEnv() {
		k8sEnvLoading = true;
		k8sErrorMessage = undefined;
		try {
			k8sEnv = await api.k8s.getEnv();
			if (!k8sEnv.in_cluster) {
				k8sNamespaces = [];
				selectedK8sNamespace = undefined;
			}
		} catch (error) {
			console.error('Failed to load kubernetes environment', error);
			k8sErrorMessage = 'Failed to load kubernetes environment.';
		} finally {
			k8sEnvLoading = false;
		}
	}

	async function loadK8sNamespaces() {
		if (!k8sEnv?.in_cluster) return;
		k8sNamespacesLoading = true;
		k8sErrorMessage = undefined;
		try {
			const data: K8sNamespacesResponse = await api.k8s.getNamespaces();
			k8sNamespaces = data.namespaces;
			selectedK8sNamespace =
				k8sEnv.current_namespace && data.namespaces.includes(k8sEnv.current_namespace)
					? k8sEnv.current_namespace
					: data.namespaces.at(0);
		} catch (error) {
			console.error('Failed to load kubernetes namespaces', error);
			k8sErrorMessage = 'Failed to load kubernetes namespaces.';
		} finally {
			k8sNamespacesLoading = false;
		}
	}

	async function buildResolveRequest(): Promise<UpdateConfigRequest | undefined> {
		if (deploySource === FILE && selectedFilePath) {
			return {
				mode: 'resolve',
				resolver: 'fs',
				config: { path: selectedFilePath }
			};
		}
		if (deploySource === K8S && selectedK8sNamespace) {
			return {
				mode: 'resolve',
				resolver: 'k8s',
				config: { gateway_namespace: selectedK8sNamespace }
			};
		}
		return undefined;
	}

	async function handlePreview() {
		previewErrorMessage = undefined;
		const request = await buildResolveRequest();
		if (!request || request.mode !== 'resolve') {
			previewErrorMessage = 'No valid preview source selected.';
			return;
		}
		previewLoading = true;
		try {
			const result = await api.resolve.service_config({
				resolver: request.resolver,
				config: request.config
			});
			previewConfig = result.config;
			previewPanelOpen = true;
		} catch (error) {
			console.error('Preview failed', error);
			previewErrorMessage = 'Preview failed due to server error.';
		} finally {
			previewLoading = false;
		}
	}

	async function handleDeploy(event: SubmitEvent) {
		event.preventDefault();
		deployErrorMessage = undefined;
		const request = await buildResolveRequest();
		if (!request) {
			deployErrorMessage = 'No valid deploy source selected.';
			return;
		}
		deployLoading = true;
		try {
			const report = await api.kernelManager.updateConfig(request);
			if (report.status.status === 'succeeded') {
				await goto(`/admin/dashboard?deployed=1&tx=${encodeURIComponent(report.transaction_id)}`);
			} else {
				deployErrorMessage = `Deploy failed at phase: ${report.status.phase}`;
			}
		} catch (error) {
			console.error('Deploy failed', error);
			deployErrorMessage = 'Deploy failed due to server error.';
		} finally {
			deployLoading = false;
		}
	}
</script>

<form class="flex h-full flex-col gap-4 p-4" id="deploy-form" onsubmit={handleDeploy}>
	<div class="flex items-center justify-between">
		<h2 class="h2">Deploy</h2>
		<div class="flex flex-row justify-end gap-2">
			{#if selected}
				<button
					type="button"
					class="btn preset-tonal-secondary"
					disabled={deployLoading || previewLoading}
					onclick={handlePreview}
				>
					{#if previewLoading}
						Previewing...
					{:else}
						Preview <EyeIcon />
					{/if}
				</button>
				<button type="submit" class="btn preset-tonal-primary" disabled={deployLoading || previewLoading}>
					{#if deployLoading}
						Deploying...
					{:else}
						Deploy <CloudUploadIcon />
					{/if}
				</button>
			{/if}
		</div>
	</div>

	{#if previewErrorMessage}
		<div class="alert preset-tonal-error">{previewErrorMessage}</div>
	{/if}
	{#if deployErrorMessage}
		<div class="alert preset-tonal-error">{deployErrorMessage}</div>
	{/if}

	<div class="flex flex-col gap-4">
		<div>
			<h3 class="h3">Config Source</h3>
			<SegmentedControl value={deploySource} onValueChange={(details) => (deploySource = details.value as DeploySource)}>
				<SegmentedControl.Control>
					<SegmentedControl.Indicator />
					<SegmentedControl.Item value={FILE}>
						<SegmentedControl.ItemText>From File</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item value={STORAGE}>
						<SegmentedControl.ItemText>From Storage</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
					<SegmentedControl.Item value={K8S}>
						<SegmentedControl.ItemText>From Kubernetes</SegmentedControl.ItemText>
						<SegmentedControl.ItemHiddenInput />
					</SegmentedControl.Item>
				</SegmentedControl.Control>
			</SegmentedControl>
		</div>

		<div>
			{#if deploySource === FILE}
				<FileTree bind:selectedFilePath />
			{:else if deploySource === STORAGE}
				<div class="text-sm opacity-70">Storage source is not implemented yet.</div>
			{:else if deploySource === K8S}
				<div class="space-y-2">
					{#if k8sErrorMessage}
						<div class="alert preset-tonal-error">{k8sErrorMessage}</div>
					{/if}
					{#if k8sEnvLoading}
						<div class="text-sm opacity-70">Loading kubernetes environment...</div>
					{:else if k8sEnv?.in_cluster}
						<label class="label" for="k8s-namespace-select">
							<span>Namespace</span>
						</label>
						<select
							id="k8s-namespace-select"
							class="select"
							bind:value={selectedK8sNamespace}
							disabled={k8sNamespacesLoading || k8sNamespaces.length === 0}
						>
							{#each k8sNamespaces as namespace (namespace)}
								<option value={namespace}>{namespace}</option>
							{/each}
						</select>
						<div class="text-xs opacity-70">
							Runtime namespace: {k8sEnv.current_namespace ?? 'unknown'}
						</div>
					{:else}
						<div class="text-sm opacity-70">
							Controller is not running in kubernetes. Namespace list is unavailable.
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</form>

<FloatingPanel
	open={previewPanelOpen}
	onOpenChange={(details) => (previewPanelOpen = details.open)}
	minSize={{ width: 900, height: 560 }}
	defaultSize={{ width: window.innerWidth * 0.85, height: window.innerHeight * 0.82 }}
>
	<Portal>
		<FloatingPanel.Positioner class="z-50">
			<FloatingPanel.Content>
				<FloatingPanel.DragTrigger>
					<FloatingPanel.Header>
						<FloatingPanel.Title>
							<GripVerticalIcon class="size-4" />
							Deploy Preview
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
							<button
								type="button"
								class="btn-icon hover:preset-tonal"
								onclick={() => {
									previewPanelOpen = false;
								}}
							>
								<XIcon class="size-4" />
							</button>
						</FloatingPanel.Control>
					</FloatingPanel.Header>
				</FloatingPanel.DragTrigger>
				<FloatingPanel.Body class="overflow-auto  bg-surface-50-950">
					{#if previewConfig}
						<ServiceConfigEditor config={previewConfig} readonly={true} />
					{:else}
						<div class="p-4 text-sm opacity-70">No preview data loaded.</div>
					{/if}
				</FloatingPanel.Body>
				<FloatingPanel.ResizeTrigger axis="se" />
			</FloatingPanel.Content>
		</FloatingPanel.Positioner>
	</Portal>
</FloatingPanel>
