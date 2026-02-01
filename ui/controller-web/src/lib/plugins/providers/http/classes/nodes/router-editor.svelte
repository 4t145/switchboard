<script lang="ts">
	import type { HttpClassEditorProps, HttpNodeClassPlugin } from '$lib/plugins/types';
	export type PathTree = {
		[pattern: string]: string;
	}
	export type RuleBucket = {
		rules: string[][],
		target: string;
	} | string

	export type RouterConfig = {
		output: {
			[port: string]: {
				target: string;
				filters: string[];
			};
		},
		hostname: {
			[pattern: string]: string;
		},
	}

	type Props = HttpClassEditorProps<RouterConfig>;

	let { value = $bindable(), readonly = false }: Props = $props();
</script>

<div class="space-y-6">

	<!-- Output Definitions -->
	<div class="space-y-2">

	</div>
	
</div>
