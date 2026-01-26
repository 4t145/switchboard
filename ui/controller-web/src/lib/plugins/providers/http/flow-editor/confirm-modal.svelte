<script lang="ts">
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import { AlertTriangle, X } from 'lucide-svelte';

	type Props = {
		open: boolean;
		title: string;
		message: string;
		confirmLabel?: string;
		cancelLabel?: string;
		type?: 'danger' | 'warning' | 'info';
		onConfirm: () => void;
		onCancel: () => void;
	};

	let {
		open,
		title,
		message,
		confirmLabel = 'Confirm',
		cancelLabel = 'Cancel',
		type = 'danger',
		onConfirm,
		onCancel
	}: Props = $props();

	let internalOpen = $state(open);
	let contentRef: HTMLDivElement;

	$effect(() => {
		internalOpen = open;
	});

	function handleOpenChange(details: { open: boolean }) {
		if (!details.open) {
			onCancel();
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			onConfirm();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			onCancel();
		}
	}

	$effect(() => {
		if (contentRef) {
			contentRef.focus();
		}
	});

	const typeStyles = {
		danger: {
			iconColor: 'text-error-500',
			iconBg: 'preset-tonal-error',
			buttonClass: 'preset-filled-error'
		},
		warning: {
			iconColor: 'text-warning-500',
			iconBg: 'preset-tonal-warning',
			buttonClass: 'preset-filled-warning'
		},
		info: {
			iconColor: 'text-info-500',
			iconBg: 'preset-tonal-info',
			buttonClass: 'preset-filled-primary'
		}
	};
</script>

<Dialog open={internalOpen} onOpenChange={handleOpenChange} role="alertdialog">
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content class="bg-surface-100-900 shadow-2xl max-w-md w-full">
				<div
					bind:this={contentRef}
					tabindex="-1"
					role="dialog"
					onkeydown={handleKeyDown}
					class="outline-none"
				>
					<header class="flex items-start justify-between gap-4 p-6">
						<div class="flex items-center gap-3">
							<div class="rounded-full p-3 {typeStyles[type].iconBg}">
								<AlertTriangle class="{typeStyles[type].iconColor} size-5" />
							</div>
							<div>
								<h3 class="text-lg font-semibold">{title}</h3>
								<p class="text-sm text-surface-500-400 mt-1">{message}</p>
							</div>
						</div>
						<Dialog.CloseTrigger class="btn-icon preset-tonal">
							<X class="size-5" />
						</Dialog.CloseTrigger>
					</header>

					<footer class="flex justify-end gap-2 p-6 pt-0">
						<button class="btn preset-tonal" onclick={onCancel}>
							{cancelLabel}
						</button>
						<button class="btn {typeStyles[type].buttonClass}" onclick={onConfirm}>
							{confirmLabel}
						</button>
					</footer>
				</div>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
