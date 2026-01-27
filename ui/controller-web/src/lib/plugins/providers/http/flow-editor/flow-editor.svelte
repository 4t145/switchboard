<script lang="ts">
	import type { FlowConfig } from '../types';

	type Props = {
		value: FlowConfig;
		readonly?: boolean;
	};
	let { value = $bindable(), readonly = false }: Props = $props();
</script>
