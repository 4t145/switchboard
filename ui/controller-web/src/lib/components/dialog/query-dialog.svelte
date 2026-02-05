<script lang="ts" generics="Options extends string">
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import type { DialogOption, DialogQuery } from '.';
	import { XIcon } from '@lucide/svelte';

	type Props = DialogQuery<Options>;
	let { title, message, options, role }: Props = $props();
	let open = $state(true);
	type Resolve = (value: Options | null) => void;
	let pendingResolves = $state<Resolve[]>([]);
	let resolvedValue: Options | null | undefined = $state(undefined);
	function resolveInternal(value: Options | null) {
		resolvedValue = value;
		open = false;
		pendingResolves.forEach((resolve) => resolve(value));
	}
	function selectOption(value: Options) {
		resolveInternal(value);
	}
	export function closeDialogWithoutSelection() {
		resolveInternal(null);
	}
	export function awaitSelection(): Promise<Options | null> {
		return new Promise<Options | null>((resolve) => {
			if (resolvedValue !== undefined) {
				resolve(resolvedValue);
			} else {
				pendingResolves.push(resolve);
			}
		});
	}
</script>

<Dialog
	{open}
	onOpenChange={({ open: isOpen }) => {
		open = isOpen;
	}}
	{role}
>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content class="w-full max-w-xl space-y-4 card bg-surface-100-900 p-4 shadow-xl">
				<header class="flex items-center justify-between">
					<Dialog.Title class="text-lg font-bold">{title}</Dialog.Title>
					<Dialog.CloseTrigger class="btn-icon hover:preset-tonal">
						<XIcon class="size-4" />
					</Dialog.CloseTrigger>
				</header>
				<Dialog.Description>
					{#if typeof message === 'string'}
						<p>{message}</p>
					{:else}
						{@render message()}
					{/if}
				</Dialog.Description>
				<footer class="flex justify-end gap-2">
					{#each Object.entries(options as Record<string, DialogOption>) as [value, option]}
						<button
							class={option.class}
							onclick={() => {
								selectOption(value as Options);
							}}
						>
							{#if option.icon}
								{@const Icon = option.icon}
								<Icon class="mr-2 size-4" />
							{/if}
							{option.label ?? value}
						</button>
					{/each}
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
