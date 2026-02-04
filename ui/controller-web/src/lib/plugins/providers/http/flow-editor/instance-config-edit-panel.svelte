<script lang="ts">
	import { Dialog, FloatingPanel, Portal } from '@skeletonlabs/skeleton-svelte';
	import {
		GripVerticalIcon,
		MaximizeIcon,
		MinimizeIcon,
		MinusIcon,
		SaveIcon,
		TrashIcon,
		XIcon
	} from '@lucide/svelte';

	import InstanceConfigEditor from './instance-config-editor.svelte';
	import type { HttpEditorContext, InstanceDataWithoutType } from '../types';
	import { dialogQuery } from '$lib/components/dialog';
	import type { Component } from 'svelte';
	let { value, onValueSave, instanceType, instanceId, httpEditorContext } = $props<{
		value: InstanceDataWithoutType;
		onValueSave: (value: InstanceDataWithoutType) => void;
		instanceType: 'node' | 'filter';
		instanceId: string;
		httpEditorContext: HttpEditorContext;
	}>();
	export function closeWithoutSaving() {
		editDialogOpen = false;
	}
	export function saveAndClose() {
		onValueSave(value);
		editDialogOpen = false;
	}
	export function focus() {
		open();
	}
	let editDialogOpen = $state(true);
	function open() {
		editDialogOpen = true;
	}
	type ConfirmDiscardOptions = 'close-without-saving' | 'cancel' | 'save-and-close';
	async function confirmSavingBeforeClose(): Promise<ConfirmDiscardOptions> {
		return (
			(await dialogQuery<ConfirmDiscardOptions>({
				title: 'Discard Changes?',
				message: 'You have unsaved changes. Are you sure you want to close without saving?',
				options: {
					'close-without-saving': {
						label: 'Discard',
						class: 'btn preset-tonal-warning',
						icon: TrashIcon
					},
					cancel: {
						label: 'Cancel',
						class: 'btn preset-tonal-secondary',
						icon: XIcon
					},
					'save-and-close': {
						label: 'Save',
						class: 'btn preset-tonal-primary',
						icon: SaveIcon
					}
				},
				role: 'alertdialog'
			})) ?? 'cancel'
		);
	}
	function onOpenChange(details: { open: boolean }) {
		if (!details.open) {
			confirmSavingBeforeClose().then((option) => {
				if (option === 'save-and-close') {
					saveAndClose();
				} else if (option === 'close-without-saving') {
					closeWithoutSaving();
				} else {
					// keep open
					open();
				}
			});
		} else {
			open();
		}
	}
	let changed = $derived.by(() => {
		return false;
	});
</script>

<FloatingPanel
	open={editDialogOpen}
	{onOpenChange}
	minSize={{ width: 400, height: 300 }}
	defaultSize={{ width: window.innerWidth * 0.7, height: window.innerHeight * 0.7 }}
>
	<Portal>
		<FloatingPanel.Positioner class="z-50">
			<FloatingPanel.Content>
				<FloatingPanel.DragTrigger>
					<FloatingPanel.Header>
						<FloatingPanel.Title>
							<GripVerticalIcon class="size-4" />
							editing {instanceType} "{instanceId}" {#if changed}*{/if}
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
								aria-label="Close panel"
								onclick={(e) => {
									e.preventDefault();
									onOpenChange({ open: false });
								}}
							>
								<XIcon class="size-4" />
							</button>
						</FloatingPanel.Control>
					</FloatingPanel.Header>
				</FloatingPanel.DragTrigger>
				<FloatingPanel.Body class="flex flex-col justify-between bg-surface-100-900">
					<content class="h-full w-full overflow-auto">
						<InstanceConfigEditor
							{instanceId}
							bind:config={value}
							{instanceType}
							{httpEditorContext}
						></InstanceConfigEditor>
					</content>
					<footer class="mt-4 flex justify-end gap-2 pb-[48px]">
						<button class="btn preset-tonal-secondary" onclick={closeWithoutSaving}>
							Cancel
						</button>
						<button class="btn preset-tonal-primary" onclick={saveAndClose}> Save </button>
					</footer>
				</FloatingPanel.Body>
				<FloatingPanel.ResizeTrigger axis="se" />
			</FloatingPanel.Content>
		</FloatingPanel.Positioner>
	</Portal>
</FloatingPanel>
