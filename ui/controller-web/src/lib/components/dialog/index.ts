import { type IconProps } from '@lucide/svelte';
import { mount, unmount, type Component, type Snippet } from 'svelte';
import QueryDialogComponent from './query-dialog.svelte';

export type DialogOption = {
	label?: string;
	class?: string;
	icon?: Component<IconProps>;
};
export type DialogQuery<Options extends string> = {
	title: string;
	message: string | Snippet<[]>;
	options: Record<Options, DialogOption>;
	role?: 'dialog' | 'alertdialog';
};

export type QueryRenderOptions = {
	target: HTMLElement | Document | ShadowRoot;
};

export async function dialogQuery<Options extends string>(
	query: DialogQuery<Options>,
	renderOptions?: QueryRenderOptions
): Promise<Options | null> {
	const dialog = createDialogQuery(query, renderOptions);
	const result = await dialog.awaitSelection();
	unmount(dialog, {
		outro: true
	});
	return result;
}

export function createDialogQuery<Options extends string>(
	query: DialogQuery<Options>,
	renderOptions?: QueryRenderOptions
): QueryDialogComponent<Options> {
	return mount(QueryDialogComponent<Options>, {
		target: renderOptions?.target || document.body,
		props: query,
        intro: true
	}) as QueryDialogComponent<Options>;
}
